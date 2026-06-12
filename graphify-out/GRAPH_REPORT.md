# Graph Report - obey.md  (2026-06-12)

## Corpus Check
- 21 files · ~11,428 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 219 nodes · 455 edges · 18 communities (15 shown, 3 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 7 edges (avg confidence: 0.63)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2b1d15d8`
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
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]

## God Nodes (most connected - your core abstractions)
1. `Orchestrator` - 16 edges
2. `call_llm()` - 15 edges
3. `Manifest` - 15 edges
4. `Value` - 14 edges
5. `validate_gltf_json()` - 13 edges
6. `IterationState` - 11 edges
7. `PRD: GLTF Scene Orchestrator` - 11 edges
8. `add_trs_keyframe()` - 10 edges
9. `validate_manifest()` - 10 edges
10. `build_root_scene()` - 10 edges

## Surprising Connections (you probably didn't know these)
- `call_opencode()` --calls--> `estimate_tokens_str()`  [INFERRED]
  src/llm_client.rs → src/context_manager.rs
- `main()` --calls--> `configure_logging()`  [INFERRED]
  src/main.rs → src/config.rs
- `IterationState` --uses--> `Manifest`  [INFERRED]
  src/orchestrator.rs → src/models.rs
- `Orchestrator` --uses--> `Manifest`  [INFERRED]
  src/orchestrator.rs → src/models.rs
- `IterationState` --uses--> `IterationState`  [INFERRED]
  src/orchestrator.rs → src/models.rs

## Import Cycles
- 1-file cycle: `src/gltf_resolver.rs -> src/gltf_resolver.rs`
- 1-file cycle: `src/orchestrator.rs -> src/orchestrator.rs`
- 1-file cycle: `src/context_manager.rs -> src/context_manager.rs`
- 1-file cycle: `src/models.rs -> src/models.rs`
- 1-file cycle: `src/manifest.rs -> src/manifest.rs`
- 1-file cycle: `src/scene_builder.rs -> src/scene_builder.rs`
- 1-file cycle: `src/visualizer.rs -> src/visualizer.rs`
- 1-file cycle: `src/llm_client.rs -> src/llm_client.rs`
- 1-file cycle: `src/validator.rs -> src/validator.rs`

## Communities (18 total, 3 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.19
Nodes (17): Error, CharacterConfig, IterationState, load_manifest_from_string(), Manifest, Option, Result, Self (+9 more)

### Community 1 - "Community 1"
Cohesion: 0.17
Nodes (27): add_accessor(), add_buffer_view(), add_morph_weight_track(), add_rotation_keyframe(), add_scale_keyframe(), add_translation_keyframe(), add_trs_keyframe(), apply_animation_to_scene() (+19 more)

### Community 2 - "Community 2"
Cohesion: 0.16
Nodes (14): F, configure_logging(), Option, estimate_tokens(), estimate_tokens_str(), String, Value, Vec (+6 more)

### Community 3 - "Community 3"
Cohesion: 0.40
Nodes (14): call_llm(), call_llm_json(), call_openai(), call_opencode(), extract_video_frames(), get_api_key(), get_base_url(), get_model() (+6 more)

### Community 4 - "Community 4"
Cohesion: 0.32
Nodes (9): IterationState, Orchestrator, Manifest, Result, Self, String, Value, Vec (+1 more)

### Community 5 - "Community 5"
Cohesion: 0.29
Nodes (10): String, Value, Vec, test_animation_missing_sampler(), test_empty_scenes(), test_missing_asset(), test_missing_scenes(), validate_animation() (+2 more)

### Community 6 - "Community 6"
Cohesion: 0.17
Nodes (11): Design Considerations, Functional Requirements, Goals, Introduction/Overview, Non-Goals (Out of Scope), Open Questions, PRD: GLTF Scene Orchestrator, Process Flow (+3 more)

### Community 7 - "Community 7"
Cohesion: 0.23
Nodes (14): parse_manifest(), Manifest, Result, String, Value, Vec, test_parse_manifest_file_not_found(), test_parse_manifest_valid_file() (+6 more)

### Community 8 - "Community 8"
Cohesion: 0.22
Nodes (8): Configuration, Development, Environment Variables, GLTF Scene Orchestrator, How It Works, Manifest Schema, Project Structure, Usage

### Community 9 - "Community 9"
Cohesion: 0.33
Nodes (5): Agent Dispatch, Inputs and Outputs, Process Logics, Prompt execution, Test command

### Community 10 - "Community 10"
Cohesion: 0.33
Nodes (5): Instructions for Completing Tasks, Notes, Relevant Files, Tasks, Tasks: GLTF Scene Orchestrator

### Community 14 - "Community 14"
Cohesion: 0.18
Nodes (12): Path, copy_dir_recursively(), resolve_all(), resolve_local(), HashMap, Manifest, Result, String (+4 more)

### Community 15 - "Community 15"
Cohesion: 0.23
Nodes (18): Map, add_character_node(), build_empty_scene(), build_root_scene(), embed_extensions_from(), get_or_create_array(), merge_character_gltf(), remap_texture_infos() (+10 more)

### Community 16 - "Community 16"
Cohesion: 0.41
Nodes (11): check_tools_available(), generate_final_previews(), generate_intermediate_frames(), generate_png_preview(), generate_webm_preview(), HashMap, Option, Result (+3 more)

## Knowledge Gaps
- **51 isolated node(s):** `$schema`, `plugin`, `@opencode-ai/plugin`, `String`, `Option` (+46 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `call_llm()` connect `Community 3` to `Community 2`, `Community 4`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Why does `Manifest` connect `Community 0` to `Community 2`, `Community 4`, `Community 7`, `Community 14`, `Community 15`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Why does `Path` connect `Community 14` to `Community 16`, `Community 2`, `Community 15`, `Community 7`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Orchestrator` (e.g. with `IterationState` and `Manifest`) actually correct?**
  _`Orchestrator` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `Manifest` (e.g. with `IterationState` and `Orchestrator`) actually correct?**
  _`Manifest` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `$schema`, `plugin`, `@opencode-ai/plugin` to the rest of the system?**
  _51 weakly-connected nodes found - possible documentation gaps or missing edges._