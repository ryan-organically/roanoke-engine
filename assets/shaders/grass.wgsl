// Grass Shader with Wind Animation and Shadows

struct CameraUniform {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    time: f32,
    _padding1: vec3<f32>,
    sun_dir: vec3<f32>,
    fog_density: f32,
    view_pos: vec3<f32>,
    fog_start: f32,
    fog_color: vec3<f32>,
    fog_end: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var t_shadow: texture_depth_2d;
@group(0) @binding(2)
var s_shadow: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) shadow_pos: vec3<f32>,
};

// Simple wind animation
// Uses sine waves based on actual elapsed time for organic movement
fn apply_wind(world_pos: vec3<f32>, height_factor: f32, time: f32) -> vec3<f32> {
    // Wind direction and strength
    let wind_strength = 0.15;
    let wind_direction = vec2<f32>(1.0, 0.5);

    // Multiple sine waves for organic motion using REAL TIME
    // Add world position for spatial variation so all grass doesn't move identically
    let wave1 = sin(time * 2.0 + world_pos.x * 0.5) * wind_strength;
    let wave2 = sin(time * 1.5 + world_pos.z * 0.7) * wind_strength * 0.5;

    // Only affect the top of the grass (based on height_factor)
    let wind_amount = height_factor * height_factor; // Quadratic falloff

    // Apply wind offset
    let wind_offset = vec3<f32>(
        (wave1 + wave2) * wind_direction.x * wind_amount,
        0.0,
        (wave1 + wave2) * wind_direction.y * wind_amount
    );

    return world_pos + wind_offset;
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate height factor (0 at base, 1 at tip)
    let height_factor = saturate(vertex.position.y / 1.0);

    // Apply wind animation with real time
    let animated_position = apply_wind(vertex.position, height_factor, camera.time);

    out.clip_position = camera.view_proj * vec4<f32>(animated_position, 1.0);
    out.color = vertex.color;
    out.world_position = animated_position;

    // Calculate shadow position
    let pos_from_light = camera.light_view_proj * vec4<f32>(animated_position, 1.0);
    let shadow_ndc = pos_from_light.xyz / pos_from_light.w;
    out.shadow_pos = vec3<f32>(
        shadow_ndc.x * 0.5 + 0.5,
        -shadow_ndc.y * 0.5 + 0.5,
        shadow_ndc.z
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sun direction from uniform (points FROM sun TO scene)
    let light_dir = normalize(camera.sun_dir);

    // Grass normal - mostly up, with slight variation based on position for visual interest
    let normal = normalize(vec3<f32>(
        sin(in.world_position.x * 0.5) * 0.1,
        1.0,
        cos(in.world_position.z * 0.5) * 0.1
    ));

    // Dynamic sun color matching terrain shader
    let sun_elevation = -light_dir.y;

    // Day factor: 0 = night, 1 = full day
    let day_factor = smoothstep(-0.1, 0.3, sun_elevation);

    // Night lighting from moon
    let moon_color = vec3<f32>(0.12, 0.15, 0.2);

    // Daytime sun color
    let day_sun_color = mix(
        vec3<f32>(1.8, 0.6, 0.2),  // Sunrise/sunset
        vec3<f32>(1.4, 1.3, 1.1),  // Midday
        clamp(sun_elevation * 2.0, 0.0, 1.0)
    );

    let sun_color = mix(moon_color, day_sun_color, day_factor);

    // Night ambient (very dark)
    let night_ambient = vec3<f32>(0.02, 0.025, 0.04);

    let day_ambient = mix(
        vec3<f32>(0.15, 0.10, 0.08),
        vec3<f32>(0.12, 0.14, 0.18),
        clamp(sun_elevation * 2.0, 0.0, 1.0)
    );

    let ambient_color = mix(night_ambient, day_ambient, day_factor);

    // Diffuse lighting
    let n_dot_l = max(dot(normal, -light_dir), 0.0);

    // Shadow calculation
    let shadow_uv = in.shadow_pos.xy;
    let shadow_depth = in.shadow_pos.z;

    var shadow = 1.0;
    if (shadow_uv.x >= 0.0 && shadow_uv.x <= 1.0 &&
        shadow_uv.y >= 0.0 && shadow_uv.y <= 1.0 &&
        shadow_depth >= 0.0 && shadow_depth <= 1.0) {
        shadow = textureSampleCompare(t_shadow, s_shadow, shadow_uv, shadow_depth);
        shadow = shadow * 0.8 + 0.2;
    }

    // Apply lighting
    let diffuse_contribution = sun_color * n_dot_l * 2.0 * shadow;
    let lighting = ambient_color + diffuse_contribution;
    var final_color = in.color * lighting;

    // Apply distance fog
    let dist_to_camera = distance(in.world_position, camera.view_pos);
    let fog_factor = saturate((dist_to_camera - camera.fog_start) / (camera.fog_end - camera.fog_start));
    let fog_amount = fog_factor * fog_factor * camera.fog_density;
    final_color = mix(final_color, camera.fog_color, fog_amount);

    return vec4<f32>(final_color, 1.0);
}
