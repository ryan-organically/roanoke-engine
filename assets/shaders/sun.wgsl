// Sun Billboard Shader - renders a sun disk in the sky with horizon shimmer

struct Uniforms {
    view_proj: mat4x4<f32>,
    sun_world_pos: vec3<f32>,
    sun_size: f32,
    sun_color: vec3<f32>,
    time: f32,
    camera_right: vec3<f32>,
    sun_elevation: f32,  // For horizon shimmer effect
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

// Simple noise for shimmer
fn hash(p: vec2<f32>) -> f32 {
    let p2 = fract(p * 0.3183099 + vec2<f32>(0.71, 0.113));
    return fract(sin(dot(p2, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from center of quad
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center) * 2.0; // 0 at center, 1 at edge

    // Sun disk with soft glow
    let core_radius = 0.3;
    let corona_radius = 1.0;

    // Horizon shimmer effect - stronger when sun is near horizon (elevation 0-0.2)
    let horizon_factor = 1.0 - smoothstep(0.0, 0.25, uniforms.sun_elevation);

    // Animated shimmer pattern
    let shimmer_uv = in.uv * 10.0 + uniforms.time * 0.5;
    let shimmer = hash(shimmer_uv) * 0.15 + hash(shimmer_uv * 2.3 + 1.7) * 0.1;
    let shimmer_intensity = shimmer * horizon_factor;

    if dist < core_radius {
        // Bright core - white to yellow with shimmer
        let core_blend = dist / core_radius;
        var core_color = mix(
            vec3<f32>(1.0, 1.0, 0.95),  // White center
            uniforms.sun_color,          // Sun color at edge
            core_blend * core_blend
        );

        // Add shimmer near horizon - makes sun appear to dance/flicker
        core_color = core_color * (1.0 + shimmer_intensity * 0.3);

        return vec4<f32>(core_color, 1.0);
    } else if dist < corona_radius {
        // Corona glow with shimmer
        let corona_blend = (dist - core_radius) / (corona_radius - core_radius);
        var glow = exp(-corona_blend * 4.0);

        // Enhanced glow at horizon with shimmer
        let horizon_glow_boost = horizon_factor * 0.4;
        glow = glow * (1.0 + horizon_glow_boost + shimmer_intensity);

        // Color shifts more orange/red near horizon
        let horizon_color = mix(uniforms.sun_color, vec3<f32>(1.0, 0.4, 0.1), horizon_factor * 0.5);

        return vec4<f32>(horizon_color, glow * 0.8);
    } else {
        // Outside sun
        discard;
    }
}
