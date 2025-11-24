// Grass Shader with Wind Animation

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
};

// Simple wind animation
// Uses sine waves based on world position for organic movement
fn apply_wind(world_pos: vec3<f32>, height_factor: f32) -> vec3<f32> {
    // Time simulation (we'll use world position as proxy for now)
    // In a real implementation, you'd pass time as a uniform
    let time = world_pos.x * 0.1 + world_pos.z * 0.1;

    // Wind direction and strength
    let wind_strength = 0.15;
    let wind_direction = vec2<f32>(1.0, 0.5);

    // Multiple sine waves for organic motion
    let wave1 = sin(time * 2.0 + world_pos.x * 0.5) * wind_strength;
    let wave2 = sin(time * 1.5 + world_pos.z * 0.7) * wind_strength * 0.5;

    // Only affect the top of the grass (based on height_factor)
    let wind_amount = height_factor * height_factor; // Quadratic falloff

    // Apply wind offset
    let wind_offset = vec3<f32>(
        (wave1 + wave2) * wind_direction.x * wind_amount,
        0.0,
        (wave1 + wave2) * wind_direction.y * wind_amount
    );

    return world_pos + wind_offset;
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate height factor (0 at base, 1 at tip)
    // Assumes grass blades are oriented vertically
    let base_height = 0.0; // You could pass this as a uniform
    let height_factor = saturate(vertex.position.y / 1.0); // Normalize to ~1.0 max height

    // Apply wind animation
    let animated_position = apply_wind(vertex.position, height_factor);

    out.clip_position = camera.view_proj * vec4<f32>(animated_position, 1.0);
    out.color = vertex.color;
    out.world_position = animated_position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple lighting based on height
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let normal = vec3<f32>(0.0, 1.0, 0.0); // Simplified normal (pointing up)

    let ambient = 0.4;
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.6;
    let lighting = ambient + diffuse;

    let final_color = in.color * lighting;

    return vec4<f32>(final_color, 1.0);
}
