// Viewmodel shader for first-person arms and weapons

struct ViewModelUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: ViewModelUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.color = input.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple shading - add some basic lighting effect
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let ambient = 0.6;
    let diffuse = 0.4;

    // Simple diffuse lighting based on color variation
    let lighting = ambient + diffuse;
    let final_color = input.color * lighting;

    return vec4<f32>(final_color, 1.0);
}
