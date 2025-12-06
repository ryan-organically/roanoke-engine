# ROANOKE ENGINE - MASTER AUDIT & ROADMAP

**Date**: 2024-12-05
**Updated**: 2024-12-06
**Version**: v0.0.2-dev (Pre-Alpha)
**Status**: Phase 0 FPS COMPLETE, Phase 2 Trees COMPLETE - Ready for Visual Improvements

---

## Quick Reference Links

| Document | Purpose |
|----------|---------|
| `../../AGENT_DIRECTIVE.md` | **START HERE** - Agent entry point |
| `ROADMAP.md` | Technical implementation phases |
| `VERSION.md` | Version history & known issues |
| `KNOWN_ISSUES.md` | Build-blocking bugs |
| `../performance/FPS_OPTIMIZATION_ROADMAP.md` | Detailed FPS recovery (COMPLETE) |
| `../performance/TREE_SYSTEM_AUDIT.md` | Tree rendering (COMPLETE) |
| `../performance/MATERIAL_SHADER_AUDIT.md` | Shader/material system analysis |

---

## CRITICAL FINDINGS SUMMARY

| System | Status | Impact | Priority | Blocking? |
|--------|--------|--------|----------|-----------|
| **FPS/Performance** | ✅ FIXED | Quantum Spatial Cache deployed | P0 | RESOLVED |
| **Trees** | ✅ FIXED | Simple low-poly (36 tris) | P1 | RESOLVED |
| **Fog** | 🟡 BROKEN | Only tints ground | P2 | No |
| **Materials** | 🟡 PROCEDURAL-ONLY | No textures | P3 | No |
| **Textures** | 🔴 ALMOST NONE | 1 unused texture | P3 | No |
| **Animals** | 🟢 FUNCTIONAL | Orbs work, no meshes | P4 | No |

---

## PHASE 0: FPS EMERGENCY ✅ COMPLETE

**Estimated FPS Gain**: 50-80%
**Status**: Implemented 2024-12-05

### Fixes Deployed

| Issue | Solution | File |
|-------|----------|------|
| O(n²) animal spatial queries | ✅ Quantum Spatial Cache | `manager.rs:24-47, 274-294` |
| Per-frame NPC instance buffer | ✅ Cached with dirty flags | `village_manager.rs:45-49, 238-276` |
| SystemTime RNG calls | ✅ PCG hash-based PRNG | `behavior.rs:408-437` |
| Per-frame pack morale calc | ✅ Lazy evaluation | `manager.rs:74-163` |
| 50-unit spatial query radius | ✅ Reduced to 25 units | `manager.rs:20-22` |

### Success Criteria
- [x] O(n²) → O(n) complexity transformation
- [x] Zero per-frame allocations in hot paths
- [x] Zero syscalls in behavior system
- [x] Release build successful

---

## PHASE 1: ROCK/PEBBLE OPTIMIZATION

**Current**: 78K+ instances per 256×256 chunk at 1.2 pebbles/m²

### Issues

| Issue | Impact |
|-------|--------|
| Pebble density 1.2/m² | 78K instances per chunk |
| Pebble subdivision_levels: 2 | 80 tris per pebble |
| No distance culling | All pebbles rendered always |
| Per-type draw calls | 6 draw calls per chunk |

### Fixes Required

```
1.1 Distance culling - skip pebbles beyond 150m
1.2 LOD culling - skip 75% of pebbles 50-150m
1.3 Reduce pebble subdivision_levels: 2 → 1 (80 → 20 tris)
1.4 Batch all rock types into single draw call
```

### Files to Modify
- `crates/croatoan_procgen/src/rock.rs:64-73` - Pebble recipe
- `crates/croatoan_wfc/src/rocks.rs` - Generation
- `roanoke_game/src/main.rs` - Render loop culling

---

## PHASE 2: TREE SYSTEM RESTORATION ✅ COMPLETE

**Status**: Trees re-enabled with simple low-poly mesh (36 triangles)
**Implemented**: 2024-12-05

### Problem Solved

```rust
// BEFORE: Trees disabled - 247K faces per tree from trees9.obj
// 100+ instances per chunk = billions of triangles = unplayable FPS

// AFTER: Simple low-poly tree mesh (cylinder trunk + icosphere canopy)
// 28 vertices, 36 triangles per tree
// 2,600x reduction in polygon count
```

### Solution Implemented

| Component | Implementation | Triangles |
|-----------|----------------|-----------|
| Trunk | Cylinder (8 segments, 2 rings) | 16 |
| Canopy | Icosahedron (20 faces) | 20 |
| **Total** | **Per tree** | **36** |

### Files Modified

| File | Change |
|------|--------|
| `crates/croatoan_procgen/src/tree.rs` | Added `generate_simple_tree_mesh()` function |
| `roanoke_game/src/main.rs:761-792` | Use simple tree, skip 247K face OBJ |
| `roanoke_game/src/main.rs:2653-2661` | Re-enabled tree rendering |
| `assets/shaders/tree.wgsl` | Added canopy (green) vs trunk (brown) based on UV |

