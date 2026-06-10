from __future__ import annotations

import json
import os

from src.models import Manifest, CharacterConfig


def parse_manifest(path: str) -> Manifest:
    if not os.path.exists(path):
        raise FileNotFoundError(f"Manifest not found: {path}")
    with open(path, "r") as f:
        data = json.load(f)
    errors = validate_manifest(data)
    if errors:
        raise ValueError(f"Manifest validation failed: {'; '.join(errors)}")
    return Manifest.from_dict(data)


def validate_manifest(data: dict) -> list[str]:
    errors = []
    if "prompt" not in data or not isinstance(data.get("prompt"), str) or not data["prompt"].strip():
        errors.append("Missing or empty 'prompt' field")
    if "characters" not in data or not isinstance(data.get("characters"), list) or len(data["characters"]) == 0:
        errors.append("Missing or empty 'characters' list")
    else:
        for i, c in enumerate(data["characters"]):
            if not isinstance(c, dict):
                errors.append(f"characters[{i}]: expected object")
                continue
            if "name" not in c or not isinstance(c["name"], str) or not c["name"].strip():
                errors.append(f"characters[{i}]: missing or invalid 'name'")
            if "gltf_ref" not in c or not isinstance(c["gltf_ref"], str) or not c["gltf_ref"].strip():
                errors.append(f"characters[{i}]: missing or invalid 'gltf_ref'")
            elif ".gltf" not in c["gltf_ref"]:
                errors.append(f"characters[{i}]: 'gltf_ref' must contain '.gltf'")
    if "duration_seconds" in data:
        dur = data["duration_seconds"]
        if not isinstance(dur, (int, float)) or dur <= 0:
            errors.append("'duration_seconds' must be a positive number")
    return errors
