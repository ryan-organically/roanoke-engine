// Pond and Lake Water Shader
// Renders calm water surfaces for inland water bodies (ponds, lakes, wetlands, marsh pools)

// ============================================================================
// UNIFORMS
// ============================================================================

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_position: vec3<f32>,
    time: f32,
}

struct MaterialUniform {
    deep_color: vec4<f32>,
    shallow_color: vec4<f32>,
    foam_color: vec4<f32>,
    wave_amplitude: f32,
    wave_frequency: f32,
    turbidity: f32,
    transparency_depth: f32,
}

struct InstanceUniform {
    center: vec2<f32>,
    radius: f32,
    water_level: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> material: MaterialUniform;
@group(1) @binding(1) var<uniform> instance: InstanceUniform;

// ============================================================================
// VERTEX SHADER
// ============================================================================

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) edge_distance: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate distance from center for edge effects
    let dist_from_center = length(in.position.xz - instance.center);
    let edge_factor = dist_from_center / instance.radius;

    // Apply gentle wave displacement
    var displaced_pos = in.position;

    // Multiple overlapping sine waves for organic ripple pattern
    let wave1 = sin(in.position.x * material.wave_frequency * 2.0 + camera.time * 0.8)
              * cos(in.position.z * material.wave_frequency * 1.5 + camera.time * 0.6);
    let wave2 = sin(in.position.x * material.wave_frequency * 3.5 + camera.time * 1.2)
              * cos(in.position.z * material.wave_frequency * 2.8 + camera.time * 0.9);
    let wave3 = sin((in.position.x + in.position.z) * material.wave_frequency + camera.time * 0.5);

    // Combine waves with decreasing amplitude
    let combined_wave = wave1 * 0.5 + wave2 * 0.3 + wave3 * 0.2;

    // Reduce wave amplitude near edges (calmer at shoreline)
    let edge_damping = 1.0 - smoothstep(0.7, 1.0, edge_factor);
    displaced_pos.y += combined_wave * material.wave_amplitude * edge_damping;

    out.world_position = displaced_pos;
    out.clip_position = camera.view_proj * vec4<f32>(displaced_pos, 1.0);
    out.uv = in.uv;
    out.edge_distance = edge_factor;

    return out;
}

// ============================================================================
// FRAGMENT SHADER
// ============================================================================

// Simple hash for noise
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Value noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(hash21(i + vec2<f32>(0.0, 0.0)), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// Fractal brownian motion for organic patterns
fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var freq = 1.0;
    var pos = p;

    for (var i = 0; i < octaves; i++) {
        value += amplitude * noise(pos * freq);
        amplitude *= 0.5;
        freq *= 2.0;
    }

    return value;
}

// Caustic pattern for underwater light
fn caustic_pattern(p: vec2<f32>, time: f32) -> f32 {
    let scale = 0.3;
    let p1 = p * scale + vec2<f32>(time * 0.05, time * 0.03);
    let p2 = p * scale * 1.3 + vec2<f32>(-time * 0.04, time * 0.06);

    let n1 = noise(p1 * 8.0);
    let n2 = noise(p2 * 8.0);

    // Create bright caustic lines where noise gradients align
    let caustic_base = abs(sin(n1 * 6.28) * sin(n2 * 6.28));
    let caustic = caustic_base * caustic_base;

    return caustic;
}

