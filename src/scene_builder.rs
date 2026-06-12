use std::collections::HashMap;
use std::path::Path;

use log::{debug, info};
use serde_json::{json, Value};

use crate::models::Manifest;

pub fn build_empty_scene() -> Value {
    json!({
        "asset": {
            "version": "2.0",
            "generator": "gltf-scene-orchestrator",
        },
        "scenes": [
            {
                "name": "Root Scene",
                "nodes": [],
            }
        ],
        "scene": 0,
        "nodes": [],
        "animations": [],
    })
}

pub fn add_character_node(
    scene: &mut Value,
    character_name: &str,
    _gltf_uri: &str,
    mesh_index: Option<i64>,
) -> i64 {
    let node_index = scene["nodes"].as_array().map(|a| a.len()).unwrap_or(0) as i64;
    let mut node = json!({"name": character_name});
    if let Some(mi) = mesh_index {
        node["mesh"] = json!(mi);
    }
    scene["nodes"]
        .as_array_mut()
        .unwrap()
        .push(node);
    scene["scenes"][0]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!(node_index));
    node_index
}

pub fn build_root_scene(manifest: &Manifest, output_dir: &str) -> Value {
    info!(
        "Building root scene with {} characters",
        manifest.characters.len()
    );
    let mut scene = build_empty_scene();
    for char in &manifest.characters {
        let gltf_uri = &char.gltf_ref;
        let scene_path = Path::new(output_dir).join(gltf_uri).join(gltf_uri);
        if scene_path.exists() {
            let text = std::fs::read_to_string(&scene_path).unwrap_or_default();
            if let Ok(char_gltf) = serde_json::from_str::<Value>(&text) {
                let offsets = merge_character_gltf(&mut scene, &char_gltf, &char.name);
                let char_root_idx = scene["nodes"].as_array().map(|a| a.len()).unwrap_or(0) as i64;
                let mut char_root_node = json!({"name": char.name});
                let root_nodes = char_gltf["scenes"]
                    .as_array()
                    .and_then(|s| s.first())
                    .and_then(|s| s["nodes"].as_array())
                    .cloned()
                    .unwrap_or_default();
                if !root_nodes.is_empty() {
                    let nodes_offset = offsets.get("nodes").copied().unwrap_or(0) as i64;
                    let remapped: Vec<Value> = root_nodes
                        .iter()
                        .map(|rn| json!(rn.as_i64().unwrap_or(0) + nodes_offset))
                        .collect();
                    char_root_node["children"] = Value::Array(remapped);
                }
                scene["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(char_root_node);
                scene["scenes"][0]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(char_root_idx));
                embed_extensions_from(&mut scene, &char_gltf);
            } else {
                debug!(
                    "Character GLTF at {} is invalid JSON, adding placeholder node",
                    scene_path.display()
                );
                add_character_node(&mut scene, &char.name, gltf_uri, None);
            }
        } else {
            debug!(
                "Character GLTF not found at {}, adding placeholder node",
                scene_path.display()
            );
            add_character_node(&mut scene, &char.name, gltf_uri, None);
        }
    }
    scene
}

fn get_or_create_array<'a>(map: &'a mut serde_json::Map<String, Value>, key: &str) -> &'a mut Vec<Value> {
    map.entry(key.to_string())
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .unwrap()
}

