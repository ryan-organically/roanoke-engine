# Material & Shader System Audit

**Date**: 2024-12-05
**Status**: Procedural-Only System - No Textures
**Priority**: HIGH - Visual believability limited

---

## Executive Summary

The engine uses a **procedural-first, vertex-color-only** rendering approach. There are no PBR materials, no texture atlases, and only 1 texture file in the entire project (unused). This creates a distinctive look but limits visual fidelity.

---

## Current Architecture

### Shader Inventory (13 WGSL files)

| Shader | Purpose | Textures Used | Fog Support |
|--------|---------|---------------|-------------|
| `terrain.wgsl` | Main terrain + water | Shadow depth only | ✅ Yes |
| `grass.wgsl` | Grass blades with wind | Shadow depth only | ❌ NO |
| `tree.wgsl` | Tree trunks (disabled) | Default white | ❌ NO |
| `building.wgsl` | Structures | None (vertex colors) | ✅ Yes |
| `detritus.wgsl` | Rocks, logs | None (procedural) | ❌ NO |
| `water.wgsl` | Ocean surface | Displacement/normal (computed) | ❌ NO |
| `water_compute.wgsl` | Wave simulation | Phillips spectrum | N/A |
| `sky.wgsl` | Procedural sky | None (FBM clouds) | ❌ Needs horizon fog |
| `sun.wgsl` | Sun billboard | None (procedural glow) | N/A |
| `light_shafts.wgsl` | God rays post-process | Scene color input | N/A |
| `animal_orb.wgsl` | Animal spheres | None (instance colors) | ❌ NO |
| `viewmodel.wgsl` | First-person arms | None (vertex colors) | N/A |
| `coastline.wgsl` | Beach transitions | Height texture | ❌ NO |

### Material Properties per Pipeline

| Pipeline | Diffuse | Normal Map | Roughness | Metallic | AO |
|----------|---------|------------|-----------|----------|----|
| Terrain | Vertex color | ❌ | ❌ | ❌ | ❌ |
| Grass | Vertex color | ❌ | ❌ | ❌ | ❌ |
| Tree | Procedural noise | ❌ | ❌ | ❌ | ❌ |
| Building | Vertex color | ❌ | ❌ | ❌ | ❌ |
| Detritus | Procedural noise | ❌ | ❌ | ❌ | ❌ |
| Water | Hardcoded colors | ✅ (computed) | Defined but unused | Defined but unused | ❌ |
| Animal Orb | Instance color | ❌ | ❌ | ❌ | ❌ |

---

## Lighting Model

### Day/Night Cycle (All shaders)

```wgsl
let sun_elevation = -light_dir.y;
let day_factor = smoothstep(-0.1, 0.3, sun_elevation);

// Night
let moon_color = vec3<f32>(0.15, 0.18, 0.25);
let night_ambient = vec3<f32>(0.02, 0.03, 0.05);

// Day
let day_sun_color = mix(
    vec3<f32>(1.8, 0.6, 0.2),  // Sunrise/sunset: warm orange
    vec3<f32>(1.4, 1.3, 1.1),  // Midday: bright white-yellow
    clamp(sun_elevation * 2.0, 0.0, 1.0)
);
let day_ambient = mix(
    vec3<f32>(0.15, 0.10, 0.08), // Sunrise: warm
    vec3<f32>(0.12, 0.14, 0.18), // Midday: cool sky
    clamp(sun_elevation * 2.0, 0.0, 1.0)
);
```

### Shadow System

- **Depth-only shadow pass** via `ShadowPipeline`
- **Comparison sampler** for hardware PCF
- Applied to: terrain, grass, buildings (NOT trees, water, detritus, animals)
- Shadow bias: Hardware `DepthBiasState`, no shader-side bias

---

## Texture Assets Status

### Current Textures (Almost None)

| File | Size | Usage |
|------|------|-------|
| `assets/oak-compressed.jpg` | 41KB | UNUSED (tree pipeline uses default white) |
| `assets/ui/*.png/jpg` | Various | UI only, not 3D rendering |
| Shadow depth texture | GPU-generated | Runtime only |
| Water displacement/normal | GPU-computed | Runtime only |

