from __future__ import annotations

import os
import shutil
from pathlib import Path

from src.models import Manifest


def resolve_local(source_dir: str, dest_dir: str) -> str:
    if not os.path.isdir(source_dir):
        raise FileNotFoundError(f"Source directory not found: {source_dir}")

    dest = Path(dest_dir)
    shutil.copytree(source_dir, str(dest), dirs_exist_ok=True)

    gltf_files = [f for f in os.listdir(source_dir) if f.endswith(".gltf")]
    if not gltf_files:
        raise FileNotFoundError(f"No .gltf file found in {source_dir}")

    return str(dest / gltf_files[0])


def resolve_all(
    manifest: Manifest,
    input_dir: str,
    output_dir: str,
) -> dict[str, str]:
    resolved: dict[str, str] = {}
    for char in manifest.characters:
        source_dir = os.path.join(input_dir, char.gltf_ref)
        dest_dir = os.path.join(output_dir, char.gltf_ref)
        resolved_path = resolve_local(source_dir, dest_dir)
        resolved[char.name] = resolved_path
    return resolved
