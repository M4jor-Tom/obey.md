use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use log::{info, warn};

pub fn check_tools_available() -> HashMap<String, bool> {
    let mut result = HashMap::new();
    result.insert(
        "gltf_to_png".to_string(),
        which("gltf_to_png").is_some(),
    );
    result.insert(
        "gltf_to_webm".to_string(),
        which("gltf_to_webm").is_some(),
    );
    result
}

fn which(name: &str) -> Option<String> {
    std::env::var("PATH").ok().and_then(|path| {
        for dir in path.split(':') {
            let full = Path::new(dir).join(name);
            if full.exists() {
                return Some(full.to_string_lossy().to_string());
            }
        }
        None
    })
}

pub fn generate_png_preview(scene_gltf_path: &str, output_path: &str) -> Result<String, String> {
    let script = which("gltf_to_png")
        .ok_or_else(|| "gltf_to_png not found on PATH".to_string())?;
    info!(
        "Rendering PNG: {} -> {}",
        Path::new(scene_gltf_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        Path::new(output_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
    );
    let output = Command::new(&script)
        .args(["-i", scene_gltf_path, "-o", output_path])
        .output()
        .map_err(|e| format!("Failed to run gltf_to_png: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gltf_to_png failed: {}", stderr));
    }
    Ok(output_path.to_string())
}

pub fn generate_webm_preview(scene_gltf_path: &str, output_path: &str) -> Result<String, String> {
    let script = which("gltf_to_webm")
        .ok_or_else(|| "gltf_to_webm not found on PATH".to_string())?;
    info!(
        "Rendering WebM: {} -> {}",
        Path::new(scene_gltf_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        Path::new(output_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
    );
    let output = Command::new(&script)
        .args(["-i", scene_gltf_path, "-o", output_path])
        .output()
        .map_err(|e| format!("Failed to run gltf_to_webm: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gltf_to_webm failed: {}", stderr));
    }
    Ok(output_path.to_string())
}

pub fn generate_intermediate_frames(
    scene_gltf_path: &str,
    output_dir: &str,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output dir: {}", e))?;
    let preview_path = format!("{}/intermediate_preview.webm", output_dir);
    generate_webm_preview(scene_gltf_path, &preview_path)?;
    Ok(vec![preview_path])
}

pub fn generate_final_previews(
    scene_path: &str,
    output_dir: &str,
) -> HashMap<String, String> {
    info!("Generating final previews in {}", output_dir);
    let _ = std::fs::create_dir_all(output_dir);
    let png_path = format!("{}/preview.png", output_dir);
    let webm_path = format!("{}/preview.webm", output_dir);
    let mut results = HashMap::new();

    let tools = check_tools_available();
    if *tools.get("gltf_to_png").unwrap_or(&false) {
        match generate_png_preview(scene_path, &png_path) {
            Ok(p) => {
                results.insert("png".to_string(), p);
            }
            Err(e) => {
                warn!("PNG preview failed: {}", e);
                results.insert("png".to_string(), String::new());
            }
        }
    }
    if *tools.get("gltf_to_webm").unwrap_or(&false) {
        match generate_webm_preview(scene_path, &webm_path) {
            Ok(w) => {
                results.insert("webm".to_string(), w);
            }
            Err(e) => {
                warn!("WebM preview failed: {}", e);
                results.insert("webm".to_string(), String::new());
            }
        }
    }
    results
}
