import os
import tempfile

from src.gltf_resolver import resolve_local, resolve_all
from src.models import Manifest, CharacterConfig


def _create_gltf_dir(base_dir: str, ref: str) -> str:
    dir_path = os.path.join(base_dir, ref)
    os.makedirs(dir_path, exist_ok=True)
    gltf_path = os.path.join(dir_path, ref)
    with open(gltf_path, "w") as f:
        f.write(f'{{"asset": {{"version": "2.0"}}, "file": "{ref}"}}')
    return dir_path


def test_resolve_local():
    with tempfile.TemporaryDirectory() as tmpdir:
        src_dir = _create_gltf_dir(tmpdir, "char.gltf")
        dest_dir = os.path.join(tmpdir, "output", "char.gltf")
        result = resolve_local(src_dir, dest_dir)
        expected = os.path.join(dest_dir, "char.gltf")
        assert result == expected
        assert os.path.exists(result)
        with open(result) as f:
            assert f.read() == '{"asset": {"version": "2.0"}, "file": "char.gltf"}'


def test_resolve_local_copies_all_files():
    with tempfile.TemporaryDirectory() as tmpdir:
        src_dir = os.path.join(tmpdir, "char.gltf")
        os.makedirs(src_dir)
        with open(os.path.join(src_dir, "char.gltf"), "w") as f:
            f.write("{}")
        with open(os.path.join(src_dir, "tex.png"), "w") as f:
            f.write("pngdata")
        with open(os.path.join(src_dir, "data.bin"), "wb") as f:
            f.write(b"\x00\x01\x02")

        dest_dir = os.path.join(tmpdir, "output", "char.gltf")
        resolve_local(src_dir, dest_dir)
        assert os.path.isfile(os.path.join(dest_dir, "char.gltf"))
        assert os.path.isfile(os.path.join(dest_dir, "tex.png"))
        assert os.path.isfile(os.path.join(dest_dir, "data.bin"))


def test_resolve_all_local():
    manifest = Manifest(
        prompt="test",
        characters=[
            CharacterConfig(name="a", gltf_ref="a.gltf"),
            CharacterConfig(name="b", gltf_ref="b.gltf"),
        ],
    )
    with tempfile.TemporaryDirectory() as tmpdir:
        input_dir = os.path.join(tmpdir, "input")
        output_dir = os.path.join(tmpdir, "output")
        os.makedirs(output_dir)
        for ref in ["a.gltf", "b.gltf"]:
            _create_gltf_dir(input_dir, ref)
        resolved = resolve_all(manifest, input_dir, output_dir)
        assert "a" in resolved
        assert "b" in resolved
        for name, path in resolved.items():
            assert os.path.exists(path), f"{path} does not exist"


def test_resolve_local_missing_source():
    with tempfile.TemporaryDirectory() as tmpdir:
        try:
            resolve_local(os.path.join(tmpdir, "nonexistent"), os.path.join(tmpdir, "out"))
            assert False, "Expected FileNotFoundError"
        except FileNotFoundError:
            pass


def test_resolve_local_no_gltf_file():
    with tempfile.TemporaryDirectory() as tmpdir:
        src_dir = os.path.join(tmpdir, "empty.gltf")
        os.makedirs(src_dir)
        with open(os.path.join(src_dir, "readme.txt"), "w") as f:
            f.write("no gltf here")
        try:
            resolve_local(src_dir, os.path.join(tmpdir, "out"))
            assert False, "Expected FileNotFoundError"
        except FileNotFoundError:
            pass


def test_resolve_all_missing_local():
    manifest = Manifest(
        prompt="test",
        characters=[
            CharacterConfig(name="x", gltf_ref="missing.gltf"),
        ],
    )
    with tempfile.TemporaryDirectory() as tmpdir:
        try:
            resolve_all(manifest, tmpdir, tmpdir)
            assert False, "Expected FileNotFoundError"
        except FileNotFoundError:
            pass
