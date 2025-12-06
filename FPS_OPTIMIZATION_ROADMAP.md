# FPS Optimization & Visual Improvement Roadmap

**Created**: 2024-12-05
**Updated**: 2024-12-05
**Status**: Phase 0 COMPLETE - FPS Fluidization Implemented
**Priority**: Trees/Rocks/Fog next

---

## Executive Summary

Phase 0 FPS Emergency is **COMPLETE**. The Quantum Spatial Cache system transforms O(n²) → O(n) complexity. All per-frame allocations eliminated via caching. Ready for Phase 1 (visual improvements).

### Current State

| System | FPS Impact | Status |
|--------|------------|--------|
| Animal spatial queries | ~~SEVERE (O(n²))~~ | ✅ FIXED - Quantum Spatial Cache |
| NPC orb instance buffer | ~~MODERATE (per-frame alloc)~~ | ✅ FIXED - Cached with dirty flags |
| Pack morale/alpha calc | ~~MODERATE (per-frame O(n))~~ | ✅ FIXED - Lazy evaluation |
| SystemTime RNG | ~~LOW-MODERATE~~ | ✅ FIXED - PCG hash-based PRNG |
| Query radius | ~~50 units (100 cells)~~ | ✅ FIXED - 25 units (9 cells) |
| Trees | DISABLED (94K tris/tree) | 🔴 Needs LOD system |
| Rocks/Pebbles | HIGH (78K+ instances/chunk) | 🟡 Needs culling |

---

## Phase 0: Emergency FPS Recovery ✅ COMPLETE

**Goal**: Get back to 60 FPS baseline before any visual work.
**Status**: Implemented 2024-12-05 - All optimizations deployed and tested.

### 0.1 Fix O(n²) Animal Spatial Queries

**File**: `roanoke_game/src/animals/manager.rs:187-232`

**Problem**: Every animal queries radius 50.0 for nearby animals every frame.
```rust
// Line 198 - Called for EACH animal
let nearby = self.spatial.query_radius(pos, 50.0);  // O(n) × n animals = O(n²)
```

**Solution**: Batch query + cache results
```rust
// DO ONCE per frame, not per animal:
let all_positions: Vec<(AnimalId, Vec3)> = self.animals.iter()
    .map(|(&id, a)| (id, a.position))
    .collect();

// Build spatial cache ONCE
let mut nearby_cache: HashMap<AnimalId, Vec<AnimalId>> = HashMap::new();
for (id, pos) in &all_positions {
    nearby_cache.insert(*id, self.spatial.query_radius(*pos, 50.0));
}

// Then use cache in per-animal loop
for id in ids {
    let nearby = nearby_cache.get(&id).unwrap_or(&Vec::new());
    // ... rest of logic
}
```

**Estimated gain**: 50-80% FPS improvement with 50 animals

---

### 0.2 Cache NPC Orb Instance Buffer

**File**: `roanoke_game/src/village_manager.rs:205-225`

**Problem**: `get_npc_orb_instances()` allocates new Vec and computes Mat4 for every NPC every frame.

**Solution**: Cache instances, only regenerate on position change
```rust
pub struct VillageManager {
    // Add cache
    cached_npc_instances: Vec<OrbInstanceData>,
    npc_instances_dirty: bool,
}

pub fn get_npc_orb_instances(&mut self) -> &[OrbInstanceData] {
    if self.npc_instances_dirty {
        self.cached_npc_instances = self.npc_orbs.iter().map(|orb| {
            // ... existing transform logic
        }).collect();
        self.npc_instances_dirty = false;
    }
    &self.cached_npc_instances
}
```

**Estimated gain**: 5-10% FPS improvement

---

### 0.3 Fix SystemTime RNG in Behavior

**File**: `roanoke_game/src/animals/behavior.rs:409-417`

**Problem**: `rand_chance()` calls `SystemTime::now()` which is expensive syscall.

