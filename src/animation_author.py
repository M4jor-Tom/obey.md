from __future__ import annotations

import json
import copy
from typing import Optional

from loguru import logger


ANIMATION_AUTHORING_PROMPT_TEMPLATE = """
You are an expert GLTF animation engineer. Below is the current scene GLTF JSON and the user's animation prompt.

Your task is to produce a JSON object containing animation data that will be merged into the scene GLTF.

The animation data must follow this structure:
{{
  "animations": [
    {{
      "name": "<animation_name>",
      "channels": [
        {{
          "sampler": <sampler_index>,
          "target": {{
            "node": <node_index>,
            "path": "<translation|rotation|scale|weights>"
          }}
        }}
      ],
      "samplers": [
        {{
          "input": <accessor_index>,
          "output": <accessor_index>,
          "interpolation": "LINEAR"
        }}
      ]
    }}
  ],
  "accessors": [
    {{
      "bufferView": <bufferView_index>,
      "componentType": <FLOAT=5126>,
      "count": <count>,
      "type": "<VEC3|VEC4|SCALAR>",
      "max": [...],
      "min": [...]
    }}
  ],
  "bufferViews": [
    {{
      "buffer": 0,
      "byteOffset": <offset>,
      "byteLength": <length>,
      "target": <ARRAY_BUFFER=34962>
    }}
  ],
  "buffers": [
    {{
      "uri": "<data_uri_or_path>",
      "byteLength": <length>
    }}
  ],
  "replace_existing": <true|false>
}}

Rules:
- All characters animate concurrently on a single shared timeline.
- Use "translation", "rotation", "scale", and "weights" (morph target) paths as appropriate.
- Keyframe times should be in seconds, starting at 0.
- Do not exceed 10,000 keyframes per channel.
- Scene duration should be inferred from the prompt or use {duration_seconds}s.
- Ensure all accessor indices and bufferView indices are sequential and valid.

Current scene GLTF:
{scene_gltf_json}

Character manifest:
{manifest_json}

Iteration state (previous critiques, current progress):
{iteration_state_json}

Respond with ONLY the valid JSON animation data. No markdown fences, no explanation.
"""


def build_authoring_prompt(
    scene_gltf: dict,
    manifest_json: str,
    iteration_state_json: str,
    duration_seconds: float = 10.0,
) -> str:
    prompt = ANIMATION_AUTHORING_PROMPT_TEMPLATE.format(
        scene_gltf_json=json.dumps(scene_gltf, indent=2),
        manifest_json=manifest_json,
        iteration_state_json=iteration_state_json,
        duration_seconds=duration_seconds,
    )
    logger.debug("Built authoring prompt ({} chars), duration={}s", len(prompt), duration_seconds)
    return prompt


def add_translation_keyframe(
    scene_gltf: dict,
    node_index: int,
    time: float,
    value: list[float],
    animation_index: int = 0,
) -> dict:
    return _add_trs_keyframe(scene_gltf, node_index, "translation", time, value, animation_index)


def add_rotation_keyframe(
    scene_gltf: dict,
    node_index: int,
    time: float,
    value: list[float],
    animation_index: int = 0,
) -> dict:
    return _add_trs_keyframe(scene_gltf, node_index, "rotation", time, value, animation_index)


def add_scale_keyframe(
    scene_gltf: dict,
    node_index: int,
    time: float,
    value: list[float],
    animation_index: int = 0,
) -> dict:
    return _add_trs_keyframe(scene_gltf, node_index, "scale", time, value, animation_index)


def _add_trs_keyframe(
    scene_gltf: dict,
    node_index: int,
    path: str,
    time: float,
    value: list[float],
    animation_index: int,
) -> dict:
    scene = copy.deepcopy(scene_gltf)
    _ensure_animation_structures(scene)
    if not scene["animations"]:
        scene["animations"].append({"name": "animation_0", "channels": [], "samplers": []})
    anim = scene["animations"][animation_index] if animation_index < len(scene["animations"]) else scene["animations"][-1]

    anim["channels"].append({
        "sampler": len(anim["samplers"]),
        "target": {"node": node_index, "path": path},
    })
    anim["samplers"].append({
        "input": _next_accessor_index(scene),
        "output": _next_accessor_index(scene) + 1,
        "interpolation": "LINEAR",
    })
    time_buffer = _float32_to_bytes([time])
    value_buffer = _float32_to_bytes(value)
    vec_type = "VEC3" if len(value) == 3 else "VEC4" if len(value) == 4 else "SCALAR"
    comp_count = len(value)

    _add_buffer_view(scene, time_buffer)
    _add_accessor(scene, len(scene["bufferViews"]) - 1, 5126, 1, "SCALAR", [time], [time])
    _add_buffer_view(scene, value_buffer)
    _add_accessor(scene, len(scene["bufferViews"]) - 1, 5126, comp_count, vec_type, value, value)

    return scene


