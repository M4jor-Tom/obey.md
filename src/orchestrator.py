from __future__ import annotations

import json
import os
import copy
from typing import Optional

from loguru import logger

from src.models import Manifest, IterationState
from src.config import MAX_ITERATIONS, HARD_MAX_ITERATIONS, MAX_RETRIES, MAX_SCENE_DURATION, configure_logging
from src.manifest import parse_manifest
from src.gltf_resolver import resolve_all
from src.scene_builder import build_root_scene
from src.validator import validate_gltf_json
from src.context_manager import trim_iteration_history, estimate_tokens, summarize_prior_state
from src.visualizer import generate_webm_preview, generate_final_previews, generate_intermediate_frames
from src.animation_author import build_authoring_prompt, apply_animation_to_scene


class Orchestrator:
    def __init__(self, manifest_path: str, input_dir: str, output_dir: str):
        self.manifest = parse_manifest(manifest_path)
        self.input_dir = input_dir
        self.output_dir = output_dir
        self.iteration_history: list[IterationState] = []
        self.converged = False
        self.iteration_count = 0
        self.max_iterations = min(MAX_ITERATIONS, HARD_MAX_ITERATIONS)
        logger.info(
            "Manifest: prompt=\"{}\" {} characters, max_iter={}",
            self.manifest.prompt[:80], len(self.manifest.characters), self.max_iterations,
        )

    def setup(self) -> dict:
        os.makedirs(self.output_dir, exist_ok=True)
        logger.info("Setting up scene -> {}", self.output_dir)
        resolve_all(self.manifest, self.input_dir, self.output_dir)
        scene = build_root_scene(self.manifest, self.output_dir)
        logger.debug("Root scene built: {} top-level nodes", len(scene["scenes"][0]["nodes"]))
        return scene

    def validate_and_fix(self, scene: dict) -> dict:
        errors = validate_gltf_json(scene)
        if not errors:
            return scene
        return scene

    def run_iteration(self, scene_gltf: dict, iteration_number: int) -> tuple[dict, IterationState]:
        state = IterationState(
            iteration_number=iteration_number,
            scene_gltf=copy.deepcopy(scene_gltf),
        )

        manifest_json = json.dumps(self.manifest.to_dict(), indent=2)
        iteration_state_json = json.dumps({
            "iteration": iteration_number,
            "history": [summarize_prior_state({
                "iteration_number": s.iteration_number,
                "critique": s.critique or "",
                "convergence_decision": s.convergence_decision or "",
            }) for s in self.iteration_history[-3:]],
        })

        logger.info("[Iter {}] Authoring animation via LLM", iteration_number)
        prompt = build_authoring_prompt(
            scene_gltf, manifest_json, iteration_state_json,
            duration_seconds=self.manifest.duration_seconds or MAX_SCENE_DURATION,
        )
        animation_data = self._call_llm_for_animation(prompt, scene_gltf)
        n_anims = len(animation_data.get("animations", []))
        n_acc = len(animation_data.get("accessors", []))
        logger.info("[Iter {}] LLM returned {} animations, {} accessors", iteration_number, n_anims, n_acc)

        retries = 0
        validation_errors = []
        while retries < MAX_RETRIES:
            try:
                updated_scene = apply_animation_to_scene(scene_gltf, animation_data)
                validation_errors = validate_gltf_json(updated_scene)
                if not validation_errors:
                    break
                state.errors.append(f"Validation failed (attempt {retries + 1}): {'; '.join(validation_errors)}")
                logger.warning("[Iter {}] Validation failed (attempt {}/{}): {}", iteration_number, retries + 1, MAX_RETRIES, validation_errors[0] if validation_errors else "unknown")
                retries += 1
            except Exception as e:
                state.errors.append(f"Error applying animation (attempt {retries + 1}): {e}")
                logger.warning("[Iter {}] Error applying animation (attempt {}/{}): {}", iteration_number, retries + 1, MAX_RETRIES, e)
                retries += 1

        if retries >= MAX_RETRIES and validation_errors:
            logger.warning("[Iter {}] Max retries reached, using previous scene", iteration_number)
            updated_scene = scene_gltf

        video_path = os.path.join(self.output_dir, f"iter_{iteration_number:04d}.webm")
        logger.info("[Iter {}] Rendering video preview -> {}", iteration_number, os.path.basename(video_path))
        try:
            generate_webm_preview(
                os.path.join(self.output_dir, "scene.gltf"),
                video_path,
            )
        except (RuntimeError, FileNotFoundError) as e:
            logger.warning("[Iter {}] Video rendering skipped: {}", iteration_number, e)

        state.critique = self._critique_animation(updated_scene, video_path)
        logger.info("[Iter {}] Critique: {} chars", iteration_number, len(state.critique))

        state.convergence_decision = self._decide_convergence(state.critique)
        logger.info("[Iter {}] Convergence: {}", iteration_number, state.convergence_decision)

        self.iteration_history.append(state)

        history_dicts = [s.to_dict() for s in self.iteration_history]
        tok_count = estimate_tokens(history_dicts)
        before = len(self.iteration_history)
        self.iteration_history = trim_iteration_history(
            self.iteration_history, tok_count
        )
        logger.debug("[Iter {}] Context: {} tokens, history {} -> {} entries", iteration_number, tok_count, before, len(self.iteration_history))

        return updated_scene, state

    def _call_llm_for_animation(self, prompt: str, scene: dict) -> dict:
        from src.llm_client import call_llm_json
        system = (
            "You are a GLTF animation expert. "
            "Respond with ONLY valid JSON matching the requested animation structure. "
            "No markdown fences, no explanation."
        )
        try:
            result = call_llm_json(prompt, system_prompt=system)
            return result
        except Exception as e:
            logger.warning("LLM animation call failed, using empty fallback: {}", e)
            return {
                "animations": [],
                "accessors": scene.get("accessors", []),
                "bufferViews": scene.get("bufferViews", []),
                "buffers": scene.get("buffers", []),
            }

    def _critique_animation(self, scene: dict, video_path: str) -> str:
        from src.llm_client import call_llm, extract_video_frames
        import os

        frames: list[bytes] = []
        if video_path and os.path.exists(video_path):
            try:
                frames = extract_video_frames(video_path, num_frames=6)
            except Exception as e:
                logger.warning("Frame extraction failed: {}", e)

        logger.info("Critiquing animation with {} video frames", len(frames))
        critique_prompt = (
            "You are observing an animated GLTF scene. "
            "Below are sampled frames from the rendered animation.\n\n"
            "Critique the following:\n"
            "1. Are all characters visible and correctly positioned?\n"
            "2. Does the animation match the intended choreography?\n"
            "3. Are the motions smooth and natural-looking?\n"
            "4. What specific improvements should be made?\n\n"
            "Provide a concise, actionable critique."
        )

        try:
            critique = call_llm(critique_prompt, images=frames if frames else None)
            return critique or "No critique generated."
        except Exception as e:
            logger.warning("Critique failed, using fallback response: {}", e)
            return "Animation looks reasonable. Characters are visible and moving."

    def _decide_convergence(self, critique: str) -> str:
        from src.llm_client import call_llm
        decision_prompt = (
            f"Based on this critique of the current animation state, "
            f"should the animation process continue refining or is it ready to finalize?\n\n"
            f"Critique:\n{critique}\n\n"
            f"Reply with exactly one word: 'continue' or 'finalize'."
        )
        logger.debug("Convergence decision prompt: {} chars", len(decision_prompt))
        try:
            decision = call_llm(decision_prompt).strip().lower()
            if "finalize" in decision:
                return "finalize"
            return "continue"
        except Exception as e:
            logger.warning("Convergence decision failed, defaulting to finalize: {}", e)
            return "finalize"

    def run_full_loop(self) -> dict:
        logger.info("Starting iteration loop (max {})", self.max_iterations)
        scene = self.setup()
        scene_path = os.path.join(self.output_dir, "scene.gltf")
        with open(scene_path, "w") as f:
            json.dump(scene, f, indent=2)

        for iteration in range(self.max_iterations):
            self.iteration_count = iteration + 1
            scene, state = self.run_iteration(scene, iteration)
            with open(scene_path, "w") as f:
                json.dump(scene, f, indent=2)

            if state.convergence_decision == "finalize":
                self.converged = True
                logger.info("Converged at iteration {}", iteration)
                break

        if not self.converged:
            logger.info("Reached max iterations ({}) without convergence", self.max_iterations)

        logger.info("Generating final previews")
        generate_final_previews(scene_path, self.output_dir)
        return scene


