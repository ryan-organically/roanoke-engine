# Procedural Grass System (Archived)

**Status:** Removed in favor of model-based grass (grass2/grass3 LOD system)
**Removal Date:** 2025-12-20

## Overview

The procedural grass system generated individual grass blades as geometry, with per-vertex colors and wind animation support. While visually decent, it caused performance issues due to high vertex counts.

## Components

### Generation (`crates/croatoan_wfc/src/vegetation.rs`)
- `generate_vegetation_for_chunk()` - Main entry point
- Generated up to ~250K vertices per chunk (1.5 density × 256² × 10 verts/blade)
- Used `GrassBladeRecipe` for blade geometry (4 segments, curved ribbon)

### Species System (`crates/croatoan_procgen/src/grass_species.rs`)
Claimed 5 species based on biome, but diversity was minimal in practice:

| Species | Biome | Height | Color | Notes |
|---------|-------|--------|-------|-------|
| Sea Oats | Beach | 0.6-2.2m | Bleached tan | Wispy, graceful droop |
| Cordgrass | Salt Marsh | 0.6-2.0m | Dark marsh green | Dense, stiff blades |
| Sawgrass | Grassland | 0.5-1.0m | Bright meadow green | Flowing motion |
| Forest Floor | Forest | 0.8-1.5m | Dark forest green | Pronounced droop |
| Alpine | Mountains | 0.2-0.5m | Mountain green | Short, compact tufts |

### Rendering (`crates/croatoan_render/src/grass_pipeline.rs`)
- `GrassPipeline` - Custom wgpu pipeline
- Shader: `assets/shaders/grass.wgsl`
- Features: Wind animation via `local_height` attribute, shadow receiving
- Max vertices: 1.2M per chunk

### Blade Generation (`crates/croatoan_procgen/src/grass.rs`)
- `generate_grass_blade()` - Single blade with curved ribbon geometry
- `generate_grass_patch()` - Batch generation with biome filtering
- Each blade: 10 vertices, 8 triangles (4 segments × 2 tris)

## Why Removed

1. **Performance:** High vertex counts (~1M+ per visible area)
2. **Visual uniformity:** Despite 5 species, all looked similar in practice
3. **Better alternative:** Model-based grass with LOD (8-55 tris per clump) scales better

## Replacement

- **grass3** (beach): Tall wispy sea oats style, 3 LODs (24/12/8 tris)
- **grass2** (inland): General ground cover, 3 LODs (55/18/12 tris)

Both use instanced rendering via `TreePipeline` for efficient batching.

## Files That Were Involved

```
crates/croatoan_wfc/src/vegetation.rs      - Generation logic
crates/croatoan_procgen/src/grass.rs       - Blade geometry
crates/croatoan_procgen/src/grass_species.rs - Species configs
crates/croatoan_render/src/grass_pipeline.rs - Rendering
assets/shaders/grass.wgsl                  - Wind shader
```

## Restoration

If needed in future, the code remains in the crate files but is not called.
Key entry point was `generate_vegetation_for_chunk()` in main.rs around line 1584.
