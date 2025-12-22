// Moon Billboard Shader - Screen-space circle (no FOV distortion)

struct Uniforms {
    view_proj: mat4x4<f32>,
    moon_world_pos: vec3<f32>,
    moon_size: f32,  // Size in clip space units
    moon_color: vec3<f32>,
    phase: f32,
    camera_right: vec3<f32>,
    moon_elevation: f32,
    camera_up: vec3<f32>,
    time: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    let pos_2d = positions[vertex_index];

    // Project moon center to clip space
    let center_clip = uniforms.view_proj * vec4<f32>(uniforms.moon_world_pos, 1.0);

    // Offset in clip space (no aspect correction)
    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        center_clip.x + pos_2d.x * uniforms.moon_size * center_clip.w,
        center_clip.y + pos_2d.y * uniforms.moon_size * center_clip.w,
        center_clip.z,
        center_clip.w
    );
    out.uv = pos_2d * 0.5 + 0.5;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center) * 2.0;

    // Moon disc radius
    let moon_radius = 0.8;

    if dist < moon_radius {
        // Dimmer silvery moon (less bright than before)
        let moon_color = vec3<f32>(0.75, 0.77, 0.82);
        return vec4<f32>(moon_color, 1.0);
    } else {
        discard;
    }
}
