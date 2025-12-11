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
    _padding: f32,       // Padding
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
}

// Tree wind animation - slower and more subtle than grass
fn apply_tree_wind(world_pos: vec3<f32>, local_height: f32, time: f32) -> vec3<f32> {
    // Wind is subtle for trees - they sway slowly
    let wind_strength = 0.08;
    let wind_direction = vec2<f32>(1.0, 0.3);

    // Slow sine waves for tree sway
    let wave1 = sin(time * 0.8 + world_pos.x * 0.1) * wind_strength;
    let wave2 = sin(time * 0.5 + world_pos.z * 0.15) * wind_strength * 0.6;

    // Height-based influence: trunk stays still, branches sway more
    // local_height is in model space (0 = base, higher = branches)
    let height_factor = saturate(local_height / 5.0); // Normalize to ~5 units
    let wind_amount = height_factor * height_factor; // Quadratic falloff

    let wind_offset = vec3<f32>(
        (wave1 + wave2) * wind_direction.x * wind_amount,
        0.0,
        (wave1 + wave2) * wind_direction.y * wind_amount
    );

    return world_pos + wind_offset;
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

    // Apply wind animation
    let animated_position = apply_tree_wind(world_position.xyz, local_height, camera.time);

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

    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
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
    let diffuse = pow(n_dot_l * 0.5 + 0.5, 2.0);

    // Ambient - moonlit night, greener for canopy
    let night_ambient = vec3<f32>(0.05, 0.07, 0.10); // Soft moonlit blue
    var day_ambient = vec3<f32>(0.20, 0.18, 0.15);
    if (is_canopy) {
        day_ambient = vec3<f32>(0.15, 0.22, 0.12); // Greener ambient for foliage
    }
    let ambient = mix(night_ambient, day_ambient, day_factor);

    // Sun color
    let sunrise_color = vec3<f32>(1.3, 0.6, 0.3);
    let midday_color = vec3<f32>(1.1, 1.05, 0.95);
    let sun_color = mix(sunrise_color, midday_color, saturate(sun_elevation * 2.0));

    let diffuse_strength = mix(0.1, 0.7, day_factor);
    let lighting = ambient + sun_color * diffuse * diffuse_strength * shadow;

    var final_color = base_color * lighting;

    // Apply distance fog
    let dist_to_camera = distance(in.world_position, camera.view_pos);
    let fog_factor = saturate((dist_to_camera - camera.fog_start) / (camera.fog_end - camera.fog_start));
    let fog_amount = fog_factor * fog_factor * camera.fog_density; // Quadratic falloff

    // Blend with fog color
    final_color = mix(final_color, camera.fog_color, fog_amount);

    return vec4<f32>(final_color, 1.0);
}