### Missing Textures (Needed for Visual Quality)

**Priority 1 - Terrain (Biggest Visual Win)**:
- `grass_diffuse.png` (1024×1024, tileable)
- `grass_normal.png` (1024×1024)
- `dirt_diffuse.png` (1024×1024, tileable)
- `dirt_normal.png` (1024×1024)
- `rock_diffuse.png` (1024×1024, tileable)
- `rock_normal.png` (1024×1024)
- `sand_diffuse.png` (1024×1024, tileable)
- `sand_normal.png` (1024×1024)

**Priority 2 - Trees**:
- `bark_oak.png` (512×512, tileable)
- `bark_pine.png` (512×512, tileable)
- `bark_birch.png` (512×512, tileable)
- `leaf_cluster_oak.png` (256×256, alpha cutout)
- `leaf_cluster_pine.png` (256×256, alpha cutout)

**Priority 3 - Vegetation**:
- `bush_fern.png` (256×256, alpha cutout)
- `bush_shrub.png` (256×256, alpha cutout)

---

## Procedural Generation (Current Strengths)

### Tree Bark (tree.wgsl)
```wgsl
let noise = fract(sin(dot(in.world_position.xz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
let noise2 = fract(sin(dot(in.world_position.xy * 0.5, vec2<f32>(39.346, 11.135))) * 43758.5453);

let bark_dark = vec3<f32>(0.25, 0.15, 0.08);
let bark_light = vec3<f32>(0.45, 0.30, 0.18);
let bark_color = mix(bark_dark, bark_light, noise * 0.6 + noise2 * 0.4);
```

### Detritus Rocks/Logs (detritus.wgsl)
```wgsl
let pos_hash = fract(sin(dot(input.world_position.xz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
let log_color = vec3<f32>(0.28, 0.18, 0.10);   // Dark brown bark
let rock_color = vec3<f32>(0.45, 0.42, 0.38);  // Grey stone
let base_color = mix(log_color, rock_color, step(0.6, pos_hash));

// Moss on upward-facing surfaces
let moss_factor = max(0.0, input.world_normal.y - 0.5) * 0.3;
let moss_color = vec3<f32>(0.15, 0.25, 0.08);
```

### Sky Clouds (sky.wgsl)
- 5-octave FBM (Fractal Brownian Motion)
- Dynamic sun color based on elevation
- Star field at night

### Water Waves (water_compute.wgsl)
- Phillips spectrum ocean simulation
- Sum-of-sines approximation (16 waves)
- Outputs: displacement XYZ + normal + jacobian (for foam)

---

## Improvement Roadmap

### Phase 1: Add Fog Consistency (Easy)

Add fog calculation to shaders that lack it:

```wgsl
// Template to add to grass.wgsl, tree.wgsl, detritus.wgsl:
let dist = distance(world_pos, uniforms.view_pos);
let fog_amount = clamp((dist / uniforms.fog_end) * uniforms.fog_density, 0.0, 1.0);
final_color = mix(final_color, uniforms.fog_color, fog_amount * fog_amount);
```

### Phase 2: Triplanar Terrain Texturing (Medium)

```wgsl
// Add to terrain.wgsl
@group(1) @binding(0) var t_grass: texture_2d<f32>;
@group(1) @binding(1) var t_dirt: texture_2d<f32>;
@group(1) @binding(2) var t_rock: texture_2d<f32>;
@group(1) @binding(3) var s_terrain: sampler;

fn triplanar_sample(tex: texture_2d<f32>, samp: sampler, world_pos: vec3<f32>, normal: vec3<f32>) -> vec4<f32> {
    let blend = abs(normal);
    let blend_norm = blend / (blend.x + blend.y + blend.z);

    let x_sample = textureSample(tex, samp, world_pos.yz * 0.1);
    let y_sample = textureSample(tex, samp, world_pos.xz * 0.1);
    let z_sample = textureSample(tex, samp, world_pos.xy * 0.1);

    return x_sample * blend_norm.x + y_sample * blend_norm.y + z_sample * blend_norm.z;
}

// In fragment shader:
let grass_tex = triplanar_sample(t_grass, s_terrain, world_pos, normal);
let dirt_tex = triplanar_sample(t_dirt, s_terrain, world_pos, normal);
let rock_tex = triplanar_sample(t_rock, s_terrain, world_pos, normal);

// Blend based on slope and height
let slope = 1.0 - normal.y;
let height_blend = smoothstep(5.0, 15.0, world_pos.y);
let base_color = mix(
    mix(grass_tex.rgb, dirt_tex.rgb, slope),
    rock_tex.rgb,
    smoothstep(0.3, 0.6, slope)
);
```

