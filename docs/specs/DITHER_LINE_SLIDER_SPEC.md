# Dither Line Slider Spec

## Overview

A player-accessible graphics setting that controls the distance at which objects begin to dither-fade out. This provides a performance optimization lever for players who prefer higher framerates over visual distance, creating a "nearer-sighted experience" that hides draw distance limitations naturally through dithering rather than harsh pop-out.

## Current System Analysis

### Existing Fog System
- **fog_start**: Distance where fog begins (typically 3-12% of render_distance)
- **fog_end**: Distance where fog is fully opaque (tied to render_distance ~0.9x)
- **fog_density**: Intensity modifier (0.0-1.0), varies by time of day and weather
- Fog blends objects smoothly to fog_color, controlled by `AtmosphereEngine`

### Existing Dither System
- **Bayer 4x4 matrix** dithering in `tree.wgsl` (lines 80-112)
- Currently used **only for LOD transitions** (LOD0 fade-out, LOD1 fade-in)
- Spatially stable (no temporal flickering)
- Parameters: `lod_fade_start`, `lod_fade_end`, `lod_fade_mode`

### Current render_distance Setting
- **Default**: 400.0
- **Range**: 150.0 - 600.0
- **Effects**: Scales chunk load radius, fog_end, LOD distances

## Proposed Feature

### Dither Line Slider

A new slider in Graphics Settings that controls **when objects begin dithering out** independent of (but coordinated with) fog.

```
Dither Distance: [========|----] 85%
                 Near         Far
```

### Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `dither_distance_ratio` | f32 | 0.85 | 0.5 - 1.0 | Ratio of render_distance where dithering begins |
| `dither_fade_width` | f32 | 50.0 | 20.0 - 100.0 | Width of dither transition zone (units) |

**Effective distances**:
- `dither_start = render_distance * dither_distance_ratio`
- `dither_end = dither_start + dither_fade_width`

### Preset Levels (for simplified UI)

| Preset | Ratio | Effective @ 400 RD | FPS Impact |
|--------|-------|---------------------|------------|
| **Ultra (Far)** | 1.0 | 400 (no dither cull) | Baseline |
| **High** | 0.85 | 340 | ~5-10% gain |
| **Medium** | 0.70 | 280 | ~15-25% gain |
| **Low (Near)** | 0.55 | 220 | ~30-40% gain |

## Implementation

### 1. State Extension (main.rs)

```rust
struct GameState {
    // Existing
    render_distance: f32,

    // New
    dither_distance_ratio: f32,  // 0.5-1.0, default 0.85
    dither_fade_width: f32,      // 20-100, default 50.0
}
```

### 2. Shader Uniform Extension

Add to `CameraUniform` in all affected shaders:

```wgsl
struct CameraUniform {
    // ... existing fields ...

    // Distance dither culling (separate from LOD dither)
    dither_cull_start: f32,  // Distance where dither culling begins
    dither_cull_end: f32,    // Distance where objects fully culled
}
```

### 3. Fragment Shader Modification

Add distance-based dithering **before** existing LOD dither logic:

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //=========================================================================
    // DISTANCE DITHER CULLING (player performance setting)
    //=========================================================================
    if (camera.dither_cull_end > camera.dither_cull_start) {
        let dist = distance(in.world_position, camera.view_pos);
        if (dist > camera.dither_cull_start) {
            let fade_range = camera.dither_cull_end - camera.dither_cull_start;
            let fade_t = saturate((dist - camera.dither_cull_start) / fade_range);
            let threshold = dither_threshold(in.clip_position.xy);

            // Discard pixels progressively based on distance
            if (fade_t > threshold) {
                discard;
            }
        }
    }

    // ... rest of existing shader (LOD dither, lighting, fog) ...
}
```

### 4. UI Addition (Settings Panel)

```rust
// In settings UI section
ui.label(egui::RichText::new("Dither Distance:").color(egui::Color32::BLACK));
ui.horizontal(|ui| {
    ui.add_space((ui.available_width() - 300.0) / 2.0);
    ui.add(egui::Slider::new(&mut state.dither_distance_ratio, 0.5..=1.0)
        .text("Distance")
        .custom_formatter(|n, _| {
            let effective = n * state.render_distance;
            format!("{:.0}%  ({:.0}m)", n * 100.0, effective)
        }));
});
ui.label(egui::RichText::new("Lower = better FPS, shorter sight")
    .small()
    .color(egui::Color32::GRAY));
