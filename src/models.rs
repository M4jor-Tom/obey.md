use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConfig {
    pub name: String,
    pub gltf_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub prompt: String,
    pub characters: Vec<CharacterConfig>,
    pub duration_seconds: Option<f64>,
}

impl Manifest {
    pub fn from_dict(data: &Value) -> Self {
        let chars: Vec<CharacterConfig> = data["characters"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|c| CharacterConfig {
                        name: c["name"].as_str().unwrap_or("").to_string(),
                        gltf_ref: c["gltf_ref"].as_str().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Manifest {
            prompt: data["prompt"].as_str().unwrap_or("").to_string(),
            characters: chars,
            duration_seconds: data["duration_seconds"].as_f64(),
        }
    }

    pub fn to_dict(&self) -> Value {
        serde_json::json!(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationState {
    pub iteration_number: i32,
    pub scene_gltf: Value,
    pub critique: Option<String>,
    pub convergence_decision: Option<String>,
    pub errors: Vec<String>,
}

impl IterationState {
    pub fn new(iteration_number: i32, scene_gltf: Value) -> Self {
        IterationState {
            iteration_number,
            scene_gltf,
            critique: None,
            convergence_decision: None,
            errors: Vec::new(),
        }
    }

    pub fn to_dict(&self) -> Value {
        serde_json::json!({
            "iteration_number": self.iteration_number,
            "critique": self.critique,
            "convergence_decision": self.convergence_decision,
            "errors": self.errors,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SceneState {
    pub manifest: Manifest,
    pub current_scene_gltf: Value,
    pub iteration_history: Vec<IterationState>,
    pub iteration_count: i32,
    pub converged: bool,
}

pub fn load_manifest_from_string(text: &str) -> Result<Manifest, serde_json::Error> {
    let data: Value = serde_json::from_str(text)?;
    Ok(Manifest::from_dict(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_manifest_from_dict() {
        let data = json!({
            "prompt": "A knight fights a dragon",
            "characters": [
                {"name": "knight", "gltf_ref": "knight.gltf"},
                {"name": "dragon", "gltf_ref": "dragon.gltf"},
            ],
            "duration_seconds": 30.0,
        });
        let manifest = Manifest::from_dict(&data);
        assert_eq!(manifest.prompt, "A knight fights a dragon");
        assert_eq!(manifest.characters.len(), 2);
        assert_eq!(manifest.characters[0].name, "knight");
        assert_eq!(manifest.characters[1].gltf_ref, "dragon.gltf");
        assert_eq!(manifest.duration_seconds, Some(30.0));
    }

    #[test]
    fn test_manifest_to_dict() {
        let manifest = Manifest {
            prompt: "test".to_string(),
            characters: vec![CharacterConfig {
                name: "hero".to_string(),
                gltf_ref: "hero.gltf".to_string(),
            }],
            duration_seconds: None,
        };
        let d = manifest.to_dict();
        assert_eq!(d["prompt"], "test");
        assert_eq!(d["characters"][0]["name"], "hero");
    }

    #[test]
    fn test_iteration_state_new() {
        let state = IterationState::new(0, json!({"key": "val"}));
        assert_eq!(state.iteration_number, 0);
        assert!(state.critique.is_none());
        assert!(state.errors.is_empty());
    }

    #[test]
    fn test_iteration_state_to_dict() {
        let mut state = IterationState::new(3, json!({}));
        state.critique = Some("good".to_string());
        state.convergence_decision = Some("finalize".to_string());
        let d = state.to_dict();
        assert_eq!(d["iteration_number"], 3);
        assert_eq!(d["critique"], "good");
        assert_eq!(d["convergence_decision"], "finalize");
    }

    #[test]
    fn test_load_manifest_from_string() {
        let text = r#"{"prompt": "test", "characters": [{"name": "a", "gltf_ref": "a.gltf"}]}"#;
        let manifest = load_manifest_from_string(text).unwrap();
        assert_eq!(manifest.prompt, "test");
        assert_eq!(manifest.characters.len(), 1);
    }
}
