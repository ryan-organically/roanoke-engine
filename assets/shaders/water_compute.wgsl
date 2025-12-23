// Water Compute Shader - Refined Coastal Ocean
// Base jiggly surface + distant swell mounds + N-S coastal variation

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;
const G: f32 = 9.81;
const N: u32 = 256u;

struct WaterUniforms {
    time: f32,
    delta_time: f32,
    wind_direction: vec2<f32>,
    wind_speed: f32,
    amplitude: f32,
    choppiness: f32,
    size: f32,
    world_offset_x: f32,
    world_offset_z: f32,
    shoreline_x: f32,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: WaterUniforms;
@group(0) @binding(1) var h0_texture: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> hkt_texture: array<vec2<f32>>;
@group(0) @binding(3) var butterfly_texture: texture_2d<f32>;
@group(0) @binding(4) var displacement_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(5) var normal_map_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(6) var shore_distance_texture: texture_2d<f32>;
@group(0) @binding(7) var shore_distance_sampler: sampler;

// ============================================================================
// NOISE FUNCTIONS
// ============================================================================

fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i + vec2<f32>(0.0, 0.0)), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// ============================================================================
// WAVE FUNCTIONS
// ============================================================================

// Base ocean surface - the jiggly "being out in deep water" feel
fn ocean_jiggle(pos: vec2<f32>, time: f32) -> vec4<f32> {
    var height = 0.0;
    var dx = 0.0;
    var dz = 0.0;

    // Multiple overlapping sine waves at different scales
    // This creates the chaotic but natural ocean surface
    for (var i = 0u; i < 8u; i++) {
        let fi = f32(i);
        let freq = 0.02 + fi * 0.015;
        let amp = 0.15 / (1.0 + fi * 0.5);
        let speed = 0.5 + fi * 0.2;
        let angle = fi * 0.7 + 0.3;

        let dir = vec2<f32>(cos(angle), sin(angle));
        let phase = dot(pos * freq, dir) + time * speed;

        height += sin(phase) * amp;
        dx += cos(phase) * amp * freq * dir.x;
        dz += cos(phase) * amp * freq * dir.y;
    }

    // Add some higher frequency ripples
    let ripple1 = sin(pos.x * 0.15 + time * 1.5) * sin(pos.y * 0.12 + time * 1.2) * 0.08;
    let ripple2 = sin(pos.x * 0.22 - time * 1.8) * sin(pos.y * 0.18 - time * 1.4) * 0.05;
    height += ripple1 + ripple2;

    return vec4<f32>(dx, height, dz, 0.0);
}

// Large distant swell mounds - big smooth hills of water
fn distant_swell(pos: vec2<f32>, time: f32, shore_dist: f32) -> vec4<f32> {
    // Only visible far from shore (40m+)
    let swell_factor = smoothstep(30.0, 80.0, shore_dist);
    if (swell_factor < 0.01) { return vec4<f32>(0.0); }

    var height = 0.0;
    var dx = 0.0;
    var dz = 0.0;

    // Large, slow swells - like gentle hills rolling across the ocean
    // Swell 1: Primary - very long wavelength
    let wavelength1 = 150.0;
    let period1 = 12.0;
    let phase1 = pos.x / wavelength1 + time / period1;
    let swell1 = sin(phase1 * TAU) * 1.5;
    let swell1_dx = cos(phase1 * TAU) * 1.5 * TAU / wavelength1;

    // Swell 2: Secondary - slightly different angle
    let angle2 = 0.15;
    let dir2 = vec2<f32>(cos(angle2), sin(angle2));
    let wavelength2 = 100.0;
    let period2 = 10.0;
    let phase2 = dot(pos, dir2) / wavelength2 + time / period2;
    let swell2 = sin(phase2 * TAU) * 1.0;
    let swell2_dx = cos(phase2 * TAU) * 1.0 * TAU / wavelength2 * dir2.x;
    let swell2_dz = cos(phase2 * TAU) * 1.0 * TAU / wavelength2 * dir2.y;

    // Swell 3: Cross-swell for complexity
    let angle3 = -0.2;
    let dir3 = vec2<f32>(cos(angle3), sin(angle3));
    let wavelength3 = 80.0;
    let period3 = 8.0;
    let phase3 = dot(pos, dir3) / wavelength3 + time / period3;
    let swell3 = sin(phase3 * TAU) * 0.6;

    height = (swell1 + swell2 + swell3) * swell_factor;
    dx = (swell1_dx + swell2_dx) * swell_factor;
    dz = swell2_dz * swell_factor;

    return vec4<f32>(dx * 0.3, height, dz * 0.3, 0.0);
}

