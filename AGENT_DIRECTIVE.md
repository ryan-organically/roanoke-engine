# Agent Directive - Roanoke Engine

**Version**: 0.0.2-dev
**Last Updated**: 2024-12-06
**Status**: Active Development

---

## Quick Start for Agents

1. **Read this file first** - Current state and priorities
2. **Check `docs/status/KNOWN_ISSUES.md`** - Build-blocking issues
3. **Review `docs/status/MASTER_AUDIT.md`** - Technical status details
4. **See `docs/status/ROADMAP.md`** - Implementation phases

---

## Current State Summary

### Completed Systems
| System | Status | Notes |
|--------|--------|-------|
| FPS Optimization | COMPLETE | Quantum Spatial Cache deployed |
| Trees | COMPLETE | Simple low-poly (36 tris), was 247K |
| Terrain Streaming | COMPLETE | Chunked LOD |
| Weather System | COMPLETE | 5 types |
| Day/Night Cycle | COMPLETE | T/Y keys |
| Village Generation | COMPLETE | Native American longhouses |
| Frustum Culling | COMPLETE | ~50% fewer draws |
| Shadow System | COMPLETE | Texel snapping |

### In Progress
| System | Status | Blocking? |
|--------|--------|-----------|
| Fog | Broken - only tints ground | No |
| Rock/Pebble | 78K instances/chunk | No |
| Materials | Procedural-only, no textures | No |
| Animals | Orbs work, no meshes | No |

### Pending Work (Priority Order)
1. **Phase 1**: Rock/Pebble optimization (distance culling)
2. **Phase 3**: Fog system fix (atmospheric fog)
3. **Phase 4**: Material/texture system (needs asset generation)
4. **Phase 5**: Water improvements
5. **Future**: Animal meshes (leave orbs for now)

---

## Documentation Structure

```
/                           <- You are here
├── AGENT_DIRECTIVE.md      <- START HERE (this file)
├── README.md               <- Project overview
│
└── docs/
    ├── status/             <- Project status (check frequently)
    │   ├── MASTER_AUDIT.md     # Technical status & phases
    │   ├── ROADMAP.md          # Implementation roadmap
    │   ├── VERSION.md          # Version history
    │   ├── KNOWN_ISSUES.md     # Build-blocking bugs
    │   └── PROJECT_STRUCTURE.md # Architecture diagrams
    │
    ├── guides/             <- How-to documents
    │   ├── ONBOARDING.md       # Build & run instructions
    │   ├── MARCHING_ORDERS.md  # User asset tasks
    │   └── AGENT_SCOPE.md      # Detailed agent context
    │
    ├── specs/              <- System specifications (13 files)
    │   ├── ANIMAL_SYSTEM_SPEC.md
    │   ├── DOCILE_FAUNA_SPEC.md
    │   ├── FLORA_PLANT_SYSTEM_SPEC.md
    │   ├── NPC_VILLAGE_SPECIFICATION.md
    │   ├── FACTION_SYSTEM_SPEC.md
    │   ├── ECONOMY_IMPLEMENTATION.md
    │   ├── MARKETPLACE_LOOT_SYSTEM_SPEC.md
    │   ├── HUNTING_SKILL_TREE_SPEC.md
    │   ├── ARCHAEOLOGY_SKILL_TREE_SPEC.md
    │   ├── BOW_WEAPON_WHEEL_SPEC.md
    │   ├── NAVAL_COMBAT_SHIP_BATTLES_SPEC.md
    │   ├── NATURE_DISCOVERY_ENCYCLOPEDIA_SPEC.md
    │   └── NATURE_MORALITY_WEATHER_KARMA_SPEC.md
    │
    ├── performance/        <- Optimization docs
    │   ├── FPS_OPTIMIZATION_ROADMAP.md
    │   ├── FPS_OPTIMIZATIONS.md
    │   ├── VRAM_OBSERVABILITY_SPEC.md
    │   ├── TREE_SYSTEM_AUDIT.md
    │   └── MATERIAL_SHADER_AUDIT.md
    │
    ├── vision/             <- Long-term planning
    │   ├── TRILLION_DOLLAR_VISION.md
    │   └── DECADE_FINANCIAL_ROADMAP.md
    │
    ├── technical/          <- Technical references
    │   ├── PROCGEN_FRAMEWORK.md
    │   ├── VOBJ_SPECIFICATION.md
    │   └── DEVELOPMENT_LOG.md
    │
    ├── assets/             <- Asset requirements
    │   └── EXTERNAL_ASSETS_NEEDED.md
    │
    └── archive/            <- Archived/completed
        ├── AGENT_SCOPE_LOCK.md
        └── RECOVERY.md
```

---

## Document Quick Reference

### Status Docs (Check First)
| Document | Path | Purpose |
|----------|------|---------|
| Master Audit | `docs/status/MASTER_AUDIT.md` | Technical status |
| Roadmap | `docs/status/ROADMAP.md` | Implementation phases |
| Known Issues | `docs/status/KNOWN_ISSUES.md` | Build blockers |
| Version | `docs/status/VERSION.md` | Release history |

