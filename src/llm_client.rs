use std::process::{Command, Stdio};

use base64::Engine;
use log::{debug, info, warn};
use serde_json::Value;

use crate::context_manager::estimate_tokens_str;

fn has_remote_config() -> bool {
    let key_set = std::env::var("OPENAI_API_KEY").is_ok()
        && !std::env::var("OPENAI_API_KEY").unwrap_or_default().is_empty();
    let url_set = std::env::var("OPENAI_BASE_URL").is_ok()
        && !std::env::var("OPENAI_BASE_URL").unwrap_or_default().is_empty();
    let key_str = if key_set { "set" } else { "not set" };
    let url_str = if url_set { "set" } else { "not set" };
    debug!("Remote config: key={}, base_url={}", key_str, url_str);
    key_set || url_set
}

fn get_api_key() -> Option<String> {
    std::env::var("OPENAI_API_KEY").ok()
}

fn get_base_url() -> Option<String> {
    std::env::var("OPENAI_BASE_URL").ok()
}

fn get_model() -> String {
    std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string())
}

fn call_opencode(
    prompt: &str,
    system_prompt: Option<&str>,
    images: Option<&[Vec<u8>]>,
) -> Result<String, String> {
    let base_prompt = match system_prompt {
        Some(sys) => format!("<system>\n{}\n</system>\n\n{}", sys, prompt),
        None => prompt.to_string(),
    };

    info!("Using opencode subprocess backend");
    debug!("opencode prompt (first 2000 chars): {}", if prompt.len() > 2000 { &prompt[..2000] } else { prompt });

    let response_file = format!(
        "/tmp/opencode_response_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );

    let full_prompt = format!(
        "{}\n\nWrite your entire final answer to the file at `{}`. Use the Write tool to write to this path.",
        base_prompt, response_file
    );

    let mut cmd = Command::new("opencode");
    cmd.args(["run", "--dangerously-skip-permissions"]);

    if let Some(imgs) = images {
        let tmp_dir = tempfile::Builder::new()
            .prefix("opencode_images_")
            .tempdir()
            .map_err(|e| format!("Failed to create temp dir: {}", e))?;
        for (i, img_bytes) in imgs.iter().enumerate() {
            let path = tmp_dir.path().join(format!("frame_{}.png", i));
            std::fs::write(&path, img_bytes)
                .map_err(|e| format!("Failed to write image {}: {}", i, e))?;
            cmd.arg("--file");
            cmd.arg(&path);
        }
        debug!("Attached {} image(s) to opencode call", imgs.len());
    }

    debug!("Running opencode subprocess");
    let input_len = estimate_tokens_str(&full_prompt);
    debug!("Prompt length: {} tokens (approx)", input_len);

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn opencode: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(full_prompt.as_bytes())
            .map_err(|e| format!("Failed to write to opencode stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to read opencode output: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    debug!(
        "opencode subprocess done: returncode={}, stdout_len={}, stderr_len={}",
        output.status.code().unwrap_or(-1),
        stdout.len(),
        stderr.len(),
    );

    if !output.status.success() {
        let stderr_short = if stderr.len() > 2000 {
            &stderr[..2000]
        } else {
            &stderr
        };
        debug!("opencode stderr: {}", stderr_short);
        return Err(format!(
            "opencode run failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr_short
        ));
    }

    let result = std::fs::read_to_string(&response_file).map_err(|e| {
        format!(
            "Failed to read opencode response from {}: {}. stderr: {}",
            response_file,
            e,
            if stderr.len() > 2000 { &stderr[..2000] } else { &stderr }
        )
    })?;

    let _ = std::fs::remove_file(&response_file);

    let result = result.trim().to_string();
    debug!("opencode response (from file): {} chars", result.len());
    debug!("opencode response content (first 2000 chars): {}", if result.len() > 2000 { &result[..2000] } else { &result });

    if result.is_empty() {
        return Err("opencode returned empty response (file was empty)".to_string());
    }

    Ok(result)
}

fn call_openai(
    prompt: &str,
    system_prompt: Option<&str>,
    images: Option<&[Vec<u8>]>,
    api_key: &str,
    base_url: Option<&str>,
    model: &str,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();

    let mut messages: Vec<Value> = Vec::new();
    if let Some(sys) = system_prompt {
        messages.push(serde_json::json!({"role": "system", "content": sys}));
    }

    let mut content: Vec<Value> = vec![serde_json::json!({"type": "text", "text": prompt})];
    if let Some(imgs) = images {
        for img_bytes in imgs {
            let b64 = base64::engine::general_purpose::STANDARD.encode(img_bytes);
            content.push(serde_json::json!({
                "type": "image_url",
                "image_url": {"url": format!("data:image/png;base64,{}", b64)}
            }));
        }
    }
    messages.push(serde_json::json!({"role": "user", "content": content}));

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": 0.7,
    });

    let url = format!(
        "{}/chat/completions",
        base_url.unwrap_or("https://api.openai.com/v1").trim_end_matches('/')
    );

    info!("LLM call: backend=remote, model={}", model);
    debug!("OpenAI request body: {}", serde_json::to_string(&body).unwrap_or_default());

    let mut req = client.post(&url).json(&body);
    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = req
        .send()
        .map_err(|e| format!("OpenAI request failed: {}", e))?;

    let data: Value = resp
        .json()
        .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

    let result = data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    debug!("LLM response: {} chars", result.len());
    debug!("LLM response content (first 3000 chars): {}", if result.len() > 3000 { &result[..3000] } else { &result });
    Ok(result)
}