fn merge_character_gltf(
    root_scene: &mut Value,
    char_gltf: &Value,
    char_name: &str,
) -> HashMap<String, usize> {
    let mut offsets: HashMap<String, usize> = HashMap::new();

    let merge_order = [
        "extensionsUsed", "extensionsRequired", "buffers", "bufferViews",
        "accessors", "samplers", "images", "textures", "materials",
        "meshes", "skins", "cameras", "nodes", "animations",
    ];

    let map = root_scene.as_object_mut().unwrap();

    for &section in &merge_order {
        match section {
            "extensionsUsed" | "extensionsRequired" => {
                if let Some(items) = char_gltf.get(section).and_then(|v| v.as_array()) {
                    let existing = get_or_create_array(map, section);
                    for ext in items {
                        if let Some(s) = ext.as_str() {
                            if !existing.iter().any(|v: &Value| v.as_str() == Some(s)) {
                                existing.push(json!(s));
                            }
                        }
                    }
                }
                offsets.insert(section.to_string(), 0);
            }
            _ => {
                let items = char_gltf
                    .get(section)
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if items.is_empty() {
                    offsets.insert(section.to_string(), 0);
                    continue;
                }

                let arr = get_or_create_array(map, section);
                let offset = arr.len();
                offsets.insert(section.to_string(), offset);

                for item in &items {
                    arr.push(item.clone());
                }

                let buf_off = *offsets.get("buffers").unwrap_or(&0);
                let bv_off = *offsets.get("bufferViews").unwrap_or(&0);
                let acc_off = *offsets.get("accessors").unwrap_or(&0);
                let img_off = *offsets.get("images").unwrap_or(&0);
                let samp_off = *offsets.get("samplers").unwrap_or(&0);
                let tex_off = *offsets.get("textures").unwrap_or(&0);
                let mat_off = *offsets.get("materials").unwrap_or(&0);
                let mesh_off = *offsets.get("meshes").unwrap_or(&0);
                let node_off = *offsets.get("nodes").unwrap_or(&0);
                let skin_off = *offsets.get("skins").unwrap_or(&0);
                let cam_off = *offsets.get("cameras").unwrap_or(&0);

                match section {
                    "bufferViews" => {
                        for bv in arr.iter_mut().skip(offset) {
                            if let Some(b) = bv.get("buffer").and_then(|v: &Value| v.as_i64()) {
                                bv["buffer"] = json!(b + buf_off as i64);
                            }
                        }
                    }
                    "accessors" => {
                        for acc in arr.iter_mut().skip(offset) {
                            if let Some(bv) = acc.get("bufferView").and_then(|v: &Value| v.as_i64()) {
                                acc["bufferView"] = json!(bv + bv_off as i64);
                            }
                        }
                    }
                    "textures" => {
                        for tex in arr.iter_mut().skip(offset) {
                            if let Some(src) = tex.get("source").and_then(|v: &Value| v.as_i64()) {
                                tex["source"] = json!(src + img_off as i64);
                            }
                            if let Some(samp) = tex.get("sampler").and_then(|v: &Value| v.as_i64()) {
                                tex["sampler"] = json!(samp + samp_off as i64);
                            }
                        }
                    }
                    "materials" => {
                        for mat in arr.iter_mut().skip(offset) {
                            remap_texture_infos(mat, tex_off as i64);
                            for &chan in &["emissiveTexture", "normalTexture", "occlusionTexture"] {
                                if let Some(info) = mat.get(chan).and_then(|v: &Value| v.as_object()) {
                                    if let Some(idx) = info.get("index").and_then(|v: &Value| v.as_i64()) {
                                        mat[chan]["index"] = json!(idx + tex_off as i64);
                                    }
                                }
                            }
                        }
                    }
                    "meshes" => {
                        for mesh in arr.iter_mut().skip(offset) {
                            if let Some(prims) = mesh["primitives"].as_array_mut() {
                                for prim in prims {
                                    if let Some(attrs) = prim["attributes"].as_object() {
                                        let remapped: serde_json::Map<String, Value> = attrs
                                            .iter()
                                            .map(|(k, v): (&String, &Value)| {
                                                (k.clone(), json!(v.as_i64().unwrap_or(0) + acc_off as i64))
                                            })
                                            .collect();
                                        prim["attributes"] = Value::Object(remapped);
                                    }
                                    if let Some(idx) = prim["indices"].as_i64() {
                                        prim["indices"] = json!(idx + acc_off as i64);
                                    }
                                    if let Some(mat) = prim["material"].as_i64() {
                                        prim["material"] = json!(mat + mat_off as i64);
                                    }
                                    if let Some(targets) = prim["targets"].as_array_mut() {
                                        for target in targets {
                                            if let Some(obj) = target.as_object() {
                                                let remapped: serde_json::Map<String, Value> = obj
                                                    .iter()
                                                    .map(|(k, v): (&String, &Value)| {
                                                        (k.clone(), json!(v.as_i64().unwrap_or(0) + acc_off as i64))
                                                    })
                                                    .collect();
                                                *target = Value::Object(remapped);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "skins" => {
                        for skin in arr.iter_mut().skip(offset) {
                            if let Some(ibm) = skin["inverseBindMatrices"].as_i64() {
                                skin["inverseBindMatrices"] = json!(ibm + acc_off as i64);
                            }
                            if let Some(joins) = skin["joints"].as_array() {
                                let remapped: Vec<Value> = joins
                                    .iter()
                                    .map(|j: &Value| json!(j.as_i64().unwrap_or(0) + node_off as i64))
                                    .collect();
                                skin["joints"] = Value::Array(remapped);
                            }
                            if let Some(skel) = skin["skeleton"].as_i64() {
                                skin["skeleton"] = json!(skel + node_off as i64);
                            }
                        }
                    }
                    "nodes" => {
                        for node in arr.iter_mut().skip(offset) {
                            if let Some(children) = node["children"].as_array() {
                                let remapped: Vec<Value> = children
                                    .iter()
                                    .map(|c: &Value| json!(c.as_i64().unwrap_or(0) + node_off as i64))
                                    .collect();
                                node["children"] = Value::Array(remapped);
                            }
                            if let Some(mesh) = node["mesh"].as_i64() {
                                node["mesh"] = json!(mesh + mesh_off as i64);
                            }
                            if let Some(skin) = node["skin"].as_i64() {
                                node["skin"] = json!(skin + skin_off as i64);
                            }
                            if let Some(cam) = node["camera"].as_i64() {
                                node["camera"] = json!(cam + cam_off as i64);
                            }
                        }
                    }
                    "animations" => {
                        for anim in arr.iter_mut().skip(offset) {
                            if let Some(channels) = anim["channels"].as_array_mut() {
                                for ch in channels {
                                    if let Some(target) = ch["target"].as_object() {
                                        if let Some(n) = target.get("node").and_then(|v: &Value| v.as_i64()) {
                                            ch["target"]["node"] = json!(n + node_off as i64);
                                        }
                                    }
                                }
                            }
                            if let Some(samplers) = anim["samplers"].as_array_mut() {
                                for samp in samplers {
                                    if let Some(inp) = samp["input"].as_i64() {
                                        samp["input"] = json!(inp + acc_off as i64);
                                    }
                                    if let Some(out) = samp["output"].as_i64() {
                                        samp["output"] = json!(out + acc_off as i64);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let n_nodes = char_gltf
        .get("nodes")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let n_meshes = char_gltf
        .get("meshes")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    debug!(
        "Merged {}: +{} nodes, +{} meshes",
        if char_name.is_empty() {
            "unknown"
        } else {
            char_name
        },
        n_nodes,
        n_meshes
    );
    offsets
}

fn remap_texture_infos(mat: &mut Value, tex_offset: i64) {
    if let Some(pbr) = mat.get_mut("pbrMetallicRoughness") {
        if let Some(obj) = pbr.as_object() {
            let keys: Vec<String> = obj.keys().cloned().collect();
            for key in keys {
                if key == "baseColorTexture" || key == "metallicRoughnessTexture" {
                    if let Some(info) = pbr.get(&key).and_then(|v| v.as_object()) {
                        if let Some(idx) = info.get("index").and_then(|v| v.as_i64()) {
                            pbr[&key]["index"] = json!(idx + tex_offset);
                        }
                    }
                }
            }
        }
    }
}

fn embed_extensions_from(scene: &mut Value, char_gltf: &Value) {
    let map = scene.as_object_mut().unwrap();
    if let Some(used) = char_gltf.get("extensionsUsed").and_then(|v| v.as_array()) {
        let arr = get_or_create_array(map, "extensionsUsed");
        for ext in used {
            if let Some(s) = ext.as_str() {
                if !arr.iter().any(|v: &Value| v.as_str() == Some(s)) {
                    arr.push(json!(s));
                }
            }
        }
    }
    if let Some(required) = char_gltf
        .get("extensionsRequired")
        .and_then(|v| v.as_array())
    {
        let arr = get_or_create_array(map, "extensionsRequired");
        for ext in required {
            if let Some(s) = ext.as_str() {
                if !arr.iter().any(|v: &Value| v.as_str() == Some(s)) {
                    arr.push(json!(s));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CharacterConfig, Manifest};

    #[test]
    fn test_build_empty_scene() {
        let scene = build_empty_scene();
        assert_eq!(scene["asset"]["version"], "2.0");
        assert_eq!(scene["scenes"][0]["nodes"].as_array().unwrap().len(), 0);
        assert_eq!(scene["nodes"].as_array().unwrap().len(), 0);
        assert_eq!(scene["animations"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_add_character_node() {
        let mut scene = build_empty_scene();
        let idx = add_character_node(&mut scene, "knight", "knight.gltf", None);
        assert_eq!(idx, 0);
        assert_eq!(scene["nodes"][0]["name"], "knight");
    }

    #[test]
    fn test_add_character_node_with_mesh() {
        let mut scene = build_empty_scene();
        let idx = add_character_node(&mut scene, "robot", "robot.gltf", Some(2));
        assert_eq!(scene["nodes"][idx as usize]["mesh"], 2);
    }

    #[test]
    fn test_build_root_scene_placeholder() {
        let manifest = Manifest {
            prompt: "test".to_string(),
            characters: vec![CharacterConfig {
                name: "ghost".to_string(),
                gltf_ref: "nonexistent.gltf".to_string(),
            }],
            duration_seconds: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let scene = build_root_scene(&manifest, dir.path().to_str().unwrap());
        assert_eq!(scene["nodes"].as_array().unwrap().len(), 1);
        assert_eq!(scene["nodes"][0]["name"], "ghost");
    }
}
