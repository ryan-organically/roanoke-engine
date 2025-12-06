//! Biome-Aware Flora and Fauna Spawning System
//!
//! This module handles procedural spawning of plants and animals based on:
//! - Biome type and conditions
//! - Elevation and moisture
//! - Biome transitions and blending
//! - Habitat overlap between biomes
//!
//! All spawning is deterministic based on world seed.

use crate::biome::{BiomeType, FloraType, FaunaType, get_flora_weights, get_fauna_weights};
use crate::terrain::{TerrainGenerator, TerrainData};
use crate::noise_util;
use glam::Vec3;

// ============================================================================
// SPAWN CONFIGURATION
// ============================================================================

/// Configuration for biome-aware spawning
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub seed: u32,
    pub flora_density: f32,        // Base flora per square meter
    pub fauna_density: f32,        // Base fauna per chunk
    pub enable_overlap: bool,      // Allow cross-biome spawning
    pub overlap_distance: f32,     // How far into adjacent biome to spawn
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            flora_density: 0.5,
            fauna_density: 2.0,
            enable_overlap: true,
            overlap_distance: 50.0,
        }
    }
}

// ============================================================================
// FLORA SPAWNING
// ============================================================================

/// A spawned flora instance
#[derive(Debug, Clone)]
pub struct FloraInstance {
    pub flora_type: FloraType,
    pub position: Vec3,
    pub rotation: f32,
    pub scale: f32,
    pub health: f32,          // 0.0-1.0, affects appearance
    pub growth_stage: f32,    // 0.0 = seedling, 1.0 = mature
    pub biome_source: BiomeType,
}

/// Flora spawner for a specific area
pub struct FloraSpawner {
    config: SpawnConfig,
}

impl FloraSpawner {
    pub fn new(config: SpawnConfig) -> Self {
        Self { config }
    }

    /// Generate flora for a chunk
    pub fn spawn_flora_for_chunk(
        &self,
        terrain_gen: &TerrainGenerator,
        chunk_x: i32,
        chunk_z: i32,
        chunk_size: f32,
    ) -> Vec<FloraInstance> {
        let mut instances = Vec::new();

        let offset_x = chunk_x as f32 * chunk_size;
        let offset_z = chunk_z as f32 * chunk_size;

        // Grid-based spawning with jitter
        let base_step = (1.0 / self.config.flora_density).sqrt().max(2.0);

        let mut z = 0.0;
        while z < chunk_size {
            let mut x = 0.0;
            while x < chunk_size {
                let world_x = offset_x + x;
                let world_z = offset_z + z;

                // Get terrain data
                let terrain = terrain_gen.get_terrain_at(world_x, world_z);

                // Skip water
                if terrain.height < 0.0 && !matches!(terrain.biome_type, BiomeType::SaltMarsh | BiomeType::Wetland) {
                    x += base_step;
                    continue;
                }

                // Get spawn candidates
                let candidates = self.get_spawn_candidates(&terrain);

                // Try to spawn
                if let Some(instance) = self.try_spawn_flora(
                    world_x, world_z, terrain.height, &terrain, &candidates
                ) {
                    instances.push(instance);
                }

                x += base_step;
            }
            z += base_step;
        }

        instances
    }

    /// Get flora types that can spawn at this terrain location
    fn get_spawn_candidates(&self, terrain: &TerrainData) -> Vec<(FloraType, f32)> {
        let mut candidates = get_flora_weights(terrain.biome_type);

        // Add secondary biome flora with reduced weight
        if let Some(secondary) = terrain.secondary_biome {
            if self.config.enable_overlap {
                let secondary_flora = get_flora_weights(secondary);
                for (flora, weight) in secondary_flora {
                    // Scale by blend factor
                    let adjusted_weight = weight * terrain.blend_factor * 0.5;
                    candidates.push((flora, adjusted_weight));
                }
            }
        }

        // Adjust weights based on moisture and temperature
        for (flora, weight) in &mut candidates {
            *weight *= self.get_environmental_modifier(*flora, terrain);
        }

        candidates
    }

