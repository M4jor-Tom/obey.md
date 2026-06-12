use std::path::Path;

use anyhow::{bail, Context, Result};
use log::info;

use crate::models::Manifest;

pub fn resolve_local(source_dir: &str, dest_dir: &str) -> Result<String> {
    let src = Path::new(source_dir);
    if !src.is_dir() {
        bail!("Source directory not found: {}", source_dir);
    }

    let dest = Path::new(dest_dir);
    if dest.exists() {
        std::fs::remove_dir_all(dest)?;
    }
    copy_dir_recursively(src, dest)?;
    info!("Copied {} -> {}", source_dir, dest_dir);

    let gltf_files: Vec<_> = std::fs::read_dir(source_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "gltf")
                .unwrap_or(false)
        })
        .collect();

    if gltf_files.is_empty() {
        bail!("No .gltf file found in {}", source_dir);
    }

    Ok(dest.join(gltf_files[0].file_name()).to_string_lossy().to_string())
}

fn copy_dir_recursively(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dest.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursively(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

pub fn resolve_all(manifest: &Manifest, input_dir: &str, output_dir: &str) -> Result<std::collections::HashMap<String, String>> {
    info!(
        "Resolving {} character GLTFs -> {}",
        manifest.characters.len(),
        output_dir
    );
    let mut resolved = std::collections::HashMap::new();
    for char in &manifest.characters {
        let source_dir = Path::new(input_dir).join(&char.gltf_ref);
        let dest_dir = Path::new(output_dir).join(&char.gltf_ref);
        let resolved_path =
            resolve_local(&source_dir.to_string_lossy(), &dest_dir.to_string_lossy())
                .with_context(|| format!("Failed to resolve {}", char.gltf_ref))?;
        resolved.insert(char.name.clone(), resolved_path);
    }
    Ok(resolved)
}
