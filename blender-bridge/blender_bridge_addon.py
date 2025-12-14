"""
Blender Bridge Addon
TCP socket server that allows external control of Blender via JSON commands.
Binds to 127.0.0.1:9876 - localhost only for security.
"""

bl_info = {
    "name": "Blender Bridge",
    "author": "Claude + Ryan",
    "version": (1, 1, 0),
    "blender": (4, 0, 0),
    "location": "View3D > Sidebar > Bridge",
    "description": "TCP socket server for external Blender control",
    "category": "Development",
}

import bpy
import json
import socket
import traceback
import os
from mathutils import Vector, Euler, Matrix

# Server configuration
# Bind to 0.0.0.0 to accept connections from WSL (still safe - not exposed to network)
HOST = '0.0.0.0'
PORT = 9876
BUFFER_SIZE = 65536

# Global state
server_socket = None
is_running = False


def get_object_info(obj):
    """Extract serializable info from a Blender object."""
    info = {
        "name": obj.name,
        "type": obj.type,
        "location": list(obj.location),
        "rotation": list(obj.rotation_euler),
        "scale": list(obj.scale),
        "visible": obj.visible_get(),
    }
    if obj.type == 'MESH':
        info["vertices"] = len(obj.data.vertices)
        info["edges"] = len(obj.data.edges)
        info["faces"] = len(obj.data.polygons)
    if obj.active_material:
        info["material"] = obj.active_material.name
    return info


def get_material_info(mat):
    """Extract serializable info from a material."""
    info = {
        "name": mat.name,
        "use_nodes": mat.use_nodes,
    }
    if mat.use_nodes and mat.node_tree:
        # Find principled BSDF if present
        for node in mat.node_tree.nodes:
            if node.type == 'BSDF_PRINCIPLED':
                base_color = node.inputs.get('Base Color')
                if base_color:
                    info["base_color"] = list(base_color.default_value)
                metallic = node.inputs.get('Metallic')
                if metallic:
                    info["metallic"] = metallic.default_value
                roughness = node.inputs.get('Roughness')
                if roughness:
                    info["roughness"] = roughness.default_value
                break
    return info


def handle_command(cmd_data):
    """Process a command and return the result."""
    try:
        command = cmd_data.get("command", "")
        args = cmd_data.get("args", {})

        # Route to appropriate handler
        if command == "exec":
            return handle_exec(args)
        elif command == "list_objects":
            return handle_list_objects(args)
        elif command == "list_materials":
            return handle_list_materials(args)
        elif command == "select":
            return handle_select(args)
        elif command == "delete":
            return handle_delete(args)
        elif command == "add_primitive":
            return handle_add_primitive(args)
        elif command == "export_fbx":
            return handle_export_fbx(args)
        elif command == "export_gltf":
            return handle_export_gltf(args)
        elif command == "get_selected":
            return handle_get_selected(args)
        elif command == "screenshot":
            return handle_screenshot(args)
        elif command == "get_scene_info":
            return handle_get_scene_info(args)
        elif command == "set_location":
            return handle_set_location(args)
        elif command == "set_rotation":
            return handle_set_rotation(args)
        elif command == "set_scale":
            return handle_set_scale(args)
        elif command == "duplicate":
            return handle_duplicate(args)
        elif command == "rename":
            return handle_rename(args)
        # v1.1.0: Custom properties
        elif command == "get_custom_props":
            return handle_get_custom_props(args)
        elif command == "set_custom_prop":
            return handle_set_custom_prop(args)
        elif command == "delete_custom_prop":
            return handle_delete_custom_prop(args)
        # v1.1.0: Animation
        elif command == "list_actions":
            return handle_list_actions(args)
        elif command == "set_active_action":
            return handle_set_active_action(args)
        # v1.1.0: Armature
        elif command == "get_armature_info":
            return handle_get_armature_info(args)
        # v1.1.0: Transform utilities
        elif command == "apply_transforms":
            return handle_apply_transforms(args)
        elif command == "ping":
            return {"success": True, "result": "pong", "blender_version": bpy.app.version_string}
        else:
            return {"success": False, "error": f"Unknown command: {command}"}

    except Exception as e:
        return {
            "success": False,
            "error": str(e),
            "traceback": traceback.format_exc()
        }


