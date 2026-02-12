# Multiplayer Guide

**Last Updated**: 2026-02-11

Co-op dev testing over LAN or tunneled connections. Both players see the same procedural world (server syncs seed). Remote players appear as colored orbs with nametags.

---

## Prerequisites

- **Rust toolchain** (rustup + cargo) — all dependencies resolve via `cargo build`.

---

## Quick Start

### 1. Host starts the game

```bash
cargo run -p roanoke_game --release
```

### 2. Host opens to LAN

- Press **Esc** to pause
- Click **Multiplayer**
- Set port (default: 7878)
- Click **Open to LAN**

The embedded server starts automatically — no separate server binary needed.

### 3. Friend joins

Same game binary, then:

- Press **Esc** → **Multiplayer**
- Enter host's address: `192.168.1.X:7878`
- Enter a player name
- Click **Join Game**

The joiner's world regenerates with the host's seed so terrain matches.

---

## Remote Access (Not on Same Network)

### Option A: Bore tunnel (built into game UI)

After hosting, click **"Share via Tunnel (bore)"** in the Multiplayer menu. The game spawns bore automatically and displays a public URL your friend can use.

Requires bore CLI installed:
```bash
cargo install bore-cli
```

### Option B: Manual bore

```bash
bore local 7878 --to bore.pub
```

Friend connects with the bore.pub address shown.

### Option C: ngrok

```bash
ngrok tcp 7878
```

### Option D: CLI launch

```bash
# Host
cargo run -p roanoke_game -- --host 7878

# Join
cargo run -p roanoke_game -- --join 192.168.1.X:7878 --name Steve
```

---

## Architecture

The multiplayer system is embedded in the game binary — no separate server crate needed.

### Network Module (`roanoke_game/src/network/`)

| File | Purpose |
|------|---------|
| `mod.rs` | Module root, re-exports |
| `messages.rs` | Protocol types (bincode serialized) |
| `server.rs` | Embedded WebSocket server (tokio) |
| `client.rs` | WebSocket client |
| `manager.rs` | Unified NetworkManager (Host/Join/Offline modes) |
| `cli.rs` | CLI argument parsing (`--host`, `--join`) |
| `remote_renderer.rs` | Remote player orbs + nametags |

### How It Works

1. **Host** starts an embedded WebSocket server on the chosen port
2. **Joiners** connect via WebSocket, receive the host's seed in `JoinAccepted`
3. All players send position/rotation updates at 20Hz
4. Server broadcasts `PlayerSync` to all other clients
5. Remote players are lerp-interpolated for smooth movement

### Protocol (bincode over WebSocket binary frames)

| Message | Direction | Purpose |
|---------|-----------|---------|
| `JoinRequest` | Client -> Server | Name + protocol version |
| `JoinAccepted` | Server -> Client | Player ID + seed + player list |
| `PlayerSync` | Both | Position, velocity, rotation |
| `PlayerJoined` | Server -> All | New player notification |
| `PlayerLeft` | Server -> All | Disconnect notification |
| `Chat` | Both | Chat messages |
| `Ping/Pong` | Both | Latency measurement |

---

## In-Game UI

### Pause Menu -> Multiplayer

**When hosting:**
- Player list with positions
- "Share via Tunnel (bore)" button — spawns bore, shows public URL with copy button
- "Stop Tunnel" / "Disconnect" buttons

**When not connected:**
- **Host section**: Port input + "Open to LAN" button
- **Join section**: Address + name inputs + "Join Game" button

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| "Failed to host" | Port already in use. Try a different port. |
| "Failed to join" | Check address and port. Is the host running? |
| Terrain doesn't match | Both must be on the same game version (same commit). |
| "bore not found" | Install: `cargo install bore-cli` |
| Firewall blocking | Allow TCP on the host port (default 7878). |

---

## Limitations

- **Position sync only** — no shared game state (animals, weather, time are independent)
- **No authoritative server** — pure relay, no validation
- **No persistence** — server state is in-memory
- **No player models** — remote players are colored orbs
- **Seed changes on join** — joiner's world regenerates to match host

---

## Future

1. Player character models instead of orbs
2. Action sync (combat, building, items)
3. Time/weather sync from host
4. Chat UI overlay
5. Authoritative server with game state validation
