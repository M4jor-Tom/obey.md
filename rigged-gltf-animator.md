---
name: rigged-gltf-animator
description: >
  Generates procedural or choreographed animations for rigged GLTF files and writes them into the
  GLTF's `animations` field. Use this skill whenever the user wants to animate a GLTF model,
  make two GLTF characters fight/dance/interact, add movement to a rigged 3D file, or produce
  an animated GLTF output from one or more rigged .gltf/.glb sources. Trigger even if the user
  just says "make X do something" or "animate X" while a .gltf file is involved.
compatibility:
  tools:
    - gltf_to_png   # render a GLTF frame to PNG for visual inspection
    - gltf_to_webm  # render a GLTF animation to video for playback review
  python: ">=3.8"
  stdlib_only: true   # no external deps required; pure json + struct + base64
---

# rigged-gltf-animator

Produce animation data for one or more rigged GLTF files, writing the result back into the
`animations` array of the output GLTF. The skill works entirely from the GLTF's structural
data — it never loads textures, mesh geometry, or material data.

---

## Step 0 — Understand the request

Identify:
- **Input files**: one or more `.gltf` (JSON) or `.glb` (binary) paths
- **Animation goal**: fight, walk, idle, wave, interact, custom choreography, etc.
- **Output path(s)**: where to write the animated GLTF(s) (default: add `_animated` suffix)

If the request is vague ("make them fight"), invent a plausible choreography and narrate it to
the user so they know what to expect.

---

## Step 1 — Parse the rig (GLTF structure only)

Read the `.gltf` JSON (or extract the JSON chunk from a `.glb`).  
**Only look at these top-level keys** — skip everything else for speed:

| Key | Why |
|-----|-----|
| `nodes` | bone names, parent–child hierarchy, default TRS (translation/rotation/scale) |
| `skins` | skeleton root, joint list (array of node indices) |
| `scenes` / `scene` | root nodes |
| `accessors` | needed to understand existing animation data types/counts |
| `bufferViews` | byte layout for reading/writing binary blobs |
| `buffers` | total byte lengths; the actual `uri` data if embedded |
| `animations` | existing clips (read so new clips don't overwrite them) |

**Do NOT read**: `meshes`, `materials`, `textures`, `images`, `samplers` (texture samplers),
`cameras`, `extensions` — they waste tokens and aren't needed for animation.

### Validation

After parsing, run these checks:

```
✓  At least one skin exists                          → else: write error.log, abort
✓  skin.joints is non-empty                          → else: write error.log, abort
✓  All joint indices resolve to valid nodes          → else: write error.log, abort
✓  nodes referenced by joints have names             → warn in error.log, continue
✓  File is valid JSON (for .gltf) or has JSON chunk  → else: write error.log, abort
```

**error.log format** (write to same directory as input):
```
ERROR: <filename>
Reason: <short description>
Detail: <what was found vs. what was expected>
Timestamp: <ISO 8601>
```

### Build a bone map

Collect from `skins[0].joints` (or all skins if multiple):

```python
bones = {
    node_index: {
        "name": nodes[i].get("name", f"bone_{i}"),
        "translation": nodes[i].get("translation", [0,0,0]),
        "rotation":    nodes[i].get("rotation",    [0,0,0,1]),   # xyzw quaternion
        "scale":       nodes[i].get("scale",        [1,1,1]),
        "children":    nodes[i].get("children",     []),
    }
    for i in skin["joints"]
}
```

---

## Step 2 — Design the animation

Use the bone map to plan keyframes. Common bone name patterns to recognise:

| Pattern | Role |
|---------|------|
| `*hip*`, `*pelvis*`, `*root*` | root / centre of mass |
| `*spine*`, `*chest*`, `*torso*` | torso chain |
| `*shoulder*`, `*upper_arm*`, `*forearm*`, `*hand*` | arm chain |
| `*thigh*`, `*leg*`, `*shin*`, `*foot*` | leg chain |
| `*head*`, `*neck*` | head chain |
| `*sword*`, `*weapon*`, `*shield*` | held objects |

Match case-insensitively. If bones can't be identified by name, fall back to hierarchy
position (root = highest-level joint).

### Keyframe design guidelines

- Use **quaternion rotations** (xyzw) — GLTF's `rotation` sampler always expects these.
- Keep timing realistic: 24–30 fps equivalent; most actions 0.5–2 s.
- For fights/interactions between two models, **synchronise timestamps** across both files
  so the exported clips can be played together at t=0.
- Interpolation type: prefer `LINEAR`; use `CUBICSPLINE` for smoother arcs on major joints.
- Always include a **rest keyframe at t=0** matching the bind pose so the model doesn't snap.

See `references/animation-recipes.md` for pre-built keyframe tables for common actions
(idle, walk, punch, sword swing, dodge, etc.).

---

## Step 3 — Encode animation data into GLTF binary

GLTF animations require **accessors → bufferViews → buffer** in addition to the
`animations[].samplers` and `animations[].channels` arrays.

### 3a. Serialise keyframe data

For each animated channel you need two float arrays:
- **input**: timestamps `[t0, t1, t2, …]` in seconds (monotonically increasing, ≥ 2 values)
- **output**: values at each timestamp
  - translation: `[x,y,z]` per timestamp → stride 3
  - rotation: `[x,y,z,w]` per timestamp → stride 4
  - scale: `[x,y,z]` per timestamp → stride 3

Pack as **little-endian IEEE 754 float32**:

```python
import struct, base64

def pack_floats(values: list[float]) -> bytes:
    return struct.pack(f"<{len(values)}f", *values)
```

### 3b. Append to buffer

Collect all byte blobs in order. Record each blob's `byteOffset` and `byteLength`.
Concatenate into one buffer.

If the original GLTF has a `buffers[0].uri` that is a `data:` URI, decode it, append,
and re-encode. If it is an external `.bin` file path, write it separately.
If there are no existing buffers, create `buffers[0]` from scratch.

### 3c. Create bufferViews

One bufferView per blob:

```json
{
  "buffer": 0,
  "byteOffset": <offset>,
  "byteLength": <length>,
  "target": 34963
}
```

(`target` 34963 = ELEMENT_ARRAY_BUFFER, acceptable for animation data; omit if you prefer.)

### 3d. Create accessors

One accessor per timestamp array and one per value array:

```json
{
  "bufferView": <index>,
  "byteOffset": 0,
  "componentType": 5126,
  "count": <number of keyframes>,
  "type": "SCALAR",          // for input (timestamps)
  "min": [<t_min>],
  "max": [<t_max>]
}
```

```json
{
  "bufferView": <index>,
  "byteOffset": 0,
  "componentType": 5126,
  "count": <number of keyframes>,
  "type": "VEC3",            // or "VEC4" for rotation
  "normalized": false
}
```

(`componentType` 5126 = FLOAT)

### 3e. Create animation samplers and channels

```json
{
  "name": "MyClip",
  "samplers": [
    { "input": <input_accessor_index>, "interpolation": "LINEAR", "output": <output_accessor_index> }
  ],
  "channels": [
    {
      "sampler": 0,
      "target": { "node": <node_index>, "path": "rotation" }
    }
  ]
}
```

Append to `gltf["animations"]` (create the array if absent).

---

## Step 4 — Write output

1. Serialise the modified GLTF dict back to JSON (`json.dumps`, indent=2).
2. Write to `<original_stem>_animated.gltf` (or user-specified path).
3. If the buffer was embedded as a data URI, write it inline; if external, write the `.bin`.
4. Print a summary: clip name, duration, number of channels, output path.

---

## Step 5 — Visual verification (optional but recommended)

If `gltf_to_png` is available, render a mid-animation frame:
```
gltf_to_png <output.gltf> --frame 0.5 --out preview.png
```

If `gltf_to_webm` is available, render the full clip:
```
gltf_to_webm <output.gltf> --out preview.webm
```

Show the preview to the user and ask if they want adjustments.

---

## Error handling cheat-sheet

| Situation | Action |
|-----------|--------|
| No `skins` key | Write error.log: "Model is unrigged" |
| `skins[].joints` is empty | Write error.log: "Skin has no joints" |
| Invalid JSON / not a GLTF | Write error.log: "Invalid GLTF format" |
| GLB magic bytes wrong | Write error.log: "Not a valid GLB file" |
| Joint node index out of range | Write error.log + warn, skip that joint |
| Bone names unrecognisable | Continue; animate root/spine only; note in output |
| No root bone found | Use `skins[0].joints[0]` as fallback root |