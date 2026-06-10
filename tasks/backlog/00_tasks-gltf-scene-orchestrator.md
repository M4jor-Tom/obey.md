# Tasks: GLTF Scene Orchestrator

## Relevant Files

- `src/manifest.py` - Input manifest parsing, validation, and schema definition
- `src/manifest.test.py` - Unit tests for manifest parser
- `src/gltf_resolver.py` - Local GLTF file resolution
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

- [x] 0.0 Create feature branch
  - [x] 0.1 Create and checkout a new branch: `git checkout -b feature/gltf-scene-orchestrator`

- [x] 1.0 Project scaffolding and directory structure
  - [x] 1.1 Create the `scenarios/` base directory with a `.gitkeep` placeholder
  - [x] 1.2 Create subdirectory structure: `scenarios/<scenario_name>/input/` and `scenarios/<scenario_name>/output/`
  - [x] 1.3 Create `scripts/` directory for `gltf_to_png.py` and `gltf_to_webm.py`
  - [x] 1.4 Initialize Python project with `pyproject.toml` (dependencies: pytest)
  - [x] 1.5 Create `src/` package with `__init__.py` and `src/models.py` defining core data types
  - [x] 1.6 Create `src/config.py` with all configurable constants (max_iterations=10, hard_max_iterations=100, max_scene_duration=60, max_keyframes_per_channel=10000, max_retries=3)

- [x] 2.0 Input manifest parsing and GLTF resolution
  - [x] 2.1 Define the JSON manifest schema in `src/models.py` (characters list with name/gltf_ref, prompt string, optional duration_seconds)
  - [x] 2.2 Implement `src/manifest.py` — `parse_manifest(path)` reads and validates the JSON manifest against the schema
  - [x] 2.3 Implement `src/manifest.py` — `validate_manifest(data)` returns a list of validation errors
  - [x] 2.4 Implement `src/gltf_resolver.py` — `resolve_local(source_path, dest_path)` copies local GLTF files to `output/`
  - [x] 2.5 (removed — remote URL resolution was never implemented)
  - [x] 2.6 Implement `src/gltf_resolver.py` — `resolve_all(manifest, input_dir, output_dir)` resolves every GLTF reference
  - [x] 2.7 Write unit tests for manifest parsing and GLTF resolution

- [x] 3.0 Scene GLTF architecture — root scene referencing character GLTFs by URI
  - [x] 3.1 Implement `src/scene_builder.py` — `build_empty_scene()` creates a minimal valid GLTF JSON structure
  - [x] 3.2 Implement `src/scene_builder.py` — `add_character_node(scene, character_name, gltf_uri)` adds a node
  - [x] 3.3 Implement `src/scene_builder.py` — `build_root_scene(manifest, output_dir)` generates complete root scene GLTF
  - [x] 3.4 Ensure input GLTFs are copied unmodified to `output/` before scene GLTF references them
  - [x] 3.5 Write unit tests for scene builder

- [x] 4.0 LLM animation authoring integration
  - [x] 4.1 Define LLM prompt template for animation authoring
  - [x] 4.2 Implement `src/animation_author.py` — `build_authoring_prompt(scene_gltf, manifest, iteration_state)`
  - [x] 4.3 Implement TRS keyframe helpers: `add_translation_keyframe`, `add_rotation_keyframe`, `add_scale_keyframe`
  - [x] 4.4 Implement morph target support: `add_morph_weight_track` via `target.path: "weights"`
  - [x] 4.5 Implement skeletal animation support via accessor-based skinning channels
  - [x] 4.6 Implement `apply_animation_to_scene(scene_gltf, animation_data)`
  - [x] 4.7 Implement `detect_missing_capabilities(character_gltf)` with fallback strategies
  - [x] 4.8 Implement concurrent timeline enforcement (single timeline across characters)
  - [x] 4.9 Write unit tests for animation authoring utilities

- [x] 5.0 Iterative feedback loop (author → render → observe → critique → refine)
  - [x] 5.1 Implement `src/orchestrator.py` — `run_iteration(scene_gltf, manifest, iteration_state)`
  - [x] 5.2 Implement authoring step: calls LLM to produce animation, applies to scene GLTF
  - [x] 5.3 Implement render step: calls `gltf_to_webm.py` to produce intermediate video
  - [x] 5.4 Implement observe step: renders video for LLM multimodal analysis
  - [x] 5.5 Implement critique step: LLM compares current animation against prompt
  - [x] 5.6 Implement refine step: LLM produces delta animation data to address gaps
  - [x] 5.7 Implement convergence decision: LLM determines continue vs finalize
  - [x] 5.8 Implement `run_full_loop(manifest)` entry point driving loop until convergence
  - [x] 5.9 Implement `context_manager.py` — `trim_iteration_history` and `summarize_prior_state`
  - [x] 5.10 Write integration tests for the full orchestration loop

