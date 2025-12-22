# Blender Asset Workflow

**Master reference for Claude agents preparing assets for Blender export.**

---

## Agent Checklist (Do This Every Time)

When a new asset type is requested, complete these steps BEFORE Blender exports:

### 1. Create Export Directory
```bash
mkdir -p "C:/dev/roanoke engine/assets/models/<category>/"
```

Categories: `trees/`, `shrubs/`, `rocks/`, `debris/`, `animals/`, `buildings/`

### 2. Create LOD Spec
Write a JSON spec at `docs/specs/<ASSET>_LOD_SPEC.json`:

```json
{
  "asset": "boulder",
  "description": "Large boulder rock for terrain scatter",

  "lods": {
    "lod0": { "triangles": "800-1200", "distance": "0-200 units" },
    "lod1": { "triangles": "150-250", "distance": "150-500 units" },
    "lod2": { "triangles": "40-60", "distance": "450-1200 units" }
  },

  "export": {
    "format": "GLB",
    "path": "assets/models/rocks/",
    "filenames": ["boulder_lod0.glb", "boulder_lod1.glb", "boulder_lod2.glb"]
  }
}
```

#### Required Spec Sections (All New Assets)

Every LOD spec **must** include these considerations:

1. **Textures & UVs**
   ```json
   "textures": {
     "uv_required": true,
     "material_setup": "Principled BSDF with Image Texture -> Base Color",
     "embedding": "Textures embedded in GLB (Images: Automatic)",
     "notes": "Engine reads TEXCOORD_0 for UVs, extracts baseColorTexture from PBR"
   }
   ```

2. **Decimation Targets** (explicit triangle counts for Blender)
   ```json
   "decimation_targets": {
     "lod0": 800,
     "lod1": 150,
     "lod2": 40
   }
   ```

3. **Pipeline Hook** (how it integrates with rendering)
   ```json
   "pipeline": {
     "renderer": "TreePipeline or new dedicated pipeline",
     "registry_keys": ["asset_lod0", "asset_lod1"],
     "lod_distances": { "lod0_max": 200.0, "lod1_max": 500.0 }
   }
   ```

See `docs/specs/CHEST_LOD_SPEC.json` for a complete example.

### 3. Register Mesh in Pipeline

Add to `main.rs` mesh registry section (~line 2330):

```rust
// Load boulder LODs
for lod in 0..=2 {
    let path = format!("assets/models/rocks/boulder_lod{}.glb", lod);
    if let Ok((positions, normals, uvs, indices, texture)) = load_glb_model(&path, ctx.device(), ctx.queue()) {
        let gpu_mesh = TreePipeline::create_mesh(ctx.device(), &positions, &normals, &uvs, &indices, texture);
        state.mesh_registry.insert(format!("boulder_lod{}", lod), gpu_mesh);
    }
}
```

### 4. Update Chunk Generation

In `rocks.rs` or equivalent, tag instances with LOD:
```rust
// Return LOD-aware instances
instances.push(("boulder_lod0".to_string(), transform));
```

### 5. Add Distance-Based LOD Rendering

In render loop (`main.rs` ~line 5328):
```rust
let boulder_lod = if dist < 200.0 { "lod0" }
                  else if dist < 500.0 { "lod1" }
                  else { "lod2" };
```

---

## Export Paths Quick Reference

| Asset Type | Export Path | Naming Convention |
|------------|-------------|-------------------|
| Trees | `assets/models/trees/` | `<species>_<variant>_lod<n>.glb` |
| Shrubs/Ferns | `assets/models/shrubs/` | `<type>_<variant>.glb` |
| Rocks | `assets/models/rocks/` | `<type>_lod<n>.glb` |
| Debris | `assets/models/debris/` | `<type>_<variant>_lod<n>.glb` |
| Animals | `assets/models/animals/` | `<species>.glb` |
| Buildings | `assets/models/buildings/` | `<type>_<variant>.glb` |

---

## Standard LOD Triangle Budgets

| Distance | LOD | Triangles | Use Case |
|----------|-----|-----------|----------|
| 0-200 | LOD0 | 800-1500 | Close detail |
| 150-500 | LOD1 | 150-300 | Mid range |
| 450-1200 | LOD2 | 40-80 | Far fill |

---

## Blender Export Settings

```
Format:        glTF Binary (.glb)
Transform:     +Y Up
Include:       Selected Objects only
Geometry:      Apply Modifiers, UVs, Normals
Materials:     Export (or vertex colors)
Compression:   OFF (CRITICAL - Draco not supported!)
```

**IMPORTANT: Draco mesh compression is NOT supported by the engine.**
If you see `KHR_draco_mesh_compression: Unsupported extension` errors, re-export with compression disabled.

---

## Blender Bridge Commands

Connect via TCP:9876 (start server in Blender N-panel > Bridge > Start Server)

```bash
# From project root
python blender-bridge/blender_client.py --ping
python blender-bridge/blender_client.py --export-gltf "C:/dev/roanoke engine/assets/models/rocks/boulder_lod0.glb"
```

---

## Related Specs

**Engine specs** (in `docs/specs/`):
- `BLENDER_BRIDGE_SPEC.md` - Full bridge protocol
- `BOULDER_LOD_SPEC.json` - Boulder LOD example
- `DEAD_CONIFER_LOD_SPEC.json` - Dead standing tree
- `DEADWOOD_LOD_SPEC.json` - Fallen log / debris

**Blender export specs** (in `blender-bridge/targets/`):
- `roanoke_birch.json` - Full tree export example
- `roanoke_pine.json` - Pine tree export
- `roanoke_deadwood.json` - Deadwood/debris export
- `roanoke_foliage.json` - Shrubs and grass

**Guides** (in `docs/guides/`):
- `HORSE_BLENDER_BRIDGE.md` - Animated model example
- `BEACH_GRASS_BLENDER_BRIDGE.md` - Foliage example