// Coastal waves - these roll toward shore with N-S variation
fn coastal_wave(world_x: f32, world_z: f32, time: f32, shore_dist: f32) -> vec4<f32> {
    // N-S variation: different parts of coast have waves reaching different distances
    // Uses world_z to create stretches where waves come up more or less
    let coastal_variation = sin(world_z * 0.008) * 0.4 + sin(world_z * 0.003 + 1.5) * 0.3;
    let adjusted_shore = shore_dist + coastal_variation * 15.0;  // +/- 10m variation

    // Waves only active in coastal zone (0-50m from shore)
    let coastal_factor = smoothstep(60.0, 15.0, adjusted_shore);
    if (coastal_factor < 0.01) { return vec4<f32>(0.0); }

    var height = 0.0;
    var foam = 0.0;

    // Generate 4 wave sets with different timings
    for (var i = 0u; i < 4u; i++) {
        let fi = f32(i);
        let period = 7.0 + fi * 2.0;  // 7, 9, 11, 13 second periods
        let wavelength = 20.0 + fi * 8.0;  // 20-44m

        // Wave phase - moves toward shore (negative X direction)
        // Use smooth continuous function for proper looping
        let wave_time = time / period + fi * 0.25;
        let spatial_phase = world_x / wavelength;
        let total_phase = spatial_phase + wave_time;

        // Smooth sine wave that loops properly
        let wave_shape = sin(total_phase * TAU);

        // Shoaling: waves grow taller as they approach shore
        let shoal = 1.0 + smoothstep(40.0, 8.0, adjusted_shore) * 1.2;

        // Wave amplitude - larger waves further out, smaller near shore
        let base_amp = 0.4 + fi * 0.15;
        let amp = base_amp * shoal * (1.0 - fi * 0.15);

        // Wave only above water line (positive part of sine)
        let wave_height = max(0.0, wave_shape) * amp;

        // Breaking zone - waves steepen and collapse
        let break_zone = smoothstep(10.0, 3.0, adjusted_shore);
        let breaking = wave_height * (1.0 - break_zone * 0.5);

        height += breaking * coastal_factor;

        // Foam at crests in breaking zone
        if (wave_shape > 0.6 && adjusted_shore < 12.0) {
            foam += (wave_shape - 0.6) * break_zone * 0.5;
        }
    }

    // Add subtle N-S wave curvature (waves aren't perfectly straight)
    let z_curve = sin(world_z * 0.015 + time * 0.3) * 0.15;
    height += z_curve * coastal_factor * 0.3;

    return vec4<f32>(0.0, height, 0.0, clamp(foam, 0.0, 0.8));
}

