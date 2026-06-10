import json
import os
import tempfile

from src.orchestrator import Orchestrator
from src.models import Manifest, CharacterConfig
from src.scene_builder import build_empty_scene


def _create_minimal_manifest(tmpdir: str) -> str:
    manifest = {
        "prompt": "A simple test scene with two cubes",
        "characters": [
            {"name": "cube_a", "gltf_ref": "cube_a.gltf"},
            {"name": "cube_b", "gltf_ref": "cube_b.gltf"},
        ],
        "duration_seconds": 5.0,
    }
    manifest_path = os.path.join(tmpdir, "manifest.json")
    with open(manifest_path, "w") as f:
        json.dump(manifest, f)
    return manifest_path


def _create_cube_gltf(path: str):
    os.makedirs(path, exist_ok=True)
    gltf = {
        "asset": {"version": "2.0", "generator": "test"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"name": "cube", "mesh": 0}],
        "meshes": [{
            "primitives": [{"attributes": {"POSITION": 0}, "indices": 1}],
        }],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": 24, "type": "VEC3", "max": [1, 1, 1], "min": [-1, -1, -1]},
            {"bufferView": 1, "componentType": 5123, "count": 36, "type": "SCALAR", "max": [35], "min": [0]},
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 288, "target": 34962},
            {"buffer": 0, "byteOffset": 288, "byteLength": 72, "target": 34963},
        ],
        "buffers": [{"uri": "data:application/octet-stream;base64,AAAAAACAgIA", "byteLength": 360}],
    }
    gltf_file = os.path.join(path, os.path.basename(path))
    with open(gltf_file, "w") as f:
        json.dump(gltf, f)


def test_orchestrator_setup():
    with tempfile.TemporaryDirectory() as tmpdir:
        input_dir = os.path.join(tmpdir, "input")
        output_dir = os.path.join(tmpdir, "output")
        os.makedirs(input_dir)
        os.makedirs(output_dir)

        manifest_path = _create_minimal_manifest(tmpdir)
        _create_cube_gltf(os.path.join(input_dir, "cube_a.gltf"))
        _create_cube_gltf(os.path.join(input_dir, "cube_b.gltf"))

        orch = Orchestrator(manifest_path, input_dir, output_dir)
        scene = orch.setup()
        assert "asset" in scene
        assert len(scene["scenes"][0]["nodes"]) == 2


def test_orchestrator_run_iteration():
    with tempfile.TemporaryDirectory() as tmpdir:
        input_dir = os.path.join(tmpdir, "input")
        output_dir = os.path.join(tmpdir, "output")
        os.makedirs(input_dir)
        os.makedirs(output_dir)

        manifest_path = _create_minimal_manifest(tmpdir)
        _create_cube_gltf(os.path.join(input_dir, "cube_a.gltf"))
        _create_cube_gltf(os.path.join(input_dir, "cube_b.gltf"))

        orch = Orchestrator(manifest_path, input_dir, output_dir)
        scene = orch.setup()
        updated_scene, state = orch.run_iteration(scene, 0)
        assert state.iteration_number == 0
        assert state.convergence_decision is not None


def test_orchestrator_full_loop():
    with tempfile.TemporaryDirectory() as tmpdir:
        input_dir = os.path.join(tmpdir, "input")
        output_dir = os.path.join(tmpdir, "output")
        os.makedirs(input_dir)
        os.makedirs(output_dir)

        manifest_path = _create_minimal_manifest(tmpdir)
        _create_cube_gltf(os.path.join(input_dir, "cube_a.gltf"))
        _create_cube_gltf(os.path.join(input_dir, "cube_b.gltf"))

        orch = Orchestrator(manifest_path, input_dir, output_dir)
        scene = orch.run_full_loop()
        assert "asset" in scene
        assert orch.converged


def test_orchestrator_max_iterations():
    with tempfile.TemporaryDirectory() as tmpdir:
        input_dir = os.path.join(tmpdir, "input")
        output_dir = os.path.join(tmpdir, "output")
        os.makedirs(input_dir)
        os.makedirs(output_dir)

        manifest_path = _create_minimal_manifest(tmpdir)
        _create_cube_gltf(os.path.join(input_dir, "cube_a.gltf"))
        _create_cube_gltf(os.path.join(input_dir, "cube_b.gltf"))

        orch = Orchestrator(manifest_path, input_dir, output_dir)
        orch._decide_convergence = lambda c: "continue"
        scene = orch.run_full_loop()
        assert orch.iteration_count == orch.max_iterations
        assert not orch.converged
