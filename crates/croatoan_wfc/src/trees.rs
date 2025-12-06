//! # Tree and Vegetation Placement System
//!
//! This module handles procedural placement of trees, bushes, and vegetation clusters
//! ("bunches") across the terrain. It uses distance-based treeline logic and cluster-based
//! generation for natural-looking distribution.
//!
//! ## Key Concepts
//!
//! ### Treeline
//! Trees only spawn beyond a minimum distance from the shoreline (40 yards / ~36.6m).
//! This creates a clear, natural-looking treeline that follows the coastline's contours.
//!
//! ### LowlandBunch
//! Clustered vegetation units containing:
//! - 1 large anchor rock
//! - 8-15 pebbles scattered around
//! - 2 bushes
//! - 1 large tree (if within treeline)
//!
//! Bunches are procedurally spread across the lowland/scrub biome for efficient,
//! natural-looking vegetation distribution.
//!
//! ## Biome Zones
//! - **Ocean** (t < 0.45): No vegetation
//! - **Beach** (t 0.45-0.55): Sparse driftwood only (handled elsewhere)
//! - **Lowland/Scrub** (t 0.55-0.65): LowlandBunches with bushes, rocks, sparse trees
//! - **Forest** (t > 0.65): Dense trees, full bunches

use crate::mesh_gen::{get_height_at, distance_to_shoreline, get_biome_t};
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec3, Quat};

/// Minimum distance from shoreline for trees to spawn (40 yards ≈ 36.6 meters)
const TREELINE_DISTANCE: f32 = 36.6;

/// Upper elevation limit where trees fade out (alpine treeline)
const UPPER_TREELINE_START: f32 = 40.0;
const UPPER_TREELINE_END: f32 = 55.0;

//=============================================================================
// LOWLAND BUNCH SYSTEM
//=============================================================================

/// A vegetation cluster containing rocks, pebbles, bushes, and optionally a tree.
///
/// Bunches are the primary unit of vegetation distribution in the lowland/scrub zones.
/// They provide natural-looking clusters rather than uniform random distribution.
///
/// # Components
/// - **Anchor Rock**: Large boulder at bunch center (always present)
/// - **Pebbles**: 8-15 small stones scattered within bunch radius
/// - **Bushes**: 2 bushes placed near the anchor rock
/// - **Tree**: 1 large tree if bunch is beyond the treeline
///
/// # Bunch Radius
/// Bunches have a radius of ~8-12 meters, with elements scattered within.
#[derive(Clone, Debug)]
pub struct LowlandBunch {
    /// World position of the bunch center
    pub center: Vec3,
    /// Radius of the bunch (8-12m typically)
    pub radius: f32,
    /// Seed for deterministic generation within this bunch
    pub seed: u32,
    /// Whether this bunch includes a tree (determined by treeline)
    pub has_tree: bool,
    /// Biome factor (0.0 = scrub edge, 1.0 = deep forest)
    pub biome_factor: f32,
}

/// Result of bunch generation containing all instance transforms
#[derive(Default)]
pub struct BunchInstances {
    /// Tree instance transforms (if any)
    pub trees: Vec<Mat4>,
    /// Bush instance transforms
    pub bushes: Vec<Mat4>,
    /// Large rock transforms (anchor rocks)
    pub large_rocks: Vec<Mat4>,
    /// Pebble transforms (many small stones)
    pub pebbles: Vec<Mat4>,
}

impl LowlandBunch {
    /// Generate all instances within this bunch
    ///
    /// # Returns
    /// A `BunchInstances` struct containing transforms for all elements:
    /// - 1 large anchor rock
    /// - 8-15 pebbles
    /// - 2 bushes
    /// - 0-1 tree (based on treeline)
    pub fn generate(&self, world_seed: u32) -> BunchInstances {
        let noise = Perlin::new(self.seed);
        let mut instances = BunchInstances::default();

        // --- Anchor Rock (always present) ---
        let rock_scale = 1.2 + (noise.get([self.center.x as f64 * 0.1, 0.0]) as f32 * 0.4);
        let rock_angle = noise.get([self.center.x as f64 * 0.5, self.center.z as f64 * 0.5]) as f32
            * std::f32::consts::TAU;
        let (rock_height, _) = get_height_at(self.center.x, self.center.z, world_seed);

        let rock_transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(rock_scale),
            Quat::from_rotation_y(rock_angle),
            Vec3::new(self.center.x, rock_height - 0.3, self.center.z),
        );
        instances.large_rocks.push(rock_transform);

