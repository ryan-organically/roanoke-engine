use crate::mesh_gen::get_height_at;
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec3, Quat};

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
    let noise = Perlin::new(seed + 888); // Different seed offset for rocks

    // Density settings
    let rock_density = 0.04; // Increased from 0.01
    let potential_rocks = (chunk_size * chunk_size * rock_density) as u32;

    let mut instances = Vec::new();

    for i in 0..potential_rocks {
        // Pseudo-random position within chunk
        let rand_x = noise.get([i as f64 * 0.1, 200.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 300.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        // Get terrain height
        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Calculate Slope (approximate by sampling neighbors)
        let sample_dist = 1.0;
        let (h_dx, _) = get_height_at(world_x + sample_dist, world_z, seed);
        let (h_dz, _) = get_height_at(world_x, world_z + sample_dist, seed);
        let slope_x = (h_dx - height) / sample_dist;
        let slope_z = (h_dz - height) / sample_dist;
        let slope = (slope_x * slope_x + slope_z * slope_z).sqrt();

        // --- Placement Logic ---

        // 1. Slope Constraint: Rocks like slopes, but not vertical cliffs (too unstable)
        // Slope > 0.5 is steep
        let is_steep = slope > 0.3;

        // 2. Biome Constraint: "RockyScrub" or "RiverBank"
        // Use a noise map to define rocky areas
        let rocky_noise = noise.get([world_x as f64 * 0.05, world_z as f64 * 0.05]) as f32;
        let is_rocky_biome = rocky_noise > 0.2;

        // 3. Height Constraint: Avoid deep water, but allow river banks/beaches
        let is_above_water = height > 0.5;

        // Decision
        let should_place = is_above_water && (is_steep || is_rocky_biome);

        if !should_place {
            continue;
        }

        // Random rotation
        let angle = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * 3.14;
        
        // Scale variation
        let base_scale = 1.0; 
        let scale_var = noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]) as f32;
        let scale = base_scale + scale_var * 0.5;

        // Create transform matrix
        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_rotation_y(angle),
            Vec3::new(world_x, height - 0.2, world_z), // Sink slightly
        );

        // For now, we only have one rock type: "rock_boulder"
        instances.push(("rock_boulder".to_string(), transform));
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
        
        for (name, instance) in instances {
            assert_eq!(name, "rock_boulder");
            assert!(instance.w_axis.w == 1.0);
        }
    }
}
