# Claude Code Guidelines for Blender Bridge

## Blender Connection (WSL)

When connecting to Blender from WSL, use the Windows host IP (not localhost):

```bash
# Get the WSL gateway IP
ip route show | grep -i default | awk '{ print $3}'

# Use it with the client
python3 blender_client.py --host <GATEWAY_IP> --ping
python3 blender_client.py --host <GATEWAY_IP> --scene-info
```

Typical gateway: `172.19.16.1` (may vary)

**Quick connect sequence:**
1. `ip route show | grep default | awk '{print $3}'` → get host IP
2. `python3 blender_client.py --host <IP> --ping` → verify connection
3. `python3 blender_client.py --host <IP> --scene-info` → inspect scene

## Bridge Command Tips

**exec needs expressions to return values:**
```bash
# WRONG - statements return null
--exec "result = [a.name for a in bpy.data.actions]; print(result)"

# RIGHT - expression returns the value
--exec "[a.name for a in bpy.data.actions]"
```

**Useful one-liner patterns:**
```bash
# List all actions
--exec "[a.name for a in bpy.data.actions]"

# Get action frame ranges
--exec "[(a.name, a.frame_range[0], a.frame_range[1]) for a in bpy.data.actions]"

# Get current active action on armature
--exec "bpy.data.objects['ArmatureName'].animation_data.action.name if bpy.data.objects['ArmatureName'].animation_data and bpy.data.objects['ArmatureName'].animation_data.action else None"

# Check NLA tracks
--exec "[{'track': t.name, 'strips': [s.name for s in t.strips]} for t in bpy.data.objects['ArmatureName'].animation_data.nla_tracks]"

# Get bone names from armature
--exec "[b.name for b in bpy.data.objects['ArmatureName'].data.bones]"

# Check if action loops (first/last frame match)
--exec "bpy.data.actions['ActionName'].frame_range"
```

**For multi-statement operations, use --json with exec:**
```bash
--json '{"command": "exec", "args": {"code": "bpy.context.scene.frame_set(1)"}}'
```

## Core Principle: Discussion Before Action

**When the user asks questions, assume they want to form a strategy first.**

Do NOT immediately execute commands or make edits when the user:
- Asks "how do we..." or "how could I..."
- Asks conceptual questions ("what is UV mapping?")
- Explores options ("could an AI generate textures?")
- Discusses workflow possibilities

Instead:
1. Explain the concepts and options
2. Discuss tradeoffs and approaches
3. **Ask** if they want you to proceed with any specific action
4. Only execute after explicit confirmation

## When Edits ARE Appropriate

Proceed with edits when:
- User explicitly says "do it", "go ahead", "make that change"
- User gives a direct command: "create a UV map for the horse"
- User confirms a proposed action: "yes, export the layout"
- The task is clearly diagnostic (reading data, listing objects, checking status)

## Blender Scene Changes Require Confirmation

The Blender scene is the user's creative work. Before modifying it:
- State what you're about to do
- Ask for confirmation
- Explain any irreversible changes

Examples of changes that need confirmation:
- Creating/modifying UV maps
- Baking animations
- Applying modifiers
- Deleting objects or data
- Exporting files

Examples that don't need confirmation:
- `ping` / connection checks
- Reading scene info
- Listing objects, materials, actions
- Getting properties (read-only inspection)

## Question Patterns

| User says | They likely want |
|-----------|------------------|
| "How do we..." | Strategy discussion |
| "What is..." | Explanation |
| "Could we..." | Options exploration |
| "Can you..." | Capability check, then discussion |
| "Do X" | Action (but confirm if destructive) |
| "Go ahead" | Execute previously discussed action |

## Conversation Flow

```
User asks question
    ↓
Explain concepts/options
    ↓
Propose possible approaches
    ↓
Ask: "Would you like me to [specific action]?"
    ↓
User confirms → Execute
User discusses → Continue planning
```

## Remember

- Questions are conversation starters, not action requests
- The user is the creative director; Claude is the technical advisor
- Rushing to "help" by executing can feel presumptuous
- It's always better to ask than to assume

## PNG Garbage Collection

When rendering preview images:
- Store PNGs in the project root (`/mnt/c/dev/blender-bridge/`)
- Before creating new renders, check PNG count: `ls *.png | wc -l`
- If 5+ PNGs exist, delete all before rendering new ones: `rm *.png`
- Keep renders minimal - prefer viewport playback over many stills