```

### 5. Affected Shaders

Apply distance dither culling to:

| Shader | Priority | Notes |
|--------|----------|-------|
| `tree.wgsl` | HIGH | Trees are primary FPS cost |
| `grass.wgsl` | HIGH | Dense grass is expensive |
| `foliage.wgsl` | HIGH | Ferns, understory |
| `detritus.wgsl` | MED | Already disabled for perf |
| `animal_model.wgsl` | LOW | Sparse, important to see |
| `building.wgsl` | LOW | Landmarks, keep visible |
| `terrain.wgsl` | SKIP | Terrain should not dither |
| `rain.wgsl` | SKIP | Weather effect |

## Optimization Opportunities

### 1. Early-Out in Vertex Shader (GPU optimization)

```wgsl
@vertex
fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Calculate world position
    let world_pos = (model_matrix * vec4(input.position, 1.0)).xyz;
    let dist = distance(world_pos, camera.view_pos);

    // If beyond dither_cull_end, clip to degenerate triangle
    if (dist > camera.dither_cull_end * 1.05) {
        // Move all vertices to same point (GPU will cull)
        output.clip_position = vec4(0.0, 0.0, 2.0, 1.0);
        return output;
    }
    // ... normal vertex processing ...
}
```

This skips fragment shader entirely for distant geometry.

### 2. CPU-Side Instance Culling

Before building instance buffers, skip instances beyond `dither_cull_end`:

```rust
// In render loop
let dither_end = state.render_distance * state.dither_distance_ratio
                 + state.dither_fade_width;

// Filter tree instances
let visible_trees: Vec<_> = chunk.trees
    .iter()
    .filter(|t| t.distance_to_camera() < dither_end)
    .collect();
```

### 3. Adaptive Dither Pattern

For higher quality at cost:
- 8x8 Bayer matrix for smoother transitions
- Blue noise dithering (requires texture lookup)

## Interaction with Existing Systems

### Fog Coordination

```
Distance:  0 -------- fog_start -------- dither_start -------- dither_end -- fog_end -- render_dist
                          |                   |                    |            |
                     Fog begins          Dither begins       Fully culled   Fog opaque

Constraint: dither_end <= fog_end (objects must fade before fog edge)
```

The fog will naturally cover any remaining dithered fragments, creating seamless fading.

### LOD System

Dither culling operates **independently** from LOD dithering:
- LOD dither: Smooth transition between detail levels (LOD0 -> LOD1)
- Distance dither: Performance culling at render boundary

Both can be active simultaneously without conflict.

### Weather System

Heavy fog/rain reduces effective visibility - could auto-reduce dither_distance_ratio:

```rust
let weather_reduction = match weather.current_weather {
    WeatherType::Foggy => 0.7,
    WeatherType::Stormy => 0.8,
    _ => 1.0,
};
let effective_dither = dither_distance_ratio * weather_reduction;
```

## Performance Projections

Based on typical forest scene at 1080p:

| Setting | Trees Rendered | Grass Blades | Est. FPS Impact |
|---------|----------------|--------------|-----------------|
| Ultra (100%) | ~2000 | ~500k | Baseline |
| High (85%) | ~1500 | ~380k | +8-12% |
| Medium (70%) | ~1100 | ~280k | +18-25% |
| Low (55%) | ~700 | ~180k | +30-40% |

## UI/UX Considerations

### Visual Feedback

When player adjusts slider, show preview circle on minimap indicating visible range:
```
    /---\
   / . . \   <- dashed line shows dither boundary
  |   P   |  <- player at center
   \ . . /
    \---/
```

### Tooltip

> "Controls how far you can see before objects fade. Lower values improve performance but reduce visible distance. Objects fade naturally into fog - no harsh pop-out."

### Settings Persistence

Save to config file:
```toml
[graphics]
render_distance = 400.0
dither_distance_ratio = 0.85
dither_fade_width = 50.0
```

## Testing Checklist

- [ ] Slider adjusts dither boundary in real-time
- [ ] No harsh pop-in/pop-out at boundaries
- [ ] Dither pattern stable (no flickering)
- [ ] FPS improves at lower settings
- [ ] Fog properly covers dithered fragments
- [ ] Works with all affected shaders
- [ ] Weather modifiers work correctly
- [ ] Settings persist across sessions
- [ ] Works with LOD transitions simultaneously

## Future Enhancements

1. **Per-category dither distances**: Trees vs grass vs rocks
2. **Temporal anti-aliasing integration**: TAA can smooth dither patterns
3. **Dynamic auto-adjustment**: Lower dither distance when FPS drops
4. **VR mode**: Aggressive near-dither for 90fps requirement