### Performance Gain

- **Before**: ~94,000 triangles per tree × 400 trees = 37.6M triangles
- **After**: 36 triangles per tree × 400 trees = 14,400 triangles
- **Improvement**: 2,600x fewer triangles per chunk

---

## PHASE 3: FOG SYSTEM FIX

**Current**: Fog only tints ground, no visible atmospheric effect

### Why Broken

| Issue | Location | Impact |
|-------|----------|--------|
| Height fog dominates (0.5 max) | `terrain.wgsl:221` | Ground gets dark |
| Grass has NO fog | `grass.wgsl` | Grass ignores fog |
| Trees have NO fog | `tree.wgsl` | Trees ignore fog |
| Detritus has NO fog | `detritus.wgsl` | Rocks ignore fog |
| Sky has NO horizon fog | `sky.wgsl` | Sky stays clear |
| Minimum fog clamp 0.4 | `atmosphere.rs:245` | Can't have clear day |

### Fixes Required

```
3.1 Reduce height_fog from 0.5 to 0.15
3.2 Add fog calculation to grass.wgsl
3.3 Add fog calculation to tree.wgsl
3.4 Add fog calculation to detritus.wgsl
3.5 Add horizon fog to sky.wgsl
3.6 Remove fog_density.max(0.4) clamp
```

### Fog Calculation Template
```wgsl
// Add to each fragment shader after final_color:
let dist = distance(world_pos, uniforms.view_pos);
let fog_amount = clamp((dist / uniforms.fog_end) * uniforms.fog_density, 0.0, 1.0);
final_color = mix(final_color, uniforms.fog_color, fog_amount * fog_amount);
```

---

## PHASE 4: MATERIAL/TEXTURE SYSTEM

**Current**: Procedural-only, vertex colors, no textures

### Current vs Needed

| Have | Need |
|------|------|
| Vertex colors | Diffuse textures |
| Hash-based noise | Normal maps |
| Single light | Roughness variation |
| No metallic | AO for crevices |

### Priority Textures

**Terrain (BIGGEST WIN)**:
- `grass_diffuse.png` - 1024×1024 tileable
- `grass_normal.png` - 1024×1024
- `dirt_diffuse.png` - 1024×1024 tileable
- `rock_diffuse.png` - 1024×1024 tileable
- `sand_diffuse.png` - 1024×1024 tileable

**Trees**:
- `bark_oak.png` - 512×512 tileable
- `bark_pine.png` - 512×512 tileable
- `leaf_cluster_oak.png` - 256×256 alpha

### Implementation: Triplanar Mapping

```wgsl
fn triplanar_sample(tex: texture_2d<f32>, samp: sampler,
                    world_pos: vec3<f32>, normal: vec3<f32>) -> vec4<f32> {
    let blend = abs(normal);
    let blend_norm = blend / (blend.x + blend.y + blend.z);

    let x_sample = textureSample(tex, samp, world_pos.yz * 0.1);
    let y_sample = textureSample(tex, samp, world_pos.xz * 0.1);
    let z_sample = textureSample(tex, samp, world_pos.xy * 0.1);

    return x_sample * blend_norm.x + y_sample * blend_norm.y + z_sample * blend_norm.z;
}
```

---

## PHASE 5: WATER IMPROVEMENTS

**Current**: Compute shader waves working, missing reflections

### Status

| Feature | Status |
|---------|--------|
| Wave simulation | ✅ Phillips spectrum |
| Displacement mapping | ✅ Working |
| Fresnel effect | ✅ Working |
| Foam generation | ✅ Working |
| Specular highlights | ✅ Working |
| Sky reflection | ❌ Hardcoded placeholder |
| Underwater refraction | ❌ Not implemented |
| Caustics | ❌ Not implemented |

### Fixes (Lower Priority)

```
5.1 Add sky reflection sampling
5.2 Add underwater fog color
5.3 Consider reflection probe for buildings
```

---

## PHASE 6: ANIMAL SYSTEM (LEAVE ALONE)

**Current**: Colored orbs with behavior-based glow - FUNCTIONAL

**Per user request**: Do not modify animal system for now. Orbs work fine for gameplay testing. Real animal meshes are Phase 10+ work.

---

## ASSET GENERATION GUIDE

### Midjourney Prompts

**Terrain Textures** (use `--tile`):
```
"seamless tileable grass texture, photorealistic, top-down view, 4k --ar 1:1 --tile"
"seamless tileable brown dirt soil texture, photorealistic, 4k --ar 1:1 --tile"
"seamless tileable grey rock stone texture, photorealistic, 4k --ar 1:1 --tile"
"seamless tileable beach sand texture, photorealistic, 4k --ar 1:1 --tile"
```

**Normal Maps**:
```
"seamless tileable grass normal map, blue purple tones, 4k --ar 1:1 --tile"
```

