# Claude Code Guidelines

## Coordination Rules

- **Never assume** you are the only Claude working on this project
- Before implementing a feature, ask if another terminal is already working on it
- If the user mentions another terminal is handling something, stop immediately and defer

## Investigation Before Action

- When the user asks to **analyze**, **investigate**, **locate**, or **find out why** something isn't working, **STOP and REPORT BACK** before making any changes
- Do not assume you know the fix. Present findings first, wait for approval
- Failed fixes compound problems. Humility > confidence when debugging
- If a fix didn't work, the next attempt requires **deeper investigation**, not another quick patch
- **Succeeding "quick fixes" without addressing root causes compounds problems** — each workaround adds technical debt and makes the real fix harder to identify

## Blender Asset Workflow

**When preparing any new asset for Blender export, read `docs/guides/BLENDER_ASSET_WORKFLOW.md` first.**

Before Blender exports:
1. Create export directory (`assets/models/<category>/`)
2. Write LOD spec (`docs/specs/<ASSET>_LOD_SPEC.json`)
3. Prepare pipeline hookup code
4. Have everything ready before export

## Build & Run

```bash
# Single-player
cargo run -p roanoke_game --release

# Release build
cargo build --release
```

## Multiplayer

Multiplayer is built into the game — no separate server binary needed.

**Dependencies:** Just Rust. For remote tunneling: `cargo install bore-cli`

**Host:** In-game pause menu → Multiplayer → Host → "Open to LAN"
**Join:** In-game pause menu → Multiplayer → Join → enter host's address
**CLI:** `cargo run -p roanoke_game -- --host 7878` or `--join 192.168.1.X:7878 --name YourName`
**Remote:** After hosting, click "Share via Tunnel (bore)" to get a public URL

Full guide: `docs/guides/MULTIPLAYER.md`

## Active Work Areas

Track what's being worked on to avoid conflicts:

(none currently)
