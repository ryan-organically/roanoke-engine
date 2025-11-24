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
    let light_dir = normalize(uniforms.sun_dir);

    // Warm Golden Sunlight for Sunrise - VERY BRIGHT
    let sun_color = vec3<f32>(1.5, 1.1, 0.7); // Intense warm sunrise
    let ambient_color = vec3<f32>(0.08, 0.10, 0.15); // Very low ambient for maximum contrast

    // Diffuse lighting - strong directional sun
    let diff = max(dot(normal, -light_dir), 0.0);

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
        // Make shadows MUCH darker: 1.0 = lit, 0.2 = deep shadow
        shadow = shadow * 0.8 + 0.2;
    }

    // Apply shadow to sun color only - EXTREMELY dramatic sunrise lighting
    // Very high multiplier (3.5) to create intense highlights showing clear direction
    let lighting = ambient_color + (sun_color * diff * 3.5 * shadow);

    // Apply lighting to surface color
    var final_color = input.color * lighting;

    // Water Specular Highlight (Sun Sparkle)
    if (is_water) {
        let view_dir = normalize(uniforms.view_pos - input.world_pos);
        let reflect_dir = reflect(-light_dir, normal);

        // Tighter specular for sharp sparkles
        let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 64.0);

        // Brighter sparkles for distance visibility
        let specular = 1.5 * spec * sun_color * shadow;
        final_color += specular;
    }

    // FOG CALCULATION (Atmospheric Haze)
    // Distance from Camera
    let dist = distance(input.world_pos, uniforms.view_pos);
    
    // Linear Fog
    let fog_factor = clamp((dist - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    
    // Mix lit color with fog
    final_color = mix(final_color, uniforms.fog_color, fog_factor);

    return vec4<f32>(final_color, 1.0);
}
