# Roanoke Engine - Development Log

**Last Updated**: 2024-12-06
**Version**: v0.0.2-dev

A procedural 3D open-world game engine built in Rust with wgpu, set in 1580s Virginia.

---

## Project Overview

**Stack:** Rust, wgpu 0.19, winit 0.29, glam, egui
**Lines of Code:** ~61,000+ (excluding generated/target)
**Status:** Active Development

---

## Architecture

```
roanoke_game/src/           (~45,000 lines)
├── main.rs                 Game loop, rendering, UI
├── game_state.rs           Shared state management
├── village_manager.rs      Village streaming & NPC tracking
├── audio_system.rs         Audio playback
├── procedural_synth.rs     Procedural audio generation
├── safe_ops.rs             Safe math operations
├── systems_manager.rs      System orchestration
├── data_pipeline.rs        Data flow management
│
├── animals/                Dangerous wildlife system
│   ├── manager.rs          Quantum Spatial Cache (O(n) queries)
│   ├── behavior.rs         Hierarchical FSM AI
│   ├── spawner.rs          Chunk-based procedural spawning
│   ├── combat.rs           Damage processing
│   ├── spatial.rs          Spatial hashing
│   └── types.rs            10 species definitions
│
├── npc/                    NPC & village inhabitants
│   ├── npc_manager.rs      Central orchestration
│   ├── dialogue.rs         Branching dialogue trees
│   ├── relationships.rs    Memory & disposition tracking
│   └── trading.rs          Trade inventories & bartering
│
├── flora/                  Plant system
│   ├── growth.rs           Plant lifecycle
│   ├── harvest.rs          Harvesting mechanics
│   └── medicinal.rs        Medicinal plant effects
│
├── economy/                Economic system
│   ├── currency.rs         Wampum/Tobacco dual currency
│   ├── inventory.rs        Item storage
│   ├── item.rs             Item definitions
│   ├── loot.rs             Loot tables
│   └── drops.rs            Drop mechanics
│
├── progression/            Player progression
│   ├── skills.rs           Skill trees
│   ├── faction.rs          Faction definitions
│   ├── faction_manager.rs  Reputation tracking
│   ├── quests.rs           Quest system
│   └── player_state.rs     Player data
│
├── ecology/                Ecological simulation
│   ├── population.rs       Animal populations
│   ├── habitat.rs          Habitat management
│   └── consequences.rs     Player action effects
│
├── naval/                  Ship & naval combat
│   ├── ships.rs            Ship definitions
│   ├── sailing.rs          Sailing mechanics
│   ├── combat.rs           Naval combat
│   └── crew.rs             Crew management
│
├── weather/                Weather system
│   ├── storms.rs           Storm generation
│   └── effects.rs          Weather effects
│
└── encyclopedia/           Discovery tracking
    ├── entries.rs          Encyclopedia entries
    └── observer.rs         Discovery detection

crates/                     (~16,500 lines)
├── croatoan_core/          Window/event handling
│
├── croatoan_render/        GPU pipelines
│   ├── terrain_pipeline.rs Terrain rendering
│   ├── grass_pipeline.rs   Grass with wind
│   ├── tree_pipeline.rs    Tree instancing
│   ├── building_pipeline.rs Building rendering
│   ├── animal_orb_pipeline.rs Animal visualization
│   ├── shadows.rs          Shadow mapping
│   ├── frustum.rs          Frustum culling
│   ├── sky_pipeline.rs     Sky rendering
│   └── light_shaft_pipeline.rs God rays
│
├── croatoan_wfc/           World generation
│   ├── mesh_gen.rs         Terrain mesh
│   ├── biome.rs            Biome definitions
│   ├── biome_spawner.rs    Biome-based spawning
│   ├── vegetation.rs       Grass placement
│   ├── trees.rs            Tree placement
│   ├── rocks.rs            Rock placement
│   ├── villages.rs         Village world integration
│   ├── rivers.rs           River generation
│   ├── caves.rs            Cave generation
│   └── terrain.rs          Terrain generation
│
├── croatoan_procgen/       Procedural generation
│   ├── tree.rs             L-System trees
│   ├── grass.rs            Grass geometry
│   ├── rock.rs             Rock meshes
│   ├── longhouse.rs        Longhouse generation
│   ├── npc.rs              NPC appearance generation
│   └── village.rs          Village layout
│
└── croatoan_neural/        (Placeholder)

assets/shaders/             (~15 WGSL files)
├── terrain.wgsl            Terrain with lighting, shadows, fog
├── grass.wgsl              Wind animation, shadows
├── tree.wgsl               Tree rendering with fog
├── sky.wgsl                Sky gradient, clouds
├── water.wgsl              Water surface
├── water_compute.wgsl      Wave simulation
├── light_shafts.wgsl       God rays post-process
├── building.wgsl           Building rendering
├── animal_orb.wgsl         Animal visualization
└── detritus.wgsl           Ground clutter
```

