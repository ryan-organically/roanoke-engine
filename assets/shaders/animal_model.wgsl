// Animal Model Shader
// Renders 3D animal models with skeletal animation, shadows and moody lighting

// Maximum joints for skeletal animation
const MAX_JOINTS: u32 = 64u;

struct CameraUniform {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,  // Shadow mapping
    camera_pos: vec3<f32>,
    time: f32,
    light_dir: vec3<f32>,
    ambient_dimming: f32,
    fog_color: vec3<f32>,
    fog_start: f32,
    fog_end: f32,
    fog_density: f32,
    shadow_strength: f32,
    rain_wetness: f32,
}

// Joint matrices for skeletal animation
struct JointMatrices {
    matrices: array<mat4x4<f32>, 64>,  // MAX_JOINTS
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Texture bindings (group 1)
@group(1) @binding(0)
var animal_texture: texture_2d<f32>;
@group(1) @binding(1)
var animal_sampler: sampler;

// Shadow map bindings (group 2)
@group(2) @binding(0) var t_shadow: texture_depth_2d;
@group(2) @binding(1) var s_shadow: sampler_comparison;

// Joint matrices binding (group 3) - for skinned models
@group(3) @binding(0)
var<storage, read> joints: JointMatrices;

// Vertex input with optional skinning data
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // Skinning attributes (joint indices and weights)
    @location(3) joint_indices: vec4<u32>,
    @location(4) joint_weights: vec4<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color: vec3<f32>,
    @location(10) emissive: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) emissive: f32,
    @location(4) uv: vec2<f32>,
    @location(5) view_distance: f32,
    @location(6) shadow_pos: vec3<f32>,
}