### Phase 3: Normal Maps (Medium-Hard)

Perturb surface normals based on normal map samples for micro-detail.

### Phase 4: PBR Materials (Hard)

Requires:
- Roughness/metallic maps per material
- Environment map for reflections
- Significant shader rewrites

**Recommendation**: Skip PBR for now. Focus on textures + normal maps first.

---

## Water System Status

### What Works
- ✅ Compute shader wave simulation (Phillips spectrum)
- ✅ Displacement mapping
- ✅ Fresnel effect (Schlick approximation)
- ✅ Foam generation based on wave peaks
- ✅ Blinn-Phong specular

### What's Missing
- ❌ Sky reflection (uses hardcoded placeholder)
- ❌ Underwater refraction
- ❌ Caustics
- ❌ Underwater fog color

### Water Material Struct (Partially Implemented)
```wgsl
struct WaterMaterial {
    deep_color: vec4<f32>,
    shallow_color: vec4<f32>,
    foam_color: vec4<f32>,
    smoothness: f32,    // Used for specular power
    metallic: f32,      // DEFINED BUT NOT USED
}
```

---

## Texture Generation Recommendations

### Midjourney Prompts

**Terrain Textures** (use `--tile` for seamless):
```
"seamless tileable grass texture, photorealistic, top-down view, 4k resolution, no visible repeating pattern --ar 1:1 --tile"

"seamless tileable brown dirt soil texture, photorealistic, 4k --ar 1:1 --tile"

"seamless tileable grey rock stone texture, photorealistic, 4k --ar 1:1 --tile"

"seamless tileable beach sand texture, photorealistic, 4k --ar 1:1 --tile"
```

**Normal Maps** (blue-purple color scheme):
```
"seamless tileable grass normal map, blue and purple tones, 4k --ar 1:1 --tile"
```

**Bark Textures**:
```
"seamless tileable oak tree bark texture, photorealistic, 4k --ar 1:1 --tile"

"seamless tileable pine tree bark texture, photorealistic, 4k --ar 1:1 --tile"

"seamless tileable birch tree bark texture, white with black marks, 4k --ar 1:1 --tile"
```

**Leaf Clusters** (needs alpha cleanup in GIMP/Photoshop):
```
"oak leaves cluster arrangement, transparent background, top-down view, scattered natural pattern --ar 1:1"
```

---

## Files Reference

### Shader Files
```
assets/shaders/
├── terrain.wgsl       # Main terrain + inline water
├── grass.wgsl         # Grass blades + wind
├── tree.wgsl          # Tree bark (procedural)
├── building.wgsl      # Buildings (vertex color)
├── detritus.wgsl      # Rocks/logs (procedural)
├── water.wgsl         # Ocean render
├── water_compute.wgsl # Wave simulation
├── sky.wgsl           # Procedural sky
├── sun.wgsl           # Sun billboard
├── light_shafts.wgsl  # God rays
├── animal_orb.wgsl    # Animal spheres
├── viewmodel.wgsl     # First-person arms
└── coastline.wgsl     # Beach biome blend
```

### Pipeline Files
```
crates/croatoan_render/src/
├── terrain_pipeline.rs
├── grass_pipeline.rs
├── tree_pipeline.rs
├── building_pipeline.rs
├── detritus_pipeline.rs
├── sky_pipeline.rs
├── sun_pipeline.rs
├── animal_orb_pipeline.rs
├── viewmodel_pipeline.rs
├── light_shaft_pipeline.rs
├── shadows.rs
├── camera.rs
├── frustum.rs
└── lib.rs
```

---

*Document generated from Claude Code audit session 2024-12-05*
