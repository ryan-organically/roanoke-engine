//! Fern Generation System
//!
//! Generates fern instances for the forest understory. Ferns spawn in the
//! DeciduousForest biome and treeline zones with increased density near trees.
//!
//! ## Placement Rules
//! - Biome: Forest and treeline zones (biome_t > 0.65)
//! - Density: 2x rate near trees, base rate elsewhere
//! - Size: 2x base scale (6.0 base, varies 3-18)
//! - Avoids: Water, very low terrain
//!
//! ## Wind Animation
//! Ferns use the standard tree.wgsl shader which derives wind from vertex Y position.
//! With origin at base and tips at Y~0.29m, fronds will have subtle sway.

use crate::mesh_gen::{get_height_at, get_biome_t, calculate_river_depth};
use glam::{Mat4, Quat, Vec3};
use noise::{NoiseFn, Perlin};

/// Fern instance with transform and model index
#[derive(Clone, Debug)]
pub struct FernInstance {
    pub transform: Mat4,
    pub model_index: usize,
}

/// Result of fern generation for a chunk
#[derive(Default)]
pub struct FernInstances {
    pub ferns: Vec<FernInstance>,
}

impl FernInstances {
    /// Get fern transforms grouped by model name (fern_02, fern_03, etc.)
    /// NOTE: fern_01 was corrupted, so we start at fern_02
    pub fn by_model(&self, model_count: usize) -> std::collections::HashMap<String, Vec<Mat4>> {
        let mut result = std::collections::HashMap::new();
        let count = model_count.max(1);
        for inst in &self.ferns {
            // Use fern_02, fern_03 naming (starting at 02 since 01 is corrupted)
            let model_name = format!("fern_{:02}", (inst.model_index % count) + 2);
            result.entry(model_name).or_insert_with(Vec::new).push(inst.transform);
        }
        result
    }

    /// Total fern count
    pub fn len(&self) -> usize {
        self.ferns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ferns.is_empty()
    }
}

/// Height range for forest ferns
const FOREST_HEIGHT_MIN: f32 = 3.0;  // Lower to include treeline
const FOREST_HEIGHT_MAX: f32 = 50.0; // Higher for rolling hills

/// Minimum biome_t for treeline zone (ferns start at treeline)
const TREELINE_BIOME_T: f32 = 0.65;

/// Biome_t for full forest (higher density)
const FOREST_BIOME_T: f32 = 0.72;

