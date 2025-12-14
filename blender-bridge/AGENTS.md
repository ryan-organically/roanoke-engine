# Agent Breadcrumbs

> This file helps Claude Code (and other agents) understand the project state and where to contribute improvements. Read this first when starting a session.

## Project Status

**Core Bridge:** Stable, functional. Preserve at all costs.
**Pipeline Layer:** Scaffolded, needs refinement with real project data.
**Context Layer:** Templates exist, need project-specific content.

---

## Architecture Overview

```
blender-bridge/
├── blender_bridge_addon.py   # SERVER - Blender addon (TCP socket)
├── blender_client.py         # CLIENT - CLI tool
├── context/                  # Project-specific knowledge for agents
│   └── {project}/           # e.g., roanoke/, webapp/, etc.
├── targets/                  # Export profile definitions
├── assets/                   # Asset manifests and templates
└── AGENTS.md                # This file
```

**Golden Rule:** The bridge (`blender_bridge_addon.py`, `blender_client.py`) should stay dumb. Intelligence lives in context files and orchestration scripts built on top.

---

## Current Capabilities

### Bridge Commands (as of v1.1.0)
- Object CRUD: `list_objects`, `add_primitive`, `delete`, `duplicate`, `rename`
- Transforms: `set_location`, `set_rotation`, `set_scale`, `apply_transforms`
- Selection: `select`, `get_selected`
- Materials: `list_materials` (read-only)
- Custom Properties: `get_custom_props`, `set_custom_prop`, `delete_custom_prop`
- Animation: `list_actions`, `set_active_action`
- Armature: `get_armature_info`
- Export: `export_fbx`, `export_gltf`
- Scene: `get_scene_info`, `screenshot`
- Meta: `ping`, `exec` (arbitrary Python)

### What's Missing (Improvement Opportunities)

See `## Improvement Queue` below.

---

## Improvement Queue

> Agents: Pick items from here when asked to improve the bridge. Mark completed items with [x] and date.

### High Priority

- [x] **Custom properties:** `get_custom_props`, `set_custom_prop`, `delete_custom_prop` - (v1.1.0, 2025-12)
- [x] **Animation basics:** `list_actions`, `set_active_action` - (v1.1.0, 2025-12)
- [x] **Armature inspection:** `get_armature_info` - (v1.1.0, 2025-12)
- [x] **Apply transforms:** `apply_transforms` - (v1.1.0, 2025-12)

### Medium Priority

