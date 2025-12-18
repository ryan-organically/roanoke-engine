# Beach Grass Export Spec

## Quick Reference

| File | Export Path | Size |
|------|-------------|------|
| `beach_grass_0` | `C:/dev/roanoke engine/assets/models/shrubs/beach_grass_0.glb` | ~0.4m tall |
| `beach_grass_1` | `C:/dev/roanoke engine/assets/models/shrubs/beach_grass_1.glb` | ~0.5m tall |
| `beach_grass_2` | `C:/dev/roanoke engine/assets/models/shrubs/beach_grass_2.glb` | ~0.6m tall |
| `beach_grass_3` | `C:/dev/roanoke engine/assets/models/shrubs/beach_grass_3.glb` | ~0.8m tall |

---

## Export Steps

### 1. Prep Each Variation

```
Ctrl+A → All Transforms (apply scale/rotation)
Origin → Base center (bottom of grass clump, Y=0)
Normals → Recalculate Outside
```

### 2. Export Settings (GLB)

```
Format:        glTF Binary (.glb)
Transform:     +Y Up, +Z Forward
Include:       Selected Objects only
Geometry:      Apply Modifiers, UVs, Normals, Vertex Colors
Compression:   OFF (Draco not needed for small meshes)
```

### 3. Export Commands

Select each variation and export:

```bash
# Variation 0 (shortest - lower beach)
C:/dev/roanoke engine/assets/models/shrubs/beach_grass_0.glb

# Variation 1 (mid-height)
C:/dev/roanoke engine/assets/models/shrubs/beach_grass_1.glb

# Variation 2 (taller)
C:/dev/roanoke engine/assets/models/shrubs/beach_grass_2.glb

# Variation 3 (tallest - treeline edge)
C:/dev/roanoke engine/assets/models/shrubs/beach_grass_3.glb
```

---

## Model Specs

| Property | Value |
|----------|-------|
| Verts | ~168 (your current clump) |
| Origin | Base center, Y=0 at ground |
| Scale | 1 unit = 1 meter |
| Material | Vertex colors preferred |
| Colors | Sandy tan base → bleached gold tips |

### Variation Guidelines

Make each visually distinct by varying:
- Blade count (5-12 per clump)
- Curvature (upright vs wind-bent)
- Spread (tight vs loose clump)
- Height mix within clump

---

## Engine Behavior

Spawns on **upper beach** (height 2.0m - 5.0m):
- Sparse near water, denser toward treeline
- Shorter variations (0,1) spawn lower
- Taller variations (2,3) spawn near forest edge
- Much lower density than procedural grass
