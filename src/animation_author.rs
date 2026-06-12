use base64::Engine;
use serde_json::{json, Value};

pub const ANIMATION_AUTHORING_PROMPT_TEMPLATE: &str = r#"
You are an expert GLTF animation engineer. Below is the current scene GLTF JSON and the user's animation prompt.

Your task is to produce a JSON object containing animation data that will be merged into the scene GLTF.

The animation data must follow this structure:
{
  "animations": [
    {
      "name": "<animation_name>",
      "channels": [
        {
          "sampler": <sampler_index>,
          "target": {
            "node": <node_index>,
            "path": "<translation|rotation|scale|weights>"
          }
        }
      ],
      "samplers": [
        {
          "input": <accessor_index>,
          "output": <accessor_index>,
          "interpolation": "LINEAR"
        }
      ]
    }
  ],
  "accessors": [
    {
      "bufferView": <bufferView_index>,
      "componentType": <FLOAT=5126>,
      "count": <count>,
      "type": "<VEC3|VEC4|SCALAR>",
      "max": [...],
      "min": [...]
    }
  ],
  "bufferViews": [
    {
      "buffer": 0,
      "byteOffset": <offset>,
      "byteLength": <length>,
      "target": <ARRAY_BUFFER=34962>
    }
  ],
  "buffers": [
    {
      "uri": "<data_uri_or_path>",
      "byteLength": <length>
    }
  ],
  "replace_existing": <true|false>
}

Rules:
- All characters animate concurrently on a single shared timeline.
- Use "translation", "rotation", "scale", and "weights" (morph target) paths as appropriate.
- Keyframe times should be in seconds, starting at 0.
- Do not exceed 10,000 keyframes per channel.
- Scene duration should be inferred from the prompt or use {duration_seconds}s.
- Ensure all accessor indices and bufferView indices are sequential and valid.

Current scene GLTF:
{scene_gltf_json}

Character manifest:
{manifest_json}

Iteration state (previous critiques, current progress):
{iteration_state_json}

Respond with ONLY the valid JSON animation data. No markdown fences, no explanation.
"#;

pub fn build_authoring_prompt(
    scene_gltf: &Value,
    manifest_json: &str,
    iteration_state_json: &str,
    duration_seconds: f64,
) -> String {
    let prompt = ANIMATION_AUTHORING_PROMPT_TEMPLATE
        .replace("{scene_gltf_json}", &serde_json::to_string_pretty(scene_gltf).unwrap_or_default())
        .replace("{manifest_json}", manifest_json)
        .replace("{iteration_state_json}", iteration_state_json)
        .replace("{duration_seconds}", &duration_seconds.to_string());
    log::debug!(
        "Built authoring prompt ({} chars), duration={}s",
        prompt.len(),
        duration_seconds
    );
    prompt
}

fn float32_to_bytes(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

fn ensure_animation_structures(scene: &mut Value) {
    if !scene.get("animations").map_or(false, |v| v.is_array()) {
        scene["animations"] = json!([]);
    }
    if !scene.get("accessors").map_or(false, |v| v.is_array()) {
        scene["accessors"] = json!([]);
    }
    if !scene.get("bufferViews").map_or(false, |v| v.is_array()) {
        scene["bufferViews"] = json!([]);
    }
    if !scene.get("buffers").map_or(false, |v| v.is_array()) {
        scene["buffers"] = json!([]);
    }
}

fn next_accessor_index(scene: &Value) -> i64 {
    scene["accessors"]
        .as_array()
        .map(|a| a.len() as i64)
        .unwrap_or(0)
}

fn ensure_buffer(scene: &mut Value, data: &[u8]) -> i64 {
    let b64 = base64::engine::general_purpose::STANDARD.encode(data);
    let uri = format!("data:application/octet-stream;base64,{}", b64);
    let arr = scene["buffers"].as_array_mut().unwrap();
    let idx = arr.len() as i64;
    arr.push(json!({
        "uri": uri,
        "byteLength": data.len(),
    }));
    idx
}

fn buffer_offset(scene: &Value, buffer_index: i64) -> i64 {
    let mut offset: i64 = 0;
    if let Some(bvs) = scene["bufferViews"].as_array() {
        for bv in bvs {
            if bv["buffer"].as_i64() == Some(buffer_index) {
                let bo = bv["byteOffset"].as_i64().unwrap_or(0);
                let bl = bv["byteLength"].as_i64().unwrap_or(0);
                offset = offset.max(bo + bl);
            }
        }
    }
    offset
}

fn add_buffer_view(scene: &mut Value, data: &[u8]) -> i64 {
    let bv_index = scene["bufferViews"]
        .as_array()
        .map(|a| a.len() as i64)
        .unwrap_or(0);
    let buf_index = ensure_buffer(scene, data);
    let offset = buffer_offset(scene, buf_index);
    scene["bufferViews"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "buffer": buf_index,
            "byteOffset": offset,
            "byteLength": data.len(),
            "target": 34962,
        }));
    bv_index
}