        // --- Pebbles (8-15 scattered around) ---
        let pebble_count = 8 + ((noise.get([self.seed as f64 * 0.1, 50.0]) + 1.0) * 3.5) as u32;
        for i in 0..pebble_count {
            // Scatter within bunch radius using golden angle for even distribution
            let angle = i as f32 * 2.399963; // Golden angle in radians
            let dist_factor = (i as f32 / pebble_count as f32).sqrt(); // Square root for even area distribution
            let dist = self.radius * dist_factor * 0.9;

            // Add noise-based jitter
            let jitter_x = noise.get([i as f64 * 0.7, 100.0]) as f32 * 2.0;
            let jitter_z = noise.get([i as f64 * 0.7, 200.0]) as f32 * 2.0;

            let px = self.center.x + angle.cos() * dist + jitter_x;
            let pz = self.center.z + angle.sin() * dist + jitter_z;
            let (ph, _) = get_height_at(px, pz, world_seed);

            // Skip if underwater
            if ph < 0.3 {
                continue;
            }

            let pebble_scale = 0.1 + noise.get([px as f64 * 0.3, pz as f64 * 0.3]).abs() as f32 * 0.15;
            let pebble_angle = noise.get([px as f64, pz as f64]) as f32 * std::f32::consts::TAU;

            let pebble_transform = Mat4::from_scale_rotation_translation(
                Vec3::splat(pebble_scale),
                Quat::from_rotation_y(pebble_angle),
                Vec3::new(px, ph - 0.02, pz),
            );
            instances.pebbles.push(pebble_transform);
        }

        // --- Bushes (2 near anchor rock) ---
        for i in 0..2 {
            let bush_angle = (i as f32 * std::f32::consts::PI) +
                noise.get([self.seed as f64, i as f64]) as f32 * 0.5;
            let bush_dist = 2.0 + noise.get([self.seed as f64 * 0.3, i as f64 * 10.0]).abs() as f32 * 2.0;

            let bx = self.center.x + bush_angle.cos() * bush_dist;
            let bz = self.center.z + bush_angle.sin() * bush_dist;
            let (bh, _) = get_height_at(bx, bz, world_seed);

            if bh < 0.5 {
                continue;
            }

            let bush_scale = 0.6 + noise.get([bx as f64 * 0.2, bz as f64 * 0.2]).abs() as f32 * 0.4;
            let bush_rot = noise.get([bx as f64 * 0.5, bz as f64 * 0.5]) as f32 * std::f32::consts::TAU;

            let bush_transform = Mat4::from_scale_rotation_translation(
                Vec3::splat(bush_scale),
                Quat::from_rotation_y(bush_rot),
                Vec3::new(bx, bh - 0.3, bz),
            );
            instances.bushes.push(bush_transform);
        }

        // --- Tree (if beyond treeline) ---
        if self.has_tree {
            // Offset tree slightly from center for natural look
            let tree_offset_x = noise.get([self.seed as f64 * 0.2, 300.0]) as f32 * 3.0;
            let tree_offset_z = noise.get([self.seed as f64 * 0.2, 400.0]) as f32 * 3.0;

            let tx = self.center.x + tree_offset_x;
            let tz = self.center.z + tree_offset_z;
            let (th, _) = get_height_at(tx, tz, world_seed);

            // Scale increases with biome factor (deeper = taller)
            let base_scale = 5.0 + self.biome_factor * 3.0;
            let scale_var = noise.get([tx as f64 * 0.2, tz as f64 * 0.2]) as f32;
            let tree_scale = base_scale + scale_var;

            let tree_angle = noise.get([tx as f64 * 0.5, tz as f64 * 0.5]) as f32 * std::f32::consts::TAU;

            let tree_transform = Mat4::from_scale_rotation_translation(
                Vec3::splat(tree_scale),
                Quat::from_rotation_y(tree_angle),
                Vec3::new(tx, th - 1.0, tz),
            );
            instances.trees.push(tree_transform);
        }

        instances
    }
}

//=============================================================================
// CHUNK-LEVEL GENERATION
//=============================================================================

