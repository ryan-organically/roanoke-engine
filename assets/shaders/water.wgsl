// Ocean Water Surface Shader
// Shiny, blue, transparent water with visible rolling waves

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}

struct TimeUniform {
    time: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> time_data: TimeUniform;

struct WaterMaterial {
    deep_color: vec4<f32>,
    shallow_color: vec4<f32>,
    foam_color: vec4<f32>,
    smoothness: f32,
    metallic: f32,
    turbidity: f32,
    max_transparency_depth: f32,
}

struct WaterBiomeData {
    biome_type: u32,
    _padding: vec3<u32>,
}

@group(1) @binding(0) var<uniform> material: WaterMaterial;
@group(1) @binding(1) var<uniform> biome_data: WaterBiomeData;
@group(1) @binding(2) var displacement_texture: texture_2d<f32>;
@group(1) @binding(3) var displacement_sampler: sampler;
@group(1) @binding(4) var normal_texture: texture_2d<f32>;
@group(1) @binding(5) var normal_sampler: sampler;
@group(1) @binding(6) var shore_texture: texture_2d<f32>;
@group(1) @binding(7) var shore_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) view_vector: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Sample displacement from compute shader
    let disp = textureSampleLevel(displacement_texture, displacement_sampler, input.uv, 0.0);

    // Apply displacement - this creates the actual wave geometry
    let displaced_pos = input.position + vec3<f32>(disp.x, disp.y, disp.z);

    output.world_position = displaced_pos;
    output.clip_position = camera.view_proj * vec4<f32>(displaced_pos, 1.0);
    output.uv = input.uv;
    output.view_vector = camera.position - displaced_pos;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(input.view_vector);

    // Sample data from compute shader
    let disp_data = textureSample(displacement_texture, displacement_sampler, input.uv);
    let wave_height = disp_data.y;
    let foam = disp_data.w;

    let normal_data = textureSample(normal_texture, normal_sampler, input.uv);
    let normal = normalize(normal_data.xyz);
    let shore_dist = normal_data.w * 100.0;  // Unpack to meters

    // ========================================================================
    // WATER COLOR - Rich ocean blue
    // ========================================================================

    // Deep blue for deep water, aqua/teal for shallows
    let deep_blue = vec3<f32>(0.02, 0.08, 0.22);      // Rich deep ocean
    let mid_blue = vec3<f32>(0.05, 0.18, 0.35);       // Mid depth
    let shallow_aqua = vec3<f32>(0.12, 0.40, 0.48);   // Shallow turquoise
    let very_shallow = vec3<f32>(0.25, 0.55, 0.55);   // Very shallow, almost see-through

    // Blend based on distance from shore
    var water_color: vec3<f32>;
    if (shore_dist > 60.0) {
        water_color = deep_blue;
    } else if (shore_dist > 30.0) {
        let t = (shore_dist - 30.0) / 30.0;
        water_color = mix(mid_blue, deep_blue, t);
    } else if (shore_dist > 10.0) {
        let t = (shore_dist - 10.0) / 20.0;
        water_color = mix(shallow_aqua, mid_blue, t);
    } else {
        let t = shore_dist / 10.0;
        water_color = mix(very_shallow, shallow_aqua, t);
    }

    // ========================================================================
    // LIGHTING - Shiny reflective surface
    // ========================================================================

    let sun_dir = normalize(vec3<f32>(0.3, 0.8, 0.4));  // High sun
    let sun_color = vec3<f32>(1.0, 0.98, 0.92);  // Warm sunlight

    // Fresnel - controls reflection vs refraction
    let NdotV = max(dot(normal, view_dir), 0.001);
    let fresnel = 0.02 + 0.98 * pow(1.0 - NdotV, 4.0);  // Schlick approximation

    // Specular highlight - sharp sun reflection
    let half_vec = normalize(view_dir + sun_dir);
    let NdotH = max(dot(normal, half_vec), 0.0);
    let specular = pow(NdotH, 512.0) * 2.0;  // Very sharp highlight

    // Secondary softer specular
    let spec_soft = pow(NdotH, 64.0) * 0.3;

    // Diffuse lighting
    let NdotL = max(dot(normal, sun_dir), 0.0);
    let diffuse = NdotL * 0.3 + 0.7;  // Subtle shading

    // Sky reflection
    let sky_color = vec3<f32>(0.5, 0.7, 0.95);
    let horizon_color = vec3<f32>(0.7, 0.8, 0.9);

    // Reflect view dir around normal for sky sampling
    let reflect_dir = reflect(-view_dir, normal);
    let sky_blend = smoothstep(-0.1, 0.5, reflect_dir.y);
    let reflected_sky = mix(horizon_color, sky_color, sky_blend);

    // ========================================================================
    // FOAM - Only at breaking wave crests
    // ========================================================================

    let foam_color = vec3<f32>(0.95, 0.97, 1.0);  // White with slight blue
    let foam_amount = clamp(foam, 0.0, 1.0);

    // ========================================================================
    // COMBINE
    // ========================================================================

    // Base water with diffuse
    var color = water_color * diffuse;

    // Add sky reflection based on fresnel
    color = mix(color, reflected_sky, fresnel * 0.7);

    // Add specular highlights
    color += sun_color * (specular + spec_soft);

    // Add foam on top
    color = mix(color, foam_color, foam_amount * 0.9);

    // ========================================================================
    // TRANSPARENCY
    // ========================================================================

    // Shallow water is transparent, deep water is opaque
    var alpha: f32;
    if (shore_dist < 5.0) {
        // Very shallow - highly transparent
        alpha = 0.3 + shore_dist * 0.08;  // 0.3 to 0.7
    } else if (shore_dist < 20.0) {
        // Shallow to mid
        let t = (shore_dist - 5.0) / 15.0;
        alpha = mix(0.7, 0.85, t);
    } else {
        // Deep water - mostly opaque but still some transparency
        alpha = min(0.85 + shore_dist * 0.001, 0.92);
    }

    // Fresnel makes glancing angles more opaque (realistic)
    alpha = mix(alpha, 0.95, fresnel * 0.3);

    // Foam is opaque
    alpha = mix(alpha, 0.98, foam_amount);

    // Wave crests catch more light and are more visible
    if (wave_height > 0.3) {
        let crest_factor = smoothstep(0.3, 1.0, wave_height);
        color += vec3<f32>(0.05, 0.08, 0.1) * crest_factor;  // Slight brightening at crests
    }

    return vec4<f32>(color, alpha);
}
