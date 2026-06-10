from __future__ import annotations

import json
import os
import subprocess
import tempfile
from typing import Optional

from loguru import logger
from openai import OpenAI

_prompted_config: dict[str, str] = {}


def _has_remote_config() -> bool:
    key_set = bool(os.environ.get("OPENAI_API_KEY"))
    url_set = bool(os.environ.get("OPENAI_BASE_URL"))
    logger.debug("Remote config: key={}, base_url={}", key_set, url_set)
    return key_set or url_set


def _ensure_config() -> None:
    if _prompted_config.get("_done"):
        return

    if not _has_remote_config():
        _prompted_config["_done"] = True
        return

    api_key = os.environ.get("OPENAI_API_KEY")
    base_url = os.environ.get("OPENAI_BASE_URL")
    model = os.environ.get("OPENAI_MODEL")

    if not api_key:
        api_key = input("OpenAI API key (press Enter to skip if using a local provider): ").strip()
        _prompted_config["api_key"] = api_key

    if not base_url:
        base_url = input("API base URL (press Enter for OpenAI default): ").strip()
        _prompted_config["base_url"] = base_url

    if not model:
        model_raw = input("Model name (default: gpt-4o): ").strip()
        model = model_raw or "gpt-4o"
        _prompted_config["model"] = model

    _prompted_config["_done"] = True


def _get_client() -> OpenAI:
    _ensure_config()
    api_key = os.environ.get("OPENAI_API_KEY") or _prompted_config.get("api_key")
    base_url = os.environ.get("OPENAI_BASE_URL") or _prompted_config.get("base_url")
    if not api_key and not base_url:
        raise ValueError("OPENAI_API_KEY or OPENAI_BASE_URL must be set")
    kwargs = {}
    if api_key:
        kwargs["api_key"] = api_key
    if base_url:
        kwargs["base_url"] = base_url
    return OpenAI(**kwargs)


def _get_model() -> str:
    _ensure_config()
    return os.environ.get("OPENAI_MODEL") or _prompted_config.get("model", "gpt-4o")


def _call_opencode(
    prompt: str,
    system_prompt: Optional[str] = None,
    images: Optional[list[bytes]] = None,
) -> str:
    full_prompt = prompt
    if system_prompt:
        full_prompt = f"<system>\n{system_prompt}\n</system>\n\n{prompt}"

    logger.info("Using opencode subprocess backend")
    cmd = [
        "opencode", "run",
        "--format", "json",
        "--dangerously-skip-permissions",
    ]

    if images:
        tmp_dir = tempfile.mkdtemp()
        for i, img_bytes in enumerate(images):
            path = os.path.join(tmp_dir, f"frame_{i}.png")
            with open(path, "wb") as f:
                f.write(img_bytes)
            cmd.extend(["--file", path])
        logger.debug("Attached {} image(s) to opencode call", len(images))

    proc = subprocess.run(
        cmd,
        input=full_prompt,
        capture_output=True,
        text=True,
        timeout=600,
    )

    if images:
        import shutil
        shutil.rmtree(tmp_dir, ignore_errors=True)

    if proc.returncode != 0:
        raise RuntimeError(f"opencode run failed (exit {proc.returncode}): {proc.stderr}")

    parts: list[str] = []
    for line in proc.stdout.strip().splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            event = json.loads(line)
            if event.get("type") == "text":
                parts.append(event["part"]["text"])
        except (json.JSONDecodeError, KeyError):
            continue

    result = "".join(parts)
    logger.debug("opencode response: {} chars", len(result))
    return result


def call_llm(
    prompt: str,
    system_prompt: Optional[str] = None,
    images: Optional[list[bytes]] = None,
) -> str:
    if not _has_remote_config():
        return _call_opencode(prompt, system_prompt, images)

    client = _get_client()
    model = _get_model()
    logger.info("LLM call: backend=remote, model={}", model)

    messages: list[dict] = []
    if system_prompt:
        messages.append({"role": "system", "content": system_prompt})

    content: list[dict] = [{"type": "text", "text": prompt}]
    if images:
        import base64
        for img_bytes in images:
            b64 = base64.b64encode(img_bytes).decode()
            content.append({
                "type": "image_url",
                "image_url": {"url": f"data:image/png;base64,{b64}"},
            })
    messages.append({"role": "user", "content": content})

    response = client.chat.completions.create(
        model=model,
        messages=messages,
        temperature=0.7,
    )
    result = response.choices[0].message.content or ""
    logger.debug("LLM response: {} chars", len(result))
    return result


def call_llm_json(
    prompt: str,
    system_prompt: Optional[str] = None,
    images: Optional[list[bytes]] = None,
) -> dict:
    text = call_llm(prompt, system_prompt, images)
    text = text.strip()
    if text.startswith("```"):
        text = text.split("\n", 1)[-1]
        text = text.rsplit("```", 1)[0]
    try:
        return json.loads(text.strip())
    except json.JSONDecodeError as e:
        logger.warning("JSON parse failed: {} — raw preview: {}...", e, text[:200])
        raise


def extract_video_frames(video_path: str, num_frames: int = 6) -> list[bytes]:
    import subprocess

    probe = subprocess.run(
        ["ffprobe", "-v", "error", "-select_streams", "v:0",
         "-show_entries", "stream=duration",
         "-of", "json", video_path],
        capture_output=True, text=True,
    )
    probe_data = json.loads(probe.stdout) if probe.returncode == 0 else {}
    duration = float(probe_data.get("streams", [{}])[0].get("duration", 5))

    logger.info("Extracting {} frames from {} (duration={}s)", num_frames, os.path.basename(video_path), duration)

    frames: list[bytes] = []
    for i in range(num_frames):
        t = (i + 1) * duration / (num_frames + 1)
        result = subprocess.run(
            ["ffmpeg", "-ss", str(t), "-i", video_path,
             "-vframes", "1", "-f", "image2pipe", "-vcodec", "png", "-"],
            capture_output=True,
        )
        if result.returncode == 0 and result.stdout:
            frames.append(result.stdout)

    logger.debug("Extracted {} frames successfully", len(frames))
    return frames