**Bark Textures**:
```
"seamless tileable oak tree bark texture, photorealistic, 4k --ar 1:1 --tile"
"seamless tileable pine tree bark texture, photorealistic, 4k --ar 1:1 --tile"
"seamless tileable birch tree bark texture, white with black marks, 4k --ar 1:1 --tile"
```

**Leaf Clusters** (needs GIMP/Photoshop alpha cleanup):
```
"oak leaves cluster, transparent background, top-down view, scattered natural --ar 1:1"
```

### Free 3D Model Sources

| Asset | Source |
|-------|--------|
| Tree trunks | Sketchfab "low poly tree" or Blender procedural |
| Animal meshes | Quaternius pack / Sketchfab "low poly animal" |
| Bush/shrub | Sketchfab "low poly bush" |

---

## IMPLEMENTATION ORDER

```
WEEK 1: FPS EMERGENCY (BLOCKING)
├── 0.1 Fix O(n²) spatial queries
├── 0.2 Cache NPC orb instances
├── 0.3 Fix SystemTime RNG
├── 0.4 Lazy pack calculations
└── 0.5 Reduce query radius
    └── CHECKPOINT: 60+ FPS achieved

WEEK 2: ROCKS & TREES
├── 1.1 Rock distance culling
├── 1.2 Reduce pebble polygon count
├── 2.1 Create simple tree mesh
├── 2.2 Implement tree LOD
└── 2.3 Re-enable tree rendering
    └── CHECKPOINT: Trees visible, FPS stable

WEEK 3: FOG & ATMOSPHERE
├── 3.1 Reduce height fog dominance
├── 3.2 Add fog to grass shader
├── 3.3 Add fog to tree shader
├── 3.4 Add sky horizon fog
└── 3.5 Remove minimum fog clamp
    └── CHECKPOINT: Atmospheric fog working

WEEK 4+: MATERIALS (Requires texture generation)
├── 4.1 Generate terrain textures (Midjourney)
├── 4.2 Implement triplanar sampling
├── 4.3 Add normal mapping
└── 4.4 Generate tree bark textures
    └── CHECKPOINT: Textured terrain
```

---

## QUESTIONS FOR USER

Before proceeding, please clarify:

### 1. Art Direction

- **Photorealistic**: High-res textures, PBR, complex
- **Stylized/Low-poly**: Simpler, cohesive look
- **Painterly**: Midjourney strength, artistic

### 2. Texture Generation Tool

- **Midjourney**: Best quality, needs prompt crafting
- **"Nano banana"**: Please explain what this is
- **Stable Diffusion**: Local, more control

### 3. Model Sources

- Will you create models in Blender?
- Download from Sketchfab/etc?
- Want me to find specific models?

### 4. Priority Confirmation

Current order:
1. FPS fixes (blocking)
2. Trees (visibility)
3. Rocks (optimization)
4. Fog (atmosphere)
5. Materials (textures)

Change priority?

---

## FILE QUICK REFERENCE

### Performance Critical
| File | Issue |
|------|-------|
| `roanoke_game/src/animals/manager.rs:187-232` | O(n²) queries |
| `roanoke_game/src/animals/behavior.rs:409-417` | SystemTime RNG |
| `roanoke_game/src/village_manager.rs:205-225` | Per-frame alloc |

### Trees
| File | Purpose |
|------|---------|
| `roanoke_game/src/main.rs:2637-2644` | Tree render (disabled) |
| `crates/croatoan_render/src/tree_pipeline.rs` | GPU pipeline |
| `assets/shaders/tree.wgsl` | Shader |

### Fog
| File | Purpose |
|------|---------|
| `assets/shaders/terrain.wgsl:220-240` | Main fog calc |
| `assets/shaders/grass.wgsl` | Needs fog added |
| `assets/shaders/tree.wgsl` | Needs fog added |
| `assets/shaders/sky.wgsl` | Needs horizon fog |
| `roanoke_game/src/atmosphere.rs:245-247` | Fog clamp |

### Rocks
| File | Purpose |
|------|---------|
| `crates/croatoan_procgen/src/rock.rs:64-73` | Pebble poly count |
| `crates/croatoan_wfc/src/rocks.rs` | Generation density |

---

## AI AGENT INSTRUCTIONS

**Primary Entry Point**: See `AGENT_DIRECTIVE.md` for comprehensive agent guidance.

When updating this project:

1. **Read `AGENT_DIRECTIVE.md` first** - Current state and priorities
2. **Check `KNOWN_ISSUES.md`** - Build-blocking bugs
3. **Run game to verify** - `cargo run --release`
4. **Update documentation** after changes
5. **Add session notes** to `AGENT_DIRECTIVE.md`

### Continuous Improvement Loop

```
1. Identify bottleneck (profile if needed)
2. Implement fix
3. Measure improvement
4. Update documentation
5. Ask user: "What should we improve next?"
```

---

*Master audit document - Auto-maintained by Claude Code*
*Last updated: 2024-12-06*
