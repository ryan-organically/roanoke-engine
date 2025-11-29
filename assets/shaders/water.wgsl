// Water Surface Shader
// Renders the water mesh using displacement maps from the compute shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct WaterMaterial {
    deep_color: vec4<f32>,
    shallow_color: vec4<f32>,
    foam_color: vec4<f32>,
    smoothness: f32,
    metallic: f32,
}

@group(1) @binding(0)
var<uniform> material: WaterMaterial;

// Displacement Map (from Compute Shader)
@group(1) @binding(1)
var displacement_texture: texture_2d<f32>;
@group(1) @binding(2)
var displacement_sampler: sampler;

// Normal/Jacobian Map (from Compute Shader)
@group(1) @binding(3)
var normal_texture: texture_2d<f32>;
@group(1) @binding(4)
var normal_sampler: sampler;

// Environment Map (Skybox) - Optional, for reflection
// @group(1) @binding(5)
// var env_texture: texture_cube<f32>;
// @group(1) @binding(6)
// var env_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) view_vector: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Sample displacement
    // UVs for the water grid are 0..1
    let disp = textureSampleLevel(displacement_texture, displacement_sampler, input.uv, 0.0);
    
    // Apply displacement
    // disp.x, disp.z are horizontal displacement (choppiness)
    // disp.y is vertical height
    let displaced_pos = input.position + vec3<f32>(disp.x, disp.y, disp.z);
    
    output.world_position = displaced_pos;
    output.clip_position = camera.view_proj * vec4<f32>(displaced_pos, 1.0);
    output.uv = input.uv;
    output.view_vector = camera.position - displaced_pos;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(input.view_vector);
    
    // Sample Normal and Jacobian
    let normal_data = textureSample(normal_texture, normal_sampler, input.uv);
    let normal = normalize(normal_data.xyz); // World space normal
    let jacobian = normal_data.w; // Foam factor
    
    // Lighting (Sun)
    let sun_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));
    let half_dir = normalize(view_dir + sun_dir);
    
    // Fresnel (Schlick approximation)
    let F0 = 0.02; // Water is non-metallic
    let NdotV = max(dot(normal, view_dir), 0.0);
    let fresnel = F0 + (1.0 - F0) * pow(1.0 - NdotV, 5.0);
    
    // Specular (Blinn-Phong or GGX simplified)
    let NdotH = max(dot(normal, half_dir), 0.0);
    let specular = pow(NdotH, material.smoothness * 100.0);
    
    // Sub-surface Scattering (SSS) / Color
    // Simple approximation: mix deep and shallow based on view angle or height?
    // Actually, SSS is better approximated by light wrapping or thickness, but for ocean surface:
    // We see deep color when looking down, shallow/sky when looking at grazing angles (Fresnel).
    // Also, wave peaks (jacobian < 1) are thinner/foamier.
    
    var base_color = mix(material.deep_color, material.shallow_color, jacobian); // Foam/Churn brightens it
    
    // Add foam based on Jacobian
    let foam_threshold = 0.8;
    if (jacobian < foam_threshold) {
        let foam_intensity = (foam_threshold - jacobian) / foam_threshold;
        base_color = mix(base_color, material.foam_color, foam_intensity);
    }
    
    // Combine
    // Reflection would come from skybox here. For now, use sky color approximation.
    let sky_color = vec3<f32>(0.5, 0.7, 0.9); // Light blue sky
    let reflection = sky_color * fresnel;
    
    let final_color = base_color.rgb * (1.0 - fresnel) + reflection + vec3<f32>(specular);
    
    return vec4<f32>(final_color, 1.0);
}