/// Generate fern instances for a terrain chunk.
///
/// Ferns spawn only in the DeciduousForest biome at sparse density,
/// creating natural woodland floor coverage.
///
/// # Arguments
/// * `seed` - World seed for deterministic generation
/// * `chunk_size` - Chunk size in world units
/// * `offset_x`, `offset_z` - Chunk world position
/// * `model_count` - Number of fern model variants available
///
/// # Returns
/// FernInstances containing transforms for all fern placements
pub fn generate_ferns_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
    model_count: usize,
) -> FernInstances {
    let noise = Perlin::new(seed + 4242); // Unique seed offset for ferns
    let mut result = FernInstances::default();

    let model_count = model_count.max(1);

    // Grid-based placement with jitter for natural distribution
    // Denser grid (8m) for better forest floor coverage
    let grid_size = 8.0;
    let cells_per_row = (chunk_size / grid_size).ceil() as i32;

    for gz in 0..cells_per_row {
        for gx in 0..cells_per_row {
            // Grid center with jitter
            let grid_x = offset_x + (gx as f32 + 0.5) * grid_size;
            let grid_z = offset_z + (gz as f32 + 0.5) * grid_size;

            // Large jitter for natural scatter
            let jitter_x = noise.get([grid_x as f64 * 0.1, grid_z as f64 * 0.1]) as f32 * grid_size * 0.9;
            let jitter_z = noise.get([grid_x as f64 * 0.1 + 50.0, grid_z as f64 * 0.1]) as f32 * grid_size * 0.9;

            let world_x = grid_x + jitter_x;
            let world_z = grid_z + jitter_z;

            // Get terrain data
            let (height, _) = get_height_at(world_x, world_z, seed);
            let biome_t = get_biome_t(world_x, world_z, seed);

            // === BIOME CHECK: Treeline and Forest zones ===
            // Ferns start at treeline (0.65) and continue into forest
            if biome_t < TREELINE_BIOME_T {
                continue;
            }
            if height < FOREST_HEIGHT_MIN || height > FOREST_HEIGHT_MAX {
                continue;
            }

            // === DENSITY CHECK ===
            // 2x density in treeline (near trees) vs deeper forest
            // Treeline (0.65-0.72): very dense ferns among the trees
            // Forest (0.72+): moderate fern density
            let is_treeline = biome_t < FOREST_BIOME_T;
            let density_threshold = if is_treeline { 0.15 } else { 0.35 };

            let density_roll = (noise.get([world_x as f64 * 0.08, world_z as f64 * 0.08]) + 1.0) * 0.5;
            if density_roll < density_threshold as f64 {
                continue;
            }

            // === LIGHT CLUMPING ===
            // Only skip ~10% for natural gaps
            let clump_noise = noise.get([world_x as f64 * 0.12, world_z as f64 * 0.12]) as f32;
            if clump_noise < -0.8 {
                continue; // Small gaps in fern coverage
            }

            // === SPAWN FERN ===
            // DOUBLED size: base 6.0 (was 3.0), variation 3-18 (was 1.5-9.0)
            let scale_base = 6.0;
            let scale_var = noise.get([world_x as f64 * 0.25, world_z as f64 * 0.25]).abs() as f32;
            let scale_mult = 0.5 + scale_var * 2.5; // 0.5x to 3.0x multiplier
            let scale = scale_base * scale_mult; // Final: 3.0 to 18.0

            let rotation = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * std::f32::consts::TAU;

            // Model selection based on position hash
            let model_idx = ((world_x.abs() as u32).wrapping_mul(73856093)
                ^ (world_z.abs() as u32).wrapping_mul(19349663)) as usize
                % model_count;

            // Sink ferns 10% lower to prevent floating - some may clip like grass
            let y_offset = -0.3 - scale * 0.1;

            result.ferns.push(FernInstance {
                transform: Mat4::from_scale_rotation_translation(
                    Vec3::splat(scale),
                    Quat::from_rotation_y(rotation),
                    Vec3::new(world_x, height + y_offset, world_z),
                ),
                model_index: model_idx,
            });
        }
    }

    // === PHASE 2: RIVER MEGA-FERNS ===
    // Giant ferns along rivers and streams (low height, non-beach)
    // These create lush riparian vegetation

    let river_grid_size = 6.0; // Denser near water
    let river_cells = (chunk_size / river_grid_size).ceil() as i32;

    for gz in 0..river_cells {
        for gx in 0..river_cells {
            let grid_x = offset_x + (gx as f32 + 0.5) * river_grid_size;
            let grid_z = offset_z + (gz as f32 + 0.5) * river_grid_size;

            let jitter_x = noise.get([grid_x as f64 * 0.15, grid_z as f64 * 0.15 + 100.0]) as f32 * river_grid_size * 0.8;
            let jitter_z = noise.get([grid_x as f64 * 0.15 + 100.0, grid_z as f64 * 0.15]) as f32 * river_grid_size * 0.8;

            let world_x = grid_x + jitter_x;
            let world_z = grid_z + jitter_z;

            let (height, _) = get_height_at(world_x, world_z, seed);

            // River ferns still need minimum elevation (above beach/water)
            if height < FOREST_HEIGHT_MIN || height > FOREST_HEIGHT_MAX {
                continue;
            }

            // River zone: use actual river detection
            // calculate_river_depth returns 0.0-1.0 where >0 means in/near river
            let river_depth = calculate_river_depth(world_x, world_z, seed);

            // Also check nearby for riverbank ferns (within ~10m of river)
            let nearby_river = if river_depth < 0.05 {
                let sample_offsets = [(6.0, 0.0), (-6.0, 0.0), (0.0, 6.0), (0.0, -6.0)];
                sample_offsets.iter().any(|(dx, dz)| {
                    calculate_river_depth(world_x + dx, world_z + dz, seed) > 0.15
                })
            } else {
                false
            };

            let is_river_zone = river_depth > 0.05 || nearby_river;

            if !is_river_zone {
                continue;
            }

            // Density roll - ~40% spawn rate along rivers
            let density_roll = (noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]) + 1.0) * 0.5;
            if density_roll < 0.6 {
                continue;
            }

            // HUGE ferns: 12-30 scale (double the forest ferns)
            let scale_base = 12.0;
            let scale_var = noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]).abs() as f32;
            let scale_mult = 1.0 + scale_var * 1.5;
            let scale = scale_base * scale_mult;

            let rotation = noise.get([world_x as f64 * 0.4, world_z as f64 * 0.4]) as f32 * std::f32::consts::TAU;

            let model_idx = ((world_x.abs() as u32).wrapping_mul(73856093)
                ^ (world_z.abs() as u32).wrapping_mul(19349663)) as usize
                % model_count;

            let y_offset = -0.5 - scale * 0.08;

            result.ferns.push(FernInstance {
                transform: Mat4::from_scale_rotation_translation(
                    Vec3::splat(scale),
                    Quat::from_rotation_y(rotation),
                    Vec3::new(world_x, height + y_offset, world_z),
                ),
                model_index: model_idx,
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fern_generation() {
        let result = generate_ferns_for_chunk(12345, 256.0, 0.0, 0.0, 1);
        println!("Generated {} ferns", result.len());

        // Verify transforms are valid
        for inst in &result.ferns {
            assert!(inst.transform.w_axis.w == 1.0, "Invalid transform");
        }
    }

    #[test]
    fn test_fern_by_model() {
        let result = generate_ferns_for_chunk(12345, 256.0, 0.0, 0.0, 2);
        let by_model = result.by_model(2);

        println!("Ferns by model:");
        for (name, transforms) in &by_model {
            println!("  {}: {} instances", name, transforms.len());
        }

        // Should have fern_01 and/or fern_02
        for name in by_model.keys() {
            assert!(name.starts_with("fern_"), "Invalid model name: {}", name);
        }
    }

    #[test]
    fn test_forest_biome_only() {
        // Generate in a chunk and verify all ferns are in valid forest range
        let seed = 42;
        let result = generate_ferns_for_chunk(seed, 256.0, -128.0, -128.0, 1);

        for inst in &result.ferns {
            let pos = inst.transform.w_axis.truncate();
            let (height, _) = get_height_at(pos.x, pos.z, seed);
            let biome_t = get_biome_t(pos.x, pos.z, seed);

            assert!(
                biome_t >= TREELINE_BIOME_T,
                "Fern outside treeline/forest biome: biome_t={} at ({}, {})",
                biome_t, pos.x, pos.z
            );
            assert!(
                height >= FOREST_HEIGHT_MIN && height <= FOREST_HEIGHT_MAX,
                "Fern outside forest elevation: height={} at ({}, {})",
                height, pos.x, pos.z
            );
        }
    }
}
