# GLTF Scene Orchestrator

LLM-driven system that takes GLTF character models and a text prompt, then produces an animated GLTF scene where characters perform concurrently on a single timeline.

## How It Works

The system operates as an iterative visual feedback loop:

1. **Setup** — A root scene GLTF is created referencing each input character via URI. Input GLTFs are copied to `output/` unmodified.
2. **Author** — The LLM writes animation data directly into the GLTF JSON (TRS keyframes, morph targets, skeletal animation).
3. **Render & Observe** — `gltf_to_webm.py` renders the scene to video. The LLM observes sampled frames.
4. **Critique & Refine** — The LLM compares output to the prompt and produces delta animation data.
5. **Loop** — Steps 2–4 repeat until convergence or max iterations.
6. **Final Output** — Animated scene GLTF plus PNG and WebM previews.

## Project Structure

```
scenarios/<scenario_name>/
├── input/                       # Manifest JSON + character directories
│   ├── manifest.json
│   ├── character.gltf/          # 1 entity = 1 directory
│   │   ├── character.gltf       # Actual GLTF file
│   │   └── texture.png          # Companion assets
│   └── other.gltf/
│       ├── other.gltf
│       └── ...
├── output/                      # Generated scene GLTF + previews
│   ├── scene.gltf
│   ├── character.gltf/          # Copied from input
│   │   ├── character.gltf
│   │   └── texture.png
│   ├── preview.png
│   ├── preview.webm
│   └── iter_0000.webm
scripts/                         # (Optional) manual tool placement; Nix provides gltf_to_png, gltf_to_webm on PATH
src/                             # Python package
├── manifest.py                  # Manifest parsing and validation
├── gltf_resolver.py             # Local GLTF file resolution
├── scene_builder.py             # Root scene GLTF generation
├── animation_author.py          # LLM prompt templating and animation helpers
├── validator.py                 # GLTF JSON schema validation
├── orchestrator.py              # Main iterative feedback loop
├── context_manager.py           # Context window management
├── visualizer.py                # Preview generation integration
├── models.py                    # Shared data types
└── config.py                    # Configuration constants
```

## Manifest Schema

A JSON manifest defines the scene:

```json
{
  "prompt": "Description of the animated scene",
  "characters": [
    {
      "name": "character_name",
      "gltf_ref": "character.gltf"
    }
  ],
  "duration_seconds": 15.0
}
```

- `prompt` (required): Scene description driving all animation decisions
- `characters` (required): Array of character references
  - `name`: Reference name used in prompts
  - `gltf_ref`: Directory name (must contain `.gltf`) — the directory holds the actual GLTF file, textures, and other assets. 1 entity = 1 directory.
- `duration_seconds` (optional): Scene duration in seconds (default: LLM-inferred, max: 60)

## Usage

```bash
# Run with explicit input and output directories
python -m src.orchestrator -i scenarios/example/input -o scenarios/example/output

# The input directory must contain a manifest.json and one directory per character:
# scenarios/example/input/
#   manifest.json
#   character.gltf/
#     character.gltf
#     textures/...

# Run with nix
nix run . -- -i scenarios/example/input -o scenarios/example/output

# Run all tests
python -m pytest src/ -v
```

## Development

```bash
# Enter dev shell (Nix)
nix develop

# Run tests
python -m pytest src/ -v
```

## Dependencies

- Python 3.10+

## Configuration

See `src/config.py`:
- `MAX_ITERATIONS` (default: 10)
- `HARD_MAX_ITERATIONS` (default: 100)
- `MAX_SCENE_DURATION` (default: 60s)
- `MAX_KEYFRAMES_PER_CHANNEL` (default: 10000)
- `MAX_RETRIES` (default: 3)
- `CONTEXT_BUDGET_TOKENS` (default: 128000)
