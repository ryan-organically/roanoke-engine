//! # Rock and Pebble Generation System
//!
//! This module handles procedural placement of rocks, boulders, and pebbles across the terrain.
//! It uses a multi-tier approach for efficient, dense coverage:
//!
//! ## Generation Tiers
//!
//! ### Tier 1: Bunch-Integrated Rocks
//! Large anchor rocks and pebble clusters within LowlandBunches (from trees.rs).
//! These provide the primary rock distribution in lowland/scrub zones.
//!
//! ### Tier 2: Scattered Large Rocks
//! Independent large boulders on steep slopes and rocky biomes.
//! Density: ~0.05 per sq meter (reduced from 0.2 for performance).
//!
//! ### Tier 3: Dense Pebble Fields
//! Hundreds of tiny stones per chunk for ground detail.
//! Density: ~1.2 per sq meter (10x previous).
//! Uses simplified transforms for GPU efficiency.
//!
//! ## Rock Types
//! - **Pebble**: Tiny stones (0.05-0.2m), appear everywhere above water
//! - **SmallRock**: Small rocks (0.3-0.5m), scattered ground detail
//! - **MediumRock**: Mid-size boulders (0.6-1.0m), moderate rarity
//! - **LargeBoulder**: Prominent boulders (1.2-2.0m), sparse, landmark-like
//! - **FlatRock**: Flat stepping stones, near water/paths
//! - **MossyRock**: Rocks with moss in damp/shaded areas
//!
//! ## Performance Notes
//! - Pebbles use minimal vertex counts (4-8 verts each)
//! - Instance transforms are batched by type for efficient rendering
//! - Clustering reduces apparent randomness while maintaining density

use crate::mesh_gen::{get_height_at, get_biome_t, calculate_river_depth};
use crate::trees::generate_bunches_for_chunk;
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec2, Vec3, Quat};

//=============================================================================
// CORN FIELD EXCLUSION ZONES
//=============================================================================

/// Exclusion zone for corn fields - rocks won't spawn inside these areas
#[derive(Debug, Clone, Copy)]
pub struct CornFieldBounds {
    pub center: Vec2,
    pub half_size: Vec2, // Half width and half depth
}

impl CornFieldBounds {
    /// Check if a world position is inside this corn field
    pub fn contains(&self, x: f32, z: f32) -> bool {
        let dx = (x - self.center.x).abs();
        let dz = (z - self.center.y).abs(); // Vec2.y represents world Z
        dx <= self.half_size.x && dz <= self.half_size.y
    }
}

/// Check if a position is inside any corn field exclusion zone
fn is_in_corn_field(x: f32, z: f32, exclusion_zones: &[CornFieldBounds]) -> bool {
    exclusion_zones.iter().any(|zone| zone.contains(x, z))
}

//=============================================================================
// ROCK TYPE DEFINITIONS
//=============================================================================

/// Rock types with different sizes, characteristics, and spawn conditions.
///
/// Each type has specific:
/// - Visual scale range
/// - Sink depth (how deep it sits in terrain)
/// - Spawn biome preferences
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RockType {
    /// Tiny scattered stones (0.05-0.2m)
    /// Appear everywhere above water, extremely common
    Pebble,

    /// Small rocks (0.3-0.5m)
    /// Common ground detail, clustered
    SmallRock,

    /// Medium boulders (0.6-1.0m)
    /// Moderate rarity, slopes and rocky areas
    MediumRock,

    /// Large prominent boulders (1.2-2.0m)
    /// Sparse, act as landmarks
    LargeBoulder,

    /// Flat stepping stones (0.4-0.8m)
    /// Near water, paths, clearings
    FlatRock,

    /// Moss-covered rocks (0.5-0.9m)
    /// Damp areas, forest shade
    MossyRock,
}

impl RockType {
    /// Get the mesh/texture identifier for this rock type
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

    /// Base scale multiplier for this rock type
    /// NOTE: Pebbles increased 3x from 0.12 to 0.35 for visibility
    pub fn base_scale(&self) -> f32 {
        match self {
            RockType::Pebble => 0.35,      // Was 0.12 - too small to see from game camera
            RockType::SmallRock => 0.45,    // Bumped slightly
            RockType::MediumRock => 0.85,
            RockType::LargeBoulder => 1.8,
            RockType::FlatRock => 0.65,
            RockType::MossyRock => 0.75,
        }
    }

