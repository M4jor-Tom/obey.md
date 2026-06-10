# Graph Report - obey.md  (2026-06-11)

## Corpus Check
- 29 files · ~10,406 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 199 nodes · 420 edges · 16 communities (13 shown, 3 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 8 edges (avg confidence: 0.5)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `5b82adb8`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]

## God Nodes (most connected - your core abstractions)
1. `Manifest` - 22 edges
2. `Orchestrator` - 17 edges
3. `validate_gltf_json()` - 14 edges
4. `CharacterConfig` - 13 edges
5. `build_root_scene()` - 13 edges
6. `call_llm()` - 11 edges
7. `PRD: GLTF Scene Orchestrator` - 11 edges
8. `_add_trs_keyframe()` - 9 edges
9. `parse_manifest()` - 9 edges
10. `add_morph_weight_track()` - 8 edges

## Surprising Connections (you probably didn't know these)
- `Manifest` --uses--> `Manifest`  [INFERRED]
  src/gltf_resolver.py → src/models.py
- `IterationState` --uses--> `Manifest`  [INFERRED]
  src/orchestrator.py → src/models.py
- `Orchestrator` --uses--> `Manifest`  [INFERRED]
  src/orchestrator.py → src/models.py
- `Manifest` --uses--> `Manifest`  [INFERRED]
  src/scene_builder.py → src/models.py
- `test_build_authoring_prompt()` --calls--> `build_authoring_prompt()`  [EXTRACTED]
  src/animation_author_test.py → src/animation_author.py

## Import Cycles
- 1-file cycle: `src/llm_client.py -> src/llm_client.py`

## Communities (16 total, 3 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.13
Nodes (31): Manifest, resolve_all(), resolve_local(), _create_gltf_dir(), test_resolve_all_local(), test_resolve_all_missing_local(), test_resolve_local(), test_resolve_local_copies_all_files() (+23 more)

### Community 1 - "Community 1"
Cohesion: 0.18
Nodes (24): _add_accessor(), _add_buffer_view(), add_morph_weight_track(), add_rotation_keyframe(), add_scale_keyframe(), add_translation_keyframe(), _add_trs_keyframe(), apply_animation_to_scene() (+16 more)

### Community 2 - "Community 2"
Cohesion: 0.18
Nodes (17): Any, configure_logging(), estimate_tokens(), summarize_prior_state(), test_estimate_tokens_dict(), test_estimate_tokens_string(), test_summarize_prior_state(), test_trim_iteration_history_over_limit() (+9 more)

### Community 3 - "Community 3"
Cohesion: 0.21
Nodes (21): OpenAI, call_llm(), call_llm_json(), _call_opencode(), _ensure_config(), _get_client(), _get_model(), _has_remote_config() (+13 more)

### Community 4 - "Community 4"
Cohesion: 0.21
Nodes (11): IterationState, extract_video_frames(), IterationState, Orchestrator, run_full_loop(), _create_cube_gltf(), _create_minimal_manifest(), test_orchestrator_full_loop() (+3 more)

### Community 5 - "Community 5"
Cohesion: 0.31
Nodes (10): test_animation_missing_sampler(), test_empty_scenes(), test_missing_asset(), test_missing_scenes(), test_valid_animation(), test_valid_minimal_scene(), test_validate_animations(), _validate_animation() (+2 more)

### Community 6 - "Community 6"
Cohesion: 0.17
Nodes (11): Design Considerations, Functional Requirements, Goals, Introduction/Overview, Non-Goals (Out of Scope), Open Questions, PRD: GLTF Scene Orchestrator, Process Flow (+3 more)

### Community 7 - "Community 7"
Cohesion: 0.30
Nodes (10): parse_manifest(), test_parse_manifest_file_not_found(), test_parse_manifest_invalid_json(), test_parse_manifest_valid_file(), test_validate_manifest_empty_characters(), test_validate_manifest_gltf_ref_missing_dot_gltf(), test_validate_manifest_invalid_duration(), test_validate_manifest_missing_prompt() (+2 more)

### Community 8 - "Community 8"
Cohesion: 0.22
Nodes (8): Configuration, Dependencies, Development, GLTF Scene Orchestrator, How It Works, Manifest Schema, Project Structure, Usage

### Community 9 - "Community 9"
Cohesion: 0.33
Nodes (5): Agent Dispatch, Inputs and Outputs, Process Logics, Prompt execution, Test command

### Community 10 - "Community 10"
Cohesion: 0.33
Nodes (5): Instructions for Completing Tasks, Notes, Relevant Files, Tasks, Tasks: GLTF Scene Orchestrator

## Knowledge Gaps
- **31 isolated node(s):** `$schema`, `plugin`, `@opencode-ai/plugin`, `Any`, `SceneState` (+26 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `call_llm()` connect `Community 3` to `Community 2`, `Community 4`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._
- **Why does `Manifest` connect `Community 0` to `Community 2`, `Community 4`?**
  _High betweenness centrality (0.082) - this node is a cross-community bridge._
- **Why does `validate_gltf_json()` connect `Community 5` to `Community 2`, `Community 4`?**
  _High betweenness centrality (0.067) - this node is a cross-community bridge._
- **Are the 5 inferred relationships involving `Manifest` (e.g. with `IterationState` and `Manifest`) actually correct?**
  _`Manifest` has 5 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `Orchestrator` (e.g. with `IterationState` and `Manifest`) actually correct?**
  _`Orchestrator` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `$schema`, `plugin`, `@opencode-ai/plugin` to the rest of the system?**
  _31 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.1349527665317139 - nodes in this community are weakly interconnected._