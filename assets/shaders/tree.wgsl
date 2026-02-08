struct CameraUniform {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,  // Shadow map transform
    sun_dir: vec3<f32>,
    time: f32,           // For wind animation
    view_pos: vec3<f32>, // Camera position for fog distance
    fog_density: f32,    // Fog intensity
    fog_color: vec3<f32>,// Fog color
    fog_start: f32,      // Fog start distance
    fog_end: f32,        // Fog end distance
    alpha_cutoff: f32,   // Alpha cutoff for masked rendering
    use_texture: f32,    // 1.0 = sample texture, 0.0 = procedural
    // LOD dither fade parameters
    lod_fade_start: f32, // Distance where fade begins
    lod_fade_end: f32,   // Distance where fade ends
    lod_fade_mode: f32,  // 0.0 = disabled, 1.0 = LOD0 (fade out), 2.0 = LOD1 (fade in)
    wind_enabled: f32,   // 1.0 = wind on, 0.0 = wind off (for boulders)
    // Campfire lights for canopy illumination (xyz = position, w = intensity)
    campfire_lights: array<vec4<f32>, 4>,
    campfire_count: u32,
    _padding2: vec3<f32>, // alignment padding
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var t_shadow: texture_depth_2d;
@group(0) @binding(2)
var s_shadow: sampler_comparison;

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
    @location(3) local_height: f32, // Height in local space for bark gradient
    @location(4) shadow_pos: vec3<f32>, // Shadow map position
    @location(5) distance_to_camera: f32, // For LOD dither fade
}

// Tree/foliage wind animation
// CRITICAL: Wind must ONLY affect HIGH local_height (tips), NEVER low (bases)
// local_height = vertex Y in model space (0 = base, higher = tips/canopy)
fn apply_tree_wind(world_pos: vec3<f32>, local_height: f32, time: f32) -> vec3<f32> {
    // If this vertex is at the base (low local_height), no wind at all
    // This excludes boulder bases and tree trunks
    if (local_height < 0.2) {
        return world_pos;
    }

    // Height-based influence: tips move most
    let grass_factor = saturate(local_height / 1.2); // Grass tips at 1-2 units
    let tree_factor = saturate(local_height / 5.0);  // Tree canopy at 5+ units

    // Quadratic falloff
    let grass_wind = grass_factor * grass_factor;
    let tree_wind = tree_factor * tree_factor;

    // Determine if this is beach-level based on world position
    let is_beach = world_pos.y < 10.0;

    // BOULDER EXCLUSION: At beach level, skip wind for solid objects
    // Boulders have vertices at low-to-moderate local_height (0.2-2.0) sitting at ground level
    // Only apply beach wind to grass tips (local_height 0.5-2.5) or tree canopy (3.0+)
    if (is_beach && local_height < 0.5) {
        return world_pos; // Skip - likely boulder or solid object base
    }

    // Spatial coherence: nearby grass moves together with slight offset
    let cell_size = 8.0; // 8 meter cells for larger, more spread out wave motion
    let cell_x = floor(world_pos.x / cell_size);
    let cell_z = floor(world_pos.z / cell_size);
    let cell_phase = fract(sin(cell_x * 12.9898 + cell_z * 78.233) * 43758.5453) * 6.28;
    let local_offset = fract(sin(world_pos.x * 0.5 + world_pos.z * 0.3) * 1000.0) * 0.4;

    // Base wind parameters
    let wind_direction = vec2<f32>(1.0, 0.3);
    var wind_strength = 0.08;
    var time_mult = 1.0;
    var wind_amount = tree_wind;

    if (is_beach) {
        if (local_height < 3.0) {
            // Beach grass: gentle but visible swaying
            wind_amount = grass_wind;
            wind_strength = 0.7;  // Moderate - visible but gentle
            time_mult = 2.0;      // Slower, more graceful motion
        } else {
            // Beach tree canopy: strong coastal wind
            wind_amount = tree_wind;
            wind_strength = 0.22;
            time_mult = 1.5;
        }
    }

    // Skip if wind amount is negligible
    if (wind_amount < 0.01) {
        return world_pos;
    }

    // Wind waves with spatial coherence
    let anim_time = time * time_mult;
    let coherent_time = anim_time + cell_phase + local_offset;

    // Multiple wave frequencies for organic whipping motion
    let wave1 = sin(coherent_time * 1.2 + world_pos.x * 0.06) * wind_strength;
    let wave2 = sin(coherent_time * 0.8 + world_pos.z * 0.08) * wind_strength * 0.5;

    // Beach-specific gentle swaying with subtle variation
    var whip = 0.0;
    var gust_mult = 1.0;
    if (is_beach && local_height < 3.0) {
        // Gentle swaying - lower frequency, softer motion
        let sway1 = sin(coherent_time * 2.0) * wind_strength * 0.25;
        let sway2 = sin(coherent_time * 1.5 + 1.0) * sin(coherent_time * 1.0) * wind_strength * 0.15;
        whip = sway1 + sway2;

        // Gentle sweeping gusts across the beach
        let gust_wave = sin(time * 0.15 + cell_x * 0.05 + cell_z * 0.03);
        gust_mult = 1.0 + max(gust_wave, 0.0) * 0.6; // Gusts up to 1.6x
    }

    let total_wave = (wave1 + wave2 + whip) * gust_mult;

    let wind_offset = vec3<f32>(
        total_wave * wind_direction.x * wind_amount,
        0.0,
        total_wave * wind_direction.y * wind_amount
    );

    return world_pos + wind_offset;
}

