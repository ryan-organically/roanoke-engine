//! Multi-model foliage generation (trees + shrubs with variety)
//!
//! Generates foliage instances with model indices for multi-model rendering.
//! Supports species-based communities (e.g., birch groves) using noise zones.

use crate::mesh_gen::{get_height_at, distance_to_shoreline, get_biome_t, calculate_river_depth};
use crate::trees::{TREELINE_DISTANCE, UPPER_TREELINE_START, UPPER_TREELINE_END};
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec3, Quat};
use std::collections::HashMap;

/// Tree species indices - must match order in main.rs tree_models array
/// NOTE: tree_0, tree_1 (oak/maple) disabled - single-LOD placeholders
pub const TREE_SPECIES_BIRCH: usize = 0;       // birch_0
pub const TREE_SPECIES_PINE: usize = 1;        // pine_0
pub const TREE_SPECIES_DEAD_CONIFER: usize = 2; // dead_conifer_0
pub const TREE_SPECIES_FIR: usize = 3;         // fir_0 (bushy, forest edges)
pub const TREE_SPECIES_NOBLEFIR: usize = 4;    // noblefir0 (bushy fir, forest edges/clearings)

/// Per-species Y offset to ground models correctly
/// Positive = raise model up, Negative = sink into ground
/// NOTE: Optimized models should be exported with base at Y=0 (on origin)
pub fn tree_species_y_offset(_species: usize) -> f32 {
    // All optimized models should be on origin - no offset needed
    0.0
}

/// Shrub model indices - must match order in main.rs shrub_models array
/// NOTE: shrub_0, bush_0, grass_0 disabled (single-LOD placeholders)
pub const SHRUB_SPECIES_BEACH_GRASS_0: usize = 0; // beach_grass_0
pub const SHRUB_SPECIES_CONIFER_SHRUB: usize = 1; // conifer_shrub_0 (3 LODs)

/// Instance with model index for multi-model rendering
#[derive(Clone, Debug)]
pub struct FoliageInstance {
    pub transform: Mat4,
    pub model_index: usize,
    /// Megaflora: giant ancient trees (4x scale, future: harder to chop)
    pub is_megaflora: bool,
}

/// Check if a pine tree should become megaflora (12% chance)
/// Megaflora are ancient giant pines - 4x normal size
fn is_megaflora(world_x: f32, world_z: f32, seed: u32) -> bool {
    let mega_noise = Perlin::new(seed.wrapping_add(99999));
    let roll = (mega_noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]) + 1.0) * 0.5;
    roll < 0.12 // ~12% chance
}

/// Determine if a world position is in a birch community zone.
/// Uses low-frequency noise to create large, coherent birch groves.
#[allow(dead_code)]
fn is_birch_zone(world_x: f32, world_z: f32, seed: u32) -> bool {
    // Use a separate noise layer with very low frequency for large zones
    let birch_noise = Perlin::new(seed.wrapping_add(31337)); // Different seed for birch zones

    // Large-scale noise (0.008 frequency = ~125m features)
    let zone_value = birch_noise.get([world_x as f64 * 0.008, world_z as f64 * 0.008]);

    // Birch zones occur when noise > 0.3 (roughly 30% of forest area)
    zone_value > 0.3
}

/// Get birch zone strength (0.0 = not birch zone, 1.0 = deep in birch zone)
/// Used for smooth transitions at zone edges
/// Birch only spawns INLAND (biome_t > 0.82) - not near coast or treeline
fn birch_zone_strength(world_x: f32, world_z: f32, seed: u32) -> f32 {
    // Check if we're far enough inland for birch (beyond coastal forest)
    let biome_t = crate::mesh_gen::get_biome_t(world_x, world_z, seed);
    if biome_t < 0.82 {
        return 0.0; // No birch near coast, treeline, or coastal forest
    }

    // Birch strength increases with distance inland
    let inland_factor = ((biome_t - 0.82) / 0.18).clamp(0.0, 1.0);

    let birch_noise = Perlin::new(seed.wrapping_add(31337));
    let zone_value = birch_noise.get([world_x as f64 * 0.008, world_z as f64 * 0.008]) as f32;

    // Remap: 0.3 -> 0.0, 0.7 -> 1.0, then scale by inland factor
    let base_strength = ((zone_value - 0.3) / 0.4).clamp(0.0, 1.0);
    base_strength * inland_factor
}

