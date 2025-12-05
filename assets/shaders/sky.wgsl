struct Uniforms {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    sun_dir: vec3<f32>,
    time: f32,
    sun_color: vec3<f32>,
    cloud_coverage: f32,
    cloud_color_base: vec3<f32>,
    cloud_density: f32,
    cloud_color_shade: vec3<f32>,
    cloud_scale: f32,
    wind_offset: vec2<f32>,
    padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ndc_pos: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    let pos = positions[in_vertex_index];

    var output: VertexOutput;
    output.clip_position = vec4<f32>(pos, 1.0, 1.0);
    output.ndc_pos = pos;
    return output;
}

// Simple Hash Function
fn hash(p: vec2<f32>) -> f32 {
    var p2 = p;
    p2 = 50.0 * fract(p2 * 0.3183099 + vec2<f32>(0.71, 0.113));
    return -1.0 + 2.0 * fract(p2.x * p2.y * (p2.x + p2.y));
}

// 2D Noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(mix(hash(i + vec2<f32>(0.0, 0.0)), 
                   hash(i + vec2<f32>(1.0, 0.0)), u.x),
               mix(hash(i + vec2<f32>(0.0, 1.0)), 
                   hash(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

// FBM (Fractal Brownian Motion)
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 0.0;
    var p2 = p;
    
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(p2);
        p2 = p2 * 2.0;
        amplitude *= 0.5;
    }
    return value;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Reconstruct world-space ray direction from NDC
    let ndc = vec4<f32>(input.ndc_pos, 1.0, 1.0);
    let world_pos = uniforms.inv_view_proj * ndc;
    let ray_dir = normalize(world_pos.xyz / world_pos.w);

    // Sun elevation determines day/night (-1 = below horizon, 1 = zenith)
    let sun_elevation = -uniforms.sun_dir.y; // sun_dir points FROM sun, so negate

    // Day/Night sky colors
    let day_top = vec3<f32>(0.2, 0.4, 0.8);
    let day_horizon = vec3<f32>(0.6, 0.7, 0.9);
    let night_top = vec3<f32>(0.02, 0.02, 0.06);
    let night_horizon = vec3<f32>(0.05, 0.05, 0.1);
    let sunset_horizon = vec3<f32>(0.9, 0.4, 0.2);

    // Calculate day factor (0 = night, 1 = day)
    let day_factor = smoothstep(-0.2, 0.3, sun_elevation);

    // Sunset factor (peaks when sun is at horizon)
    let sunset_factor = smoothstep(-0.3, 0.0, sun_elevation) * smoothstep(0.3, 0.0, sun_elevation);

    // Interpolate top and horizon colors
    var top_color = mix(night_top, day_top, day_factor);
    var horizon_color = mix(night_horizon, day_horizon, day_factor);

    // Add sunset glow to horizon
    horizon_color = mix(horizon_color, sunset_horizon, sunset_factor * 0.7);

    // Sky gradient based on ray direction
    let y = ray_dir.y * 0.5 + 0.5; // -1..1 to 0..1
    var sky_color = mix(horizon_color, top_color, pow(clamp(y, 0.0, 1.0), 0.5));

    // Stars at night - sparse, subtle
    if (day_factor < 0.4 && ray_dir.y > 0.1) {
        let star_pos = ray_dir.xz / (ray_dir.y + 0.001) * 80.0;
        let star_hash = hash(floor(star_pos * 15.0)); // Fewer cells = fewer stars
        // Only 0.5% of cells have stars (was 3%)
        let star_brightness = step(0.995, star_hash) * (1.0 - day_factor * 2.5);
        let twinkle = sin(uniforms.time * 1.5 + star_hash * 50.0) * 0.2 + 0.8;
        // Dimmer stars, vary size slightly
        let star_intensity = star_brightness * twinkle * 0.7;
        sky_color += vec3<f32>(star_intensity * 0.9, star_intensity * 0.95, star_intensity);
    }

    // Cloud Rendering - wispy, translucent clouds that allow light through
    let cloud_height = 500.0;

    // Only render clouds when looking up (ray_dir.y > 0)
    if (ray_dir.y > 0.01) {
        let t = cloud_height / ray_dir.y;
        let cloud_pos = ray_dir.xz * t;

        let cloud_speed = 0.008;
        let time_offset = uniforms.time * cloud_speed;
        let wind = uniforms.wind_offset * 20.0 + vec2<f32>(time_offset, time_offset * 0.3);

        let uv_scaled = cloud_pos * 0.002 * uniforms.cloud_scale + wind;

        // Multi-layered noise for wispy effect
        var n = fbm(uv_scaled);
        let detail = fbm(uv_scaled * 3.0) * 0.3; // Fine detail layer
        n = (n * 0.5 + 0.5) + detail;

        let threshold = 1.0 - uniforms.cloud_coverage;
        // Softer edge for wispy appearance
        let cloud_alpha = smoothstep(threshold - 0.15, threshold + 0.25, n);

        let horizon_fade = smoothstep(0.01, 0.2, ray_dir.y);

        // Fade clouds at night (but don't completely hide them - moonlit clouds)
        let night_cloud_fade = mix(0.2, 1.0, day_factor);

        // Maximum cloud opacity reduced for translucent/phantom effect
        let max_opacity = 0.65; // Clouds never fully opaque
        let density = cloud_alpha * uniforms.cloud_density * horizon_fade * night_cloud_fade * max_opacity;

        if (density > 0.01) {
            let color_mix = smoothstep(threshold, threshold + 0.4, n);
            var cloud_rgb = mix(uniforms.cloud_color_base, uniforms.cloud_color_shade, color_mix);

            // Darken clouds at night
            let night_cloud_color = cloud_rgb * 0.2; // Slightly brighter for moonlit effect
            cloud_rgb = mix(night_cloud_color, cloud_rgb, day_factor);

            // Sun/moon scattering through clouds (light rays effect)
            let sun_dot = max(dot(ray_dir, -uniforms.sun_dir), 0.0);
            let scatter = pow(sun_dot, 4.0) * 0.4; // Soft glow around sun/moon through clouds
            let scatter_color = mix(vec3<f32>(0.3, 0.35, 0.5), uniforms.sun_color, day_factor);
            cloud_rgb += scatter_color * scatter * (1.0 - density * 0.5);

            let highlight = smoothstep(0.75, 1.0, n);
            let highlight_color = mix(vec3<f32>(0.25, 0.25, 0.35), vec3<f32>(1.0, 0.95, 0.9), day_factor);
            let final_cloud_color = mix(cloud_rgb, highlight_color, highlight * 0.4);

            sky_color = mix(sky_color, final_cloud_color, density);
        }
    }

    return vec4<f32>(sky_color, 1.0);
}
