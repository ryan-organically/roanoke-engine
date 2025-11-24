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
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) shadow_pos: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    var world_pos = input.position;

    // WATER ANIMATION: Detect water vertices (blue channel > 0.5)
    // Adjust threshold or logic if needed based on new colors
    // Water is Turquoise [0.2, 0.8, 0.8] or Teal [0.05, 0.3, 0.4]
    // Both have high Blue component relative to Red.
    // Sand is [0.92, 0.90, 0.85]. Blue is high there too!
    // We need a better way to detect water.
    // Height check? Water is below 0.0 (or -0.5).
    // Let's use world_pos.y.
    if world_pos.y < 0.0 {
        // Ocean breathing - dual wave system
        let wave1 = sin(world_pos.x * 0.2 + uniforms.time * 1.5) * 0.3;
        let wave2 = cos(world_pos.z * 0.15 + uniforms.time * 1.2) * 0.25;
        world_pos.y += wave1 + wave2;
    }

    // Transform position by view-projection matrix
    output.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);

    // Pass through color and world position
    output.color = input.color;
    output.world_pos = world_pos;

    // Calculate shadow position
    // Transform world position to light space
    // Range [-1, 1] -> [0, 1] for texture sampling
    let pos_from_light = uniforms.light_view_proj * vec4<f32>(world_pos, 1.0);
    
    // Convert to texture coordinates
    // Flip Y because texture coordinates are top-down
    output.shadow_pos = vec3<f32>(
        pos_from_light.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5),
        pos_from_light.z
    );

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate normal from world position derivatives (flat shading)
    let dx = dpdx(input.world_pos);
    let dy = dpdy(input.world_pos);
    let normal = -normalize(cross(dx, dy)); // Flip normal to point up

    // Sun Direction from Uniforms
    let light_dir = normalize(uniforms.sun_dir);

    // Warm Golden Sunlight
    let sun_color = vec3<f32>(1.0, 0.98, 0.9); 
    let ambient_color = vec3<f32>(0.3, 0.35, 0.4); // Blue-ish shadows

    // Diffuse lighting
    let diff = max(dot(normal, -light_dir), 0.0);
    
    // Shadow Calculation (DISABLED FOR DEBUGGING)
    let shadow = 1.0;

    // Apply shadow to sun color only
    let lighting = ambient_color + (sun_color * diff * shadow);

    // Apply lighting to surface color
    var final_color = input.color * lighting;

    // Water Specular Highlight
    // Check if it's water (Blue > Red and Blue > Green significantly, or just height check if passed)
    // We used height check in VS to animate. We can infer water if color is blue-ish.
    // Water colors: [0.05, 0.3, 0.4] to [0.2, 0.8, 0.8]
    if (input.color.b > input.color.r + 0.1) {
        let view_dir = normalize(uniforms.view_pos - input.world_pos);
        let reflect_dir = reflect(-light_dir, normal);
        let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
        let specular = 0.8 * spec * sun_color * shadow; // Sun reflection
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
