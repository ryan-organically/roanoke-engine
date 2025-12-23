// Ember and Smoke Particle Shader
// Renders rising ember particles and realistic smoke columns from campfires
// Smoke rises high above trees (30m+) and is visible from far distances
// Includes lingering canopy smoke for forest atmosphere

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
    camera_right: vec3<f32>,
    _pad1: f32,
    camera_up: vec3<f32>,
    _pad2: f32,
    // Campfire positions (xyz = position, w = intensity)
    campfire_data: array<vec4<f32>, 8>,
    campfire_count: u32,
    _pad3: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
    @location(2) color: vec3<f32>,
    @location(3) particle_type: f32,  // 0=ember, 1=rising smoke, 2=lingering smoke
}

// Hash functions for pseudo-random particle properties
fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 = p3 + dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash33(p: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // Particle distribution per campfire:
    // 0-79: Embers (80)
    // 80-179: Rising smoke column (100) - goes high into sky
    // 180-249: Lingering canopy smoke (70) - drifts at tree level
    let particles_per_fire = 250u;
    let embers_count = 80u;
    let rising_smoke_count = 100u;
    let lingering_smoke_count = 70u;

    let campfire_idx = instance_index / particles_per_fire;
    let particle_idx = instance_index % particles_per_fire;

    // Determine particle type
    var particle_type = 0u;  // ember
    var local_idx = particle_idx;
    if (particle_idx >= embers_count + rising_smoke_count) {
        particle_type = 2u;  // lingering smoke
        local_idx = particle_idx - embers_count - rising_smoke_count;
    } else if (particle_idx >= embers_count) {
        particle_type = 1u;  // rising smoke
        local_idx = particle_idx - embers_count;
    }

    out.particle_type = f32(particle_type);

    // Skip if no campfire at this index
    if (campfire_idx >= uniforms.campfire_count) {
        out.clip_position = vec4<f32>(0.0, 0.0, -2.0, 1.0);
        out.alpha = 0.0;
        out.uv = vec2<f32>(0.0, 0.0);
        out.color = vec3<f32>(0.0, 0.0, 0.0);
        return out;
    }

    let campfire = uniforms.campfire_data[campfire_idx];
    let campfire_pos = campfire.xyz;
    let intensity = campfire.w;

    // Skip if campfire is off
    if (intensity <= 0.0) {
        out.clip_position = vec4<f32>(0.0, 0.0, -2.0, 1.0);
        out.alpha = 0.0;
        out.uv = vec2<f32>(0.0, 0.0);
        out.color = vec3<f32>(0.0, 0.0, 0.0);
        return out;
    }

    // Pseudo-random particle properties
    let seed = vec3<f32>(f32(local_idx), f32(local_idx) * 1.7, f32(local_idx) * 0.9 + f32(campfire_idx) * 100.0);
    let rand = hash33(seed);

    var particle_pos: vec3<f32>;
    var particle_size: f32;

    if (particle_type == 2u) {
        // LINGERING CANOPY SMOKE - hangs around at tree level, drifts slowly
        let life_duration = 12.0 + rand.z * 8.0;  // 12-20 seconds
        let life_offset = hash31(seed + vec3<f32>(200.0, 0.0, 0.0)) * life_duration;
        let life_progress = fract((uniforms.time + life_offset) / life_duration);

        // Spawn at varying heights around canopy level (8-18m)
        let base_height = 8.0 + rand.x * 10.0;

        // Slow rise with lots of horizontal drift
        let rise_height = life_progress * 4.0;  // Only rises 4m total

        // Wind-driven horizontal drift - moves away from fire
        let wind_time = uniforms.time * 0.3 + rand.y * 10.0;
        let wind_dir = vec2<f32>(
            sin(wind_time * 0.7) * 0.6 + 0.4,  // Mostly one direction
            cos(wind_time * 0.5) * 0.3
        );
        let drift_distance = life_progress * 8.0;  // Drifts up to 8m away

        // Swirling motion
        let swirl_phase = uniforms.time * 0.5 + rand.x * 6.28;
        let swirl_radius = 1.0 + life_progress * 2.0;

        particle_pos = campfire_pos + vec3<f32>(
            wind_dir.x * drift_distance + sin(swirl_phase) * swirl_radius,
            base_height + rise_height,
            wind_dir.y * drift_distance + cos(swirl_phase) * swirl_radius,
        );

        // Large, diffuse particles
        let base_size = 0.8 + rand.x * 0.6;  // 0.8-1.4m
        let size_growth = 1.0 + life_progress * 2.0;
        particle_size = base_size * size_growth * intensity;

        // Very transparent, fades slowly
        let fade_in = smoothstep(0.0, 0.2, life_progress);
        let fade_out = 1.0 - smoothstep(0.6, 1.0, life_progress);
        out.alpha = fade_in * fade_out * intensity * 0.15;  // Very transparent

        // Light gray, slightly warm tinted
        let smoke_color = vec3<f32>(0.5, 0.48, 0.45);
        out.color = smoke_color;

    } else if (particle_type == 1u) {
        // RISING SMOKE COLUMN - goes high into sky, visible from far away
        let life_duration = 10.0 + rand.z * 6.0;  // 10-16 seconds
        let life_offset = hash31(seed + vec3<f32>(100.0, 0.0, 0.0)) * life_duration;
        let life_progress = fract((uniforms.time + life_offset) / life_duration);

        // Spawn position - tight column at base
        let spawn_radius = 0.2 + rand.x * 0.3;
        let spawn_angle = rand.y * 6.28318;

        // Rise height - goes VERY high (30-40 meters, well above trees)
        let rise_speed = 0.85 + rand.z * 0.3;
        let max_height = 35.0;
        // Accelerate then decelerate (realistic buoyancy)
        let rise_curve = 1.0 - pow(1.0 - life_progress, 2.0);
        let rise_height = rise_curve * max_height * rise_speed;

        // Billowing motion - more chaotic as it rises
        let billow_phase = uniforms.time * 0.6 + rand.x * 6.28;
        let billow_strength = life_progress * 1.5;
        let drift_x = sin(billow_phase + rise_height * 0.1) * billow_strength;
        let drift_z = cos(billow_phase * 0.8 + rand.y * 3.14) * billow_strength;

        // Column expands as it rises
        let expansion = 1.0 + life_progress * 3.0;

        particle_pos = campfire_pos + vec3<f32>(
            (spawn_radius * cos(spawn_angle) + drift_x) * expansion,
            rise_height + 0.5,
            (spawn_radius * sin(spawn_angle) + drift_z) * expansion,
        );

        // Size increases significantly with height for distant visibility
        let base_size = 0.3 + rand.x * 0.2;
        let size_growth = 1.0 + life_progress * 4.0;  // Gets 5x larger
        particle_size = base_size * size_growth * intensity;

        // Alpha - denser at bottom, more transparent at top
        let fade_in = smoothstep(0.0, 0.1, life_progress);
        let fade_out = 1.0 - smoothstep(0.7, 1.0, life_progress);
        let height_fade = 1.0 - life_progress * 0.5;  // Fade as it rises
        out.alpha = fade_in * fade_out * height_fade * intensity * 0.4;

        // Smoke color gradient - darker at bottom, lighter at top
        let dark_smoke = vec3<f32>(0.2, 0.18, 0.16);
        let light_smoke = vec3<f32>(0.55, 0.53, 0.50);
        out.color = mix(dark_smoke, light_smoke, life_progress * 0.8);

    } else {
        // EMBER PARTICLE - glowing sparks
        let life_duration = 2.5 + rand.z * 2.5;  // 2.5-5 seconds
        let life_offset = hash31(seed) * life_duration;
        let life_progress = fract((uniforms.time + life_offset) / life_duration);

        // Spawn position - small radius from center
        let spawn_radius = 0.3 * rand.x;
        let spawn_angle = rand.y * 6.28318;

        // Rise height (0-6 meters)
        let rise_height = life_progress * 6.0;

        // Swaying drift
        let drift_x = sin(uniforms.time * 2.5 + rand.x * 6.28) * 0.5 * life_progress;
        let drift_z = cos(uniforms.time * 2.0 + rand.y * 6.28) * 0.5 * life_progress;

        particle_pos = campfire_pos + vec3<f32>(
            spawn_radius * cos(spawn_angle) + drift_x,
            rise_height + 0.15,
            spawn_radius * sin(spawn_angle) + drift_z,
        );

        // Ember size - shrinks as it cools
        let base_size = 0.02 + rand.x * 0.015;
        let size_decay = 1.0 - life_progress * 0.7;
        particle_size = base_size * size_decay * intensity;

        // Bright fade
        let fade_in = smoothstep(0.0, 0.08, life_progress);
        let fade_out = 1.0 - smoothstep(0.5, 1.0, life_progress);
        out.alpha = fade_in * fade_out * intensity;

        // Orange to yellow-white
        let core_color = vec3<f32>(1.0, 0.35, 0.08);
        let cool_color = vec3<f32>(1.0, 0.75, 0.35);
        out.color = mix(core_color, cool_color, life_progress * 0.8);
    }

    // Billboard vertices
    var local_offset: vec2<f32>;
    switch vertex_index % 6u {
        case 0u: { local_offset = vec2<f32>(-1.0, 1.0); }
        case 1u: { local_offset = vec2<f32>(1.0, 1.0); }
        case 2u: { local_offset = vec2<f32>(-1.0, -1.0); }
        case 3u: { local_offset = vec2<f32>(1.0, 1.0); }
        case 4u: { local_offset = vec2<f32>(1.0, -1.0); }
        default: { local_offset = vec2<f32>(-1.0, -1.0); }
    }

    // Billboard facing camera
    let right = normalize(uniforms.camera_right);
    let up = normalize(uniforms.camera_up);
    let world_pos = particle_pos
        + right * local_offset.x * particle_size
        + up * local_offset.y * particle_size;

    out.uv = (local_offset + 1.0) * 0.5;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv - 0.5) * 2.0;

    var circle_alpha: f32;
    var final_color: vec3<f32>;

    if (in.particle_type > 0.5) {
        // SMOKE (both rising and lingering): Very soft, volumetric edges
        // Gaussian-like falloff for realistic smoke
        let gaussian = exp(-dist * dist * 2.0);
        circle_alpha = gaussian;
        final_color = in.color;
    } else {
        // EMBER: Sharp glow with soft edge
        circle_alpha = 1.0 - smoothstep(0.0, 0.9, dist);
        let glow = 1.0 + (1.0 - dist) * 0.6;
        final_color = in.color * glow;
    }

    let final_alpha = in.alpha * circle_alpha;

    if (final_alpha < 0.005) {
        discard;
    }

    return vec4<f32>(final_color, final_alpha);
}
