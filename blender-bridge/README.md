# Blender Bridge

TCP socket bridge for controlling Blender from the terminal. Built so Claude can drive Blender programmatically.

## Architecture

```
Terminal (client) --> TCP socket (localhost:9876) --> Blender addon (server) --> bpy --> JSON response
```

## Installation

### 1. Install the Addon

Copy `blender_bridge_addon.py` to your Blender addons folder:

- **Windows**: `%APPDATA%\Blender Foundation\Blender\4.x\scripts\addons\`
- **Linux**: `~/.config/blender/4.x/scripts/addons/`
- **Mac**: `/Users/<user>/Library/Application Support/Blender/4.x/scripts/addons/`

Or install via Blender:
1. Edit > Preferences > Add-ons
2. Install... > Select `blender_bridge_addon.py`
3. Enable "Blender Bridge" in the addon list

### 2. Start the Server

In Blender:
1. Open the sidebar (N key) in the 3D Viewport
2. Go to the "Bridge" tab
3. Click "Start Server"

Or via Python console in Blender:
```python
import blender_bridge_addon
blender_bridge_addon.start_server()
```

## Usage

### CLI Arguments

```bash
# Test connection
python blender_client.py --ping

# Execute arbitrary Python
python blender_client.py --exec "bpy.ops.mesh.primitive_cube_add()"

# List objects
python blender_client.py --list-objects
python blender_client.py --list-objects MESH    # Filter by type

# List materials
python blender_client.py --list-materials

# Object operations
python blender_client.py --select Cube
python blender_client.py --delete Cube
python blender_client.py --add-primitive sphere 1 2 3
python blender_client.py --duplicate Cube
python blender_client.py --rename Cube MyCube

# Transform
python blender_client.py --set-location Cube 1 2 3
python blender_client.py --set-rotation Cube 45 0 90    # Degrees
python blender_client.py --set-scale Cube 2 2 2

# Export
python blender_client.py --export-fbx /path/to/file.fbx
python blender_client.py --export-gltf /path/to/file.glb

# Scene info
python blender_client.py --scene-info
python blender_client.py --get-selected
python blender_client.py --screenshot /path/to/render.png

# Raw JSON output
python blender_client.py --list-objects --raw
```

### Interactive Mode

```bash
python blender_client.py -i
```

```
blender> help
blender> list
blender> add cube 0 0 2
blender> select Cube
blender> move Cube 1 1 1
blender> exec bpy.context.object.scale = (2, 2, 2)
blender> quit
```

### Raw JSON Commands

```bash
python blender_client.py --json '{"command": "add_primitive", "args": {"type": "monkey", "location": [0, 0, 0]}}'
```

## Available Commands

| Command | Description |
|---------|-------------|
| `ping` | Test connection, returns Blender version |
| `exec` | Execute arbitrary Python code |
| `list_objects` | List all scene objects |
| `list_materials` | List all materials |
| `select` | Select object by name |
| `delete` | Delete object by name |
| `add_primitive` | Add mesh primitive (cube, sphere, plane, cylinder, cone, torus, monkey, ico_sphere, circle, grid) |
| `get_selected` | Get info about selected objects |
| `get_scene_info` | Get scene metadata |
| `set_location` | Set object position |
| `set_rotation` | Set object rotation (degrees) |
| `set_scale` | Set object scale |
| `duplicate` | Duplicate object |
| `rename` | Rename object |
| `export_fbx` | Export selection to FBX |
| `export_gltf` | Export selection to glTF/GLB |
| `screenshot` | Save viewport render |

## JSON Protocol

### Request Format

```json
{
  "command": "command_name",
  "args": {
    "key": "value"
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

## Security

- Server binds to `127.0.0.1` only (no network exposure)
- Only accepts connections from localhost
- No authentication (localhost trust model)

## Troubleshooting

**Connection refused**: Make sure Blender is running and the Bridge server is started (check the Bridge panel in the sidebar).

**Blender freezes**: The addon uses `bpy.app.timers` for non-blocking operation. If you experience issues, restart Blender.

**Port already in use**: Another instance may be running. Check `netstat -an | grep 9876` or change the port in both addon and client.
