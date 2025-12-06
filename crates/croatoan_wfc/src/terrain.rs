//! Advanced Terrain Generation System
//!
//! This module integrates all biome systems to generate square miles of terrain:
//! - Rolling mountains with ridged noise
//! - River valleys and waterfalls
//! - Salt marshes and wetlands
//! - Coastal features
//! - Cave entrances
//!
//! Everything is seed-based using Perlin noise for determinism.

use crate::biome::{BiomeGenerator, BiomeType, BiomeData, WorldGenConfig};
use crate::caves::{CaveGenerator, CaveGenConfig, CaveSystem};
use crate::rivers::{RiverGenerator, RiverGenConfig, RiverSystem};
use crate::noise_util::{self, fbm, ridged, turbulence};
use glam::{Vec2, Vec3};

// ============================================================================
// TERRAIN GENERATOR - MAIN INTEGRATION
// ============================================================================

/// Complete terrain generator integrating all systems
pub struct TerrainGenerator {
    pub config: TerrainConfig,
    biome_gen: BiomeGenerator,
    cave_gen: CaveGenerator,
    river_gen: RiverGenerator,

    // Cached data
    pub rivers: Vec<RiverSystem>,
    pub caves: Vec<CaveSystem>,
}

/// Master configuration for terrain generation
#[derive(Debug, Clone)]
pub struct TerrainConfig {
    pub seed: u32,
    pub world_size: f32,          // Total world size in meters
    pub chunk_size: f32,          // Size of each chunk
    pub sea_level: f32,
    pub mountain_height: f32,
    pub enable_caves: bool,
    pub enable_rivers: bool,
    pub enable_marshes: bool,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            world_size: 16000.0,  // ~10 square miles (16km x 16km)
            chunk_size: 256.0,
            sea_level: 0.0,
            mountain_height: 150.0,
            enable_caves: true,
            enable_rivers: true,
            enable_marshes: true,
        }
    }
}

impl TerrainGenerator {
    pub fn new(config: TerrainConfig) -> Self {
        let biome_config = WorldGenConfig {
            seed: config.seed,
            world_size: config.world_size,
            sea_level: config.sea_level,
            mountain_scale: config.mountain_height,
            river_frequency: 0.0008,
            cave_frequency: 0.003,
        };

        let cave_config = CaveGenConfig {
            seed: config.seed,
            ..Default::default()
        };

        let river_config = RiverGenConfig {
            seed: config.seed,
            ..Default::default()
        };

        Self {
            config,
            biome_gen: BiomeGenerator::new(biome_config),
            cave_gen: CaveGenerator::new(cave_config),
            river_gen: RiverGenerator::new(river_config),
            rivers: Vec::new(),
            caves: Vec::new(),
        }
    }

    /// Initialize world features (rivers, caves) - call once at world creation
    pub fn initialize_world_features(&mut self) {
        if self.config.enable_rivers {
            self.generate_world_rivers();
        }
        // Caves are generated on-demand per chunk
    }

    /// Generate river systems for the world
    fn generate_world_rivers(&mut self) {
        let half_world = self.config.world_size * 0.5;

        // Scan for river sources in highland areas
        let scan_step = 500.0;
        let mut sources = Vec::new();

        let mut z = -half_world;
        while z < half_world {
            let mut x = -half_world;
            while x < half_world {
                let biome = self.biome_gen.get_biome_at(x, z);

                if self.river_gen.should_have_river_source(x, z, biome.height, biome.moisture) {
                    sources.push(Vec3::new(x, biome.height, z));
                }
                x += scan_step;
            }
            z += scan_step;
        }

        // Generate rivers from sources
        for source in sources.iter().take(20) { // Limit number of major rivers
            let terrain_sampler = |sx: f32, sz: f32| -> f32 {
                self.biome_gen.get_biome_at(sx, sz).height
            };

            let local_seed = (source.x as u32).wrapping_mul(73856093)
                ^ (source.z as u32).wrapping_mul(19349663);

            let river = self.river_gen.generate_river_system(
                *source,
                self.config.sea_level,
                &terrain_sampler,
                local_seed
            );

            self.rivers.push(river);
        }
    }