    /// Get environmental modifier for a flora type
    fn get_environmental_modifier(&self, flora: FloraType, terrain: &TerrainData) -> f32 {
        let moisture_factor = match flora {
            // Water-loving plants
            FloraType::Cypress | FloraType::Willow | FloraType::Cattail |
            FloraType::Bulrush | FloraType::Pickerelweed | FloraType::WaterLily => {
                terrain.moisture.powf(0.5)
            }
            // Drought-tolerant
            FloraType::Palmetto | FloraType::Yaupon | FloraType::SeaOats => {
                1.0 - (terrain.moisture - 0.3).max(0.0) * 0.5
            }
            // Moderate
            _ => 1.0 - (terrain.moisture - 0.5).abs() * 0.3
        };

        let temp_factor = match flora {
            // Cold-hardy
            FloraType::Spruce | FloraType::Birch | FloraType::Lichen => {
                1.0 - terrain.temperature * 0.3
            }
            // Heat-loving
            FloraType::Palmetto | FloraType::LiveOak => {
                terrain.temperature.powf(0.5)
            }
            _ => 1.0
        };

        (moisture_factor * temp_factor).clamp(0.1, 2.0)
    }

    /// Try to spawn a flora instance at a position
    fn try_spawn_flora(
        &self,
        x: f32,
        z: f32,
        height: f32,
        terrain: &TerrainData,
        candidates: &[(FloraType, f32)],
    ) -> Option<FloraInstance> {
        let seed = self.config.seed
            .wrapping_add((x as u32).wrapping_mul(73856093))
            .wrapping_add((z as u32).wrapping_mul(19349663));

        // Spawn probability check
        let spawn_roll = noise_util::hash(seed);
        let density_mod = self.get_biome_density_modifier(terrain.biome_type);

        if spawn_roll > self.config.flora_density * density_mod {
            return None;
        }

        // Select flora type based on weights
        let total_weight: f32 = candidates.iter().map(|(_, w)| w).sum();
        if total_weight < 0.01 {
            return None;
        }

        let selection_roll = noise_util::hash(seed + 1) * total_weight;
        let mut cumulative = 0.0;
        let mut selected_flora = candidates[0].0;
        let mut source_biome = terrain.biome_type;

        for (flora, weight) in candidates {
            cumulative += weight;
            if selection_roll <= cumulative {
                selected_flora = *flora;
                break;
            }
        }

        // Check if this is a secondary biome flora
        if terrain.secondary_biome.is_some() {
            let primary_flora = get_flora_weights(terrain.biome_type);
            if !primary_flora.iter().any(|(f, _)| *f == selected_flora) {
                source_biome = terrain.secondary_biome.unwrap();
            }
        }

        // Generate instance properties
        let rotation = noise_util::hash(seed + 2) * std::f32::consts::PI * 2.0;
        let base_scale = self.get_flora_base_scale(selected_flora);
        let scale = base_scale * (0.7 + noise_util::hash(seed + 3) * 0.6);
        let health = 0.6 + noise_util::hash(seed + 4) * 0.4;
        let growth_stage = 0.5 + noise_util::hash(seed + 5) * 0.5;

        // Apply jitter to position
        let jitter_x = (noise_util::hash(seed + 6) - 0.5) * 2.0;
        let jitter_z = (noise_util::hash(seed + 7) - 0.5) * 2.0;

        Some(FloraInstance {
            flora_type: selected_flora,
            position: Vec3::new(x + jitter_x, height, z + jitter_z),
            rotation,
            scale,
            health,
            growth_stage,
            biome_source: source_biome,
        })
    }

    /// Get base scale for flora type
    fn get_flora_base_scale(&self, flora: FloraType) -> f32 {
        match flora {
            // Large trees
            FloraType::LiveOak | FloraType::Cypress | FloraType::Pine => 8.0,
            FloraType::Willow | FloraType::Birch | FloraType::Spruce => 6.0,
            FloraType::DeadTree => 5.0,

            // Shrubs
            FloraType::Palmetto | FloraType::Waxmyrtle => 2.0,
            FloraType::Yaupon | FloraType::Blueberry | FloraType::Azalea => 1.5,
            FloraType::MountainLaurel => 2.5,

            // Ground cover
            FloraType::SawGrass | FloraType::Cordgrass | FloraType::SeaOats => 1.0,
            FloraType::Fern => 0.6,
            FloraType::Moss | FloraType::Lichen => 0.3,

            // Marsh
            FloraType::Cattail | FloraType::Bulrush => 1.2,
            FloraType::Pickerelweed => 0.8,

            // Aquatic
            FloraType::Seaweed => 0.5,
            FloraType::WaterLily => 0.4,
        }
    }

