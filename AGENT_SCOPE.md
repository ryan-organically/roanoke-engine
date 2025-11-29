# Agent Scope

**Version**: 0.0.2-dev
**Last Update**: 2024-11-29
**Status**: Active Development

---

## Quick Context

Roanoke is a procedural open-world game engine built in Rust with wgpu. The agent should read this file first to understand current state and priorities.

---

## Current State

### Working Systems
- [x] Chunked terrain streaming with LOD
- [x] Procedural trees, rocks, vegetation, buildings (WFC)
- [x] Dynamic weather system (5 weather types)
- [x] Day/night cycle (T/Y keys)
- [x] Sky rendering with world-space clouds (slow drift)
- [x] Water compute shader
- [x] Save/Load system
- [x] Home screen menu (ROANOKE v0.0.1)
- [x] **Atmosphere Engine** - Time-based fog, ambient lighting
- [x] **Rock Variety** - 6 types: pebble, small, medium, boulder, flat, mossy
- [x] **Light Shafts** - God rays post-process with radial blur

### In Progress
- [ ] Terrain detail texturing
- [ ] Custom serif font for UI
- [ ] Settings menu functionality
- [ ] Audio system

### Known Issues
<!-- AGENT: Update this section when you find or fix bugs -->
| Issue | Location | Severity | Notes |
|-------|----------|----------|-------|
| WaterSystem unused | `water_system.rs` | Warning | Built but not wired up |
| Menu needs serif font | `main.rs:850-910` | Polish | Using Proportional, want serif |

---

## File Map

<!-- AGENT: Keep this updated as you modify files -->

### Documentation
| File | Purpose | Last Modified |
|------|---------|---------------|
| `AGENT_SCOPE.md` | Agent context and task tracking | 2024-11-29 |
| `TREE_SYSTEM_AUDIT.md` | **NEW** Tree system audit, asset pipeline spec | 2024-11-29 |
| `VERSION.md` | Version history | 2024-11-28 |

### Core Game
| File | Purpose | Last Modified |
|------|---------|---------------|
| `roanoke_game/src/main.rs` | Main loop, UI, game state | 2024-11-29 |
| `roanoke_game/src/weather_system.rs` | Weather state machine | 2024-11-29 |
| `roanoke_game/src/atmosphere.rs` | **NEW** Fog, light shafts, time-based FX | 2024-11-29 |
| `roanoke_game/src/water_system.rs` | Water compute (unused) | 2024-11-28 |

### Render Pipelines
| File | Purpose |
|------|---------|
| `croatoan_render/src/sky_pipeline.rs` | Sky dome + clouds |
| `croatoan_render/src/terrain_pipeline.rs` | Terrain mesh |
| `croatoan_render/src/grass_pipeline.rs` | Grass billboards |
| `croatoan_render/src/tree_pipeline.rs` | Instanced trees |
| `croatoan_render/src/building_pipeline.rs` | Instanced buildings |
| `croatoan_render/src/detritus_pipeline.rs` | Ground clutter |
| `croatoan_render/src/light_shaft_pipeline.rs` | **NEW** God rays post-process |

### Shaders
| File | Purpose |
|------|---------|
| `assets/shaders/sky.wgsl` | Sky gradient + cloud noise |
| `assets/shaders/terrain.wgsl` | Terrain with biome colors |
| `assets/shaders/water.wgsl` | Water surface |
| `assets/shaders/water_compute.wgsl` | Wave simulation |
| `assets/shaders/light_shafts.wgsl` | **NEW** God rays radial blur |

### Procgen
| File | Purpose |
|------|---------|
| `croatoan_wfc/src/buildings.rs` | Building placement |
| `croatoan_wfc/src/vegetation.rs` | Grass/flower placement |
| `croatoan_wfc/src/trees.rs` | Tree placement |
| `croatoan_wfc/src/rocks.rs` | Rock placement |

---

## Improvement Areas

<!-- AGENT: Add items here when you identify improvements. Mark done when complete. -->

### High Priority
- [ ] **FIX TREE SYSTEM** - See `TREE_SYSTEM_AUDIT.md` for full details
  - [ ] Delete broken `trees/trees9.obj` (966K lines, 247K faces)
  - [ ] Create clean trunk meshes (<2K faces each)
  - [ ] Implement leaf cluster billboard system
  - [ ] Add biome-based tree selection
