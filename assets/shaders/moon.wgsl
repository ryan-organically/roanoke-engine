// Moon Billboard Shader - renders a soft, ethereal moon

struct Uniforms {
    view_proj: mat4x4<f32>,
    moon_world_pos: vec3<f32>,
    moon_size: f32,
    moon_color: vec3<f32>,
    phase: f32,  // 0 = new, 0.5 = full, 1 = new again
    camera_right: vec3<f32>,
    _padding2: f32,
    camera_up: vec3<f32>,
    time: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Billboard quad vertices (two triangles)
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

    // Billboard in world space
    let world_pos = uniforms.moon_world_pos
        + uniforms.camera_right * pos_2d.x * uniforms.moon_size
        + uniforms.camera_up * pos_2d.y * uniforms.moon_size;

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = pos_2d * 0.5 + 0.5;

    return out;
}

// Simple noise for surface texture
fn hash2(p: vec2<f32>) -> f32 {
    let p2 = 50.0 * fract(p * 0.3183099 + vec2<f32>(0.71, 0.113));
    return fract(p2.x * p2.y * (p2.x + p2.y));
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    return mix(mix(hash2(i + vec2<f32>(0.0, 0.0)),
                   hash2(i + vec2<f32>(1.0, 0.0)), u.x),
               mix(hash2(i + vec2<f32>(0.0, 1.0)),
                   hash2(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center) * 2.0;

    // Moon disk radius
    let moon_radius = 0.35;

    // Silvery moon colors
    let moon_bright = vec3<f32>(0.95, 0.95, 1.0);    // Bright silver-white
    let moon_mid = vec3<f32>(0.85, 0.87, 0.92);      // Slightly blue-grey
    let moon_dark = vec3<f32>(0.6, 0.62, 0.68);      // Darker maria regions

    if (dist < moon_radius) {
        // Moon surface with subtle texture
        let surface_uv = (in.uv - center) * 8.0;

        // Create subtle surface variation (maria/highlands)
        let surface_noise = noise2(surface_uv * 2.0) * 0.5 +
                           noise2(surface_uv * 4.0) * 0.25 +
                           noise2(surface_uv * 8.0) * 0.125;

        // Limb darkening - edges of moon are slightly darker
        let limb_factor = 1.0 - pow(dist / moon_radius, 2.0) * 0.15;

        // Mix surface colors based on noise
        var surface_color = mix(moon_mid, moon_bright, surface_noise * 0.6);

        // Add some darker "maria" patches
        let maria_noise = noise2(surface_uv * 1.5 + vec2<f32>(3.7, 2.1));
        if (maria_noise > 0.55) {
            surface_color = mix(surface_color, moon_dark, (maria_noise - 0.55) * 1.5);
        }

        surface_color *= limb_factor;

        return vec4<f32>(surface_color, 1.0);
    }

    discard;
}