### Specs (Read When Working On System)
| System | Document |
|--------|----------|
| Animals | `docs/specs/ANIMAL_SYSTEM_SPEC.md` |
| Plants | `docs/specs/FLORA_PLANT_SYSTEM_SPEC.md` |
| NPCs/Villages | `docs/specs/NPC_VILLAGE_SPECIFICATION.md` |
| Economy | `docs/specs/ECONOMY_IMPLEMENTATION.md` |
| Factions | `docs/specs/FACTION_SYSTEM_SPEC.md` |
| Combat | `docs/specs/NAVAL_COMBAT_SHIP_BATTLES_SPEC.md` |

### Performance (Optimization Work)
| Document | Path |
|----------|------|
| FPS Roadmap | `docs/performance/FPS_OPTIMIZATION_ROADMAP.md` |
| Tree Audit | `docs/performance/TREE_SYSTEM_AUDIT.md` |
| Shader Audit | `docs/performance/MATERIAL_SHADER_AUDIT.md` |

---

## Critical Code Files

### Performance-Critical
| File | Purpose |
|------|---------|
| `roanoke_game/src/animals/manager.rs` | Animal spatial queries |
| `roanoke_game/src/animals/behavior.rs` | Animal AI |
| `roanoke_game/src/village_manager.rs` | NPC instances |

### Rendering
| File | Purpose |
|------|---------|
| `roanoke_game/src/main.rs` | Main loop, render calls |
| `assets/shaders/*.wgsl` | All GPU shaders |
| `crates/croatoan_render/src/` | Render pipelines |

### World Generation
| File | Purpose |
|------|---------|
| `crates/croatoan_wfc/src/` | Wave Function Collapse |
| `crates/croatoan_procgen/src/` | Procedural meshes |

---

## Development Workflow

### Before Starting Work
```bash
# 1. Check for issues
cargo check

# 2. If build fails, check docs/status/KNOWN_ISSUES.md

# 3. Run the game
cargo run --release
```

### After Making Changes
1. Update relevant documentation in `docs/`
2. Add session notes to this file (below)
3. Mark completed items in `docs/status/VERSION.md`
4. Update `docs/status/KNOWN_ISSUES.md` if bugs found/fixed

---

## Session Notes

### 2024-12-06 (Documentation Reorganization)
- **REORGANIZED**: All docs into `docs/` folder structure
  - `docs/status/` - Project status (MASTER_AUDIT, ROADMAP, VERSION, KNOWN_ISSUES)
  - `docs/guides/` - How-to docs (ONBOARDING, MARCHING_ORDERS, AGENT_SCOPE)
  - `docs/specs/` - 13 system specifications
  - `docs/performance/` - Optimization docs
  - `docs/vision/` - Long-term planning
  - `docs/technical/` - Technical references
  - `docs/archive/` - Archived docs
- Root now contains only `AGENT_DIRECTIVE.md` and `README.md`

### 2024-12-06 (Documentation Consolidation)
- Created `AGENT_DIRECTIVE.md` as unified agent entry point
- Consolidated overlapping roadmaps and checklists
- Archived stale `AGENT_SCOPE_LOCK.md`
- Updated VERSION.md with proper completion status

### 2024-12-05 (FPS Fluidization + Tree Restoration)
- **COMPLETE**: Phase 0 FPS Emergency
  - O(n2) -> O(n) via Quantum Spatial Cache
  - Cached NPC instances with dirty flags
  - PCG hash-based PRNG replaces SystemTime
  - Lazy pack morale calculation
  - Query radius 50 -> 25 units
- **COMPLETE**: Tree System Restoration
  - Simple low-poly trees (36 tris vs 247K)
  - Trees re-enabled in render loop
  - Fog added to tree shader

### 2024-12-05 (Native American Villages)
- **COMPLETE**: Village generation system
  - Longhouse mesh generation (Iroquoian style)
  - NPC appearance and role generation
  - Village layout with fire pits, corn fields
  - World integration via VillageManager

### 2024-11-29 (Tree Audit)
- **FINDING**: trees9.obj has 247K faces - disabled
- Created TREE_SYSTEM_AUDIT.md
- Designed TreeBiomeSpec system

### 2024-11-28 (Foundation)
- World-space clouds
- Home screen menu
- VERSION.md and AGENT_SCOPE.md

---

## Controls Reference

| Key | Action |
|-----|--------|
| WASD | Move |
| Mouse | Look |
| Space | Jump |
| T/Y | Time forward/back |
| \ | Fog density |
| Esc | Menu |

---

## Do NOT Touch (Per User)
- Animal orb system - works fine for gameplay testing
- Blockchain/NFT integration - not needed
- Multiplayer - single-player first

---

*This document is the primary agent entry point. Update session notes as you work.*
