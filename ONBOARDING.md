# Onboarding Guide

For new developers and agents joining the project.

<!-- AGENT: Read AGENT_SCOPE.md for current state and priorities -->

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
├── AGENT_SCOPE.md      # ← AGENT: Read this first!
├── VERSION.md          # Version history & roadmap
├── ONBOARDING.md       # This file
├── Cargo.toml          # Workspace root
├── roanoke_game/       # Main game binary
│   └── src/
│       ├── main.rs           # Entry point, game loop, UI
│       ├── weather_system.rs # Weather state machine
│       └── water_system.rs   # Water simulation (WIP)
├── crates/
│   ├── croatoan_core/    # Camera, player, utilities
│   ├── croatoan_render/  # GPU pipelines
│   ├── croatoan_procgen/ # Procedural generation
│   ├── croatoan_wfc/     # Wave Function Collapse
│   └── croatoan_neural/  # Neural experiments
├── assets/
│   ├── shaders/          # WGSL shaders
│   ├── ui/               # UI images
│   └── trees/            # (fallback path)
└── trees/                # Tree model pack (tracked)
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

1. **Read `AGENT_SCOPE.md` first** - Has current state, issues, priorities
2. **Check `VERSION.md`** - For version history and roadmap
3. **Update docs as you work** - Keep them in sync with code
4. **Session notes** - Add dated notes to AGENT_SCOPE.md

### Key Files to Know

| When working on... | Read these files |
|--------------------|------------------|
| UI/Menu | `main.rs:797-1050` |
| Weather | `weather_system.rs`, `sky.wgsl` |
| Terrain | `croatoan_wfc/`, `terrain.wgsl` |
| Rendering | `croatoan_render/src/` |

---

*Last updated: 2024-11-28*
