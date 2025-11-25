# Roanoke Engine - Technical Roadmap

Prioritized implementation guide. Each section includes the problem, approach, and specific files to modify.

---

## Completion Status

| Phase | Status | Notes |
|-------|--------|-------|
| 1.1-1.2 Directional Lighting | ✅ DONE | Dynamic sun color based on elevation |
| 1.3 Sun Billboard | ⏳ PENDING | Visual sun disk not implemented |
| 1.4 Time of Day | ✅ DONE | T/Y keys, dynamic sky/fog colors |
| 2.1-2.3 Shadow Fixes | ✅ DONE | Texel snapping, stable projection |
| 3.1-3.3 Frustum Culling | ✅ DONE | ~50% fewer draw calls |
| 4.1-4.3 LOD System | ✅ DONE | Distance culling for grass/trees |
| 5.1-5.3 Chunk Streaming | 🔧 READY | Infrastructure in chunk_manager.rs |

---

## Phase 1: Directional Lighting & Sun Object

**Why first:** Shadows are meaningless without visible directional lighting. Fix the foundation before the effects.

### 1.1 Debug Current Lighting

**Problem:** Sun direction is set but terrain appears flat-lit despite extreme multipliers.

**Files:** `assets/shaders/terrain.wgsl`, `roanoke_game/src/main.rs`

**Steps:**
1. Add debug visualization mode to terrain shader:
   ```wgsl
   // Temporarily output normal as color to verify they're correct
   let debug_normal = (in.world_normal * 0.5 + 0.5);
   return vec4<f32>(debug_normal, 1.0);
   ```
2. If normals look wrong (uniform color), the issue is in `mesh_gen.rs` normal calculation
3. If normals look correct (varied colors based on slope), issue is in lighting math

**Common bugs to check:**
- [ ] `dot(normal, light_dir)` sign might be inverted (try `-light_dir`)
- [ ] Light direction not normalized
- [ ] Uniform buffer not updating (check `queue.write_buffer` call)
- [ ] Fog overriding lighting (disable fog temporarily)

### 1.2 Fix Diffuse Calculation

**File:** `assets/shaders/terrain.wgsl`

**Current (likely broken):**
```wgsl
let diffuse = max(dot(normal, -light_dir), 0.0);
```

**Verify this pattern:**
```wgsl
// Light points FROM sun TO surface, so negate for dot product
let n_dot_l = max(dot(normalize(in.world_normal), -normalize(sun_direction)), 0.0);
let diffuse = sun_color * n_dot_l;
let ambient = ambient_color;
let final_color = base_color * (ambient + diffuse);
```

### 1.3 Add Sun Visual Object

**Problem:** No visual reference for where light comes from.

**New file:** `crates/croatoan_render/src/sun_pipeline.rs`

**Approach - Billboard Quad:**
```rust
// Sun is a textured quad that always faces camera
// Position: camera_pos + sun_direction * far_distance
// Scale: constant screen size (divide by distance)

pub struct SunPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,  // Simple quad
    uniform_buffer: wgpu::Buffer, // Position, size, color
}
```

**Shader approach (`sun.wgsl`):**
```wgsl
// Vertex: billboard that faces camera
// Fragment: radial gradient, white center fading to yellow/orange
// Add bloom later (separate pass)
```

