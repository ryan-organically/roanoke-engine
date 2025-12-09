//! Multi-model foliage generation (trees + shrubs with variety)
//!
//! Generates foliage instances with model indices for multi-model rendering.

use crate::mesh_gen::{get_height_at, distance_to_shoreline, get_biome_t};
use crate::trees::{TREELINE_DISTANCE, UPPER_TREELINE_START, UPPER_TREELINE_END};
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec3, Quat};
use std::collections::HashMap;

/// Instance with model index for multi-model rendering
#[derive(Clone, Debug)]
pub struct FoliageInstance {
    pub transform: Mat4,
    pub model_index: usize,
}

/// Result of foliage generation with model variety
#[derive(Default)]
pub struct FoliageInstances {
    /// Tree instances with model index
    pub trees: Vec<FoliageInstance>,
    /// Shrub instances with model index
    pub shrubs: Vec<FoliageInstance>,
}

impl FoliageInstances {
    /// Get tree transforms grouped by model name (tree_0, tree_1, etc.)
    pub fn trees_by_model(&self, model_count: usize) -> HashMap<String, Vec<Mat4>> {
        let mut result = HashMap::new();
        let count = model_count.max(1);
        for inst in &self.trees {
            let model_name = format!("tree_{}", inst.model_index % count);
            result.entry(model_name).or_insert_with(Vec::new).push(inst.transform);
        }
        result
    }

    /// Get shrub transforms grouped by model name (shrub_0, shrub_1, etc.)
    pub fn shrubs_by_model(&self, model_count: usize) -> HashMap<String, Vec<Mat4>> {
        let mut result = HashMap::new();
        let count = model_count.max(1);
        for inst in &self.shrubs {
            let model_name = format!("shrub_{}", inst.model_index % count);
            result.entry(model_name).or_insert_with(Vec::new).push(inst.transform);
        }
        result
    }

    /// Combine tree and shrub instances into a single map
    pub fn all_by_model(&self, tree_count: usize, shrub_count: usize) -> HashMap<String, Vec<Mat4>> {
        let mut result = self.trees_by_model(tree_count);
        result.extend(self.shrubs_by_model(shrub_count));
        result
    }
}