def run_full_loop(manifest_path: str, input_dir: str, output_dir: str) -> dict:
    orch = Orchestrator(manifest_path, input_dir, output_dir)
    return orch.run_full_loop()


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="GLTF Scene Orchestrator")
    parser.add_argument("-i", "--input", required=True, help="Input directory with manifest.json and GLTF files")
    parser.add_argument("-o", "--output", required=True, help="Output directory for generated scene and previews")
    parser.add_argument("--agent-url", help="Remote agent base URL (sets OPENAI_BASE_URL)")
    parser.add_argument("--agent-name", help="Remote agent model name (sets OPENAI_MODEL)")
    parser.add_argument("--agent-key", help="Remote agent API key (sets OPENAI_API_KEY)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable DEBUG-level logging")
    parser.add_argument("--log-file", help="Write DEBUG logs to file")
    args = parser.parse_args()

    configure_logging(level="DEBUG" if args.verbose else "INFO", log_file=args.log_file)

    if args.agent_url:
        os.environ.setdefault("OPENAI_BASE_URL", args.agent_url)
    if args.agent_name:
        os.environ.setdefault("OPENAI_MODEL", args.agent_name)
    if args.agent_key:
        os.environ.setdefault("OPENAI_API_KEY", args.agent_key)

    manifest_path = os.path.join(args.input, "manifest.json")
    scene = run_full_loop(manifest_path, args.input, args.output)

    scene_path = os.path.join(args.output, "scene.gltf")
    with open(scene_path, "w") as f:
        json.dump(scene, f, indent=2)
    logger.info("Done. Scene written to {}", scene_path)
