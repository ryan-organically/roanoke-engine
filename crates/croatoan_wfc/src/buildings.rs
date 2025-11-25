use crate::mesh_gen::get_height_at;
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec3, Quat};

/// Generate buildings for a terrain chunk based on terrain features
///
/// Buildings require flat ground and are sparse.
/// Returns a list of (mesh_name, transform) tuples.
pub fn generate_buildings_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<(String, Mat4)> {
    let noise = Perlin::new(seed + 999); // Different seed offset for buildings

    // Density settings: Very sparse (e.g., 1 per 2 chunks on average)
    // We check a grid of potential sites
    let site_spacing = 100.0; 
    let grid_size = (chunk_size / site_spacing).ceil() as u32;

    let mut instances = Vec::new();

    for x in 0..grid_size {
        for z in 0..grid_size {
            // Potential site center
            let local_x = x as f32 * site_spacing + site_spacing * 0.5;
            let local_z = z as f32 * site_spacing + site_spacing * 0.5;

            // Add some jitter
            let jitter_x = noise.get([local_x as f64 * 0.1, 0.0]) as f32 * 20.0;
            let jitter_z = noise.get([0.0, local_z as f64 * 0.1]) as f32 * 20.0;

            let world_x = offset_x + local_x + jitter_x;
            let world_z = offset_z + local_z + jitter_z;

            // Check bounds (don't spawn too close to edge to avoid mesh clipping)
            if world_x < offset_x + 10.0 || world_x > offset_x + chunk_size - 10.0 ||
               world_z < offset_z + 10.0 || world_z > offset_z + chunk_size - 10.0 {
                continue;
            }

            // 1. Density Check (Noise)
            let density_roll = noise.get([world_x as f64 * 0.01, world_z as f64 * 0.01]) as f32;
            if density_roll < 0.6 { // Only top 20% of noise range (0.6 to 1.0 approx)
                continue;
            }

            // 2. Flatness Check
            // Sample height at center and corners of a 10x10 footprint
            let (h_center, _) = get_height_at(world_x, world_z, seed);
            
            // Water check
            if h_center < 2.0 { // Avoid beaches/water
                continue;
            }

            let footprint = 5.0;
            let (h_n, _) = get_height_at(world_x, world_z - footprint, seed);
            let (h_s, _) = get_height_at(world_x, world_z + footprint, seed);
            let (h_e, _) = get_height_at(world_x + footprint, world_z, seed);
            let (h_w, _) = get_height_at(world_x - footprint, world_z, seed);

            let max_diff = (h_center - h_n).abs()
                .max((h_center - h_s).abs())
                .max((h_center - h_e).abs())
                .max((h_center - h_w).abs());

            if max_diff > 1.5 { // Too steep
                continue;
            }

            // Place Building
            let angle = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * 3.14;
            
            let transform = Mat4::from_scale_rotation_translation(
                Vec3::splat(1.0),
                Quat::from_rotation_y(angle),
                Vec3::new(world_x, h_center, world_z),
            );

            // Determine type based on noise or random
            // For now, just "building_cabin"
            instances.push(("building_cabin".to_string(), transform));
        }
    }

    instances
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_generation() {
        let instances = generate_buildings_for_chunk(
            12345,
            256.0,
            0.0,
            0.0,
        );

        println!("Generated {} building instances", instances.len());
        
        for (name, instance) in instances {
            assert_eq!(name, "building_cabin");
            assert!(instance.w_axis.w == 1.0);
        }
    }
}