    /// Get density modifier for biome
    fn get_biome_density_modifier(&self, biome: BiomeType) -> f32 {
        match biome {
            BiomeType::DeciduousForest => 2.0,
            BiomeType::Wetland | BiomeType::SaltMarsh => 1.5,
            BiomeType::RollingMountains => 0.8,
            BiomeType::MountainPeak => 0.3,
            BiomeType::Beach => 0.2,
            BiomeType::Ocean => 0.1,
            _ => 1.0,
        }
    }
}

// ============================================================================
// FAUNA SPAWNING
// ============================================================================

/// A spawned fauna instance
#[derive(Debug, Clone)]
pub struct FaunaInstance {
    pub fauna_type: FaunaType,
    pub position: Vec3,
    pub facing: f32,          // Direction in radians
    pub health: f32,
    pub age: FaunaAge,
    pub behavior_state: BehaviorState,
    pub home_biome: BiomeType,
    pub is_pack_leader: bool,
    pub pack_id: Option<u32>,
}

/// Age categories for fauna
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaunaAge {
    Juvenile,
    Adult,
    Elder,
}

/// Current behavior state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorState {
    Idle,
    Foraging,
    Resting,
    Alert,
    Traveling,
}

/// Fauna spawner
pub struct FaunaSpawner {
    config: SpawnConfig,
}

impl FaunaSpawner {
    pub fn new(config: SpawnConfig) -> Self {
        Self { config }
    }

    /// Generate fauna for a chunk
    pub fn spawn_fauna_for_chunk(
        &self,
        terrain_gen: &TerrainGenerator,
        chunk_x: i32,
        chunk_z: i32,
        chunk_size: f32,
    ) -> Vec<FaunaInstance> {
        let mut instances = Vec::new();

        let offset_x = chunk_x as f32 * chunk_size;
        let offset_z = chunk_z as f32 * chunk_size;

        // Sample terrain at chunk center and corners for biome distribution
        let center_terrain = terrain_gen.get_terrain_at(
            offset_x + chunk_size * 0.5,
            offset_z + chunk_size * 0.5
        );

        // Get primary biome fauna
        let candidates = self.get_fauna_candidates(&center_terrain);

        // Determine spawn count
        let spawn_count = (self.config.fauna_density *
            self.get_biome_fauna_modifier(center_terrain.biome_type)) as usize;

        // Track pack spawning
        let mut pack_counter: u32 = 0;

        for i in 0..spawn_count {
            let seed = self.config.seed
                .wrapping_add(chunk_x as u32 * 1000000)
                .wrapping_add(chunk_z as u32 * 1000)
                .wrapping_add(i as u32);

            if let Some(instance) = self.try_spawn_fauna(
                seed,
                offset_x, offset_z, chunk_size,
                &candidates,
                terrain_gen,
                &mut pack_counter,
            ) {
                instances.push(instance);
            }
        }

        instances
    }

    /// Get fauna candidates for terrain
    fn get_fauna_candidates(&self, terrain: &TerrainData) -> Vec<(FaunaType, f32)> {
        let mut candidates = get_fauna_weights(terrain.biome_type);

        // Add secondary biome fauna
        if let Some(secondary) = terrain.secondary_biome {
            if self.config.enable_overlap {
                let secondary_fauna = get_fauna_weights(secondary);
                for (fauna, weight) in secondary_fauna {
                    let adjusted = weight * terrain.blend_factor * 0.3;
                    candidates.push((fauna, adjusted));
                }
            }
        }

        // Environmental modifiers
        for (fauna, weight) in &mut candidates {
            *weight *= self.get_fauna_environmental_modifier(*fauna, terrain);
        }

        candidates
    }

    /// Environmental modifier for fauna
    fn get_fauna_environmental_modifier(&self, fauna: FaunaType, terrain: &TerrainData) -> f32 {
        let moisture_mod = match fauna {
            // Aquatic/semi-aquatic
            FaunaType::AmericanAlligator | FaunaType::Cottonmouth |
            FaunaType::Beaver | FaunaType::Muskrat => terrain.moisture,
            // Water-dependent
            FaunaType::GreatBlueHeron | FaunaType::Osprey => terrain.moisture.sqrt(),
            // Most animals prefer some moisture
            _ => 0.5 + terrain.moisture * 0.5
        };

        let height_mod = match fauna {
            // Mountain specialists
            FaunaType::EasternCougar | FaunaType::Bobcat => {
                if terrain.height > 30.0 { 1.5 } else { 0.7 }
            }
            // Lowland preference
            FaunaType::AmericanAlligator | FaunaType::WildBoar => {
                if terrain.height < 30.0 { 1.2 } else { 0.5 }
            }
            _ => 1.0
        };

        (moisture_mod * height_mod).clamp(0.1, 2.0)
    }