---

## Implemented Systems

### Core Engine
- Chunked terrain streaming with LOD
- Dynamic time-of-day (T/Y keys)
- Weather system (5 types: Clear, PartlyCloudy, Overcast, Stormy, Foggy)
- Save/load system (JSON)
- First-person camera with collision
- egui-based UI

### World Generation
- Noise-based terrain (Perlin/FBM)
- Biome system (Ocean, Beach, Grassland, Forest, Mountain, Swamp)
- Native American longhouse villages (3 architectural styles)
- Rivers and caves
- Procedural vegetation (grass, trees, rocks)

### NPC System
- Village NPCs with roles (Chief, Shaman, Warrior, Hunter, etc.)
- Daily schedules with hourly activities
- Branching dialogue trees with faction checks
- Trading with reputation requirements
- Relationship and memory tracking

### Animal System
- 10 dangerous species with unique behaviors
- Hierarchical FSM AI (Idle, Alert, Pursue, Attack, Flee)
- Pack behavior for wolves
- Quantum Spatial Cache for O(n) queries
- Status effects (Bleeding, Poisoned, Stunned)

### Economy System
- Dual currency (Wampum + Tobacco)
- Item provenance tracking
- Loot tables with rarity
- Trading mechanics

### Progression
- Skill trees (Hunting, Archaeology, etc.)
- Faction reputation system
- Quest infrastructure

### Rendering
- Forward rendering with depth testing
- Shadow mapping (2048x2048) with texel snapping
- Frustum culling (~50% fewer draw calls)
- Distance-based LOD
- God rays post-process
- Fog system (atmosphere-driven)

---

## Performance Optimizations

### FPS Recovery (2024-12-05)
| Issue | Solution | Impact |
|-------|----------|--------|
| O(n²) animal queries | Quantum Spatial Cache | 50-80% FPS gain |
| Per-frame NPC buffer | Cached with dirty flags | 5-10% FPS gain |
| SystemTime RNG | PCG hash-based PRNG | 2-5% FPS gain |
| 247K tri trees | Simple 36-tri mesh | 2,600x reduction |
| Query radius 50u | Reduced to 25u | 4x fewer cells |

### Current Bottlenecks
- Rock/pebbles: 78K instances/chunk (needs distance culling)
- Fog: Only tints ground (needs atmospheric fix)

---

## Development History

### Foundation (2024-11)
1. Initial terrain generation with noise
2. Procedural grass system with wind
3. L-System tree generation
4. Shadow system implementation
5. Frustum culling
6. Distance-based LOD

### Systems Expansion (2024-12)
7. Animal system with 10 species
8. Native American village generation
9. NPC system with dialogue, trading, relationships
10. Economy system with dual currency
11. Flora/medicinal plant system
12. Naval combat framework
13. Faction and reputation system
14. FPS optimization (Quantum Spatial Cache)
15. Tree restoration (247K → 36 triangles)
16. Documentation consolidation

---

## How to Run

```bash
cargo run --release
```

**Controls:**
| Key | Action |
|-----|--------|
| WASD | Move |
| Mouse | Look |
| Space | Jump |
| T/Y | Time forward/back |
| \ | Fog density |
| Esc | Menu |

---

## Documentation

See `AGENT_DIRECTIVE.md` in project root for:
- Current project state
- Document navigation
- Development workflow

---

*This log tracks major development milestones. For current status, see `docs/status/MASTER_AUDIT.md`.*
