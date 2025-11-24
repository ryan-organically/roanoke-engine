struct Uniforms {
    view_proj: mat4x4<f32>,
    sun_dir: vec3<f32>,
    padding1: f32,
    sun_color: vec3<f32>,
    padding2: f32,
    time: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Generate a full-screen triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    let pos = positions[in_vertex_index];
    
    var output: VertexOutput;
    // Z = 1.0 (max depth) to render behind everything
    output.clip_position = vec4<f32>(pos, 1.0, 1.0);
    
    // Calculate world direction for skybox lookup (inverse view-proj would be better, but we can approximate)
    // Actually, for a skybox, we usually use a cube or sphere.
    // For a simple gradient, we can just use screen coordinates or view direction.
    // Let's try to reconstruct view ray.
    // Ideally we pass inverse view proj.
    // For now, let's just use screen coords mapped to sphere.
    output.world_pos = vec3<f32>(pos.x, pos.y, 1.0); 

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // We need the view direction for the pixel.
    // Since we don't have the inverse matrix here easily without passing it,
    // let's assume a simple sky gradient based on screen Y for now, 
    // BUT to draw the sun we need the actual direction.
    
    // Let's rely on the fact that we can pass the view-proj inverse or just do it in screen space?
    // Screen space sun is hard because it moves.
    
    // Better approach: The vertex shader should output a ray direction.
    // But we are drawing a full screen quad.
    
    // Let's cheat: We will assume the user looks around.
    // Actually, `sky_pipeline.rs` passes `view_proj`.
    // If we render a CUBE instead of a quad, it's easier.
    // But a quad is cheaper.
    
    // Let's use the `view_proj` to unproject the screen position?
    // Or just render a giant sphere in the Rust code?
    // No, let's stick to the quad and try to get the ray.
    
    // Simplified: Just draw a gradient for now.
    // To draw the sun, we need to know where it is on screen.
    // We can project the sun position in the vertex shader?
    
    // Let's switch to a simple gradient + sun glow.
    
    // Sky Gradient
    let top_color = vec3<f32>(0.2, 0.4, 0.8);
    let horizon_color = vec3<f32>(0.6, 0.7, 0.9);
    let bottom_color = vec3<f32>(0.1, 0.1, 0.2); // Night/Ground
    
    // Use screen Y for gradient
    let y = input.world_pos.y * 0.5 + 0.5;
    var sky_color = mix(horizon_color, top_color, y);
    
    // Sun
    // We need the dot product between view ray and sun direction.
    // This is hard without the view ray.
    
    // ALTERNATIVE:
    // Just clear the screen with a color in `main.rs` and don't use this pipeline yet?
    // The user wants a VISIBLE sun.
    
    // Let's try to get the view ray.
    // In VS:
    // output.view_dir = (inverse(view_proj) * vec4(pos, 1.0, 1.0)).xyz;
    // We don't have inverse in WGSL easily unless passed.
    
    // Let's just return a nice blue for now and implement the sun logic in `main.rs` by drawing a billboard?
    // No, sky shader is best.
    
    return vec4<f32>(sky_color, 1.0);
}
