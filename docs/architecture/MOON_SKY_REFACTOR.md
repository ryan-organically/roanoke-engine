# Moon & Sky Refactor - Session Notes

## Issues Addressed

### 1. Moon Rendering Wrong Color (Burnt Sienna)
**Root Cause:** `MOON_PIPELINE` in `main.rs` was incorrectly using `SunPipeline` instead of `MoonPipeline`.

**Fix:** Changed the pipeline type:
```rust
// Before (wrong)
static MOON_PIPELINE: OnceLock<Mutex<SunPipeline>> = OnceLock::new();

// After (correct)
static MOON_PIPELINE: OnceLock<Mutex<MoonPipeline>> = OnceLock::new();
```

### 2. White Polygon Artifacts in Night Sky
**Root Cause:** The **stars implementation** in `sky.wgsl` was causing white polygonal/trapezoidal artifacts visible in the night sky.

**Investigation Method:** Systematic component isolation:
1. Disabled entire sky pipeline - artifacts gone
2. Created minimal sky shader (just gradient) - no artifacts
3. Added atmospheric scattering - no artifacts
4. Added stars - CRASH/artifacts appeared
5. Added clouds without stars - works fine

**Current State:** Stars are disabled in `sky.wgsl`. Clouds work correctly with original code.

### 3. Sun/Moon FOV Distortion
**Issue:** Sun and moon appeared stretched/ovular when not centered on screen due to perspective projection.

**Fix:** Changed from world-space billboards to clip-space billboards:
```wgsl
// Project center to clip space first
let center_clip = uniforms.view_proj * vec4<f32>(uniforms.moon_world_pos, 1.0);

// Offset in clip space (after projection)
out.clip_position = vec4<f32>(
    center_clip.x + pos_2d.x * uniforms.moon_size * center_clip.w,
    center_clip.y + pos_2d.y * uniforms.moon_size * center_clip.w,
    center_clip.z,
    center_clip.w
);
```

### 4. Moon Glow/Blur Attempts (Abandoned)
Multiple attempts to add atmospheric glow around the moon caused issues:
- Outer glow layers created visible square billboard edges
- Edge fade functions cut off glow too sharply
- Exponential falloff never reaches zero, leaving visible boundaries

**Final Decision:** Stripped moon to simple bright circle - no glow effects.

## Current Configuration

### Moon (`moon.wgsl` + `moon_pipeline.rs`)
- Simple bright disc (80% of billboard radius)
- Clip-space rendering (no FOV distortion)
- Size: 0.025 clip space units
- Color: dimmer silver (0.75, 0.77, 0.82)

### Sun (`sun.wgsl` + `sun_pipeline.rs`)
- Clip-space rendering (no FOV distortion)
- Size: 0.1 clip space units
- Retains corona glow and horizon shimmer effects

## Files Modified
- `roanoke_game/src/main.rs` - Fixed moon pipeline type
- `assets/shaders/moon.wgsl` - Simplified to bright circle, clip-space rendering
- `assets/shaders/sun.wgsl` - Clip-space rendering
- `assets/shaders/sky.wgsl` - Stars disabled
- `crates/croatoan_render/src/moon_pipeline.rs` - Size adjustment
- `crates/croatoan_render/src/sun_pipeline.rs` - Size adjustment

## Pending Work
- [ ] Fix stars implementation (currently disabled)
- [ ] Fix water sunrise reflection
