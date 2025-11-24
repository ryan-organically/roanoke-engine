struct CameraUniform {
    view_proj: mat4x4<f32>,
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
    // Sun direction (fixed for now)
    let sun_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));

    // Simple diffuse lighting
    let normal = normalize(input.world_normal);
    let diffuse = max(dot(normal, sun_dir), 0.0);

    // Ambient lighting
    let ambient = 0.3;

    // Determine if this is a branch or leaf based on UV
    // Branches have low V coordinate, leaves have high V coordinate
    let is_leaf = input.uv.y > 0.8;

    var base_color: vec3<f32>;
    if (is_leaf) {
        // Leaf color - bright green
        base_color = vec3<f32>(0.3, 0.65, 0.2);
    } else {
        // Bark color - brown with variation based on UV
        let bark_variation = fract(input.uv.y * 20.0) * 0.15;
        base_color = vec3<f32>(0.35 + bark_variation, 0.25 + bark_variation, 0.15);
    }

    // Apply lighting
    let lit_color = base_color * (ambient + diffuse * 0.7);

    // Alpha (leaves can be slightly transparent)
    var alpha = 1.0;
    if (is_leaf) {
        alpha = 0.95;
    }

    return vec4<f32>(lit_color, alpha);
}
