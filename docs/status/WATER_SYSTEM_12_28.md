# Water System Status - 12.28

## Architecture

Two separate water rendering systems:

| System | File | Coverage | Wave Source |
|--------|------|----------|-------------|
| Terrain Water | `terrain.wgsl` | Entire world (y < 0.5) | Vertex shader sine waves |
| Ocean Patch | `water.wgsl` + `water_compute.wgsl` | 512m × 512m at (456, 0) | GPU compute shader |

---

## Compute Shader Math (`water_compute.wgsl`)

Four layered wave functions, summed per-pixel:

### Layer 1: `ocean_jiggle()` - Base Surface
- 8 overlapping sine waves at varying frequencies/angles
- Plus high-frequency ripple overlay
- Creates chaotic "being out at sea" feel
- Active everywhere

### Layer 2: `distant_swell()` - Rolling Hills
- 3 large sine swells: 150m, 100m, 80m wavelengths
- 12s, 10s, 8s periods
- Fades in 30-80m from shore
- Creates sense of ocean scale

### Layer 3: `coastal_wave()` - Shore-Bound Waves
- 4 wave sets with 7/9/11/13 second periods
- Shoaling effect (waves grow taller approaching shore)
- N-S coastal variation creates curved wave fronts:
  ```wgsl
  let coastal_variation = sin(world_z * 0.008) * 0.4 + sin(world_z * 0.003 + 1.5) * 0.3;
  ```
- Breaking zone foam generation
- Active 0-60m from shore

### Layer 4: `beach_swash()` - Thin Water Layer
- 3 swash cycles with rush-up/hold/recede pattern
- N-S variation in reach (3-13m depending on coast position)
- Foam at leading edge
- Active 0-12m from shore

---

## Fresnel Reflection

### Schlick Approximation (Implemented 12.28)

```wgsl
// Water IOR ~1.33, F0 ≈ 0.02
let F0 = 0.02;
let fresnel = F0 + (1.0 - F0) * pow(1.0 - NdotV, 5.0);
```

- **Looking straight down**: ~2% reflection (see through to water color)
- **Grazing angles**: ~98% reflection (mirror-like sky reflection)

Applied to both `terrain.wgsl` and `water.wgsl`.

---

## Known Issues

### Single Ocean Patch
The compute-driven ocean mesh only covers 512m × 512m centered at z=0. Outside this zone (z < -256 or z > 256), only the simpler terrain water renders.

**Fix options:**
1. Increase `patch_size` to 1024+ meters
2. Tile multiple patches along coast
3. Camera-following ocean mesh

### Spiral Wave Fronts
Coastal waves exhibit spiral/curved patterns due to N-S variation in `coastal_wave()`. This is intentional for realism (wave refraction) but may appear artificial at extreme angles.

---

## File Reference

```
assets/shaders/
├── water.wgsl          # Ocean fragment shader (Fresnel, color, foam)
├── water_compute.wgsl  # Wave simulation (4 layers)
├── terrain.wgsl        # Base water + terrain (lines 243-276)
└── pond_water.wgsl     # Inland water bodies

roanoke_game/src/
├── water_system.rs     # Ocean mesh + compute dispatch
└── pond_water_system.rs # Lakes/ponds
```

---

## Uniforms

### Water Compute (`WaterUniforms`)
| Field | Value | Purpose |
|-------|-------|---------|
| amplitude | 0.8 | Global wave height multiplier |
| choppiness | 0.6 | Horizontal displacement |
| size | 512.0 | Patch size in meters |
| shoreline_x | 200.0 | X coordinate of shoreline |

### Water Material
| Field | Value | Purpose |
|-------|-------|---------|
| smoothness | 0.98 | Specular power |
| turbidity | 0.02 | Water clarity (0 = crystal) |
| max_transparency_depth | 15.0 | Depth for full opacity |
