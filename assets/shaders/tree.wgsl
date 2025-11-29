struct CameraUniform {
    view_proj: mat4x4<f32>,
    sun_dir: vec3<f32>,
    time: f32, // For wind animation
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) local_height: f32, // Height in local space for bark gradient
}

// Tree wind animation - slower and more subtle than grass
fn apply_tree_wind(world_pos: vec3<f32>, local_height: f32, time: f32) -> vec3<f32> {
    // Wind is subtle for trees - they sway slowly
    let wind_strength = 0.08;
    let wind_direction = vec2<f32>(1.0, 0.3);

    // Slow sine waves for tree sway
    let wave1 = sin(time * 0.8 + world_pos.x * 0.1) * wind_strength;
    let wave2 = sin(time * 0.5 + world_pos.z * 0.15) * wind_strength * 0.6;

    // Height-based influence: trunk stays still, branches sway more
    // local_height is in model space (0 = base, higher = branches)
    let height_factor = saturate(local_height / 5.0); // Normalize to ~5 units
    let wind_amount = height_factor * height_factor; // Quadratic falloff

    let wind_offset = vec3<f32>(
        (wave1 + wave2) * wind_direction.x * wind_amount,
        0.0,
        (wave1 + wave2) * wind_direction.y * wind_amount
    );

    return world_pos + wind_offset;
}

@vertex
fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {
    var output: VertexOutput;

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // Store local height before transformation
    let local_height = input.position.y;
    output.local_height = local_height;

    let world_position = model_matrix * vec4<f32>(input.position, 1.0);

    // Apply wind animation
    let animated_position = apply_tree_wind(world_position.xyz, local_height, camera.time);

    output.clip_position = camera.view_proj * vec4<f32>(animated_position, 1.0);
    output.world_position = animated_position;

    // Transform normal (assuming uniform scaling, otherwise need normal matrix)
    output.world_normal = (model_matrix * vec4<f32>(input.normal, 0.0)).xyz;
    output.uv = input.uv;

    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Procedural bark color with variation - no texture needed
    let noise = fract(sin(dot(in.world_position.xz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let noise2 = fract(sin(dot(in.world_position.xy * 0.5, vec2<f32>(39.346, 11.135))) * 43758.5453);

    // Base bark brown with variation
    let bark_dark = vec3<f32>(0.25, 0.15, 0.08);  // Dark bark
    let bark_light = vec3<f32>(0.45, 0.30, 0.18); // Light bark
    let bark_color = mix(bark_dark, bark_light, noise * 0.6 + noise2 * 0.4);

    // Height-based variation (darker at base, lighter higher up)
    let height_factor = saturate(in.local_height / 8.0);
    let final_bark = mix(bark_color * 0.8, bark_color * 1.1, height_factor);

    // Lighting
    let light_dir = normalize(camera.sun_dir);
    let sun_elevation = -light_dir.y;
    let day_factor = smoothstep(-0.1, 0.3, sun_elevation);

    // Diffuse with half-lambert for softer shadows
    let n_dot_l = dot(normalize(in.world_normal), -light_dir);
    let diffuse = pow(n_dot_l * 0.5 + 0.5, 2.0);

    // Ambient
    let night_ambient = vec3<f32>(0.03, 0.04, 0.06);
    let day_ambient = vec3<f32>(0.20, 0.18, 0.15);
    let ambient = mix(night_ambient, day_ambient, day_factor);

    // Sun color
    let sunrise_color = vec3<f32>(1.3, 0.6, 0.3);
    let midday_color = vec3<f32>(1.1, 1.05, 0.95);
    let sun_color = mix(sunrise_color, midday_color, saturate(sun_elevation * 2.0));

    let diffuse_strength = mix(0.1, 0.7, day_factor);
    let lighting = ambient + sun_color * diffuse * diffuse_strength;

    return vec4<f32>(final_bark * lighting, 1.0);
}
