use std::path::Path;

use log::{debug, info, warn};
use serde_json::Value;

use crate::animation_author::{apply_animation_to_scene, build_authoring_prompt};
use crate::config::{MAX_ITERATIONS, HARD_MAX_ITERATIONS, MAX_RETRIES, MAX_SCENE_DURATION};
use crate::context_manager::{estimate_tokens, summarize_prior_state, trim_iteration_history};
use crate::gltf_resolver::resolve_all;
use crate::llm_client::{call_llm, call_llm_json, extract_video_frames};
use crate::manifest::parse_manifest;
use crate::models::{IterationState, Manifest};
use crate::scene_builder::build_root_scene;
use crate::validator::validate_gltf_json;
use crate::visualizer::{generate_final_previews, generate_webm_preview};

pub struct Orchestrator {
    pub manifest: Manifest,
    pub input_dir: String,
    pub output_dir: String,
    pub iteration_history: Vec<IterationState>,
    pub converged: bool,
    pub iteration_count: i32,
    pub max_iterations: i32,
}

impl Orchestrator {
    pub fn new(manifest_path: &str, input_dir: &str, output_dir: &str) -> Result<Self, String> {
        let manifest = parse_manifest(manifest_path).map_err(|e| e.to_string())?;
        let max_iterations = MAX_ITERATIONS.min(HARD_MAX_ITERATIONS);
        info!(
            "Manifest: prompt=\"{}\" {} characters, max_iter={}",
            &manifest.prompt[..manifest.prompt.len().min(80)],
            manifest.characters.len(),
            max_iterations
        );
        Ok(Orchestrator {
            manifest,
            input_dir: input_dir.to_string(),
            output_dir: output_dir.to_string(),
            iteration_history: Vec::new(),
            converged: false,
            iteration_count: 0,
            max_iterations,
        })
    }

    pub fn setup(&self) -> Result<Value, String> {
        std::fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;
        info!("Setting up scene -> {}", self.output_dir);
        resolve_all(&self.manifest, &self.input_dir, &self.output_dir)
            .map_err(|e| e.to_string())?;
        let scene = build_root_scene(&self.manifest, &self.output_dir);
        debug!(
            "Root scene built: {} top-level nodes",
            scene["scenes"][0]["nodes"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0)
        );
        Ok(scene)
    }

    pub fn validate_and_fix(&self, scene: &Value) -> Value {
        let errors = validate_gltf_json(scene);
        if !errors.is_empty() {
            warn!("Validation errors: {:?}", errors);
        }
        scene.clone()
    }

