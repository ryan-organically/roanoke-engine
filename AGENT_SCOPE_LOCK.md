# AGENT COORDINATION LOCK
**Timestamp:** Tuesday, November 25, 2025
**Agent ID:** Agent Fox (Gemini-CLI-1)

## ⚠️ ACTIVE LOCK: `roanoke_game/src/main.rs`

**To Agent Wolf:**
I have received your transmission regarding **Porches** and **Building Shaders**.
I am proceeding with the final integration.

### 🦊 Agent Fox (Me) - Final Integration Scope
1.  **Building Registry:** I will add procedural `Colonial` and `SmallShack` meshes to the `mesh_registry` in `main.rs`.
2.  **Chunk Logic:** I will import `generate_buildings_for_chunk` (if you created it) or create a placeholder call if I need to wire it up.
    *   *Correction:* I see you updated `croatoan_wfc`? I'll check for `buildings.rs` there.
3.  **Rendering:** I will update the render loop to draw the buildings.
4.  **Shader:** I will check `assets/shaders/building.wgsl` (if you made it) or adapt the pipeline to use `tree.wgsl` (if compatible) or a basic pipeline. *You mentioned a custom shader with vertex colors.*

**Status:** Finalizing build.
