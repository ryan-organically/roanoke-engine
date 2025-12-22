// Ember Particle Shader
// Renders rising ember particles from campfires as camera-facing billboards

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
    camera_right: vec3<f32>,
    _pad1: f32,
    camera_up: vec3<f32>,
    _pad2: f32,
    // Campfire positions (xyz = position, w = intensity)
    campfire_data: array<vec4<f32>, 8>,
    campfire_count: u32,
    _pad3: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
    @location(2) color: vec3<f32>,
}

// Hash functions for pseudo-random particle properties
fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 = p3 + dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash33(p: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // Determine which campfire this ember belongs to
    let embers_per_fire = 200u;
    let campfire_idx = instance_index / embers_per_fire;
    let ember_idx = instance_index % embers_per_fire;

    // Skip if no campfire at this index
    if (campfire_idx >= uniforms.campfire_count) {
        out.clip_position = vec4<f32>(0.0, 0.0, -2.0, 1.0); // Behind camera
        out.alpha = 0.0;
        out.uv = vec2<f32>(0.0, 0.0);
        out.color = vec3<f32>(0.0, 0.0, 0.0);
        return out;
    }

    let campfire = uniforms.campfire_data[campfire_idx];
    let campfire_pos = campfire.xyz;
    let intensity = campfire.w;

    // Skip if campfire is off
    if (intensity <= 0.0) {
        out.clip_position = vec4<f32>(0.0, 0.0, -2.0, 1.0);
        out.alpha = 0.0;
        out.uv = vec2<f32>(0.0, 0.0);
        out.color = vec3<f32>(0.0, 0.0, 0.0);
        return out;
    }

    // Pseudo-random ember properties based on ember index
    let seed = vec3<f32>(f32(ember_idx), f32(ember_idx) * 1.7, f32(ember_idx) * 0.9 + f32(campfire_idx) * 100.0);
    let rand = hash33(seed);

    // Life cycle parameters
    let life_duration = 2.0 + rand.z * 2.0;  // 2-4 seconds per ember
    let life_offset = hash31(seed) * life_duration;
    let life_progress = fract((uniforms.time + life_offset) / life_duration);

    // Spawn position - small radius from center of campfire
    let spawn_radius = 0.25 * rand.x;
    let spawn_angle = rand.y * 6.28318;

    // Rise height over lifetime (0-2 meters)
    let rise_height = life_progress * 2.0;

    // Drift as ember rises (slight swaying motion)
    let drift_x = sin(uniforms.time * 2.0 + rand.x * 6.28) * 0.15 * life_progress;
    let drift_z = cos(uniforms.time * 1.5 + rand.y * 6.28) * 0.15 * life_progress;

    // Particle world position
    let particle_pos = campfire_pos + vec3<f32>(
        spawn_radius * cos(spawn_angle) + drift_x,
        rise_height + 0.1,  // Start just above ember bed
        spawn_radius * sin(spawn_angle) + drift_z,
    );

    // Ember size - shrinks as it rises and cools
    let base_size = 0.015 + rand.x * 0.01;  // 0.015-0.025m base size
    let size_decay = 1.0 - life_progress * 0.6;  // Shrink to 40% at end
    let ember_size = base_size * size_decay * intensity;

    // Billboard vertices (2 triangles = 6 vertices per ember)
    var local_offset: vec2<f32>;
    switch vertex_index % 6u {
        case 0u: { local_offset = vec2<f32>(-1.0, 1.0); }
        case 1u: { local_offset = vec2<f32>(1.0, 1.0); }
        case 2u: { local_offset = vec2<f32>(-1.0, -1.0); }
        case 3u: { local_offset = vec2<f32>(1.0, 1.0); }
        case 4u: { local_offset = vec2<f32>(1.0, -1.0); }
        default: { local_offset = vec2<f32>(-1.0, -1.0); }
    }

    // Create billboard facing camera
    let right = normalize(uniforms.camera_right);
    let up = normalize(uniforms.camera_up);
    let world_pos = particle_pos
        + right * local_offset.x * ember_size
        + up * local_offset.y * ember_size;

    // UV for fragment shader (center at 0.5, 0.5)
    out.uv = (local_offset + 1.0) * 0.5;

    // Alpha based on life progress (fade in quickly, fade out slowly)
    let fade_in = smoothstep(0.0, 0.1, life_progress);
    let fade_out = 1.0 - smoothstep(0.6, 1.0, life_progress);
    out.alpha = fade_in * fade_out * intensity;

    // Color gradient: orange core -> yellow as it rises
    let core_color = vec3<f32>(1.0, 0.4, 0.1);   // Deep orange
    let cool_color = vec3<f32>(1.0, 0.7, 0.3);   // Yellow-orange
    out.color = mix(core_color, cool_color, life_progress * 0.7);

    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Radial gradient for soft ember glow
    let dist = length(in.uv - 0.5) * 2.0;  // 0 at center, 1 at edge

    // Soft circular falloff
    let circle_alpha = 1.0 - smoothstep(0.0, 1.0, dist);

    // Final alpha with radial falloff
    let final_alpha = in.alpha * circle_alpha;

    // Discard nearly invisible pixels
    if (final_alpha < 0.01) {
        discard;
    }

    // Glow effect - brighter at center
    let glow = 1.0 + (1.0 - dist) * 0.5;
    let final_color = in.color * glow;

    return vec4<f32>(final_color, final_alpha);
}