    /// Try to spawn a fauna instance
    fn try_spawn_fauna(
        &self,
        seed: u32,
        offset_x: f32,
        offset_z: f32,
        chunk_size: f32,
        candidates: &[(FaunaType, f32)],
        terrain_gen: &TerrainGenerator,
        pack_counter: &mut u32,
    ) -> Option<FaunaInstance> {
        // Position in chunk
        let x = offset_x + noise_util::hash(seed) * chunk_size;
        let z = offset_z + noise_util::hash(seed + 1) * chunk_size;

        let terrain = terrain_gen.get_terrain_at(x, z);

        // Skip water (except for aquatic species)
        if terrain.height < 0.0 {
            return None;
        }

        // Select fauna type
        let total_weight: f32 = candidates.iter().map(|(_, w)| w).sum();
        if total_weight < 0.01 {
            return None;
        }

        let selection_roll = noise_util::hash(seed + 2) * total_weight;
        let mut cumulative = 0.0;
        let mut selected_fauna = candidates[0].0;

        for (fauna, weight) in candidates {
            cumulative += weight;
            if selection_roll <= cumulative {
                selected_fauna = *fauna;
                break;
            }
        }

        // Generate properties
        let facing = noise_util::hash(seed + 3) * std::f32::consts::PI * 2.0;
        let health = 0.7 + noise_util::hash(seed + 4) * 0.3;

        // Age distribution
        let age_roll = noise_util::hash(seed + 5);
        let age = if age_roll < 0.15 { FaunaAge::Juvenile }
            else if age_roll < 0.85 { FaunaAge::Adult }
            else { FaunaAge::Elder };

        // Behavior state
        let behavior_roll = noise_util::hash(seed + 6);
        let behavior_state = if behavior_roll < 0.3 { BehaviorState::Idle }
            else if behavior_roll < 0.6 { BehaviorState::Foraging }
            else if behavior_roll < 0.8 { BehaviorState::Resting }
            else { BehaviorState::Traveling };

        // Pack behavior for social animals
        let (is_pack_leader, pack_id) = self.determine_pack_status(
            selected_fauna, seed, pack_counter
        );

        Some(FaunaInstance {
            fauna_type: selected_fauna,
            position: Vec3::new(x, terrain.height, z),
            facing,
            health,
            age,
            behavior_state,
            home_biome: terrain.biome_type,
            is_pack_leader,
            pack_id,
        })
    }

    /// Determine pack status for social animals
    fn determine_pack_status(
        &self,
        fauna: FaunaType,
        seed: u32,
        pack_counter: &mut u32,
    ) -> (bool, Option<u32>) {
        let is_social = matches!(fauna,
            FaunaType::GrayFox | FaunaType::RedWolf |
            FaunaType::WildBoar | FaunaType::WhitetailDeer
        );

        if !is_social {
            return (false, None);
        }

        let pack_roll = noise_util::hash(seed + 10);

        // 30% chance to be pack leader (starts new pack)
        if pack_roll < 0.3 {
            *pack_counter += 1;
            (true, Some(*pack_counter))
        } else if *pack_counter > 0 {
            // Join existing pack
            let pack = (noise_util::hash(seed + 11) * *pack_counter as f32) as u32 + 1;
            (false, Some(pack))
        } else {
            (false, None)
        }
    }

    /// Get fauna density modifier for biome
    fn get_biome_fauna_modifier(&self, biome: BiomeType) -> f32 {
        match biome {
            BiomeType::DeciduousForest => 1.5,
            BiomeType::Wetland => 1.8,
            BiomeType::SaltMarsh => 1.3,
            BiomeType::Grassland => 1.2,
            BiomeType::River => 1.4,
            BiomeType::MountainPeak => 0.3,
            BiomeType::Ocean => 0.1,
            BiomeType::Beach => 0.3,
            _ => 1.0,
        }
    }

