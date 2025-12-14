# Roanoke Engine Tools

Development tools for the Roanoke Engine.

---

## Blender Bridge (Primary Tool)

**Location**: `../blender-bridge/` (project root)
**Spec**: `docs/specs/BLENDER_BRIDGE_SPEC.md`

TCP socket-based remote control system for Blender. Allows CLI, game engine, and AI agents to control Blender programmatically.

### Components

| File | Purpose |
|------|---------|
| `blender_bridge_addon.py` | Blender addon (TCP server on port 9876) |
| `blender_client.py` | CLI client for sending commands |
| `quadruped_engine.py` | High-level quadruped rig control |

### Quick Start

```bash
# 1. Install addon in Blender
#    Edit > Preferences > Add-ons > Install > blender_bridge_addon.py

# 2. Start server in Blender
#    Sidebar (N) > Bridge tab > Start Server

# 3. Test connection
python blender-bridge/blender_client.py --ping

# 4. Interactive mode
python blender-bridge/blender_client.py -i
```

### WSL Users

```bash
# Get Windows host IP
HOST=$(ip route show | grep default | awk '{print $3}')

# Connect
python blender-bridge/blender_client.py --host $HOST --ping
```

See `docs/specs/BLENDER_BRIDGE_SPEC.md` for full command reference.

---

## Planned Tools

### SAM Analysis Server
Local Segment Anything Model server for asset optimization.
See: `docs/specs/SEGMENT_ANYTHING_MEMORY_SPEC.md`

### Asset Audit Script
CLI tool to validate all exported models.

### Hot Reload Server
WebSocket server for live asset reloading.
