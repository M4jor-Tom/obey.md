# PRD: GLTF Scene Orchestrator

## Introduction/Overview

The GLTF Scene Orchestrator is an agent-driven system that takes one or more provided GLTF character models and a textual prompt, then produces an animated GLTF scene where those characters perform concurrently on a single timeline. The system follows a strict 1:1 input-to-output relationship: each input (a JSON manifest referencing GLTF files plus a text prompt) yields one output (an animated GLTF scene).

The problem it solves: 3D artists and general users need a repeatable, automated way to compose multi-character animated 3D scenes without manual keyframing in a DCC tool.

## Goals

1. Enable a user to define an animated GLTF scene entirely through a text prompt and a set of provided GLTF character files.
2. Maintain a clean file-based project structure under a `scenarios/` directory with separate `input/` and `output/` folders per scenario.
3. Support every available animation technique (transform keyframes, morph targets/blend shapes, and any other animation channels supported by GLTF) on every character simultaneously.
4. Ensure the process is fully automated by an agent using the specified visualization tools (`gltf_to_png.py`, `gltf_to_webm.py`) for preview and verification.
5. Guarantee deterministic 1:1 mapping — one input always produces exactly one output scene.

## User Stories

1. As a 3D artist, I want to place two character GLTFs (a knight and a dragon) in a `scenarios/battle/input/` folder and write a prompt describing their fight, so the system generates a single animated GLTF scene showing them circling and striking each other.
2. As a general user, I want to reference a GLTF hosted on GitHub via URL in my JSON manifest, so I don't need to download assets manually.
3. As a user, I want the system to use every animation type available (position, rotation, scale keyframes AND morph targets) on my characters so the scene is as rich as possible.
4. As a user previewing results, I want the agent to generate a PNG thumbnail and a WebM video of the output scene so I can quickly verify the result without loading a 3D viewer.

## Functional Requirements

1. **Input Manifest Format**: The system must accept a single JSON file that contains:
   - A list of GLTF references (either local filesystem paths, or GitHub repository URLs for sparse cloning)
   - A text prompt describing the desired scene and character actions
   - Character names that map 1:1 to provided GLTF filenames

2. **Project Structure**: The system must organize work under `scenarios/<scenario_name>/` with:
   - `input/` — user-provided manifest JSON and GLTF files
   - `output/` — process-generated files (input GLTFs copied, plus the resulting animated GLTF scene)

3. **GLTF Input Resolution**: The system must support both:
   - Local GLTF file references (already present in `input/`)
   - GitHub repository URLs (sparse-cloned into `input/` at runtime)

4. **GLTF Input Copying**: The system must copy all input GLTFs into the `output/` directory unmodified.

5. **Animation Generation**: The system must produce a single animated GLTF scene that:
   - Incorporates all provided character GLTFs
   - Applies concurrent animation on a single timeline
   - Uses every animation technique available (transform keyframes, morph targets/blend shapes, plus all other GLTF-compatible animation channels)
   - Executes the actions described in the text prompt

6. **Character Referencing**: Characters must be provided as GLTF files. Their filenames (without extension) serve as their reference names in prompts. The system must never generate new GLTF character geometry.

7. **Visualization Tools**: The agent must use:
   - `gltf_to_png.py` to produce a static preview image of the output scene
   - `gltf_to_webm.py` to produce a video preview of the output animation

8. **1:1 Output Guarantee**: Each input manifest must produce exactly one output animated GLTF scene.

## Non-Goals (Out of Scope)

1. Generating new 3D character geometry from text — all characters must be provided as input GLTFs.
2. Interactive or branching scene structures — output is always a single linear timeline.
3. A graphical user interface — the system is agent-driven and CLI/file-based.
4. Real-time rendering or game-engine integration — the output is a standard GLTF file for offline use.
5. Procedural texture or material generation beyond what is present in input GLTFs.
6. Audio/sound integration in the output GLTF.

## Design Considerations

- The `scenarios/` directory structure mirrors common creative-project conventions, making it intuitive for artists.
- JSON manifest format should be minimal and human-writable (no schema compilation step needed).
- When a GLTF is sourced from GitHub via sparse clone, the agent should record the source URL for reproducibility.

## Technical Considerations

1. **Dependencies**: The agent requires `gltf_to_png.py` and `gltf_to_webm.py` from `github.com/M4jor-Tom/` for visualization.
2. **GLTF Animation**: The output GLTF uses the standard KHR_animation_pointer or node/channel animation structures. All animations run on the same timeline (no separate clips).
3. **Git Sparse Clone**: When resolving GitHub URLs, the agent will use `git sparse-checkout` to clone only the relevant GLTF files, not entire repositories.
4. **Concurrent Animation**: All characters animate simultaneously on a single timeline. The prompt determines relative timing, not separate scene chapters.

## Success Metrics

1. A user can define a multi-character scene entirely through a JSON manifest + prompt and receive a valid animated GLTF as output.
2. The output GLTF loads and plays correctly in standard GLTF viewers (e.g., Babylon.js Sandbox, three.js editor).
3. The generated PNG and WebM previews accurately represent the output animation.
4. Zero unsolicited geometry generation — every triangle in the output originates from input GLTFs.

## Open Questions

1. How should the prompt language be structured to reliably map character names to actions? (To be refined through agent-guided Q&A at runtime.)
2. Should there be a maximum scene duration or animation budget?
3. What fallback behavior applies when a requested animation type (e.g., morph target) is unavailable in a given character GLTF?
4. Should the manifest support per-character configuration (speed, scale offset) alongside the global prompt?
