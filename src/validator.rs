use serde_json::Value;

pub fn validate_gltf_json(gltf: &Value) -> Vec<String> {
    let mut errors = Vec::new();

    if !gltf.get("asset").is_some() {
        errors.push("Missing 'asset' field".to_string());
    } else if !gltf["asset"].get("version").is_some() {
        errors.push("Missing 'asset.version'".to_string());
    }

    match gltf.get("scenes") {
        None => errors.push("Missing or invalid 'scenes'".to_string()),
        Some(s) => {
            if !s.is_array() {
                errors.push("Missing or invalid 'scenes'".to_string());
            } else if s.as_array().unwrap().is_empty() {
                errors.push("'scenes' array is empty".to_string());
            }
        }
    }

    if !gltf.get("scene").is_some() {
        errors.push("Missing 'scene' index".to_string());
    }

    match gltf.get("nodes") {
        None => errors.push("Missing or invalid 'nodes'".to_string()),
        Some(n) => {
            if !n.is_array() {
                errors.push("Missing or invalid 'nodes'".to_string());
            }
        }
    }

    if let Some(anims) = gltf.get("animations") {
        if !anims.is_array() {
            errors.push("'animations' must be an array".to_string());
        } else {
            for (i, anim) in anims.as_array().unwrap().iter().enumerate() {
                errors.extend(validate_animation(anim, i));
            }
        }
    }

    errors
}

fn validate_animation(anim: &Value, index: usize) -> Vec<String> {
    let mut errors = Vec::new();
    let prefix = format!("animations[{}]", index);

    match anim.get("channels") {
        None => errors.push(format!("{}: missing or invalid 'channels'", prefix)),
        Some(c) => {
            if !c.is_array() {
                errors.push(format!("{}: missing or invalid 'channels'", prefix));
            }
        }
    }

    match anim.get("samplers") {
        None => errors.push(format!("{}: missing or invalid 'samplers'", prefix)),
        Some(s) => {
            if !s.is_array() {
                errors.push(format!("{}: missing or invalid 'samplers'", prefix));
            }
        }
    }

    if let Some(channels) = anim["channels"].as_array() {
        for (ci, ch) in channels.iter().enumerate() {
            if !ch.is_object() {
                errors.push(format!("{}.channels[{}]: expected object", prefix, ci));
                continue;
            }
            let ch_prefix = format!("{}.channels[{}]", prefix, ci);
            match ch.get("sampler") {
                None => errors.push(format!("{}: missing or invalid 'sampler'", ch_prefix)),
                Some(s) => {
                    if !s.is_i64() {
                        errors.push(format!("{}: missing or invalid 'sampler'", ch_prefix));
                    }
                }
            }
            match ch.get("target") {
                None => errors.push(format!("{}: missing or invalid 'target'", ch_prefix)),
                Some(t) => {
                    if !t.is_object() {
                        errors.push(format!("{}: missing or invalid 'target'", ch_prefix));
                    } else {
                        if !t.get("node").is_some() {
                            errors.push(format!("{}.target: missing 'node'", ch_prefix));
                        }
                        if !t.get("path").is_some() {
                            errors.push(format!("{}.target: missing 'path'", ch_prefix));
                        }
                    }
                }
            }
        }
    }

    if let Some(samplers) = anim["samplers"].as_array() {
        for (si, s) in samplers.iter().enumerate() {
            if !s.is_object() {
                errors.push(format!("{}.samplers[{}]: expected object", prefix, si));
                continue;
            }
            let s_prefix = format!("{}.samplers[{}]", prefix, si);
            match s.get("input") {
                None => errors.push(format!("{}: missing or invalid 'input'", s_prefix)),
                Some(inp) => {
                    if !inp.is_i64() {
                        errors.push(format!("{}: missing or invalid 'input'", s_prefix));
                    }
                }
            }
            match s.get("output") {
                None => errors.push(format!("{}: missing or invalid 'output'", s_prefix)),
                Some(out) => {
                    if !out.is_i64() {
                        errors.push(format!("{}: missing or invalid 'output'", s_prefix));
                    }
                }
            }
        }
    }

    errors
}

pub fn validate_animations(gltf: &Value) -> Vec<String> {
    validate_gltf_json(gltf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_minimal_scene() {
        let gltf = json!({
            "asset": {"version": "2.0"},
            "scene": 0,
            "scenes": [{"nodes": []}],
            "nodes": [],
        });
        assert!(validate_gltf_json(&gltf).is_empty());
    }

    #[test]
    fn test_missing_asset() {
        let gltf = json!({"scene": 0, "scenes": [{"nodes": []}], "nodes": []});
        let errors = validate_gltf_json(&gltf);
        assert!(errors.iter().any(|e| e.contains("asset")));
    }

    #[test]
    fn test_missing_scenes() {
        let gltf = json!({"asset": {"version": "2.0"}, "scene": 0, "nodes": []});
        let errors = validate_gltf_json(&gltf);
        assert!(errors.iter().any(|e| e.contains("scenes")));
    }

    #[test]
    fn test_empty_scenes() {
        let gltf = json!({"asset": {"version": "2.0"}, "scene": 0, "scenes": [], "nodes": []});
        let errors = validate_gltf_json(&gltf);
        assert!(errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn test_valid_animation() {
        let gltf = json!({
            "asset": {"version": "2.0"},
            "scene": 0,
            "scenes": [{"nodes": []}],
            "nodes": [],
            "animations": [
                {
                    "channels": [
                        {"sampler": 0, "target": {"node": 0, "path": "translation"}}
                    ],
                    "samplers": [
                        {"input": 0, "output": 1, "interpolation": "LINEAR"}
                    ],
                }
            ],
        });
        assert!(validate_gltf_json(&gltf).is_empty());
    }

    #[test]
    fn test_animation_missing_sampler() {
        let gltf = json!({
            "asset": {"version": "2.0"},
            "scene": 0,
            "scenes": [{"nodes": []}],
            "nodes": [],
            "animations": [
                {
                    "channels": [{"target": {"node": 0, "path": "translation"}}],
                    "samplers": [],
                }
            ],
        });
        let errors = validate_gltf_json(&gltf);
        assert!(errors.iter().any(|e| e.contains("sampler")));
    }

    #[test]
    fn test_validate_animations_alias() {
        let gltf = json!({
            "asset": {"version": "2.0"},
            "scene": 0,
            "scenes": [{"nodes": []}],
            "nodes": [],
            "animations": [],
        });
        assert_eq!(validate_animations(&gltf), validate_gltf_json(&gltf));
    }
}