// Ripple rings from disturbances (insects, fish, etc)
fn ripple_ring(p: vec2<f32>, center: vec2<f32>, time: f32, birth_time: f32) -> f32 {
    let age = time - birth_time;
    if (age < 0.0 || age > 5.0) {
        return 0.0;
    }

    let dist = length(p - center);
    let ring_radius = age * 3.0; // Expand at 3 units/sec
    let ring_width = 0.3 + age * 0.1;

    // Ring intensity decreases with age
    let intensity = 1.0 - age / 5.0;

    // Ring pattern
    let ring_d = (dist - ring_radius) / ring_width;
    let ring = exp(-ring_d * ring_d);

    return ring * intensity * 0.3;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_position;
    let uv = in.uv;
    let edge = in.edge_distance;

    // Calculate view direction for fresnel
    let view_dir = normalize(camera.camera_position - world_pos);
    let up = vec3<f32>(0.0, 1.0, 0.0);

    // Generate animated normal from noise
    let noise_scale = 2.0;
    let noise_time = camera.time * 0.3;
    let n1 = fbm(world_pos.xz * noise_scale + vec2<f32>(noise_time, 0.0), 3);
    let n2 = fbm(world_pos.xz * noise_scale + vec2<f32>(0.0, noise_time), 3);

    // Perturbed normal
    let normal = normalize(vec3<f32>(
        (n1 - 0.5) * 0.2,
        1.0,
        (n2 - 0.5) * 0.2
    ));

    // Fresnel effect - more reflection at grazing angles
    let f_base = 1.0 - max(dot(view_dir, normal), 0.0);
    let fresnel = f_base * f_base * f_base;

    // Depth simulation based on distance from edge
    // Edges are shallower, center is deeper
    let depth_factor = 1.0 - edge;
    let apparent_depth = depth_factor * material.transparency_depth;

    // Color based on depth
    let water_color = mix(material.shallow_color.rgb, material.deep_color.rgb, depth_factor);

    // Caustics (only visible in shallow areas)
    let caustic = caustic_pattern(world_pos.xz, camera.time);
    let caustic_strength = (1.0 - depth_factor) * 0.15 * (1.0 - material.turbidity);

    // Add ripple rings at pseudo-random locations
    var ripple_sum = 0.0;
    for (var i = 0; i < 4; i++) {
        let seed = f32(i) * 1234.5;
        let ripple_center = instance.center + vec2<f32>(
            sin(seed) * instance.radius * 0.6,
            cos(seed * 1.7) * instance.radius * 0.6
        );
        // Stagger birth times
        let birth = floor((camera.time + seed) / 8.0) * 8.0 - seed;
        ripple_sum += ripple_ring(world_pos.xz, ripple_center, camera.time, birth);
    }

    // Edge foam/scum (organic debris at shoreline)
    let edge_start = 0.85;
    let edge_foam = smoothstep(edge_start, 1.0, edge) * (1.0 - smoothstep(0.95, 1.0, edge));
    let foam_noise = fbm(world_pos.xz * 4.0 + camera.time * 0.1, 2);
    let foam_factor = edge_foam * foam_noise * 0.5;

    // Combine all effects
    var final_color = water_color;

    // Add caustics
    final_color += vec3<f32>(caustic_strength * caustic);

    // Add ripple highlights
    final_color += vec3<f32>(ripple_sum * 0.2);

    // Mix in foam
    final_color = mix(final_color, material.foam_color.rgb, foam_factor);

    // Sky reflection (simplified)
    let sky_color = vec3<f32>(0.6, 0.7, 0.9);
    final_color = mix(final_color, sky_color, fresnel * 0.4);

    // Transparency based on turbidity and depth
    // Crystal clear ponds should be highly transparent
    let clarity = 1.0 - material.turbidity * 0.6;

    // Base alpha from depth - shallow edges are very transparent
    let base_alpha = mix(0.15, 0.75, depth_factor); // 15% at edges, 75% in center

    // Turbid water is more opaque
    let turbid_alpha = mix(base_alpha, 0.9, material.turbidity);

    // Fresnel adds opacity at grazing angles
    let fresnel_alpha = turbid_alpha + fresnel * 0.25 * (1.0 - turbid_alpha);

    // Foam is opaque
    let final_alpha = mix(fresnel_alpha, 0.95, foam_factor);

    // Clamp to reasonable range
    let clamped_alpha = clamp(final_alpha, 0.1, 0.88);

    // Darken edges slightly for natural vignette
    let edge_darken = 1.0 - edge * 0.1;
    final_color *= edge_darken;

    return vec4<f32>(final_color, clamped_alpha);
}
