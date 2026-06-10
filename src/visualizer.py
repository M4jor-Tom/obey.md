from __future__ import annotations

import os
import subprocess
import shutil

from loguru import logger


def _find_script(name: str) -> str:
    return shutil.which(name) or os.path.join(os.path.dirname(__file__), "..", "scripts", name)


def check_tools_available() -> dict[str, bool]:
    png_tool = shutil.which("gltf_to_png")
    webm_tool = shutil.which("gltf_to_webm")
    return {
        "gltf_to_png": bool(png_tool),
        "gltf_to_webm": bool(webm_tool),
    }


def generate_png_preview(scene_gltf_path: str, output_path: str) -> str:
    script = _find_script("gltf_to_png")
    logger.info("Rendering PNG: {} -> {}", os.path.basename(scene_gltf_path), os.path.basename(output_path))
    result = subprocess.run(
        [script, "-i", scene_gltf_path, "-o", output_path],
        capture_output=True, text=True, timeout=120,
    )
    if result.returncode != 0:
        raise RuntimeError(f"gltf_to_png failed: {result.stderr}")
    return output_path


def generate_webm_preview(scene_gltf_path: str, output_path: str) -> str:
    script = _find_script("gltf_to_webm")
    logger.info("Rendering WebM: {} -> {}", os.path.basename(scene_gltf_path), os.path.basename(output_path))
    result = subprocess.run(
        [script, "-i", scene_gltf_path, "-o", output_path],
        capture_output=True, text=True, timeout=300,
    )
    if result.returncode != 0:
        raise RuntimeError(f"gltf_to_webm failed: {result.stderr}")
    return output_path


def generate_intermediate_frames(scene_gltf_path: str, output_dir: str) -> list[str]:
    os.makedirs(output_dir, exist_ok=True)
    preview_path = os.path.join(output_dir, "intermediate_preview.webm")
    generate_webm_preview(scene_gltf_path, preview_path)
    return [preview_path]


def generate_final_previews(scene_path: str, output_dir: str) -> dict[str, str]:
    logger.info("Generating final previews in {}", output_dir)
    os.makedirs(output_dir, exist_ok=True)
    png_path = os.path.join(output_dir, "preview.png")
    webm_path = os.path.join(output_dir, "preview.webm")
    results = {}
    if check_tools_available().get("gltf_to_png"):
        try:
            results["png"] = generate_png_preview(scene_path, png_path)
        except RuntimeError:
            results["png"] = ""
    if check_tools_available().get("gltf_to_webm"):
        try:
            results["webm"] = generate_webm_preview(scene_path, webm_path)
        except RuntimeError:
            results["webm"] = ""
    return results