// 4x4 Bayer dither matrix for LOD transitions
// Returns threshold value 0.0-1.0 based on screen position
// Using Bayer pattern for temporally stable dithering (no flickering)
// Pattern:  0  8  2 10
//          12  4 14  6
//           3 11  1  9
//          15  7 13  5
fn dither_threshold(screen_pos: vec2<f32>) -> f32 {
    let x = u32(screen_pos.x) % 4u;
    let y = u32(screen_pos.y) % 4u;
    let idx = y * 4u + x;

    // WGSL requires constant array indices, so use switch instead
    switch idx {
        case 0u:  { return 0.0 / 16.0; }
        case 1u:  { return 8.0 / 16.0; }
        case 2u:  { return 2.0 / 16.0; }
        case 3u:  { return 10.0 / 16.0; }
        case 4u:  { return 12.0 / 16.0; }
        case 5u:  { return 4.0 / 16.0; }
        case 6u:  { return 14.0 / 16.0; }
        case 7u:  { return 6.0 / 16.0; }
        case 8u:  { return 3.0 / 16.0; }
        case 9u:  { return 11.0 / 16.0; }
        case 10u: { return 1.0 / 16.0; }
        case 11u: { return 9.0 / 16.0; }
        case 12u: { return 15.0 / 16.0; }
        case 13u: { return 7.0 / 16.0; }
        case 14u: { return 13.0 / 16.0; }
        case 15u: { return 5.0 / 16.0; }
        default: { return 0.0; }
    }
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

    // Store local height before transformation
    let local_height = input.position.y;
    output.local_height = local_height;

    let world_position = model_matrix * vec4<f32>(input.position, 1.0);

    // Apply wind animation only if enabled (disabled for boulders/static objects)
    var animated_position = world_position.xyz;
    if (camera.wind_enabled > 0.5) {
        animated_position = apply_tree_wind(world_position.xyz, local_height, camera.time);
    }

    output.clip_position = camera.view_proj * vec4<f32>(animated_position, 1.0);
    output.world_position = animated_position;

    // Transform normal (assuming uniform scaling, otherwise need normal matrix)
    output.world_normal = (model_matrix * vec4<f32>(input.normal, 0.0)).xyz;
    output.uv = input.uv;

    // Calculate shadow position
    let pos_from_light = camera.light_view_proj * vec4<f32>(animated_position, 1.0);
    let shadow_ndc = pos_from_light.xyz / pos_from_light.w;
    output.shadow_pos = vec3<f32>(
        shadow_ndc.x * 0.5 + 0.5,
        -shadow_ndc.y * 0.5 + 0.5,
        shadow_ndc.z
    );

    // Calculate distance for LOD fade
    output.distance_to_camera = distance(animated_position, camera.view_pos);

    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //=========================================================================
    // LOD DITHER FADE
    //=========================================================================
    // lod_fade_mode: 0.0 = disabled, 1.0 = LOD0 (fade out), 2.0 = LOD1 (fade in)
    if (camera.lod_fade_mode > 0.5) {
        // Calculate fade factor (0 = fully visible, 1 = fully faded)
        let fade_range = camera.lod_fade_end - camera.lod_fade_start;
        if (fade_range > 0.001) {
            let fade_t = saturate((in.distance_to_camera - camera.lod_fade_start) / fade_range);
            let threshold = dither_threshold(in.clip_position.xy);

            if (camera.lod_fade_mode < 1.5) {
                // LOD0: fade OUT as distance increases
                // Discard more pixels as we get farther
                if (fade_t > threshold) {
                    discard;
                }
            } else {
                // LOD1: fade IN as distance increases
                // Discard more pixels as we get closer (inverted)
                if ((1.0 - fade_t) > threshold) {
                    discard;
                }
            }
        }
    }

    // Sample texture
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);

    // Alpha discard for masked materials (leaves, etc.)
    if (camera.alpha_cutoff > 0.0 && tex_color.a < camera.alpha_cutoff) {
        discard;
    }

    var base_color: vec3<f32>;

    // Determine if this is trunk (uv.y <= 1.0) or canopy (uv.y > 1.0)
    // This heuristic works for both textured and procedural trees
    let is_canopy = in.uv.y > 1.0;

    // Use texture color if enabled, otherwise procedural
    if (camera.use_texture > 0.5) {
        base_color = tex_color.rgb;
    } else {
        // Procedural fallback for L-system trees
        // Multi-octave noise for organic variation
        let noise = fract(sin(dot(in.world_position.xz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        let noise2 = fract(sin(dot(in.world_position.xy * 0.5, vec2<f32>(39.346, 11.135))) * 43758.5453);
        let noise3 = fract(sin(dot(in.world_position.yz * 0.3, vec2<f32>(71.231, 28.947))) * 43758.5453);
        let detail_noise = noise * 0.5 + noise2 * 0.3 + noise3 * 0.2;

        if (is_canopy) {
            // CANOPY: Rich forest greens with seasonal variation
            let leaf_deep = vec3<f32>(0.08, 0.22, 0.06);    // Deep shadow green
            let leaf_dark = vec3<f32>(0.12, 0.32, 0.10);    // Dark forest green
            let leaf_mid = vec3<f32>(0.18, 0.42, 0.14);     // Mid green
            let leaf_light = vec3<f32>(0.28, 0.52, 0.20);   // Sunlit green
            let leaf_highlight = vec3<f32>(0.35, 0.58, 0.22); // Bright highlight

            // Create layered color mixing for depth
            let base_green = mix(leaf_dark, leaf_mid, detail_noise);
            let varied_green = mix(base_green, leaf_light, noise2 * 0.4);

            // Add subtle highlight clusters
            let highlight_factor = smoothstep(0.7, 0.9, noise3);
            base_color = mix(varied_green, leaf_highlight, highlight_factor * 0.3);

            // Normal-based shading: undersides darker, tops brighter
            let top_factor = saturate(in.world_normal.y * 0.5 + 0.5);
            let underside_dark = mix(leaf_deep, base_color, 0.6);
            base_color = mix(underside_dark, base_color * 1.15, top_factor);

            // Subtle color temperature shift based on position
            let warm_shift = vec3<f32>(0.02, -0.01, -0.02) * (noise - 0.5);
            base_color = base_color + warm_shift;
        } else {
            // TRUNK: Rich bark with texture-like variation
            let bark_deep = vec3<f32>(0.15, 0.08, 0.04);   // Deep bark shadow
            let bark_dark = vec3<f32>(0.28, 0.18, 0.10);   // Dark bark
            let bark_mid = vec3<f32>(0.38, 0.26, 0.15);    // Mid bark
            let bark_light = vec3<f32>(0.48, 0.34, 0.20);  // Light bark

            // Vertical streaking for bark texture effect
            let streak = fract(sin(in.world_position.x * 5.0 + in.world_position.z * 3.0) * 43758.5453);
            let bark_base = mix(bark_dark, bark_mid, detail_noise * 0.7);
            let bark_varied = mix(bark_base, bark_light, streak * 0.35);

            // Height-based variation (moss/lichen at base, drier higher up)
            let height_factor = saturate(in.local_height / 8.0);
            let base_tint = vec3<f32>(0.22, 0.20, 0.12); // Slight green-brown at base
            base_color = mix(mix(bark_varied, base_tint, 0.15), bark_varied, height_factor);

            // Add depth in crevices based on normal
            let facing_factor = abs(in.world_normal.x) + abs(in.world_normal.z);
            base_color = mix(base_color, bark_deep, facing_factor * 0.2 * (1.0 - height_factor));
        }
    }

    // Beach boulder tint: deep burnt red for rocks at low world Y (beach level)
    // Only tint very low objects (Y < 3m) to avoid affecting grass blades
    // Boulders sit at Y 0.5-3m, grass extends from 2m terrain to 6m+ tips
    let beach_tint_factor = 1.0 - saturate((in.world_position.y - 0.5) / 2.5);
    if (beach_tint_factor > 0.01) {
        // Deep burnt red - strongly tint the texture toward this color
        let burnt_red = vec3<f32>(0.35, 0.15, 0.08);
        // Strong blend for visibility on textures
        let tinted = mix(base_color, base_color * burnt_red * 3.0, beach_tint_factor * 0.8);
        base_color = tinted;
    }

    // Lighting (same for both trunk and canopy)
    let light_dir = normalize(camera.sun_dir);
    let sun_elevation = -light_dir.y;
    let day_factor = smoothstep(-0.1, 0.3, sun_elevation);

    // Shadow calculation
    let shadow_uv = in.shadow_pos.xy;
    let shadow_depth = in.shadow_pos.z;

    var shadow = 1.0;
    if (shadow_uv.x >= 0.0 && shadow_uv.x <= 1.0 &&
        shadow_uv.y >= 0.0 && shadow_uv.y <= 1.0 &&
        shadow_depth >= 0.0 && shadow_depth <= 1.0) {
        shadow = textureSampleCompare(t_shadow, s_shadow, shadow_uv, shadow_depth);
        // Softer shadows for trees: 1.0 = lit, 0.25 = shadow
        shadow = shadow * 0.75 + 0.25;
    }

    // Diffuse with half-lambert for softer shadows
    let n_dot_l = dot(normalize(in.world_normal), -light_dir);
    let hl = n_dot_l * 0.5 + 0.5;
    let diffuse = hl * hl;

    // Bright ambient - grass and foliage catch lots of sky light
    let night_ambient = vec3<f32>(0.08, 0.10, 0.14); // Soft moonlit blue
    var day_ambient = vec3<f32>(0.45, 0.42, 0.35); // Bright base ambient
    if (is_canopy) {
        day_ambient = vec3<f32>(0.40, 0.50, 0.32); // Greener/brighter for foliage
    }
    let ambient = mix(night_ambient, day_ambient, day_factor);

    // Sun color - warm and strong
    let sunrise_color = vec3<f32>(1.4, 0.7, 0.4);
    let midday_color = vec3<f32>(1.2, 1.15, 1.05);
    let sun_color = mix(sunrise_color, midday_color, saturate(sun_elevation * 2.0));

    // Strong diffuse during day, shadow provides contrast
    let diffuse_strength = mix(0.15, 0.9, day_factor);
    var lighting = ambient + sun_color * diffuse * diffuse_strength * shadow;

    // CAMPFIRE LIGHTS - Flickering warm light on canopy (especially visible at night)
    let normal = normalize(in.world_normal);
    for (var i = 0u; i < camera.campfire_count; i++) {
        let light = camera.campfire_lights[i];
        let light_pos = light.xyz;
        let light_intensity = light.w;

        if (light_intensity > 0.0) {
            let light_to_surface = in.world_position - light_pos;
            let light_dist = length(light_to_surface);
            let light_dir_cf = light_to_surface / max(light_dist, 0.001);

            // Campfire light properties - larger radius for canopy (35m)
            let campfire_radius = 35.0;
            let attenuation = 1.0 / (1.0 + light_dist * 0.08 + light_dist * light_dist * 0.004);
            let range_falloff = max(0.0, 1.0 - light_dist / campfire_radius);

            // Diffuse lighting from campfire
            let cf_n_dot_l = max(dot(normal, -light_dir_cf), 0.0);

            // Warm orange campfire color
            let campfire_color = vec3<f32>(1.0, 0.55, 0.2);

            // Campfire light is stronger at night
            let night_boost = mix(1.0, 3.0, 1.0 - day_factor);

            // Canopy gets more light (upward-facing surfaces catch more firelight)
            var canopy_boost = 1.0;
            if (is_canopy) {
                // Leaves facing down toward fire get extra illumination
                let down_factor = max(0.0, -normal.y);
                canopy_boost = 1.0 + down_factor * 2.0;
            }

            // Apply campfire lighting - strong contribution for visible flickering
            let contribution = campfire_color * cf_n_dot_l * attenuation * range_falloff * light_intensity * night_boost * canopy_boost * 12.0;
            lighting += contribution;
        }
    }

    var final_color = base_color * lighting;

    // Apply distance fog
    let dist_to_camera = distance(in.world_position, camera.view_pos);
    let fog_factor = saturate((dist_to_camera - camera.fog_start) / (camera.fog_end - camera.fog_start));
    let fog_amount = fog_factor * fog_factor * camera.fog_density; // Quadratic falloff

    // Blend with fog color
    final_color = mix(final_color, camera.fog_color, fog_amount);

    return vec4<f32>(final_color, 1.0);
}
