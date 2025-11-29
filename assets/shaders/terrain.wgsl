// Terrain Shader - Atmospheric rendering with water animation and fog

struct Uniforms {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    fog_color: vec3<f32>,
    time: f32,
    fog_start: f32,
    fog_end: f32,
    padding1: vec2<f32>,
    sun_dir: vec3<f32>,
    padding2: f32,
    view_pos: vec3<f32>,
    padding3: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) shadow_pos: vec3<f32>,
    @location(3) normal: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    var world_pos = input.position;

    // WATER ANIMATION with shore breaking
    // Water is below y = 0.5 (includes shallow water)
    if world_pos.y < 0.5 {
        // Calculate depth factor (0 = shore, 1 = deep water)
        let depth_factor = clamp((0.5 - world_pos.y) / 5.0, 0.0, 1.0);

        // Deep ocean waves (rolling)
        let wave1 = sin(world_pos.x * 0.2 + uniforms.time * 1.5) * 0.3;
        let wave2 = cos(world_pos.z * 0.15 + uniforms.time * 1.2) * 0.25;
        let deep_waves = (wave1 + wave2) * depth_factor;

        // Shore waves (breaking - faster, directional toward shore)
        let shore_factor = 1.0 - depth_factor;
        // Waves move toward shore (negative X direction based on Roanoke's east-facing ocean)
        let breaking_wave = sin(world_pos.x * 0.8 + uniforms.time * 3.0) * 0.15 * shore_factor;

        // Combine: deep rolling waves + shore breaking waves
        world_pos.y += deep_waves + breaking_wave;
    }

    // Transform position by view-projection matrix
    output.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);

    // Pass through color, world position, and normal
    output.color = input.color;
    output.world_pos = world_pos;
    output.normal = input.normal;

    // Calculate shadow position
    // Transform world position to light space
    let pos_from_light = uniforms.light_view_proj * vec4<f32>(world_pos, 1.0);

    // Perspective divide to get NDC coordinates
    let shadow_ndc = pos_from_light.xyz / pos_from_light.w;

    // Convert NDC [-1, 1] to texture coordinates [0, 1]
    // Flip Y because texture coordinates are top-down
    output.shadow_pos = vec3<f32>(
        shadow_ndc.x * 0.5 + 0.5,
        -shadow_ndc.y * 0.5 + 0.5,
        shadow_ndc.z
    );

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // For water, calculate normals from derivatives to capture wave sparkle
    // For land, use smooth interpolated normals
    var normal: vec3<f32>;
    let is_water = input.world_pos.y < 0.5; // Water is below this height

    if (is_water) {
        // Calculate normal from world position derivatives for dynamic waves
        let dx = dpdx(input.world_pos);
        let dy = dpdy(input.world_pos);
        normal = normalize(cross(dx, dy));
    } else {
        // Use smooth interpolated normal for terrain
        normal = normalize(input.normal);
    }

    // Sun Direction from Uniforms
    // sun_dir points FROM the sun TO the scene (direction light travels)
    let light_dir = normalize(uniforms.sun_dir);

    // Dynamic sun color based on sun elevation (y component of light direction)
    // When sun is low (horizon), warm orange. When high, bright white-yellow.
    let sun_elevation = -light_dir.y; // Higher = sun is higher in sky
    let sun_color = mix(
        vec3<f32>(1.8, 0.6, 0.2),  // Sunrise/sunset: warm orange
        vec3<f32>(1.4, 1.3, 1.1),  // Midday: bright white-yellow
        clamp(sun_elevation * 2.0, 0.0, 1.0)
    );

    // Ambient also shifts - bluer at midday, warmer at sunrise/sunset
    let ambient_color = mix(
        vec3<f32>(0.15, 0.10, 0.08), // Sunrise: warm ambient
        vec3<f32>(0.12, 0.14, 0.18), // Midday: cool sky ambient
        clamp(sun_elevation * 2.0, 0.0, 1.0)
    );

    // Diffuse lighting - use the direction light is coming FROM (negate light_dir)
    // light_dir points toward scene, so -light_dir points toward light source
    let n_dot_l = max(dot(normal, -light_dir), 0.0);
    let diff = n_dot_l;

    // Shadow Calculation - Sample shadow map
    // Use the pre-calculated shadow position from vertex shader (already in texture space)
    // The vertex shader already did the light_view_proj transform and texture coordinate conversion
    let shadow_uv = input.shadow_pos.xy;
    let shadow_depth = input.shadow_pos.z;

    var shadow = 1.0;
    var in_shadow_map = false;

    // Only sample if within shadow map bounds
    if (shadow_uv.x >= 0.0 && shadow_uv.x <= 1.0 &&
        shadow_uv.y >= 0.0 && shadow_uv.y <= 1.0 &&
        shadow_depth >= 0.0 && shadow_depth <= 1.0) {
        in_shadow_map = true;
        // Use comparison sampler for PCF (Percentage Closer Filtering)
        // NO bias in shader - rely entirely on hardware depth bias
        shadow = textureSampleCompare(t_shadow, s_shadow, shadow_uv, shadow_depth);
        // Make shadows MUCH darker: 1.0 = lit, 0.1 = deep shadow (Increased contrast)
        shadow = shadow * 0.9 + 0.1;
    }

    // Rim Lighting (Fresnel-like effect for terrain definition)
    let view_dir_to_cam = normalize(uniforms.view_pos - input.world_pos);
    let rim_dot = 1.0 - max(dot(view_dir_to_cam, normal), 0.0);
    let rim = pow(rim_dot, 4.0) * 0.3 * sun_color * shadow;

    // Apply shadow to sun color only
    // Multiplier adjusted for more natural look
    let diffuse_contribution = sun_color * diff * 1.3 * shadow; // Increased intensity
    let lighting = ambient_color + diffuse_contribution + rim;

    // Apply lighting to surface color
    var final_color = input.color * lighting;

    // Water Specular Highlight (Sun Sparkle)
    if (is_water) {
        let reflect_dir = reflect(-light_dir, normal);

        // Tighter specular for sharp sparkles
        let spec = pow(max(dot(view_dir_to_cam, reflect_dir), 0.0), 64.0);

        // Brighter sparkles for distance visibility
        let specular = 1.8 * spec * sun_color * shadow;
        final_color += specular;
    }

    // FOG CALCULATION (Atmospheric Haze)
    // Distance from Camera
    let dist = distance(input.world_pos, uniforms.view_pos);
    
    // FOG CALCULATION (Atmospheric Volumetric Fog)
    
    // 1. Distance Fog (Base)
    let dist_factor = clamp((dist - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    
    // 2. Height Fog (Denser at sea level, thins out upwards)
    let fog_height_falloff = 40.0; // Height where fog disappears
    let height_factor = 1.0 - clamp((input.world_pos.y + 5.0) / fog_height_falloff, 0.0, 1.0);
    let height_fog = height_factor * height_factor; // Quadratic falloff for "settled" look

    // Combine Fog Density
    // Distance fog is always present, height fog adds density in low areas
    let fog_density = clamp(dist_factor + height_fog * 0.6, 0.0, 1.0);

    // 3. Sun Scattering (Glow when looking at sun)
    let view_dir = normalize(input.world_pos - uniforms.view_pos);
    let sun_dot = max(dot(view_dir, normalize(uniforms.sun_dir)), 0.0);
    let sun_scatter = pow(sun_dot, 16.0); // Sharp glow near sun
    
    // Fog Color with Scattering
    // Mix base fog color with a warm sun tint based on scatter
    let scatter_color = vec3<f32>(1.0, 0.9, 0.7); // Warm sunlight
    let final_fog_color = mix(uniforms.fog_color, scatter_color, sun_scatter * 0.5);

    // Apply Fog
    final_color = mix(final_color, final_fog_color, fog_density);

    return vec4<f32>(final_color, 1.0);
}
