// Grass Shader with Wind Animation and Shadows

struct CameraUniform {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    time: f32,
    _padding1: vec3<f32>,
    sun_dir: vec3<f32>,
    fog_density: f32,
    view_pos: vec3<f32>,
    fog_start: f32,
    fog_color: vec3<f32>,
    fog_end: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var t_shadow: texture_depth_2d;
@group(0) @binding(2)
var s_shadow: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) local_height: f32,  // 0.0 at base, 1.0 at tip - for wind animation
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) shadow_pos: vec3<f32>,
};

// Area-based wind system with gusts and calm periods
// Creates natural wave-like wind patterns that flow across the landscape

// Simple hash function for pseudo-random values
fn hash2d(p: vec2<f32>) -> f32 {
    let p2 = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)), dot(p, vec2<f32>(269.5, 183.3)));
    return fract(sin(dot(p2, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// Smooth noise for wind patterns
fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Smooth interpolation
    let u = f * f * (3.0 - 2.0 * f);

    // Four corners
    let a = hash2d(i);
    let b = hash2d(i + vec2<f32>(1.0, 0.0));
    let c = hash2d(i + vec2<f32>(0.0, 1.0));
    let d = hash2d(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Fractal Brownian Motion for organic wind patterns
fn fbm_wind(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pos = p;

    // 3 octaves of noise
    for (var i = 0; i < 3; i++) {
        value += amplitude * noise2d(pos);
        pos *= 2.0;
        amplitude *= 0.5;
    }

    return value;
}

fn apply_wind(world_pos: vec3<f32>, height_factor: f32, time: f32) -> vec3<f32> {
    // Base wind parameters
    let base_wind_strength = 0.12;
    let wind_direction = normalize(vec2<f32>(1.0, 0.3));  // Primary wind direction

    // === LAYER 1: Large-scale wind gusts ===
    // Gusts travel across the landscape over time
    let gust_scale = 0.02;  // Size of gust areas (larger = bigger gusts)
    let gust_speed = 0.3;   // How fast gusts travel
    let gust_pos = vec2<f32>(
        world_pos.x * gust_scale - time * gust_speed * wind_direction.x,
        world_pos.z * gust_scale - time * gust_speed * wind_direction.y
    );

    // Gust intensity varies smoothly across space and time
    let gust_noise = fbm_wind(gust_pos);
    // Remap to create calm periods (below 0.3 = calm, above = gusty)
    let gust_intensity = smoothstep(0.25, 0.7, gust_noise);

    // === LAYER 2: Medium-scale wave ripples ===
    // These are the visible "waves" of grass movement
    let wave_scale = 0.08;
    let wave_speed = 1.2;
    let wave_pos = vec2<f32>(
        world_pos.x * wave_scale - time * wave_speed,
        world_pos.z * wave_scale - time * wave_speed * 0.7
    );
    let wave = sin(wave_pos.x * 3.14159) * cos(wave_pos.y * 2.5);

    // === LAYER 3: Local turbulence ===
    // Small rapid variations for realism
    let turb_scale = 0.15;
    let turb_pos = vec2<f32>(
        world_pos.x * turb_scale + time * 0.5,
        world_pos.z * turb_scale + time * 0.3
    );
    let turbulence = noise2d(turb_pos) * 2.0 - 1.0;

    // === LAYER 4: Slow swaying (always present, very gentle) ===
    let sway = sin(time * 0.5 + world_pos.x * 0.02 + world_pos.z * 0.015) * 0.3;

    // === Combine layers ===
    // Gust modulates the intensity of waves
    let wind_power = gust_intensity * 0.8 + 0.2;  // Always some minimal movement

    // Calculate wind displacement
    let wave_contribution = wave * wind_power * base_wind_strength;
    let turb_contribution = turbulence * wind_power * base_wind_strength * 0.3;
    let sway_contribution = sway * base_wind_strength * 0.4;

    let total_wind = wave_contribution + turb_contribution + sway_contribution;

    // Height-based falloff - base stays still, tips move most
    // Using cubic falloff for more natural bend
    let wind_amount = height_factor * height_factor * height_factor;

    // Apply wind in primary direction with some perpendicular variation
    let perpendicular = vec2<f32>(-wind_direction.y, wind_direction.x);
    let wind_offset = vec3<f32>(
        (total_wind * wind_direction.x + turbulence * perpendicular.x * 0.2) * wind_amount,
        0.0,
        (total_wind * wind_direction.y + turbulence * perpendicular.y * 0.2) * wind_amount
    );

    return world_pos + wind_offset;
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Use local_height directly (0.0 at base, 1.0 at tip)
    // This is independent of world Y position, fixing the floating grass bug
    let height_factor = vertex.local_height;

    // Apply wind animation with real time
    let animated_position = apply_wind(vertex.position, height_factor, camera.time);

    out.clip_position = camera.view_proj * vec4<f32>(animated_position, 1.0);
    out.color = vertex.color;
    out.world_position = animated_position;

    // Calculate shadow position
    let pos_from_light = camera.light_view_proj * vec4<f32>(animated_position, 1.0);
    let shadow_ndc = pos_from_light.xyz / pos_from_light.w;
    out.shadow_pos = vec3<f32>(
        shadow_ndc.x * 0.5 + 0.5,
        -shadow_ndc.y * 0.5 + 0.5,
        shadow_ndc.z
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sun direction from uniform (points FROM sun TO scene)
    let light_dir = normalize(camera.sun_dir);

    // Grass normal - mostly up, with slight variation based on position for visual interest
    let normal = normalize(vec3<f32>(
        sin(in.world_position.x * 0.5) * 0.1,
        1.0,
        cos(in.world_position.z * 0.5) * 0.1
    ));

    // Dynamic sun color matching terrain shader
    let sun_elevation = -light_dir.y;

    // Day factor: 0 = night, 1 = full day
    // Now correctly receives sun_dir (not moon_dir) so this works properly
    let day_factor = smoothstep(-0.1, 0.3, sun_elevation);

    // Night lighting from moon - subtle, not washing out colors
    let moon_color = vec3<f32>(0.06, 0.08, 0.12);

    // Daytime sun color (cooler for less yellow grass)
    let day_sun_color = mix(
        vec3<f32>(1.4, 0.7, 0.4),  // Sunrise/sunset (less extreme orange)
        vec3<f32>(1.1, 1.1, 1.0),  // Midday (more neutral)
        clamp(sun_elevation * 2.0, 0.0, 1.0)
    );

    let sun_color = mix(moon_color, day_sun_color, day_factor);

    // Night ambient - dark with hint of blue, preserves grass color
    let night_ambient = vec3<f32>(0.015, 0.02, 0.035);

    let day_ambient = mix(
        vec3<f32>(0.15, 0.10, 0.08),
        vec3<f32>(0.12, 0.14, 0.18),
        clamp(sun_elevation * 2.0, 0.0, 1.0)
    );

    let ambient_color = mix(night_ambient, day_ambient, day_factor);

    // Diffuse lighting
    let n_dot_l = max(dot(normal, -light_dir), 0.0);

    // Shadow calculation
    let shadow_uv = in.shadow_pos.xy;
    let shadow_depth = in.shadow_pos.z;

    var shadow = 1.0;
    if (shadow_uv.x >= 0.0 && shadow_uv.x <= 1.0 &&
        shadow_uv.y >= 0.0 && shadow_uv.y <= 1.0 &&
        shadow_depth >= 0.0 && shadow_depth <= 1.0) {
        shadow = textureSampleCompare(t_shadow, s_shadow, shadow_uv, shadow_depth);
        shadow = shadow * 0.8 + 0.2;
    }

    // Apply lighting (reduced multiplier for darker grass)
    // Scale diffuse by day_factor to prevent bright grass at night when sun is below horizon
    let diffuse_contribution = sun_color * n_dot_l * 1.2 * shadow * day_factor;
    let lighting = ambient_color + diffuse_contribution;

    // At night, desaturate and darken the grass color to prevent neon glow
    // day_factor: 0 = night, 1 = day
    let grass_color = in.color;
    let grass_luminance = dot(grass_color, vec3<f32>(0.299, 0.587, 0.114));
    let desaturated_grass = vec3<f32>(grass_luminance);
    // At night: mostly desaturated and darker, during day: full color
    let night_grass = desaturated_grass * 0.3; // Very dark gray at night
    let adjusted_grass = mix(night_grass, grass_color, day_factor);

    var final_color = clamp(adjusted_grass * lighting, vec3<f32>(0.0), vec3<f32>(1.0));

    // Night brightness is now handled correctly via day_factor
    // (Fixed in main.rs by passing sun_dir instead of light_dir/moon_dir)

    // Apply distance fog
    let dist_to_camera = distance(in.world_position, camera.view_pos);
    let fog_factor = saturate((dist_to_camera - camera.fog_start) / (camera.fog_end - camera.fog_start));
    let fog_amount = fog_factor * fog_factor * camera.fog_density;
    final_color = mix(final_color, camera.fog_color, fog_amount);

    return vec4<f32>(final_color, 1.0);
}
