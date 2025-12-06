# Roanoke Engine

Procedural open-world game engine built in Rust with wgpu.

**Version**: 0.0.2-dev | **Status**: Active Development

---

## For AI Agents

**Start here:** [`AGENT_DIRECTIVE.md`](AGENT_DIRECTIVE.md)

This is the single entry point for all agent sessions with:
- Current project state
- Document navigation
- Session notes
- Development workflow

---

## Quick Start

```bash
# Build
cargo build --release

# Run
cargo run --release
```

### Controls
| Key | Action |
|-----|--------|
| WASD | Move |
| Mouse | Look |
| T/Y | Time forward/back |
| Esc | Menu |

---

## Project Structure

```
roanoke-engine/
├── AGENT_DIRECTIVE.md      <- Agent entry point
├── README.md               <- This file
│
├── docs/                   <- All documentation
│   ├── status/             # Project status (MASTER_AUDIT, ROADMAP, VERSION)
│   ├── guides/             # How-to docs (ONBOARDING, MARCHING_ORDERS)
│   ├── specs/              # System specifications (13 files)
│   ├── performance/        # Optimization docs
│   ├── vision/             # Long-term planning
│   ├── technical/          # Technical references
│   ├── assets/             # Asset requirements
│   └── archive/            # Archived docs
│
├── roanoke_game/           <- Main game binary
│   └── src/
│       ├── main.rs         # Entry point
│       ├── animals/        # Animal system
│       ├── flora/          # Plant system
│       ├── npc/            # NPC system
│       └── economy/        # Economy system
│
├── crates/                 <- Engine libraries
│   ├── croatoan_core/      # Camera, player, utilities
│   ├── croatoan_render/    # GPU pipelines
│   ├── croatoan_procgen/   # Procedural generation
│   └── croatoan_wfc/       # Wave Function Collapse
│
├── assets/                 <- Game assets
│   ├── shaders/            # WGSL shaders
│   ├── ui/                 # UI images
│   └── textures/           # Texture assets
│
└── marketing/              <- Marketing documentation
```

---

## Documentation Map

| Need | Go To |
|------|-------|
| Agent starting point | `AGENT_DIRECTIVE.md` |
| Technical status | `docs/status/MASTER_AUDIT.md` |
| Implementation phases | `docs/status/ROADMAP.md` |
| Build-blocking bugs | `docs/status/KNOWN_ISSUES.md` |
| Build instructions | `docs/guides/ONBOARDING.md` |
| System specs | `docs/specs/` |
| Performance optimization | `docs/performance/` |

---

## Tech Stack

- **Language**: Rust
- **Graphics**: wgpu (WebGPU)
- **UI**: egui
- **Math**: glam

---

## Current State

### Working
- Chunked terrain streaming with LOD
- Procedural trees, rocks, vegetation, buildings (WFC)
- Dynamic weather system (5 types)
- Day/night cycle
- Native American village generation
- FPS-optimized animal system (Quantum Spatial Cache)

### In Progress
- Rock/pebble optimization
- Fog system fix
- Terrain texturing

See [`AGENT_DIRECTIVE.md`](AGENT_DIRECTIVE.md) for full details.

---

*Last updated: 2024-12-06*
