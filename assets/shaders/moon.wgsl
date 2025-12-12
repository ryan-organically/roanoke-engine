// Moon Billboard Shader - renders a soft, ethereal moon with horizon shimmer

struct Uniforms {
    view_proj: mat4x4<f32>,
    moon_world_pos: vec3<f32>,
    moon_size: f32,
    moon_color: vec3<f32>,
    phase: f32,  // 0 = new, 0.5 = full, 1 = new again
    camera_right: vec3<f32>,
    moon_elevation: f32,  // For horizon shimmer effect
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

// Simple hash for shimmer effect
fn shimmer_hash(p: vec2<f32>) -> f32 {
    let p2 = fract(p * 0.3183099 + vec2<f32>(0.71, 0.113));
    return fract(sin(dot(p2, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from center of quad (same technique as sun shader)
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center) * 2.0; // 0 at center, 1 at edge

    // Moon disk - smaller than sun, subtle glow
    let core_radius = 0.25;   // Smaller moon disk
    let glow_radius = 1.0;    // Soft glow extends to edge

    // Horizon shimmer effect - subtle, silvery
    let horizon_factor = 1.0 - smoothstep(0.0, 0.25, uniforms.moon_elevation);

    // Animated shimmer pattern (slower, more ethereal than sun)
    let shimmer_uv = in.uv * 8.0 + uniforms.time * 0.3;
    let shimmer = shimmer_hash(shimmer_uv) * 0.08 + shimmer_hash(shimmer_uv * 1.7 + 2.3) * 0.05;
    let shimmer_intensity = shimmer * horizon_factor;

    // Pure silver colors - no orange/warm tones
    let moon_silver = vec3<f32>(0.92, 0.94, 0.98);
    let glow_silver = vec3<f32>(0.75, 0.78, 0.88);

    if dist < core_radius {
        // Moon disk - soft bright silver with gentle edge blend
        let core_blend = dist / core_radius;

        // Soft edge falloff like sun - no harsh boundary
        let edge_softness = 1.0 - smoothstep(0.6, 1.0, core_blend);

        // Subtle surface variation
        let surface_uv = (in.uv - center) * 6.0;
        let surface_var = noise2(surface_uv * 2.0) * 0.08;

        var moon_color = moon_silver * (0.95 + surface_var);
        moon_color = moon_color * edge_softness;

        // Subtle shimmer near horizon
        moon_color = moon_color * (1.0 + shimmer_intensity * 0.15);

        // Blend alpha at edges for soft boundary
        let alpha = smoothstep(1.0, 0.7, core_blend);
        return vec4<f32>(moon_color, alpha);
    } else if dist < glow_radius {
        // Soft diffuse glow - like sun's blur but more subtle
        let glow_blend = (dist - core_radius) / (glow_radius - core_radius);

        // Softer exponential falloff - less defined than sun
        var glow = exp(-glow_blend * 5.0) * 0.4;

        // Very subtle horizon enhancement - stays silver
        glow = glow * (1.0 + horizon_factor * 0.15 + shimmer_intensity * 0.5);

        // Pure silver glow - no warm color shift
        return vec4<f32>(glow_silver, glow);
    } else {
        discard;
    }
}
