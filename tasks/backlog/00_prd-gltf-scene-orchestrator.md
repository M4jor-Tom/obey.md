# PRD: GLTF Scene Orchestrator

## Introduction/Overview

The GLTF Scene Orchestrator is an LLM-driven system that takes one or more provided GLTF character models and a textual prompt, then produces an animated GLTF scene where those characters perform concurrently on a single timeline. The system follows a strict 1:1 input-to-output relationship: each input (a JSON manifest referencing GLTF files plus a text prompt) yields one output (an animated GLTF scene).

The core mechanism is an **iterative visual feedback loop**: the LLM authors animation data directly into the GLTF JSON structure, renders a video preview via `gltf_to_webm.py`, observes the result, critiques it against the prompt, and refines the animation until convergence or a max iteration cap is reached.

The problem it solves: 3D artists and general users need a repeatable, automated way to compose multi-character animated 3D scenes without manual keyframing in a DCC tool.

## Goals

1. Enable a user to define an animated GLTF scene entirely through a text prompt and a set of provided GLTF character files.
2. Maintain a clean file-based project structure under a `scenarios/` directory with separate `input/` and `output/` folders per scenario.
3. Support every available animation technique (transform keyframes, morph targets/blend shapes, skeletal animation, and any other animation channels supported by GLTF) on every character simultaneously.
4. Ensure the process is fully automated by an LLM agent using multimodal vision (observing rendered video frames) and the specified visualization tools (`gltf_to_png.py`, `gltf_to_webm.py`) for preview and verification.
5. Guarantee deterministic 1:1 mapping — one input always produces exactly one output scene.

## User Stories

1. As a 3D artist, I want to place two character GLTFs (a knight and a dragon) in a `scenarios/battle/input/` folder and write a prompt describing their fight, so the system generates a single animated GLTF scene showing them circling and striking each other.
2. As a user, I want the system to use every animation type available (position, rotation, scale keyframes AND morph targets) on my characters so the scene is as rich as possible.
4. As a user previewing results, I want the agent to generate a PNG thumbnail and a WebM video of the output scene so I can quickly verify the result without loading a 3D viewer.

## Process Flow

The system operates as an iterative LLM-driven refinement loop:

1. **Scene Setup** — A root scene GLTF is created that references each input character GLTF via URI (no monolithic mesh merge). All input GLTFs are copied to `output/` as unmodified working copies alongside the scene GLTF.

2. **Animation Authoring** — The LLM writes animation data directly into the scene GLTF's JSON structure: TRS keyframes on node channels, skeletal animation via accessors, morph target weight tracks. The LLM manipulates GLTF JSON natively — no intermediate animation tool.

3. **Render & Observe** — `gltf_to_webm.py` renders the current scene to a video. The LLM receives this video (as sampled frames) and evaluates alignment with the prompt.

4. **Critique & Refine** — The LLM compares current animation state against the prompt, identifies gaps, and creates/modifies/deletes animation channels accordingly. Invalid GLTF JSON output by the LLM is caught and triggers a retry of the authoring step.

5. **Loop** — Steps 2–4 repeat until the LLM determines convergence is reached or max iterations is reached.

6. **Final Output** — The scene GLTF is finalized. `gltf_to_png.py` and `gltf_to_webm.py` produce the previews.

## Functional Requirements

1. **Input Manifest Format**: The system must accept a single JSON file that contains:
   - A list of GLTF references
   - A text prompt describing the desired scene and character actions
   - Character names that map 1:1 to provided GLTF filenames

2. **Project Structure**: The system must organize work under `scenarios/<scenario_name>/` with:
   - `input/` — user-provided manifest JSON and GLTF files
   - `output/` — process-generated files (input GLTFs copied, plus the resulting animated scene GLTF, plus preview PNG and WebM)

3. **GLTF Input Resolution**: The system must read GLTF files from the input directory.

4. **GLTF Input Copying**: The system must copy all input GLTFs into the `output/` directory unmodified. These serve as working copies referenced by the scene GLTF.

