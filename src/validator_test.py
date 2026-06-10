from src.validator import validate_gltf_json, validate_animations


def test_valid_minimal_scene():
    gltf = {
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": []}],
        "nodes": [],
    }
    assert validate_gltf_json(gltf) == []


def test_missing_asset():
    gltf = {"scene": 0, "scenes": [{"nodes": []}], "nodes": []}
    errors = validate_gltf_json(gltf)
    assert any("asset" in e for e in errors)


def test_missing_scenes():
    gltf = {"asset": {"version": "2.0"}, "scene": 0, "nodes": []}
    errors = validate_gltf_json(gltf)
    assert any("scenes" in e for e in errors)


def test_empty_scenes():
    gltf = {"asset": {"version": "2.0"}, "scene": 0, "scenes": [], "nodes": []}
    errors = validate_gltf_json(gltf)
    assert any("empty" in e for e in errors)


def test_valid_animation():
    gltf = {
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": []}],
        "nodes": [],
        "animations": [
            {
                "channels": [
                    {"sampler": 0, "target": {"node": 0, "path": "translation"}}
                ],
                "samplers": [
                    {"input": 0, "output": 1, "interpolation": "LINEAR"}
                ],
            }
        ],
    }
    assert validate_gltf_json(gltf) == []


def test_animation_missing_sampler():
    gltf = {
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": []}],
        "nodes": [],
        "animations": [
            {
                "channels": [{"target": {"node": 0, "path": "translation"}}],
                "samplers": [],
            }
        ],
    }
    errors = validate_gltf_json(gltf)
    assert any("sampler" in e for e in errors)


def test_validate_animations():
    gltf = {
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": []}],
        "nodes": [],
        "animations": [],
    }
    assert validate_animations(gltf) == validate_gltf_json(gltf)
