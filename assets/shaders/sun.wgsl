// Sun Billboard Shader - renders a sun disk in the sky

struct Uniforms {
    view_proj: mat4x4<f32>,
    sun_world_pos: vec3<f32>,
    sun_size: f32,
    sun_color: vec3<f32>,
    _padding: f32,
    camera_right: vec3<f32>,
    _padding2: f32,
    camera_up: vec3<f32>,
    _padding3: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Billboard quad vertices (two triangles)
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate quad vertices from index
    // 0: (-1, -1), 1: (1, -1), 2: (-1, 1), 3: (1, 1)
    // Triangles: 0-1-2, 2-1-3
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    let pos_2d = positions[vertex_index];

    // Billboard in world space - offset from sun position using camera basis vectors
    let world_pos = uniforms.sun_world_pos 
        + uniforms.camera_right * pos_2d.x * uniforms.sun_size 
        + uniforms.camera_up * pos_2d.y * uniforms.sun_size;

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = pos_2d * 0.5 + 0.5; // Convert -1,1 to 0,1

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from center of quad
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center) * 2.0; // 0 at center, 1 at edge

    // Sun disk with soft glow
    // Core: bright white/yellow, sharp edge
    // Corona: soft orange glow fading out

    let core_radius = 0.3;
    let corona_radius = 1.0;

    if dist < core_radius {
        // Bright core - white to yellow
        let core_blend = dist / core_radius;
        let core_color = mix(
            vec3<f32>(1.0, 1.0, 0.95),  // White center
            uniforms.sun_color,          // Sun color at edge
            core_blend * core_blend
        );
        return vec4<f32>(core_color, 1.0);
    } else if dist < corona_radius {
        // Corona glow - Soft exponential falloff
        let corona_blend = (dist - core_radius) / (corona_radius - core_radius);
        // Use exponential falloff for a "glowing" look rather than linear/quadratic
        let glow = exp(-corona_blend * 4.0); 
        let corona_color = uniforms.sun_color;
        return vec4<f32>(corona_color, glow * 0.8);
    } else {
        // Outside sun
        discard;
    }
}
