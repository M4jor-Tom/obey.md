import json
import os
import tempfile

from src.manifest import parse_manifest, validate_manifest


def test_validate_manifest_valid():
    data = {
        "prompt": "A knight fights a dragon",
        "characters": [
            {"name": "knight", "gltf_ref": "knight.gltf"},
            {"name": "dragon", "gltf_ref": "dragon.gltf"},
        ],
        "duration_seconds": 30.0,
    }
    errors = validate_manifest(data)
    assert errors == []


def test_validate_manifest_missing_prompt():
    data = {"characters": [{"name": "k", "gltf_ref": "k.gltf"}]}
    errors = validate_manifest(data)
    assert any("prompt" in e for e in errors)


def test_validate_manifest_empty_characters():
    data = {"prompt": "test", "characters": []}
    errors = validate_manifest(data)
    assert any("characters" in e for e in errors)


def test_validate_manifest_gltf_ref_missing_dot_gltf():
    data = {
        "prompt": "test",
        "characters": [{"name": "k", "gltf_ref": "knight"}],
    }
    errors = validate_manifest(data)
    assert any("gltf_ref" in e and ".gltf" in e for e in errors)


def test_validate_manifest_invalid_duration():
    data = {
        "prompt": "test",
        "characters": [{"name": "k", "gltf_ref": "k.gltf"}],
        "duration_seconds": -5,
    }
    errors = validate_manifest(data)
    assert any("duration" in e for e in errors)


def test_parse_manifest_valid_file():
    data = {
        "prompt": "A test scene",
        "characters": [{"name": "robot", "gltf_ref": "robot.gltf"}],
    }
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(data, f)
        path = f.name
    try:
        manifest = parse_manifest(path)
        assert manifest.prompt == "A test scene"
        assert len(manifest.characters) == 1
        assert manifest.characters[0].name == "robot"
    finally:
        os.unlink(path)


def test_parse_manifest_file_not_found():
    try:
        parse_manifest("/nonexistent/manifest.json")
        assert False, "Expected FileNotFoundError"
    except FileNotFoundError:
        pass


def test_parse_manifest_invalid_json():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        f.write("{invalid json}")
        path = f.name
    try:
        try:
            parse_manifest(path)
            assert False, "Expected exception"
        except (json.JSONDecodeError, ValueError):
            pass
    finally:
        os.unlink(path)