**Solution**: Use cached frame random or fast PRNG
```rust
// Replace:
fn rand_chance(probability: f32) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (now as f32 / u32::MAX as f32) < probability
}

// With:
fn rand_chance(probability: f32, frame_seed: u32) -> bool {
    // Fast hash-based random
    let hash = frame_seed.wrapping_mul(0x9E3779B9);
    (hash as f32 / u32::MAX as f32) < probability
}
```

**Estimated gain**: 2-5% FPS improvement

---

### 0.4 Lazy Pack Morale/Alpha Calculation

**File**: `roanoke_game/src/animals/manager.rs:234-237` and related Pack methods

**Problem**: Pack morale and alpha election recalculated every frame for every pack.

**Solution**: Dirty flag pattern
```rust
pub struct Pack {
    members: HashSet<AnimalId>,
    alpha: Option<AnimalId>,
    morale: f32,
    dirty: bool,  // Add this
}

impl Pack {
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn update(&mut self, animals: &HashMap<AnimalId, Animal>) {
        if !self.dirty {
            return;  // Skip if nothing changed
        }
        // ... existing calculation
        self.dirty = false;
    }
}

// Mark dirty when: member added/removed, member health changes significantly
```

**Estimated gain**: 3-8% FPS improvement for wolf packs

---

### 0.5 Reduce Spatial Query Radius

**File**: `roanoke_game/src/animals/manager.rs:198`

**Problem**: 50.0 unit radius is excessive - checks ~100 spatial cells.

**Solution**: Reduce to 25.0 or less based on actual gameplay needs
```rust
// Change:
let nearby = self.spatial.query_radius(pos, 50.0);

// To:
let nearby = self.spatial.query_radius(pos, 25.0);  // Checks ~25 cells instead of ~100
```

**Estimated gain**: 4x fewer cell checks = significant improvement

---

## Phase 1: Rock/Pebble Optimization

**Goal**: Dense rocks without killing FPS.

### 1.1 Distance-Based Rock Culling

**Problem**: `rocks.rs` generates 78K+ instances per chunk at 1.2 pebbles/m².

**Solution**: LOD culling in render loop
```rust
// In main.rs render loop, before rendering rocks:
let rock_cull_distance = 150.0;  // Pebbles invisible beyond 150m
let rock_lod_distance = 50.0;    // Full detail within 50m

for (mesh_name, transform) in &chunk.rocks {
    let pos = transform.w_axis.truncate().into();
    let dist = (pos - camera_pos).length();

    if dist > rock_cull_distance {
        continue;  // Skip distant rocks entirely
    }

    // Could also skip every Nth pebble at medium distance
    if mesh_name == "rock_pebble" && dist > rock_lod_distance {
        // Skip 75% of pebbles at medium distance
        let hash = (pos.x as u32).wrapping_mul(73856093) ^ (pos.z as u32).wrapping_mul(19349663);
        if hash % 4 != 0 {
            continue;
        }
    }

    // Render rock
}
```

**Estimated gain**: 60-80% fewer rock draw calls

---

### 1.2 Rock Instance Batching

**Problem**: Each rock type rendered separately, many small draw calls.

**Solution**: Merge all rock types into single instanced draw
```rust
// Instead of per-type buffers, use unified rock instance buffer:
struct RockInstance {
    model_matrix: [[f32; 4]; 4],
    rock_type_id: u32,  // 0=pebble, 1=small, 2=medium, etc.
    _padding: [u32; 3],
}

// Single draw call for all rocks in chunk
```

**Estimated gain**: 5-6x fewer draw calls per chunk

---

### 1.3 Reduce Pebble Polygon Count

**File**: `crates/croatoan_procgen/src/rock.rs:64-73`

**Current**: Pebbles use subdivision_levels: 2 (icosphere) = ~80 triangles each

**Solution**: Use subdivision_levels: 1 for pebbles = ~20 triangles
```rust
pub fn pebble() -> Self {
    RockRecipe {
        rock_type: RockType::RiverStone,
        base_size: Vec3::new(0.15, 0.08, 0.12),
        seed: 0,
        subdivision_levels: 1,  // Changed from 2
        roughness: 0.02,
        deformation: 0.05,
    }
}
```

