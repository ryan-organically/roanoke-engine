"""
Quadruped Engine - Limb control system for four-legged creatures
Works with Blender via blender-bridge
"""

import json
import socket
from dataclasses import dataclass, field
from typing import Optional
from enum import Enum


class LimbType(Enum):
    FRONT_LEFT = "FL"
    FRONT_RIGHT = "FR"
    BACK_LEFT = "BL"
    BACK_RIGHT = "BR"


@dataclass
class Joint:
    """Represents a single joint/bone in the skeleton"""
    name: str
    parent: Optional[str] = None
    children: list = field(default_factory=list)
    joint_type: str = "unknown"  # shoulder, upper_leg, lower_leg, hoof, etc.


@dataclass
class Limb:
    """Represents a complete limb chain"""
    limb_type: LimbType
    joints: list  # ordered from root to tip
    ik_target: Optional[str] = None
    pole_target: Optional[str] = None


@dataclass
class QuadrupedRig:
    """Complete quadruped skeleton definition"""
    armature_name: str
    root_bone: str
    spine_chain: list  # Body -> Back -> Torso chain
    neck_chain: list   # Neck bones
    head_bone: str
    limbs: dict        # LimbType -> Limb
    tail_chain: list


class QuadrupedEngine:
    """
    Engine for controlling quadruped rigs in Blender
    """

    def __init__(self, host: str = "172.19.16.1", port: int = 9876):
        self.host = host
        self.port = port
        self.rig: Optional[QuadrupedRig] = None

    def _send_command(self, command: dict) -> dict:
        """Send command to Blender bridge"""
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect((self.host, self.port))
            s.sendall(json.dumps(command).encode() + b'\n')
            response = b''
            while True:
                chunk = s.recv(4096)
                if not chunk:
                    break
                response += chunk
            return json.loads(response.decode())

    def exec_blender(self, code: str):
        """Execute Python code in Blender"""
        return self._send_command({"command": "exec", "args": {"code": code}})

    def get_bone_hierarchy(self, armature_name: str) -> list:
        """Get all bones with parent relationships"""
        code = f"[(b.name, b.parent.name if b.parent else None) for b in bpy.data.objects['{armature_name}'].data.bones]"
        result = self._send_command({"command": "exec", "args": {"code": code}})
        if result.get("success"):
            return result.get("result", [])
        return []

    def identify_quadruped_joints(self, armature_name: str) -> QuadrupedRig:
        """
        Auto-identify joints in a quadruped skeleton.
        Uses naming conventions and hierarchy analysis.
        """
        hierarchy = self.get_bone_hierarchy(armature_name)

        # Build lookup structures
        bones = {}
        children_map = {}

        for bone_name, parent_name in hierarchy:
            bones[bone_name] = parent_name
            if parent_name:
                if parent_name not in children_map:
                    children_map[parent_name] = []
                children_map[parent_name].append(bone_name)

        # Find root (no parent)
        roots = [name for name, parent in bones.items() if parent is None]

        # Classify bones by naming patterns
        limb_patterns = {
            LimbType.FRONT_LEFT: ["front", "l", "left"],
            LimbType.FRONT_RIGHT: ["front", "r", "right"],
            LimbType.BACK_LEFT: ["back", "rear", "hind", "l", "left"],
            LimbType.BACK_RIGHT: ["back", "rear", "hind", "r", "right"],
        }

        # Identify limbs
        limbs = {}
        identified_limb_bones = set()

        for limb_type in LimbType:
            limb_bones = self._find_limb_chain(bones, children_map, limb_type)
            if limb_bones:
                ik_target = self._find_ik_target(bones, limb_type)
                pole_target = self._find_pole_target(bones, limb_type)
                limbs[limb_type] = Limb(
                    limb_type=limb_type,
                    joints=limb_bones,
                    ik_target=ik_target,
                    pole_target=pole_target
                )
                identified_limb_bones.update(limb_bones)

        # Identify spine (bones with "body", "back", "torso", "spine")
        spine_chain = self._find_chain_by_pattern(bones, children_map,
            ["body", "back", "torso", "spine"], exclude=identified_limb_bones)

        # Identify neck
        neck_chain = self._find_chain_by_pattern(bones, children_map,
            ["neck"], exclude=identified_limb_bones)

        # Identify head
        head_bone = self._find_bone_by_pattern(bones, ["head"])

        # Identify tail
        tail_chain = self._find_chain_by_pattern(bones, children_map,
            ["tail"], exclude=identified_limb_bones)

        self.rig = QuadrupedRig(
            armature_name=armature_name,
            root_bone=roots[0] if roots else "Body",
            spine_chain=spine_chain,
            neck_chain=neck_chain,
            head_bone=head_bone or "Head",
            limbs=limbs,
            tail_chain=tail_chain
        )

        return self.rig

    def _find_limb_chain(self, bones: dict, children_map: dict, limb_type: LimbType) -> list:
        """Find the bone chain for a specific limb"""
        suffix = ".L" if "LEFT" in limb_type.name else ".R"
        prefix = "Front" if "FRONT" in limb_type.name else "Back"

        # Look for shoulder/hip as chain start
        chain = []

        # Try common naming patterns
        shoulder_patterns = [
            f"{prefix}Shoulder{suffix}",
            f"{prefix}Hip{suffix}",
            f"Shoulder_{prefix}{suffix}",
        ]

        start_bone = None
        for pattern in shoulder_patterns:
            if pattern in bones:
                start_bone = pattern
                break

        if not start_bone:
            # Fuzzy search
            for bone_name in bones:
                name_lower = bone_name.lower()
                if suffix.lower() in name_lower:
                    if "front" in name_lower and "FRONT" in limb_type.name:
                        if "shoulder" in name_lower or "hip" in name_lower:
                            start_bone = bone_name
                            break
                    elif ("back" in name_lower or "rear" in name_lower) and "BACK" in limb_type.name:
                        if "shoulder" in name_lower or "hip" in name_lower:
                            start_bone = bone_name
                            break

        if not start_bone:
            return []

        # Walk down the chain
        chain = [start_bone]
        current = start_bone
        while current in children_map:
            # Find the main child (not IK targets or pole targets)
            main_children = [c for c in children_map[current]
                           if "ik" not in c.lower() and "pole" not in c.lower() and "target" not in c.lower()]
            if main_children:
                # Prefer children with "leg" in name
                leg_children = [c for c in main_children if "leg" in c.lower()]
                next_bone = leg_children[0] if leg_children else main_children[0]
                chain.append(next_bone)
                current = next_bone
            else:
                break

        return chain

    def _find_ik_target(self, bones: dict, limb_type: LimbType) -> Optional[str]:
        """Find IK target bone for a limb"""
        suffix = ".L" if "LEFT" in limb_type.name else ".R"
        prefix = "Front" if "FRONT" in limb_type.name else "Back"

        patterns = [
            f"IK{prefix}Leg{suffix}",
            f"IK_{prefix}_{suffix}",
            f"{prefix}_IK{suffix}",
        ]

        for pattern in patterns:
            if pattern in bones:
                return pattern

        # Fuzzy search
        for bone_name in bones:
            name_lower = bone_name.lower()
            if "ik" in name_lower and suffix.lower() in name_lower:
                if ("front" in name_lower and "FRONT" in limb_type.name) or \
                   ("back" in name_lower and "BACK" in limb_type.name):
                    return bone_name

        return None

    def _find_pole_target(self, bones: dict, limb_type: LimbType) -> Optional[str]:
        """Find pole target bone for a limb"""
        suffix = ".L" if "LEFT" in limb_type.name else ".R"
        is_front = "FRONT" in limb_type.name

        for bone_name in bones:
            name_lower = bone_name.lower()
            if "pole" in name_lower and suffix.lower() in name_lower:
                if is_front and "back" not in name_lower:
                    return bone_name
                elif not is_front and "back" in name_lower:
                    return bone_name

        return None

    def _find_chain_by_pattern(self, bones: dict, children_map: dict,
                                patterns: list, exclude: set = None) -> list:
        """Find a chain of bones matching patterns"""
        exclude = exclude or set()
        matching = []

        for bone_name in bones:
            if bone_name in exclude:
                continue
            name_lower = bone_name.lower()
            for pattern in patterns:
                if pattern in name_lower:
                    matching.append(bone_name)
                    break

        # Sort by hierarchy (parents first)
        def get_depth(bone):
            depth = 0
            current = bone
            while bones.get(current):
                current = bones[current]
                depth += 1
            return depth

        matching.sort(key=get_depth)
        return matching

    def _find_bone_by_pattern(self, bones: dict, patterns: list) -> Optional[str]:
        """Find a single bone matching patterns"""
        for bone_name in bones:
            name_lower = bone_name.lower()
            for pattern in patterns:
                if pattern in name_lower:
                    return bone_name
        return None

    # ==================== LIMB CONTROL API ====================

    def set_limb_position(self, limb_type: LimbType, x: float, y: float, z: float):
        """Move a limb's IK target to world position"""
        if not self.rig or limb_type not in self.rig.limbs:
            return {"error": "Rig not loaded or limb not found"}

        limb = self.rig.limbs[limb_type]
        if not limb.ik_target:
            return {"error": f"No IK target for {limb_type.name}"}

        code = f"""
import bpy
arm = bpy.data.objects["{self.rig.armature_name}"]
bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode="POSE")
bone = arm.pose.bones["{limb.ik_target}"]
bone.location = ({x}, {y}, {z})
bpy.ops.object.mode_set(mode="OBJECT")
"OK"
"""
        return self.exec_blender(code)

    def set_limb_ik_position(self, limb_type: LimbType, x: float, y: float, z: float):
        """Set IK target position (relative to bone's rest position)"""
        return self.set_limb_position(limb_type, x, y, z)

    def rotate_joint(self, bone_name: str, rx: float, ry: float, rz: float, degrees: bool = True):
        """Rotate a specific joint"""
        if not self.rig:
            return {"error": "Rig not loaded"}

        if degrees:
            import math
            rx, ry, rz = math.radians(rx), math.radians(ry), math.radians(rz)

        code = f"""
import bpy
arm = bpy.data.objects["{self.rig.armature_name}"]
bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode="POSE")
bone = arm.pose.bones["{bone_name}"]
bone.rotation_mode = "XYZ"
bone.rotation_euler = ({rx}, {ry}, {rz})
bpy.ops.object.mode_set(mode="OBJECT")
"OK"
"""
        return self.exec_blender(code)

    def bend_neck(self, angle: float, axis: str = "x"):
        """Bend the neck chain by distributing rotation across neck bones"""
        if not self.rig or not self.rig.neck_chain:
            return {"error": "Rig not loaded or no neck chain"}

        import math
        per_bone_angle = math.radians(angle) / len(self.rig.neck_chain)

        rotations = []
        for bone in self.rig.neck_chain:
            if axis.lower() == "x":
                rotations.append(f'arm.pose.bones["{bone}"].rotation_euler.x += {per_bone_angle}')
            elif axis.lower() == "y":
                rotations.append(f'arm.pose.bones["{bone}"].rotation_euler.y += {per_bone_angle}')
            else:
                rotations.append(f'arm.pose.bones["{bone}"].rotation_euler.z += {per_bone_angle}')

        code = f"""
import bpy
arm = bpy.data.objects["{self.rig.armature_name}"]
bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode="POSE")
for bone in arm.pose.bones:
    bone.rotation_mode = "XYZ"
{chr(10).join(rotations)}
bpy.ops.object.mode_set(mode="OBJECT")
"Neck bent {angle} degrees"
"""
        return self.exec_blender(code)

    def reset_pose(self):
        """Reset all bones to rest pose"""
        if not self.rig:
            return {"error": "Rig not loaded"}

        code = f"""
import bpy
arm = bpy.data.objects["{self.rig.armature_name}"]
bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode="POSE")
bpy.ops.pose.select_all(action="SELECT")
bpy.ops.pose.transforms_clear()
bpy.ops.object.mode_set(mode="OBJECT")
"Pose reset"
"""
        return self.exec_blender(code)

    def get_rig_summary(self) -> dict:
        """Get a summary of the identified rig"""
        if not self.rig:
            return {"error": "No rig loaded"}

        return {
            "armature": self.rig.armature_name,
            "root": self.rig.root_bone,
            "spine": self.rig.spine_chain,
            "neck": self.rig.neck_chain,
            "head": self.rig.head_bone,
            "tail": self.rig.tail_chain,
            "limbs": {
                lt.name: {
                    "joints": self.rig.limbs[lt].joints,
                    "ik_target": self.rig.limbs[lt].ik_target,
                    "pole_target": self.rig.limbs[lt].pole_target
                }
                for lt in self.rig.limbs
            }
        }


    # ==================== PROCEDURAL GAITS ====================

    def create_graze_pose(self, reach_depth: float = 75.0):
        """
        Create a grazing pose with the head reaching down.
        reach_depth: how far down the head reaches (degrees)
        """
        if not self.rig:
            return {"error": "Rig not loaded"}

        import math
        neck_bend = reach_depth * 0.6  # 60% of reach in neck
        head_bend = reach_depth * 0.4  # 40% in head

        # Negative X rotation = bend DOWN
        neck_per_bone = math.radians(-neck_bend / len(self.rig.neck_chain))
        head_angle = math.radians(-head_bend)

        neck_rotations = "\n".join([
            f'arm.pose.bones["{bone}"].rotation_euler.x = {neck_per_bone}'
            for bone in self.rig.neck_chain
        ])

        code = f"""
import bpy
import math

arm = bpy.data.objects["{self.rig.armature_name}"]
bpy.context.view_layer.objects.active = arm
bpy.ops.object.mode_set(mode="POSE")

# Set rotation mode
for bone in arm.pose.bones:
    bone.rotation_mode = "XYZ"

# Bend neck
{neck_rotations}

# Bend head
arm.pose.bones["{self.rig.head_bone}"].rotation_euler.x = {head_angle}

bpy.ops.object.mode_set(mode="OBJECT")
"Graze pose: {reach_depth} degrees reach"
"""
        return self.exec_blender(code)

    def create_walk_cycle(self, stride_length: float = 0.5, num_frames: int = 24):
        """
        Create a procedural walk cycle animation.
        Uses diagonal gait: FL+BR together, FR+BL together.
        """
        if not self.rig:
            return {"error": "Rig not loaded"}

        import math

        code = f"""
import bpy
import math

arm = bpy.data.objects["{self.rig.armature_name}"]
bpy.context.view_layer.objects.active = arm

# Create new action
action = bpy.data.actions.new(name="Procedural_Walk")
if not arm.animation_data:
    arm.animation_data_create()
arm.animation_data.action = action

bpy.ops.object.mode_set(mode="POSE")

# Reset pose
bpy.ops.pose.select_all(action="SELECT")
bpy.ops.pose.transforms_clear()

stride = {stride_length}
frames = {num_frames}
half = frames // 2

# IK targets for each leg
ik_bones = {{
    "FL": arm.pose.bones["IKFrontLeg.L"],
    "FR": arm.pose.bones["IKFrontLeg.R"],
    "BL": arm.pose.bones["IKBackLeg.L"],
    "BR": arm.pose.bones["IKBackLeg.R"],
}}

# Diagonal pairs: FL+BR are in phase, FR+BL are opposite
for frame in range(frames + 1):
    bpy.context.scene.frame_set(frame)
    t = frame / frames  # 0 to 1

    # Sinusoidal motion
    phase_a = math.sin(t * 2 * math.pi)  # FL, BR
    phase_b = math.sin((t + 0.5) * 2 * math.pi)  # FR, BL

    # Vertical lift (only when moving forward)
    lift_a = max(0, math.sin(t * 2 * math.pi)) * 0.15
    lift_b = max(0, math.sin((t + 0.5) * 2 * math.pi)) * 0.15

    # FL - Front Left
    ik_bones["FL"].location.y = phase_a * stride * 0.5
    ik_bones["FL"].location.z = lift_a
    ik_bones["FL"].keyframe_insert(data_path="location", frame=frame)

    # BR - Back Right (in phase with FL)
    ik_bones["BR"].location.y = phase_a * stride * 0.5
    ik_bones["BR"].location.z = lift_a
    ik_bones["BR"].keyframe_insert(data_path="location", frame=frame)

    # FR - Front Right (opposite phase)
    ik_bones["FR"].location.y = phase_b * stride * 0.5
    ik_bones["FR"].location.z = lift_b
    ik_bones["FR"].keyframe_insert(data_path="location", frame=frame)

    # BL - Back Left (opposite phase)
    ik_bones["BL"].location.y = phase_b * stride * 0.5
    ik_bones["BL"].location.z = lift_b
    ik_bones["BL"].keyframe_insert(data_path="location", frame=frame)

# Set scene range
bpy.context.scene.frame_start = 0
bpy.context.scene.frame_end = frames
bpy.context.scene.frame_set(0)

bpy.ops.object.mode_set(mode="OBJECT")
f"Walk cycle created: {{frames}} frames, stride {{stride}}"
"""
        return self.exec_blender(code)

    def create_trot_cycle(self, stride_length: float = 0.7, num_frames: int = 16):
        """
        Create a trot cycle - faster diagonal gait with suspension.
        """
        if not self.rig:
            return {"error": "Rig not loaded"}

        code = f"""
import bpy
import math

arm = bpy.data.objects["{self.rig.armature_name}"]
bpy.context.view_layer.objects.active = arm

action = bpy.data.actions.new(name="Procedural_Trot")
if not arm.animation_data:
    arm.animation_data_create()
arm.animation_data.action = action

bpy.ops.object.mode_set(mode="POSE")
bpy.ops.pose.select_all(action="SELECT")
bpy.ops.pose.transforms_clear()

stride = {stride_length}
frames = {num_frames}

ik_bones = {{
    "FL": arm.pose.bones["IKFrontLeg.L"],
    "FR": arm.pose.bones["IKFrontLeg.R"],
    "BL": arm.pose.bones["IKBackLeg.L"],
    "BR": arm.pose.bones["IKBackLeg.R"],
}}

# Body bone for suspension
body = arm.pose.bones["Body"]

for frame in range(frames + 1):
    bpy.context.scene.frame_set(frame)
    t = frame / frames

    phase_a = math.sin(t * 2 * math.pi)
    phase_b = math.sin((t + 0.5) * 2 * math.pi)

    # Higher lift for trot
    lift_a = max(0, math.sin(t * 2 * math.pi)) * 0.25
    lift_b = max(0, math.sin((t + 0.5) * 2 * math.pi)) * 0.25

    # Suspension - body lifts when both diagonal pairs are in air
    suspension = abs(math.sin(t * 4 * math.pi)) * 0.05
    body.location.z = suspension
    body.keyframe_insert(data_path="location", frame=frame)

    # FL + BR
    ik_bones["FL"].location.y = phase_a * stride * 0.5
    ik_bones["FL"].location.z = lift_a
    ik_bones["FL"].keyframe_insert(data_path="location", frame=frame)

    ik_bones["BR"].location.y = phase_a * stride * 0.5
    ik_bones["BR"].location.z = lift_a
    ik_bones["BR"].keyframe_insert(data_path="location", frame=frame)

    # FR + BL
    ik_bones["FR"].location.y = phase_b * stride * 0.5
    ik_bones["FR"].location.z = lift_b
    ik_bones["FR"].keyframe_insert(data_path="location", frame=frame)

    ik_bones["BL"].location.y = phase_b * stride * 0.5
    ik_bones["BL"].location.z = lift_b
    ik_bones["BL"].keyframe_insert(data_path="location", frame=frame)

bpy.context.scene.frame_start = 0
bpy.context.scene.frame_end = frames
bpy.context.scene.frame_set(0)

bpy.ops.object.mode_set(mode="OBJECT")
f"Trot cycle created: {{frames}} frames"
"""
        return self.exec_blender(code)

    def play_animation(self):
        """Start animation playback"""
        return self.exec_blender("bpy.ops.screen.animation_play(); 'Playing'")

    def stop_animation(self):
        """Stop animation playback"""
        return self.exec_blender("bpy.ops.screen.animation_cancel(); 'Stopped'")


