use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::models::Manifest;

pub fn parse_manifest(path: &str) -> Result<Manifest> {
    if !Path::new(path).exists() {
        bail!("Manifest not found: {}", path);
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest: {}", path))?;
    let data: Value = serde_json::from_str(&text)
        .with_context(|| format!("Invalid JSON in manifest: {}", path))?;
    let errors = validate_manifest(&data);
    if !errors.is_empty() {
        bail!("Manifest validation failed: {}", errors.join("; "));
    }
    Ok(Manifest::from_dict(&data))
}

pub fn validate_manifest(data: &Value) -> Vec<String> {
    let mut errors = Vec::new();

    match data.get("prompt") {
        None => errors.push("Missing or empty 'prompt' field".to_string()),
        Some(p) => {
            if !p.is_string() || p.as_str().unwrap_or("").trim().is_empty() {
                errors.push("Missing or empty 'prompt' field".to_string());
            }
        }
    }

    match data.get("characters") {
        None => errors.push("Missing or empty 'characters' list".to_string()),
        Some(c) => {
            if !c.is_array() || c.as_array().unwrap().is_empty() {
                errors.push("Missing or empty 'characters' list".to_string());
            } else {
                for (i, char_entry) in c.as_array().unwrap().iter().enumerate() {
                    if !char_entry.is_object() {
                        errors.push(format!("characters[{}]: expected object", i));
                        continue;
                    }
                    let name = char_entry.get("name");
                    if name.is_none() || !name.unwrap().is_string() || name.unwrap().as_str().unwrap_or("").trim().is_empty() {
                        errors.push(format!("characters[{}]: missing or invalid 'name'", i));
                    }
                    let gltf = char_entry.get("gltf_ref");
                    if gltf.is_none() || !gltf.unwrap().is_string() || gltf.unwrap().as_str().unwrap_or("").trim().is_empty() {
                        errors.push(format!("characters[{}]: missing or invalid 'gltf_ref'", i));
                    } else if !gltf.unwrap().as_str().unwrap_or("").contains(".gltf") {
                        errors.push(format!("characters[{}]: 'gltf_ref' must contain '.gltf'", i));
                    }
                }
            }
        }
    }

    if let Some(dur) = data.get("duration_seconds") {
        if !dur.is_f64() || dur.as_f64().unwrap_or(0.0) <= 0.0 {
            errors.push("'duration_seconds' must be a positive number".to_string());
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_manifest_valid() {
        let data = json!({
            "prompt": "A knight fights a dragon",
            "characters": [
                {"name": "knight", "gltf_ref": "knight.gltf"},
                {"name": "dragon", "gltf_ref": "dragon.gltf"},
            ],
            "duration_seconds": 30.0,
        });
        let errors = validate_manifest(&data);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_manifest_missing_prompt() {
        let data = json!({"characters": [{"name": "k", "gltf_ref": "k.gltf"}]});
        let errors = validate_manifest(&data);
        assert!(errors.iter().any(|e| e.contains("prompt")));
    }

    #[test]
    fn test_validate_manifest_empty_characters() {
        let data = json!({"prompt": "test", "characters": []});
        let errors = validate_manifest(&data);
        assert!(errors.iter().any(|e| e.contains("characters")));
    }

    #[test]
    fn test_validate_manifest_gltf_ref_no_dot() {
        let data = json!({
            "prompt": "test",
            "characters": [{"name": "k", "gltf_ref": "knight"}],
        });
        let errors = validate_manifest(&data);
        assert!(errors.iter().any(|e| e.contains("gltf_ref") && e.contains(".gltf")));
    }

    #[test]
    fn test_validate_manifest_invalid_duration() {
        let data = json!({
            "prompt": "test",
            "characters": [{"name": "k", "gltf_ref": "k.gltf"}],
            "duration_seconds": -5,
        });
        let errors = validate_manifest(&data);
        assert!(errors.iter().any(|e| e.contains("duration")));
    }

    #[test]
    fn test_parse_manifest_valid_file() {
        let data = json!({
            "prompt": "A test scene",
            "characters": [{"name": "robot", "gltf_ref": "robot.gltf"}],
        });
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.json");
        std::fs::write(&path, serde_json::to_string(&data).unwrap()).unwrap();
        let manifest = parse_manifest(path.to_str().unwrap()).unwrap();
        assert_eq!(manifest.prompt, "A test scene");
        assert_eq!(manifest.characters.len(), 1);
        assert_eq!(manifest.characters[0].name, "robot");
    }

    #[test]
    fn test_parse_manifest_file_not_found() {
        let result = parse_manifest("/nonexistent/manifest.json");
        assert!(result.is_err());
    }
}
