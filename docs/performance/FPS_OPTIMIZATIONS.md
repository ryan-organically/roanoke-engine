# FPS Optimization Summary

**Date**: 2024-12-05
**Target**: Fix 5 FPS issue and achieve playable framerates

---

## Critical Issues Identified

### 1. Grass System (MAIN CULPRIT)
- **Before**: 8 blades/m² × 256×256 chunks = 524K potential blades per chunk
- **Each blade**: 5 segments × 2 sides = 10 vertices
- **Total**: ~50K blades × 10 verts = **500K vertices per chunk just for grass**
- **With 25 chunks**: 12.5 MILLION grass vertices

### 2. Detritus System
- Generating thousands of rocks/logs per chunk
- Each with unique geometry (not instanced)

### 3. Tree Bunches
- Grid spacing of 18m = 196 bunches per chunk
- Each bunch has trees, bushes, rocks

### 4. Shader Complexity
- Cloud FBM: 5 iterations per pixel
- Light shafts: 64 texture samples per pixel
- Shadow map: 2048×2048

### 5. Animal Query
- Was querying `render_distance * 256 = 38,400 meters`
- Processing animals way beyond visible range

---

## Optimizations Applied

### Grass (26x + 40% reduction)
| Setting | Before | After | Reduction |
|---------|--------|-------|-----------|
| Density | 8.0 blades/m² | 0.3 blades/m² | **26x fewer** |
| Blade segments | 5 | 3 | **40% fewer verts/blade** |
| Render distance | 75% | 25% | **3x closer only** |

**Net grass reduction**: ~100x fewer grass vertices rendered

### Detritus (DISABLED)
- `detritus_density`: 0.02 → **0.0** (disabled)
- Re-enable after implementing instanced rendering

### Trees (4-6x reduction)
| Setting | Before | After |
|---------|--------|-------|
| Bunch grid spacing | 18m | 32m |
| Scattered tree density | 0.0008 | 0.0002 |
| Bunches per chunk | ~196 | ~64 |

### Chunk Loading (2-4x fewer chunks)
| Setting | Before | After |
|---------|--------|-------|
| Load radius | 2-5 | 1-2 |
| Render distance | 250 | 150 |
| Max visible chunks | ~81 | ~9-25 |

### Shadows (4x reduction)
- Shadow map: 2048 → 1024 pixels

### Shader Complexity
| Shader | Before | After |
|--------|--------|-------|
| Sky FBM iterations | 5 | 3 |
| Light shaft samples | 64 | 24 |

### Animal System
- Query radius: `render_distance * 256` → `render_distance * 0.5`
- Was querying 38,400m → now 75m

---

## Expected Results

Based on the changes:

| Metric | Before (est.) | After (est.) |
|--------|---------------|--------------|
| Grass vertices/frame | 12.5M | ~125K |
| Tree instances/frame | ~1000 | ~200 |
| Draw calls | 100+ | ~30 |
| Shadow map pixels | 4M | 1M |
| Light shaft ops/pixel | 64 | 24 |

**Expected FPS improvement**: 10-50x

---

## Render Distance Settings

With default `render_distance = 150`:
- Grass visible: 37.5m (very close only)
- Trees visible: 52.5m
- Detritus: disabled
- Buildings: 150m
- Chunks loaded: ~9

---

## Future Optimizations (If Needed)

1. **Instanced Rendering**
   - All trees share one mesh, one draw call
   - All rocks share one mesh, one draw call
   - Would allow re-enabling detritus

2. **LOD System**
   - High-detail grass/trees near camera
   - Low-detail or billboard at distance

3. **Occlusion Culling**
   - Skip rendering objects behind terrain/buildings

4. **Compute Shader Grass**
   - Generate grass on GPU instead of CPU

---

## Files Modified

- `crates/croatoan_wfc/src/vegetation.rs` - Grass density, segments, detritus
- `crates/croatoan_wfc/src/trees.rs` - Tree bunch grid, scatter density
- `roanoke_game/src/main.rs` - Render distances, chunk radius, shadows
- `roanoke_game/src/chunk_manager.rs` - Load/unload radius
- `crates/croatoan_render/src/light_shaft_pipeline.rs` - Sample count
- `assets/shaders/sky.wgsl` - FBM iterations

---

*Run the game and check FPS. If still slow, we can disable more systems or implement instancing.*
