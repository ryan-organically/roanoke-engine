# Agent Scope

**Version**: 0.0.2-dev
**Last Update**: 2024-12-06
**Status**: Active Development

---

## Primary Entry Point

**IMPORTANT**: See `AGENT_DIRECTIVE.md` for the unified agent entry point with current state, priorities, and document map.

This file (`AGENT_SCOPE.md`) is retained for session notes and detailed system tracking. Start with `AGENT_DIRECTIVE.md`.

---

## Quick Context

Roanoke is a procedural open-world game engine built in Rust with wgpu.

---

## Current State

### Working Systems (2024-12-06)
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
- [x] **FPS Optimization** - Quantum Spatial Cache (O(n) queries)
- [x] **Trees Restored** - Simple low-poly (36 tris vs 247K)
- [x] **Native Villages** - Longhouse generation with NPCs
- [x] **Frustum Culling** - ~50% fewer draw calls
- [x] **Shadow System** - Texel snapping, stable projection

### In Progress
- [ ] Rock optimization (78K instances/chunk)
- [ ] Fog system fix (only tints ground)
- [ ] Terrain detail texturing
- [ ] Custom serif font for UI
- [ ] Settings menu functionality
- [ ] Audio system

### Known Issues
<!-- AGENT: Update this section when you find or fix bugs. See KNOWN_ISSUES.md for full list -->
| Issue | Location | Severity | Notes |
|-------|----------|----------|-------|
| WaterSystem unused | `water_system.rs` | Warning | Built but not wired up |
| Menu needs serif font | `main.rs:850-910` | Polish | Using Proportional |
| Flora enum mismatch | `flora/medicinal.rs` | Build-blocking | See KNOWN_ISSUES.md |

---

## File Map

<!-- AGENT: Keep this updated as you modify files -->

### Documentation
| File | Purpose | Last Modified |
|------|---------|---------------|
| `AGENT_SCOPE.md` | Agent context and task tracking | 2024-11-29 |
| `TREE_SYSTEM_AUDIT.md` | Tree system audit, asset pipeline spec | 2024-11-29 |
| `NPC_VILLAGE_SPECIFICATION.md` | **NEW** NPC behavior & longhouse village spec | 2024-12-05 |
| `VERSION.md` | Version history | 2024-12-05 |

### Core Game
| File | Purpose | Last Modified |
|------|---------|---------------|
| `roanoke_game/src/main.rs` | Main loop, UI, game state | 2024-12-05 |
| `roanoke_game/src/weather_system.rs` | Weather state machine | 2024-11-29 |
| `roanoke_game/src/atmosphere.rs` | Fog, light shafts, time-based FX | 2024-11-29 |
| `roanoke_game/src/water_system.rs` | Water compute (unused) | 2024-11-28 |
| `roanoke_game/src/village_manager.rs` | **NEW** Village tracking and streaming | 2024-12-05 |

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
| `croatoan_wfc/src/villages.rs` | **NEW** Village world integration |
| `croatoan_procgen/src/longhouse.rs` | **NEW** Longhouse mesh generation |
| `croatoan_procgen/src/npc.rs` | **NEW** NPC appearance and mesh generation |
| `croatoan_procgen/src/village.rs` | **NEW** Village layout, fire pits, corn fields |

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

### 2024-12-06 (Documentation Consolidation)
- **CREATED**: `AGENT_DIRECTIVE.md` - Unified agent entry point
  - Combined guidance from AGENT_SCOPE, MASTER_AUDIT, ROADMAP
  - Single source of truth for agent onboarding
  - Document map showing which docs to read when
- **ARCHIVED**: `AGENT_SCOPE_LOCK.md` - Stale coordination lock
- **UPDATED**: VERSION.md - Fixed completion status (FPS blockers marked done)
- **UPDATED**: MASTER_AUDIT.md - Added AGENT_DIRECTIVE.md as primary link
- **UPDATED**: ROADMAP.md - Split into completed/pending phases
- **UPDATED**: AGENT_SCOPE.md - Redirect to AGENT_DIRECTIVE.md
- **UPDATED**: ONBOARDING.md - New document structure

### 2024-12-05 (Native American Village System + Game Integration)
- **IMPLEMENTED**: Complete Native American village generation system
  - `croatoan_procgen/src/longhouse.rs` - Iroquoian longhouse mesh generation
    - 3 architectural styles: Iroquoian, Algonquian, Coastal
    - Procedural frame poles, bark shell, smoke holes, interior hearths
    - Configurable by family units (3-10), determining length
  - `croatoan_procgen/src/npc.rs` - NPC appearance and character generation
    - 9 roles: Chief, Shaman, Warrior, Hunter, Farmer, etc.
    - Procedural appearance: height, build, skin tone, hair, clothing
    - Culturally-authentic name generation using syllable patterns
  - `croatoan_procgen/src/village.rs` - Village layout generation
    - Longhouses arranged in oval around central ceremonial fire
    - Corn fields with Three Sisters mounds (5 growth stages)
    - Prayer sites at cardinal directions
    - Fire pit mesh generation (ceremonial and domestic)
  - `croatoan_wfc/src/villages.rs` - World integration
    - Terrain-aware site selection (elevation, flatness, spacing)
    - Chunk-based structure streaming for rendering
- **GAME INTEGRATION**:
  - `roanoke_game/src/village_manager.rs` - Village tracking and streaming
    - Discovers villages at game start (2km radius, max 10 villages)
    - Per-chunk structure queries for LOD streaming
    - Integrates with building pipeline for rendering
  - Village structures rendered using existing BuildingPipeline
    - Longhouses, fire pits, corn plants added to chunk buildings
    - Vertex format: position[3], normal[3], uv[2], color[3]
    - World transform applied per-structure
  - VillageManager initialized on New Game / Load Game
- **DOCUMENTATION**: Updated `NPC_VILLAGE_SPECIFICATION.md` with implementation details
- **TESTS**: All procgen and village integration tests passing
- **FIXED**: wgpu API compatibility in `animal_orb_pipeline.rs`

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
  - **NPC/Village spec created**: See `NPC_VILLAGE_SPECIFICATION.md` for full design
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
