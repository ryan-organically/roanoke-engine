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
        height_range: (0.4, 1.2),  // Taller grass (was 0.2-0.6)
        blade_segments: 4,          // More segments for smoother curves
        curve_factor: 0.5,          // More curve for natural look
        width_base: 0.08,           // Slightly wider base
        width_tip: 0.01,
        color_base: [0.12, 0.35, 0.08],  // Darker, richer green
        color_tip: [0.25, 0.55, 0.15],   // Brighter tips
    };

    // Grass density varies by biome
    let base_density = 2.5; // blades per square unit (increased)

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
            // Biome filter: Grass starts at end of beach
            let (height, _color) = get_height_at(x, z, seed);

            // Grass starts where beach ends and scrub begins
            // Beach ends at ~height 1.5-2.0 (t=0.53-0.55)
            // Scrub: height 2.0-6.0
            // Forest: height 6.0+

            // Start grass at end of beach (height > 1.5)
            // Continue through scrub and into forest
            if height < 1.5 {
                return false;
            }

            // Allow grass up to forest heights
            height >= 1.5 && height < 20.0
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
