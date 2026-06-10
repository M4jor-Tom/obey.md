from src.context_manager import estimate_tokens, trim_iteration_history, summarize_prior_state


def test_estimate_tokens_dict():
    data = {"key": "value"}
    assert estimate_tokens(data) > 0


def test_estimate_tokens_string():
    assert estimate_tokens("hello world") > 0


def test_trim_iteration_history_under_limit():
    history = [{"n": 1}, {"n": 2}, {"n": 3}]
    trimmed = trim_iteration_history(history, 1000000)
    assert trimmed == history


def test_trim_iteration_history_over_limit():
    history = [{"data": "x" * 1000}] * 100
    trimmed = trim_iteration_history(history, 1000)
    assert len(trimmed) < len(history)


def test_summarize_prior_state():
    state = {"iteration_number": 5, "critique": "Needs more action", "convergence_decision": "continue"}
    summary = summarize_prior_state(state)
    assert "Iteration 5" in summary
    assert "continue" in summary
