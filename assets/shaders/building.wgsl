// Building Shader - Vertex Colors + Shadow Mapping + Moody Lighting

struct Uniforms {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,  // For shadow mapping
    light_dir: vec3<f32>,
    _padding: f32,
    view_pos: vec3<f32>,
    ambient_dimming: f32,         // Moody atmosphere dimming
    fog_color: vec3<f32>,
    _padding3: f32,
    fog_start: f32,
    fog_end: f32,
    shadow_strength: f32,         // How dark shadows appear (0-1)
    rain_wetness: f32,            // Darkens surfaces when wet
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Shadow map bindings
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;

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
    @location(3) shadow_pos: vec3<f32>,
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

    // Calculate shadow position
    let pos_from_light = uniforms.light_view_proj * world_pos;
    let shadow_ndc = pos_from_light.xyz / pos_from_light.w;
    let shadow_pos = vec3<f32>(
        shadow_ndc.x * 0.5 + 0.5,
        -shadow_ndc.y * 0.5 + 0.5,
        shadow_ndc.z
    );

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * world_pos;
    out.color = input.color;
    out.normal = world_normal;
    out.world_pos = world_pos.xyz;
    out.shadow_pos = shadow_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Lighting
    let light_dir = uniforms.light_dir; // pre-normalized on CPU
    let normal = normalize(in.normal);
    let sun_elevation = -light_dir.y;

    // Day factor: 0 = night, 1 = full day
    let day_factor = smoothstep(-0.1, 0.3, sun_elevation);

    // Shadow calculation
    var shadow = 1.0;
    let shadow_uv = in.shadow_pos.xy;
    let shadow_depth = in.shadow_pos.z;

    if (shadow_uv.x >= 0.0 && shadow_uv.x <= 1.0 &&
        shadow_uv.y >= 0.0 && shadow_uv.y <= 1.0 &&
        shadow_depth >= 0.0 && shadow_depth <= 1.0) {
        // PCF shadow sampling for soft shadows
        shadow = textureSampleCompare(t_shadow, s_shadow, shadow_uv, shadow_depth);
        // Apply shadow strength - darker shadows for moody atmosphere
        let shadow_darkness = 0.15 + (1.0 - uniforms.shadow_strength) * 0.35;
        shadow = shadow * (1.0 - shadow_darkness) + shadow_darkness;
    }

    // Diffuse lighting
    let diff = max(dot(normal, -light_dir), 0.0);

    // Ambient - reduced for moody atmosphere
    let night_ambient = 0.04;
    let day_ambient = 0.2 * (1.0 - uniforms.ambient_dimming * 0.5);
    let ambient = mix(night_ambient, day_ambient, day_factor);

    // Diffuse strength - reduced in overcast
    let night_diffuse = 0.08;
    let day_diffuse = 0.55 * (1.0 - uniforms.ambient_dimming * 0.4);
    let diffuse_strength = mix(night_diffuse, day_diffuse, day_factor);

    // View direction for specular/rim
    let view_dir = normalize(uniforms.view_pos - in.world_pos);

    // Subtle rim lighting for shape definition in moody lighting
    let brim = 1.0 - max(dot(view_dir, normal), 0.0);
    let brim2 = brim * brim;
    let rim = brim2 * brim2 * 0.15 * day_factor;

    // Apply shadow to diffuse only
    let lighting = ambient + (diff * diffuse_strength + rim) * shadow;

    // Base color - darken when wet from rain
    var base_color = in.color;
    if (uniforms.rain_wetness > 0.0) {
        // Wet surfaces are darker and slightly more saturated
        let wetness = uniforms.rain_wetness;
        base_color = base_color * (1.0 - wetness * 0.25);
        // Add subtle specular highlight on wet surfaces
        let half_dir = normalize(-light_dir + view_dir);
        var bsp = max(dot(normal, half_dir), 0.0);
        bsp *= bsp; bsp *= bsp; bsp *= bsp; bsp *= bsp; bsp *= bsp;
        let spec = bsp * wetness * 0.3 * day_factor * shadow;
        base_color = base_color + vec3<f32>(spec);
    }

    let lit_color = base_color * lighting;

    // Fog - apply moody atmosphere color
    let dist = distance(in.world_pos, uniforms.view_pos);
    let fog_factor = clamp((dist - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    let final_color = mix(lit_color, uniforms.fog_color, fog_factor);

    return vec4<f32>(final_color, 1.0);
}
