# Tasks: GLTF Scene Orchestrator

## Relevant Files

- `src/manifest.py` - Input manifest parsing, validation, and schema definition
- `src/manifest.test.py` - Unit tests for manifest parser
- `src/gltf_resolver.py` - Local and GitHub GLTF file resolution (sparse clone)
- `src/gltf_resolver.test.py` - Unit tests for GLTF resolver
- `src/scene_builder.py` - Root scene GLTF generation with URI-referenced character nodes
- `src/scene_builder.test.py` - Unit tests for scene builder
- `src/animation_author.py` - LLM prompt templating and animation data GLTF JSON manipulation
- `src/animation_author.test.py` - Unit tests for animation authoring utilities
- `src/orchestrator.py` - Main iterative feedback loop (author → render → observe → critique → refine)
- `src/orchestrator.test.py` - Integration tests for full loop
- `src/validator.py` - GLTF JSON schema validation and error recovery
- `src/validator.test.py` - Unit tests for validator
- `src/visualizer.py` - Integration glue for `gltf_to_png.py` and `gltf_to_webm.py`
- `src/visualizer.test.py` - Unit tests for visualizer integration
- `src/models.py` - Shared data models and types (manifest, scene state, iteration state)
- `src/config.py` - Configuration constants (max iterations, max keyframes, scene duration limits)
- `src/context_manager.py` - Context window management (trim/summarize old iteration state)
- `src/context_manager.test.py` - Unit tests for context management
- `scripts/gltf_to_png.py` - Static preview generation tool (external dependency)
- `scripts/gltf_to_webm.py` - Video preview generation tool (external dependency)
- `README.md` - System documentation, usage examples, manifest schema reference

### Notes

- Unit tests should be placed alongside the code files they test (e.g., `manifest.py` and `manifest.test.py` in the same directory).
- Use `pytest` to run tests: `pytest` runs all tests; `pytest tests/path/to/test.py` runs a specific file.
- External visualization tools (`gltf_to_png.py`, `gltf_to_webm.py`) are sourced from `github.com/M4jor-Tom/` and should be placed in `scripts/`.
- The iterative loop is the core architectural pattern — all other components feed into it.

## Instructions for Completing Tasks

**IMPORTANT:** As you complete each task, you must check it off in this markdown file by changing `- [ ]` to `- [x]`. This helps track progress and ensures you don't skip any steps.

Example:
- `- [ ] 1.1 Read file` → `- [x] 1.1 Read file` (after completing)

Update the file after completing each sub-task, not just after completing an entire parent task.

## Tasks

- [ ] 0.0 Create feature branch
  - [ ] 0.1 Create and checkout a new branch: `git checkout -b feature/gltf-scene-orchestrator`

- [ ] 1.0 Project scaffolding and directory structure
  - [ ] 1.1 Create the `scenarios/` base directory with a `.gitkeep` placeholder
  - [ ] 1.2 Create subdirectory structure: `scenarios/<scenario_name>/input/` and `scenarios/<scenario_name>/output/`
  - [ ] 1.3 Create `scripts/` directory for `gltf_to_png.py` and `gltf_to_webm.py`
  - [ ] 1.4 Initialize Python project with `pyproject.toml` (dependencies: pytest, pygltflib or similar GLTF library)
  - [ ] 1.5 Create `src/` package with `__init__.py` and `src/models.py` defining core data types
  - [ ] 1.6 Create `src/config.py` with all configurable constants (max_iterations=10, hard_max_iterations=100, max_scene_duration=60, max_keyframes_per_channel=10000, max_retries=3)

- [ ] 2.0 Input manifest parsing and GLTF resolution
  - [ ] 2.1 Define the JSON manifest schema in `src/models.py` (characters list with name/gltf_ref/url_type, prompt string, optional duration_seconds, optional per-character config)
  - [ ] 2.2 Implement `src/manifest.py` — `parse_manifest(path)` reads and validates the JSON manifest against the schema
  - [ ] 2.3 Implement `src/manifest.py` — `validate_manifest(data)` returns a list of validation errors, raising on critical failures
  - [ ] 2.4 Implement `src/gltf_resolver.py` — `resolve_local(source_path, dest_path)` copies local GLTF files to `output/`
  - [ ] 2.5 Implement `src/gltf_resolver.py` — `resolve_github_url(url, dest_path)` sparse-clones only the GLTF file from a GitHub repo
  - [ ] 2.6 Implement `src/gltf_resolver.py` — `resolve_all(manifest, input_dir, output_dir)` resolves every GLTF reference and records source URLs for reproducibility
  - [ ] 2.7 Write unit tests for manifest parsing and GLTF resolution

- [ ] 3.0 Scene GLTF architecture — root scene referencing character GLTFs by URI
  - [ ] 3.1 Implement `src/scene_builder.py` — `build_empty_scene()` creates a minimal valid GLTF JSON structure (asset, scene, nodes, scenes)
  - [ ] 3.2 Implement `src/scene_builder.py` — `add_character_node(scene, character_name, gltf_uri)` adds a node that references a character GLTF via URI
  - [ ] 3.3 Implement `src/scene_builder.py` — `build_root_scene(manifest, output_dir)` generates the complete root scene GLTF referencing all character working copies
  - [ ] 3.4 Ensure input GLTFs are copied unmodified to `output/` before scene GLTF references them
  - [ ] 3.5 Write unit tests for scene builder

