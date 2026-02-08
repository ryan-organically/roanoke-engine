# Phase 3: Global Camera Uniform Refactor

## Problem

Every pipeline instance (Tree, Terrain, Building, Detritus, Grass, Animal) has its own
uniform buffer containing a **full copy** of camera/fog/lighting data. Per frame:

- ~1,080 TreePipeline instances × 304 bytes = 329 KB written
- ~36 TerrainPipelines × 288 bytes = 10 KB
- ~36 BuildingPipelines × 192 bytes = 7 KB
- ~36 DetritusPipelines × 112 bytes = 4 KB
- ~36 GrassPipelines × 208 bytes = 7 KB
- ~10 AnimalPipelines × 192 bytes = 2 KB

**Total: ~1,200+ `queue.write_buffer()` calls writing ~360 KB/frame** of mostly identical data
(view_proj, sun_dir, fog params are the same for every pipeline).

## Solution

Extract the common "global" fields into a single `GlobalCameraUniform` buffer written
**once per frame**, bound at `@group(0) @binding(0)` for all pipelines. Per-pipeline
specialization goes into a small `@group(1)` or higher bind group.

## Global Uniform Layout (176 bytes)

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalCameraUniform {
    pub view_proj:       [[f32; 4]; 4],  // 64 bytes  (0-63)
    pub light_view_proj: [[f32; 4]; 4],  // 64 bytes  (64-127)
    pub sun_dir:         [f32; 3],       // 12 bytes  (128-139)
    pub time:            f32,            // 4 bytes   (140-143)
    pub view_pos:        [f32; 3],       // 12 bytes  (144-155)
    pub fog_density:     f32,            // 4 bytes   (156-159)
    pub fog_color:       [f32; 3],       // 12 bytes  (160-171)
    pub fog_start:       f32,            // 4 bytes   (172-175)
    pub fog_end:         f32,            // 4 bytes   (176-179)
    pub _pad:            [f32; 3],       // 12 bytes  (180-191) — pad to 192 (multiple of 16)
}
// Total: 192 bytes (16-byte aligned for GPU)
```

Notes:
- `sun_dir` and `light_dir` are the same value (Building/Animal just name it differently)
- `view_pos` and `camera_pos` are the same value
- Detritus doesn't use `light_view_proj` or `time` — shader just ignores them (zero cost)

## Global Bind Group Layout (group 0)

```
@group(0) @binding(0) var<uniform> globals: GlobalCameraUniform;
@group(0) @binding(1) var shadow_map: texture_depth_2d;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;
```

This bind group is **created once** and **set once per render pass**, not per draw call.
Shadow map is included because 5 of 6 pipelines use it. Detritus binds it but ignores it.

## Per-Pipeline Specialization (group 1+)

### Tree (128 bytes)
```wgsl
struct TreeParams {
    alpha_cutoff:    f32,
    use_texture:     f32,
    lod_fade_start:  f32,
    lod_fade_end:    f32,
    lod_fade_mode:   f32,
    wind_enabled:    f32,
    _pad:            vec2<f32>,
    campfire_lights: array<vec4<f32>, 4>,  // 64 bytes
    campfire_count:  u32,
    _pad2:           vec3<f32>,            // align to 16
};
// @group(1) @binding(0) var<uniform> params: TreeParams;
// @group(2) @binding(0) var diffuse_texture;
// @group(2) @binding(1) var diffuse_sampler;
```

### Terrain (96 bytes)
```wgsl
struct TerrainParams {
    flash_pos:       vec3<f32>,
    flash_intensity: f32,
    campfire_lights: array<vec4<f32>, 4>,  // 64 bytes
    campfire_count:  u32,
    _pad:            vec3<f32>,
};
// @group(1) @binding(0) var<uniform> params: TerrainParams;
// @group(2) @binding(0) var grass_texture;
// @group(2) @binding(1) var terrain_sampler;
```

### Building (16 bytes)
```wgsl
struct BuildingParams {
    ambient_dimming:  f32,
    shadow_strength:  f32,
    rain_wetness:     f32,
    _pad:             f32,
};
// @group(1) @binding(0) var<uniform> params: BuildingParams;
```

### Animal (16 bytes)
```wgsl
struct AnimalParams {
    ambient_dimming:  f32,
    shadow_strength:  f32,
    rain_wetness:     f32,
    _pad:             f32,
};
// @group(1) @binding(0) var<uniform> params: AnimalParams;
// @group(2) @binding(0) var animal_texture;
// @group(2) @binding(1) var animal_sampler;
// @group(3) @binding(0) var<storage> joint_matrices;
```

### Grass (0 bytes — no per-pipeline params!)
```
// Grass only uses global fields. No group(1) needed.
// @group(1) @binding(0) var shadow_map;  -- if we don't put shadow in group(0)
```

### Detritus (0 bytes — no per-pipeline params!)
```
// Detritus only uses global fields. No group(1) needed.
```

## Impact

| Metric | Before | After | Savings |
|--------|--------|-------|---------|
| `write_buffer()` calls | ~1,200/frame | ~240/frame | **-80%** |
| Bytes written | ~360 KB/frame | ~50 KB/frame | **-86%** |
| Bind group sets | 1 per draw | 1 per pass + 1 per draw | ~same |
| GPU uniform reads | 1,200 cache misses | 1 cached + 240 small | **significant** |

The biggest wins:
1. **Tree pipelines (1,080 instances)**: Each wrote 304 bytes. Now: 0 writes for global portion,
   only ~128 instances need per-pipeline params (the rest share identical LOD/campfire values).
2. **Grass/Detritus**: Zero per-pipeline uniform writes needed.
3. **GPU cache**: One 192-byte buffer stays in L1 cache for the entire frame.

## Implementation Plan

### Step 1: Create GlobalCameraUniform (Rust side)
- New file: `crates/croatoan_render/src/global_uniform.rs`
- `GlobalCameraUniform` struct (192 bytes, Pod/Zeroable)
- `GlobalUniformManager` struct:
  - Creates the buffer + bind group layout + bind group (with shadow map)
  - `update()` method: single `queue.write_buffer()` per frame
  - `bind_group()` method: returns `&BindGroup` for render pass

### Step 2: Convert Detritus (simplest — 0 per-pipeline params)
- Remove `CameraUniform` from detritus_pipeline.rs
- Remove `camera_buffer` and `camera_bind_group` fields
- Change WGSL: `@group(0) @binding(0)` reads `GlobalCameraUniform`
- `render()` takes `&BindGroup` (global) as parameter instead of self-binding
- **Eliminates 36 `write_buffer()` calls/frame**

### Step 3: Convert Grass (also 0 per-pipeline params)
- Same approach as Detritus
- Remove per-pipeline uniform buffer
- Shader reads from global bind group
- **Eliminates 36 `write_buffer()` calls/frame**

### Step 4: Convert Building (16 bytes per-pipeline)
- Replace 192-byte uniform with 16-byte `BuildingParams`
- Add `@group(1) @binding(0)` for per-pipeline params
- **Reduces 36 writes from 192→16 bytes each**

### Step 5: Convert Animal (16 bytes per-pipeline)
- Same as Building
- Must also shift texture/sampler/joints to group(2)/group(3)
- **Reduces ~10 writes from 192→16 bytes each**

### Step 6: Convert Terrain (96 bytes per-pipeline)
- Replace 288-byte uniform with 96-byte `TerrainParams`
- Campfire lights + muzzle flash stay per-pipeline
- **Reduces 36 writes from 288→96 bytes each**

### Step 7: Convert Tree (128 bytes per-pipeline — biggest complexity)
- Replace 304-byte uniform with 128-byte `TreeParams`
- Many `update_camera_*` variants simplify to just updating the params buffer
- Trees with identical LOD/campfire params could share a params buffer (future opt)
- **Reduces ~1,080 writes from 304→128 bytes each**
- Many TreePipelines (ferns, grass_clumps, beach_grass) need 0 per-pipeline params
  and can skip the write entirely

### Step 8: Wire up in main.rs
- Create `GlobalUniformManager` in game init
- Call `global_uniforms.update(queue, ...)` once at frame start
- Pass `global_uniforms.bind_group()` to each render call
- Remove all per-pipeline `update_camera()` calls that only set global fields

## Risk Assessment

- **WGSL alignment bugs**: vec3 in WGSL has 16-byte alignment. The global struct is
  carefully designed to avoid vec3 padding traps by packing scalars after vec3s.
- **Bind group compatibility**: All pipelines that share group(0) must use the exact
  same bind group layout. Shadow map inclusion in group(0) means Detritus binds an
  unused shadow texture — negligible cost.
- **Animal pipeline complexity**: Uses 4 bind groups (0-3). Shifting everything up by 1
  means group(3) → group(4), but some GPUs only guarantee 4 bind groups. Need to
  verify `maxBindGroups` or merge texture+uniform into one group.

## Alternative: Partial Refactor (Lower Risk)

If the full refactor is too invasive, a **partial approach** gets 80% of the benefit:

1. Create `GlobalCameraUniform` buffer, written once/frame
2. Keep existing per-pipeline uniform structs BUT remove the global fields
3. Each pipeline reads global from `@group(0)` and per-pipeline from `@group(1)`
4. Start with Detritus + Grass (zero per-pipeline params, biggest ROI per effort)
5. Convert other pipelines incrementally

This avoids the big-bang shader rewrite and lets each pipeline migrate independently.

## Prerequisites

- SharedPipelineState must be done for all pipelines (DONE)
- Shadow map must be accessible as a shared resource (DONE — already passed around)
- All shaders must compile with the new group assignments (test each individually)
