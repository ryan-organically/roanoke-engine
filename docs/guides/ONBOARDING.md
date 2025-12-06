# Onboarding Guide

**Last Updated**: 2024-12-06

For new developers and agents joining the project.

---

## For Agents - Start Here

**Primary Entry Point**: `../../AGENT_DIRECTIVE.md` (in project root)

This is the unified document with:
- Current state and priorities
- Document map (what to read when)
- Session notes
- Controls reference

---

## Quick Start

```bash
# 1. Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Build
cargo build --release

# 3. Run
cargo run --release
```

---

## Project Structure

```
roanoke-engine/
├── AGENT_DIRECTIVE.md      # ← AGENT: Start here!
├── README.md               # Project overview
├── Cargo.toml              # Workspace root
│
├── docs/                   # All documentation
│   ├── status/             # Project status
│   │   ├── MASTER_AUDIT.md
│   │   ├── ROADMAP.md
│   │   ├── VERSION.md
│   │   └── KNOWN_ISSUES.md
│   ├── guides/             # How-to docs (you are here)
│   ├── specs/              # System specifications
│   ├── performance/        # Optimization docs
│   ├── vision/             # Long-term planning
│   └── archive/            # Archived docs
│
├── roanoke_game/           # Main game binary
│   └── src/
│       ├── main.rs         # Entry point
│       ├── animals/        # Animal system
│       ├── flora/          # Plant system
│       ├── npc/            # NPC system
│       └── economy/        # Economy system
│
├── crates/                 # Engine libraries
│   ├── croatoan_core/      # Camera, player
│   ├── croatoan_render/    # GPU pipelines
│   ├── croatoan_procgen/   # Procedural generation
│   └── croatoan_wfc/       # Wave Function Collapse
│
├── assets/                 # Game assets
│   ├── shaders/            # WGSL shaders
│   └── ui/                 # UI images
│
└── marketing/              # Marketing docs
```

---

## What's In The Repo

<!-- AGENT: Update if assets change -->

| Asset | Location | Size | Notes |
|-------|----------|------|-------|
| Tree models | `trees/trees9.obj` | ~33MB | Tracked in git |
| Tree textures | `trees/Texture/` | ~100KB | 8 JPG files |
| Shaders | `assets/shaders/` | ~50KB | 10 WGSL files |
| UI placeholder | `assets/ui/` | ~2.5MB | PNG images |

---

## What's Gitignored

<!-- AGENT: Don't commit these -->

| Path | Purpose | Regenerate |
|------|---------|------------|
| `target/` | Build artifacts | `cargo build` |
| `saves/` | Player saves | Runtime generated |
| `.vscode/` | Editor config | Manual setup |
| `.idea/` | JetBrains config | Manual setup |

---

## Optional Assets

The game works without these (uses defaults):

| File | Purpose |
|------|---------|
| `assets/ui/background.png` | Home screen background |
| `assets/ui/loading/loading.png` | Loading screen |

---

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Mouse | Look |
| Space | Jump |
| T | Advance time |
| Y | Reverse time |
| Esc | Menu |

---

## For Agents

1. **Read `../../AGENT_DIRECTIVE.md` first** - Unified entry point
2. **Check `../status/KNOWN_ISSUES.md`** - Build-blocking bugs
3. **Check `../status/MASTER_AUDIT.md`** - Technical status details
4. **Update docs as you work** - Keep them in sync with code
5. **Add session notes** - To `AGENT_DIRECTIVE.md`

### Key Files to Know

| When working on... | Read these files |
|--------------------|------------------|
| UI/Menu | `main.rs`, `MARCHING_ORDERS.md` |
| Animals | `animals/`, `../specs/ANIMAL_SYSTEM_SPEC.md` |
| Plants/Flora | `flora/`, `../specs/FLORA_PLANT_SYSTEM_SPEC.md` |
| NPCs/Villages | `npc/`, `../specs/NPC_VILLAGE_SPECIFICATION.md` |
| Economy | `economy/`, `../specs/ECONOMY_IMPLEMENTATION.md` |
| Weather | `weather_system.rs`, `sky.wgsl` |
| Terrain | `croatoan_wfc/`, `terrain.wgsl` |
| Rendering | `croatoan_render/src/` |
| Performance | `../performance/FPS_OPTIMIZATION_ROADMAP.md` |

### Document Hierarchy

```
/AGENT_DIRECTIVE.md              <- Start here (root)
└── docs/
    ├── status/
    │   ├── MASTER_AUDIT.md      <- Technical details
    │   ├── ROADMAP.md           <- Implementation phases
    │   ├── KNOWN_ISSUES.md      <- Bugs to fix
    │   └── VERSION.md           <- Release history
    ├── specs/                   <- System specifications
    ├── performance/             <- Optimization docs
    └── vision/                  <- Long-term planning
```

---

*Last updated: 2024-12-06*
