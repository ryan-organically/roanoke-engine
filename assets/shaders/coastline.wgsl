// Coastline Biome Shader - Ported from HLSL to WGSL
// Renders ocean, beach, and coastal terrain with smooth transitions

struct CoastlineUniforms {
    sea_level: f32,
    beach_level: f32,
    shallow_level: f32,
    deep_level: f32,

    // Colors
    deep_ocean_color: vec4<f32>,
    shallow_ocean_color: vec4<f32>,
    beach_color: vec4<f32>,
    coast_color: vec4<f32>,

    // Material properties
    water_metallic: f32,
    water_roughness: f32,
    sand_metallic: f32,
    sand_roughness: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: CoastlineUniforms;

@group(0) @binding(1)
var height_texture: texture_2d<f32>;

@group(0) @binding(2)
var height_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) height: f32,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = vec4<f32>(input.position, 1.0);
    output.world_position = input.position;
    output.uv = input.uv;
    output.normal = normalize(input.normal);

    // Sample height from texture
    let height_sample = textureSample(height_texture, height_sampler, input.uv);
    output.height = height_sample.r;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;

    let height = input.height;

    // Determine biome based on height thresholds
    var final_color: vec4<f32>;

    // Deep ocean (below deep_level)
    if (height < uniforms.deep_level) {
        final_color = uniforms.deep_ocean_color;
    }
    // Shallow ocean (between deep_level and sea_level)
    else if (height < uniforms.sea_level) {
        let t = (height - uniforms.deep_level) / (uniforms.sea_level - uniforms.deep_level);
        let t_clamped = clamp(t, 0.0, 1.0);
        final_color = mix(uniforms.deep_ocean_color, uniforms.shallow_ocean_color, t_clamped);
    }
    // Beach (between sea_level and beach_level)
    else if (height < uniforms.beach_level) {
        let t = (height - uniforms.sea_level) / (uniforms.beach_level - uniforms.sea_level);
        let t_clamped = clamp(t, 0.0, 1.0);

        // Smooth transition from shallow water to beach
        // Add some foam effect near the shore
        let foam = smoothstep(0.0, 0.2, t_clamped) * (1.0 - smoothstep(0.8, 1.0, t_clamped));
        let foam_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

        let base_transition = mix(uniforms.shallow_ocean_color, uniforms.beach_color, t_clamped);
        final_color = mix(base_transition, foam_color, foam * 0.3);
    }
    // Coast (between beach_level and shallow_level)
    else if (height < uniforms.shallow_level) {
        let t = (height - uniforms.beach_level) / (uniforms.shallow_level - uniforms.beach_level);
        let t_clamped = clamp(t, 0.0, 1.0);
        final_color = mix(uniforms.beach_color, uniforms.coast_color, t_clamped);
    }
    // Above coastline - use coast color
    else {
        final_color = uniforms.coast_color;
    }

    // Apply simple lighting based on normal
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let diffuse = max(dot(input.normal, light_dir), 0.0);
    let ambient = 0.3;
    let lighting = clamp(ambient + diffuse * 0.7, 0.0, 1.0);

    final_color = vec4<f32>(final_color.rgb * lighting, final_color.a);

    output.color = final_color;

    return output;
}

// Additional utility functions

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}