/// Generate foliage (trees + shrubs) with model variety for a chunk.
///
/// Creates a layered forest with:
/// - **Canopy layer**: Large trees (models named tree_0 through tree_N)
/// - **Understory layer**: Shrubs (models named shrub_0 through shrub_N)
///
/// # Arguments
/// * `seed` - World seed
/// * `chunk_size` - Chunk size in world units
/// * `offset_x`, `offset_z` - Chunk world position
/// * `tree_model_count` - Number of tree models (named tree_0, tree_1, etc.)
/// * `shrub_model_count` - Number of shrub models (named shrub_0, shrub_1, etc.)
pub fn generate_foliage_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
    tree_model_count: usize,
    shrub_model_count: usize,
) -> FoliageInstances {
    let noise = Perlin::new(seed + 777);
    let mut result = FoliageInstances::default();

    let tree_count = tree_model_count.max(1);
    let shrub_count = shrub_model_count.max(1);

    let bunch_grid_size = 32.0;
    let bunches_per_row = (chunk_size / bunch_grid_size).ceil() as i32;

    for bz in 0..bunches_per_row {
        for bx in 0..bunches_per_row {
            let grid_x = offset_x + (bx as f32 + 0.5) * bunch_grid_size;
            let grid_z = offset_z + (bz as f32 + 0.5) * bunch_grid_size;
            let jitter_x = noise.get([grid_x as f64 * 0.1, grid_z as f64 * 0.1]) as f32 * bunch_grid_size * 0.4;
            let jitter_z = noise.get([grid_x as f64 * 0.1, grid_z as f64 * 0.1 + 100.0]) as f32 * bunch_grid_size * 0.4;
            let world_x = grid_x + jitter_x;
            let world_z = grid_z + jitter_z;

            let biome_t = get_biome_t(world_x, world_z, seed);
            let (height, _) = get_height_at(world_x, world_z, seed);

            if biome_t < 0.52 || height < 1.5 {
                continue;
            }

            let bunch_threshold = if biome_t < 0.65 { 0.3 } else { 0.5 };
            let density_roll = (noise.get([world_x as f64 * 0.05, world_z as f64 * 0.05]) + 1.0) * 0.5;
            if density_roll < bunch_threshold {
                continue;
            }

            let shore_dist = distance_to_shoreline(world_x, world_z, seed);
            let beyond_treeline = shore_dist > TREELINE_DISTANCE;
            let biome_factor = if biome_t > 0.65 {
                ((biome_t - 0.65) / 0.35).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let above_upper = height > UPPER_TREELINE_END;
            let in_transition = height > UPPER_TREELINE_START && height <= UPPER_TREELINE_END;
            let mut has_tree = beyond_treeline && !above_upper;
            if in_transition {
                let fade = 1.0 - (height - UPPER_TREELINE_START) / (UPPER_TREELINE_END - UPPER_TREELINE_START);
                let roll = (noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3]) + 1.0) * 0.5;
                has_tree = has_tree && roll < fade as f64;
            }

            let local_seed = seed.wrapping_add((world_x as u32) ^ (world_z as u32).rotate_left(16));
            let local_noise = Perlin::new(local_seed);

            // SHRUBS (understory) - 2-4 per bunch
            let shrub_local = 2 + ((local_noise.get([local_seed as f64 * 0.1, 60.0]) + 1.0) * 1.5) as u32;
            for i in 0..shrub_local {
                let shrub_angle = (i as f32 * std::f32::consts::PI * 0.7)
                    + local_noise.get([local_seed as f64, i as f64]) as f32 * 0.8;
                let shrub_dist = 3.0
                    + local_noise.get([local_seed as f64 * 0.3, i as f64 * 10.0]).abs() as f32 * 5.0;
                let sx = world_x + shrub_angle.cos() * shrub_dist;
                let sz = world_z + shrub_angle.sin() * shrub_dist;
                let (sh, _) = get_height_at(sx, sz, seed);
                if sh < 0.5 {
                    continue;
                }

                let shrub_scale =
                    0.8 + local_noise.get([sx as f64 * 0.2, sz as f64 * 0.2]).abs() as f32 * 0.6;
                let shrub_rot =
                    local_noise.get([sx as f64 * 0.5, sz as f64 * 0.5]) as f32 * std::f32::consts::TAU;
                let model_idx = ((sx.abs() as u32).wrapping_mul(73856093)
                    ^ (sz.abs() as u32).wrapping_mul(19349663)) as usize
                    % shrub_count;

                result.shrubs.push(FoliageInstance {
                    transform: Mat4::from_scale_rotation_translation(
                        Vec3::splat(shrub_scale),
                        Quat::from_rotation_y(shrub_rot),
                        Vec3::new(sx, sh - 0.1, sz),
                    ),
                    model_index: model_idx,
                });
            }

            // TREE (canopy)
            if has_tree {
                let tx = world_x + local_noise.get([local_seed as f64 * 0.2, 300.0]) as f32 * 3.0;
                let tz = world_z + local_noise.get([local_seed as f64 * 0.2, 400.0]) as f32 * 3.0;
                let (th, _) = get_height_at(tx, tz, seed);
                let base_scale = 5.0 + biome_factor * 3.0;
                let tree_scale =
                    base_scale + local_noise.get([tx as f64 * 0.2, tz as f64 * 0.2]) as f32;
                let tree_angle =
                    local_noise.get([tx as f64 * 0.5, tz as f64 * 0.5]) as f32 * std::f32::consts::TAU;
                let model_idx = ((tx.abs() as u32).wrapping_mul(83492791)
                    ^ (tz.abs() as u32).wrapping_mul(41729563)) as usize
                    % tree_count;

                result.trees.push(FoliageInstance {
                    transform: Mat4::from_scale_rotation_translation(
                        Vec3::splat(tree_scale),
                        Quat::from_rotation_y(tree_angle),
                        Vec3::new(tx, th - 1.0, tz),
                    ),
                    model_index: model_idx,
                });
            }
        }
    }

    // Phase 2: Scattered forest trees
    let scattered_density = 0.0002;
    let potential = (chunk_size * chunk_size * scattered_density) as u32;
    for i in 0..potential {
        let rx = noise.get([i as f64 * 0.1, 700.0]) as f32;
        let rz = noise.get([i as f64 * 0.1, 800.0]) as f32;
        let world_x = offset_x + (rx + 1.0) * 0.5 * chunk_size;
        let world_z = offset_z + (rz + 1.0) * 0.5 * chunk_size;
        let biome_t = get_biome_t(world_x, world_z, seed);
        let (height, _) = get_height_at(world_x, world_z, seed);

        if biome_t < 0.65 {
            continue;
        }
        if distance_to_shoreline(world_x, world_z, seed) < TREELINE_DISTANCE {
            continue;
        }
        if height > UPPER_TREELINE_END {
            continue;
        }

        let forest_depth = (biome_t - 0.65) / 0.35;
        let threshold = 0.3 + forest_depth * 0.5;
        if (noise.get([world_x as f64 * 0.02, world_z as f64 * 0.02]) + 1.0) * 0.5 > threshold as f64
        {
            continue;
        }

        if height > UPPER_TREELINE_START {
            let fade =
                1.0 - (height - UPPER_TREELINE_START) / (UPPER_TREELINE_END - UPPER_TREELINE_START);
            if (noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3 + 50.0]) + 1.0) * 0.5
                > fade as f64
            {
                continue;
            }
        }

        let angle =
            noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * std::f32::consts::TAU;
        let scale = 5.5
            + forest_depth * 2.5
            + noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]) as f32;
        let model_idx = ((world_x.abs() as u32).wrapping_mul(83492791)
            ^ (world_z.abs() as u32).wrapping_mul(41729563)) as usize
            % tree_count;

        result.trees.push(FoliageInstance {
            transform: Mat4::from_scale_rotation_translation(
                Vec3::splat(scale),
                Quat::from_rotation_y(angle),
                Vec3::new(world_x, height - 1.0, world_z),
            ),
            model_index: model_idx,
        });
    }

    println!(
        "[FOLIAGE] Chunk ({}, {}): {} trees, {} shrubs",
        offset_x,
        offset_z,
        result.trees.len(),
        result.shrubs.len()
    );
    result
}
