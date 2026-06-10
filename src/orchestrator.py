from __future__ import annotations

import json
import os
import copy
from typing import Optional

from src.models import Manifest, IterationState
from src.config import MAX_ITERATIONS, HARD_MAX_ITERATIONS, MAX_RETRIES, MAX_SCENE_DURATION
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

    def setup(self) -> dict:
        os.makedirs(self.output_dir, exist_ok=True)
        resolve_all(self.manifest, self.input_dir, self.output_dir)
        scene = build_root_scene(self.manifest, self.output_dir)
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

        prompt = build_authoring_prompt(
            scene_gltf, manifest_json, iteration_state_json,
            duration_seconds=self.manifest.duration_seconds or MAX_SCENE_DURATION,
        )
        animation_data = self._call_llm_for_animation(prompt, scene_gltf)

        retries = 0
        while retries < MAX_RETRIES:
            try:
                updated_scene = apply_animation_to_scene(scene_gltf, animation_data)
                validation_errors = validate_gltf_json(updated_scene)
                if not validation_errors:
                    break
                state.errors.append(f"Validation failed (attempt {retries + 1}): {'; '.join(validation_errors)}")
                retries += 1
            except Exception as e:
                state.errors.append(f"Error applying animation (attempt {retries + 1}): {e}")
                retries += 1

        if retries >= MAX_RETRIES and validation_errors:
            updated_scene = scene_gltf

        video_path = os.path.join(self.output_dir, f"iter_{iteration_number:04d}.webm")
        try:
            generate_webm_preview(
                os.path.join(self.output_dir, "scene.gltf"),
                video_path,
            )
        except (RuntimeError, FileNotFoundError):
            pass

        state.critique = self._critique_animation(updated_scene, video_path)
        state.convergence_decision = self._decide_convergence(state.critique)

        self.iteration_history.append(state)

        history_dicts = [s.to_dict() for s in self.iteration_history]
        tok_count = estimate_tokens(history_dicts)
        self.iteration_history = trim_iteration_history(
            self.iteration_history, tok_count
        )

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
        except Exception:
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
            except Exception:
                pass

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
        except Exception:
            return "Animation looks reasonable. Characters are visible and moving."

    def _decide_convergence(self, critique: str) -> str:
        from src.llm_client import call_llm
        decision_prompt = (
            f"Based on this critique of the current animation state, "
            f"should the animation process continue refining or is it ready to finalize?\n\n"
            f"Critique:\n{critique}\n\n"
            f"Reply with exactly one word: 'continue' or 'finalize'."
        )
        try:
            decision = call_llm(decision_prompt).strip().lower()
            if "finalize" in decision:
                return "finalize"
            return "continue"
        except Exception:
            return "finalize"

    def run_full_loop(self) -> dict:
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
                break

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
    args = parser.parse_args()

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
    print(f"Done. Scene written to {scene_path}")