- [ ] **Material creation:** `create_material`, `assign_material` - currently read-only
- [ ] **UV inspection:** `get_uv_info` - check UV mapping status
- [ ] **Modifier application:** `apply_modifiers` - clean geometry for export
- [ ] **Profile-aware export:** `export_with_profile` - use targets/*.json definitions

### Low Priority (Nice to Have)

- [ ] **LOD generation:** `generate_lods` - decimate copies at specified ratios
- [ ] **Collision export:** `export_collision` - separate collision geometry
- [ ] **Batch operations:** `batch_rename`, `batch_transform` - efficiency for large scenes
- [ ] **Keyframe manipulation:** `add_keyframe`, `get_keyframes` - animation authoring

### Completed

- [x] 2024-01: Initial bridge with 23 commands (v1.0.0)
- [x] 2025-12: Custom properties, animation, armature, apply transforms (v1.1.0) - 7 new commands

---

## Context Files

Context files teach agents about specific projects. They live in `context/{project}/`.

### Required Files per Project

| File | Purpose |
|------|---------|
| `GAME_CONTEXT.md` | World, lore, tone, what belongs/doesn't belong |
| `ART_STYLE.md` | Visual language, colors, mesh budgets, textures |
| `technical/*.md` | Rig standards, animation specs, export settings |
| `behaviors/*.md` | AI/behavior patterns (for games with entities) |

### How Agents Should Use Context

1. **Before creating assets:** Read `GAME_CONTEXT.md` and `ART_STYLE.md`
2. **Before rigging:** Read `technical/rig_standards.md`
3. **Before animating:** Read `technical/animation_specs.md`
4. **Before exporting:** Read `technical/export_checklist.md`
5. **For entity behaviors:** Read `behaviors/*.md`

### Filling In Context

The context files are **templates with placeholder content**. When working on a specific project:

1. Ask the user about their project specifics
2. Update the context files with real information
3. Remove placeholder/example content
4. Add project-specific details the user provides

---

## Target Profiles

Target profiles (`targets/*.json`) define export settings for specific destinations.

### Using Targets

Currently manual - agent reads the JSON, applies settings via bridge commands.

**Future:** `export_with_profile` command that reads target JSON directly.

### Creating New Targets

1. Copy `targets/generic.json` as starting point
2. Modify for your target engine/platform
3. Document any unusual requirements

---

## Asset Manifests

Each exported asset should have a `.asset.json` manifest documenting:
- What it is (name, category, description)
- Technical specs (polycount, textures, animations)
- Project-specific metadata (entity type, behaviors, spawn info)

Template: `assets/.templates/asset_manifest.template.json`

---

## Session Guidelines for Agents

### Starting a Session

1. Read this file (AGENTS.md)
2. Check `## Improvement Queue` for pending work
3. Read relevant context files for the current project
4. Ask user what they want to accomplish

### During Work

- **Preserve the bridge** - don't break existing commands
- **Add incrementally** - one capability at a time
- **Test changes** - verify with `ping` and basic operations
- **Update this file** - mark completed items, add new discoveries

### Ending a Session

1. Note any incomplete work in this file
2. Mark completed improvements with [x] and date
3. Add new improvement ideas discovered during work
4. Leave the bridge in a working state

---

## Connecting to Blender (WSL Detection)

The bridge server runs inside Blender on Windows. When Claude Code runs in WSL, `localhost` won't reach it.

### Quick Detection

```bash
# Test default localhost first
python3 blender_client.py --ping

# If refused, try WSL gateway (Windows host)
python3 blender_client.py --host $(ip route show | grep default | awk '{print $3}') --ping
```

### Environment Detection Pattern

```bash
# Detect if running in WSL
if grep -qi microsoft /proc/version 2>/dev/null; then
    # WSL: use gateway IP to reach Windows
    BLENDER_HOST=$(ip route show | grep default | awk '{print $3}')
else
    # Native Linux/Mac: use localhost
    BLENDER_HOST="localhost"
fi

python3 blender_client.py --host "$BLENDER_HOST" --ping
```

### Common Host Values

| Environment | Host |
|-------------|------|
| Native Windows | `localhost` or `127.0.0.1` |
| Native Linux/Mac | `localhost` |
| WSL2 | Gateway IP (e.g., `172.19.16.1`) |
| WSL2 + mirrored networking | `localhost` may work |

**Tip:** The gateway IP can change on WSL restart. Always detect dynamically.

---

## Common Patterns

### Adding a New Bridge Command

1. In `blender_bridge_addon.py`, add handler function:
   ```python
   def handle_your_command(args):
       # implementation
       return {"your": "data"}
   ```

2. Register in `COMMAND_HANDLERS` dict:
   ```python
   "your_command": handle_your_command,
   ```

3. Test via client:
   ```bash
   python blender_client.py your_command --arg1 value1
   ```

4. Add CLI support in `blender_client.py` if needed

### Reading Project Context

```python
# Agent pattern for loading context
context_path = "context/roanoke/ART_STYLE.md"
# Read and apply constraints from this file
```

---

## Notes & Discoveries

> Agents: Add useful findings here for future sessions.

- WSL users: Bridge binds to 0.0.0.0 for cross-boundary access
- Large responses: Buffer is 65536 bytes, chunking may be needed for huge scenes
- Exec command: Allows arbitrary Python, use for anything not covered by commands

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-01 | 23 commands, basic scene manipulation |
| 1.1.0 | 2025-12 | +7 commands (custom props, animation, armature), context system, target profiles, agent breadcrumbs |

---

_Last updated by agent session: 2025-12-10_
