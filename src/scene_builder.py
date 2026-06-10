from __future__ import annotations

import copy
import json
import os
from typing import Optional

from src.models import Manifest


def build_empty_scene() -> dict:
    return {
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
    }


def add_character_node(scene: dict, character_name: str, gltf_uri: str, mesh_index: Optional[int] = None) -> int:
    node_index = len(scene["nodes"])
    node: dict = {"name": character_name}
    if mesh_index is not None:
        node["mesh"] = mesh_index
    scene["nodes"].append(node)
    scene["scenes"][0]["nodes"].append(node_index)
    return node_index


def build_root_scene(manifest: Manifest, output_dir: str) -> dict:
    scene = build_empty_scene()
    for char in manifest.characters:
        gltf_uri = char.gltf_ref
        scene_path = os.path.join(output_dir, gltf_uri, gltf_uri)
        if os.path.exists(scene_path):
            with open(scene_path) as f:
                char_gltf = json.load(f)
            offsets = _merge_character_gltf(scene, char_gltf)
            char_root_idx = len(scene["nodes"])
            char_root_node = {"name": char.name}
            root_nodes = char_gltf.get("scenes", [{}])[0].get("nodes", [])
            if root_nodes:
                char_root_node["children"] = [
                    rn + offsets["nodes"] for rn in root_nodes
                ]
            scene["nodes"].append(char_root_node)
            scene["scenes"][0]["nodes"].append(char_root_idx)
            _embed_extensions_from(scene, char_gltf)
        else:
            add_character_node(scene, char.name, gltf_uri)
    return scene


def _merge_character_gltf(root_scene: dict, char_gltf: dict) -> dict[str, int]:
    offsets: dict[str, int] = {}
    merge_order = [
        "extensionsUsed", "extensionsRequired",
        "buffers", "bufferViews", "accessors",
        "samplers", "images", "textures", "materials",
        "meshes", "skins", "cameras", "nodes", "animations",
    ]

    def _offset(key: str) -> int:
        return offsets.get(key, 0)

    for section in merge_order:
        if section in ("extensionsUsed", "extensionsRequired"):
            for ext in char_gltf.get(section, []):
                if ext not in root_scene.setdefault(section, []):
                    root_scene[section].append(ext)
            continue

        items = char_gltf.get(section, [])
        if not items:
            offsets[section] = 0
            continue

        offsets[section] = len(root_scene.get(section, []))

        for item in items:
            cloned = copy.deepcopy(item)
            root_scene.setdefault(section, []).append(cloned)

        if section == "bufferViews":
            for bv in root_scene["bufferViews"][offsets["bufferViews"]:]:
                bv["buffer"] = bv.get("buffer", 0) + _offset("buffers")

        elif section == "accessors":
            for acc in root_scene["accessors"][offsets["accessors"]:]:
                acc["bufferView"] = acc.get("bufferView", 0) + _offset("bufferViews")

        elif section == "textures":
            for tex in root_scene["textures"][offsets["textures"]:]:
                if "source" in tex:
                    tex["source"] = tex["source"] + _offset("images")
                if "sampler" in tex:
                    tex["sampler"] = tex["sampler"] + _offset("samplers")

        elif section == "materials":
            for mat in root_scene["materials"][offsets["materials"]:]:
                _remap_texture_infos(mat, _offset("textures"))
                for chan in ("emissiveTexture", "normalTexture", "occlusionTexture"):
                    info = mat.get(chan)
                    if isinstance(info, dict) and "index" in info:
                        info["index"] = info["index"] + _offset("textures")

        elif section == "meshes":
            for mesh in root_scene["meshes"][offsets["meshes"]:]:
                for prim in mesh.get("primitives", []):
                    for attr, idx in prim.get("attributes", {}).items():
                        prim["attributes"][attr] = idx + _offset("accessors")
                    if "indices" in prim:
                        prim["indices"] = prim["indices"] + _offset("accessors")
                    if "material" in prim:
                        prim["material"] = prim["material"] + _offset("materials")
                    for target in prim.get("targets", []):
                        for attr, idx in target.items():
                            target[attr] = idx + _offset("accessors")

        elif section == "skins":
            for skin in root_scene["skins"][offsets["skins"]:]:
                if "inverseBindMatrices" in skin:
                    skin["inverseBindMatrices"] = skin["inverseBindMatrices"] + _offset("accessors")
                skin["joints"] = [j + _offset("nodes") for j in skin.get("joints", [])]
                if "skeleton" in skin:
                    skin["skeleton"] = skin["skeleton"] + _offset("nodes")

        elif section == "nodes":
            for node in root_scene["nodes"][offsets["nodes"]:]:
                if "children" in node:
                    node["children"] = [c + _offset("nodes") for c in node["children"]]
                if "mesh" in node:
                    node["mesh"] = node["mesh"] + _offset("meshes")
                if "skin" in node:
                    node["skin"] = node["skin"] + _offset("skins")
                if "camera" in node:
                    node["camera"] = node["camera"] + _offset("cameras")

        elif section == "animations":
            for anim in root_scene["animations"][offsets["animations"]:]:
                for ch in anim.get("channels", []):
                    if "target" in ch and "node" in ch["target"]:
                        ch["target"]["node"] = ch["target"]["node"] + _offset("nodes")
                for samp in anim.get("samplers", []):
                    if "input" in samp:
                        samp["input"] = samp["input"] + _offset("accessors")
                    if "output" in samp:
                        samp["output"] = samp["output"] + _offset("accessors")

    return offsets


def _remap_texture_infos(mat: dict, tex_offset: int) -> None:
    pbr = mat.get("pbrMetallicRoughness")
    if isinstance(pbr, dict):
        for key in ("baseColorTexture", "metallicRoughnessTexture"):
            info = pbr.get(key)
            if isinstance(info, dict) and "index" in info:
                info["index"] = info["index"] + tex_offset


def _embed_extensions_from(scene: dict, char_gltf: dict) -> dict:
    used = char_gltf.get("extensionsUsed", [])
    required = char_gltf.get("extensionsRequired", [])
    for ext in used:
        if ext not in scene.setdefault("extensionsUsed", []):
            scene["extensionsUsed"].append(ext)
    for ext in required:
        if ext not in scene.setdefault("extensionsRequired", []):
            scene["extensionsRequired"].append(ext)
    return scene