fn add_accessor(
    scene: &mut Value,
    buffer_view_index: i64,
    component_type: i64,
    count: i64,
    type_str: &str,
    max_vals: &[f64],
    min_vals: &[f64],
) -> i64 {
    let idx = scene["accessors"]
        .as_array()
        .map(|a| a.len() as i64)
        .unwrap_or(0);
    scene["accessors"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "bufferView": buffer_view_index,
            "byteOffset": 0,
            "componentType": component_type,
            "count": count,
            "type": type_str,
            "max": max_vals,
            "min": min_vals,
        }));
    idx
}

pub fn add_translation_keyframe(
    scene_gltf: &Value,
    node_index: i64,
    time: f32,
    value: [f32; 3],
    animation_index: usize,
) -> Value {
    add_trs_keyframe(scene_gltf, node_index, "translation", time, &value, animation_index)
}

pub fn add_rotation_keyframe(
    scene_gltf: &Value,
    node_index: i64,
    time: f32,
    value: [f32; 4],
    animation_index: usize,
) -> Value {
    add_trs_keyframe(scene_gltf, node_index, "rotation", time, &value, animation_index)
}

pub fn add_scale_keyframe(
    scene_gltf: &Value,
    node_index: i64,
    time: f32,
    value: [f32; 3],
    animation_index: usize,
) -> Value {
    add_trs_keyframe(scene_gltf, node_index, "scale", time, &value, animation_index)
}

fn add_trs_keyframe(
    scene_gltf: &Value,
    node_index: i64,
    path: &str,
    time: f32,
    value: &[f32],
    animation_index: usize,
) -> Value {
    let mut scene = scene_gltf.clone();
    ensure_animation_structures(&mut scene);

    if scene["animations"].as_array().map_or(true, |a| a.is_empty()) {
        scene["animations"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "animation_0", "channels": [], "samplers": []}));
    }

    let anim_len = scene["animations"].as_array().map_or(0, |a| a.len());
    let anim_idx = if animation_index < anim_len {
        animation_index
    } else {
        anim_len.saturating_sub(1)
    };

    let sampler_idx = scene["animations"][anim_idx]["samplers"]
        .as_array()
        .map(|a| a.len() as i64)
        .unwrap_or(0);

    let input_acc = next_accessor_index(&scene);
    let output_acc = input_acc + 1;

    scene["animations"][anim_idx]["channels"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "sampler": sampler_idx,
            "target": {"node": node_index, "path": path},
        }));

    scene["animations"][anim_idx]["samplers"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "input": input_acc,
            "output": output_acc,
            "interpolation": "LINEAR",
        }));

    let time_buffer = float32_to_bytes(&[time]);
    let value_buffer = float32_to_bytes(value);

    let vec_type = match value.len() {
        3 => "VEC3",
        4 => "VEC4",
        _ => "SCALAR",
    };
    let comp_count = value.len() as i64;

    let time_bv = add_buffer_view(&mut scene, &time_buffer);
    add_accessor(&mut scene, time_bv, 5126, 1, "SCALAR", &[time as f64], &[time as f64]);

    let value_bv = add_buffer_view(&mut scene, &value_buffer);
    let vals: Vec<f64> = value.iter().map(|v| *v as f64).collect();
    add_accessor(&mut scene, value_bv, 5126, comp_count, vec_type, &vals, &vals);

    scene
}

