// Weapon viewmodel shader for first-person weapons (GLB models with textures)

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    muzzle_flash: vec4<f32>, // x = intensity, yzw = barrel tip position
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) model_pos: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;

    // Transform normal (using upper 3x3 of model matrix)
    let normal_matrix = mat3x3<f32>(
        uniforms.model[0].xyz,
        uniforms.model[1].xyz,
        uniforms.model[2].xyz
    );
    output.world_normal = normalize(normal_matrix * input.normal);
    output.uv = input.uv;
    output.model_pos = input.position;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(t_diffuse, s_diffuse, input.uv);

    // Simple lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.4;
    let diffuse = max(dot(input.world_normal, light_dir), 0.0) * 0.6;
    let lighting = ambient + diffuse;

    var final_color = tex_color.rgb * lighting;

    // Muzzle flash - explosive emission at barrel tip only
    let flash_intensity = uniforms.muzzle_flash.x;
    let barrel_tip = uniforms.muzzle_flash.yzw;

    if flash_intensity > 0.0 {
        // Distance from barrel tip
        let dist_to_barrel = length(input.model_pos - barrel_tip);

        // Only render flash at the very end of the barrel
        let flash_radius = 0.12;
        if dist_to_barrel < flash_radius {
            // Bright explosive core - white hot center fading to orange
            let core_t = dist_to_barrel / flash_radius;
            let core_intensity = (1.0 - core_t * core_t) * flash_intensity;

            // White-hot center, orange-red edges
            let inner_color = vec3<f32>(1.0, 1.0, 0.9);  // White hot
            let outer_color = vec3<f32>(1.0, 0.5, 0.1);  // Orange fire
            let flash_color = mix(inner_color, outer_color, core_t);

            // Additive blend for emissive look
            final_color += flash_color * core_intensity * 4.0;
        }

        // Subtle rim glow just around barrel opening (very localized)
        let rim_start = 0.12;
        let rim_end = 0.2;
        if dist_to_barrel >= rim_start && dist_to_barrel < rim_end {
            let rim_t = (dist_to_barrel - rim_start) / (rim_end - rim_start);
            let rim_glow = (1.0 - rim_t) * flash_intensity * 0.3;
            final_color += vec3<f32>(1.0, 0.6, 0.2) * rim_glow;
        }
    }

    return vec4<f32>(final_color, tex_color.a);
}
