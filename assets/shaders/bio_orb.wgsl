// Bioluminescent Orb Shader
// Renders glowing fungi, moss, and crystal orbs in caves
// Uses soft glow falloff and pulsing animation for organic feel

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct InstanceInput {
    @location(5) world_position: vec3<f32>,
    @location(6) radius: f32,
    @location(7) color: vec3<f32>,
    @location(8) intensity: f32,
    @location(9) pulse_phase: f32,
    @location(10) pulse_speed: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) local_position: vec3<f32>,
    @location(3) color: vec3<f32>,
    @location(4) intensity: f32,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Scale and position the sphere
    let scaled_pos = vertex.position * instance.radius;
    let world_pos = scaled_pos + instance.world_position;

    // Calculate pulsing intensity
    let pulse = sin(camera.time * instance.pulse_speed * 6.28318 + instance.pulse_phase);
    let pulsed_intensity = instance.intensity * (0.7 + pulse * 0.3);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.world_normal = vertex.normal;
    out.world_position = world_pos;
    out.local_position = vertex.position;
    out.color = instance.color;
    out.intensity = pulsed_intensity;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(camera.camera_pos - in.world_position);

    // Soft glow falloff from center
    let dist_from_center = length(in.local_position);
    let core_glow = exp(-dist_from_center * 1.5);

    // Fresnel rim for ethereal look
    let ndotv = max(dot(in.world_normal, view_dir), 0.0);
    let fresnel = pow(1.0 - ndotv, 2.5) * 0.6;

    // Inner glow (brighter at center)
    let inner_glow = pow(max(1.0 - dist_from_center, 0.0), 2.0);

    // Combine for final glow
    let glow = (core_glow * 0.6 + inner_glow * 0.3 + fresnel * 0.4) * in.intensity;

    // Subtle color shift at edges (cooler tint)
    let edge_tint = vec3<f32>(0.7, 0.9, 1.0);
    let final_color = mix(in.color, edge_tint, fresnel * 0.2) * glow;

    // Soft falloff alpha for smooth blending
    let alpha = clamp(glow * 0.7, 0.0, 0.9);

    return vec4<f32>(final_color, alpha);
}
