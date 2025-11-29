struct Uniforms {
    view_proj: mat4x4<f32>,
    sun_dir: vec3<f32>,
    time: f32,
    sun_color: vec3<f32>,
    cloud_coverage: f32,
    cloud_color_base: vec3<f32>,
    cloud_density: f32,
    cloud_color_shade: vec3<f32>,
    cloud_scale: f32,
    wind_offset: vec2<f32>,
    padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    let pos = positions[in_vertex_index];
    
    var output: VertexOutput;
    output.clip_position = vec4<f32>(pos, 1.0, 1.0);
    output.world_pos = vec3<f32>(pos.x, pos.y, 1.0);
    output.uv = pos * 0.5 + 0.5; // 0..1 range
    return output;
}

// Simple Hash Function
fn hash(p: vec2<f32>) -> f32 {
    var p2 = p;
    p2 = 50.0 * fract(p2 * 0.3183099 + vec2<f32>(0.71, 0.113));
    return -1.0 + 2.0 * fract(p2.x * p2.y * (p2.x + p2.y));
}

// 2D Noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(mix(hash(i + vec2<f32>(0.0, 0.0)), 
                   hash(i + vec2<f32>(1.0, 0.0)), u.x),
               mix(hash(i + vec2<f32>(0.0, 1.0)), 
                   hash(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

// FBM (Fractal Brownian Motion)
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 0.0;
    var p2 = p;
    
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(p2);
        p2 = p2 * 2.0;
        amplitude *= 0.5;
    }
    return value;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sky Gradient
    let top_color = vec3<f32>(0.2, 0.4, 0.8);
    let horizon_color = vec3<f32>(0.6, 0.7, 0.9);
    let y = input.world_pos.y * 0.5 + 0.5;
    var sky_color = mix(horizon_color, top_color, pow(y, 0.5));
    
    // Cloud Rendering
    // Project UVs to "sky plane"
    // We want clouds to look like they are on a plane above.
    // Simple approximation: Use UVs + time
    
    let cloud_speed = 0.05;
    let time_offset = uniforms.time * cloud_speed;
    let wind = uniforms.wind_offset + vec2<f32>(time_offset, time_offset * 0.5);
    
    // Scale UVs for cloud texture
    let uv_scaled = (input.world_pos.xy * 2.0) * uniforms.cloud_scale + wind;
    
    // Generate Noise
    var n = fbm(uv_scaled);
    
    // Shape clouds
    // Remap noise from [-1, 1] to [0, 1]
    n = n * 0.5 + 0.5;
    
    // Apply coverage threshold
    // coverage 0.0 = no clouds, 1.0 = full clouds
    // We want to discard low noise values based on coverage
    // If coverage is high, we keep more low values.
    // Let's say threshold = 1.0 - coverage
    let threshold = 1.0 - uniforms.cloud_coverage;
    
    // Soft threshold
    let cloud_alpha = smoothstep(threshold - 0.1, threshold + 0.1, n);
    
    // Density
    let density = cloud_alpha * uniforms.cloud_density;
    
    if (density > 0.01) {
        // Cloud Color Gradient
        // Mix between base (Burnt Sienna) and shade (Pink) based on noise "thickness"
        // Thicker parts (higher n) might be lighter or darker depending on style.
        // Let's make thicker parts the "shade" color (maybe darker pink/purple)
        // and edges the "base" color (burnt sienna).
        
        let color_mix = smoothstep(threshold, threshold + 0.4, n);
        let cloud_rgb = mix(uniforms.cloud_color_base, uniforms.cloud_color_shade, color_mix);
        
        // Lighting/Shading fake
        // Add a bit of white highlight on "top" (based on sun dir? or just noise derivative?)
        // Simple: lighter color for very high density
        let highlight = smoothstep(0.8, 1.0, n);
        let final_cloud_color = mix(cloud_rgb, vec3<f32>(1.0, 0.9, 0.9), highlight * 0.5);
        
        // Blend with sky
        sky_color = mix(sky_color, final_cloud_color, density);
    }
    
    // Sun Glow (Simple)
    // We don't have exact view ray here easily for a quad, but we can approximate.
    // Or just rely on the SunPipeline for the actual sun disk.
    // Let's add a subtle glow if looking up?
    // Nah, let's keep it clean.
    
    return vec4<f32>(sky_color, 1.0);
}