// Apply skeletal skinning to position and normal
fn apply_skinning(
    position: vec3<f32>,
    normal: vec3<f32>,
    joint_indices: vec4<u32>,
    joint_weights: vec4<f32>
) -> array<vec3<f32>, 2> {
    // Check if this vertex has skinning (weight sum > 0)
    let weight_sum = joint_weights.x + joint_weights.y + joint_weights.z + joint_weights.w;

    if (weight_sum < 0.001) {
        // No skinning, return original position/normal
        return array<vec3<f32>, 2>(position, normal);
    }

    var skinned_pos = vec3<f32>(0.0);
    var skinned_normal = vec3<f32>(0.0);

    // Apply influence from each joint
    for (var i = 0u; i < 4u; i = i + 1u) {
        var weight: f32;
        var joint_idx: u32;

        // Extract weight and index for this influence
        if (i == 0u) {
            weight = joint_weights.x;
            joint_idx = joint_indices.x;
        } else if (i == 1u) {
            weight = joint_weights.y;
            joint_idx = joint_indices.y;
        } else if (i == 2u) {
            weight = joint_weights.z;
            joint_idx = joint_indices.z;
        } else {
            weight = joint_weights.w;
            joint_idx = joint_indices.w;
        }

        if (weight > 0.001 && joint_idx < MAX_JOINTS) {
            let joint_mat = joints.matrices[joint_idx];

            // Transform position by joint matrix
            skinned_pos += weight * (joint_mat * vec4<f32>(position, 1.0)).xyz;

            // Transform normal by joint matrix (using upper 3x3)
            let normal_mat = mat3x3<f32>(
                joint_mat[0].xyz,
                joint_mat[1].xyz,
                joint_mat[2].xyz
            );
            skinned_normal += weight * (normal_mat * normal);
        }
    }

    // Normalize the result
    return array<vec3<f32>, 2>(skinned_pos, normalize(skinned_normal));
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Reconstruct model matrix from instance data
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // Apply skeletal skinning if this vertex has joint weights
    let skinned = apply_skinning(
        vertex.position,
        vertex.normal,
        vertex.joint_indices,
        vertex.joint_weights
    );
    let local_pos = skinned[0];
    let local_normal = skinned[1];

    // Transform skinned position to world space
    let world_pos = model_matrix * vec4<f32>(local_pos, 1.0);

    // Transform skinned normal to world space (using upper 3x3 of model matrix)
    let normal_matrix = mat3x3<f32>(
        instance.model_matrix_0.xyz,
        instance.model_matrix_1.xyz,
        instance.model_matrix_2.xyz,
    );
    let world_normal = normalize(normal_matrix * local_normal);

    // Calculate view distance for fog
    let view_distance = length(camera.camera_pos - world_pos.xyz);

    // Calculate shadow position
    let pos_from_light = camera.light_view_proj * world_pos;
    let shadow_ndc = pos_from_light.xyz / pos_from_light.w;
    let shadow_pos = vec3<f32>(
        shadow_ndc.x * 0.5 + 0.5,
        -shadow_ndc.y * 0.5 + 0.5,
        shadow_ndc.z
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_pos;
    out.world_normal = world_normal;
    out.world_position = world_pos.xyz;
    out.color = instance.color;
    out.emissive = instance.emissive;
    out.uv = vertex.uv;
    out.view_distance = view_distance;
    out.shadow_pos = shadow_pos;

    return out;
}

// Simple hash function for procedural variation
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(animal_texture, animal_sampler, in.uv);

    // Light direction from uniform
    let light_dir = normalize(camera.light_dir);
    let sun_elevation = -light_dir.y;
    let day_factor = smoothstep(-0.1, 0.3, sun_elevation);

    // Shadow calculation
    var shadow = 1.0;
    let shadow_uv = in.shadow_pos.xy;
    let shadow_depth = in.shadow_pos.z;

    if (shadow_uv.x >= 0.0 && shadow_uv.x <= 1.0 &&
        shadow_uv.y >= 0.0 && shadow_uv.y <= 1.0 &&
        shadow_depth >= 0.0 && shadow_depth <= 1.0) {
        shadow = textureSampleCompare(t_shadow, s_shadow, shadow_uv, shadow_depth);
        // Apply shadow strength for deeper, moodier shadows
        let shadow_darkness = 0.12 + (1.0 - camera.shadow_strength) * 0.38;
        shadow = shadow * (1.0 - shadow_darkness) + shadow_darkness;
    }

    // Day/night light colors
    let day_light_color = vec3<f32>(0.95, 0.9, 0.85);
    let night_light_color = vec3<f32>(0.15, 0.18, 0.25);
    let light_color = mix(night_light_color, day_light_color, day_factor);

    // View direction for specular
    let view_dir = normalize(camera.camera_pos - in.world_position);
    let half_dir = normalize(-light_dir + view_dir);

    // Diffuse lighting - reduced for moody atmosphere
    let ndotl = max(dot(in.world_normal, -light_dir), 0.0);
    let day_diffuse = 0.55 * (1.0 - camera.ambient_dimming * 0.4);
    let night_diffuse = 0.08;
    let diffuse = ndotl * mix(night_diffuse, day_diffuse, day_factor);

    // Specular highlight - reduced and wet surfaces get more
    let ndoth = max(dot(in.world_normal, half_dir), 0.0);
    let base_specular = pow(ndoth, 16.0) * 0.1 * day_factor;
    let wet_specular = pow(ndoth, 32.0) * camera.rain_wetness * 0.25 * day_factor;
    let specular = (base_specular + wet_specular) * shadow;

    // Ambient light - reduced for moody atmosphere
    let sky_ambient = 0.1 * max(in.world_normal.y, 0.0) * day_factor;
    let day_ambient = (0.18 + sky_ambient) * (1.0 - camera.ambient_dimming * 0.5);
    let night_ambient = 0.04;
    let ambient = mix(night_ambient, day_ambient, day_factor);

    // Subtle subsurface scattering for organic look (fur/skin)
    let sss = max(0.0, dot(light_dir, in.world_normal)) * 0.08 * day_factor * shadow;

    // Rim lighting for silhouette definition in low light
    let rim = pow(1.0 - max(dot(view_dir, in.world_normal), 0.0), 4.0) * 0.12 * day_factor;

    // Combine lighting - shadow affects diffuse/specular, not ambient
    let lighting = ambient + (diffuse + sss + rim) * shadow + specular;

    // Add subtle fur texture variation
    let fur_noise = hash(in.uv * 50.0) * 0.04;

    // Base color from texture, tinted by instance color
    var base_color = tex_color.rgb * in.color;

    // Wet surfaces are darker
    if (camera.rain_wetness > 0.0) {
        base_color = base_color * (1.0 - camera.rain_wetness * 0.2);
    }

    var final_color = base_color * lighting * light_color;
    final_color = final_color * (0.98 + fur_noise);

    // Add emissive glow (for damage flash, aggressive states)
    if in.emissive > 0.0 {
        let flash_color = vec3<f32>(1.0, 0.3, 0.2);
        final_color = mix(final_color, flash_color, in.emissive * 0.5);
        final_color = final_color + flash_color * in.emissive * 0.3;
    }

    // Apply fog
    let fog_factor = clamp(
        (in.view_distance - camera.fog_start) / (camera.fog_end - camera.fog_start),
        0.0,
        1.0
    );
    let fog_amount = 1.0 - exp(-fog_factor * camera.fog_density);
    final_color = mix(final_color, camera.fog_color, fog_amount);

    return vec4<f32>(final_color, tex_color.a);
}
