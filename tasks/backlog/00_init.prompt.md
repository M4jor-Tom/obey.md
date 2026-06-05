## Inputs and Outputs
- I want to prompt scenarios (.md files) for an agent to build scenes in an output gltf global scene animation.
- Each input must generate in output (1 to 1 relationship)
- Inputs:
    - Contains:
        - Several gltf (either a github url to sparse clone it, either the folders themselves)
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

## Prompt execution
- From that prompt, create a PRD using https://github.com/snarktank/ai-dev-tasks/blob/main/create-prd.md
- Ask me every subject that needs to be clarified
