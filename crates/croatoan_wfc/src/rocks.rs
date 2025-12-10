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
//! Density: ~0.2 per sq meter (10x previous).
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

use crate::mesh_gen::{get_height_at, get_biome_t};
use crate::trees::generate_bunches_for_chunk;
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec3, Quat};

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
    pub fn base_scale(&self) -> f32 {
        match self {
            RockType::Pebble => 0.12,
            RockType::SmallRock => 0.35,
            RockType::MediumRock => 0.75,
            RockType::LargeBoulder => 1.5,
            RockType::FlatRock => 0.55,
            RockType::MossyRock => 0.65,
        }
    }

    /// How deep the rock sinks into terrain (prevents floating)
    pub fn sink_amount(&self) -> f32 {
        match self {
            RockType::Pebble => 0.03,
            RockType::SmallRock => 0.08,
            RockType::MediumRock => 0.18,
            RockType::LargeBoulder => 0.35,
            RockType::FlatRock => 0.12,
            RockType::MossyRock => 0.22,
        }
    }

    /// Scale variation range (multiplier applied to base_scale)
    pub fn scale_variation(&self) -> (f32, f32) {
        match self {
            RockType::Pebble => (0.5, 1.5),      // High variation for visual interest
            RockType::SmallRock => (0.7, 1.3),
            RockType::MediumRock => (0.8, 1.2),
            RockType::LargeBoulder => (0.85, 1.15),
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
        let bunch_instances = bunch.generate(seed);

        // Add anchor rocks (large boulders)
        for transform in bunch_instances.large_rocks {
            instances.push((RockType::LargeBoulder.mesh_name().to_string(), transform));
        }

        // Add bunch pebbles
        for transform in bunch_instances.pebbles {
            instances.push((RockType::Pebble.mesh_name().to_string(), transform));
        }
    }

    //=========================================================================
    // PHASE 2: Scattered Large/Medium Rocks (10x density)
    //=========================================================================
    // Independent rocks on steep slopes and rocky biome areas
    // Previous density: 0.02, New density: 0.2

    let large_rock_density = 0.2; // 10x increase
    let potential_large = (chunk_size * chunk_size * large_rock_density) as u32;

    for i in 0..potential_large {
        let rand_x = noise.get([i as f64 * 0.1, 200.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 300.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

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
        // - Beach edges (biome_t 0.5-0.58, height 0.5-3)
        let is_steep = slope > 0.25;
        let is_rocky_biome = rocky_noise_val > 0.15;
        let is_beach_edge = biome_t > 0.48 && biome_t < 0.58 && height > 0.5 && height < 3.0;

        if !is_steep && !is_rocky_biome && !is_beach_edge {
            continue;
        }

        // Density filtering (still skip ~60% for natural clustering)
        let density_roll = (noise.get([world_x as f64 * 0.08, world_z as f64 * 0.08]) + 1.0) * 0.5;
        if density_roll > 0.4 {
            continue;
        }

        // Select rock type based on conditions
        let type_noise = noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3]) as f32;
        let rock_type = if is_beach_edge {
            // Beach gets flat rocks and small rocks
            if type_noise > 0.3 { RockType::FlatRock } else { RockType::SmallRock }
        } else if type_noise > 0.6 {
            RockType::LargeBoulder
        } else if type_noise > 0.2 {
            RockType::MediumRock
        } else if height < 4.0 && rocky_noise_val < 0.35 {
            RockType::MossyRock // Mossy in damp lowlands
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
    // PHASE 3: Dense Pebble Fields (10x density)
    //=========================================================================
    // Hundreds of tiny stones scattered everywhere above water
    // Previous density: 0.12, New density: 1.2
    // Uses clustering for natural distribution

    let pebble_density = 0.005; // Minimal - just a few per chunk (was 1.2)
    let potential_pebbles = (chunk_size * chunk_size * pebble_density) as u32;

    for i in 0..potential_pebbles {
        // Use different noise for pebble positions (avoid overlap with large rocks)
        let rand_x = pebble_noise.get([i as f64 * 0.17, 400.0]) as f32;
        let rand_z = pebble_noise.get([i as f64 * 0.17, 500.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Skip water (but include beach)
        if height < 0.2 {
            continue;
        }

        // Clustering: Use low-frequency noise to create pebble-rich and pebble-sparse areas
        let cluster_val = cluster_noise.get([world_x as f64 * 0.08, world_z as f64 * 0.08]) as f32;

        // Skip ~40% of pebbles in sparse areas for natural clustering
        if cluster_val < -0.2 {
            continue;
        }

        // Determine pebble vs small rock
        let type_roll = pebble_noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32;
        let rock_type = if type_roll > 0.7 && cluster_val > 0.3 {
            RockType::SmallRock // 30% chance of small rock in dense clusters
        } else {
            RockType::Pebble
        };

        // Simplified transform for pebbles (no tilt, faster)
        let angle = pebble_noise.get([world_x as f64 * 0.8, world_z as f64 * 0.8]) as f32 * std::f32::consts::TAU;
        let (scale_min, scale_max) = rock_type.scale_variation();
        let scale_t = (pebble_noise.get([world_x as f64 * 0.4, world_z as f64 * 0.4]) + 1.0) * 0.5;
        let scale = rock_type.base_scale() * (scale_min + scale_t as f32 * (scale_max - scale_min));

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_rotation_y(angle),
            Vec3::new(world_x, height - rock_type.sink_amount(), world_z),
        );

        instances.push((rock_type.mesh_name().to_string(), transform));
    }

    //=========================================================================
    // PHASE 4: Beach Pebble Strips
    //=========================================================================
    // Dense lines of pebbles along the water's edge
    // Creates natural-looking tide line debris

    let beach_pebble_density = 0.8;
    let potential_beach = (chunk_size * chunk_size * beach_pebble_density) as u32;

    for i in 0..potential_beach {
        let rand_x = cluster_noise.get([i as f64 * 0.23, 600.0]) as f32;
        let rand_z = cluster_noise.get([i as f64 * 0.23, 700.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        let (height, _color) = get_height_at(world_x, world_z, seed);
        let biome_t = get_biome_t(world_x, world_z, seed);

        // Only in beach zone (t 0.45-0.55) at tide line (height 0.3-1.5)
        if biome_t < 0.45 || biome_t > 0.56 {
            continue;
        }
        if height < 0.3 || height > 1.8 {
            continue;
        }

        // Dense at tide line (height ~0.5-1.0)
        let tide_factor = 1.0 - ((height - 0.75).abs() / 0.5).clamp(0.0, 1.0);
        let roll = (cluster_noise.get([world_x as f64 * 0.15, world_z as f64 * 0.15]) + 1.0) * 0.5;
        if roll > (tide_factor * 0.8) as f64 {
            continue;
        }

        // Beach pebbles are small
        let rock_type = RockType::Pebble;
        let angle = cluster_noise.get([world_x as f64, world_z as f64]) as f32 * std::f32::consts::TAU;
        let scale = rock_type.base_scale() * (0.6 + roll as f32 * 0.8);

        let transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_rotation_y(angle),
            Vec3::new(world_x, height - 0.02, world_z),
        );

        instances.push((rock_type.mesh_name().to_string(), transform));
    }

    instances
}

//=============================================================================
// TESTS
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

        // Should have significantly more rocks now (10x density)
        // For a 256x256 chunk:
        // - Large rocks: ~0.2 * 65536 * 0.4 (filter) = ~5200 potential, ~2000 actual
        // - Pebbles: ~1.2 * 65536 * 0.6 (filter) = ~47000 potential, ~28000 actual
        // Plus bunch-integrated rocks
        assert!(instances.len() > 10000, "Expected >10k rocks with 10x density");
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