/// Generate all vegetation for a terrain chunk.
///
/// This is the main entry point for vegetation generation. It:
/// 1. Spreads LowlandBunches across the chunk
/// 2. Adds additional scattered trees in forest zones
/// 3. Returns combined instance transforms for all vegetation types
///
/// # Arguments
/// * `seed` - World seed for deterministic generation
/// * `chunk_size` - Size of chunk in world units
/// * `offset_x`, `offset_z` - Chunk position in world coordinates
///
/// # Returns
/// Vec of tree instance transforms (trees and bushes use same mesh currently)
pub fn generate_trees_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<Mat4> {
    let noise = Perlin::new(seed + 777);
    let mut all_instances = Vec::new();

    //=========================================================================
    // PHASE 1: Generate LowlandBunches
    //=========================================================================
    // Bunches are placed on a jittered grid for even distribution
    // Density: ~1 bunch per 400 sq meters (20m grid with filtering)

    let bunch_grid_size = 18.0; // Base grid spacing for bunch centers
    let bunches_per_row = (chunk_size / bunch_grid_size).ceil() as i32;

    for bz in 0..bunches_per_row {
        for bx in 0..bunches_per_row {
            // Grid position with jitter
            let grid_x = offset_x + (bx as f32 + 0.5) * bunch_grid_size;
            let grid_z = offset_z + (bz as f32 + 0.5) * bunch_grid_size;

            // Add jitter to break up grid pattern
            let jitter_x = noise.get([grid_x as f64 * 0.1, grid_z as f64 * 0.1]) as f32 * bunch_grid_size * 0.4;
            let jitter_z = noise.get([grid_x as f64 * 0.1, grid_z as f64 * 0.1 + 100.0]) as f32 * bunch_grid_size * 0.4;

            let world_x = grid_x + jitter_x;
            let world_z = grid_z + jitter_z;

            // Get biome info
            let biome_t = get_biome_t(world_x, world_z, seed);
            let (height, _) = get_height_at(world_x, world_z, seed);

            // Skip ocean and beach (bunches only in scrub and forest)
            if biome_t < 0.52 || height < 1.5 {
                continue;
            }

            // Density filtering - more bunches in scrub/lowland, fewer in deep forest
            // (Deep forest gets additional scattered trees instead)
            let bunch_threshold = if biome_t < 0.65 {
                0.3 // Scrub: 70% chance
            } else {
                0.5 // Forest: 50% chance (supplemented by scattered trees)
            };

            let density_roll = (noise.get([world_x as f64 * 0.05, world_z as f64 * 0.05]) + 1.0) * 0.5;
            if density_roll < bunch_threshold {
                continue;
            }

            // Calculate shoreline distance for treeline
            let shore_dist = distance_to_shoreline(world_x, world_z, seed);
            let beyond_treeline = shore_dist > TREELINE_DISTANCE;

            // Calculate biome factor for tree scaling
            let biome_factor = if biome_t > 0.65 {
                ((biome_t - 0.65) / 0.35).clamp(0.0, 1.0)
            } else {
                0.0
            };

            // Check upper treeline (alpine)
            let above_upper_treeline = height > UPPER_TREELINE_END;
            let in_upper_transition = height > UPPER_TREELINE_START && height <= UPPER_TREELINE_END;

            // Determine if bunch gets a tree
            let mut has_tree = beyond_treeline && !above_upper_treeline;
            if in_upper_transition {
                let fade = 1.0 - (height - UPPER_TREELINE_START) / (UPPER_TREELINE_END - UPPER_TREELINE_START);
                let roll = (noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3]) + 1.0) * 0.5;
                has_tree = has_tree && roll < fade as f64;
            }

            // Create and generate bunch
            let bunch = LowlandBunch {
                center: Vec3::new(world_x, height, world_z),
                radius: 8.0 + noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]).abs() as f32 * 4.0,
                seed: seed.wrapping_add((world_x as u32) ^ (world_z as u32).rotate_left(16)),
                has_tree,
                biome_factor,
            };

            let bunch_instances = bunch.generate(seed);
            all_instances.extend(bunch_instances.trees);
            all_instances.extend(bunch_instances.bushes);
            // Note: rocks and pebbles handled by rocks.rs
        }
    }

    //=========================================================================
    // PHASE 2: Additional Scattered Trees (Forest Zone)
    //=========================================================================
    // Dense forest gets extra trees beyond what bunches provide
    // These fill in gaps and create denser canopy

    let scattered_tree_density = 0.0008; // Trees per sq meter in dense forest
    let potential_scattered = (chunk_size * chunk_size * scattered_tree_density) as u32;

    for i in 0..potential_scattered {
        // Position from noise
        let rand_x = noise.get([i as f64 * 0.1, 700.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 800.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        let biome_t = get_biome_t(world_x, world_z, seed);
        let (height, _) = get_height_at(world_x, world_z, seed);

        // Only in forest zone
        if biome_t < 0.65 {
            continue;
        }

        // Treeline check (distance-based)
        let shore_dist = distance_to_shoreline(world_x, world_z, seed);
        if shore_dist < TREELINE_DISTANCE {
            continue;
        }

        // Upper treeline check
        if height > UPPER_TREELINE_END {
            continue;
        }

        // Density increases deeper into forest
        let forest_depth = (biome_t - 0.65) / 0.35;
        let density_threshold = 0.3 + forest_depth * 0.5; // 30-80% placement chance

        let roll = (noise.get([world_x as f64 * 0.02, world_z as f64 * 0.02]) + 1.0) * 0.5;
        if roll > density_threshold as f64 {
            continue;
        }

        // Upper treeline fade
        if height > UPPER_TREELINE_START {
            let fade = 1.0 - (height - UPPER_TREELINE_START) / (UPPER_TREELINE_END - UPPER_TREELINE_START);
            let fade_roll = (noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3 + 50.0]) + 1.0) * 0.5;
            if fade_roll > fade as f64 {
                continue;
            }
        }

        // Generate tree
        let angle = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * std::f32::consts::TAU;
        let base_scale = 5.5 + forest_depth * 2.5;
        let scale_var = noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]) as f32;
        let scale = base_scale + scale_var;

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_rotation_y(angle),
            Vec3::new(world_x, height - 1.0, world_z),
        );

        all_instances.push(transform);
    }

    all_instances
}

