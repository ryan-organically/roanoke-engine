# Roanoke Engine - Development Log

A procedural 3D game engine built in Rust with wgpu, focusing on Git-friendly asset generation.

## Project Overview

**Stack:** Rust, wgpu 0.19, winit 0.29, glam, egui
**Lines of Code:** ~5,000 (excluding generated/target)
**Development Time:** ~12 hours

## Architecture

```
roanoke_game/              Main game executable (~1000 lines)
  ├── main.rs              Game loop, rendering, UI
  ├── player.rs            Player movement (67 lines)
  └── chunk_manager.rs     Chunk streaming infrastructure (140 lines)

crates/
  croatoan_core/           Window/event handling
  croatoan_render/         GPU pipelines (~1,400 lines)
    ├── terrain_pipeline.rs
    ├── grass_pipeline.rs
    ├── tree_pipeline.rs
    ├── shadows.rs
    ├── frustum.rs         Frustum culling (NEW)
    └── sky_pipeline.rs
  croatoan_wfc/            World generation (~620 lines)
    ├── mesh_gen.rs        Terrain mesh
    ├── vegetation.rs      Grass placement
    └── trees.rs           Tree placement
  croatoan_procgen/        Procedural generation (~580 lines)
    └── tree.rs            L-System trees
  croatoan_neural/         (Placeholder for AI)

assets/shaders/
  ├── terrain.wgsl         Dynamic lighting, shadows, fog
  ├── grass.wgsl           Wind animation, shadows
  ├── tree.wgsl
  └── sky.wgsl
```

## Implemented Systems

### Terrain Generation
- Noise-based heightmap (Perlin/FBM)
- Biome zones: Ocean → Beach → Scrubland → Forest
- 25 chunks (5x5 grid), 256 units each = 1.28km² world
- Background thread generation with progressive loading

### Vegetation System
- **Grass:** Curved ribbon geometry, biome-density based (10-100%)
- **Trees:** L-System branching, 7 species (Oak, Pine, Willow, Birch, Palm, Maple, Spruce)
- Per-chunk generation, automatic biome filtering

### Rendering
- Forward rendering with depth testing
- **Dynamic time-of-day system** (T/Y keys to change)
- **Directional sun lighting** with elevation-based color
- **Shadow mapping** (2048x2048) with texel-snapped projection
- **Frustum culling** - skip chunks outside camera view
- **Distance LOD** - grass culled at 350u, trees at 600u
- Fog system (400-800 unit range), color matches sky
- Wind animation in grass shader

### Game Systems
- Save/load system (JSON)
- Main menu with seed input
- First-person camera controls
- Loading screen with progress
- FPS and time display

## Technical Decisions

### Procedural Generation Philosophy
Instead of storing OBJ files (50MB+ each), store generation recipes (~500 bytes):
- Tree recipe → Infinite tree variations
- Grass recipe → Infinite blade variations
- Result: Entire world in <50KB of recipes

### Memory Management
Original approach crashed (8.3GB buffer). Fixed by:
- Reducing chunk count: 625 → 25
- Per-chunk pipelines instead of global accumulation
- L-System iterations: 6 → 3-4
- Tree density: 0.02 → 0.001

### Shadow Stability
Shadow flickering fixed by:
- Snapping light projection to texel grid
- Using player position (not camera target) for shadow center
- Adjusting depth bias (constant: 4, slope: 2.5)

## Performance Optimizations

| Optimization | Impact |
|--------------|--------|
| Frustum culling | ~50% fewer draw calls when looking at horizon |
| Grass distance cull | Eliminates invisible grass beyond fog |
| Tree distance cull | Reduces far-field rendering |
| Texel-snapped shadows | Eliminates shadow swimming |

## Known Issues

### Still Needs Work
- Sun billboard not implemented (sun direction works, no visual disk)
- Grass doesn't cast shadows (vertex stride mismatch - intentional skip)
- Full chunk streaming not integrated (infrastructure ready in chunk_manager.rs)
- Tree collision detection not implemented

## Performance Targets

| Metric | Current |
|--------|---------|
| Chunks | 25 (5x5) |
| World Size | 1.28 km² |
| Terrain vertices | ~105K |
| Grass vertices | ~1.5M |
| Tree vertices | ~300K |
| Total VRAM | ~100MB |
| Target FPS | 60 |

## How to Run

```bash
cargo run -p roanoke_game --release
```

**Controls:**
- WASD: Move
- Mouse: Look
- Space: Jump
- T: Advance time (+1 hour)
- Y: Reverse time (-1 hour)
- Enter seed → "New Game"

## Development History

1. Initial terrain generation with noise
2. Procedural grass system with wind animation
3. L-System tree generation (7 species)
4. Per-chunk vegetation with biome filtering
5. Memory optimization (8.3GB crash fix)
6. Shadow system implementation (partial)
7. Loading screen with progress tracking
8. **Dynamic directional lighting with elevation-based color**
9. **Time-of-day system with keyboard controls**
10. **Shadow stabilization (texel snapping)**
11. **Frustum culling for chunks**
12. **Distance-based LOD culling**
13. **Dynamic sky/fog color based on time**
14. **Chunk streaming infrastructure (ready for integration)**
