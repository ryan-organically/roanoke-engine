// Animal Model Shader
// Renders actual 3D animal models with per-instance transforms

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
    fog_color: vec3<f32>,
    fog_start: f32,
    fog_end: f32,
    fog_density: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

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
    @location(9) color: vec3<f32>,
    @location(10) emissive: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) emissive: f32,
    @location(4) uv: vec2<f32>,
    @location(5) view_distance: f32,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Reconstruct model matrix from instance data
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // Transform position to world space
    let world_pos = model_matrix * vec4<f32>(vertex.position, 1.0);

    // Transform normal to world space (using upper 3x3 of model matrix)
    let normal_matrix = mat3x3<f32>(
        instance.model_matrix_0.xyz,
        instance.model_matrix_1.xyz,
        instance.model_matrix_2.xyz,
    );
    let world_normal = normalize(normal_matrix * vertex.normal);

    // Calculate view distance for fog
    let view_distance = length(camera.camera_pos - world_pos.xyz);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_pos;
    out.world_normal = world_normal;
    out.world_position = world_pos.xyz;
    out.color = instance.color;
    out.emissive = instance.emissive;
    out.uv = vertex.uv;
    out.view_distance = view_distance;

    return out;
}

// Simple hash function for procedural variation
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Light direction (sun-like, from upper right)
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));
    let light_color = vec3<f32>(1.0, 0.95, 0.9);

    // View direction for specular
    let view_dir = normalize(camera.camera_pos - in.world_position);
    let half_dir = normalize(light_dir + view_dir);

    // Diffuse lighting
    let ndotl = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = ndotl * 0.7;

    // Specular highlight (Blinn-Phong) - reduced for matte animal fur
    let ndoth = max(dot(in.world_normal, half_dir), 0.0);
    let specular = pow(ndoth, 16.0) * 0.15;

    // Ambient light with slight sky contribution
    let sky_ambient = 0.15 * max(in.world_normal.y, 0.0);
    let ambient = 0.25 + sky_ambient;

    // Subtle subsurface scattering approximation for organic look
    let sss = max(0.0, dot(-light_dir, in.world_normal)) * 0.1;

    // Combine lighting
    let lighting = ambient + diffuse + specular + sss;

    // Add subtle fur texture variation using UV
    let fur_noise = hash(in.uv * 50.0) * 0.1;

    // Base color with lighting and variation
    var final_color = in.color * lighting * light_color;
    final_color = final_color * (0.95 + fur_noise);

    // Add emissive glow (for damage flash, aggressive states)
    if in.emissive > 0.0 {
        // Red damage flash
        let flash_color = vec3<f32>(1.0, 0.3, 0.2);
        final_color = mix(final_color, flash_color, in.emissive * 0.5);
        final_color = final_color + flash_color * in.emissive * 0.3;
    }

    // Apply fog
    let fog_factor = clamp(
        (in.view_distance - camera.fog_start) / (camera.fog_end - camera.fog_start),
        0.0,
        1.0
    );
    let fog_amount = 1.0 - exp(-fog_factor * camera.fog_density);
    final_color = mix(final_color, camera.fog_color, fog_amount);

    return vec4<f32>(final_color, 1.0);
}
