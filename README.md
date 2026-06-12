# GLTF Scene Orchestrator

LLM-driven system that takes GLTF character models and a text prompt, then produces an animated GLTF scene where characters perform concurrently on a single timeline.

## How It Works

The system operates as an iterative visual feedback loop:

1. **Setup** — A root scene GLTF is created referencing each input character via URI. Input GLTFs are copied to `output/` unmodified.
2. **Author** — The LLM writes animation data directly into the GLTF JSON (TRS keyframes, morph targets, skeletal animation).
3. **Render & Observe** — Renders the scene to video via `gltf_to_webm`. The LLM observes sampled frames.
4. **Critique & Refine** — The LLM compares output to the prompt and produces delta animation data.
5. **Loop** — Steps 2–4 repeat until convergence or max iterations.
6. **Final Output** — Animated scene GLTF plus PNG and WebM previews.

## Project Structure

```
src/                             # Rust crate
├── main.rs                     # CLI entry point
├── lib.rs                      # Module declarations
├── manifest.rs                 # Manifest parsing and validation
├── gltf_resolver.rs            # Local GLTF file resolution
├── scene_builder.rs            # Root scene GLTF generation
├── animation_author.rs         # LLM prompt templating and animation helpers
├── validator.rs                # GLTF JSON schema validation
├── orchestrator.rs             # Main iterative feedback loop
├── context_manager.rs          # Context window management
├── visualizer.rs               # Preview generation integration
├── models.rs                   # Shared data types
├── config.rs                   # Configuration constants
└── llm_client.rs               # LLM API client (OpenAI / opencode)
Cargo.toml                      # Rust dependencies
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
# Build
cargo build

# Run with explicit input and output directories
cargo run -- -i scenarios/example/input -o scenarios/example/output

# Run with nix
nix run . -- -i scenarios/example/input -o scenarios/example/output

# Run all tests
cargo test
```

## Environment Variables

| Variable | Description |
|---|---|
| `OPENAI_API_KEY` | API key for remote LLM provider |
| `OPENAI_BASE_URL` | Base URL for OpenAI-compatible API |
| `OPENAI_MODEL` | Model name (default: gpt-4o) |

When no remote config is set, the system falls back to the `opencode` CLI as a subprocess backend.

## Development

```bash
# Enter dev shell (Nix)
nix develop

# Build
cargo build

# Run tests
cargo test
```

## Configuration

See `src/config.rs`:
- `MAX_ITERATIONS` (default: 10)
- `HARD_MAX_ITERATIONS` (default: 100)
- `MAX_SCENE_DURATION` (default: 60s)
- `MAX_KEYFRAMES_PER_CHANNEL` (default: 10000)
- `MAX_RETRIES` (default: 3)
- `CONTEXT_BUDGET_TOKENS` (default: 128000)
