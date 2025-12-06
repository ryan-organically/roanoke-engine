// Animal Orb Shader
// Renders colored spheres representing animals with behavior-based effects

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
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

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_pos;
    out.world_normal = world_normal;
    out.world_position = world_pos.xyz;
    out.color = instance.color;
    out.emissive = instance.emissive;

    return out;
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

    // Specular highlight (Blinn-Phong)
    let ndoth = max(dot(in.world_normal, half_dir), 0.0);
    let specular = pow(ndoth, 32.0) * 0.4;

    // Ambient light
    let ambient = 0.3;

    // Fresnel rim lighting for orb effect
    let ndotv = max(dot(in.world_normal, view_dir), 0.0);
    let fresnel = pow(1.0 - ndotv, 3.0) * 0.5;

    // Combine lighting
    let lighting = ambient + diffuse + specular + fresnel;

    // Base color with lighting
    var final_color = in.color * lighting * light_color;

    // Add emissive glow (for aggressive/attacking states)
    if in.emissive > 0.0 {
        // Pulsing glow effect based on emissive value
        let glow_intensity = in.emissive * (0.5 + 0.5 * sin(in.world_position.y * 10.0));
        final_color = final_color + in.color * glow_intensity;

        // Add rim glow
        let rim_glow = pow(1.0 - ndotv, 2.0) * in.emissive * 0.8;
        final_color = final_color + in.color * rim_glow;
    }

    // Slight transparency for ethereal effect
    let alpha = 0.9 + fresnel * 0.1;

    return vec4<f32>(final_color, alpha);
}
