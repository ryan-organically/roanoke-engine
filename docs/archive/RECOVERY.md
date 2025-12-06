# Recovery Guide

**Last Updated**: 2024-12-06
**Status**: Archived (one-time use document)

What you need to set up after cloning the repo.

---

## Already in Repo (No Action Needed)

- **Shaders** (`assets/shaders/`) - All WGSL shaders are tracked
- **UI Assets** (`assets/ui/`) - UI images and placeholders
- **Documentation** (`docs/`) - All documentation

## Gitignored (Needs Local Setup)

### Build Artifacts
- `target/` - Run `cargo build --release` to regenerate

### Editor Settings
- `.vscode/` - VS Code settings (optional)
- `.idea/` - JetBrains IDE settings (optional)

### Save Files
- `saves/` - Player save data (generated at runtime)

---

## First-Time Setup

```bash
# 1. Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Build the project
cargo build --release

# 3. Run the game
cargo run --release
```

---

## Optional Assets

These are optional - the game uses procedural generation if missing:

| Asset | Purpose |
|-------|---------|
| `assets/ui/background.png` | Home screen background |
| `assets/ui/loading/loading.png` | Loading screen |
| `assets/textures/*.png` | Terrain textures (procedural fallback) |

---

## Notes

- **Trees**: Fully procedural (36 triangles/tree), no external models needed
- **Villages**: Procedurally generated Native American longhouses
- **Animals**: Rendered as colored orbs (no mesh assets needed)
- **Terrain**: Procedural with noise-based heightmaps

---

## Troubleshooting

### Build Fails
1. Check `docs/status/KNOWN_ISSUES.md` for build-blocking bugs
2. Run `cargo check` to see detailed errors
3. Ensure Rust is up to date: `rustup update`

### Low FPS
- FPS optimizations are deployed (Quantum Spatial Cache)
- See `docs/performance/FPS_OPTIMIZATION_ROADMAP.md`

---

*For development guidance, see `AGENT_DIRECTIVE.md` in project root.*
