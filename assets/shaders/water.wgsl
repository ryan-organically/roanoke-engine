// Ocean Water Surface Shader
// Shiny, blue, transparent water with visible rolling waves

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}

struct TimeAndLightUniform {
    time: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    sun_dir: vec3<f32>,
    _pad3: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> time_data: TimeAndLightUniform;

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

    // Blend based on distance from shore (branchless smoothstep)
    let t_shallow = smoothstep(0.0, 10.0, shore_dist);
    let t_mid = smoothstep(10.0, 30.0, shore_dist);
    let t_deep = smoothstep(30.0, 60.0, shore_dist);
    var water_color = mix(very_shallow, shallow_aqua, t_shallow);
    water_color = mix(water_color, mid_blue, t_mid);
    water_color = mix(water_color, deep_blue, t_deep);

    // ========================================================================
    // LIGHTING - Blue water with subtle reflections
    // ========================================================================

    let sun_dir = time_data.sun_dir; // pre-normalized on CPU
    // Dynamic sun color based on elevation (sun_dir.y = how high the sun is)
    let sun_elevation = sun_dir.y;
    let day_factor = smoothstep(-0.1, 0.3, sun_elevation);
    let sun_color = mix(
        vec3<f32>(0.1, 0.12, 0.18),  // Night: dim blue moonlight
        mix(
            vec3<f32>(1.4, 0.6, 0.25),   // Sunrise/sunset: warm orange
            vec3<f32>(1.0, 0.95, 0.85),   // Midday: white-yellow
            clamp(sun_elevation * 2.5, 0.0, 1.0)
        ),
        day_factor
    );

    // Fresnel - proper Schlick approximation for water (IOR ~1.33, F0 ≈ 0.02)
    let NdotV = max(dot(normal, view_dir), 0.001);
    let F0 = 0.02;
    let one_minus_NdotV = 1.0 - NdotV;
    let omn2 = one_minus_NdotV * one_minus_NdotV;
    let fresnel = F0 + (1.0 - F0) * omn2 * omn2 * one_minus_NdotV;

    // Specular highlight - sun sparkles on water
    let half_vec = normalize(view_dir + sun_dir);
    let NdotH = max(dot(normal, half_vec), 0.0);
    // pow(NdotH, 256) via 8 iterative squares
    var sp = NdotH; sp *= sp; sp *= sp; sp *= sp; sp *= sp; sp *= sp; sp *= sp; sp *= sp; sp *= sp;
    let specular = sp * 0.8;

    // Diffuse lighting - subtle shading
    let NdotL = max(dot(normal, sun_dir), 0.0);
    let diffuse = NdotL * 0.2 + 0.8;  // Mostly ambient

    // Sky reflection - responds to time of day
    let night_sky = vec3<f32>(0.02, 0.03, 0.06);
    let day_sky_zenith = vec3<f32>(0.4, 0.6, 0.9);
    let day_sky_horizon = vec3<f32>(0.7, 0.75, 0.8);
    let sunset_sky = vec3<f32>(0.8, 0.45, 0.2);
    let sunset_factor = smoothstep(0.0, 0.15, sun_elevation) * (1.0 - smoothstep(0.15, 0.4, sun_elevation));
    let sky_zenith = mix(night_sky, mix(day_sky_zenith, sunset_sky, sunset_factor * 0.5), day_factor);
    let sky_horizon = mix(night_sky * 1.5, mix(day_sky_horizon, sunset_sky, sunset_factor), day_factor);

    // Reflect view dir around normal for sky sampling
    let reflect_dir = reflect(-view_dir, normal);
    let sky_blend = smoothstep(-0.2, 0.6, reflect_dir.y);
    let reflected_sky = mix(sky_horizon, sky_zenith, sky_blend);

    // ========================================================================
    // FOAM - Breaking wave crests + shore foam line
    // ========================================================================

    let foam_color = vec3<f32>(0.95, 0.97, 1.0);  // White with slight blue
    var foam_amount = clamp(foam, 0.0, 1.0);

    // Shore foam - animated white line where waves meet the beach
    if (shore_dist < 8.0) {
        let t = time_data.time;
        // Scrolling noise pattern along shore
        let foam_uv = input.world_position.xz * 0.15;
        let noise1 = sin(foam_uv.x * 3.0 + t * 1.2) * sin(foam_uv.y * 2.5 + t * 0.8);
        let noise2 = sin(foam_uv.x * 5.0 - t * 1.5) * sin(foam_uv.y * 4.0 + t * 1.1);
        let foam_noise = (noise1 + noise2) * 0.5 + 0.5;

        // Foam band concentrated near shore (peaks at 1-3m from shore)
        let shore_band = smoothstep(0.0, 1.5, shore_dist) * smoothstep(8.0, 3.0, shore_dist);
        let shore_foam = shore_band * foam_noise * 0.7;
        foam_amount = max(foam_amount, shore_foam);
    }

    // ========================================================================
    // COMBINE - Fresnel-driven reflection
    // ========================================================================

    // Base water with diffuse lighting
    var color = water_color * diffuse;

    // Fresnel reflection - at grazing angles, reflect sky; looking down, see water
    color = mix(color, reflected_sky, fresnel);

    // Add specular highlights (sun sparkle) - stronger for glint
    color += sun_color * specular * 1.5;

    // Foam only at breaking crests - reduced intensity
    color = mix(color, foam_color, foam_amount * 0.6);

    // ========================================================================
    // TRANSPARENCY
    // ========================================================================

    // Depth-based transparency using exponential falloff (Beer's law approximation)
    // Extinction coefficient controls how quickly water becomes opaque
    let extinction = 0.08; // Lower = clearer water
    var alpha = 1.0 - exp(-shore_dist * extinction);
    alpha = clamp(alpha, 0.15, 0.93); // Never fully transparent or opaque

    // Fresnel makes glancing angles more opaque (realistic)
    alpha = mix(alpha, 0.98, fresnel);

    // Foam is opaque
    alpha = mix(alpha, 0.98, foam_amount);

    // Wave crests catch more light and are more visible
    if (wave_height > 0.3) {
        let crest_factor = smoothstep(0.3, 1.0, wave_height);
        color += vec3<f32>(0.05, 0.08, 0.1) * crest_factor;  // Slight brightening at crests
    }

    return vec4<f32>(color, alpha);
}
