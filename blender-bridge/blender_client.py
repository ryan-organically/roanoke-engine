#!/usr/bin/env python3
"""
Blender Bridge Client
CLI tool for communicating with Blender via the Bridge addon.
"""

import argparse
import json
import socket
import sys

HOST = '127.0.0.1'
PORT = 9876
TIMEOUT = 30


def send_command(command: str, args: dict = None, host: str = HOST, port: int = PORT) -> dict:
    """Send a command to the Blender Bridge server and return the response."""
    payload = {"command": command}
    if args:
        payload["args"] = args

    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(TIMEOUT)
            sock.connect((host, port))
            sock.sendall(json.dumps(payload).encode('utf-8'))
            sock.shutdown(socket.SHUT_WR)  # Signal end of sending

            # Receive response
            response = b''
            while True:
                chunk = sock.recv(65536)
                if not chunk:
                    break
                response += chunk

            return json.loads(response.decode('utf-8'))

    except ConnectionRefusedError:
        return {"success": False, "error": "Connection refused. Is Blender running with the Bridge addon enabled?"}
    except socket.timeout:
        return {"success": False, "error": "Connection timed out"}
    except Exception as e:
        return {"success": False, "error": str(e)}


def print_response(response: dict, raw: bool = False):
    """Print the response in a readable format."""
    if raw:
        print(json.dumps(response, indent=2))
        return

    if not response.get("success", False):
        print(f"ERROR: {response.get('error', 'Unknown error')}")
        if response.get("traceback"):
            print(f"\nTraceback:\n{response['traceback']}")
        return

    # Pretty print based on response type
    if "objects" in response:
        print(f"Objects ({response.get('count', len(response['objects']))}):")
        for obj in response["objects"]:
            loc = obj.get("location", [0, 0, 0])
            print(f"  {obj['name']:<20} [{obj['type']:<10}] @ ({loc[0]:.2f}, {loc[1]:.2f}, {loc[2]:.2f})")

    elif "materials" in response:
        print(f"Materials ({response.get('count', len(response['materials']))}):")
        for mat in response["materials"]:
            print(f"  {mat['name']}")

    elif "selected" in response and isinstance(response["selected"], list):
        print(f"Selected objects ({response.get('count', 0)}):")
        for obj in response["selected"]:
            print(f"  {obj['name']} [{obj['type']}]")
        if response.get("active"):
            print(f"Active: {response['active']['name']}")

    elif "scene" in response:
        scene = response["scene"]
        print(f"Scene: {scene['name']}")
        print(f"  File: {scene['filepath']}")
        print(f"  Objects: {scene['object_count']}")
        print(f"  Frames: {scene['frame_start']} - {scene['frame_end']} (current: {scene['frame_current']})")
        print(f"  FPS: {scene['fps']}")
        print(f"  Resolution: {scene['resolution'][0]}x{scene['resolution'][1]}")

    elif "created" in response:
        obj = response.get("object", {})
        print(f"Created: {response['created']}")
        if obj:
            loc = obj.get("location", [0, 0, 0])
            print(f"  Location: ({loc[0]:.2f}, {loc[1]:.2f}, {loc[2]:.2f})")

    elif "exported" in response:
        print(f"Exported to: {response['exported']}")

    elif "saved" in response:
        print(f"Saved to: {response['saved']}")

    elif "deleted" in response:
        print(f"Deleted: {response['deleted']}")

    elif "duplicate" in response:
        print(f"Duplicated '{response['original']}' -> '{response['duplicate']}'")

    elif "new_name" in response:
        print(f"Renamed '{response['old_name']}' -> '{response['new_name']}'")

    elif "object" in response:
        obj = response["object"]
        print(f"Object: {obj['name']} [{obj['type']}]")
        print(f"  Location: {obj['location']}")
        print(f"  Rotation: {obj['rotation']}")
        print(f"  Scale: {obj['scale']}")

    elif "result" in response:
        result = response["result"]
        if result is not None:
            if isinstance(result, (dict, list)):
                print(json.dumps(result, indent=2))
            else:
                print(result)
        else:
            print("OK")
    else:
        # Fallback: print full response
        print(json.dumps(response, indent=2))