// Swash - thin layer of water rushing up and back on beach
fn beach_swash(shore_dist: f32, world_z: f32, time: f32) -> vec2<f32> {
    if (shore_dist > 12.0) { return vec2<f32>(0.0); }

    // N-S variation in swash reach
    let z_var = sin(world_z * 0.01 + 0.5) * 3.0 + sin(world_z * 0.004) * 2.0;
    let max_reach = 8.0 + z_var;  // 3-13m reach depending on coast position

    var total_height = 0.0;
    var total_foam = 0.0;

    // Multiple swash waves
    for (var i = 0u; i < 3u; i++) {
        let fi = f32(i);
        let period = 6.0 + fi * 2.5;
        let phase = fract(time / period + fi * 0.33 + world_z * 0.001);

        // Swash motion: rush up (0-0.35), hold (0.35-0.45), recede (0.45-1.0)
        var extent: f32;
        var intensity: f32;

        if (phase < 0.35) {
            let t = phase / 0.35;
            extent = t * t * max_reach;
            intensity = t;
        } else if (phase < 0.45) {
            extent = max_reach;
            intensity = 1.0;
        } else {
            let t = (phase - 0.45) / 0.55;
            extent = max_reach * (1.0 - t * t);
            intensity = 1.0 - t;
        }

        let in_swash = smoothstep(extent + 1.5, extent - 1.5, shore_dist);
        total_height += in_swash * 0.12 * intensity * (1.0 - fi * 0.25);

        // Foam at leading edge
        if (abs(shore_dist - extent) < 2.0) {
            total_foam += in_swash * intensity * 0.4;
        }
    }

    return vec2<f32>(total_height, clamp(total_foam, 0.0, 0.7));
}

@compute @workgroup_size(16, 16)
fn compute_displacement(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    if (x >= N || y >= N) { return; }

    let u = f32(x) / f32(N);
    let v = f32(y) / f32(N);

    // World position
    let local_x = (u - 0.5) * uniforms.size;
    let local_z = (v - 0.5) * uniforms.size;
    let world_x = local_x + uniforms.world_offset_x;
    let world_z = local_z + uniforms.world_offset_z;
    let world_pos = vec2<f32>(world_x, world_z);

    // Distance from shoreline
    let shore_dist = max(0.0, world_x - uniforms.shoreline_x);

    // ========================================================================
    // LAYER 1: Base jiggly ocean surface (everywhere)
    // ========================================================================
    let jiggle = ocean_jiggle(world_pos, uniforms.time);
    var total_height = jiggle.y * 0.8;  // Slightly reduced base jiggle
    var total_dx = jiggle.x * 0.8;
    var total_dz = jiggle.z * 0.8;
    var total_foam = 0.0;

    // ========================================================================
    // LAYER 2: Distant swell mounds (far from shore)
    // ========================================================================
    let swell = distant_swell(world_pos, uniforms.time, shore_dist);
    total_height += swell.y;
    total_dx += swell.x;
    total_dz += swell.z;

    // ========================================================================
    // LAYER 3: Coastal rolling waves (approaching shore)
    // ========================================================================
    let coastal = coastal_wave(world_x, world_z, uniforms.time, shore_dist);
    total_height += coastal.y;
    total_foam += coastal.w;

    // ========================================================================
    // LAYER 4: Beach swash (very near shore)
    // ========================================================================
    let swash = beach_swash(shore_dist, world_z, uniforms.time);
    total_height += swash.x;
    total_foam += swash.y;

    // ========================================================================
    // FINALIZE
    // ========================================================================

    // Apply global amplitude
    total_height *= uniforms.amplitude;
    total_dx *= uniforms.choppiness;
    total_dz *= uniforms.choppiness;

    // Ensure height is always positive (water surface above base plane)
    total_height = max(total_height, 0.05);

    // Store results
    total_foam = clamp(total_foam, 0.0, 0.9);
    textureStore(displacement_texture, vec2<i32>(i32(x), i32(y)),
        vec4<f32>(total_dx, total_height, total_dz, total_foam));

    // Normal calculation
    let normal = normalize(vec3<f32>(-total_dx * 3.0, 1.0, -total_dz * 3.0));
    let packed_shore = clamp(shore_dist / 100.0, 0.0, 1.0);
    textureStore(normal_map_texture, vec2<i32>(i32(x), i32(y)),
        vec4<f32>(normal, packed_shore));
}

@compute @workgroup_size(16, 16)
fn generate_spectrum(@builtin(global_invocation_id) id: vec3<u32>) {}

@compute @workgroup_size(256, 1)
fn ifft_horizontal(@builtin(global_invocation_id) id: vec3<u32>) {}
