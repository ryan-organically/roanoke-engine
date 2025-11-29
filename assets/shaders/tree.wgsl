struct CameraUniform {
    view_proj: mat4x4<f32>,
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

    let world_position = model_matrix * vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_position;
    output.world_position = world_position.xyz;
    
    // Transform normal (assuming uniform scaling, otherwise need normal matrix)
    output.world_normal = (model_matrix * vec4<f32>(input.normal, 0.0)).xyz;
    output.uv = input.uv;

    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    
    // Alpha mask (discard transparent pixels for leaves)
    if (tex_color.a < 0.5) {
        discard;
    }

    // Simple Hash Noise to break up "sloppy" flat textures
    let noise_scale = 50.0;
    let p = in.world_position * noise_scale;
    let noise = fract(sin(dot(p, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
    let noise_factor = 0.9 + noise * 0.2; // +/- 10% variation

    // Improved Lighting (Half-Lambert for softer shading)
    // Hardcoded sun direction matching terrain (approx)
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.3)); 
    let n_dot_l = dot(normalize(in.world_normal), light_dir);
    let diffuse = pow(n_dot_l * 0.5 + 0.5, 2.0); // Half-Lambert
    
    // Ambient
    let ambient = 0.3;
    let lighting = ambient + diffuse * 0.9;

    let final_color = tex_color.rgb * lighting * noise_factor;

    // Simple distance fog to blend with terrain
    // Hardcoded fog params matching terrain roughly
    // fog_start = 100.0, fog_end = 1000.0
    let dist = distance(in.world_position, vec3<f32>(0.0, 0.0, 0.0)); // Camera pos unknown here, assuming near origin for now or just skipping fog
    // Without camera pos, fog is hard. Skipping fog for now to avoid artifacts.
    
    return vec4<f32>(final_color, 1.0);
}