def interactive_mode(host: str, port: int, raw: bool):
    """Run in interactive mode."""
    print(f"Blender Bridge Interactive Mode")
    print(f"Connected to {host}:{port}")
    print("Type 'help' for commands, 'quit' to exit\n")

    while True:
        try:
            line = input("blender> ").strip()
        except (EOFError, KeyboardInterrupt):
            print("\nBye!")
            break

        if not line:
            continue

        if line.lower() in ('quit', 'exit', 'q'):
            print("Bye!")
            break

        if line.lower() == 'help':
            print("""
Available commands:
  ping                      - Test connection
  exec <code>               - Execute Python code in Blender
  list [type]               - List objects (optionally filter by type)
  materials                 - List all materials
  select <name>             - Select object by name
  delete <name>             - Delete object by name
  add <type> [x y z]        - Add primitive (cube, sphere, plane, etc.)
  move <name> <x> <y> <z>   - Set object location
  rotate <name> <x> <y> <z> - Set object rotation (degrees)
  scale <name> <x> <y> <z>  - Set object scale
  dup <name>                - Duplicate object
  rename <old> <new>        - Rename object
  selected                  - Show selected objects
  scene                     - Show scene info
  export-fbx <path>         - Export selection to FBX
  export-gltf <path>        - Export selection to glTF
  screenshot <path>         - Save viewport render
  raw                       - Toggle raw JSON output
  help                      - Show this help
  quit                      - Exit
""")
            continue

        if line.lower() == 'raw':
            raw = not raw
            print(f"Raw output: {'ON' if raw else 'OFF'}")
            continue

        # Parse and execute command
        parts = line.split(maxsplit=1)
        cmd = parts[0].lower()
        rest = parts[1] if len(parts) > 1 else ""

        response = None

        if cmd == 'ping':
            response = send_command("ping", host=host, port=port)

        elif cmd == 'exec':
            response = send_command("exec", {"code": rest}, host=host, port=port)

        elif cmd == 'list':
            args = {"type": rest} if rest else {}
            response = send_command("list_objects", args, host=host, port=port)

        elif cmd == 'materials':
            response = send_command("list_materials", host=host, port=port)

        elif cmd == 'select':
            response = send_command("select", {"name": rest}, host=host, port=port)

        elif cmd == 'delete':
            response = send_command("delete", {"name": rest}, host=host, port=port)

        elif cmd == 'add':
            add_parts = rest.split()
            prim_type = add_parts[0] if add_parts else "cube"
            location = [float(x) for x in add_parts[1:4]] if len(add_parts) > 3 else [0, 0, 0]
            response = send_command("add_primitive", {"type": prim_type, "location": location}, host=host, port=port)

        elif cmd == 'move':
            move_parts = rest.split()
            if len(move_parts) >= 4:
                response = send_command("set_location", {
                    "name": move_parts[0],
                    "location": [float(x) for x in move_parts[1:4]]
                }, host=host, port=port)
            else:
                print("Usage: move <name> <x> <y> <z>")

        elif cmd == 'rotate':
            rot_parts = rest.split()
            if len(rot_parts) >= 4:
                response = send_command("set_rotation", {
                    "name": rot_parts[0],
                    "rotation": [float(x) for x in rot_parts[1:4]]
                }, host=host, port=port)
            else:
                print("Usage: rotate <name> <x> <y> <z>")

        elif cmd == 'scale':
            scale_parts = rest.split()
            if len(scale_parts) >= 4:
                response = send_command("set_scale", {
                    "name": scale_parts[0],
                    "scale": [float(x) for x in scale_parts[1:4]]
                }, host=host, port=port)
            else:
                print("Usage: scale <name> <x> <y> <z>")

        elif cmd == 'dup':
            response = send_command("duplicate", {"name": rest}, host=host, port=port)

        elif cmd == 'rename':
            rename_parts = rest.split(maxsplit=1)
            if len(rename_parts) >= 2:
                response = send_command("rename", {
                    "name": rename_parts[0],
                    "new_name": rename_parts[1]
                }, host=host, port=port)
            else:
                print("Usage: rename <old_name> <new_name>")

        elif cmd == 'selected':
            response = send_command("get_selected", host=host, port=port)

        elif cmd == 'scene':
            response = send_command("get_scene_info", host=host, port=port)

        elif cmd in ('export-fbx', 'exportfbx', 'fbx'):
            response = send_command("export_fbx", {"filepath": rest}, host=host, port=port)

        elif cmd in ('export-gltf', 'exportgltf', 'gltf', 'glb'):
            response = send_command("export_gltf", {"filepath": rest}, host=host, port=port)

        elif cmd == 'screenshot':
            response = send_command("screenshot", {"filepath": rest}, host=host, port=port)

        else:
            print(f"Unknown command: {cmd}. Type 'help' for available commands.")

        if response:
            print_response(response, raw)