    /// How deep the rock sinks into terrain (prevents floating)
    /// Reduced to keep more of rock visible above ground
    pub fn sink_amount(&self) -> f32 {
        match self {
            RockType::Pebble => 0.02,      // Was 0.03
            RockType::SmallRock => 0.05,   // Was 0.08
            RockType::MediumRock => 0.12,  // Was 0.18
            RockType::LargeBoulder => 0.25, // Was 0.35
            RockType::FlatRock => 0.08,    // Was 0.12
            RockType::MossyRock => 0.15,   // Was 0.22
        }
    }

    /// Scale variation range (multiplier applied to base_scale)
    /// LargeBoulder has massive 10x variation for dramatic size differences
    pub fn scale_variation(&self) -> (f32, f32) {
        match self {
            RockType::Pebble => (0.5, 1.5),      // High variation for visual interest
            RockType::SmallRock => (0.7, 1.3),
            RockType::MediumRock => (0.8, 2.5),  // Can get fairly large
            RockType::LargeBoulder => (0.8, 8.0), // HUGE variation: 1.44 to 14.4 scale
            RockType::FlatRock => (0.7, 1.3),
            RockType::MossyRock => (0.8, 1.2),
        }
    }
}

//=============================================================================
// MAIN GENERATION FUNCTION
//=============================================================================

/// Generate all rocks for a terrain chunk.
///
/// This combines multiple generation strategies:
/// 1. Bunch-integrated rocks (from LowlandBunch system)
/// 2. Scattered large rocks on slopes/rocky biomes
/// 3. Dense pebble fields everywhere above water
/// 4. Beach pebbles and driftwood-adjacent stones
///
/// # Arguments
/// * `seed` - World seed for deterministic generation
/// * `chunk_size` - Size of chunk in world units
/// * `offset_x`, `offset_z` - Chunk position in world coordinates
/// * `corn_field_exclusions` - Optional list of corn field boundaries to exclude rocks from
///
/// # Returns
/// Vector of (mesh_name, transform) tuples for all rocks in the chunk.
/// Rocks are grouped by type for efficient batch rendering.
pub fn generate_rocks_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<(String, Mat4)> {
    generate_rocks_for_chunk_with_exclusions(seed, chunk_size, offset_x, offset_z, &[])
}

