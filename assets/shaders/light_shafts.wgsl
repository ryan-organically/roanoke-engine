// Light Shafts (God Rays) Post-Process Shader
//
// Technique: Radial blur from sun screen position
// The effect samples along rays from each pixel toward the sun,
// accumulating brightness to create volumetric light beams.

struct Uniforms {
    sun_screen_pos: vec2<f32>,  // Sun position in screen space (0-1)
    intensity: f32,              // Overall intensity
    decay: f32,                  // How quickly rays fade (0.9-0.99)
    density: f32,                // Scattering density
    weight: f32,                 // Sample weight
    exposure: f32,               // Final exposure adjustment
    num_samples: i32,            // Number of samples along ray
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var t_scene: texture_2d<f32>;
@group(0) @binding(2) var s_scene: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Full-screen triangle
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    let pos = positions[vertex_index];

    var output: VertexOutput;
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = pos * 0.5 + 0.5;
    output.uv.y = 1.0 - output.uv.y; // Flip Y for texture coordinates
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;

    // Vector from pixel to sun
    let delta_uv = (uv - uniforms.sun_screen_pos) * (1.0 / f32(uniforms.num_samples)) * uniforms.density;

    // Dithered start offset to hide banding with fewer samples
    let dither = fract(dot(input.position.xy, vec2<f32>(0.46164, 0.96454)));
    var sample_uv = uv - delta_uv * dither;
    var accumulated_light = vec3<f32>(0.0);
    var illumination_decay = 1.0;

    // March toward sun, accumulating light
    for (var i = 0; i < uniforms.num_samples; i++) {
        sample_uv -= delta_uv;

        // Sample scene - bright areas contribute to light shafts
        let sample_color = textureSample(t_scene, s_scene, sample_uv).rgb;

        // Extract brightness (simple luminance)
        let luminance = dot(sample_color, vec3<f32>(0.299, 0.587, 0.114));

        // Only bright areas contribute (threshold)
        let bright = max(luminance - 0.5, 0.0) * 2.0;

        // Accumulate with decay
        accumulated_light += sample_color * bright * illumination_decay * uniforms.weight;
        illumination_decay *= uniforms.decay;
    }

    // Apply exposure and intensity
    let light_shaft_color = accumulated_light * uniforms.exposure * uniforms.intensity;

    // Get original scene color
    let scene_color = textureSample(t_scene, s_scene, uv).rgb;

    // Additive blend - light shafts add to scene
    let final_color = scene_color + light_shaft_color;

    return vec4<f32>(final_color, 1.0);
}

// Alternative: Occlusion-based light shafts (for use with depth buffer)
// This version uses depth to determine sky vs geometry
@fragment
fn fs_occlusion(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;

    // Distance from sun affects intensity
    let sun_dist = length(uv - uniforms.sun_screen_pos);
    let sun_falloff = 1.0 - saturate(sun_dist * 1.5);

    // Skip if sun is off-screen or too far
    if (sun_falloff < 0.01) {
        return textureSample(t_scene, s_scene, uv);
    }

    let delta_uv = (uv - uniforms.sun_screen_pos) * (1.0 / f32(uniforms.num_samples)) * uniforms.density;

    // Dithered start offset to hide banding
    let dither2 = fract(dot(input.position.xy, vec2<f32>(0.46164, 0.96454)));
    var sample_uv = uv - delta_uv * dither2;
    var accumulated_light = vec3<f32>(0.0);
    var illumination_decay = 1.0;

    // Warm sun color for shafts
    let shaft_color = vec3<f32>(1.0, 0.9, 0.7);

    for (var i = 0; i < uniforms.num_samples; i++) {
        sample_uv -= delta_uv;

        // Clamp to valid range
        let clamped_uv = clamp(sample_uv, vec2<f32>(0.0), vec2<f32>(1.0));

        let sample_color = textureSample(t_scene, s_scene, clamped_uv).rgb;
        let luminance = dot(sample_color, vec3<f32>(0.299, 0.587, 0.114));

        // Sky is bright, geometry is dark - this creates occlusion
        let sky_mask = smoothstep(0.4, 0.8, luminance);

        accumulated_light += shaft_color * sky_mask * illumination_decay * uniforms.weight;
        illumination_decay *= uniforms.decay;
    }

    let light_shafts = accumulated_light * uniforms.exposure * uniforms.intensity * sun_falloff;

    let scene_color = textureSample(t_scene, s_scene, uv).rgb;
    let final_color = scene_color + light_shafts;

    return vec4<f32>(final_color, 1.0);
}
