use croatoan_procgen::{generate_grass_patch, GrassBladeRecipe};
use crate::mesh_gen::get_height_at;

/// Generate vegetation (grass) for a terrain chunk based on biome
///
/// Returns (positions, colors, indices) for grass mesh
pub fn generate_vegetation_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let recipe = GrassBladeRecipe {
        height_range: (0.2, 0.6),
        blade_segments: 3,
        curve_factor: 0.4,
        width_base: 0.06,
        width_tip: 0.01,
        color_base: [0.15, 0.4, 0.1],
        color_tip: [0.3, 0.6, 0.2],
    };

    // Grass density varies by biome
    let base_density = 2.0; // blades per square unit

    generate_grass_patch(
        &recipe,
        seed,
        (offset_x, offset_z),
        chunk_size,
        base_density,
        |x, z| {
            // Get terrain height from existing terrain system
            let (height, _color) = get_height_at(x, z, seed);
            height
        },
        |x, z| {
            // Biome filter: Only spawn grass in certain biomes
            let (height, _color) = get_height_at(x, z, seed);

            // Grass grows in:
            // - Beaches (above water line)
            // - Subtropical Scrub
            // - Coastal Forest (sparse in forest)

            // No grass underwater or on deep sand
            if height < 0.5 {
                return false;
            }

            // Reduce density in forest (t > 0.75)
            // For now, allow grass everywhere above water
            height > 0.5 && height < 20.0
        },
    )
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