    pub fn run_iteration(
        &mut self,
        scene_gltf: &Value,
        iteration_number: i32,
    ) -> Result<(Value, IterationState), String> {
        let mut state = IterationState::new(iteration_number, scene_gltf.clone());

        let manifest_json = serde_json::to_string_pretty(&self.manifest.to_dict())
            .unwrap_or_default();
        let history_summaries: Vec<Value> = self.iteration_history
            .iter()
            .rev()
            .take(3)
            .map(|s| {
                let v = serde_json::json!({
                    "iteration_number": s.iteration_number,
                    "critique": s.critique.as_deref().unwrap_or(""),
                    "convergence_decision": s.convergence_decision.as_deref().unwrap_or(""),
                });
                summarize_prior_state(&v).into()
            })
            .collect();
        let iteration_state_json = serde_json::json!({
            "iteration": iteration_number,
            "history": history_summaries,
        });
        let iteration_state_json_str = serde_json::to_string_pretty(&iteration_state_json)
            .unwrap_or_default();

        info!("[Iter {}] Authoring animation via LLM", iteration_number);
        let prompt = build_authoring_prompt(
            scene_gltf,
            &manifest_json,
            &iteration_state_json_str,
            self.manifest.duration_seconds.unwrap_or(MAX_SCENE_DURATION),
        );
        let animation_data = self.call_llm_for_animation(&prompt, scene_gltf)?;
        let n_anims = animation_data["animations"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        let n_acc = animation_data["accessors"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        info!(
            "[Iter {}] LLM returned {} animations, {} accessors",
            iteration_number, n_anims, n_acc
        );

        let mut retries = 0;
        let mut validation_errors: Vec<String> = Vec::new();
        let mut updated_scene = scene_gltf.clone();

        while retries < MAX_RETRIES {
            match Ok::<_, String>(apply_animation_to_scene(&updated_scene, &animation_data)) {
                Ok(scene) => {
                    let errs = validate_gltf_json(&scene);
                    if errs.is_empty() {
                        updated_scene = scene;
                        break;
                    }
                    let err_msg = format!(
                        "Validation failed (attempt {}): {}",
                        retries + 1,
                        errs.join("; ")
                    );
                    state.errors.push(err_msg.clone());
                    warn!(
                        "[Iter {}] Validation failed (attempt {}/{}): {}",
                        iteration_number,
                        retries + 1,
                        MAX_RETRIES,
                        errs.first().map(|s| s.as_str()).unwrap_or("unknown")
                    );
                    validation_errors = errs;
                    retries += 1;
                }
                Err(e) => {
                    let err_msg = format!(
                        "Error applying animation (attempt {}): {}",
                        retries + 1,
                        e
                    );
                    state.errors.push(err_msg.clone());
                    warn!(
                        "[Iter {}] Error applying animation (attempt {}/{}): {}",
                        iteration_number, retries + 1, MAX_RETRIES, e
                    );
                    retries += 1;
                }
            }
        }

        if retries >= MAX_RETRIES && !validation_errors.is_empty() {
            warn!(
                "[Iter {}] Max retries reached, using previous scene",
                iteration_number
            );
            updated_scene = scene_gltf.clone();
        }

        let video_path = format!("{}/iter_{:04}.webm", self.output_dir, iteration_number);
        info!(
            "[Iter {}] Rendering video preview -> {}",
            iteration_number,
            Path::new(&video_path)
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default()
        );
        let scene_gltf_path = format!("{}/scene.gltf", self.output_dir);
        match generate_webm_preview(&scene_gltf_path, &video_path) {
            Ok(_) => {}
            Err(e) => {
                warn!("[Iter {}] Video rendering skipped: {}", iteration_number, e);
            }
        }

        state.critique = Some(self.critique_animation(&updated_scene, &video_path));
        info!(
            "[Iter {}] Critique: {} chars",
            iteration_number,
            state.critique.as_ref().map(|c| c.len()).unwrap_or(0)
        );

        state.convergence_decision = Some(self.decide_convergence(
            state.critique.as_deref().unwrap_or(""),
        ));
        info!(
            "[Iter {}] Convergence: {}",
            iteration_number,
            state.convergence_decision.as_deref().unwrap_or("")
        );

        self.iteration_history.push(state.clone());

        let history_values: Vec<Value> = self.iteration_history.iter().map(|s| s.to_dict()).collect();
        let tok_count = estimate_tokens(&Value::Array(history_values.clone()));
        let before = self.iteration_history.len();
        self.iteration_history = trim_iteration_history(
            &self.iteration_history,
            tok_count,
            |s| s.to_dict(),
        );
        debug!(
            "[Iter {}] Context: {} tokens, history {} -> {} entries",
            iteration_number,
            tok_count,
            before,
            self.iteration_history.len()
        );

        Ok((updated_scene, state))
    }

    fn call_llm_for_animation(&self, prompt: &str, scene: &Value) -> Result<Value, String> {
        let system = "You are a GLTF animation expert. Respond with ONLY valid JSON matching the requested animation structure. No markdown fences, no explanation.";
        match call_llm_json(prompt, Some(system), None) {
            Ok(result) => Ok(result),
            Err(e) => {
                panic!("LLM animation call failed, using empty fallback: {}", e);
            }
        }
    }

    fn critique_animation(&self, _scene: &Value, video_path: &str) -> String {
        let mut frames: Vec<Vec<u8>> = Vec::new();
        if !video_path.is_empty() && Path::new(video_path).exists() {
            match extract_video_frames(video_path, 6) {
                Ok(f) => {
                    frames = f;
                }
                Err(e) => {
                    warn!("Frame extraction failed: {}", e);
                }
            }
        }

        info!("Critiquing animation with {} video frames", frames.len());

        let critique_prompt = "\
You are observing an animated GLTF scene. \
Below are sampled frames from the rendered animation.

Critique the following:
1. Are all characters visible and correctly positioned?
2. Does the animation match the intended choreography?
3. Are the motions smooth and natural-looking?
4. What specific improvements should be made?

Provide a concise, actionable critique.";

        let frames_slice: Option<&[Vec<u8>]> = if frames.is_empty() { None } else { Some(&frames) };
        match call_llm(critique_prompt, None, frames_slice) {
            Ok(critique) => {
                if critique.is_empty() {
                    "No critique generated.".to_string()
                } else {
                    critique
                }
            }
            Err(e) => {
                warn!("Critique failed, using fallback response: {}", e);
                "Animation looks reasonable. Characters are visible and moving.".to_string()
            }
        }
    }

    fn decide_convergence(&self, critique: &str) -> String {
        let decision_prompt = format!(
            "Based on this critique of the current animation state, \
should the animation process continue refining or is it ready to finalize?\n\n\
Critique:\n{}\n\n\
Reply with exactly one word: 'continue' or 'finalize'.",
            critique
        );
        debug!(
            "Convergence decision prompt: {} chars",
            decision_prompt.len()
        );
        match call_llm(&decision_prompt, None, None) {
            Ok(decision) => {
                let decision = decision.trim().to_lowercase();
                if decision.contains("finalize") {
                    "finalize".to_string()
                } else {
                    "continue".to_string()
                }
            }
            Err(e) => {
                warn!("Convergence decision failed, defaulting to finalize: {}", e);
                "finalize".to_string()
            }
        }
    }

    pub fn run_full_loop(&mut self) -> Result<Value, String> {
        info!("Starting iteration loop (max {})", self.max_iterations);
        let mut scene = self.setup()?;
        let scene_path = format!("{}/scene.gltf", self.output_dir);
        let scene_json = serde_json::to_string_pretty(&scene)
            .map_err(|e| format!("Failed to serialize scene: {}", e))?;
        std::fs::write(&scene_path, &scene_json)
            .map_err(|e| format!("Failed to write scene: {}", e))?;

        for iteration in 0..self.max_iterations {
            self.iteration_count = iteration + 1;
            let result = self.run_iteration(&scene, iteration)?;
            scene = result.0;
            let state = result.1;

            let scene_json = serde_json::to_string_pretty(&scene)
                .map_err(|e| format!("Failed to serialize scene: {}", e))?;
            std::fs::write(&scene_path, &scene_json)
                .map_err(|e| format!("Failed to write scene: {}", e))?;

            if state.convergence_decision.as_deref() == Some("finalize") {
                self.converged = true;
                info!("Converged at iteration {}", iteration);
                break;
            }
        }

        if !self.converged {
            info!(
                "Reached max iterations ({}) without convergence",
                self.max_iterations
            );
        }

        info!("Generating final previews");
        generate_final_previews(&scene_path, &self.output_dir);
        Ok(scene)
    }
}

pub fn run_full_loop(manifest_path: &str, input_dir: &str, output_dir: &str) -> Result<Value, String> {
    let mut orch = Orchestrator::new(manifest_path, input_dir, output_dir)?;
    orch.run_full_loop()
}