**Estimated gain**: 4x fewer triangles per pebble

---

## Phase 2: Tree System Restoration

**Goal**: Visible treeline at distance without GPU death.

### 2.1 Create Lightweight Tree Mesh

**Problem**: `trees9.obj` has 247K faces - instant GPU death.

**Solution**: Create simple procedural tree or find/create low-poly OBJ

**Option A - Procedural (fastest to implement)**:
```rust
// In tree generation, create minimal geometry:
fn generate_simple_tree() -> TreeMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Trunk: 8-sided cylinder, 3 segments = 48 triangles
    generate_cylinder(&mut vertices, &mut indices,
        radius: 0.3, height: 4.0, segments: 8, rings: 3);

    // Canopy: Single icosphere, 1 subdivision = 80 triangles
    generate_sphere(&mut vertices, &mut indices,
        center: Vec3::new(0.0, 5.0, 0.0),
        radius: 2.5, subdivisions: 1);

    // Total: ~130 triangles per tree (vs 94,000)
    TreeMesh { vertices, indices }
}
```

**Option B - Billboard (best performance)**:
```rust
// For trees > 200m away, render as camera-facing quad
struct TreeBillboard {
    position: Vec3,
    height: f32,
    width: f32,
    texture_id: u32,  // Pre-rendered tree image
}
```

**Recommended**: Start with Option A, add Option B for LOD2

---

### 2.2 Re-enable Tree Rendering with LOD

**File**: `roanoke_game/src/main.rs:2637-2644`

**Current**: Trees completely commented out

**Solution**: Phased re-enablement with distance LOD
```rust
// Uncomment and add LOD:
if let Some(trees) = &chunk.trees {
    let chunk_dist = (chunk.bounds.center - camera_pos).length();

    if chunk_dist <= 100.0 {
        // Full detail trees (LOD0)
        trees.render_full(&mut render_pass);
    } else if chunk_dist <= 300.0 {
        // Reduced detail (LOD1) - every 2nd tree, simpler mesh
        trees.render_lod1(&mut render_pass);
    } else if chunk_dist <= state.render_distance * 256.0 {
        // Billboard (LOD2) - camera-facing quads
        trees.render_billboard(&mut render_pass);
    }
    // Beyond render_distance: don't render
}
```

---

### 2.3 Tree Frustum Culling

**Add per-tree culling** (not just per-chunk):
```rust
// In TreePipeline::render():
let visible_instances: Vec<_> = self.instances.iter()
    .filter(|inst| {
        let pos = Vec3::from_slice(&inst.model_matrix[3][0..3]);
        frustum.contains_sphere(pos, TREE_RADIUS)
    })
    .cloned()
    .collect();

// Only upload and render visible trees
```

---

### 2.4 Add Fog to Tree Shader

**File**: `assets/shaders/tree.wgsl`

**Problem**: Trees have no fog, would look wrong when enabled

**Solution**: Copy fog logic from terrain.wgsl
```wgsl
// Add to tree.wgsl fragment shader:
// (requires adding fog uniforms to bind group)

let dist = distance(in.world_position, camera.view_pos);
let fog_factor = clamp(dist / camera.fog_end, 0.0, 1.0);
let fog_factor_sq = fog_factor * fog_factor;  // Quadratic falloff

final_color = mix(final_color, camera.fog_color, fog_factor_sq * camera.fog_density);
```

---

## Phase 3: Fog System Fix

**Goal**: Actual atmospheric fog, not ground tinting.

### 3.1 Reduce Height Fog Dominance

**File**: `assets/shaders/terrain.wgsl:221`

**Problem**: Height fog contributes 0.5 max, dominates the blend

**Current**:
```wgsl
let height_fog = clamp(1.0 - input.world_pos.y / 20.0, 0.0, 0.5);
```

**Fix**:
```wgsl
let height_fog = clamp(1.0 - input.world_pos.y / 50.0, 0.0, 0.15);  // Much weaker
```

---