/// Get pine zone strength (0.0 = not pine zone, 1.0 = deep in pine zone)
/// Pines favor coastal areas and sandy soils - uses different noise layer
/// DOUBLED again: Pines now cover ~75% of forest area for denser canopy
fn pine_zone_strength(world_x: f32, world_z: f32, seed: u32) -> f32 {
    let pine_noise = Perlin::new(seed.wrapping_add(54321));
    let zone_value = pine_noise.get([world_x as f64 * 0.006, world_z as f64 * 0.006]) as f32;

    // Pines in ~75% of forest area (doubled from 50%)
    // Much lower threshold = pine zones almost everywhere
    ((zone_value + 0.3) / 0.6).clamp(0.0, 1.0)
}

/// Select tree species based on position and zone
fn select_tree_species(world_x: f32, world_z: f32, seed: u32, tree_count: usize) -> usize {
    if tree_count < 3 {
        // Limited models, use random from available
        return ((world_x.abs() as u32).wrapping_mul(83492791)
            ^ (world_z.abs() as u32).wrapping_mul(41729563)) as usize % tree_count;
    }

    let local_noise = Perlin::new(seed.wrapping_add(12345));
    let roll = (local_noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) + 1.0) * 0.5;

    // Check pine zones first (if pine model available)
    // Pine probability increased for denser forest (+25% total trees)
    if tree_count >= 4 {
        let pine_strength = pine_zone_strength(world_x, world_z, seed);
        if pine_strength > 0.0 {
            // Deep in zone: 95% pine, edge: 60% pine (increased from 85%/45%)
            let pine_probability = 0.60 + pine_strength * 0.35;
            if roll < pine_probability as f64 {
                // ~10% of pines become dead conifers (if model available)
                if tree_count >= 3 {
                    let dead_hash = ((world_x.abs() as u32).wrapping_mul(29473891)
                        ^ (world_z.abs() as u32).wrapping_mul(73829461)) % 100;
                    if dead_hash < 10 {
                        return TREE_SPECIES_DEAD_CONIFER;
                    }
                }
                return TREE_SPECIES_PINE;
            }
        }
    }

    // Check birch zones
    let birch_strength = birch_zone_strength(world_x, world_z, seed);
    if birch_strength > 0.0 {
        // Deep in zone: 90% birch, edge of zone: 50% birch
        let birch_probability = 0.5 + birch_strength * 0.4;
        if roll < birch_probability as f64 {
            return TREE_SPECIES_BIRCH;
        }
    }

    // Fir and Noblefir trees at forest edges (bushy, sun-loving)
    // biome_t 0.65-0.78 = coastal forest / forest edge zone (where beach transitions to forest)
    if tree_count >= 5 {
        let biome_t = get_biome_t(world_x, world_z, seed);
        if biome_t > 0.65 && biome_t < 0.78 {
            // Mixed fir/noblefir at forest edges (40% fir, 35% noblefir, 25% other)
            let edge_hash = ((world_x.abs() as u32).wrapping_mul(67891234)
                ^ (world_z.abs() as u32).wrapping_mul(98765432)) % 100;
            if edge_hash < 40 {
                return TREE_SPECIES_FIR;
            } else if edge_hash < 75 {
                return TREE_SPECIES_NOBLEFIR;
            }
        }
        // Also spawn fir/noblefir in sparse areas of deeper forest (~20% chance)
        let sparse_hash = ((world_x.abs() as u32).wrapping_mul(11223344)
            ^ (world_z.abs() as u32).wrapping_mul(55667788)) % 100;
        if sparse_hash < 10 {
            return TREE_SPECIES_FIR;
        } else if sparse_hash < 20 {
            return TREE_SPECIES_NOBLEFIR;
        }
    }

    // Fallback: birch for remaining trees
    TREE_SPECIES_BIRCH
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
    /// Get tree transforms grouped by model name (birch_0, pine_0, dead_conifer_0, fir_0)
    pub fn trees_by_model(&self, model_count: usize) -> HashMap<String, Vec<Mat4>> {
        let mut result = HashMap::new();
        let count = model_count.max(1);
        for inst in &self.trees {
            let idx = inst.model_index % count;
            // Map species index to actual model name
            let model_name = match idx {
                TREE_SPECIES_BIRCH => "birch_0".to_string(),
                TREE_SPECIES_PINE => "pine_0".to_string(),
                TREE_SPECIES_DEAD_CONIFER => "dead_conifer_0".to_string(),
                TREE_SPECIES_FIR => "fir_0".to_string(),
                TREE_SPECIES_NOBLEFIR => "fir_1".to_string(),
                _ => "birch_0".to_string(), // fallback to birch
            };
            result.entry(model_name).or_insert_with(Vec::new).push(inst.transform);
        }
        result
    }

    /// Get shrub transforms grouped by model name (beach_grass_0, conifer_shrub_0)
    pub fn shrubs_by_model(&self, model_count: usize) -> HashMap<String, Vec<Mat4>> {
        let mut result = HashMap::new();
        let count = model_count.max(1);
        for inst in &self.shrubs {
            let idx = inst.model_index % count;
            // Map species index to actual model name
            let model_name = match idx {
                SHRUB_SPECIES_BEACH_GRASS_0 => "beach_grass_0".to_string(),
                SHRUB_SPECIES_CONIFER_SHRUB => "conifer_shrub_0".to_string(),
                _ => "conifer_shrub_0".to_string(), // Default to conifer shrub
            };
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

    // Larger grid = fewer trees = better FPS
    let bunch_grid_size = 64.0;
    let bunches_per_row = (chunk_size / bunch_grid_size).ceil() as i32;

    for bz in 0..bunches_per_row {
        for bx in 0..bunches_per_row {
            let grid_x = offset_x + (bx as f32 + 0.5) * bunch_grid_size;
            let grid_z = offset_z + (bz as f32 + 0.5) * bunch_grid_size;
            // MASSIVE jitter to truly scatter trees randomly - no clustering!
            let jitter_x = noise.get([grid_x as f64 * 0.1, grid_z as f64 * 0.1]) as f32 * bunch_grid_size * 0.9;
            let jitter_z = noise.get([grid_x as f64 * 0.1, grid_z as f64 * 0.1 + 100.0]) as f32 * bunch_grid_size * 0.9;
            let world_x = grid_x + jitter_x;
            let world_z = grid_z + jitter_z;

            let biome_t = get_biome_t(world_x, world_z, seed);
            let (height, _) = get_height_at(world_x, world_z, seed);

            // Skip ocean and beach (t < 0.65)
            if biome_t < 0.65 || height < 1.5 {
                continue;
            }

            // Forest-edge/Treeline zone (0.65-0.72): EXTREMELY dense - defines the treeline
            // Coastal forest (0.72-0.82): denser with more pines
            // Inland forest (0.82+): moderate density
            let mut bunch_threshold = if biome_t < 0.72 {
                0.05 // Even denser treeline
            } else if biome_t < 0.82 {
                0.20 // Denser coastal forest (was 0.30)
            } else {
                0.35 // Denser inland (was 0.45)
            };

            // River proximity boost: denser trees near rivers
            // Check if we're near a river channel
            let river_depth = calculate_river_depth(world_x, world_z, seed);
            let near_river = if river_depth < 0.05 {
                // Sample nearby for riverbank detection
                [(10.0, 0.0), (-10.0, 0.0), (0.0, 10.0), (0.0, -10.0)]
                    .iter()
                    .any(|(dx, dz)| calculate_river_depth(world_x + dx, world_z + dz, seed) > 0.1)
            } else {
                true // Already in/near river
            };
            if near_river {
                // Reduce threshold by 50% near rivers = much denser trees
                bunch_threshold *= 0.5;
            }

            let density_roll = (noise.get([world_x as f64 * 0.05, world_z as f64 * 0.05]) + 1.0) * 0.5;
            if density_roll < bunch_threshold {
                continue;
            }

            let shore_dist = distance_to_shoreline(world_x, world_z, seed);
            let beyond_treeline = shore_dist > TREELINE_DISTANCE;
            let biome_factor = if biome_t > 0.72 {
                ((biome_t - 0.72) / 0.28).clamp(0.0, 1.0)
            } else {
                0.0 // Treeline trees are smaller/younger
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

            // SHRUBS (understory) - dense throughout forest
            // Treeline (t 0.65-0.72): 8-14 shrubs per area (very dense)
            // Regular forest: 5-9 shrubs per area
            let base_shrubs = if biome_t < 0.72 { 8 } else { 5 };
            let shrub_variation = if biome_t < 0.72 { 6.0 } else { 4.0 };
            let shrub_local = base_shrubs + ((local_noise.get([local_seed as f64 * 0.1, 60.0]) + 1.0) * shrub_variation) as u32;
            for i in 0..shrub_local {
                // Fully random placement within a wide radius, not orbiting center
                let shrub_offset_x = local_noise.get([local_seed as f64 * 0.5, i as f64 * 7.3]) as f32 * 25.0;
                let shrub_offset_z = local_noise.get([local_seed as f64 * 0.7, i as f64 * 11.1]) as f32 * 25.0;
                let sx = world_x + shrub_offset_x;
                let sz = world_z + shrub_offset_z;
                let (sh, _) = get_height_at(sx, sz, seed);
                if sh < 0.5 {
                    continue;
                }

                // Greatly varied scale: tiny 0.3 to large 2.5
                let shrub_scale =
                    0.3 + local_noise.get([sx as f64 * 0.2, sz as f64 * 0.2]).abs() as f32 * 2.2;
                let shrub_rot =
                    local_noise.get([sx as f64 * 0.5, sz as f64 * 0.5]) as f32 * std::f32::consts::TAU;
                let model_idx = ((sx.abs() as u32).wrapping_mul(73856093)
                    ^ (sz.abs() as u32).wrapping_mul(19349663)) as usize
                    % shrub_count;

                result.shrubs.push(FoliageInstance {
                    transform: Mat4::from_scale_rotation_translation(
                        Vec3::splat(shrub_scale),
                        Quat::from_rotation_y(shrub_rot),
                        Vec3::new(sx, sh, sz),
                    ),
                    model_index: model_idx,
                    is_megaflora: false,
                });
            }

            // TREE (canopy)
            if has_tree {
                // Spread tree placement more randomly from center
                let tx = world_x + local_noise.get([local_seed as f64 * 0.2, 300.0]) as f32 * 15.0;
                let tz = world_z + local_noise.get([local_seed as f64 * 0.2, 400.0]) as f32 * 15.0;
                let (th, _) = get_height_at(tx, tz, seed);

                // Skip if terrain is too low (near water) - prevents floating
                if th < 2.0 {
                    continue;
                }

                let base_scale = 5.0 + biome_factor * 3.0;
                let mut tree_scale =
                    base_scale + local_noise.get([tx as f64 * 0.2, tz as f64 * 0.2]) as f32;
                let tree_angle =
                    local_noise.get([tx as f64 * 0.5, tz as f64 * 0.5]) as f32 * std::f32::consts::TAU;
                // Use species-based selection (supports birch community zones)
                let model_idx = select_tree_species(tx, tz, seed, tree_count);

                // Megaflora: 12% of pines become giant ancient trees (4x scale)
                let mega = model_idx == TREE_SPECIES_PINE && is_megaflora(tx, tz, seed);
                if mega {
                    tree_scale *= 4.0;
                }

                // Optimized models exported on Blender origin - place directly at terrain height
                result.trees.push(FoliageInstance {
                    transform: Mat4::from_scale_rotation_translation(
                        Vec3::splat(tree_scale),
                        Quat::from_rotation_y(tree_angle),
                        Vec3::new(tx, th, tz),
                    ),
                    model_index: model_idx,
                    is_megaflora: mega,
                });
            }
        }
    }

    // Phase 2: Scattered forest trees - reduced density for FPS
    let scattered_density = 0.0001;
    let potential = (chunk_size * chunk_size * scattered_density) as u32;
    for i in 0..potential {
        let rx = noise.get([i as f64 * 0.1, 700.0]) as f32;
        let rz = noise.get([i as f64 * 0.1, 800.0]) as f32;
        let world_x = offset_x + (rx + 1.0) * 0.5 * chunk_size;
        let world_z = offset_z + (rz + 1.0) * 0.5 * chunk_size;
        let biome_t = get_biome_t(world_x, world_z, seed);
        let (height, _) = get_height_at(world_x, world_z, seed);

        // Updated threshold: forest starts at 0.72 (treeline at 0.65-0.72)
        if biome_t < 0.72 {
            continue;
        }
        if distance_to_shoreline(world_x, world_z, seed) < TREELINE_DISTANCE {
            continue;
        }
        if height > UPPER_TREELINE_END {
            continue;
        }
        // Skip low terrain to prevent floating trees
        if height < 2.5 {
            continue;
        }

        let forest_depth = (biome_t - 0.72) / 0.28;
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
        // Trees scale with forest_depth (based on new 0.62 threshold)
        let mut scale = 5.5
            + forest_depth * 2.5
            + noise.get([world_x as f64 * 0.2, world_z as f64 * 0.2]) as f32;
        // Use species-based selection (supports birch community zones)
        let model_idx = select_tree_species(world_x, world_z, seed, tree_count);

        // Megaflora: 12% of pines become giant ancient trees (4x scale)
        let mega = model_idx == TREE_SPECIES_PINE && is_megaflora(world_x, world_z, seed);
        if mega {
            scale *= 4.0;
        }

        // Optimized models exported on Blender origin - place directly at terrain height
        result.trees.push(FoliageInstance {
            transform: Mat4::from_scale_rotation_translation(
                Vec3::splat(scale),
                Quat::from_rotation_y(angle),
                Vec3::new(world_x, height, world_z),
            ),
            model_index: model_idx,
            is_megaflora: mega,
        });
    }

    // Phase 3: Beach grass - DISABLED, replaced by grass3 LOD system in main.rs
    // The old beach_grass_0 model caused performance issues
    // let beach_grass_count = generate_beach_grass_for_chunk(
    //     &mut result,
    //     seed,
    //     chunk_size,
    //     offset_x,
    //     offset_z,
    //     &noise,
    // );

    result
}

/// Generate beach grass clumps for upper beach areas.
///
/// Spawns low-poly grass clumps (beach_grass_0 through beach_grass_3) on:
/// - Height range: 2.0m - 5.0m (above wet sand, below forest)
/// - Biome_t range: 0.45 - 0.65 (beach to treeline transition)
///
/// Density increases toward treeline, taller variations spawn higher up.
fn generate_beach_grass_for_chunk(
    result: &mut FoliageInstances,
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
    noise: &Perlin,
) -> u32 {
    let mut count = 0u32;

    // Beach grass density: 0.008 instances/m² (much lower than procedural grass)
    // This gives sparse clumps that don't overwhelm the beach
    let base_density = 0.008;
    let potential = (chunk_size * chunk_size * base_density) as u32;

    // Separate noise for beach grass placement
    let beach_noise = Perlin::new(seed.wrapping_add(88888));

    for i in 0..potential {
        // Deterministic random position
        let rx = noise.get([i as f64 * 0.17, 1200.0]) as f32;
        let rz = noise.get([i as f64 * 0.17, 1300.0]) as f32;
        let world_x = offset_x + (rx + 1.0) * 0.5 * chunk_size;
        let world_z = offset_z + (rz + 1.0) * 0.5 * chunk_size;

        let (height, _) = get_height_at(world_x, world_z, seed);
        let biome_t = get_biome_t(world_x, world_z, seed);

        // Beach grass zone: full beach + treeline (height 0.5-10.0m, biome_t 0.35-0.72)
        // 0.5m = just above waterline, 10.0m = through treeline/drift plateaus
        if height < 0.5 || height > 10.0 {
            continue;
        }
        if biome_t < 0.35 || biome_t > 0.72 {
            continue;
        }

        // Even density across beach with slight increase toward treeline
        let height_factor = ((height - 0.5) / 9.5).clamp(0.0, 1.0);
        let density_threshold = 0.5 + height_factor * 0.3; // 50-80% spawn rate

        let density_roll = (beach_noise.get([world_x as f64 * 0.3, world_z as f64 * 0.3]) + 1.0) * 0.5;
        if density_roll > density_threshold as f64 {
            continue;
        }

        // Clumping: beach grass grows in scattered patches
        let clump_noise = beach_noise.get([world_x as f64 * 0.08, world_z as f64 * 0.08]) as f32;
        if clump_noise < -0.4 {
            continue; // Gap between clumps
        }

        // Scale: 3.0-6.0 (larger, visible clumps)
        let base_scale = 3.0 + height_factor * 2.0;
        let noise_scale = (beach_noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]).abs() as f32) * 1.5;
        let scale = base_scale + noise_scale;

        // Random Y rotation
        let rotation = beach_noise.get([world_x as f64 * 0.7, world_z as f64 * 0.7]) as f32
            * std::f32::consts::TAU;

        result.shrubs.push(FoliageInstance {
            transform: Mat4::from_scale_rotation_translation(
                Vec3::splat(scale),
                Quat::from_rotation_y(rotation),
                Vec3::new(world_x, height + 0.1, world_z), // Small Y offset to sit on terrain
            ),
            model_index: SHRUB_SPECIES_BEACH_GRASS_0,
            is_megaflora: false,
        });

        count += 1;
    }

    count
}
