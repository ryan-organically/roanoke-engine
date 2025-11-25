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
    // Sun direction (fixed for now, should be uniform)
    let sun_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));

    // Simple diffuse lighting
    let normal = normalize(input.world_normal);
    let diffuse = max(dot(normal, sun_dir), 0.0);

    // Ambient lighting
    let ambient = 0.4;

    // Wood/Driftwood Color
    // Bleached wood color for driftwood
    let base_color = vec3<f32>(0.6, 0.55, 0.5);
    
    // Add some noise/variation based on UV
    let noise = fract(sin(dot(input.uv, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let variation = (noise - 0.5) * 0.1;
    
    let final_color = base_color + variation;

    // Apply lighting
    let lit_color = final_color * (ambient + diffuse * 0.6);

    return vec4<f32>(lit_color, 1.0);
}
