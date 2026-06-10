from __future__ import annotations

import json
from typing import Any


def estimate_tokens(data: Any) -> int:
    text = json.dumps(data) if not isinstance(data, str) else data
    return len(text) // 4


def trim_iteration_history(history: list, max_tokens: int) -> list:
    trimmed = list(history)
    while trimmed:
        tok_count = sum(estimate_tokens(json.dumps(h.to_dict() if hasattr(h, 'to_dict') else h)) for h in trimmed)
        if tok_count <= max_tokens:
            break
        trimmed.pop(0)
    return trimmed


def summarize_prior_state(state: dict) -> str:
    iteration = state.get("iteration_number", "?")
    critique = state.get("critique", "")
    convergence = state.get("convergence_decision", "")
    summary = (
        f"[Iteration {iteration}] Critique: {critique[:200]}... "
        f"Decision: {convergence}"
    )
    return summary