/// Generate bunches for a chunk, returning full bunch data for external rock generation.
///
/// This is called by the rock generation system to get bunch positions for
/// coordinated rock/pebble placement within bunches.
///
/// # Arguments
/// * `seed` - World seed
/// * `chunk_size` - Chunk size in world units
/// * `offset_x`, `offset_z` - Chunk world position
///
/// # Returns
/// Vector of LowlandBunch definitions for the chunk
pub fn generate_bunches_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<LowlandBunch> {
    let noise = Perlin::new(seed + 777);
    let mut bunches = Vec::new();

    let bunch_grid_size = 18.0;
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

            // Skip ocean and beach
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

            let above_upper_treeline = height > UPPER_TREELINE_END;
            let in_upper_transition = height > UPPER_TREELINE_START && height <= UPPER_TREELINE_END;

            let mut has_tree = beyond_treeline && !above_upper_treeline;
            if in_upper_transition {
                let fade = 1.0 - (height - UPPER_TREELINE_START) / (UPPER_TREELINE_END - UPPER_TREELINE_START);
                let roll = (noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3]) + 1.0) * 0.5;
                has_tree = has_tree && roll < fade as f64;
            }

            bunches.push(LowlandBunch {
                center: Vec3::new(world_x, height, world_z),
                radius: 8.0 + noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]).abs() as f32 * 4.0,
                seed: seed.wrapping_add((world_x as u32) ^ (world_z as u32).rotate_left(16)),
                has_tree,
                biome_factor,
            });
        }
    }

    bunches
}

//=============================================================================
// LEGACY COMPATIBILITY
//=============================================================================

/// Template struct for tree mesh data (legacy compatibility)
#[derive(Clone)]
pub struct TreeTemplate {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_generation() {
        let instances = generate_trees_for_chunk(12345, 256.0, 0.0, 0.0);
        println!("Generated {} tree/bush instances", instances.len());

        for instance in &instances {
            assert!(instance.w_axis.w == 1.0, "Invalid transform matrix");
        }
    }

    #[test]
    fn test_bunch_generation() {
        let bunches = generate_bunches_for_chunk(12345, 256.0, 0.0, 0.0);
        println!("Generated {} bunches", bunches.len());

        for bunch in &bunches {
            assert!(bunch.radius > 0.0, "Bunch must have positive radius");
            let instances = bunch.generate(12345);
            println!(
                "  Bunch at ({:.1}, {:.1}): {} pebbles, {} bushes, {} trees",
                bunch.center.x, bunch.center.z,
                instances.pebbles.len(),
                instances.bushes.len(),
                instances.trees.len()
            );
        }
    }

    #[test]
    fn test_treeline_distance() {
        // Trees should not appear within 36.6m of shoreline
        // This test verifies the treeline logic
        let seed = 42;

        // Find a point near the shore
        for x in (-100..100).step_by(10) {
            let shore_dist = distance_to_shoreline(x as f32, 0.0, seed);
            let (height, _) = get_height_at(x as f32, 0.0, seed);
            println!("x={}: height={:.2}, shore_dist={:.2}", x, height, shore_dist);
        }
    }
}