    /// Get complete terrain data at a world position
    pub fn get_terrain_at(&self, x: f32, z: f32) -> TerrainData {
        // Get base biome data
        let biome = self.biome_gen.get_biome_at(x, z);

        // Apply river modifications
        let river_depth = if self.config.enable_rivers {
            self.river_gen.get_river_carve_depth(x, z, &self.rivers)
        } else {
            0.0
        };

        // Check for cave entrance
        let is_cave_entrance = self.config.enable_caves &&
            self.cave_gen.should_have_cave(x, z, biome.height, self.calculate_slope(x, z));

        // Final height with all modifiers
        let final_height = biome.height + river_depth;

        // Determine terrain features
        let features = self.determine_features(x, z, &biome, is_cave_entrance);

        TerrainData {
            height: final_height,
            biome_type: biome.primary_biome,
            secondary_biome: biome.secondary_biome,
            blend_factor: biome.blend_factor,
            color: biome.color,
            moisture: biome.moisture,
            temperature: biome.temperature,
            is_river: river_depth < -0.5,
            is_cave_entrance,
            features,
        }
    }

    /// Calculate terrain slope at a position
    fn calculate_slope(&self, x: f32, z: f32) -> f32 {
        let delta = 5.0;
        let h_center = self.biome_gen.get_biome_at(x, z).height;
        let h_x = self.biome_gen.get_biome_at(x + delta, z).height;
        let h_z = self.biome_gen.get_biome_at(x, z + delta).height;

        let dx = (h_x - h_center) / delta;
        let dz = (h_z - h_center) / delta;

        (dx * dx + dz * dz).sqrt()
    }

    /// Determine terrain features at a position
    fn determine_features(&self, x: f32, z: f32, biome: &BiomeData, is_cave: bool) -> Vec<TerrainFeature> {
        let mut features = Vec::new();
        let seed = self.config.seed;

        // Check for various features based on biome and noise
        let feature_noise = turbulence(
            Vec2::new(x * 0.02, z * 0.02),
            3, 2.0, 0.5, seed + 8000
        );

        match biome.primary_biome {
            BiomeType::SaltMarsh => {
                if feature_noise > 0.3 {
                    features.push(TerrainFeature::TidalChannel);
                }
                if feature_noise > 0.6 {
                    features.push(TerrainFeature::MudFlat);
                }
                if noise_util::hash((x as u32) ^ (z as u32)) > 0.7 {
                    features.push(TerrainFeature::SaltPan);
                }
            }
            BiomeType::RollingMountains | BiomeType::MountainPeak => {
                if feature_noise > 0.7 {
                    features.push(TerrainFeature::Cliff);
                }
                if biome.height > 100.0 && feature_noise > 0.5 {
                    features.push(TerrainFeature::SnowPatch);
                }
                if feature_noise > 0.8 {
                    features.push(TerrainFeature::Outcrop);
                }
            }
            BiomeType::Waterfall => {
                features.push(TerrainFeature::Waterfall);
                features.push(TerrainFeature::MistZone);
            }
            BiomeType::Wetland => {
                if feature_noise > 0.4 {
                    features.push(TerrainFeature::StandingWater);
                }
                if feature_noise > 0.6 {
                    features.push(TerrainFeature::Hummock);
                }
            }
            _ => {}
        }

        if is_cave {
            features.push(TerrainFeature::CaveEntrance);
        }

        features
    }

