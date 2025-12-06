# Roanoke Version History

**Current**: v0.0.1
**Code Location**: `main.rs:862` (menu display)

<!-- AGENT: When releasing a new version:
1. Update version number below
2. Update main.rs:862 to match
3. Move "Unreleased" items to new version section
4. Add date
-->

---

## Unreleased

<!-- AGENT: Add completed features here before release -->

### Critical Issues (2024-12-05 Audit) - RESOLVED

**FPS BLOCKERS (ALL FIXED 2024-12-05):**
- [x] O(n²) animal spatial queries -> Quantum Spatial Cache
- [x] Per-frame NPC instance buffer -> Cached with dirty flags
- [x] SystemTime RNG calls -> PCG hash-based PRNG
- [x] Per-frame pack morale recalculation -> Lazy evaluation

**VISUAL STATUS:**
- [x] Trees re-enabled with simple low-poly mesh (~36 tris vs 247K)
- [ ] Fog only tints ground, no atmospheric effect (Phase 3)
- [ ] 78K+ rock instances per chunk (Phase 1 planned)

### Added
- `FPS_OPTIMIZATION_ROADMAP.md` - Master performance recovery plan
- `MATERIAL_SHADER_AUDIT.md` - Shader/material system analysis
- Comprehensive audit of all rendering systems

### Fixed (2024-12-05 - FPS Fluidization System)
- [x] O(n²) → O(n) animal spatial queries via Quantum Spatial Cache (`manager.rs`)
- [x] Per-frame NPC instance allocation → cached with dirty flags (`village_manager.rs`)
- [x] SystemTime RNG → PCG-inspired hash-based PRNG (`behavior.rs`)
- [x] Per-frame pack morale recalc → lazy evaluation with dirty flags (`manager.rs`)
- [x] Query radius reduced 50 → 25 units (4x fewer cell checks)

### Fixed (2024-12-05 - Tree System Restoration)
- [x] Trees re-enabled - was disabled due to 247K face OBJ mesh
- [x] Created simple low-poly tree generator (cylinder trunk + icosphere canopy)
- [x] 2,600x triangle reduction: 94K → 36 triangles per tree
- [x] Updated tree shader to render green canopy + brown trunk

**Build Status**: ✅ Release build successful, game launches correctly

### Changed
- Updated `ROADMAP.md` with Phase 0 (FPS Emergency), Phase 6-8 (Trees, Fog, Rocks)
- Updated `TREE_SYSTEM_AUDIT.md` with performance impact data

### Known Issues
- Pebble density 1.2/m² = 78K instances per 256x256 chunk (Phase 1 planned)
- Fog only tints ground, no atmospheric effect (Phase 3 planned)

---

## v0.0.1 (2024-11-28) - Foundation

### Added
- Procedural terrain with chunked streaming (64x64 chunks, 3 chunk view distance)
- Tree, rock, vegetation, building placement via Wave Function Collapse
- Dynamic weather system: Clear, PartlyCloudy, Overcast, Stormy, Foggy
- Day/night cycle with T/Y key controls
- Sky rendering with world-space procedural clouds
- Water compute shader (built, not yet integrated)
- Save/Load game system with JSON serialization
- Home screen with stylistic right-aligned menu
- ROANOKE v0.0.1 title branding

### Technical
- Rust + wgpu (WebGPU) graphics backend
- egui for immediate-mode UI
- glam for math
- Inverse view-projection for proper sky ray reconstruction

---

## Roadmap

<!-- AGENT: Update estimates and check off completed items -->

### v0.0.2 - Polish (Target: TBD)
- [ ] Custom serif font
- [ ] Settings menu
- [ ] Wire up water system
- [ ] Audio foundation

### v0.0.3 - Gameplay (Target: TBD)
- [ ] Inventory system
- [ ] Basic interactions
- [ ] NPC foundation
  - [ ] NPC entity and component system
  - [ ] Behavior tree executor
  - [ ] Needs system with decay
- [ ] Native American longhouse villages
  - [ ] Longhouse procedural generation (Iroquoian style)
  - [ ] Village layout algorithm
  - [ ] Corn field and farming system
  - [ ] Fire pit and ceremonial dancing
  - [ ] Prayer sites and prayer behaviors
  - [ ] Daily NPC scheduling

### v0.1.0 - Alpha (Target: TBD)
- [ ] Multiplayer foundation
- [ ] Quest system
- [ ] Marketplace

---

## Version Numbering

```
MAJOR.MINOR.PATCH

0.0.x - Pre-alpha (current)
0.x.0 - Alpha features
1.0.0 - First stable release
```

---

*See AGENT_SCOPE.md for current work and improvement areas.*