# ==================== CLI Interface ====================

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Quadruped Engine CLI")
    parser.add_argument("--host", default="172.19.16.1", help="Blender bridge host")
    parser.add_argument("--port", type=int, default=9876, help="Blender bridge port")
    parser.add_argument("--armature", default="AnimalArmature", help="Armature object name")
    parser.add_argument("--identify", action="store_true", help="Identify and print rig structure")
    parser.add_argument("--reset", action="store_true", help="Reset pose")
    parser.add_argument("--bend-neck", type=float, help="Bend neck by degrees")
    parser.add_argument("--move-limb", nargs=4, metavar=("LIMB", "X", "Y", "Z"),
                        help="Move limb IK target (FL/FR/BL/BR x y z)")
    parser.add_argument("--graze", type=float, nargs="?", const=75.0,
                        help="Create graze pose (optional: reach depth in degrees)")
    parser.add_argument("--walk", action="store_true", help="Create walk cycle")
    parser.add_argument("--trot", action="store_true", help="Create trot cycle")
    parser.add_argument("--play", action="store_true", help="Play animation")
    parser.add_argument("--stop", action="store_true", help="Stop animation")

    args = parser.parse_args()

    engine = QuadrupedEngine(host=args.host, port=args.port)

    if args.identify:
        rig = engine.identify_quadruped_joints(args.armature)
        summary = engine.get_rig_summary()
        print(json.dumps(summary, indent=2))

    if args.reset:
        engine.identify_quadruped_joints(args.armature)
        result = engine.reset_pose()
        print(result)

    if args.bend_neck is not None:
        engine.identify_quadruped_joints(args.armature)
        result = engine.bend_neck(args.bend_neck)
        print(result)

    if args.move_limb:
        limb_map = {"FL": LimbType.FRONT_LEFT, "FR": LimbType.FRONT_RIGHT,
                    "BL": LimbType.BACK_LEFT, "BR": LimbType.BACK_RIGHT}
        limb_str, x, y, z = args.move_limb
        engine.identify_quadruped_joints(args.armature)
        result = engine.set_limb_position(limb_map[limb_str.upper()],
                                          float(x), float(y), float(z))
        print(result)

    if args.graze is not None:
        engine.identify_quadruped_joints(args.armature)
        result = engine.create_graze_pose(args.graze)
        print(result)

    if args.walk:
        engine.identify_quadruped_joints(args.armature)
        result = engine.create_walk_cycle()
        print(result)

    if args.trot:
        engine.identify_quadruped_joints(args.armature)
        result = engine.create_trot_cycle()
        print(result)

    if args.play:
        result = engine.play_animation()
        print(result)

    if args.stop:
        result = engine.stop_animation()
        print(result)
