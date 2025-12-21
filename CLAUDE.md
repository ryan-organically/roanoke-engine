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

## Active Work Areas

Track what's being worked on to avoid conflicts:

(none currently)