def main():
    parser = argparse.ArgumentParser(
        description="Blender Bridge Client - Control Blender from the terminal",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --ping                                    Test connection
  %(prog)s --exec "bpy.ops.mesh.primitive_cube_add()"   Run Python code
  %(prog)s --list-objects                            List all objects
  %(prog)s --add-primitive cube 1 2 3                Add cube at (1,2,3)
  %(prog)s --select Cube                             Select object named Cube
  %(prog)s --export-fbx /path/to/file.fbx            Export selection to FBX
  %(prog)s -i                                        Interactive mode
"""
    )

    parser.add_argument('--host', default=HOST, help=f'Server host (default: {HOST})')
    parser.add_argument('--port', type=int, default=PORT, help=f'Server port (default: {PORT})')
    parser.add_argument('--raw', '-r', action='store_true', help='Output raw JSON')
    parser.add_argument('--interactive', '-i', action='store_true', help='Interactive mode')

    # Commands
    cmd_group = parser.add_argument_group('Commands')
    cmd_group.add_argument('--ping', action='store_true', help='Test connection to Blender')
    cmd_group.add_argument('--exec', '-e', metavar='CODE', help='Execute Python code in Blender')
    cmd_group.add_argument('--list-objects', '-l', nargs='?', const='', metavar='TYPE',
                          help='List objects (optionally filter by type: MESH, LIGHT, CAMERA, etc.)')
    cmd_group.add_argument('--list-materials', action='store_true', help='List all materials')
    cmd_group.add_argument('--select', '-s', metavar='NAME', help='Select object by name')
    cmd_group.add_argument('--delete', '-d', metavar='NAME', help='Delete object by name')
    cmd_group.add_argument('--add-primitive', '-a', nargs='+', metavar='ARG',
                          help='Add primitive: TYPE [X Y Z]')
    cmd_group.add_argument('--export-fbx', metavar='PATH', help='Export selection to FBX')
    cmd_group.add_argument('--export-gltf', metavar='PATH', help='Export selection to glTF/GLB')
    cmd_group.add_argument('--get-selected', action='store_true', help='Get selected objects')
    cmd_group.add_argument('--screenshot', metavar='PATH', help='Save viewport render')
    cmd_group.add_argument('--scene-info', action='store_true', help='Get scene information')
    cmd_group.add_argument('--set-location', nargs=4, metavar=('NAME', 'X', 'Y', 'Z'),
                          help='Set object location')
    cmd_group.add_argument('--set-rotation', nargs=4, metavar=('NAME', 'X', 'Y', 'Z'),
                          help='Set object rotation (degrees)')
    cmd_group.add_argument('--set-scale', nargs=4, metavar=('NAME', 'X', 'Y', 'Z'),
                          help='Set object scale')
    cmd_group.add_argument('--duplicate', metavar='NAME', help='Duplicate object')
    cmd_group.add_argument('--rename', nargs=2, metavar=('OLD', 'NEW'), help='Rename object')

    # Direct JSON command
    cmd_group.add_argument('--json', '-j', metavar='JSON',
                          help='Send raw JSON command: {"command": "...", "args": {...}}')

    args = parser.parse_args()

    # Interactive mode
    if args.interactive:
        interactive_mode(args.host, args.port, args.raw)
        return

    # Process commands
    response = None

    if args.ping:
        response = send_command("ping", host=args.host, port=args.port)

    elif args.exec:
        response = send_command("exec", {"code": args.exec}, host=args.host, port=args.port)

    elif args.list_objects is not None:
        cmd_args = {"type": args.list_objects} if args.list_objects else {}
        response = send_command("list_objects", cmd_args, host=args.host, port=args.port)

    elif args.list_materials:
        response = send_command("list_materials", host=args.host, port=args.port)

    elif args.select:
        response = send_command("select", {"name": args.select}, host=args.host, port=args.port)

    elif args.delete:
        response = send_command("delete", {"name": args.delete}, host=args.host, port=args.port)

    elif args.add_primitive:
        prim_type = args.add_primitive[0]
        location = [float(x) for x in args.add_primitive[1:4]] if len(args.add_primitive) > 3 else [0, 0, 0]
        response = send_command("add_primitive", {"type": prim_type, "location": location},
                               host=args.host, port=args.port)

    elif args.export_fbx:
        response = send_command("export_fbx", {"filepath": args.export_fbx},
                               host=args.host, port=args.port)

    elif args.export_gltf:
        response = send_command("export_gltf", {"filepath": args.export_gltf},
                               host=args.host, port=args.port)

    elif args.get_selected:
        response = send_command("get_selected", host=args.host, port=args.port)

    elif args.screenshot:
        response = send_command("screenshot", {"filepath": args.screenshot},
                               host=args.host, port=args.port)

    elif args.scene_info:
        response = send_command("get_scene_info", host=args.host, port=args.port)

    elif args.set_location:
        response = send_command("set_location", {
            "name": args.set_location[0],
            "location": [float(x) for x in args.set_location[1:4]]
        }, host=args.host, port=args.port)

    elif args.set_rotation:
        response = send_command("set_rotation", {
            "name": args.set_rotation[0],
            "rotation": [float(x) for x in args.set_rotation[1:4]]
        }, host=args.host, port=args.port)

    elif args.set_scale:
        response = send_command("set_scale", {
            "name": args.set_scale[0],
            "scale": [float(x) for x in args.set_scale[1:4]]
        }, host=args.host, port=args.port)

    elif args.duplicate:
        response = send_command("duplicate", {"name": args.duplicate},
                               host=args.host, port=args.port)

    elif args.rename:
        response = send_command("rename", {
            "name": args.rename[0],
            "new_name": args.rename[1]
        }, host=args.host, port=args.port)

    elif args.json:
        try:
            cmd_data = json.loads(args.json)
            response = send_command(cmd_data.get("command", ""), cmd_data.get("args", {}),
                                   host=args.host, port=args.port)
        except json.JSONDecodeError as e:
            print(f"Invalid JSON: {e}")
            sys.exit(1)

    else:
        parser.print_help()
        return

    if response:
        print_response(response, args.raw)
        sys.exit(0 if response.get("success", False) else 1)


if __name__ == "__main__":
    main()
