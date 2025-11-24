use croatoan_procgen::{GrassBladeRecipe, generate_grass_blade};
use crate::mesh_gen::get_height_at;
use glam::Vec3;
use noise::{NoiseFn, Perlin};

/// Generate vegetation (grass) for a terrain chunk based on biome
///
/// Grass density and height increase toward forest edge
/// Returns (positions, colors, indices) for grass mesh
pub fn generate_vegetation_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let noise = Perlin::new(seed + 999);

    // Maximum density for sampling positions
    let max_density = 20.0; // Balanced grass coverage (increased from 12.0, reduced from 40.0 for GPU limits)
    let blade_count = (chunk_size * chunk_size * max_density) as u32;

    let mut all_positions = Vec::new();
    let mut all_colors = Vec::new();
    let mut all_indices = Vec::new();

    for i in 0..blade_count {
        // Pseudo-random position within chunk
        let rand_x = noise.get([i as f64 * 0.1, 0.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 100.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        // Get terrain height and determine biome
        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Beach: height < 1.5 (no grass)
        // Scrub: height 1.5-6.0 (sparse, short grass)
        // Forest edge: height 6.0-12.0 (dense, tall grass)
        // Deep forest: height 12.0+ (very dense, very tall grass)

        if height < 1.5 {
            continue; // No grass on beach
        }

        // Calculate biome factor (0.0 = scrub start, 1.0 = deep forest)
        let biome_factor = ((height - 1.5) / 15.0).clamp(0.0, 1.0);

        // Density increases with height (scrub = 10%, forest = 100%)
        let density_threshold = 0.1 + biome_factor * 0.9;
        let density_roll = noise.get([world_x as f64 * 3.7, world_z as f64 * 3.7]) as f32;
        if (density_roll + 1.0) * 0.5 > density_threshold {
            continue; // Skip this blade based on density
        }

        // Height range increases toward forest
        // Scrub: 0.4-0.8m
        // Forest edge: 0.8-1.6m
        // Deep forest: 1.2-2.4m
        let min_height = 0.4 + biome_factor * 0.8;
        let max_height = 0.8 + biome_factor * 1.6;

        let recipe = GrassBladeRecipe {
            height_range: (min_height, max_height),
            blade_segments: 5,
            curve_factor: 0.4 + biome_factor * 0.3, // More curve in forest
            width_base: 0.06 + biome_factor * 0.04,
            width_tip: 0.01,
            color_base: [
                0.12 - biome_factor * 0.04,  // Darker in forest
                0.30 + biome_factor * 0.10,  // More saturated
                0.08,
            ],
            color_tip: [
                0.25 - biome_factor * 0.05,
                0.50 + biome_factor * 0.15,
                0.12,
            ],
        };

        let base_pos = Vec3::new(world_x, height, world_z);
        let blade = generate_grass_blade(&recipe, seed + i, base_pos);

        // Append to combined mesh
        let vertex_offset = all_positions.len() as u32;
        all_positions.extend(blade.positions);
        all_colors.extend(blade.colors);
        all_indices.extend(blade.indices.iter().map(|idx| idx + vertex_offset));
    }

    (all_positions, all_colors, all_indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vegetation_generation() {
        let (positions, colors, indices) = generate_vegetation_for_chunk(
            1587,
            32.0,
            0.0,
            0.0,
        );

        // Should generate some grass
        assert!(!positions.is_empty());
        assert_eq!(positions.len(), colors.len());
        assert!(indices.len() % 3 == 0);

        println!("Generated {} grass blades", positions.len() / 10); // ~10 verts per blade
    }
}
