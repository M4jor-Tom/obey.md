import os
import json
from unittest.mock import patch, MagicMock

from src.llm_client import (
    _has_remote_config,
    _call_opencode,
    call_llm,
    call_llm_json,
)


def cleanup_env():
    for k in ("OPENAI_API_KEY", "OPENAI_BASE_URL", "OPENAI_MODEL"):
        os.environ.pop(k, None)


def test_has_remote_config_false_when_no_env():
    cleanup_env()
    assert not _has_remote_config()


def test_has_remote_config_true_with_api_key():
    cleanup_env()
    os.environ["OPENAI_API_KEY"] = "sk-test"
    assert _has_remote_config()
    os.environ.pop("OPENAI_API_KEY")


def test_has_remote_config_true_with_base_url():
    cleanup_env()
    os.environ["OPENAI_BASE_URL"] = "http://localhost:8080"
    assert _has_remote_config()
    os.environ.pop("OPENAI_BASE_URL")


def test_has_remote_config_false_with_only_model():
    cleanup_env()
    os.environ["OPENAI_MODEL"] = "gpt-4"
    assert not _has_remote_config()
    os.environ.pop("OPENAI_MODEL")


@patch("src.llm_client.subprocess.run")
def test_call_opencode_parses_json_events(mock_run):
    mock_run.return_value = MagicMock(
        returncode=0,
        stdout=(
            '{"type":"step_start","timestamp":1,"part":{}}\n'
            '{"type":"text","timestamp":2,"part":{"id":"p1","type":"text","text":"hello-world"}}\n'
            '{"type":"step_finish","timestamp":3,"part":{"reason":"stop"}}\n'
        ),
        stderr="",
    )
    result = _call_opencode("test prompt")
    assert result == "hello-world"
    mock_run.assert_called_once()
    args, kwargs = mock_run.call_args
    assert kwargs["input"] == "test prompt"


@patch("src.llm_client.subprocess.run")
def test_call_opencode_with_system_prompt(mock_run):
    mock_run.return_value = MagicMock(
        returncode=0,
        stdout='{"type":"text","timestamp":1,"part":{"id":"p1","type":"text","text":"ok"}}\n',
        stderr="",
    )
    result = _call_opencode("prompt", system_prompt="sys")
    assert result == "ok"
    args, kwargs = mock_run.call_args
    assert "<system>" in kwargs["input"]
    assert "sys" in kwargs["input"]
    assert "prompt" in kwargs["input"]


@patch("src.llm_client.subprocess.run")
def test_call_opencode_multiple_text_events_concatenated(mock_run):
    mock_run.return_value = MagicMock(
        returncode=0,
        stdout=(
            '{"type":"text","timestamp":1,"part":{"id":"p1","type":"text","text":"hello"}}\n'
            '{"type":"text","timestamp":2,"part":{"id":"p2","type":"text","text":" world"}}\n'
        ),
        stderr="",
    )
    result = _call_opencode("test")
    assert result == "hello world"


@patch("src.llm_client.subprocess.run")
def test_call_opencode_nonzero_exit(mock_run):
    mock_run.return_value = MagicMock(
        returncode=1,
        stdout="",
        stderr="something failed",
    )
    try:
        _call_opencode("test")
        assert False, "Expected RuntimeError"
    except RuntimeError as e:
        assert "opencode run failed" in str(e)
        assert "something failed" in str(e)


@patch("src.llm_client.subprocess.run")
def test_call_opencode_with_images(mock_run):
    mock_run.return_value = MagicMock(
        returncode=0,
        stdout='{"type":"text","timestamp":1,"part":{"id":"p1","type":"text","text":"ok"}}\n',
        stderr="",
    )
    result = _call_opencode("test", images=[b"fake-png-data"])
    assert result == "ok"


@patch("src.llm_client._call_opencode")
def test_call_llm_falls_back_to_opencode(mock_opencode):
    cleanup_env()
    mock_opencode.return_value = "fallback-response"
    result = call_llm("test prompt", system_prompt="be helpful")
    assert result == "fallback-response"
    mock_opencode.assert_called_once_with("test prompt", "be helpful", None)


@patch("src.llm_client._call_opencode")
def test_call_llm_json_parses_opencode_response(mock_opencode):
    cleanup_env()
    mock_opencode.return_value = '{"key": "value"}'
    result = call_llm_json("test prompt")
    assert result == {"key": "value"}


@patch("src.llm_client._call_opencode")
def test_call_llm_json_strips_markdown_fences(mock_opencode):
    cleanup_env()
    mock_opencode.return_value = '```json\n{"key": "value"}\n```'
    result = call_llm_json("test prompt")
    assert result == {"key": "value"}
