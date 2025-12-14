# Blender Bridge System Specification

**Status**: v1.1.0 Implemented
**Location**: `blender-bridge/`
**Blender Version**: 4.0+

---

## Overview

The Blender Bridge is a **TCP socket-based remote control system** for Blender. It allows external tools (CLI, game engine, AI agents) to control Blender programmatically via JSON commands.

```
┌─────────────────┐         TCP:9876         ┌─────────────────┐
│                 │◄────────────────────────►│                 │
│  External Tool  │      JSON Commands       │    Blender      │
│  (CLI/Engine)   │◄────────────────────────►│    (Addon)      │
│                 │      JSON Responses      │                 │
└─────────────────┘                          └─────────────────┘
```

---

## Components

### 1. `blender_bridge_addon.py` — Blender Server

TCP socket server that runs inside Blender, accepting JSON commands.

**Installation**:
1. Open Blender
2. Edit > Preferences > Add-ons > Install
3. Navigate to `blender-bridge/blender_bridge_addon.py`
4. Enable "Blender Bridge"
5. In 3D View sidebar (N) > Bridge tab > Click "Start Server"

**Server Config**:
```python
HOST = '0.0.0.0'  # Accepts connections from WSL
PORT = 9876
BUFFER_SIZE = 65536
```

### 2. `blender_client.py` — CLI Client

Command-line tool for sending commands to Blender.

**Usage**:
```bash
# Test connection
python blender_client.py --ping

# List objects
python blender_client.py --list-objects

# Execute Python code
python blender_client.py --exec "bpy.ops.mesh.primitive_cube_add()"

# Interactive mode
python blender_client.py -i
```

**WSL Connection** (if running from WSL):
```bash
# Get Windows host IP
ip route show | grep default | awk '{print $3}'

# Connect with host IP
python blender_client.py --host 172.19.16.1 --ping
```

### 3. `quadruped_engine.py` — High-Level Quadruped Control

Specialized API for controlling four-legged creature rigs.

**Features**:
- Auto-identify rig joints from naming conventions
- IK limb control (FL/FR/BL/BR)
- Procedural gaits (walk, trot)
- Pose creation (graze)

**Usage**:
```bash
# Identify rig structure
python quadruped_engine.py --armature AnimalArmature --identify

# Create walk cycle
python quadruped_engine.py --armature AnimalArmature --walk

# Create graze pose
python quadruped_engine.py --armature AnimalArmature --graze 75

# Move limb IK target
python quadruped_engine.py --armature AnimalArmature --move-limb FL 0 0.5 0.1
```

---

## Available Commands

### Core Commands

| Command | Description | Args |
|---------|-------------|------|
| `ping` | Test connection | — |
| `exec` | Execute Python code | `code: str` |
| `list_objects` | List scene objects | `type?: str` |
| `list_materials` | List all materials | — |
| `get_scene_info` | Scene metadata | — |
| `get_selected` | Selected objects info | — |

### Object Manipulation

| Command | Description | Args |
|---------|-------------|------|
| `select` | Select object | `name: str` |
| `delete` | Delete object | `name: str` |
| `add_primitive` | Add mesh primitive | `type: str, location?: [x,y,z]` |
| `duplicate` | Duplicate object | `name: str` |
| `rename` | Rename object | `name: str, new_name: str` |
| `set_location` | Set position | `name: str, location: [x,y,z]` |
| `set_rotation` | Set rotation (degrees) | `name: str, rotation: [x,y,z]` |
| `set_scale` | Set scale | `name: str, scale: [x,y,z]` |
| `apply_transforms` | Apply transforms | `name: str, location?, rotation?, scale?` |

### Properties & Animation

| Command | Description | Args |
|---------|-------------|------|
| `get_custom_props` | Get custom properties | `name: str` |
| `set_custom_prop` | Set custom property | `name: str, property: str, value: any` |
| `delete_custom_prop` | Delete property | `name: str, property: str` |
| `list_actions` | List all animations | — |
| `set_active_action` | Set active animation | `name: str, action: str` |

### Armature

| Command | Description | Args |
|---------|-------------|------|
| `get_armature_info` | Bone hierarchy & info | `name: str` |

### Export

| Command | Description | Args |
|---------|-------------|------|
| `export_fbx` | Export to FBX | `filepath: str, use_selection?: bool` |
| `export_gltf` | Export to glTF/GLB | `filepath: str, use_selection?: bool` |
| `screenshot` | Viewport render | `filepath: str` |

---

## JSON Protocol

### Request Format
```json
{
  "command": "command_name",
  "args": {
    "arg1": "value1",
    "arg2": "value2"
  }
}
```

### Response Format
```json
{
  "success": true,
  "result": "...",
  "objects": [...],
  "error": "...",
  "traceback": "..."
}
```

### Example: Execute Code
```json
// Request
{"command": "exec", "args": {"code": "[a.name for a in bpy.data.actions]"}}

// Response
{"success": true, "result": ["Idle", "Walk", "Gallop"]}
```

---

## Quadruped Engine API

### Rig Identification

The engine auto-identifies bones by naming patterns:

