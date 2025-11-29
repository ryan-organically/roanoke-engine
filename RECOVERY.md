# Recovery Guide

What you need to set up after cloning the repo.

## Already in Repo (No Action Needed)

- **Tree Pack** (`trees/`) - The trees9.obj model (~33MB) and textures are tracked in git
- **Shaders** (`assets/shaders/`) - All WGSL shaders are tracked
- **UI README** (`assets/ui/README.md`) - Instructions for UI assets

## Gitignored (Needs Local Setup)

### Build Artifacts
- `target/` - Run `cargo build` to regenerate

### Editor Settings
- `.vscode/` - VS Code settings (optional, configure as needed)
- `.idea/` - JetBrains IDE settings (optional)

### Save Files
- `saves/` - Player save data (generated at runtime)

## Optional UI Assets

These are optional and the game will use default rendering if missing:

- `assets/ui/background.png` - Home screen background
- `assets/ui/loading/loading.png` - Loading screen background

## First-Time Setup

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build --release

# Run the game
cargo run --release
```

## Notes

- The tree model pack is located at `trees/trees9.obj` (not in assets/)
- Code also checks `assets/trees/` as fallback path
