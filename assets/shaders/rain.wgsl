// Rain Particle Shader
// Renders rain drops as camera-facing stretched billboards

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
    camera_right: vec3<f32>,
    rain_intensity: f32,    // 0-1 controls particle density/opacity
    camera_up: vec3<f32>,
    wind_strength: f32,     // Affects rain angle
    fog_color: vec3<f32>,
    fog_start: f32,
    fog_end: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) alpha: f32,
    @location(1) view_distance: f32,
}

// Hash function for pseudo-random particle positions
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
    // Rain coverage area around camera
    let rain_radius = 80.0;
    let rain_height = 60.0;

    // Generate pseudo-random particle position based on instance
    let seed = vec3<f32>(f32(instance_index), f32(instance_index) * 1.3, f32(instance_index) * 0.7);
    let rand = hash33(seed);

    // Particle base position in a cylinder around camera
    var particle_pos = vec3<f32>(
        uniforms.camera_pos.x + (rand.x - 0.5) * rain_radius * 2.0,
        uniforms.camera_pos.y + rand.y * rain_height,
        uniforms.camera_pos.z + (rand.z - 0.5) * rain_radius * 2.0,
    );

    // Animate falling - use time and instance for variation
    let fall_speed = 25.0 + rand.x * 10.0;  // Variable fall speed
    let time_offset = hash31(seed) * 100.0;
    let fall_progress = fract((uniforms.time + time_offset) * fall_speed / rain_height);

    // Update Y position with looping animation
    particle_pos.y = uniforms.camera_pos.y + rain_height * (1.0 - fall_progress) - rain_height * 0.3;

    // Wind offset - rain tilts with wind
    let wind_offset = uniforms.wind_strength * fall_progress * 5.0;
    particle_pos.x += wind_offset;

    // Rain drop dimensions
    let drop_width = 0.015 + rand.y * 0.01;
    let drop_length = 0.4 + rand.z * 0.3;  // Stretched vertically for motion blur effect

    // Billboard vertices (2 triangles = 6 vertices per drop)
    // Vertices form a stretched quad oriented toward fall direction
    var local_offset: vec2<f32>;
    switch vertex_index % 6u {
        case 0u: { local_offset = vec2<f32>(-drop_width, drop_length); }
        case 1u: { local_offset = vec2<f32>(drop_width, drop_length); }
        case 2u: { local_offset = vec2<f32>(-drop_width, -drop_length); }
        case 3u: { local_offset = vec2<f32>(drop_width, drop_length); }
        case 4u: { local_offset = vec2<f32>(drop_width, -drop_length); }
        default: { local_offset = vec2<f32>(-drop_width, -drop_length); }
    }

    // Create billboard that faces camera but stretched vertically
    let right = normalize(uniforms.camera_right);
    // Mix between camera up and world up for rain direction
    let fall_dir = normalize(vec3<f32>(uniforms.wind_strength * 0.3, -1.0, 0.0));
    let world_pos = particle_pos + right * local_offset.x - fall_dir * local_offset.y;

    // Calculate view distance for fog
    let view_distance = length(uniforms.camera_pos - world_pos);

    // Alpha based on rain intensity and distance fade
    let distance_fade = 1.0 - smoothstep(rain_radius * 0.5, rain_radius, length(particle_pos.xz - uniforms.camera_pos.xz));
    let intensity_alpha = uniforms.rain_intensity * 0.6;
    let alpha = intensity_alpha * distance_fade;

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    out.alpha = alpha;
    out.view_distance = view_distance;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Rain drop color - slightly blue-grey, translucent
    let rain_color = vec3<f32>(0.7, 0.75, 0.85);

    // Distance fog fade
    let fog_factor = clamp(
        (in.view_distance - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start),
        0.0,
        1.0
    );

    // Fade rain into fog
    let final_color = mix(rain_color, uniforms.fog_color, fog_factor * 0.7);
    let final_alpha = in.alpha * (1.0 - fog_factor * 0.8);

    // Discard nearly invisible pixels
    if (final_alpha < 0.01) {
        discard;
    }

    return vec4<f32>(final_color, final_alpha);
}