/// Generate all rocks for a terrain chunk with corn field exclusion zones.
///
/// Rocks (pebbles, boulders, etc.) will not spawn inside corn field areas.
pub fn generate_rocks_for_chunk_with_exclusions(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
    corn_field_exclusions: &[CornFieldBounds],
) -> Vec<(String, Mat4)> {
    let noise = Perlin::new(seed + 888);
    let pebble_noise = Perlin::new(seed + 889);
    let cluster_noise = Perlin::new(seed + 890);

    let mut instances = Vec::new();

    //=========================================================================
    // PHASE 1: Bunch-Integrated Rocks
    //=========================================================================
    // Get bunches from trees.rs and generate their rock components
    // Each bunch contributes 1 anchor rock + 8-15 pebbles

    let bunches = generate_bunches_for_chunk(seed, chunk_size, offset_x, offset_z);

    for bunch in &bunches {
        let bunch_instances = bunch.generate(seed, &[]); // No village centers needed for rocks

        // Add anchor rocks (large boulders)
        for transform in bunch_instances.large_rocks {
            instances.push((RockType::LargeBoulder.mesh_name().to_string(), transform));
        }

        // DISABLED: Bunch pebbles - "rock_pebble" mesh doesn't exist/render,
        // these were invisible instances hurting performance
        // for transform in bunch_instances.pebbles {
        //     instances.push((RockType::Pebble.mesh_name().to_string(), transform));
        // }
    }

    //=========================================================================
    // PHASE 2: Scattered Large/Medium Rocks (REDUCED for performance)
    //=========================================================================
    // Independent rocks on steep slopes and rocky biome areas
    // Was 0.2 (way too dense), now 0.05 = 75% reduction

    let large_rock_density = 0.05; // Reduced from 0.2 - was generating 5000+ per chunk
    let potential_large = (chunk_size * chunk_size * large_rock_density) as u32;

    for i in 0..potential_large {
        let rand_x = noise.get([i as f64 * 0.1, 200.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 300.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        // Skip rocks inside corn fields
        if is_in_corn_field(world_x, world_z, corn_field_exclusions) {
            continue;
        }

        let (height, _color) = get_height_at(world_x, world_z, seed);
        let biome_t = get_biome_t(world_x, world_z, seed);

        // Skip deep water
        if height < 0.3 {
            continue;
        }

        // Calculate terrain slope
        let sample_dist = 1.0;
        let (h_dx, _) = get_height_at(world_x + sample_dist, world_z, seed);
        let (h_dz, _) = get_height_at(world_x, world_z + sample_dist, seed);
        let slope = ((h_dx - height).powi(2) + (h_dz - height).powi(2)).sqrt() / sample_dist;

        // Rocky biome noise (separate from placement noise)
        let rocky_noise_val = cluster_noise.get([world_x as f64 * 0.03, world_z as f64 * 0.03]) as f32;

        // Spawn conditions:
        // - Steep slopes (slope > 0.25)
        // - Rocky biome areas (rocky_noise > 0.15)
        // - Beach edges DISABLED - PHASE 5 handles beach boulders at lower density
        let is_steep = slope > 0.25;
        let is_rocky_biome = rocky_noise_val > 0.15;
        // let is_beach_edge = biome_t > 0.48 && biome_t < 0.58 && height > 0.5 && height < 3.0;

        if !is_steep && !is_rocky_biome {
            continue;
        }

        // Density filtering (still skip ~60% for natural clustering)
        let density_roll = (noise.get([world_x as f64 * 0.08, world_z as f64 * 0.08]) + 1.0) * 0.5;
        if density_roll > 0.4 {
            continue;
        }

        // Select rock type based on conditions - BOULDERS ARE PRIMARY
        let type_noise = noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3]) as f32;
        let rock_type = if type_noise > -0.2 {
            // ~60% of non-beach rocks are now boulders (was 20% at > 0.6)
            RockType::LargeBoulder
        } else if type_noise > -0.6 {
            RockType::MediumRock
        } else if height < 4.0 && rocky_noise_val < 0.35 {
            RockType::MossyRock
        } else {
            RockType::SmallRock
        };

        // Transform calculation
        let angle = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * std::f32::consts::TAU;
        let (scale_min, scale_max) = rock_type.scale_variation();
        let scale_t = (noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]) + 1.0) * 0.5;
        let scale = rock_type.base_scale() * (scale_min + scale_t as f32 * (scale_max - scale_min));

        // Slight tilt for natural look
        let tilt_x = noise.get([world_x as f64 * 0.7, 100.0]) as f32 * 0.12;
        let tilt_z = noise.get([world_z as f64 * 0.7, 150.0]) as f32 * 0.12;

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_euler(glam::EulerRot::XYZ, tilt_x, angle, tilt_z),
            Vec3::new(world_x, height - rock_type.sink_amount(), world_z),
        );

        instances.push((rock_type.mesh_name().to_string(), transform));
    }

    //=========================================================================
    // PHASE 3: DEDICATED BOULDER SCATTER (everywhere above water)
    //=========================================================================
    // Large boulders spawned across all terrain for visual landmark
    // These use the new GLB boulder model with LOD system

    let boulder_density = 0.3; // Reduced from 0.8 - was too dense
    let potential_boulders_scatter = (chunk_size * chunk_size * boulder_density / 1000.0) as u32;

    for i in 0..potential_boulders_scatter {
        let rand_x = pebble_noise.get([i as f64 * 0.17, 500.0]) as f32;
        let rand_z = pebble_noise.get([i as f64 * 0.17, 600.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        if is_in_corn_field(world_x, world_z, corn_field_exclusions) {
            continue;
        }

        let (height, _color) = get_height_at(world_x, world_z, seed);
        let biome_t = get_biome_t(world_x, world_z, seed);

        // Skip water, beach zones (handled by PHASE 5), and very high altitudes
        // Beach zone: biome_t 0.44-0.65
        if height < 1.0 || height > 50.0 || (biome_t > 0.44 && biome_t < 0.65) {
            continue;
        }

        let angle = pebble_noise.get([world_x as f64 * 0.4, world_z as f64 * 0.4]) as f32 * std::f32::consts::TAU;
        let scale_t = (pebble_noise.get([world_x as f64 * 0.15, world_z as f64 * 0.15]) + 1.0) * 0.5;
        // Massive 10x scale variation: 1.5 to 15.0
        let scale = 1.5 + scale_t as f32 * 13.5;

        let tilt_x = pebble_noise.get([world_x as f64 * 0.6, 110.0]) as f32 * 0.15;
        let tilt_z = pebble_noise.get([world_z as f64 * 0.6, 160.0]) as f32 * 0.15;

        // Bigger boulders sink more
        let sink = 0.25 + scale * 0.08;

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_euler(glam::EulerRot::XYZ, tilt_x, angle, tilt_z),
            Vec3::new(world_x, height - sink, world_z),
        );

        instances.push((RockType::LargeBoulder.mesh_name().to_string(), transform));
    }

    //=========================================================================
    // PHASE 4: RIVER MEGA-BOULDERS
    //=========================================================================
    // Massive boulders in and around rivers/streams (low height, non-beach)
    // These create dramatic river scenery with huge rocks jutting from water

    let river_boulder_density = 0.1; // Reduced from 0.25
    let potential_river_boulders = (chunk_size * chunk_size * river_boulder_density / 500.0) as u32;

    for i in 0..potential_river_boulders {
        let rand_x = pebble_noise.get([i as f64 * 0.23, 700.0]) as f32;
        let rand_z = pebble_noise.get([i as f64 * 0.23, 800.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        if is_in_corn_field(world_x, world_z, corn_field_exclusions) {
            continue;
        }

        let (height, _color) = get_height_at(world_x, world_z, seed);

        // River zone: use actual river detection, not biome_t
        // calculate_river_depth returns 0.0-1.0 where >0 means in/near river
        let river_depth = calculate_river_depth(world_x, world_z, seed);

        // Also check nearby for riverbank boulders (within ~12m of river)
        let nearby_river = if river_depth < 0.05 {
            // Sample nearby points to catch riverbanks
            let sample_offsets = [(8.0, 0.0), (-8.0, 0.0), (0.0, 8.0), (0.0, -8.0)];
            sample_offsets.iter().any(|(dx, dz)| {
                calculate_river_depth(world_x + dx, world_z + dz, seed) > 0.2
            })
        } else {
            false
        };

        let is_river_zone = river_depth > 0.05 || nearby_river;

        if !is_river_zone {
            continue;
        }

        // Cluster check - rivers should have boulder clusters
        let cluster_val = cluster_noise.get([world_x as f64 * 0.05, world_z as f64 * 0.05]) as f32;
        if cluster_val < -0.3 {
            continue; // Skip ~35% for clustering
        }

        let angle = pebble_noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3]) as f32 * std::f32::consts::TAU;
        let scale_t = (pebble_noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]) + 1.0) * 0.5;
        // MASSIVE scale: 2.5 - 4.5 (huge river boulders)
        let scale = 2.5 + scale_t as f32 * 2.0;

        let tilt_x = pebble_noise.get([world_x as f64 * 0.5, 120.0]) as f32 * 0.2;
        let tilt_z = pebble_noise.get([world_z as f64 * 0.5, 170.0]) as f32 * 0.2;

        // Sink deeper - some boulders partially submerged
        let sink = 0.4 + (pebble_noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]) + 1.0) as f32 * 0.3;

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_euler(glam::EulerRot::XYZ, tilt_x, angle, tilt_z),
            Vec3::new(world_x, height - sink, world_z),
        );

        instances.push((RockType::LargeBoulder.mesh_name().to_string(), transform));
    }

    //=========================================================================
    // PHASE 5: Beach Boulders (very sparse, mostly small, rare large)
    //=========================================================================

    let beach_boulder_density = 0.001; // Very sparse (halved from 0.002)
    let potential_boulders = (chunk_size * chunk_size * beach_boulder_density) as u32;

    for i in 0..potential_boulders {
        let rand_x = noise.get([i as f64 * 0.31, 850.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.31, 950.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        if is_in_corn_field(world_x, world_z, corn_field_exclusions) {
            continue;
        }

        let (height, _color) = get_height_at(world_x, world_z, seed);
        let biome_t = get_biome_t(world_x, world_z, seed);

        if biome_t < 0.44 || biome_t > 0.60 || height < 0.5 || height > 4.0 {
            continue;
        }

        let angle = noise.get([world_x as f64 * 0.6, world_z as f64 * 0.6]) as f32 * std::f32::consts::TAU;

        // Mostly small (1-3), seldom large (6-10)
        let size_roll = noise.get([world_x as f64 * 0.08, world_z as f64 * 0.08]) as f32;
        let scale = if size_roll > 0.85 {
            // ~7% chance of large beach boulder
            6.0 + (size_roll - 0.85) * 26.0 // 6-10 scale
        } else {
            // Small to medium
            1.0 + (size_roll + 1.0) * 1.0 // 1-3 scale
        };

        let tilt_x = noise.get([world_x as f64 * 0.8, 120.0]) as f32 * 0.1;
        let tilt_z = noise.get([world_z as f64 * 0.8, 170.0]) as f32 * 0.1;
        let sink = 0.15 + scale * 0.04;

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_euler(glam::EulerRot::XYZ, tilt_x, angle, tilt_z),
            Vec3::new(world_x, height - sink, world_z),
        );

        instances.push((RockType::LargeBoulder.mesh_name().to_string(), transform));
    }

    //=========================================================================
    // PHASE 6: FOREST BOULDERS (sparse, mostly small, rare huge taiga style)
    //=========================================================================

    let forest_boulder_density = 0.002; // Very rare (halved from 0.004)
    let potential_forest_boulders = (chunk_size * chunk_size * forest_boulder_density) as u32;

    for i in 0..potential_forest_boulders {
        let rand_x = noise.get([i as f64 * 0.19, 1100.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.19, 1200.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        if is_in_corn_field(world_x, world_z, corn_field_exclusions) {
            continue;
        }

        let (height, _color) = get_height_at(world_x, world_z, seed);
        let biome_t = get_biome_t(world_x, world_z, seed);

        // Forest zone
        if biome_t < 0.68 || height < 6.0 || height > 45.0 {
            continue;
        }

        let angle = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * std::f32::consts::TAU;

        // Mostly small (1.5-4), seldom huge taiga boulders (10-18)
        let size_roll = noise.get([world_x as f64 * 0.06, world_z as f64 * 0.06]) as f32;
        let scale = if size_roll > 0.88 {
            // ~6% chance of massive taiga boulder
            10.0 + (size_roll - 0.88) * 66.0 // 10-18 scale
        } else {
            // Small forest rocks
            1.5 + (size_roll + 1.0) * 1.25 // 1.5-4 scale
        };

        let tilt_x = noise.get([world_x as f64 * 0.4, 130.0]) as f32 * 0.12;
        let tilt_z = noise.get([world_z as f64 * 0.4, 180.0]) as f32 * 0.12;
        let sink = 0.3 + scale * 0.06;

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_euler(glam::EulerRot::XYZ, tilt_x, angle, tilt_z),
            Vec3::new(world_x, height - sink, world_z),
        );

        instances.push((RockType::LargeBoulder.mesh_name().to_string(), transform));
    }

    //=========================================================================
    // INSTANCE CAP - Prevent GPU overload
    //=========================================================================
    const MAX_ROCKS_PER_CHUNK: usize = 500;
    if instances.len() > MAX_ROCKS_PER_CHUNK {
        // Prioritize larger rocks (boulders) over small rocks
        // Sort by scale (embedded in transform matrix diagonal)
        instances.sort_by(|a, b| {
            let scale_a = a.1.x_axis.x; // Scale is on diagonal
            let scale_b = b.1.x_axis.x;
            scale_b.partial_cmp(&scale_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        instances.truncate(MAX_ROCKS_PER_CHUNK);
    }

    instances
}

//=============================================================================
// TESTS
//=============================================================================
// DEADWOOD GENERATION
//=============================================================================

/// Generate deadwood (fallen logs, driftwood) for a chunk.
/// Spawns in:
/// - Forest areas (height > 4m) - fallen logs
/// - Beach areas (height 1-3m) - driftwood
/// - All biomes with varying density
///
/// Returns Vec of (model_name, transform) tuples.
pub fn generate_deadwood_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<(String, Mat4)> {
    let noise = Perlin::new(seed.wrapping_add(7777));
    let mut instances = Vec::new();

    // Deadwood density varies by biome
    // ~0.01-0.02 per sq meter in forests, ~0.005 on beaches
    let base_density = 0.012;
    let potential_count = (chunk_size * chunk_size * base_density) as u32;

    for i in 0..potential_count {
        // Pseudo-random position
        let rand_x = noise.get([i as f64 * 0.31, seed as f64 * 0.1]) as f32;
        let rand_z = noise.get([i as f64 * 0.31, seed as f64 * 0.1 + 100.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Determine if this location should have deadwood
        let spawn_roll = noise.get([world_x as f64 * 0.7, world_z as f64 * 0.7]) as f32;

        let should_spawn = if height > 4.0 {
            // Forest: higher probability
            spawn_roll > 0.3
        } else if height > 1.0 && height < 3.5 {
            // Beach/shoreline: driftwood, lower probability
            spawn_roll > 0.6
        } else if height > 3.5 {
            // Scrub/transition: moderate probability
            spawn_roll > 0.5
        } else {
            // Too low (water) or other - skip
            false
        };

        if !should_spawn {
            continue;
        }

        // Random rotation (logs lie flat, rotate around Y)
        let angle = noise.get([world_x as f64 * 1.5, world_z as f64 * 1.5]) as f32 * std::f32::consts::PI;

        // Scale variation (0.7 to 1.3)
        let scale_var = 0.7 + (noise.get([world_x as f64 * 2.0, world_z as f64 * 2.0]) as f32 + 1.0) * 0.3;

        // Slight tilt for natural look (up to 15 degrees)
        let tilt_x = noise.get([world_x as f64 * 3.0, world_z as f64]) as f32 * 0.26; // ~15 deg
        let tilt_z = noise.get([world_x as f64, world_z as f64 * 3.0]) as f32 * 0.26;

        let rotation = Quat::from_euler(glam::EulerRot::YXZ, angle, tilt_x, tilt_z);
        let translation = Vec3::new(world_x, height, world_z);
        let scale = Vec3::splat(scale_var);

        let transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);

        // All deadwood uses the same base model (LOD selected at render time)
        instances.push(("dead_log_0".to_string(), transform));
    }

    instances
}

//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rock_generation_density() {
        let instances = generate_rocks_for_chunk(12345, 256.0, 0.0, 0.0);

        println!("Generated {} total rock instances", instances.len());

        // Count by type
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for (name, _) in &instances {
            *counts.entry(name.as_str()).or_insert(0) += 1;
        }

        for (name, count) in &counts {
            println!("  {}: {}", name, count);
        }

        // Pebble fields disabled - now only boulders and bunch-integrated rocks
        // For a 256x256 chunk:
        // - Large/medium rocks from PHASE 2: ~2000-3000
        // - Beach boulders from PHASE 5: ~100-200
        // - Bunch rocks + pebbles from PHASE 1: varies by biome
        assert!(instances.len() > 100, "Expected some rocks to spawn");
    }

    #[test]
    fn test_rock_types() {
        let instances = generate_rocks_for_chunk(42, 128.0, 0.0, 0.0);

        let valid_names = [
            "rock_pebble", "rock_small", "rock_medium",
            "rock_boulder", "rock_flat", "rock_mossy"
        ];

        for (name, transform) in &instances {
            assert!(valid_names.contains(&name.as_str()), "Unknown rock type: {}", name);
            assert!(transform.w_axis.w == 1.0, "Invalid transform matrix");
        }
    }

    #[test]
    fn test_bunch_integration() {
        // Verify that bunch rocks are included
        let instances = generate_rocks_for_chunk(12345, 256.0, -200.0, 0.0);

        let boulder_count = instances.iter()
            .filter(|(name, _)| name == "rock_boulder")
            .count();

        println!("Boulder count (includes bunch anchors): {}", boulder_count);

        // Should have some boulders from bunches
        assert!(boulder_count > 0, "Expected some boulders from bunch anchors");
    }
}