- [ ] 4.0 LLM animation authoring integration
  - [ ] 4.1 Define LLM prompt template for animation authoring: instruct LLM to produce valid GLTF animation JSON (channels, samplers, accessors, bufferViews)
  - [ ] 4.2 Implement `src/animation_author.py` — `build_authoring_prompt(scene_gltf, manifest, iteration_state)` assembles the prompt with current scene state
  - [ ] 4.3 Implement `src/animation_author.py` — TRS keyframe helpers: `add_translation_keyframe`, `add_rotation_keyframe`, `add_scale_keyframe` that write to GLTF JSON
  - [ ] 4.4 Implement `src/animation_author.py` — morph target support: `add_morph_weight_track(node_index, weight_keyframes)` via `target.path: "weights"`
  - [ ] 4.5 Implement `src/animation_author.py` — skeletal animation support: accessor-based skinning channels
  - [ ] 4.6 Implement `src/animation_author.py` — `apply_animation_to_scene(scene_gltf, animation_data)` merges LLM-generated animation into the scene GLTF
  - [ ] 4.7 Implement `src/animation_author.py` — `detect_missing_capabilities(character_gltf)` identifies unavailable animation types and notes fallback strategies
  - [ ] 4.8 Implement `src/animation_author.py` — concurrent timeline enforcement: all animation channels share a single timeline across characters
  - [ ] 4.9 Write unit tests for animation authoring utilities

- [ ] 5.0 Iterative feedback loop (author → render → observe → critique → refine)
  - [ ] 5.1 Implement `src/orchestrator.py` — `run_iteration(scene_gltf, manifest, iteration_state)` executes one full loop pass
  - [ ] 5.2 Implement `src/orchestrator.py` — authoring step: calls LLM to produce animation, applies it to scene GLTF
  - [ ] 5.3 Implement `src/orchestrator.py` — render step: calls `gltf_to_webm.py` on current scene to produce intermediate video
  - [ ] 5.4 Implement `src/orchestrator.py` — observe step: samples frames from rendered video, feeds to LLM for multimodal analysis
  - [ ] 5.5 Implement `src/orchestrator.py` — critique step: LLM compares current animation against prompt, identifies gaps
  - [ ] 5.6 Implement `src/orchestrator.py` — refine step: LLM produces delta animation data to address gaps
  - [ ] 5.7 Implement `src/orchestrator.py` — convergence decision: LLM determines continue vs finalize per iteration
  - [ ] 5.8 Implement `src/orchestrator.py` — `run_full_loop(manifest)` entry point that drives the loop until convergence or max iterations
  - [ ] 5.9 Implement `src/context_manager.py` — `trim_iteration_history(history, max_tokens)` and `summarize_prior_state(state)` for context window management
  - [ ] 5.10 Write integration tests for the full orchestration loop

- [ ] 6.0 GLTF JSON validation and error recovery
  - [ ] 6.1 Implement `src/validator.py` — `validate_gltf_json(gltf_dict)` validates GLTF structure (required fields, correct types, valid references)
  - [ ] 6.2 Implement `src/validator.py` — `validate_animations(gltf_dict)` validates animation channels, samplers, accessors, and bufferViews
  - [ ] 6.3 Implement retry logic in orchestrator: on validation failure, retry authoring step up to 3 times before skipping iteration
  - [ ] 6.4 Implement graceful failure: if retries exhausted, log error and proceed to next iteration (never emit unparseable GLTF)
  - [ ] 6.5 Implement context window exhaustion handling: when context budget exceeded, trim oldest iteration data or summarize prior state
  - [ ] 6.6 Write unit tests for validator

- [ ] 7.0 Visualization output (preview generation)
  - [ ] 7.1 Implement `src/visualizer.py` — `generate_png_preview(scene_gltf_path, output_path)` calls `gltf_to_png.py`
  - [ ] 7.2 Implement `src/visualizer.py` — `generate_webm_preview(scene_gltf_path, output_path)` calls `gltf_to_webm.py`
  - [ ] 7.3 Implement `src/visualizer.py` — `generate_intermediate_frames(scene_gltf_path, output_dir)` for LLM observation during iteration loop
  - [ ] 7.4 Implement `src/visualizer.py` — `generate_final_previews(scene_path, output_dir)` produces deliverable PNG and WebM
  - [ ] 7.5 Implement `src/visualizer.py` — `check_tools_available()` detects if visualization tools are present and provides fallback messaging
  - [ ] 7.6 Write unit tests for visualizer integration

- [ ] 8.0 Testing, convergence metrics, and documentation
  - [ ] 8.1 Write end-to-end test: provide a minimal manifest + two simple GLTF cube models as characters, verify full loop produces a valid output
  - [ ] 8.2 Write convergence metric tests: verify loop converges within max iterations under controlled conditions
  - [ ] 8.3 Write boundary tests: max iterations cap enforcement, scene duration cap, max keyframes per channel enforcement
  - [ ] 8.4 Write error recovery tests: invalid GLTF JSON retry logic, context window exhaustion handling
  - [ ] 8.5 Write `README.md` with system overview, setup instructions, manifest schema reference, and usage examples
  - [ ] 8.6 Add example scenario with sample manifest JSON and two simple test GLTF files to `scenarios/example/`
