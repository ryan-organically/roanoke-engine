// Terrain Shader - Atmospheric rendering with water animation and fog

struct Uniforms {
    view_proj: mat4x4<f32>,
    time: f32,
    padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    var world_pos = input.position;

    // WATER ANIMATION: Detect water vertices (blue channel > 0.5)
    if input.color.b > 0.5 {
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

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate surface normal from derivatives
    let dx = dpdx(input.world_pos);
    let dy = dpdy(input.world_pos);
    let normal = normalize(cross(dx, dy));

    // Afternoon sun direction
    let light_dir = vec3<f32>(0.5, -1.0, -0.5);

    // Simple diffuse lighting with ambient floor
    let lighting = max(dot(normal, -normalize(light_dir)), 0.1);

    // Apply lighting to color
    var lit_color = input.color * lighting;

    // FOG CALCULATION: Atlantic Grey atmosphere
    // Extract camera position from inverse view matrix (approximate from clip space)
    let view_depth = length(input.world_pos.xz - vec2<f32>(32.0, 32.0));
    let fog_density = 0.015;
    let fog_factor = 1.0 - exp(-view_depth * fog_density);
    let fog_color = vec3<f32>(0.6, 0.65, 0.7);

    // Mix lit color with fog
    let final_color = mix(lit_color, fog_color, fog_factor);

    return vec4<f32>(final_color, 1.0);
}
