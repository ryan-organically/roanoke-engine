use crate::mesh_gen::get_height_at;
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec3, Quat};

/// Rock types with different sizes and characteristics
#[derive(Clone, Copy, Debug)]
pub enum RockType {
    Pebble,       // Tiny scattered stones
    SmallRock,    // Small rocks
    MediumRock,   // Medium boulders
    LargeBoulder, // Large prominent boulders
    FlatRock,     // Flat stepping stones
    MossyRock,    // Mossy rocks in damp areas
}

impl RockType {
    pub fn mesh_name(&self) -> &'static str {
        match self {
            RockType::Pebble => "rock_pebble",
            RockType::SmallRock => "rock_small",
            RockType::MediumRock => "rock_medium",
            RockType::LargeBoulder => "rock_boulder",
            RockType::FlatRock => "rock_flat",
            RockType::MossyRock => "rock_mossy",
        }
    }

    pub fn base_scale(&self) -> f32 {
        match self {
            RockType::Pebble => 0.15,
            RockType::SmallRock => 0.4,
            RockType::MediumRock => 0.8,
            RockType::LargeBoulder => 1.5,
            RockType::FlatRock => 0.6,
            RockType::MossyRock => 0.7,
        }
    }

    pub fn sink_amount(&self) -> f32 {
        match self {
            RockType::Pebble => 0.05,
            RockType::SmallRock => 0.1,
            RockType::MediumRock => 0.2,
            RockType::LargeBoulder => 0.4,
            RockType::FlatRock => 0.15,
            RockType::MossyRock => 0.25,
        }
    }
}

/// Generate rocks for a terrain chunk based on terrain features
///
/// Rocks appear on steep slopes, river banks, and in "RockyScrub" biomes.
/// Returns a list of (mesh_name, transform) tuples.
pub fn generate_rocks_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<(String, Mat4)> {
    let noise = Perlin::new(seed + 888);
    let pebble_noise = Perlin::new(seed + 889);

    let mut instances = Vec::new();

    // --- Large/Medium rocks (sparse) ---
    let large_rock_density = 0.02;
    let potential_large = (chunk_size * chunk_size * large_rock_density) as u32;

    for i in 0..potential_large {
        let rand_x = noise.get([i as f64 * 0.1, 200.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 300.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Calculate slope
        let sample_dist = 1.0;
        let (h_dx, _) = get_height_at(world_x + sample_dist, world_z, seed);
        let (h_dz, _) = get_height_at(world_x, world_z + sample_dist, seed);
        let slope = ((h_dx - height).powi(2) + (h_dz - height).powi(2)).sqrt() / sample_dist;

        let is_steep = slope > 0.3;
        let rocky_noise_val = noise.get([world_x as f64 * 0.05, world_z as f64 * 0.05]) as f32;
        let is_rocky_biome = rocky_noise_val > 0.2;
        let is_above_water = height > 0.5;

        if !is_above_water || (!is_steep && !is_rocky_biome) {
            continue;
        }

        // Select rock type based on noise
        let type_noise = noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3]) as f32;
        let rock_type = if type_noise > 0.5 {
            RockType::LargeBoulder
        } else if type_noise > 0.1 {
            RockType::MediumRock
        } else if height < 3.0 && rocky_noise_val < 0.4 {
            RockType::MossyRock // Mossy rocks in low damp areas
        } else {
            RockType::FlatRock
        };

        let angle = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * std::f32::consts::TAU;
        let scale_var = noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]) as f32;
        let scale = rock_type.base_scale() * (1.0 + scale_var * 0.4);

        // Slight tilt for natural look
        let tilt_x = noise.get([world_x as f64 * 0.7, 100.0]) as f32 * 0.15;
        let tilt_z = noise.get([world_z as f64 * 0.7, 150.0]) as f32 * 0.15;

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_euler(glam::EulerRot::XYZ, tilt_x, angle, tilt_z),
            Vec3::new(world_x, height - rock_type.sink_amount(), world_z),
        );

        instances.push((rock_type.mesh_name().to_string(), transform));
    }

    // --- Small rocks and pebbles (dense scatter) ---
    let pebble_density = 0.12;
    let potential_pebbles = (chunk_size * chunk_size * pebble_density) as u32;

    for i in 0..potential_pebbles {
        let rand_x = pebble_noise.get([i as f64 * 0.13, 400.0]) as f32;
        let rand_z = pebble_noise.get([i as f64 * 0.13, 500.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Pebbles everywhere above water, denser near paths/rivers
        if height < 0.3 {
            continue;
        }

        // Cluster pebbles using noise
        let cluster_noise = pebble_noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]) as f32;
        if cluster_noise < 0.0 {
            continue; // Skip ~50% for natural clustering
        }

        let rock_type = if cluster_noise > 0.6 {
            RockType::SmallRock
        } else {
            RockType::Pebble
        };

        let angle = pebble_noise.get([world_x as f64 * 0.8, world_z as f64 * 0.8]) as f32 * std::f32::consts::TAU;
        let scale_var = pebble_noise.get([world_x as f64 * 0.4, world_z as f64 * 0.4]) as f32;
        let scale = rock_type.base_scale() * (0.7 + scale_var * 0.6);

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_rotation_y(angle),
            Vec3::new(world_x, height - rock_type.sink_amount(), world_z),
        );

        instances.push((rock_type.mesh_name().to_string(), transform));
    }

    instances
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rock_generation() {
        let instances = generate_rocks_for_chunk(
            12345,
            256.0,
            0.0,
            0.0,
        );

        println!("Generated {} rock instances", instances.len());

        let valid_names = ["rock_pebble", "rock_small", "rock_medium", "rock_boulder", "rock_flat", "rock_mossy"];
        for (name, transform) in &instances {
            assert!(valid_names.contains(&name.as_str()), "Unknown rock type: {}", name);
            assert!(transform.w_axis.w == 1.0);
        }
    }
}
