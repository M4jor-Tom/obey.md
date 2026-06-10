## Inputs and Outputs
- I want to prompt scenarios (.md files) for an agent to build scenes in an output gltf global scene animation.
- Each input must generate in output (1 to 1 relationship)
- Inputs:
    - Contains:
        - Several gltf (provided as directories)
        - A textual prompt
    - Format: Single JSON file, eventually accompagned by the inputs gltf
- Outputs:
    - Copied input gltfs
    - An orchestrative animated gltf scene involving gltf inputs
- I want to create a scenarios/ directory, with scenarios/<scenario> element
- Each scenario contains an input/ (user-created) and output/ (process-created) dirs
- Each scenario feature one or more gltf characters, eventually fully-generated ones if the input text prompt asks it.

## Process Logics
- The main process is an agent using skills and tools.
- Tools:
    - What's needed to visualize a picture of a gltf https://github.com/M4jor-Tom/gltf_to_png.py
    - What's needed to visualize a video of a gltf https://github.com/M4jor-Tom/gltf_to_webm.py

## Agent Dispatch
- Tasks can be executed by a remote agent (configured via agent URL, agent name, and agent key)
- If no remote agent is configured, fall back to the local `opencode` CLI installation
- The flake provides `opencode` as a dependency, available on PATH at runtime
- Agent configuration is passed via CLI flags: `--agent-url`, `--agent-name`, `--agent-key`
- When using local `opencode`, it runs as a subprocess and uses its own provider/model configuration

## Test command
- Run tests with: `nix run .#test`
- Run a specific file: `nix run .#test -- src/llm_client_test.py -v`

## Prompt execution
- From that prompt, create a PRD using https://github.com/snarktank/ai-dev-tasks/blob/main/create-prd.md
- Ask me every subject that needs to be clarified
