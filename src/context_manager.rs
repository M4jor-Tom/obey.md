use serde_json::Value;

pub fn estimate_tokens(data: &Value) -> i32 {
    let text = serde_json::to_string(data).unwrap_or_default();
    (text.len() / 4) as i32
}

pub fn estimate_tokens_str(data: &str) -> i32 {
    (data.len() / 4) as i32
}

pub fn trim_iteration_history<T, F>(history: &[T], max_tokens: i32, to_dict: F) -> Vec<T>
where
    T: Clone,
    F: Fn(&T) -> Value,
{
    let mut trimmed: Vec<T> = history.to_vec();
    loop {
        let tok_count: i32 = trimmed
            .iter()
            .map(|h| {
                let d = to_dict(h);
                estimate_tokens(&d)
            })
            .sum();
        if tok_count <= max_tokens || trimmed.is_empty() {
            break;
        }
        trimmed.remove(0);
    }
    let removed = history.len() - trimmed.len();
    if removed > 0 {
        log::info!(
            "Trimmed history: {} -> {} entries (budget: {} tokens)",
            history.len(),
            trimmed.len(),
            max_tokens
        );
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_estimate_tokens() {
        let data = json!({"key": "value"});
        assert!(estimate_tokens(&data) > 0);
    }

    #[test]
    fn test_estimate_tokens_str() {
        assert!(estimate_tokens_str("hello world") > 0);
    }

    #[test]
    fn test_trim_iteration_history_under_limit() {
        let history = vec![json!({"n": 1}), json!({"n": 2}), json!({"n": 3})];
        let trimmed = trim_iteration_history(&history, 1000000, |v| v.clone());
        assert_eq!(trimmed.len(), 3);
    }

    #[test]
    fn test_trim_iteration_history_over_limit() {
        let history: Vec<Value> = (0..100).map(|_| json!({"data": "x".repeat(1000)})).collect();
        let trimmed = trim_iteration_history(&history, 1000, |v| v.clone());
        assert!(trimmed.len() < history.len());
    }

    #[test]
    fn test_summarize_prior_state() {
        let state = json!({
            "iteration_number": 5,
            "critique": "Needs more action",
            "convergence_decision": "continue",
        });
        let summary = summarize_prior_state(&state);
        assert!(summary.contains("Iteration 5"));
        assert!(summary.contains("continue"));
    }
}

pub fn summarize_prior_state(state: &Value) -> String {
    let iteration = state["iteration_number"]
        .as_i64()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "?".to_string());
    let critique = state["critique"]
        .as_str()
        .unwrap_or("");
    let convergence = state["convergence_decision"]
        .as_str()
        .unwrap_or("");
    let critique_short = if critique.len() > 200 {
        &critique[..200]
    } else {
        critique
    };
    let summary = format!(
        "[Iteration {}] Critique: {}... Decision: {}",
        iteration, critique_short, convergence
    );
    log::debug!("Summarized iteration {} state", iteration);
    summary
}