pub fn call_llm(
    prompt: &str,
    system_prompt: Option<&str>,
    images: Option<&[Vec<u8>]>,
) -> Result<String, String> {
    if !has_remote_config() {
        return call_opencode(prompt, system_prompt, images);
    }

    let api_key = get_api_key().unwrap_or_default();
    let base_url = get_base_url();
    let model = get_model();
    call_openai(prompt, system_prompt, images, &api_key, base_url.as_deref(), &model)
}

pub fn call_llm_json(
    prompt: &str,
    system_prompt: Option<&str>,
    images: Option<&[Vec<u8>]>,
) -> Result<Value, String> {
    let text = call_llm(prompt, system_prompt, images)?;
    let text = text.trim().to_string();
    debug!("call_llm_json raw text (first 500 chars): {}", if text.len() > 500 { &text[..500] } else { &text });
    let text = if text.starts_with("```") {
        text.splitn(2, '\n')
            .nth(1)
            .unwrap_or("")
            .rsplitn(2, "```")
            .nth(1)
            .unwrap_or("")
            .to_string()
    } else {
        text
    };
    let text = text.trim().to_string();

    serde_json::from_str(&text).map_err(|e| {
        let preview = if text.len() > 200 { &text[..200] } else { &text };
        warn!("JSON parse failed: {} — raw preview: {}...", e, preview);
        format!("JSON parse failed: {}", e)
    })
}

pub fn extract_video_frames(video_path: &str, num_frames: usize) -> Result<Vec<Vec<u8>>, String> {
    let probe_output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=duration",
            "-of",
            "json",
            video_path,
        ])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    let duration = if probe_output.status.success() {
        let probe_text = String::from_utf8_lossy(&probe_output.stdout);
        if let Ok(probe_data) = serde_json::from_str::<Value>(&probe_text) {
            probe_data["streams"]
                .as_array()
                .and_then(|s| s.first())
                .and_then(|s| s["duration"].as_f64())
                .unwrap_or(5.0)
        } else {
            5.0
        }
    } else {
        5.0
    };

    info!(
        "Extracting {} frames from {} (duration={}s)",
        num_frames,
        std::path::Path::new(video_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        duration
    );

    let mut frames: Vec<Vec<u8>> = Vec::new();
    for i in 0..num_frames {
        let t = (i as f64 + 1.0) * duration / (num_frames as f64 + 1.0);
        let ffmpeg_output = Command::new("ffmpeg")
            .args([
                "-ss",
                &t.to_string(),
                "-i",
                video_path,
                "-vframes",
                "1",
                "-f",
                "image2pipe",
                "-vcodec",
                "png",
                "-",
            ])
            .output()
            .map_err(|e| format!("ffmpeg failed: {}", e))?;

        if ffmpeg_output.status.success() && !ffmpeg_output.stdout.is_empty() {
            frames.push(ffmpeg_output.stdout);
        }
    }

    debug!("Extracted {} frames successfully", frames.len());
    Ok(frames)
}