- [ ] Wire up WaterSystem to main render loop
- [ ] Add custom font loading for serif UI text
- [ ] Implement Settings menu (audio volume, graphics quality)

### Medium Priority
- [ ] Add fog distance based on weather
- [ ] Improve tree LOD system
- [ ] Add ambient occlusion to buildings
- [ ] Rain particle system for Stormy weather
- [ ] **Terrain detail texturing** - Add detail normal maps, micro-variation, biome-specific textures
- [ ] **Texture atlas system** - Combine textures into atlases to reduce draw calls and GPU state changes

### Low Priority / Polish
- [ ] Loading screen progress bar
- [ ] Smooth camera transitions
- [ ] Screenshot functionality (F12)
- [ ] Debug overlay toggle (F3)

### Technical Debt
- [ ] Remove unused `WATER_SYSTEM` static in main.rs
- [ ] Consolidate chunk loading code (some duplication)
- [ ] Add error handling for asset loading failures

---

## Constants & Magic Numbers

<!-- AGENT: Document magic numbers here so they can be tuned -->

| Constant | Location | Value | Purpose |
|----------|----------|-------|---------|
| Chunk size | `main.rs` | 64 | Terrain chunk dimension |
| View distance | `main.rs` | 3 chunks | Load range |
| Cloud height | `sky.wgsl:88` | 500.0 | Virtual cloud plane |
| Menu font size | `main.rs:904` | 42/48 | Normal/hover |
| Title font size | `main.rs:855` | 96 | ROANOKE title |

---

## Session Notes

<!-- AGENT: Add dated notes about what you worked on -->

### 2024-11-29 (Session 2 - Tree Audit)
- **CRITICAL FINDING**: Tree system broken due to `trees/trees9.obj`
  - File is 966K lines, 247K faces - far too heavy for instancing
  - Leaf geometry bypasses material filter, renders as cardboard
  - Created `TREE_SYSTEM_AUDIT.md` with full analysis
- Documented asset pipeline requirements:
  - Trunk meshes: <2K faces per species
  - Bark textures: 512x512 tileable PNG
  - Leaf clusters: 256x256 alpha cutout PNG (5-7 leaves per cluster)
- Designed `TreeBiomeSpec` system for biome-aware tree placement
  - Oak forest (temperate), Mangrove (coastal), Dryland pine, Willow (riparian)
- Identified missing systems: Animal pathing, AI agentic humans (no code exists)
- L-system procgen has leaves disabled at `tree.rs:331-340` and `tree.rs:483-525`
  - Billboard orientation wrong (faces up, not camera)
  - Would need shader rewrite for camera-facing + wind

### 2024-11-29
- Created `atmosphere.rs` - full atmosphere engine with time-of-day fog
  - 7 time periods: Night, Dawn, Morning, Midday, Afternoon, Dusk, Evening
  - Foggy mornings with height-based density
  - Light shaft parameters ready for post-process
- Slowed cloud speed 6x (0.05 → 0.008), softer colors
- Added 6 rock types: pebble, small, medium, boulder, flat, mossy
- Dense pebble scatter with clustering
- Fog now uses atmosphere state instead of hardcoded values
- Created `light_shaft_pipeline.rs` - radial blur god rays
  - Shader: `light_shafts.wgsl` with occlusion-based rays
  - Calculates sun screen position from view-proj
- **Wired up light shaft post-process in main.rs**
  - Added `OffscreenTarget` struct for intermediate render texture
  - Scene renders to offscreen → light shaft pass composites to swapchain
  - Automatically handles window resize
  - Light shafts active when sun visible and atmosphere intensity > 0

### 2024-11-28
- Fixed cloud system: was screen-space, now world-space using inverse view-proj
- Added big bold ROANOKE v0.0.1 title to home screen
- Created VERSION.md and updated AGENT_SCOPE.md for agent visibility
- Menu items now 42pt brown, right-aligned

---

## Commands

```bash
# Build
cargo build --release

# Run
cargo run --release

# Check for errors without building
cargo check
```

---

*This document is agent-maintained. Update sections as you work.*