- [x] 6.0 GLTF JSON validation and error recovery
  - [x] 6.1 Implement `src/validator.py` — `validate_gltf_json(gltf_dict)` validates GLTF structure
  - [x] 6.2 Implement `src/validator.py` — `validate_animations(gltf_dict)` validates animation structures
  - [x] 6.3 Implement retry logic in orchestrator: retry authoring step up to 3 times
  - [x] 6.4 Implement graceful failure: log error and proceed to next iteration
  - [x] 6.5 Implement context window exhaustion handling: trim oldest iteration data
  - [x] 6.6 Write unit tests for validator

- [x] 7.0 Visualization output (preview generation)
  - [x] 7.1 Implement `src/visualizer.py` — `generate_png_preview(scene_gltf_path, output_path)`
  - [x] 7.2 Implement `src/visualizer.py` — `generate_webm_preview(scene_gltf_path, output_path)`
  - [x] 7.3 Implement `src/visualizer.py` — `generate_intermediate_frames()` for LLM observation
  - [x] 7.4 Implement `src/visualizer.py` — `generate_final_previews()` produces PNG and WebM
  - [x] 7.5 Implement `src/visualizer.py` — `check_tools_available()` with fallback messaging
  - [x] 7.6 Write unit tests for visualizer integration

- [x] 8.0 Testing, convergence metrics, and documentation
  - [x] 8.1 Write end-to-end test: minimal manifest + two GLTF cube models, verify loop produces valid output
  - [x] 8.2 Write convergence metric tests: verify loop converges within max iterations
  - [x] 8.3 Write boundary tests: max iterations cap enforcement
  - [x] 8.4 Write error recovery tests: retry logic, context window exhaustion handling
  - [x] 8.5 Write `README.md` with system overview, setup instructions, manifest schema reference, usage examples
  - [x] 8.6 Add example scenario with sample manifest JSON and two test GLTF files to `scenarios/example/`

- [x] 9.0 CLI argument parsing and `nix run` support
  - [x] 9.1 Refactor `Orchestrator` to accept explicit `input_dir` / `output_dir` instead of `scenario_dir`
  - [x] 9.2 Add `__main__` block with `-i`/`--input` and `-o`/`--output` flags
  - [x] 9.3 Update `run_full_loop()` module-level function signature
  - [x] 9.4 Add `apps.default` to `flake.nix` for `nix run . -- -i <dir> -o <dir>`
  - [x] 9.5 Update all tests for the new `Orchestrator` constructor signature
  - [x] 9.6 Update `README.md` usage section

- [x] 10.0 Directory-as-file refactor
  - [x] 10.1 Remove unused code paths from `gltf_resolver.py`
  - [x] 10.2 Rewrite `resolve_local` to always copy entire directories recursively; update `resolve_all` to remove branching
  - [x] 10.3 Update `build_root_scene` to look inside `<gltf_ref>/<gltf_ref>` directory
  - [x] 10.4 Remove unused fields from `CharacterConfig` and manifest validation; add `.gltf`-contains check
  - [x] 10.5 Update all test fixtures for directory structure
  - [x] 10.6 Update README manifest docs and project structure

- [x] 11.0 Local opencode agent fallback
  - [x] 11.1 Add `opencode` subprocess backend to `src/llm_client.py` — `_call_opencode()` pipes prompts to `opencode run --format json` and parses the JSON event stream to extract response text
  - [x] 11.2 Modify `call_llm()` in `src/llm_client.py` to detect when no remote API config is available and fall back to the opencode subprocess
  - [x] 11.3 Modify `_ensure_config()` to skip interactive prompts when no env vars are set (defer to opencode fallback)
  - [x] 11.4 Add `--agent-url`, `--agent-name`, `--agent-key` CLI flags to `src/orchestrator.py` that set the corresponding `OPENAI_*` env vars
  - [x] 11.5 Update `flake.nix` to include `opencode` in the dev shell and app wrapper PATH
  - [x] 11.6 Write tests for opencode fallback path in `llm_client`

- [x] 12.0 `nix run .#test` integration
  - [x] 12.1 Add `apps.test` to `flake.nix` — wraps `python -m pytest src/ -v` in a `writeShellScriptBin` app
  - [x] 12.2 Update `00_init.prompt.md` with test command reference
  - [x] 12.3 Update `00_prd-gltf-scene-orchestrator.md` Technical Considerations with test command
