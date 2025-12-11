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
    moon_dir: vec3<f32>,
    rain_intensity: f32,  // 0-1 rain amount
    wind_offset: vec2<f32>,
    ambient_dimming: f32, // Overall atmosphere dimming for moody look
    _pad1: f32,
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

// Improved hash function - better distribution, less tiling
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash2(p: vec2<f32>) -> f32 {
    // Second hash for layering - different constants
    let p2 = fract(p * vec2<f32>(0.3183099, 0.3678794));
    let p3 = p2 + dot(p2, p2.yx + 19.19);
    return -1.0 + 2.0 * fract(p3.x * p3.y);
}

// Improved 2D noise with quintic interpolation (smoother, less grid artifacts)
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    // Quintic interpolation for smoother results
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);

    let a = hash2(i + vec2<f32>(0.0, 0.0));
    let b = hash2(i + vec2<f32>(1.0, 0.0));
    let c = hash2(i + vec2<f32>(0.0, 1.0));
    let d = hash2(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Domain-warped FBM for more organic, non-tiling clouds
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var p2 = p;
    var rot = mat2x2<f32>(0.8, 0.6, -0.6, 0.8); // Rotation to break grid patterns

    for (var i = 0; i < 4; i++) {
        value += amplitude * noise(p2);
        p2 = rot * p2 * 2.1 + vec2<f32>(1.7, 9.2); // Rotate and offset each octave
        amplitude *= 0.5;
    }
    return value;
}

