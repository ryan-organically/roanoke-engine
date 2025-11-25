use croatoan_procgen::{TreeRecipe, generate_tree, generate_tree_mesh};
use crate::mesh_gen::get_height_at;
use noise::{NoiseFn, Perlin};

#[derive(Clone)]
pub struct TreeTemplate {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

use glam::{Mat4, Vec3, Quat};

/// Generate trees for a terrain chunk based on biome
///
/// Trees appear at forest edge and become denser in deep forest
/// Returns instance matrices for the chunk
pub fn generate_trees_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<Mat4> {
    let noise = Perlin::new(seed + 777);

    // Sample potential tree positions
    // Optimization: Reduced density slightly to prevent overcrowding while maintaining lush look
    let tree_density = 0.005; 
    let potential_trees = (chunk_size * chunk_size * tree_density) as u32;

    let mut instances = Vec::new();

    // Pre-calculate constants for performance
    let lower_treeline = 12.0;
    let upper_treeline_start = 40.0;
    let upper_treeline_end = 55.0;

    for i in 0..potential_trees {
        // Pseudo-random position within chunk
        let rand_x = noise.get([i as f64 * 0.1, 0.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 100.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        // Get terrain height and determine biome
        let (height, _color) = get_height_at(world_x, world_z, seed);

        // --- Treeline Logic ---

        // 1. Lower Treeline (Coastal/Beach)
        if height < lower_treeline {
            continue; // No trees in beach or scrub
        }

        // 2. Upper Treeline (Alpine/Mountain)
        // Trees start fading out at `upper_treeline_start` and are gone by `upper_treeline_end`
        if height > upper_treeline_end {
            continue; // Above timberline
        }

        // Calculate biome factor (0.0 = forest edge start, 1.0 = deep forest)
        let mut biome_factor = ((height - lower_treeline) / 10.0).clamp(0.0, 1.0);

        // Apply upper treeline fade
        if height > upper_treeline_start {
            let fade = 1.0 - ((height - upper_treeline_start) / (upper_treeline_end - upper_treeline_start));
            biome_factor *= fade.clamp(0.0, 1.0);
        }

        // Density increases with height (forest edge = 40%, deep forest = 80%)
        // Adjusted for upper treeline fade
        let density_threshold = 0.4 + biome_factor * 0.4;
        
        // Use a different noise frequency for density map to create clumps/clearings
        let density_roll = noise.get([world_x as f64 * 0.02, world_z as f64 * 0.02]) as f32;
        if (density_roll + 1.0) * 0.5 > density_threshold {
            continue; // Skip this tree based on density
        }

        // Random rotation
        let angle = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * 3.14;
        
        // Scale variation: Taller in deep forest, shorter at edges (both coastal and alpine)
        let base_scale = 5.0 + (biome_factor * 2.0); 
        let scale_var = noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]) as f32;
        let scale = base_scale + scale_var;

        // Create transform matrix
        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_rotation_y(angle),
            Vec3::new(world_x, height - 0.5, world_z), // -0.5 to sink slightly into ground
        );

        instances.push(transform);
    }

    instances
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_generation() {
        let instances = generate_trees_for_chunk(
            12345,
            256.0,
            0.0,
            0.0,
        );

        // Should generate some trees (depends on seed and chunk)
        println!("Generated {} tree instances", instances.len());
        
        // Basic validation
        for instance in instances {
            // Check if matrix is valid (not all zeros)
            assert!(instance.w_axis.w == 1.0);
        }
    }
}