**Integration in main.rs:**
- Render sun BEFORE terrain (it's in the sky)
- Or render to skybox pass
- Sun position = `camera.position - sun_direction * 1000.0`

### 1.4 Time of Day System

**File:** `roanoke_game/src/main.rs` (already has `time_of_day: f32`)

**Make it functional:**
```rust
// In update loop
if keys.contains(&KeyCode::KeyT) {
    state.time_of_day = (state.time_of_day + delta * 2.0) % 24.0;
}

// Calculate sun direction from time
fn sun_direction_from_time(hour: f32) -> Vec3 {
    // 6 AM = east (1, 0, 0), 12 PM = up (0, 1, 0), 6 PM = west (-1, 0, 0)
    let angle = (hour - 6.0) * (std::f32::consts::PI / 12.0);
    Vec3::new(-angle.cos(), -angle.sin().abs(), -0.3).normalize()
}
```

**Update all pipelines:**
- Terrain uniform
- Grass uniform
- Tree uniform
- Shadow light matrix

---

## Phase 2: Shadow System Fixes

**Prerequisites:** Phase 1 complete (directional light visible and working)

### 2.1 Diagnose Flickering

**Problem:** Shadows shimmer/flicker during camera movement.

**Common causes:**
1. **Shadow acne** - depth bias too low
2. **Peter panning** - depth bias too high (shadows detach)
3. **Precision issues** - shadow map too small for world scale
4. **Projection swimming** - light matrix changes with camera

**File:** `crates/croatoan_render/src/shadows.rs`

**Debug steps:**
1. Freeze camera, move only with sun - does flickering stop? → Projection issue
2. Freeze sun, move camera - does flickering continue? → Bias or precision issue
3. Double shadow map resolution (4096) - does it improve? → Precision issue

### 2.2 Stable Shadow Projection

**Problem:** Orthographic projection changes as camera moves, causing texel swimming.

**Fix - Snap to texel grid:**
```rust
fn calculate_light_matrix(sun_dir: Vec3, camera_target: Vec3, shadow_map_size: f32) -> Mat4 {
    let light_pos = camera_target - sun_dir * 500.0;
    let light_view = Mat4::look_at_rh(light_pos, camera_target, Vec3::Y);

    // Orthographic projection
    let ortho_size = 800.0;  // Match fog distance
    let light_proj = Mat4::orthographic_rh(-ortho_size, ortho_size, -ortho_size, ortho_size, 1.0, 2000.0);

    let mut light_matrix = light_proj * light_view;

    // Snap to shadow map texel grid to prevent swimming
    let shadow_origin = light_matrix.transform_point3(Vec3::ZERO);
    let texel_size = (ortho_size * 2.0) / shadow_map_size;
    let snapped_x = (shadow_origin.x / texel_size).round() * texel_size;
    let snapped_y = (shadow_origin.y / texel_size).round() * texel_size;

    // Adjust matrix to snap
    light_matrix.w_axis.x += snapped_x - shadow_origin.x;
    light_matrix.w_axis.y += snapped_y - shadow_origin.y;

    light_matrix
}
```

### 2.3 Proper Depth Bias

**File:** `crates/croatoan_render/src/shadows.rs`

**Current settings may be wrong. Try these ranges:**
```rust
// In pipeline depth_stencil state
depth_bias: wgpu::DepthBiasState {
    constant: 2,        // Start low: 1-4
    slope_scale: 2.0,   // Scale with slope: 1.5-3.0
    clamp: 0.0,
}
```

**Alternative - shader-based bias:**
```wgsl
// In fragment shader when sampling shadow
let bias = max(0.005 * (1.0 - dot(normal, light_dir)), 0.001);
let shadow_depth = shadow_coord.z - bias;
let shadow = textureSampleCompare(shadow_map, shadow_sampler, shadow_coord.xy, shadow_depth);
```

### 2.4 Grass Shadow Casting

**Problem:** Grass uses 24-byte vertices, shadow pipeline expects 36-byte (with normals).

**Options:**

**Option A - Separate shadow pipeline for grass:**
```rust
// New pipeline with different vertex layout
pub struct GrassShadowPipeline { ... }
// Vertex: position only (12 bytes), no color/normal needed for depth
```

**Option B - Skip grass shadows entirely:**
- Grass self-shadowing looks bad anyway (too noisy)
- Just have grass RECEIVE shadows, not cast them
- Trees cast shadows, grass doesn't - this is common in games

**Recommendation:** Option B for now. Grass shadow casting is expensive and looks bad without careful tuning.

---

## Phase 3: Frustum Culling

**Why:** Currently rendering all 49 chunks every frame regardless of visibility.

### 3.1 Basic Frustum Extraction

**New file:** `crates/croatoan_render/src/frustum.rs`

```rust
pub struct Frustum {
    planes: [Vec4; 6],  // Left, Right, Bottom, Top, Near, Far
}

impl Frustum {
    pub fn from_view_proj(view_proj: Mat4) -> Self {
        // Extract planes from combined matrix
        let m = view_proj.to_cols_array_2d();

        Frustum {
            planes: [
                // Left:   row3 + row0
                Vec4::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0], m[3][3] + m[3][0]).normalize(),
                // Right:  row3 - row0
                Vec4::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0], m[3][3] - m[3][0]).normalize(),
                // Bottom: row3 + row1
                Vec4::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1], m[3][3] + m[3][1]).normalize(),
                // Top:    row3 - row1
                Vec4::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1], m[3][3] - m[3][1]).normalize(),
                // Near:   row3 + row2
                Vec4::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2], m[3][3] + m[3][2]).normalize(),
                // Far:    row3 - row2
                Vec4::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2], m[3][3] - m[3][2]).normalize(),
            ]
        }
    }

    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            if distance < -radius {
                return false;  // Completely outside this plane
            }
        }
        true
    }
}
```

### 3.2 Chunk Bounding Spheres

**File:** Store bounds with each chunk pipeline

```rust
struct ChunkData {
    terrain_pipeline: TerrainPipeline,
    grass_pipeline: Option<GrassPipeline>,
    tree_pipeline: Option<TreePipeline>,
    bounds_center: Vec3,
    bounds_radius: f32,
}

// When generating chunk at (offset_x, offset_z):
let chunk_center = Vec3::new(
    offset_x as f32 + 128.0,  // Half chunk size
    50.0,                      // Approximate terrain center height
    offset_z as f32 + 128.0,
);
let chunk_radius = 256.0 * 0.707;  // Diagonal of chunk
```

### 3.3 Culled Render Loop

**File:** `roanoke_game/src/main.rs`

```rust
// Before render pass
let frustum = Frustum::from_view_proj(view_proj);

// In render loop
for chunk in &chunks {
    if frustum.contains_sphere(chunk.bounds_center, chunk.bounds_radius) {
        chunk.terrain_pipeline.render(&mut render_pass);
        if let Some(ref grass) = chunk.grass_pipeline {
            grass.render(&mut render_pass);
        }
        // etc.
    }
}
```

**Expected gain:** ~50% fewer draw calls when looking at horizon (half the chunks behind you).

---

## Phase 4: Level of Detail (LOD)

**Why:** Distant chunks don't need full vertex density.

### 4.1 Terrain LOD

**Approach - Variable resolution per chunk:**

```rust
fn terrain_lod_resolution(distance: f32) -> u32 {
    if distance < 256.0 {
        64   // Near: full detail
    } else if distance < 512.0 {
        32   // Mid: half detail
    } else {
        16   // Far: quarter detail
    }
}
```

**Implementation:**
- Store multiple vertex buffers per chunk (or regenerate on LOD change)
- Swap buffer when LOD level changes
- Add hysteresis to prevent popping (don't switch exactly at boundary)

### 4.2 Tree LOD

**Approach - Reduce L-System iterations by distance:**

```rust
fn tree_iterations_for_distance(base_iterations: u32, distance: f32) -> u32 {
    if distance < 200.0 {
        base_iterations      // Full detail
    } else if distance < 400.0 {
        base_iterations - 1  // Reduced
    } else {
        2                    // Minimum (just trunk + primary branches)
    }
}
```

**Better approach - Billboards:**
- Pre-render tree to texture from 8 angles
- Beyond 500 units, render as billboard quad
- Massive performance win for distant forests

### 4.3 Grass LOD

**Options:**
1. Reduce blade density by distance
2. Reduce segments per blade
3. Don't render grass beyond 300 units (it's in the fog anyway)

**Recommendation:** Option 3. Grass beyond fog distance is invisible, don't pay for it.

```rust
// In grass generation
let max_grass_distance = 300.0;  // Less than fog_start (400)
if distance_to_camera > max_grass_distance {
    continue;  // Skip this chunk's grass
}
```

---

## Phase 5: Chunk Streaming

**Why:** Player can only see ~800 units. Why keep chunks 3000+ units away in memory?

### 5.1 Data Structure

```rust
struct ChunkManager {
    loaded_chunks: HashMap<(i32, i32), ChunkData>,
    load_radius: i32,    // Chunks to keep loaded (e.g., 4 = 9x9 grid)
    player_chunk: (i32, i32),
}

impl ChunkManager {
    fn update(&mut self, player_pos: Vec3, generation_sender: &Sender<ChunkRequest>) {
        let new_player_chunk = world_to_chunk(player_pos);

        if new_player_chunk != self.player_chunk {
            self.player_chunk = new_player_chunk;

            // Unload distant chunks
            self.loaded_chunks.retain(|&(cx, cz), _| {
                let dx = (cx - new_player_chunk.0).abs();
                let dz = (cz - new_player_chunk.1).abs();
                dx <= self.load_radius && dz <= self.load_radius
            });

            // Request new chunks
            for z in -self.load_radius..=self.load_radius {
                for x in -self.load_radius..=self.load_radius {
                    let chunk_coord = (new_player_chunk.0 + x, new_player_chunk.1 + z);
                    if !self.loaded_chunks.contains_key(&chunk_coord) {
                        generation_sender.send(ChunkRequest { coord: chunk_coord });
                    }
                }
            }
        }
    }
}
```

### 5.2 Background Generation

Already have background thread infrastructure. Extend it:

```rust
// Generation thread receives ChunkRequest, generates data, sends back ChunkData
// Main thread receives ChunkData, creates GPU resources, adds to loaded_chunks
```

### 5.3 Memory Budget

```
Per chunk:
- Terrain: ~170KB (4225 verts × 40 bytes)
- Grass: ~1.2MB (3000 blades × 400 bytes)
- Trees: ~400KB (20 trees × 20KB)
- Total: ~1.8MB per chunk

Load radius 4 = 81 chunks = ~145MB
Load radius 3 = 49 chunks = ~88MB (current)
```

---

## Time Estimates

| Phase | Tasks | Est. Hours |
|-------|-------|------------|
| 1.1-1.2 | Debug & fix lighting | 2-3 |
| 1.3 | Sun billboard | 1-2 |
| 1.4 | Time of day | 1 |
| 2.1-2.3 | Shadow fixes | 3-5 |
| 2.4 | Grass shadows (skip) | 0 |
| 3.1-3.3 | Frustum culling | 2-3 |
| 4.1-4.3 | LOD system | 4-6 |
| 5.1-5.3 | Chunk streaming | 4-6 |

**Total:** 17-26 hours for all phases

**Recommended order:** 1 → 2 → 3 → 5 → 4

Skip LOD until streaming works. Streaming is more important for gameplay (infinite exploration) than LOD (which is just optimization).

---

## Don't Do (Yet)

- **VOBJ format:** You don't have enough assets to need a custom format
- **Neural crate:** Cool idea, not needed for core gameplay
- **PBR materials:** Stylized/simple shading is fine, PBR is a rabbit hole
- **Water rendering:** Get land right first
- **Multiplayer:** Single-player first, always