pub fn add_morph_weight_track(
    scene_gltf: &Value,
    node_index: i64,
    weight_keyframes: &[Value],
    animation_index: usize,
) -> Value {
    let mut scene = scene_gltf.clone();
    ensure_animation_structures(&mut scene);

    if scene["animations"].as_array().map_or(true, |a| a.is_empty()) {
        scene["animations"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "animation_0", "channels": [], "samplers": []}));
    }

    let anim_len = scene["animations"].as_array().map_or(0, |a| a.len());
    let anim_idx = if animation_index < anim_len {
        animation_index
    } else {
        anim_len.saturating_sub(1)
    };

    let sampler_idx = scene["animations"][anim_idx]["samplers"]
        .as_array()
        .map(|a| a.len() as i64)
        .unwrap_or(0);

    let input_acc = next_accessor_index(&scene);
    let output_acc = input_acc + 1;

    scene["animations"][anim_idx]["channels"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "sampler": sampler_idx,
            "target": {"node": node_index, "path": "weights"},
        }));

    scene["animations"][anim_idx]["samplers"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "input": input_acc,
            "output": output_acc,
            "interpolation": "LINEAR",
        }));

    let times: Vec<f32> = weight_keyframes
        .iter()
        .map(|kf| kf["time"].as_f64().unwrap_or(0.0) as f32)
        .collect();
    let weights: Vec<f32> = weight_keyframes
        .iter()
        .flat_map(|kf| {
            kf["weights"]
                .as_array()
                .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect::<Vec<_>>())
                .unwrap_or_else(|| vec![0.0f32])
        })
        .collect();

    let time_buffer = float32_to_bytes(&times);
    let weights_buffer = float32_to_bytes(&weights);

    let time_bv = add_buffer_view(&mut scene, &time_buffer);
    let max_time = times.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min_time = times.iter().cloned().fold(f32::INFINITY, f32::min);
    add_accessor(
        &mut scene,
        time_bv,
        5126,
        times.len() as i64,
        "SCALAR",
        &[max_time as f64],
        &[min_time as f64],
    );

    let weights_bv = add_buffer_view(&mut scene, &weights_buffer);
    let max_w = weights.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min_w = weights.iter().cloned().fold(f32::INFINITY, f32::min);
    let w_max: &[f64] = if weights.is_empty() { &[0.0] } else { &[max_w as f64] };
    let w_min: &[f64] = if weights.is_empty() { &[0.0] } else { &[min_w as f64] };
    add_accessor(
        &mut scene,
        weights_bv,
        5126,
        weights.len() as i64,
        "SCALAR",
        w_max,
        w_min,
    );

    scene
}

