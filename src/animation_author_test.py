import json

from src.animation_author import (
    build_authoring_prompt,
    add_translation_keyframe,
    add_rotation_keyframe,
    add_scale_keyframe,
    add_morph_weight_track,
    apply_animation_to_scene,
    detect_missing_capabilities,
)


def test_build_authoring_prompt():
    scene = {"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []}
    prompt = build_authoring_prompt(scene, json.dumps({"prompt": "test"}), json.dumps({"iteration": 0}))
    assert "GLTF" in prompt
    assert "scene_gltf_json" not in prompt
    assert json.dumps(scene, indent=2) in prompt


def test_add_translation_keyframe():
    scene = {"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []}
    result = add_translation_keyframe(scene, 0, 0.0, [1.0, 2.0, 3.0])
    assert len(result["animations"]) > 0
    assert len(result["animations"][0]["channels"]) == 1
    assert result["animations"][0]["channels"][0]["target"]["path"] == "translation"


def test_add_rotation_keyframe():
    scene = {"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []}
    result = add_rotation_keyframe(scene, 0, 0.5, [0.0, 0.0, 0.0, 1.0])
    assert result["animations"][0]["channels"][0]["target"]["path"] == "rotation"


def test_add_scale_keyframe():
    scene = {"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []}
    result = add_scale_keyframe(scene, 0, 1.0, [2.0, 2.0, 2.0])
    assert result["animations"][0]["channels"][0]["target"]["path"] == "scale"


def test_add_morph_weight_track():
    scene = {"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []}
    keyframes = [
        {"time": 0.0, "weights": [0.0, 1.0]},
        {"time": 1.0, "weights": [1.0, 0.0]},
    ]
    result = add_morph_weight_track(scene, 0, keyframes)
    assert result["animations"][0]["channels"][0]["target"]["path"] == "weights"


def test_apply_animation_to_scene():
    scene = {"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []}
    anim_data = {
        "animations": [
            {
                "channels": [{"sampler": 0, "target": {"node": 0, "path": "translation"}}],
                "samplers": [{"input": 0, "output": 1, "interpolation": "LINEAR"}],
            }
        ],
        "accessors": [],
        "bufferViews": [],
        "buffers": [],
    }
    result = apply_animation_to_scene(scene, anim_data)
    assert len(result["animations"]) == 1


def test_apply_animation_replace():
    scene = {"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [],
             "animations": [{"name": "old"}]}
    anim_data = {"animations": [{"name": "new"}], "replace_existing": True}
    result = apply_animation_to_scene(scene, anim_data)
    assert result["animations"] == [{"name": "new"}]


def test_detect_missing_capabilities_no_morphs():
    gltf = {"meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}], "nodes": []}
    missing = detect_missing_capabilities(gltf)
    caps = [m["capability"] for m in missing]
    assert "morph_targets" in caps


def test_detect_missing_capabilities_no_skeleton():
    gltf = {"meshes": [], "nodes": [{"name": "root"}]}
    missing = detect_missing_capabilities(gltf)
    caps = [m["capability"] for m in missing]
    assert "skeletal_animation" in caps
