// Building Shader - Vertex Colors + Simple Lighting

struct Uniforms {
    view_proj: mat4x4<f32>,
    light_dir: vec3<f32>,
    _padding: f32,
    view_pos: vec3<f32>,
    _padding2: f32,
    fog_color: vec3<f32>,
    _padding3: f32,
    fog_start: f32,
    fog_end: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec3<f32>, // Vertex Color from procgen
    
    // Instance Transforms (Mat4 takes 4 slots)
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    // Reconstruct Model Matrix
    let model_matrix = mat4x4<f32>(
        input.model_matrix_0,
        input.model_matrix_1,
        input.model_matrix_2,
        input.model_matrix_3,
    );

    let world_pos = model_matrix * vec4<f32>(input.position, 1.0);
    let world_normal = normalize((model_matrix * vec4<f32>(input.normal, 0.0)).xyz);

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * world_pos;
    out.color = input.color;
    out.normal = world_normal;
    out.world_pos = world_pos.xyz;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Lighting
    let light_dir = normalize(uniforms.light_dir);
    let normal = normalize(in.normal);
    
    // Diffuse
    let diff = max(dot(normal, light_dir), 0.0);
    
    // Ambient (Sky light)
    let ambient = 0.3;
    
    // Combine
    let lighting = ambient + diff * 0.7;
    let lit_color = in.color * lighting;

    // Fog
    let dist = distance(in.world_pos, uniforms.view_pos);
    let fog_factor = clamp((dist - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    let final_color = mix(lit_color, uniforms.fog_color, fog_factor);

    return vec4<f32>(final_color, 1.0);
}