def add_morph_weight_track(
    scene_gltf: dict,
    node_index: int,
    weight_keyframes: list[dict],
    animation_index: int = 0,
) -> dict:
    scene = copy.deepcopy(scene_gltf)
    _ensure_animation_structures(scene)
    if not scene["animations"]:
        scene["animations"].append({"name": "animation_0", "channels": [], "samplers": []})
    anim = scene["animations"][animation_index] if animation_index < len(scene["animations"]) else scene["animations"][-1]

    times = [kf["time"] for kf in weight_keyframes]
    weights = []
    for kf in weight_keyframes:
        weights.extend(kf.get("weights", [0.0]))

    anim["channels"].append({
        "sampler": len(anim["samplers"]),
        "target": {"node": node_index, "path": "weights"},
    })
    anim["samplers"].append({
        "input": _next_accessor_index(scene),
        "output": _next_accessor_index(scene) + 1,
        "interpolation": "LINEAR",
    })

    time_buffer = _float32_to_bytes(times)
    weights_buffer = _float32_to_bytes(weights)

    _add_buffer_view(scene, time_buffer)
    _add_accessor(scene, len(scene["bufferViews"]) - 1, 5126, len(times), "SCALAR", [max(times)], [min(times)])

    _add_buffer_view(scene, weights_buffer)
    _add_accessor(scene, len(scene["bufferViews"]) - 1, 5126, len(weights), "SCALAR",
                  [max(weights)] if weights else [0], [min(weights)] if weights else [0])

    return scene


def apply_animation_to_scene(scene_gltf: dict, animation_data: dict) -> dict:
    scene = copy.deepcopy(scene_gltf)
    _ensure_animation_structures(scene)

    n_anims = 0
    replace = animation_data.get("replace_existing", False)
    if "animations" in animation_data:
        n_anims = len(animation_data["animations"])
        if replace:
            scene["animations"] = []
        scene["animations"].extend(animation_data["animations"])

    n_acc = len(animation_data.get("accessors", []))
    n_bv = len(animation_data.get("bufferViews", []))
    n_buf = len(animation_data.get("buffers", []))
    for key in ("accessors", "bufferViews", "buffers"):
        if key in animation_data:
            scene.setdefault(key, [])
            scene[key].extend(animation_data[key])

    logger.info("Applied {} animations (replace={}), +{} accessors, +{} bufferViews, +{} buffers", n_anims, replace, n_acc, n_bv, n_buf)
    return scene


def detect_missing_capabilities(character_gltf: dict) -> list[dict]:
    missing = []
    has_morph_targets = False
    has_skeletons = False

    for mesh in character_gltf.get("meshes", []):
        for prim in mesh.get("primitives", []):
            if "targets" in prim or "morphTargets" in prim:
                has_morph_targets = True
        if "weights" in mesh:
            has_morph_targets = True

    for node in character_gltf.get("nodes", []):
        if "skin" in node:
            has_skeletons = True

    if not has_morph_targets:
        missing.append({
            "capability": "morph_targets",
            "fallback": "Use TRS keyframes (rotation/scale) to approximate morph target effects",
        })
    if not has_skeletons:
        missing.append({
            "capability": "skeletal_animation",
            "fallback": "Use node TRS keyframes for animation instead of bones",
        })

    return missing


def _ensure_animation_structures(scene: dict) -> None:
    if "animations" not in scene or not isinstance(scene["animations"], list):
        scene["animations"] = []
    if "accessors" not in scene:
        scene["accessors"] = []
    if "bufferViews" not in scene:
        scene["bufferViews"] = []
    if "buffers" not in scene:
        scene["buffers"] = []


def _next_accessor_index(scene: dict) -> int:
    return len(scene.get("accessors", []))


def _add_buffer_view(scene: dict, data: bytes) -> int:
    bv_index = len(scene.get("bufferViews", []))
    buf_index = _ensure_buffer(scene, data)
    scene.setdefault("bufferViews", []).append({
        "buffer": buf_index,
        "byteOffset": _buffer_offset(scene, buf_index),
        "byteLength": len(data),
        "target": 34962,
    })
    return bv_index


def _buffer_offset(scene: dict, buffer_index: int) -> int:
    offset = 0
    for bv in scene.get("bufferViews", []):
        if bv["buffer"] == buffer_index:
            offset = max(offset, bv["byteOffset"] + bv["byteLength"])
    return offset


def _ensure_buffer(scene: dict, data: bytes) -> int:
    import base64
    uri = "data:application/octet-stream;base64," + base64.b64encode(data).decode()
    scene.setdefault("buffers", []).append({
        "uri": uri,
        "byteLength": len(data),
    })
    return len(scene["buffers"]) - 1


def _add_accessor(scene: dict, buffer_view_index: int, component_type: int, count: int,
                  type_str: str, max_vals: list[float], min_vals: list[float]) -> int:
    idx = len(scene.get("accessors", []))
    scene.setdefault("accessors", []).append({
        "bufferView": buffer_view_index,
        "byteOffset": 0,
        "componentType": component_type,
        "count": count,
        "type": type_str,
        "max": max_vals,
        "min": min_vals,
    })
    return idx


def _float32_to_bytes(values: list[float]) -> bytes:
    import struct
    return struct.pack(f"<{len(values)}f", *values)