def handle_exec(args):
    """Execute arbitrary Python code in Blender."""
    code = args.get("code", "")
    if not code:
        return {"success": False, "error": "No code provided"}

    # Create a namespace for execution
    import math
    exec_globals = {"bpy": bpy, "Vector": Vector, "Euler": Euler, "Matrix": Matrix, "math": math}
    exec_locals = {}

    try:
        # Try to evaluate as expression first (for return values)
        result = eval(code, exec_globals, exec_locals)
        # Try to make result JSON serializable
        try:
            json.dumps(result)
            return {"success": True, "result": result}
        except (TypeError, ValueError):
            return {"success": True, "result": str(result)}
    except SyntaxError:
        # Not an expression, execute as statement
        exec(code, exec_globals, exec_locals)
        return {"success": True, "result": None}


def handle_list_objects(args):
    """List all objects in the scene."""
    filter_type = args.get("type", None)
    objects = []
    for obj in bpy.context.scene.objects:
        if filter_type is None or obj.type == filter_type.upper():
            objects.append(get_object_info(obj))
    return {"success": True, "objects": objects, "count": len(objects)}


def handle_list_materials(args):
    """List all materials in the file."""
    materials = []
    for mat in bpy.data.materials:
        materials.append(get_material_info(mat))
    return {"success": True, "materials": materials, "count": len(materials)}