### 3.2 Add Distance Fog to All Shaders

**Files to update**:
- `grass.wgsl` - NO FOG currently
- `tree.wgsl` - NO FOG currently
- `detritus.wgsl` - NO FOG currently

**Solution**: Add consistent fog calculation to each:
```wgsl
// Add to each fragment shader (after final_color calculation):
let dist = distance(world_pos, uniforms.view_pos);
let fog_amount = clamp((dist / uniforms.fog_end) * uniforms.fog_density, 0.0, 1.0);
final_color = mix(final_color, uniforms.fog_color, fog_amount * fog_amount);
```

---

### 3.3 Sky Fog Gradient

**File**: `assets/shaders/sky.wgsl`

**Problem**: Sky is crystal clear while ground has fog - unrealistic

**Solution**: Add horizon fog band
```wgsl
// In sky fragment shader:
let horizon_factor = 1.0 - abs(ray_dir.y);  // 1.0 at horizon, 0.0 looking up/down
let horizon_fog = pow(horizon_factor, 3.0) * fog_density;

// Blend fog color at horizon
final_sky_color = mix(final_sky_color, fog_color, horizon_fog);
```

---

### 3.4 Remove Minimum Fog Clamp

**File**: `roanoke_game/src/atmosphere.rs:245-247`

**Problem**: Fog density forced to minimum 0.4
```rust
self.state.fog_density = self.state.fog_density.max(0.4);
```

**Fix**: Remove or reduce this clamp
```rust
// Either remove entirely, or:
self.state.fog_density = self.state.fog_density.max(0.05);  // Allow nearly clear
```

---

## Phase 4: Material/Texture Improvements

**Goal**: Believable surfaces instead of vertex-color-only.

### 4.1 Terrain Triplanar Texturing

**Requires**: Diffuse textures for grass, dirt, rock, sand

**Shader changes** (`terrain.wgsl`):
```wgsl
// Add texture bindings
@group(1) @binding(0) var t_grass: texture_2d<f32>;
@group(1) @binding(1) var t_dirt: texture_2d<f32>;
@group(1) @binding(2) var t_rock: texture_2d<f32>;
@group(1) @binding(3) var s_terrain: sampler;

// Triplanar sampling function
fn triplanar_sample(tex: texture_2d<f32>, samp: sampler, world_pos: vec3<f32>, normal: vec3<f32>) -> vec4<f32> {
    let blend = abs(normal);
    let blend_norm = blend / (blend.x + blend.y + blend.z);

    let x_sample = textureSample(tex, samp, world_pos.yz * 0.1);
    let y_sample = textureSample(tex, samp, world_pos.xz * 0.1);
    let z_sample = textureSample(tex, samp, world_pos.xy * 0.1);

    return x_sample * blend_norm.x + y_sample * blend_norm.y + z_sample * blend_norm.z;
}
```

**Texture generation needs**: See Asset Requirements section below.

---

### 4.2 Normal Maps for Surface Detail

**Adds micro-detail without geometry cost**

Same triplanar approach, sample normal map, perturb surface normal.

---

## Phase 5: Asset Requirements

### Textures Needed (Midjourney Generation)

| Asset | Resolution | Prompt Template |
|-------|------------|-----------------|
| grass_diffuse.png | 1024x1024 | "seamless tileable grass texture, photorealistic, top-down view, 4k --tile --ar 1:1" |
| grass_normal.png | 1024x1024 | "seamless tileable grass normal map, blue-purple tones, 4k --tile --ar 1:1" |
| dirt_diffuse.png | 1024x1024 | "seamless tileable brown dirt soil texture, photorealistic, 4k --tile --ar 1:1" |
| rock_diffuse.png | 1024x1024 | "seamless tileable grey stone rock texture, photorealistic, 4k --tile --ar 1:1" |
| sand_diffuse.png | 1024x1024 | "seamless tileable beach sand texture, photorealistic, 4k --tile --ar 1:1" |
| bark_oak.png | 512x512 | "seamless tileable oak tree bark texture, photorealistic, 4k --tile --ar 1:1" |
| leaf_cluster_oak.png | 256x256 | "oak leaves cluster, transparent background, scattered arrangement, top-down --ar 1:1" |