    /// Generate terrain chunk mesh
    pub fn generate_chunk(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> TerrainChunkData {
        let offset_x = chunk_x as f32 * self.config.chunk_size;
        let offset_z = chunk_z as f32 * self.config.chunk_size;

        let grid_size = 65u32; // 64x64 quads + 1
        let scale = self.config.chunk_size / 64.0;

        let mut positions = Vec::with_capacity((grid_size * grid_size) as usize);
        let mut colors = Vec::with_capacity((grid_size * grid_size) as usize);
        let mut biome_ids = Vec::with_capacity((grid_size * grid_size) as usize);

        // Generate vertices
        for z in 0..grid_size {
            for x in 0..grid_size {
                let world_x = offset_x + x as f32 * scale;
                let world_z = offset_z + z as f32 * scale;

                let terrain = self.get_terrain_at(world_x, world_z);

                positions.push([world_x, terrain.height, world_z]);
                colors.push(terrain.color);
                biome_ids.push(terrain.biome_type as u8);
            }
        }

        // Generate indices
        let mut indices = Vec::with_capacity((64 * 64 * 2 * 3) as usize);
        for z in 0..64u32 {
            for x in 0..64u32 {
                let top_left = z * grid_size + x;
                let top_right = top_left + 1;
                let bottom_left = (z + 1) * grid_size + x;
                let bottom_right = bottom_left + 1;

                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        // Calculate normals
        let normals = calculate_smooth_normals(&positions, &indices);

        TerrainChunkData {
            chunk_x,
            chunk_z,
            positions,
            colors,
            normals,
            indices,
            biome_ids,
        }
    }

    /// Get cave system at a position (generates if needed)
    pub fn get_or_create_cave(&mut self, x: f32, z: f32) -> Option<&CaveSystem> {
        let terrain = self.get_terrain_at(x, z);

        if !terrain.is_cave_entrance {
            return None;
        }

        // Check if cave already generated - use index to avoid borrow issues
        let existing_idx = self.caves.iter().position(|cave| {
            let dist = ((cave.entrance_pos.x - x).powi(2) + (cave.entrance_pos.z - z).powi(2)).sqrt();
            dist < 10.0
        });

        if let Some(idx) = existing_idx {
            return Some(&self.caves[idx]);
        }

        // Generate new cave
        let entrance = Vec3::new(x, terrain.height, z);
        let local_seed = (x as u32).wrapping_mul(73856093) ^ (z as u32).wrapping_mul(19349663);
        let cave = self.cave_gen.generate_cave_system(entrance, local_seed);
        self.caves.push(cave);

        self.caves.last()
    }
}

// ============================================================================
// TERRAIN DATA STRUCTURES
// ============================================================================

/// Complete terrain data at a position
#[derive(Debug, Clone)]
pub struct TerrainData {
    pub height: f32,
    pub biome_type: BiomeType,
    pub secondary_biome: Option<BiomeType>,
    pub blend_factor: f32,
    pub color: [f32; 3],
    pub moisture: f32,
    pub temperature: f32,
    pub is_river: bool,
    pub is_cave_entrance: bool,
    pub features: Vec<TerrainFeature>,
}

/// Terrain features that can be present
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainFeature {
    // Water features
    TidalChannel,
    StandingWater,
    Waterfall,
    MistZone,

    // Marsh features
    MudFlat,
    SaltPan,
    Hummock,

    // Mountain features
    Cliff,
    Outcrop,
    SnowPatch,
    Scree,

    // Cave features
    CaveEntrance,
    Sinkhole,

    // Coastal
    TidePool,
    Sandbar,
}

/// Data for a generated terrain chunk
#[derive(Debug)]
pub struct TerrainChunkData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub biome_ids: Vec<u8>,
}

// ============================================================================
// SALT MARSH GENERATION
// ============================================================================

/// Salt marsh-specific generation
pub struct SaltMarshGenerator {
    seed: u32,
}

impl SaltMarshGenerator {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// Get salt marsh detail at a position
    pub fn get_marsh_detail(&self, x: f32, z: f32) -> SaltMarshDetail {
        // Tidal channels use low-frequency noise
        let channel_noise = fbm(
            Vec2::new(x * 0.005, z * 0.005),
            4, 2.0, 0.5, self.seed + 9000
        );

        let is_channel = channel_noise.abs() < 0.15;

        // Salt pans in higher areas
        let pan_noise = turbulence(
            Vec2::new(x * 0.02, z * 0.02),
            3, 2.0, 0.5, self.seed + 9100
        );
        let is_salt_pan = pan_noise > 0.7 && !is_channel;

        // Mud flat vs vegetated
        let veg_noise = fbm(
            Vec2::new(x * 0.01, z * 0.01),
            3, 2.0, 0.5, self.seed + 9200
        );
        let vegetation_density = ((veg_noise + 1.0) * 0.5).clamp(0.0, 1.0);

        // Water level (tidal variation)
        let tide_level = fbm(
            Vec2::new(x * 0.001, z * 0.001),
            2, 2.0, 0.5, self.seed + 9300
        ) * 0.3;

        // Height modification for marsh
        let height_mod = if is_channel {
            -0.5 - channel_noise.abs() * 0.5
        } else if is_salt_pan {
            0.2
        } else {
            veg_noise * 0.3
        };

        // Color based on features
        let color = if is_channel {
            [0.15, 0.35, 0.40] // Water
        } else if is_salt_pan {
            [0.85, 0.82, 0.75] // White salt
        } else {
            // Blend green/brown based on vegetation
            lerp_color(
                [0.50, 0.42, 0.30], // Mud
                [0.35, 0.50, 0.25], // Cordgrass
                vegetation_density
            )
        };

        SaltMarshDetail {
            is_channel,
            is_salt_pan,
            vegetation_density,
            tide_level,
            height_mod,
            color,
        }
    }

    /// Generate cordgrass instances for a chunk
    pub fn generate_cordgrass(
        &self,
        offset_x: f32,
        offset_z: f32,
        chunk_size: f32,
    ) -> Vec<GrassInstance> {
        let mut grass = Vec::new();
        let density = 0.5; // Blades per square meter
        let step = (1.0_f32 / density).sqrt();

        let mut z = 0.0;
        while z < chunk_size {
            let mut x = 0.0;
            while x < chunk_size {
                let world_x = offset_x + x;
                let world_z = offset_z + z;

                let detail = self.get_marsh_detail(world_x, world_z);

                // Only spawn grass where appropriate
                if !detail.is_channel && !detail.is_salt_pan && detail.vegetation_density > 0.3 {
                    let spawn_chance = noise_util::hash(
                        (world_x as u32).wrapping_mul(73856093) ^
                        (world_z as u32).wrapping_mul(19349663)
                    );

                    if spawn_chance < detail.vegetation_density {
                        // Cordgrass is tall and clumped
                        let height = 0.8 + noise_util::hash(
                            (world_x as u32 + 1).wrapping_mul(73856093) ^
                            (world_z as u32 + 1).wrapping_mul(19349663)
                        ) * 1.2;

                        grass.push(GrassInstance {
                            position: [world_x, 0.0, world_z], // Y set by terrain
                            height,
                            color: detail.color,
                            bend: 0.2,
                        });
                    }
                }
                x += step;
            }
            z += step;
        }

        grass
    }
}

/// Salt marsh detail data
#[derive(Debug, Clone)]
pub struct SaltMarshDetail {
    pub is_channel: bool,
    pub is_salt_pan: bool,
    pub vegetation_density: f32,
    pub tide_level: f32,
    pub height_mod: f32,
    pub color: [f32; 3],
}

/// A grass blade instance
#[derive(Debug, Clone)]
pub struct GrassInstance {
    pub position: [f32; 3],
    pub height: f32,
    pub color: [f32; 3],
    pub bend: f32,
}

// ============================================================================
// MOUNTAIN TERRAIN DETAILS
// ============================================================================

/// Mountain-specific terrain features
pub struct MountainGenerator {
    seed: u32,
}

impl MountainGenerator {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// Get mountain detail at a position
    pub fn get_mountain_detail(&self, x: f32, z: f32, base_height: f32) -> MountainDetail {
        // Ridge lines using ridged noise
        let ridge_noise = ridged(
            Vec2::new(x * 0.003, z * 0.003),
            5, 2.2, 0.5, self.seed + 10000
        );

        // Rocky outcrops
        let outcrop_noise = turbulence(
            Vec2::new(x * 0.02, z * 0.02),
            3, 2.5, 0.5, self.seed + 10100
        );
        let is_outcrop = outcrop_noise > 0.75;

        // Cliff faces (steep areas)
        let slope_x = ridged(Vec2::new((x + 5.0) * 0.003, z * 0.003), 5, 2.2, 0.5, self.seed + 10000)
            - ridge_noise;
        let slope_z = ridged(Vec2::new(x * 0.003, (z + 5.0) * 0.003), 5, 2.2, 0.5, self.seed + 10000)
            - ridge_noise;
        let slope = (slope_x * slope_x + slope_z * slope_z).sqrt() * 10.0;
        let is_cliff = slope > 0.4;

        // Scree fields at base of cliffs
        let scree_noise = fbm(
            Vec2::new(x * 0.01, z * 0.01),
            3, 2.0, 0.5, self.seed + 10200
        );
        let is_scree = slope > 0.2 && slope < 0.4 && scree_noise > 0.3;

        // Snow above treeline
        let snow_line = 90.0 + fbm(Vec2::new(x * 0.002, z * 0.002), 2, 2.0, 0.5, self.seed + 10300) * 20.0;
        let snow_amount = ((base_height - snow_line) / 30.0).clamp(0.0, 1.0);

        // Height modification from ridges
        let height_mod = ridge_noise * 30.0;

        // Color based on features
        let base_rock_color = [0.50, 0.48, 0.45]; // Gray rock
        let snow_color = [0.95, 0.95, 0.98];
        let scree_color = [0.55, 0.52, 0.48];

        let color = if snow_amount > 0.5 {
            lerp_color(base_rock_color, snow_color, snow_amount)
        } else if is_scree {
            scree_color
        } else if is_cliff {
            [0.40, 0.38, 0.35] // Darker cliff face
        } else {
            base_rock_color
        };

        MountainDetail {
            ridge_factor: ridge_noise,
            is_outcrop,
            is_cliff,
            is_scree,
            snow_amount,
            height_mod,
            color,
        }
    }

    /// Generate rock instances for mountain terrain
    pub fn generate_mountain_rocks(
        &self,
        offset_x: f32,
        offset_z: f32,
        chunk_size: f32,
    ) -> Vec<RockInstance> {
        let mut rocks = Vec::new();
        let step = 15.0; // Check every 15 meters

        let mut z = 0.0;
        while z < chunk_size {
            let mut x = 0.0;
            while x < chunk_size {
                let world_x = offset_x + x;
                let world_z = offset_z + z;

                let rock_seed = (world_x as u32).wrapping_mul(73856093) ^
                    (world_z as u32).wrapping_mul(19349663) ^
                    self.seed;

                // Scatter probability
                if noise_util::hash(rock_seed) > 0.7 {
                    let detail = self.get_mountain_detail(world_x, world_z, 50.0);

                    // More rocks on outcrops and scree
                    let spawn_mult = if detail.is_outcrop { 3.0 }
                        else if detail.is_scree { 2.0 }
                        else { 1.0 };

                    if noise_util::hash(rock_seed + 1) < 0.3 * spawn_mult {
                        let scale = 0.5 + noise_util::hash(rock_seed + 2) * 2.5;
                        let rotation = noise_util::hash(rock_seed + 3) * std::f32::consts::PI * 2.0;

                        rocks.push(RockInstance {
                            position: [world_x, 0.0, world_z],
                            scale,
                            rotation,
                            rock_type: if detail.is_scree {
                                MountainRockType::Scree
                            } else if detail.is_outcrop {
                                MountainRockType::Outcrop
                            } else {
                                MountainRockType::Boulder
                            },
                        });
                    }
                }
                x += step;
            }
            z += step;
        }

        rocks
    }
}

/// Mountain detail data
#[derive(Debug, Clone)]
pub struct MountainDetail {
    pub ridge_factor: f32,
    pub is_outcrop: bool,
    pub is_cliff: bool,
    pub is_scree: bool,
    pub snow_amount: f32,
    pub height_mod: f32,
    pub color: [f32; 3],
}

/// A rock instance for mountains
#[derive(Debug, Clone)]
pub struct RockInstance {
    pub position: [f32; 3],
    pub scale: f32,
    pub rotation: f32,
    pub rock_type: MountainRockType,
}

/// Types of mountain rocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountainRockType {
    Boulder,
    Outcrop,
    Scree,
    Cliff,
}

// ============================================================================
// UTILITIES
// ============================================================================

/// Calculate smooth vertex normals
fn calculate_smooth_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];

    for triangle in indices.chunks(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        let p0 = Vec3::from_array(positions[i0]);
        let p1 = Vec3::from_array(positions[i1]);
        let p2 = Vec3::from_array(positions[i2]);

        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let face_normal = edge1.cross(edge2);

        for &i in &[i0, i1, i2] {
            normals[i][0] += face_normal.x;
            normals[i][1] += face_normal.y;
            normals[i][2] += face_normal.z;
        }
    }

    for normal in &mut normals {
        let n = Vec3::from_array(*normal);
        let normalized = n.normalize_or_zero();
        if normalized.length() > 0.0 {
            *normal = normalized.to_array();
        } else {
            *normal = [0.0, 1.0, 0.0]; // Default up
        }
    }

    normals
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Color interpolation
fn lerp_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_generator() {
        let config = TerrainConfig::default();
        let gen = TerrainGenerator::new(config);

        // Test various positions
        let terrain1 = gen.get_terrain_at(0.0, 0.0);
        let terrain2 = gen.get_terrain_at(1000.0, 1000.0);
        let terrain3 = gen.get_terrain_at(-2000.0, 0.0); // Inland

        // Verify data is generated
        assert!(terrain1.height.is_finite());
        assert!(terrain2.height.is_finite());
        assert!(terrain3.height > terrain1.height); // Inland should be higher
    }

    #[test]
    fn test_chunk_generation() {
        let config = TerrainConfig::default();
        let gen = TerrainGenerator::new(config);

        let chunk = gen.generate_chunk(0, 0);

        assert_eq!(chunk.positions.len(), 65 * 65);
        assert_eq!(chunk.colors.len(), 65 * 65);
        assert_eq!(chunk.normals.len(), 65 * 65);
        assert_eq!(chunk.indices.len(), 64 * 64 * 2 * 3);
    }

    #[test]
    fn test_salt_marsh() {
        let marsh_gen = SaltMarshGenerator::new(12345);
        let detail = marsh_gen.get_marsh_detail(100.0, 100.0);

        assert!(detail.vegetation_density >= 0.0 && detail.vegetation_density <= 1.0);
    }

    #[test]
    fn test_mountain_detail() {
        let mountain_gen = MountainGenerator::new(12345);
        let detail = mountain_gen.get_mountain_detail(100.0, 100.0, 80.0);

        assert!(detail.ridge_factor >= 0.0);
        assert!(detail.snow_amount >= 0.0 && detail.snow_amount <= 1.0);
    }
}