    /// Get spawn points for a specific fauna type in a region
    pub fn get_habitat_spawn_points(
        &self,
        fauna: FaunaType,
        terrain_gen: &TerrainGenerator,
        region_x: f32,
        region_z: f32,
        region_size: f32,
    ) -> Vec<Vec3> {
        let mut points = Vec::new();

        // Get suitable biomes for this fauna
        let suitable_biomes: Vec<BiomeType> = BiomeType::iter()
            .into_iter()
            .filter(|b| get_fauna_weights(*b).iter().any(|(f, w)| *f == fauna && *w > 0.0))
            .collect();

        // Sample region for suitable habitats
        let step = 20.0;
        let mut z = region_z;
        while z < region_z + region_size {
            let mut x = region_x;
            while x < region_x + region_size {
                let terrain = terrain_gen.get_terrain_at(x, z);

                if suitable_biomes.contains(&terrain.biome_type) {
                    // Additional suitability check
                    let suitability = self.get_fauna_environmental_modifier(fauna, &terrain);
                    if suitability > 0.5 {
                        points.push(Vec3::new(x, terrain.height, z));
                    }
                }

                x += step;
            }
            z += step;
        }

        points
    }
}

// Helper trait for BiomeType iteration (simplified)
trait BiomeTypeIter {
    fn iter() -> Vec<BiomeType>;
}

impl BiomeTypeIter for BiomeType {
    fn iter() -> Vec<BiomeType> {
        vec![
            BiomeType::Ocean,
            BiomeType::Beach,
            BiomeType::SaltMarsh,
            BiomeType::CoastalScrub,
            BiomeType::Grassland,
            BiomeType::DeciduousForest,
            BiomeType::Wetland,
            BiomeType::River,
            BiomeType::Foothills,
            BiomeType::RollingMountains,
            BiomeType::MountainPeak,
            BiomeType::AlpineMeadow,
            BiomeType::Cave,
            BiomeType::Waterfall,
            BiomeType::CanyonRiver,
        ]
    }
}

// ============================================================================
// SPAWNER INTEGRATION
// ============================================================================

/// Combined spawner that manages all flora and fauna
pub struct BiomeSpawner {
    flora_spawner: FloraSpawner,
    fauna_spawner: FaunaSpawner,
}

impl BiomeSpawner {
    pub fn new(config: SpawnConfig) -> Self {
        Self {
            flora_spawner: FloraSpawner::new(config.clone()),
            fauna_spawner: FaunaSpawner::new(config),
        }
    }

    /// Spawn all entities for a chunk
    pub fn spawn_for_chunk(
        &self,
        terrain_gen: &TerrainGenerator,
        chunk_x: i32,
        chunk_z: i32,
        chunk_size: f32,
    ) -> ChunkSpawns {
        ChunkSpawns {
            flora: self.flora_spawner.spawn_flora_for_chunk(
                terrain_gen, chunk_x, chunk_z, chunk_size
            ),
            fauna: self.fauna_spawner.spawn_fauna_for_chunk(
                terrain_gen, chunk_x, chunk_z, chunk_size
            ),
        }
    }
}

/// All spawns for a chunk
#[derive(Debug)]
pub struct ChunkSpawns {
    pub flora: Vec<FloraInstance>,
    pub fauna: Vec<FaunaInstance>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::TerrainConfig;

    #[test]
    fn test_flora_spawning() {
        let config = SpawnConfig::default();
        let spawner = FloraSpawner::new(config);

        let terrain_config = TerrainConfig::default();
        let terrain_gen = TerrainGenerator::new(terrain_config);

        let flora = spawner.spawn_flora_for_chunk(&terrain_gen, 0, 0, 256.0);
        assert!(flora.len() > 0);
    }

    #[test]
    fn test_fauna_spawning() {
        let config = SpawnConfig::default();
        let spawner = FaunaSpawner::new(config);

        let terrain_config = TerrainConfig::default();
        let terrain_gen = TerrainGenerator::new(terrain_config);

        let fauna = spawner.spawn_fauna_for_chunk(&terrain_gen, 0, 0, 256.0);
        // May or may not have fauna depending on biome
        println!("Spawned {} fauna", fauna.len());
    }

    #[test]
    fn test_spawn_determinism() {
        let config = SpawnConfig::default();
        let spawner = FloraSpawner::new(config.clone());

        let terrain_config = TerrainConfig::default();
        let terrain_gen = TerrainGenerator::new(terrain_config);

        let flora1 = spawner.spawn_flora_for_chunk(&terrain_gen, 5, 5, 256.0);
        let flora2 = spawner.spawn_flora_for_chunk(&terrain_gen, 5, 5, 256.0);

        assert_eq!(flora1.len(), flora2.len());
        if !flora1.is_empty() {
            assert_eq!(flora1[0].flora_type, flora2[0].flora_type);
        }
    }
}