// Additional turbulence layer for storm clouds
fn turbulence(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var p2 = p;

    for (var i = 0; i < 3; i++) {
        value += amplitude * abs(noise(p2));
        p2 = p2 * 2.2;
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

    // Rain and ambient dimming affect sky colors
    let rain = uniforms.rain_intensity;
    let dimming = uniforms.ambient_dimming;

    // Day/Night sky colors - desaturated and darker when rainy
    let day_top_clear = vec3<f32>(0.2, 0.4, 0.8);
    let day_top_rainy = vec3<f32>(0.25, 0.28, 0.35); // Grey overcast
    let day_top = mix(day_top_clear, day_top_rainy, rain) * (1.0 - dimming * 0.4);

    let day_horizon_clear = vec3<f32>(0.6, 0.7, 0.9);
    let day_horizon_rainy = vec3<f32>(0.4, 0.42, 0.48); // Muted horizon
    let day_horizon = mix(day_horizon_clear, day_horizon_rainy, rain) * (1.0 - dimming * 0.3);

    let night_top = vec3<f32>(0.002, 0.002, 0.005);
    let night_horizon = vec3<f32>(0.005, 0.005, 0.01);
    let sunset_horizon = vec3<f32>(0.9, 0.4, 0.2) * (1.0 - rain * 0.7); // Muted sunset in rain

    // Calculate day factor (0 = night, 1 = day)
    let day_factor = smoothstep(-0.2, 0.3, sun_elevation);

    // Sunset factor (peaks when sun is at horizon) - reduced in rain
    let sunset_factor = smoothstep(-0.3, 0.0, sun_elevation) * smoothstep(0.3, 0.0, sun_elevation) * (1.0 - rain * 0.8);

    // Interpolate top and horizon colors
    var top_color = mix(night_top, day_top, day_factor);
    var horizon_color = mix(night_horizon, day_horizon, day_factor);

    // Add sunset glow to horizon (reduced in overcast)
    horizon_color = mix(horizon_color, sunset_horizon, sunset_factor * 0.7);

    // Sky gradient based on ray direction
    let y = ray_dir.y * 0.5 + 0.5; // -1..1 to 0..1
    var sky_color = mix(horizon_color, top_color, pow(clamp(y, 0.0, 1.0), 0.5));

    // Stars at night - fixed on celestial sphere, no parallax
    if (day_factor < 0.4 && ray_dir.y > 0.05) {
        // Use spherical coordinates for stable star positions
        // This creates a fixed celestial sphere that doesn't shift with camera movement
        let phi = atan2(ray_dir.z, ray_dir.x); // azimuth angle
        let theta = acos(clamp(ray_dir.y, -1.0, 1.0)); // polar angle

        // Map to a grid on the celestial sphere
        let star_u = phi * 10.0; // ~60 cells around horizon
        let star_v = theta * 15.0; // cells from zenith to horizon
        let star_cell = vec2<f32>(star_u, star_v);

        let star_hash = hash(floor(star_cell));
        // Only 0.4% of cells have stars - sparse night sky
        let star_brightness = step(0.996, star_hash) * (1.0 - day_factor * 2.5);

        // Very slow, subtle twinkle - atmospheric shimmer
        let twinkle = sin(uniforms.time * 0.4 + star_hash * 30.0) * 0.15 + 0.85;

        // Dimmer, calmer stars
        let star_intensity = star_brightness * twinkle * 0.5;

        // Slight color variation - some stars warmer, some cooler
        let color_var = fract(star_hash * 7.3);
        let star_color = mix(
            vec3<f32>(0.9, 0.92, 1.0),   // Cool blue-white
            vec3<f32>(1.0, 0.95, 0.85),  // Warm white
            color_var
        );
        sky_color += star_color * star_intensity;
    }

    // Cloud Rendering - wispy clouds with storm support
    let cloud_height = 500.0;
    let storm_height = 300.0; // Lower, darker storm clouds

    // Only render clouds when looking up (ray_dir.y > 0)
    if (ray_dir.y > 0.01) {
        let t = cloud_height / ray_dir.y;
        let cloud_pos = ray_dir.xz * t;

        // Slower cloud movement, varied by layer
        let cloud_speed = 0.004;
        let time_offset = uniforms.time * cloud_speed;
        let wind = uniforms.wind_offset * 15.0 + vec2<f32>(time_offset, time_offset * 0.25);

        // Use spherical UV mapping to reduce tiling at horizon
        let dist_from_zenith = length(cloud_pos) * 0.0001;
        let angle = atan2(cloud_pos.y, cloud_pos.x);
        let spherical_uv = vec2<f32>(angle * 2.0, dist_from_zenith);

        // Blend between planar and spherical UV based on view angle
        let planar_uv = cloud_pos * 0.0015 * uniforms.cloud_scale + wind;
        let blend_factor = smoothstep(0.05, 0.4, ray_dir.y);
        let uv_scaled = mix(spherical_uv + wind * 0.5, planar_uv, blend_factor);

        // Multi-layered noise with domain warping for organic clouds
        let warp = vec2<f32>(fbm(uv_scaled * 0.5), fbm(uv_scaled * 0.5 + 7.3)) * 0.3;
        var n = fbm(uv_scaled + warp);
        let detail = fbm(uv_scaled * 2.5 + warp) * 0.25;
        n = (n * 0.5 + 0.5) + detail;

        // Storm cloud layer (lower, darker, more turbulent)
        var storm_n = 0.0;
        if (rain > 0.1) {
            let storm_t = storm_height / ray_dir.y;
            let storm_pos = ray_dir.xz * storm_t;
            let storm_uv = storm_pos * 0.003 + wind * 1.5;
            storm_n = turbulence(storm_uv) + fbm(storm_uv * 0.7) * 0.5;
            storm_n = storm_n * rain;
        }

        // Adjust threshold based on coverage - higher coverage = more clouds
        let threshold = 1.0 - uniforms.cloud_coverage;
        let cloud_alpha = smoothstep(threshold - 0.15, threshold + 0.2, n);
        let storm_alpha = smoothstep(0.2, 0.6, storm_n) * rain;

        let horizon_fade = smoothstep(0.01, 0.15, ray_dir.y);

        // Fade clouds at night (but don't completely hide them - moonlit clouds)
        let night_cloud_fade = mix(0.25, 1.0, day_factor);

        // Cloud opacity - higher in storms
        let base_opacity = mix(0.6, 0.85, rain);
        let density = cloud_alpha * uniforms.cloud_density * horizon_fade * night_cloud_fade * base_opacity;
        let storm_density = storm_alpha * horizon_fade * 0.9;

        if (density > 0.01 || storm_density > 0.01) {
            let color_mix = smoothstep(threshold, threshold + 0.35, n);

            // Cloud colors - darker and greyer in rain
            let clear_cloud_base = uniforms.cloud_color_base;
            let clear_cloud_shade = uniforms.cloud_color_shade;
            let storm_cloud_base = vec3<f32>(0.35, 0.38, 0.42); // Dark grey
            let storm_cloud_shade = vec3<f32>(0.2, 0.22, 0.25); // Darker underbelly

            let cloud_base = mix(clear_cloud_base, storm_cloud_base, rain);
            let cloud_shade = mix(clear_cloud_shade, storm_cloud_shade, rain);
            var cloud_rgb = mix(cloud_base, cloud_shade, color_mix);

            // Darken clouds at night
            let night_cloud_color = cloud_rgb * 0.15;
            cloud_rgb = mix(night_cloud_color, cloud_rgb, day_factor);

            // Reduced sun scattering in overcast (light diffused, not direct)
            let sun_dot = max(dot(ray_dir, -uniforms.sun_dir), 0.0);
            let sun_scatter = pow(sun_dot, 4.0) * 0.3 * day_factor * (1.0 - rain * 0.8);
            cloud_rgb += uniforms.sun_color * sun_scatter * (1.0 - density * 0.5);

            // Moon scattering through clouds
            let moon_dot = max(dot(ray_dir, -uniforms.moon_dir), 0.0);
            let moon_scatter = pow(moon_dot, 3.0) * 0.4 * (1.0 - day_factor);
            let moon_glow_color = vec3<f32>(0.5, 0.55, 0.7);
            cloud_rgb += moon_glow_color * moon_scatter * (1.0 - density * 0.3);

            // Highlights reduced in storm
            let highlight = smoothstep(0.75, 1.0, n) * (1.0 - rain * 0.6);
            let highlight_color = mix(vec3<f32>(0.2, 0.2, 0.28), vec3<f32>(0.95, 0.9, 0.85), day_factor);
            var final_cloud_color = mix(cloud_rgb, highlight_color, highlight * 0.35);

            // Apply storm cloud overlay
            if (storm_density > 0.01) {
                let storm_color = vec3<f32>(0.15, 0.17, 0.2) * day_factor + vec3<f32>(0.02, 0.02, 0.03);
                final_cloud_color = mix(final_cloud_color, storm_color, storm_density);
            }

            // Apply ambient dimming
            final_cloud_color = final_cloud_color * (1.0 - dimming * 0.3);

            let total_density = min(density + storm_density, 0.95);
            sky_color = mix(sky_color, final_cloud_color, total_density);
        }
    }

    // Apply overall dimming to sky
    sky_color = sky_color * (1.0 - dimming * 0.2);

    return vec4<f32>(sky_color, 1.0);
}