5. **Scene GLTF Architecture**: The output must be a single root scene GLTF that references character GLTFs via URI (e.g., `"uri": "knight.gltf"` in a node's mesh extension or via scene-level references). Character geometry and assets are never merged into a monolithic file.

6. **Animation Generation**: The system must produce animation data within the scene GLTF that:
   - Applies concurrent animation on a single timeline across all characters
   - Uses every animation technique available (TRS keyframes, morph targets/blend shapes, skeletal animation, plus all other GLTF-compatible animation channels)
   - Executes the actions described in the text prompt

7. **LLM Animation Authoring**: The LLM must write animation data directly as valid GLTF JSON structures:
   - `animation.channels` mapping node targets to samplers
   - `animation.samplers` with input (time) and output (value) accessors
   - `accessors` referencing `bufferViews` with encoded keyframe data
   - Morph target weight tracks via `target.path: "weights"`

8. **Iteration Control**:
   - The LLM drives a loop: author → render → observe → critique → refine
   - Iterations are capped at a configurable maximum (default: 10, hard upper bound: 100)
   - The LLM produces a convergence decision per iteration (continue vs. finalize)
   - At loop exit (convergence or max iterations reached), the current state is the final output

9. **Multimodal Vision Requirement**: The LLM must support multimodal (image/video) input to analyze rendered frames from `gltf_to_webm.py` output.

10. **GLTF JSON Validation**: Every animation authoring step must validate the produced GLTF JSON structure before proceeding. Invalid output causes a retry (up to 3 attempts per iteration). The system must never emit an unparseable GLTF.

11. **Character Referencing**: Characters must be provided as GLTF files. Their filenames (without extension) serve as their reference names in prompts. The system must never generate new GLTF character geometry.

12. **Visualization Tools**: The agent must use:
    - `gltf_to_png.py` to produce a static preview image of the output scene calling https://github.com/M4jor-Tom/gltf_to_png.py thanks to the flake
    - `gltf_to_webm.py` to produce a video preview of the output animation calling https://github.com/M4jor-Tom/gltf_to_webm.py thanks to the flake
    - During iteration, `gltf_to_webm.py` renders intermediate frames for LLM observation
    - At final output, both tools produce the deliverable previews

13. **1:1 Output Guarantee**: Each input manifest must produce exactly one output animated GLTF scene.

14. **Scene Duration**: If the manifest does not specify a scene duration, the LLM must infer it from the prompt. A hard maximum of 60 seconds applies unless overridden in the manifest. The LLM must not exceed 10,000 keyframes per animation channel.

15. **Missing Animation Capabilities**: When a requested animation type (e.g., morph targets) is unavailable in a given character GLTF, the LLM must detect this and fall back to an available technique (e.g., bone rotation to approximate the same action). The LLM must note the fallback in its convergence report.

16. **Agent Backend Selection**: The system must support two modes:
    - **Remote agent mode**: Configured via `--agent-url`, `--agent-name`, `--agent-key` CLI flags, which set the `OPENAI_BASE_URL`, `OPENAI_MODEL`, and `OPENAI_API_KEY` environment variables respectively.
    - **Local opencode mode**: When no remote agent is configured, the system falls back to running `opencode run` as a subprocess. The `opencode` binary is provided by the Nix flake and uses its own provider/model configuration.
    - The fallback must be transparent: all LLM calls (animation authoring, critique, convergence decision) work through either backend.

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
- Scene GLTF references character GLTFs by URI rather than merging, keeping file sizes manageable and allowing partial re-use.
- The iterative loop is necessary because LLMs cannot produce correct 3D animation from a text prompt in a single pass — visual feedback closes the gap.
- The system is bounded by three independent resource budgets: **max iterations** (loop termination), **scene duration** (timeline length), and **max keyframes per channel** (animation data volume). Exhaustion of any one terminates its respective allocation.

## Technical Considerations

1. **Dependencies**: The agent requires `gltf_to_png.py` and `gltf_to_webm.py` from `github.com/M4jor-Tom/` for visualization.
2. **GLTF Animation**: The output GLTF uses standard node/channel animation structures. All animations run on the same timeline (no separate clips).
3. **Test Command**: All tests run via `nix run .#test` (equivalent to `python -m pytest src/ -v`). The flake provides an `apps.test` entry for this.
4. **Concurrent Animation**: All characters animate simultaneously on a single timeline. The prompt determines relative timing, not separate scene chapters.
5. **LLM Constraints**: The LLM used must have sufficient context window to hold the full GLTF JSON (which can be large with keyframe data) and multimodal vision capability to analyze rendered video frames. The GLTF JSON grows with each iteration as the LLM appends or modifies animation data — context window exhaustion is a real failure mode. The system must either (a) trim older iteration data from the context, (b) summarize prior states, or (c) fail gracefully with a clear message when the context budget is exceeded.
6. **Error Recovery**: Invalid GLTF JSON produced by the LLM is caught by a validation step. The authoring step retries up to 3 times before the iteration is skipped.

## Success Metrics

1. A user can define a multi-character scene entirely through a JSON manifest + prompt and receive a valid animated GLTF as output.
2. The output GLTF loads and plays correctly in standard GLTF viewers (e.g., Babylon.js Sandbox, three.js editor).
3. The generated PNG and WebM previews accurately represent the output animation.
4. Zero unsolicited geometry generation — every triangle in the output originates from input GLTFs.
5. The iterative loop converges (LLM decides "done") within max iterations in at least 80% of runs.

## Open Questions

1. What is the exact prompt format for the LLM to reliably map character names and actions to animation structures? (To be refined through agent-guided Q&A at runtime.)
2. Should the manifest support per-character configuration (speed, scale offset, duration multiplier) alongside the global prompt?
3. How should the LLM sample frames from the rendered WebM — at a fixed rate, or selectively (e.g., key moments)?
4. What is the fallback rendering tool if `gltf_to_webm.py` is unavailable or fails?
5. How should context window exhaustion be handled — trim old iteration state, summarize, or spill to external storage?
