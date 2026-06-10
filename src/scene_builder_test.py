import json
import os
import tempfile

from src.scene_builder import build_empty_scene, add_character_node, build_root_scene
from src.models import Manifest, CharacterConfig


def test_build_empty_scene():
    scene = build_empty_scene()
    assert scene["asset"]["version"] == "2.0"
    assert scene["scenes"][0]["nodes"] == []
    assert scene["nodes"] == []
    assert scene["animations"] == []


def test_add_character_node():
    scene = build_empty_scene()
    idx = add_character_node(scene, "knight", "knight.gltf")
    assert idx == 0
    assert scene["nodes"][0]["name"] == "knight"
    assert 0 in scene["scenes"][0]["nodes"]


def test_add_character_node_with_mesh():
    scene = build_empty_scene()
    idx = add_character_node(scene, "robot", "robot.gltf", mesh_index=2)
    assert scene["nodes"][0]["mesh"] == 2


def _minimal_char_gltf() -> dict:
    return {
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"name": "root", "mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "max": [1, 1, 1], "min": [-1, -1, -1]}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962}],
        "buffers": [{"uri": "data:application/octet-stream;base64,AAAAAA==", "byteLength": 36}],
        "extensionsUsed": ["KHR_draco_mesh_compression"],
    }


def _create_gltf_file(base_dir: str, ref: str, content: dict) -> str:
    dir_path = os.path.join(base_dir, ref)
    os.makedirs(dir_path, exist_ok=True)
    file_path = os.path.join(dir_path, ref)
    with open(file_path, "w") as f:
        json.dump(content, f)
    return file_path


def test_build_root_scene():
    manifest = Manifest(
        prompt="test scene",
        characters=[
            CharacterConfig(name="char_a", gltf_ref="a.gltf"),
            CharacterConfig(name="char_b", gltf_ref="b.gltf"),
        ],
    )
    with tempfile.TemporaryDirectory() as tmpdir:
        for ref in ["a.gltf", "b.gltf"]:
            _create_gltf_file(tmpdir, ref, _minimal_char_gltf())

        scene = build_root_scene(manifest, tmpdir)
        assert len(scene["scenes"][0]["nodes"]) == 2
        wrapper_indices = scene["scenes"][0]["nodes"]
        assert scene["nodes"][wrapper_indices[0]]["name"] == "char_a"
        assert scene["nodes"][wrapper_indices[1]]["name"] == "char_b"
        assert "extensionsUsed" in scene
        assert len(scene["meshes"]) >= 2


def test_build_root_scene_merge_remaps_indices():
    char_gltf = _minimal_char_gltf()
    manifest = Manifest(
        prompt="merge test",
        characters=[
            CharacterConfig(name="char_a", gltf_ref="a.gltf"),
        ],
    )
    with tempfile.TemporaryDirectory() as tmpdir:
        _create_gltf_file(tmpdir, "a.gltf", char_gltf)

        scene = build_root_scene(manifest, tmpdir)
        assert len(scene["nodes"]) == 2
        inner_node = scene["nodes"][0]
        assert inner_node["name"] == "root"
        assert inner_node["mesh"] == 0
        char_root = scene["nodes"][1]
        assert char_root["name"] == "char_a"
        assert "children" in char_root
        assert char_root["children"] == [0]
        assert len(scene["meshes"]) == 1


def test_build_root_scene_merge_multiple_characters():
    manifest = Manifest(
        prompt="multi merge test",
        characters=[
            CharacterConfig(name="hero", gltf_ref="a.gltf"),
            CharacterConfig(name="villain", gltf_ref="b.gltf"),
        ],
    )
    with tempfile.TemporaryDirectory() as tmpdir:
        for ref in ["a.gltf", "b.gltf"]:
            _create_gltf_file(tmpdir, ref, _minimal_char_gltf())

        scene = build_root_scene(manifest, tmpdir)
        assert len(scene["scenes"][0]["nodes"]) == 2
        wrapper_indices = scene["scenes"][0]["nodes"]
        assert scene["nodes"][wrapper_indices[0]]["name"] == "hero"
        assert scene["nodes"][wrapper_indices[1]]["name"] == "villain"
        assert len(scene["nodes"]) == 4
        assert len(scene["meshes"]) == 2
        assert len(scene["buffers"]) == 2


def test_build_root_scene_with_missing_files():
    manifest = Manifest(
        prompt="test",
        characters=[
            CharacterConfig(name="ghost", gltf_ref="nonexistent.gltf"),
        ],
    )
    with tempfile.TemporaryDirectory() as tmpdir:
        scene = build_root_scene(manifest, tmpdir)
        assert len(scene["nodes"]) == 1
        assert scene["nodes"][0]["name"] == "ghost"
