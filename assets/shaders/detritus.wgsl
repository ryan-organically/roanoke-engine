struct CameraUniform {
    view_proj: mat4x4<f32>,
    sun_dir: vec3<f32>,
    fog_density: f32,
    view_pos: vec3<f32>,
    fog_start: f32,
    fog_color: vec3<f32>,
    fog_end: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_position = vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_position;
    output.world_position = input.position;
    output.world_normal = input.normal;
    output.uv = input.uv;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sun direction from uniform (already normalized on CPU)
    let light_dir = camera.sun_dir;
    let sun_elevation = -light_dir.y;

    // Day factor: 0 = night, 1 = full day
    let day_factor = smoothstep(-0.1, 0.3, sun_elevation);

    // Simple diffuse lighting
    let normal = normalize(input.world_normal);
    let diffuse = max(dot(normal, -light_dir), 0.0);

    // Night ambient (very dark)
    let night_ambient = 0.06;
    let day_ambient = 0.4;
    let ambient = mix(night_ambient, day_ambient, day_factor);

    // Night diffuse
    let night_diffuse = 0.1;
    let day_diffuse = 0.6;
    let diffuse_strength = mix(night_diffuse, day_diffuse, day_factor);

    // Ground log/rock colors
    // Use world position to determine color variation
    let pos_hash = fract(sin(dot(input.world_position.xz, vec2<f32>(12.9898, 78.233))) * 43758.5453);

    // Dark brown for logs, grey for rocks (randomized per object)
    let log_color = vec3<f32>(0.28, 0.18, 0.10);   // Dark brown bark
    let rock_color = vec3<f32>(0.45, 0.42, 0.38);  // Grey stone
    let base_color = mix(log_color, rock_color, step(0.6, pos_hash));

    // Add noise/variation for texture
    let noise = fract(sin(dot(input.uv + input.world_position.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let variation = (noise - 0.5) * 0.08;

    // Add moss/weathering on top surfaces
    let moss_factor = max(0.0, input.world_normal.y - 0.5) * 0.3;
    let moss_color = vec3<f32>(0.15, 0.25, 0.08);

    let final_color = mix(base_color + variation, moss_color, moss_factor);

    // Apply lighting
    var lit_color = final_color * (ambient + diffuse * diffuse_strength);

    // Apply distance fog
    let dist_to_camera = distance(input.world_position, camera.view_pos);
    let fog_factor = saturate((dist_to_camera - camera.fog_start) / (camera.fog_end - camera.fog_start));
    let fog_amount = fog_factor * fog_factor * camera.fog_density;
    lit_color = mix(lit_color, camera.fog_color, fog_amount);

    return vec4<f32>(lit_color, 1.0);
}