def handle_select(args):
    """Select an object by name."""
    name = args.get("name", "")
    if not name:
        return {"success": False, "error": "No object name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    # Deselect all first if requested
    if args.get("deselect_others", True):
        bpy.ops.object.select_all(action='DESELECT')

    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    return {"success": True, "selected": name}


def handle_delete(args):
    """Delete an object by name."""
    name = args.get("name", "")
    if not name:
        return {"success": False, "error": "No object name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    bpy.data.objects.remove(obj, do_unlink=True)
    return {"success": True, "deleted": name}


def handle_add_primitive(args):
    """Add a primitive mesh object."""
    prim_type = args.get("type", "cube").lower()
    location = args.get("location", [0, 0, 0])

    # Ensure location is a tuple/list of 3 floats
    if len(location) != 3:
        return {"success": False, "error": "Location must be [x, y, z]"}

    loc = tuple(float(v) for v in location)

    primitives = {
        "cube": lambda: bpy.ops.mesh.primitive_cube_add(location=loc),
        "sphere": lambda: bpy.ops.mesh.primitive_uv_sphere_add(location=loc),
        "ico_sphere": lambda: bpy.ops.mesh.primitive_ico_sphere_add(location=loc),
        "plane": lambda: bpy.ops.mesh.primitive_plane_add(location=loc),
        "cylinder": lambda: bpy.ops.mesh.primitive_cylinder_add(location=loc),
        "cone": lambda: bpy.ops.mesh.primitive_cone_add(location=loc),
        "torus": lambda: bpy.ops.mesh.primitive_torus_add(location=loc),
        "monkey": lambda: bpy.ops.mesh.primitive_monkey_add(location=loc),
        "circle": lambda: bpy.ops.mesh.primitive_circle_add(location=loc),
        "grid": lambda: bpy.ops.mesh.primitive_grid_add(location=loc),
    }

    if prim_type not in primitives:
        return {"success": False, "error": f"Unknown primitive type: {prim_type}. Available: {list(primitives.keys())}"}

    primitives[prim_type]()
    new_obj = bpy.context.active_object

    return {"success": True, "created": new_obj.name, "object": get_object_info(new_obj)}


def handle_export_fbx(args):
    """Export selected objects to FBX."""
    filepath = args.get("filepath", "")
    if not filepath:
        return {"success": False, "error": "No filepath provided"}

    # Ensure .fbx extension
    if not filepath.lower().endswith('.fbx'):
        filepath += '.fbx'

    # Make path absolute if relative
    if not os.path.isabs(filepath):
        filepath = os.path.join(os.path.dirname(bpy.data.filepath) or os.getcwd(), filepath)

    # Ensure directory exists
    os.makedirs(os.path.dirname(filepath) or '.', exist_ok=True)

    use_selection = args.get("use_selection", True)

    bpy.ops.export_scene.fbx(
        filepath=filepath,
        use_selection=use_selection,
        apply_scale_options='FBX_SCALE_ALL'
    )

    return {"success": True, "exported": filepath}


def handle_export_gltf(args):
    """Export selected objects to glTF."""
    filepath = args.get("filepath", "")
    if not filepath:
        return {"success": False, "error": "No filepath provided"}

    # Make path absolute if relative
    if not os.path.isabs(filepath):
        filepath = os.path.join(os.path.dirname(bpy.data.filepath) or os.getcwd(), filepath)

    # Ensure directory exists
    os.makedirs(os.path.dirname(filepath) or '.', exist_ok=True)

    use_selection = args.get("use_selection", True)
    export_format = 'GLB' if filepath.lower().endswith('.glb') else 'GLTF_SEPARATE'

    bpy.ops.export_scene.gltf(
        filepath=filepath,
        use_selection=use_selection,
        export_format=export_format
    )

    return {"success": True, "exported": filepath}


def handle_get_selected(args):
    """Get info about currently selected objects."""
    selected = [get_object_info(obj) for obj in bpy.context.selected_objects]
    active = None
    if bpy.context.active_object:
        active = get_object_info(bpy.context.active_object)
    return {"success": True, "selected": selected, "active": active, "count": len(selected)}


def handle_screenshot(args):
    """Save viewport render to file."""
    filepath = args.get("filepath", "")
    if not filepath:
        return {"success": False, "error": "No filepath provided"}

    # Make path absolute if relative
    if not os.path.isabs(filepath):
        filepath = os.path.join(os.path.dirname(bpy.data.filepath) or os.getcwd(), filepath)

    # Ensure directory exists
    os.makedirs(os.path.dirname(filepath) or '.', exist_ok=True)

    # Use OpenGL render for viewport screenshot
    bpy.context.scene.render.filepath = filepath
    bpy.ops.render.opengl(write_still=True)

    return {"success": True, "saved": filepath}


def handle_get_scene_info(args):
    """Get general scene information."""
    scene = bpy.context.scene
    return {
        "success": True,
        "scene": {
            "name": scene.name,
            "frame_current": scene.frame_current,
            "frame_start": scene.frame_start,
            "frame_end": scene.frame_end,
            "fps": scene.render.fps,
            "resolution": [scene.render.resolution_x, scene.render.resolution_y],
            "object_count": len(scene.objects),
            "filepath": bpy.data.filepath or "(unsaved)"
        }
    }


def handle_set_location(args):
    """Set an object's location."""
    name = args.get("name", "")
    location = args.get("location", [])

    if not name:
        return {"success": False, "error": "No object name provided"}
    if len(location) != 3:
        return {"success": False, "error": "Location must be [x, y, z]"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    obj.location = Vector(location)
    return {"success": True, "object": get_object_info(obj)}


def handle_set_rotation(args):
    """Set an object's rotation (Euler angles in degrees)."""
    name = args.get("name", "")
    rotation = args.get("rotation", [])

    if not name:
        return {"success": False, "error": "No object name provided"}
    if len(rotation) != 3:
        return {"success": False, "error": "Rotation must be [x, y, z] in degrees"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    import math
    obj.rotation_euler = Euler([math.radians(r) for r in rotation])
    return {"success": True, "object": get_object_info(obj)}


def handle_set_scale(args):
    """Set an object's scale."""
    name = args.get("name", "")
    scale = args.get("scale", [])

    if not name:
        return {"success": False, "error": "No object name provided"}
    if len(scale) != 3:
        return {"success": False, "error": "Scale must be [x, y, z]"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    obj.scale = Vector(scale)
    return {"success": True, "object": get_object_info(obj)}


def handle_duplicate(args):
    """Duplicate an object."""
    name = args.get("name", "")
    if not name:
        return {"success": False, "error": "No object name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    new_obj = obj.copy()
    if obj.data:
        new_obj.data = obj.data.copy()
    bpy.context.collection.objects.link(new_obj)

    return {"success": True, "original": name, "duplicate": new_obj.name, "object": get_object_info(new_obj)}


def handle_rename(args):
    """Rename an object."""
    name = args.get("name", "")
    new_name = args.get("new_name", "")

    if not name:
        return {"success": False, "error": "No object name provided"}
    if not new_name:
        return {"success": False, "error": "No new name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    obj.name = new_name
    return {"success": True, "old_name": name, "new_name": obj.name}


# === v1.1.0 additions: Custom Properties, Animation, Armature ===

def handle_get_custom_props(args):
    """Get custom properties from an object."""
    name = args.get("name", "")
    if not name:
        return {"success": False, "error": "No object name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    # Get custom properties (exclude internal ones starting with _)
    props = {}
    for key in obj.keys():
        if not key.startswith("_"):
            val = obj[key]
            # Convert to JSON-safe types
            if hasattr(val, "to_list"):
                props[key] = val.to_list()
            elif hasattr(val, "to_dict"):
                props[key] = val.to_dict()
            else:
                try:
                    json.dumps(val)
                    props[key] = val
                except (TypeError, ValueError):
                    props[key] = str(val)

    return {"success": True, "object": name, "properties": props}


def handle_set_custom_prop(args):
    """Set a custom property on an object."""
    name = args.get("name", "")
    prop_name = args.get("property", "")
    value = args.get("value")

    if not name:
        return {"success": False, "error": "No object name provided"}
    if not prop_name:
        return {"success": False, "error": "No property name provided"}
    if value is None:
        return {"success": False, "error": "No value provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    obj[prop_name] = value
    return {"success": True, "object": name, "property": prop_name, "value": value}


def handle_delete_custom_prop(args):
    """Delete a custom property from an object."""
    name = args.get("name", "")
    prop_name = args.get("property", "")

    if not name:
        return {"success": False, "error": "No object name provided"}
    if not prop_name:
        return {"success": False, "error": "No property name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    if prop_name not in obj.keys():
        return {"success": False, "error": f"Property '{prop_name}' not found on object"}

    del obj[prop_name]
    return {"success": True, "object": name, "deleted_property": prop_name}


def handle_list_actions(args):
    """List all animation actions in the file."""
    actions = []
    for action in bpy.data.actions:
        action_info = {
            "name": action.name,
            "frame_start": action.frame_range[0],
            "frame_end": action.frame_range[1],
            "fcurve_count": len(action.fcurves),
        }
        # Get which objects use this action
        users = []
        for obj in bpy.data.objects:
            if obj.animation_data and obj.animation_data.action == action:
                users.append(obj.name)
        action_info["users"] = users
        actions.append(action_info)

    return {"success": True, "actions": actions, "count": len(actions)}


def handle_set_active_action(args):
    """Set the active action on an object."""
    name = args.get("name", "")
    action_name = args.get("action", "")

    if not name:
        return {"success": False, "error": "No object name provided"}
    if not action_name:
        return {"success": False, "error": "No action name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    action = bpy.data.actions.get(action_name)
    if not action:
        return {"success": False, "error": f"Action '{action_name}' not found"}

    # Ensure object has animation data
    if not obj.animation_data:
        obj.animation_data_create()

    obj.animation_data.action = action
    return {"success": True, "object": name, "action": action_name}


def handle_get_armature_info(args):
    """Get bone hierarchy and info from an armature."""
    name = args.get("name", "")
    if not name:
        return {"success": False, "error": "No object name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    if obj.type != 'ARMATURE':
        return {"success": False, "error": f"Object '{name}' is not an armature (type: {obj.type})"}

    armature = obj.data
    bones = []

    for bone in armature.bones:
        bone_info = {
            "name": bone.name,
            "parent": bone.parent.name if bone.parent else None,
            "children": [child.name for child in bone.children],
            "head": list(bone.head_local),
            "tail": list(bone.tail_local),
            "length": bone.length,
            "connected": bone.use_connect,
            "deform": bone.use_deform,
        }
        bones.append(bone_info)

    # Build hierarchy for convenience
    def build_hierarchy(bone_name):
        bone = armature.bones.get(bone_name)
        if not bone:
            return None
        result = {"name": bone_name, "children": []}
        for child in bone.children:
            result["children"].append(build_hierarchy(child.name))
        return result

    root_bones = [b.name for b in armature.bones if b.parent is None]
    hierarchy = [build_hierarchy(name) for name in root_bones]

    return {
        "success": True,
        "armature": obj.name,
        "bone_count": len(bones),
        "bones": bones,
        "hierarchy": hierarchy
    }


def handle_apply_transforms(args):
    """Apply object transforms (location, rotation, scale)."""
    name = args.get("name", "")
    if not name:
        return {"success": False, "error": "No object name provided"}

    obj = bpy.data.objects.get(name)
    if not obj:
        return {"success": False, "error": f"Object '{name}' not found"}

    # Select and make active
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj

    # Apply transforms
    location = args.get("location", True)
    rotation = args.get("rotation", True)
    scale = args.get("scale", True)

    bpy.ops.object.transform_apply(location=location, rotation=rotation, scale=scale)

    return {"success": True, "object": get_object_info(obj)}


def process_client_data(data):
    """Process raw data from client and return response."""
    try:
        cmd_data = json.loads(data.decode('utf-8'))
        result = handle_command(cmd_data)
        return json.dumps(result).encode('utf-8')
    except json.JSONDecodeError as e:
        error = {"success": False, "error": f"Invalid JSON: {str(e)}"}
        return json.dumps(error).encode('utf-8')
    except Exception as e:
        error = {"success": False, "error": str(e), "traceback": traceback.format_exc()}
        return json.dumps(error).encode('utf-8')


def server_tick():
    """Called periodically by Blender's timer to handle socket connections."""
    global server_socket, is_running

    if not is_running or server_socket is None:
        return None  # Stop the timer

    try:
        # Check for new connections (non-blocking)
        server_socket.setblocking(False)
        try:
            client_socket, addr = server_socket.accept()
            client_socket.settimeout(5.0)  # 5 second timeout for client operations

            try:
                # Receive data
                data = b''
                while True:
                    chunk = client_socket.recv(BUFFER_SIZE)
                    if not chunk:
                        break
                    data += chunk
                    # Check if we got complete JSON (simple heuristic)
                    try:
                        json.loads(data.decode('utf-8'))
                        break  # Valid JSON, stop reading
                    except json.JSONDecodeError:
                        continue  # Keep reading

                if data:
                    response = process_client_data(data)
                    client_socket.sendall(response)

            finally:
                client_socket.close()

        except BlockingIOError:
            pass  # No connection waiting, that's fine
        except socket.timeout:
            pass  # Timeout, continue

    except Exception as e:
        print(f"Blender Bridge error: {e}")

    return 0.1  # Check again in 0.1 seconds


def start_server():
    """Start the TCP socket server."""
    global server_socket, is_running

    if is_running:
        return False, "Server already running"

    try:
        server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server_socket.bind((HOST, PORT))
        server_socket.listen(5)
        server_socket.setblocking(False)

        is_running = True
        bpy.app.timers.register(server_tick)

        return True, f"Server started on {HOST}:{PORT}"
    except Exception as e:
        return False, f"Failed to start server: {e}"


def stop_server():
    """Stop the TCP socket server."""
    global server_socket, is_running

    is_running = False

    if server_socket:
        try:
            server_socket.close()
        except:
            pass
        server_socket = None

    # Timer will stop itself when is_running is False
    return True, "Server stopped"


# Blender UI Panel
class BRIDGE_PT_Panel(bpy.types.Panel):
    bl_label = "Blender Bridge"
    bl_idname = "BRIDGE_PT_panel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Bridge'

    def draw(self, context):
        layout = self.layout

        if is_running:
            layout.label(text=f"Server: Running", icon='CHECKMARK')
            layout.label(text=f"Port: {PORT}")
            layout.operator("bridge.stop_server", text="Stop Server", icon='CANCEL')
        else:
            layout.label(text="Server: Stopped", icon='X')
            layout.operator("bridge.start_server", text="Start Server", icon='PLAY')


class BRIDGE_OT_StartServer(bpy.types.Operator):
    bl_idname = "bridge.start_server"
    bl_label = "Start Bridge Server"
    bl_description = "Start the TCP socket server for external control"

    def execute(self, context):
        success, message = start_server()
        self.report({'INFO' if success else 'ERROR'}, message)
        return {'FINISHED'}


class BRIDGE_OT_StopServer(bpy.types.Operator):
    bl_idname = "bridge.stop_server"
    bl_label = "Stop Bridge Server"
    bl_description = "Stop the TCP socket server"

    def execute(self, context):
        success, message = stop_server()
        self.report({'INFO'}, message)
        return {'FINISHED'}


classes = [
    BRIDGE_PT_Panel,
    BRIDGE_OT_StartServer,
    BRIDGE_OT_StopServer,
]


def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    print(f"Blender Bridge addon registered. Use the Bridge panel to start the server on port {PORT}.")


def unregister():
    stop_server()
    for cls in reversed(classes):
        bpy.utils.unregister_class(cls)
    print("Blender Bridge addon unregistered.")


if __name__ == "__main__":
    register()
