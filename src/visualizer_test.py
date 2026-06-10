import os
import tempfile

from src.visualizer import check_tools_available


def test_check_tools_available():
    available = check_tools_available()
    assert "gltf_to_png" in available
    assert "gltf_to_webm" in available
    assert isinstance(available["gltf_to_png"], bool)
