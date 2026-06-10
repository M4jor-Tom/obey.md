from __future__ import annotations


def validate_gltf_json(gltf_dict: dict) -> list[str]:
    errors = []
    if "asset" not in gltf_dict:
        errors.append("Missing 'asset' field")
    elif "version" not in gltf_dict["asset"]:
        errors.append("Missing 'asset.version'")
    if "scenes" not in gltf_dict or not isinstance(gltf_dict.get("scenes"), list):
        errors.append("Missing or invalid 'scenes'")
    elif len(gltf_dict["scenes"]) == 0:
        errors.append("'scenes' array is empty")
    if "scene" not in gltf_dict:
        errors.append("Missing 'scene' index")
    if "nodes" not in gltf_dict or not isinstance(gltf_dict.get("nodes"), list):
        errors.append("Missing or invalid 'nodes'")
    if "animations" in gltf_dict:
        if not isinstance(gltf_dict["animations"], list):
            errors.append("'animations' must be an array")
        else:
            for i, anim in enumerate(gltf_dict["animations"]):
                anim_errs = _validate_animation(anim, i)
                errors.extend(anim_errs)
    return errors


def _validate_animation(anim: dict, index: int) -> list[str]:
    errors = []
    if "channels" not in anim or not isinstance(anim.get("channels"), list):
        errors.append(f"animations[{index}]: missing or invalid 'channels'")
    if "samplers" not in anim or not isinstance(anim.get("samplers"), list):
        errors.append(f"animations[{index}]: missing or invalid 'samplers'")
    channels = anim.get("channels", [])
    for ci, ch in enumerate(channels):
        if not isinstance(ch, dict):
            errors.append(f"animations[{index}].channels[{ci}]: expected object")
            continue
        if "sampler" not in ch or not isinstance(ch["sampler"], int):
            errors.append(f"animations[{index}].channels[{ci}]: missing or invalid 'sampler'")
        if "target" not in ch or not isinstance(ch["target"], dict):
            errors.append(f"animations[{index}].channels[{ci}]: missing or invalid 'target'")
        else:
            if "node" not in ch["target"]:
                errors.append(f"animations[{index}].channels[{ci}].target: missing 'node'")
            if "path" not in ch["target"]:
                errors.append(f"animations[{index}].channels[{ci}].target: missing 'path'")
    samplers = anim.get("samplers", [])
    for si, s in enumerate(samplers):
        if not isinstance(s, dict):
            errors.append(f"animations[{index}].samplers[{si}]: expected object")
            continue
        if "input" not in s or not isinstance(s["input"], int):
            errors.append(f"animations[{index}].samplers[{si}]: missing or invalid 'input'")
        if "output" not in s or not isinstance(s["output"], int):
            errors.append(f"animations[{index}].samplers[{si}]: missing or invalid 'output'")
    return errors


def validate_animations(gltf_dict: dict) -> list[str]:
    return validate_gltf_json(gltf_dict)