pub fn apply_animation_to_scene(scene_gltf: &Value, animation_data: &Value) -> Value {
    let mut scene = scene_gltf.clone();
    ensure_animation_structures(&mut scene);

    let n_anims = animation_data["animations"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let replace = animation_data["replace_existing"]
        .as_bool()
        .unwrap_or(false);

    if n_anims > 0 {
        if replace {
            scene["animations"] = json!([]);
        }
        if let Some(new_anims) = animation_data["animations"].as_array() {
            for a in new_anims {
                scene["animations"]
                    .as_array_mut()
                    .unwrap()
                    .push(a.clone());
            }
        }
    }

    let n_acc = animation_data["accessors"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let n_bv = animation_data["bufferViews"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let n_buf = animation_data["buffers"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);

    for key in &["accessors", "bufferViews", "buffers"] {
        if let Some(items) = animation_data.get(*key).and_then(|v| v.as_array()) {
            if let Some(map) = scene.as_object_mut() {
                let target = map
                    .entry(key.to_string())
                    .or_insert_with(|| json!([]));
                target.as_array_mut().unwrap().extend(items.iter().cloned());
            }
        }
    }

    log::info!(
        "Applied {} animations (replace={}), +{} accessors, +{} bufferViews, +{} buffers",
        n_anims,
        replace,
        n_acc,
        n_bv,
        n_buf
    );
    scene
}

pub fn detect_missing_capabilities(character_gltf: &Value) -> Vec<Value> {
    let mut missing = Vec::new();
    let mut has_morph_targets = false;
    let mut has_skeletons = false;

    if let Some(meshes) = character_gltf["meshes"].as_array() {
        for mesh in meshes {
            if let Some(prims) = mesh["primitives"].as_array() {
                for prim in prims {
                    if prim.get("targets").is_some() || prim.get("morphTargets").is_some() {
                        has_morph_targets = true;
                    }
                }
            }
            if mesh.get("weights").is_some() {
                has_morph_targets = true;
            }
        }
    }

    if let Some(nodes) = character_gltf["nodes"].as_array() {
        for node in nodes {
            if node.get("skin").is_some() {
                has_skeletons = true;
            }
        }
    }

    if !has_morph_targets {
        missing.push(json!({
            "capability": "morph_targets",
            "fallback": "Use TRS keyframes (rotation/scale) to approximate morph target effects",
        }));
    }
    if !has_skeletons {
        missing.push(json!({
            "capability": "skeletal_animation",
            "fallback": "Use node TRS keyframes for animation instead of bones",
        }));
    }

    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_authoring_prompt() {
        let scene = json!({"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []});
        let prompt = build_authoring_prompt(
            &scene,
            r#"{"prompt": "test"}"#,
            r#"{"iteration": 0}"#,
            10.0,
        );
        assert!(prompt.contains("GLTF"));
        assert!(!prompt.contains("{scene_gltf_json}"));
        assert!(prompt.contains(r#""version": "2.0""#));
    }

    #[test]
    fn test_add_translation_keyframe() {
        let scene = json!({"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []});
        let result = add_translation_keyframe(&scene, 0, 0.0, [1.0, 2.0, 3.0], 0);
        assert!(result["animations"].as_array().unwrap().len() > 0);
        assert_eq!(result["animations"][0]["channels"][0]["target"]["path"], "translation");
    }

    #[test]
    fn test_add_rotation_keyframe() {
        let scene = json!({"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []});
        let result = add_rotation_keyframe(&scene, 0, 0.5, [0.0, 0.0, 0.0, 1.0], 0);
        assert_eq!(result["animations"][0]["channels"][0]["target"]["path"], "rotation");
    }

    #[test]
    fn test_add_scale_keyframe() {
        let scene = json!({"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []});
        let result = add_scale_keyframe(&scene, 0, 1.0, [2.0, 2.0, 2.0], 0);
        assert_eq!(result["animations"][0]["channels"][0]["target"]["path"], "scale");
    }

    #[test]
    fn test_add_morph_weight_track() {
        let scene = json!({"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []});
        let keyframes = vec![
            json!({"time": 0.0, "weights": [0.0, 1.0]}),
            json!({"time": 1.0, "weights": [1.0, 0.0]}),
        ];
        let result = add_morph_weight_track(&scene, 0, &keyframes, 0);
        assert_eq!(result["animations"][0]["channels"][0]["target"]["path"], "weights");
    }

    #[test]
    fn test_apply_animation_to_scene() {
        let scene = json!({"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [], "animations": []});
        let anim_data = json!({
            "animations": [
                {"channels": [{"sampler": 0, "target": {"node": 0, "path": "translation"}}],
                 "samplers": [{"input": 0, "output": 1, "interpolation": "LINEAR"}]}
            ],
            "accessors": [],
            "bufferViews": [],
            "buffers": [],
        });
        let result = apply_animation_to_scene(&scene, &anim_data);
        assert_eq!(result["animations"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_apply_animation_replace() {
        let scene = json!({"asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": []}], "nodes": [],
            "animations": [{"name": "old"}]});
        let anim_data = json!({"animations": [{"name": "new"}], "replace_existing": true});
        let result = apply_animation_to_scene(&scene, &anim_data);
        assert_eq!(result["animations"], json!([{"name": "new"}]));
    }

    #[test]
    fn test_detect_missing_capabilities_no_morphs() {
        let gltf = json!({"meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}], "nodes": []});
        let missing = detect_missing_capabilities(&gltf);
        let caps: Vec<&str> = missing.iter()
            .filter_map(|m| m["capability"].as_str())
            .collect();
        assert!(caps.contains(&"morph_targets"));
    }

    #[test]
    fn test_detect_missing_capabilities_no_skeleton() {
        let gltf = json!({"meshes": [], "nodes": [{"name": "root"}]});
        let missing = detect_missing_capabilities(&gltf);
        let caps: Vec<&str> = missing.iter()
            .filter_map(|m| m["capability"].as_str())
            .collect();
        assert!(caps.contains(&"skeletal_animation"));
    }
}