```
Front Left:  Front*.L, FL_*, Shoulder.L
Front Right: Front*.R, FR_*, Shoulder.R
Back Left:   Back*.L, BL_*, Hip.L, Rear*.L
Back Right:  Back*.R, BR_*, Hip.R, Rear*.R
```

### Expected Bone Structure

```
Root/Body
├── Spine chain (Body → Back → Torso)
├── Neck chain (Neck, Neck1, ...)
├── Head
├── Tail chain (Tail, Tail1, ...)
├── Front Left leg chain
│   ├── FrontShoulder.L
│   ├── FrontUpperLeg.L
│   ├── FrontLowerLeg.L
│   └── FrontFoot.L
│   └── IKFrontLeg.L (IK target)
├── Front Right leg chain (same pattern)
├── Back Left leg chain
└── Back Right leg chain
```

### Procedural Gaits

**Walk Cycle** (diagonal gait):
- FL + BR move together
- FR + BL move together
- Sinusoidal forward/back motion
- Vertical lift on forward phase

**Trot Cycle** (faster diagonal gait):
- Same diagonal pairing
- Higher leg lift
- Body suspension (vertical bounce)

---

## SAM Integration Opportunity

The bridge's `screenshot` and `exec` commands enable SAM integration:

```python
# 1. Capture viewport
client.screenshot("/tmp/viewport.png")

# 2. Run SAM analysis (external)
segments = sam_analyze("/tmp/viewport.png")

# 3. Map segments to objects via exec
code = """
import bpy
from bpy_extras.object_utils import world_to_camera_view
# Map screen coordinates to 3D objects
...
"""
client.exec(code)

# 4. Adjust object properties based on importance
for obj_name, importance in importance_map.items():
    client.set_custom_prop(obj_name, "sam_importance", importance)
```

### Potential SAM Commands (Future)

| Command | Description |
|---------|-------------|
| `analyze_viewport` | Capture + run SAM + return segments |
| `set_lod_by_importance` | Auto-adjust LOD based on SAM |
| `highlight_segments` | Visualize SAM segments in viewport |

---

## Usage Patterns

### From Claude Code (WSL)

```bash
# Get host IP
HOST=$(ip route show | grep default | awk '{print $3}')

# Ping
python3 blender-bridge/blender_client.py --host $HOST --ping

# List animations
python3 blender-bridge/blender_client.py --host $HOST \
  --exec "[a.name for a in bpy.data.actions]"

# Export model
python3 blender-bridge/blender_client.py --host $HOST \
  --export-gltf "C:/dev/roanoke engine/assets/models/animals/Horse.gltf"
```

### From Game Engine (Rust)

```rust
// Future: Direct TCP connection from engine
use std::net::TcpStream;
use serde_json::json;

fn send_blender_command(cmd: &str, args: serde_json::Value) -> Result<Value> {
    let mut stream = TcpStream::connect("127.0.0.1:9876")?;
    let payload = json!({"command": cmd, "args": args});
    stream.write_all(payload.to_string().as_bytes())?;
    // Read response...
}

// Hot-reload model during development
send_blender_command("export_gltf", json!({
    "filepath": "assets/models/animals/Horse.gltf",
    "use_selection": true
}))?;
```

### Interactive Session

```
$ python blender_client.py -i
Blender Bridge Interactive Mode
Connected to 127.0.0.1:9876
Type 'help' for commands, 'quit' to exit

blender> ping
pong (Blender 4.0.0)

blender> list mesh
Objects (3):
  Horse                [MESH      ] @ (0.00, 0.00, 0.00)
  Ground               [MESH      ] @ (0.00, 0.00, -0.10)
  Cube                 [MESH      ] @ (5.00, 0.00, 1.00)

blender> select Horse
Selected: Horse

blender> exec bpy.context.active_object.data.polygons.__len__()
12450

blender> quit
Bye!
```

---

## Proliferation Strategy

### 1. Documentation (This File)
- [x] Command reference
- [x] Protocol specification
- [x] Usage examples
- [x] WSL connection guide

### 2. Engine Integration
- [ ] Rust TCP client in `croatoan_core`
- [ ] Hot-reload system for development
- [ ] Asset validation on import

### 3. AI Agent Integration
- [ ] Claude Code workflow documentation
- [ ] SAM analysis pipeline
- [ ] Automated asset optimization

### 4. Discoverability
- [ ] Add to main README
- [ ] Reference in State of Union
- [ ] Link from HORSE_BLENDER_BRIDGE.md

---

## Files

```
blender-bridge/
├── blender_bridge_addon.py   # Blender TCP server addon (v1.1.0)
├── blender_client.py         # CLI client
├── quadruped_engine.py       # High-level quadruped control
└── CLAUDE.md                 # AI agent guidelines
```

---

## Changelog

### v1.1.0 (Current)
- Custom properties (get/set/delete)
- Animation listing and switching
- Armature info with bone hierarchy
- Transform application

### v1.0.0
- Core TCP server
- Object manipulation
- Export (FBX, glTF)
- Screenshot capture

---

*"The bridge between artist and engine should be invisible — but when you need it, it should be powerful."*