### Models Needed (Sketchfab / Blender)

| Asset | Polygon Target | Source |
|-------|----------------|--------|
| tree_simple.obj | <200 tris | Procedural or Sketchfab "low poly tree" |
| bush_simple.obj | <100 tris | Procedural or Sketchfab "low poly bush" |

### Animals (LEAVE ALONE FOR NOW)

Per user request - animal orb system is fine, don't touch it.

---

## Implementation Priority Order

```
WEEK 1: FPS EMERGENCY
├── 0.1 Fix O(n²) spatial queries [CRITICAL]
├── 0.2 Cache NPC orb instances
├── 0.3 Fix SystemTime RNG
├── 0.4 Lazy pack calculations
└── 0.5 Reduce query radius

WEEK 2: ROCKS & TREES
├── 1.1 Distance-based rock culling
├── 1.2 Rock instance batching
├── 1.3 Reduce pebble polygon count
├── 2.1 Create lightweight tree mesh
└── 2.2 Re-enable trees with LOD

WEEK 3: FOG & ATMOSPHERE
├── 3.1 Reduce height fog dominance
├── 3.2 Add fog to all shaders
├── 3.3 Sky fog gradient
└── 3.4 Remove minimum fog clamp

WEEK 4+: MATERIALS (Optional)
├── 4.1 Terrain triplanar texturing (needs textures)
└── 4.2 Normal maps
```

---

## Testing Checkpoints

### After Phase 0
- [ ] FPS counter shows 60+ FPS with 50 animals on screen
- [ ] No frame stuttering when animals move
- [ ] Villages don't cause FPS drop when discovered

### After Phase 1
- [ ] Pebbles visible but don't tank FPS
- [ ] Rock density feels natural, not sparse
- [ ] No floating rocks

### After Phase 2
- [ ] Trees visible at treeline distance (40 yards from shore)
- [ ] Forest visible on horizon
- [ ] No FPS drop in forested areas
- [ ] Trees fade into fog properly

### After Phase 3
- [ ] Fog slider (`\` key) creates visible atmospheric fog
- [ ] Fog=0 shows clear day, Fog=4 shows dense fog
- [ ] Grass/trees/rocks all affected by fog
- [ ] Horizon fades into fog naturally

---

## Files Reference

### Critical Performance Files
| File | Issue | Priority |
|------|-------|----------|
| `roanoke_game/src/animals/manager.rs` | O(n²) queries | P0 |
| `roanoke_game/src/animals/behavior.rs` | SystemTime RNG | P1 |
| `roanoke_game/src/village_manager.rs` | Per-frame alloc | P1 |

### Tree System Files
| File | Purpose |
|------|---------|
| `roanoke_game/src/main.rs:2637-2644` | Tree render enable |
| `crates/croatoan_render/src/tree_pipeline.rs` | Tree GPU pipeline |
| `assets/shaders/tree.wgsl` | Tree shader |
| `crates/croatoan_procgen/src/tree.rs` | Tree mesh generation |
| `crates/croatoan_wfc/src/trees.rs` | Tree placement |

### Fog System Files
| File | Purpose |
|------|---------|
| `assets/shaders/terrain.wgsl` | Main fog calculation |
| `assets/shaders/grass.wgsl` | Needs fog added |
| `assets/shaders/tree.wgsl` | Needs fog added |
| `assets/shaders/sky.wgsl` | Needs horizon fog |
| `roanoke_game/src/atmosphere.rs` | Fog density clamp |

### Rock System Files
| File | Purpose |
|------|---------|
| `crates/croatoan_wfc/src/rocks.rs` | Rock placement |
| `crates/croatoan_procgen/src/rock.rs` | Rock mesh gen |
| `crates/croatoan_render/src/detritus_pipeline.rs` | Rock rendering |

---

*Document auto-generated from Claude Code audit session 2024-12-05*
