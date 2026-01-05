//! Comprehensive Cave System
//!
//! This module generates a full cave network with:
//! - Multi-level cave systems (entrance → passages → chambers → depths)
//! - Bones and skeletal remains
//! - Archaeological artifacts
//! - Cave-specific flora and fauna
//! - Stalactites, stalagmites, and mineral deposits
//!
//! All generation is seed-based using 3D Perlin noise.

use crate::noise_util::{self, fbm, turbulence, fbm_3d, noise_gradient_3d};
use glam::{Vec2, Vec3};

// ============================================================================
// CAVE STRUCTURE DEFINITIONS
// ============================================================================

/// Types of cave sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveSectionType {
    Entrance,       // Surface opening, natural light
    Passage,        // Narrow connecting tunnels
    Chamber,        // Large open rooms
    Pool,           // Underground water
    Shaft,          // Vertical drops
    DeadEnd,        // Terminates here
    Junction,       // Multiple paths branch
    SacredChamber,  // Contains artifacts/bones
}

/// Types of cave features/decorations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveFeature {
    // Formations
    Stalactite,
    Stalagmite,
    Column,
    Flowstone,
    CavePool,
    CrystalCluster,
    MineralVein,

    // Organic
    Bones,
    AncientBones,       // Very old, partially fossilized
    MassGrave,          // Multiple skeletal remains
    AnimalDen,          // Bears, etc.

    // Artifacts
    PotteryShards,
    StoneTools,
    Arrowheads,
    CaveArt,            // Petroglyphs
    AncientFirepit,
    BurialSite,
    RitualAltar,
    CachedSupplies,     // Hidden storage

    // Environmental
    GlowingMoss,
    CaveRoot,           // Tree roots from above
    BatColony,
    GuanoDeposit,
}

/// Artifact rarity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactRarity {
    Common,         // Pottery shards, basic tools
    Uncommon,       // Arrowheads, worked bone
    Rare,           // Cave art, burial items
    Legendary,      // Ritual altars, ancient artifacts
}

/// A cave system anchored at a surface entrance
#[derive(Debug, Clone)]
pub struct CaveSystem {
    pub entrance_pos: Vec3,
    pub seed: u32,
    pub total_length: f32,
    pub max_depth: f32,
    pub num_chambers: u32,
    pub sections: Vec<CaveSection>,
    pub features: Vec<CaveFeatureInstance>,
}

/// A single section of a cave
#[derive(Debug, Clone)]
pub struct CaveSection {
    pub section_type: CaveSectionType,
    pub center: Vec3,
    pub radius: f32,          // For chambers
    pub length: f32,          // For passages
    pub direction: Vec3,      // Primary direction
    pub connections: Vec<u32>, // Indices to connected sections
    pub has_water: bool,
    pub light_level: f32,     // 0.0 = pitch black, 1.0 = entrance light
}

/// An instance of a cave feature at a specific location
#[derive(Debug, Clone)]
pub struct CaveFeatureInstance {
    pub feature: CaveFeature,
    pub position: Vec3,
    pub rotation: f32,
    pub scale: f32,
    pub section_index: u32,
    pub rarity: ArtifactRarity,
}

// ============================================================================
// PERLIN WORM STRUCTURES
// ============================================================================

/// A point along a Perlin worm tunnel path
#[derive(Debug, Clone)]
pub struct WormPoint {
    pub position: Vec3,
    pub radius: f32,              // Tunnel radius at this point
    pub tangent: Vec3,            // Direction of travel (normalized)
    pub humidity: f32,            // 0.0-1.0, affects orb spawning
    pub depth_from_surface: f32,  // Distance below terrain surface
}

/// A complete Perlin worm tunnel (can have branches)
#[derive(Debug, Clone)]
pub struct WormTunnel {
    pub seed: u32,
    pub entrance: Vec3,
    pub points: Vec<WormPoint>,
    pub branches: Vec<WormTunnel>,
    pub total_length: f32,
    pub max_depth: f32,
}

/// Configuration for Perlin worm generation
#[derive(Debug, Clone)]
pub struct WormConfig {
    pub seed: u32,
    pub step_size: f32,           // Distance between sample points (2.0)
    pub min_radius: f32,          // Minimum tunnel radius (2.0)
    pub max_radius: f32,          // Maximum tunnel radius (12.0)
    pub radius_frequency: f32,    // Noise frequency for radius variation (0.02)
    pub direction_frequency: f32, // Noise frequency for direction changes (0.015)
    pub branch_probability: f32,  // Chance per step to branch (0.02)
    pub min_tunnel_length: f32,   // Minimum worm length before termination (50.0)
    pub max_tunnel_length: f32,   // Maximum worm length (400.0)
    pub descent_bias: f32,        // Tendency to go downward (0.15)
    pub humidity_frequency: f32,  // Noise freq for humidity zones (0.05)
}

impl Default for WormConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            step_size: 2.0,
            min_radius: 2.0,
            max_radius: 12.0,
            radius_frequency: 0.02,
            direction_frequency: 0.015,
            branch_probability: 0.15, // Increased for testing (was 0.02)
            min_tunnel_length: 50.0,
            max_tunnel_length: 400.0,
            descent_bias: 0.15,
            humidity_frequency: 0.05,
        }
    }
}

// ============================================================================
// BIOLUMINESCENT ORB STRUCTURES
// ============================================================================

/// Type of bioluminescent organism
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BioOrbType {
    Fungi,       // Mushroom-like clusters (blue-green)
    Moss,        // Spreading patches (cyan)
    Crystal,     // Crystalline formations (purple)
    Pool,        // Glowing water puddle (warm yellow)
}

/// A bioluminescent orb/fungi cluster in a cave
#[derive(Debug, Clone)]
pub struct BioOrb {
    pub position: Vec3,
    pub normal: Vec3,         // Surface normal (for wall/ceiling attachment)
    pub color: [f32; 3],      // RGB glow color
    pub intensity: f32,       // Base intensity (0.5-2.0)
    pub pulse_phase: f32,     // Random phase for pulsing (0-2*PI)
    pub pulse_speed: f32,     // Pulse frequency (0.3-0.8 Hz)
    pub cluster_size: f32,    // Scale factor (0.5-2.0)
    pub orb_type: BioOrbType,
}

/// Configuration for cave generation
#[derive(Debug, Clone)]
pub struct CaveGenConfig {
    pub seed: u32,
    pub min_cave_length: f32,
    pub max_cave_length: f32,
    pub min_depth: f32,
    pub max_depth: f32,
    pub passage_width: f32,
    pub chamber_probability: f32,
    pub branch_probability: f32,
    pub water_probability: f32,
    pub artifact_density: f32,
    pub bone_density: f32,
}

impl Default for CaveGenConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            min_cave_length: 50.0,
            max_cave_length: 500.0,
            min_depth: 10.0,
            max_depth: 80.0,
            passage_width: 3.0,
            chamber_probability: 0.25,
            branch_probability: 0.15,
            water_probability: 0.20,
            artifact_density: 0.05,
            bone_density: 0.08,
        }
    }
}

// ============================================================================
// CAVE GENERATOR
// ============================================================================

/// Main cave system generator
#[allow(dead_code)]
pub struct CaveGenerator {
    config: CaveGenConfig,
    noise_seed_main: u32,
    noise_seed_detail: u32,
    noise_seed_feature: u32,
}

impl CaveGenerator {
    pub fn new(config: CaveGenConfig) -> Self {
        let seed = config.seed;
        Self {
            config,
            noise_seed_main: seed,
            noise_seed_detail: seed.wrapping_add(1000),
            noise_seed_feature: seed.wrapping_add(2000),
        }
    }

    /// Check if there should be a cave entrance at this world position
    pub fn should_have_cave(&self, x: f32, z: f32, terrain_height: f32, slope: f32) -> bool {
        // Caves need:
        // 1. Above sea level
        // 2. In hilly/mountainous terrain (slope > threshold)
        // 3. Noise check passes

        if terrain_height < 5.0 {
            return false;
        }

        if slope < 0.1 {
            return false; // Too flat
        }

        // Use noise for cave distribution
        let cave_noise = turbulence(
            Vec2::new(x * 0.003, z * 0.003),
            4, 2.5, 0.5, self.noise_seed_main
        );

        // Higher terrain = more caves
        let elevation_factor = ((terrain_height - 5.0) / 100.0).clamp(0.0, 1.0);

        cave_noise * (0.5 + elevation_factor * 0.5) > 0.75
    }

    /// Generate a complete cave system from an entrance point
    pub fn generate_cave_system(&self, entrance: Vec3, local_seed: u32) -> CaveSystem {
        let combined_seed = self.config.seed.wrapping_add(local_seed);
        let mut sections = Vec::new();
        let mut features = Vec::new();

        // Determine cave properties from noise
        let length_factor = noise_util::hash(combined_seed);
        let total_length = lerp(
            self.config.min_cave_length,
            self.config.max_cave_length,
            length_factor
        );

        let depth_factor = noise_util::hash(combined_seed + 100);
        let max_depth = lerp(
            self.config.min_depth,
            self.config.max_depth,
            depth_factor
        );

        // Create entrance section
        let entrance_section = CaveSection {
            section_type: CaveSectionType::Entrance,
            center: entrance,
            radius: 4.0 + noise_util::hash(combined_seed + 200) * 4.0,
            length: 0.0,
            direction: Vec3::new(0.0, -0.3, -1.0).normalize(), // Heading inward/down
            connections: vec![1], // Will connect to first passage
            has_water: false,
            light_level: 1.0,
        };
        sections.push(entrance_section);

        // Generate main passage from entrance
        let mut current_pos = entrance + Vec3::new(0.0, -2.0, -5.0);
        let mut current_dir = Vec3::new(
            noise_util::hash(combined_seed + 300) * 2.0 - 1.0,
            -0.2,
            -1.0
        ).normalize();
        let mut current_depth = 2.0;
        let mut section_index: u32 = 1;
        let mut remaining_length = total_length;
        let mut num_chambers = 0;

        // Track generation with a stack for branching
        let mut generation_stack: Vec<(Vec3, Vec3, f32, f32, u32)> = vec![];

        while remaining_length > 0.0 {
            let section_seed = combined_seed.wrapping_add(section_index * 1000);

            // Determine section type
            let type_roll = noise_util::hash(section_seed);
            let section_type = if current_depth > max_depth * 0.8 {
                // Deep in cave - dead ends more likely
                if type_roll > 0.7 {
                    CaveSectionType::DeadEnd
                } else if type_roll > 0.5 {
                    CaveSectionType::Chamber
                } else {
                    CaveSectionType::Passage
                }
            } else if type_roll < self.config.chamber_probability {
                CaveSectionType::Chamber
            } else if type_roll < self.config.chamber_probability + self.config.branch_probability {
                CaveSectionType::Junction
            } else {
                CaveSectionType::Passage
            };

            // Calculate section dimensions
            let (section_length, section_radius) = match section_type {
                CaveSectionType::Entrance => (0.0, 5.0),
                CaveSectionType::Passage => (15.0 + noise_util::hash(section_seed + 10) * 25.0, self.config.passage_width),
                CaveSectionType::Chamber => (0.0, 8.0 + noise_util::hash(section_seed + 20) * 15.0),
                CaveSectionType::Pool => (0.0, 6.0 + noise_util::hash(section_seed + 30) * 10.0),
                CaveSectionType::Shaft => (10.0 + noise_util::hash(section_seed + 40) * 20.0, 3.0),
                CaveSectionType::DeadEnd => (5.0 + noise_util::hash(section_seed + 50) * 10.0, 2.0),
                CaveSectionType::Junction => (0.0, 5.0 + noise_util::hash(section_seed + 60) * 5.0),
                CaveSectionType::SacredChamber => (0.0, 12.0 + noise_util::hash(section_seed + 70) * 10.0),
            };

            // Update direction with noise-based wandering
            let wander_x = (fbm(Vec2::new(current_pos.x * 0.05, current_pos.z * 0.05), 2, 2.0, 0.5, section_seed) * 0.3) as f32;
            let wander_z = (fbm(Vec2::new(current_pos.z * 0.05, current_pos.x * 0.05), 2, 2.0, 0.5, section_seed + 100) * 0.3) as f32;

            // Bias downward
            let new_dir = Vec3::new(
                current_dir.x + wander_x,
                current_dir.y - 0.1, // Tend downward
                current_dir.z + wander_z
            ).normalize();
            current_dir = new_dir;

            // Move position
            let move_dist = if section_type == CaveSectionType::Passage || section_type == CaveSectionType::Shaft {
                section_length
            } else {
                section_radius * 2.0
            };

            let new_pos = current_pos + current_dir * move_dist;
            current_depth += (current_pos.y - new_pos.y).max(0.0);

            // Check for water
            let has_water = noise_util::hash(section_seed + 500) < self.config.water_probability
                && section_type != CaveSectionType::Shaft;

            // Calculate light level (fades with depth)
            let light_level = (1.0 - current_depth / 30.0).max(0.0);

            // Determine if this should be a sacred chamber
            let is_sacred = section_type == CaveSectionType::Chamber
                && current_depth > max_depth * 0.5
                && noise_util::hash(section_seed + 600) > 0.85;

            let final_type = if is_sacred { CaveSectionType::SacredChamber } else { section_type };

            // Create section
            let section = CaveSection {
                section_type: final_type,
                center: new_pos,
                radius: section_radius,
                length: section_length,
                direction: current_dir,
                connections: vec![], // Will be updated
                has_water,
                light_level,
            };
            sections.push(section);

            // Generate features for this section
            let section_features = self.generate_section_features(
                &sections[section_index as usize],
                section_index,
                section_seed
            );
            features.extend(section_features);

            // Handle branching at junctions
            if final_type == CaveSectionType::Junction {
                // Create 1-2 branches
                let branch_count = 1 + (noise_util::hash(section_seed + 700) * 2.0) as usize;
                for i in 0..branch_count {
                    let branch_dir = Vec3::new(
                        current_dir.x + (i as f32 - 0.5) * 1.2,
                        current_dir.y,
                        current_dir.z + (i as f32 - 0.5) * 0.8
                    ).normalize();
                    generation_stack.push((
                        new_pos,
                        branch_dir,
                        current_depth,
                        remaining_length * 0.4,
                        section_index
                    ));
                }
            }

            if final_type == CaveSectionType::Chamber {
                num_chambers += 1;
            }

            // Check for dead end
            if final_type == CaveSectionType::DeadEnd || current_depth > max_depth {
                // Try to continue from a branch
                if let Some((pos, dir, depth, len, _parent)) = generation_stack.pop() {
                    current_pos = pos;
                    current_dir = dir;
                    current_depth = depth;
                    remaining_length = len;
                } else {
                    break;
                }
            } else {
                current_pos = new_pos;
                remaining_length -= move_dist;
            }

            section_index += 1;

            // Safety limit
            if section_index > 100 {
                break;
            }
        }

        CaveSystem {
            entrance_pos: entrance,
            seed: combined_seed,
            total_length,
            max_depth,
            num_chambers,
            sections,
            features,
        }
    }

    /// Generate features (stalactites, bones, artifacts, etc.) for a section
    fn generate_section_features(
        &self,
        section: &CaveSection,
        section_index: u32,
        seed: u32
    ) -> Vec<CaveFeatureInstance> {
        let mut features = Vec::new();

        // Number of features based on section size
        let base_count = match section.section_type {
            CaveSectionType::Chamber | CaveSectionType::SacredChamber => 8 + (section.radius * 0.5) as usize,
            CaveSectionType::Passage => 2 + (section.length * 0.1) as usize,
            CaveSectionType::Pool => 4,
            CaveSectionType::Junction => 3,
            CaveSectionType::DeadEnd => 2,
            _ => 1,
        };

        for i in 0..base_count {
            let feature_seed = seed.wrapping_add(i as u32 * 100);

            // Position within section
            let offset_r = noise_util::hash(feature_seed) * section.radius * 0.8;
            let offset_angle = noise_util::hash(feature_seed + 1) * std::f32::consts::PI * 2.0;
            let offset_y = noise_util::hash(feature_seed + 2) * section.radius * 0.5 - section.radius * 0.25;

            let pos = section.center + Vec3::new(
                offset_angle.cos() * offset_r,
                offset_y,
                offset_angle.sin() * offset_r
            );

            // Determine feature type
            let feature_type = self.select_feature_type(section, feature_seed);

            if let Some((feature, rarity)) = feature_type {
                let rotation = noise_util::hash(feature_seed + 3) * std::f32::consts::PI * 2.0;
                let scale = 0.5 + noise_util::hash(feature_seed + 4) * 1.5;

                features.push(CaveFeatureInstance {
                    feature,
                    position: pos,
                    rotation,
                    scale,
                    section_index,
                    rarity,
                });
            }
        }

        // Sacred chambers get guaranteed artifacts
        if section.section_type == CaveSectionType::SacredChamber {
            features.extend(self.generate_sacred_features(section, section_index, seed));
        }

        features
    }

    /// Select a feature type based on section and randomness
    fn select_feature_type(&self, section: &CaveSection, seed: u32) -> Option<(CaveFeature, ArtifactRarity)> {
        let roll = noise_util::hash(seed);
        let type_roll = noise_util::hash(seed + 10);

        // Formation features (most common)
        if roll < 0.4 {
            let formation = if type_roll < 0.3 {
                CaveFeature::Stalactite
            } else if type_roll < 0.6 {
                CaveFeature::Stalagmite
            } else if type_roll < 0.75 {
                CaveFeature::Column
            } else if type_roll < 0.85 {
                CaveFeature::Flowstone
            } else {
                CaveFeature::CrystalCluster
            };
            return Some((formation, ArtifactRarity::Common));
        }

        // Bones
        if roll < 0.4 + self.config.bone_density {
            let bone_type = if type_roll < 0.5 {
                CaveFeature::Bones
            } else if type_roll < 0.8 {
                CaveFeature::AncientBones
            } else if section.section_type == CaveSectionType::Chamber {
                CaveFeature::MassGrave
            } else {
                CaveFeature::Bones
            };

            let rarity = if bone_type == CaveFeature::MassGrave {
                ArtifactRarity::Rare
            } else if bone_type == CaveFeature::AncientBones {
                ArtifactRarity::Uncommon
            } else {
                ArtifactRarity::Common
            };

            return Some((bone_type, rarity));
        }

        // Artifacts (rare)
        if roll < 0.4 + self.config.bone_density + self.config.artifact_density {
            let artifact = if type_roll < 0.3 {
                (CaveFeature::PotteryShards, ArtifactRarity::Common)
            } else if type_roll < 0.5 {
                (CaveFeature::StoneTools, ArtifactRarity::Common)
            } else if type_roll < 0.7 {
                (CaveFeature::Arrowheads, ArtifactRarity::Uncommon)
            } else if type_roll < 0.85 {
                (CaveFeature::AncientFirepit, ArtifactRarity::Uncommon)
            } else {
                (CaveFeature::CachedSupplies, ArtifactRarity::Rare)
            };
            return Some(artifact);
        }

        // Environmental features
        if roll < 0.7 {
            let env = if section.has_water && type_roll < 0.4 {
                CaveFeature::CavePool
            } else if section.light_level > 0.5 && type_roll < 0.6 {
                CaveFeature::CaveRoot
            } else if type_roll < 0.7 {
                CaveFeature::GlowingMoss
            } else if type_roll < 0.85 {
                CaveFeature::BatColony
            } else {
                CaveFeature::GuanoDeposit
            };
            return Some((env, ArtifactRarity::Common));
        }

        // Animal dens (uncommon)
        if roll < 0.75 {
            return Some((CaveFeature::AnimalDen, ArtifactRarity::Uncommon));
        }

        None
    }

    /// Generate special features for sacred chambers
    fn generate_sacred_features(
        &self,
        section: &CaveSection,
        section_index: u32,
        seed: u32
    ) -> Vec<CaveFeatureInstance> {
        let mut features = Vec::new();

        // Central altar
        features.push(CaveFeatureInstance {
            feature: CaveFeature::RitualAltar,
            position: section.center,
            rotation: 0.0,
            scale: 1.5,
            section_index,
            rarity: ArtifactRarity::Legendary,
        });

        // Cave art on walls
        let art_count = 2 + (noise_util::hash(seed) * 4.0) as usize;
        for i in 0..art_count {
            let angle = (i as f32 / art_count as f32) * std::f32::consts::PI * 2.0;
            let pos = section.center + Vec3::new(
                angle.cos() * section.radius * 0.9,
                0.0,
                angle.sin() * section.radius * 0.9
            );
            features.push(CaveFeatureInstance {
                feature: CaveFeature::CaveArt,
                position: pos,
                rotation: angle + std::f32::consts::PI, // Face inward
                scale: 1.0,
                section_index,
                rarity: ArtifactRarity::Rare,
            });
        }

        // Burial sites
        if noise_util::hash(seed + 100) > 0.5 {
            let burial_pos = section.center + Vec3::new(
                section.radius * 0.4,
                0.0,
                section.radius * 0.3
            );
            features.push(CaveFeatureInstance {
                feature: CaveFeature::BurialSite,
                position: burial_pos,
                rotation: 0.0,
                scale: 1.2,
                section_index,
                rarity: ArtifactRarity::Rare,
            });
        }

        features
    }

    /// Sample cave interior - returns cavity value at a 3D position
    /// Use this for actual cave mesh generation
    /// Returns > 0.5 if position is inside cave (hollow), < 0.5 if solid rock
    pub fn sample_cave_3d(&self, pos: Vec3, entrance: Vec3, cave_seed: u32) -> f32 {
        // Distance from entrance
        let dist_from_entrance = (pos - entrance).length();

        // Main tunnel noise
        let tunnel_scale = 0.08;
        let tunnel_noise = self.turbulence_3d(
            pos * tunnel_scale,
            4, 2.0, 0.5,
            cave_seed
        );

        // Worm-like tunnel using 3D noise
        let worm_x = fbm(Vec2::new(pos.y * 0.05, pos.z * 0.05), 3, 2.0, 0.5, cave_seed + 100);
        let worm_z = fbm(Vec2::new(pos.x * 0.05, pos.y * 0.05), 3, 2.0, 0.5, cave_seed + 200);

        let worm_center = entrance + Vec3::new(
            worm_x * dist_from_entrance * 0.3,
            -dist_from_entrance * 0.3, // Descend
            worm_z * dist_from_entrance * 0.3 - dist_from_entrance * 0.5 // Inward
        );

        // Distance to worm center
        let dist_to_worm = (pos - worm_center).length();

        // Cave radius varies with depth
        let base_radius = 3.0 + tunnel_noise * 5.0;
        let radius_variation = 1.0 + turbulence(Vec2::new(pos.x * 0.1, pos.z * 0.1), 2, 2.0, 0.5, cave_seed + 300) * 2.0;
        let cave_radius = base_radius * radius_variation;

        // Cavity field: 1.0 at center, 0.0 at edge
        let cavity = 1.0 - (dist_to_worm / cave_radius).clamp(0.0, 1.0);

        // Fade out far from entrance
        let entrance_fade = (1.0 - dist_from_entrance / 300.0).clamp(0.0, 1.0);

        cavity * entrance_fade
    }

    /// 3D turbulence noise (approximated with 2D slices)
    fn turbulence_3d(&self, pos: Vec3, octaves: u32, lacunarity: f32, persistence: f32, seed: u32) -> f32 {
        let xy = turbulence(Vec2::new(pos.x, pos.y), octaves, lacunarity, persistence, seed);
        let xz = turbulence(Vec2::new(pos.x, pos.z), octaves, lacunarity, persistence, seed + 1);
        let yz = turbulence(Vec2::new(pos.y, pos.z), octaves, lacunarity, persistence, seed + 2);
        (xy + xz + yz) / 3.0
    }
}

// ============================================================================
// PERLIN WORM GENERATION
// ============================================================================

/// Generate a Perlin worm tunnel using 3D noise gradient following
///
/// The worm follows the gradient of 3D Perlin noise, creating natural
/// winding tunnels that feel organic and cave-like.
pub fn generate_perlin_worm<F>(
    entrance: Vec3,
    entrance_direction: Vec3,
    config: &WormConfig,
    terrain_height_fn: &F,
) -> WormTunnel
where
    F: Fn(f32, f32) -> f32,
{
    let mut points = Vec::new();
    let mut branches = Vec::new();
    let mut current_pos = entrance;
    let mut current_dir = entrance_direction.normalize();
    let mut total_length = 0.0;
    let mut max_depth: f32 = 0.0;

    // Seed offsets for different noise channels
    let dir_seed = config.seed;
    let radius_seed = config.seed.wrapping_add(3000);
    let humidity_seed = config.seed.wrapping_add(4000);
    let branch_seed = config.seed.wrapping_add(5000);

    let max_steps = (config.max_tunnel_length / config.step_size) as usize + 1;

    for step in 0..max_steps {
        let surface_height = terrain_height_fn(current_pos.x, current_pos.z);
        let depth = (surface_height - current_pos.y).max(0.0);

        // Sample 3D noise for direction influence
        let noise_scale = config.direction_frequency;
        let sample_pos = current_pos * noise_scale;

        // Get noise gradient to guide worm direction
        let gradient = noise_gradient_3d(sample_pos, dir_seed);

        // Blend current direction with noise gradient
        let noise_influence = 0.4;
        let new_dir = (current_dir * (1.0 - noise_influence)
            + gradient * noise_influence
            + Vec3::new(0.0, -config.descent_bias, 0.0))
            .normalize();

        // Clamp vertical component to prevent going back up too steeply
        let clamped_dir = Vec3::new(
            new_dir.x,
            new_dir.y.min(0.1), // Mostly downward or horizontal
            new_dir.z,
        )
        .normalize();

        current_dir = clamped_dir;

        // Calculate radius at this point using separate noise channel
        let radius_sample = fbm_3d(
            current_pos * config.radius_frequency,
            2,
            2.0,
            0.5,
            radius_seed,
        );
        let radius = lerp(
            config.min_radius,
            config.max_radius,
            (radius_sample + 1.0) * 0.5,
        );

        // Calculate humidity for orb placement
        let humidity_sample = fbm_3d(
            Vec3::new(
                current_pos.x * config.humidity_frequency,
                current_pos.y * config.humidity_frequency * 0.5,
                current_pos.z * config.humidity_frequency,
            ),
            2,
            2.0,
            0.5,
            humidity_seed,
        );
        let humidity = ((humidity_sample + 1.0) * 0.5).clamp(0.0, 1.0);

        // Add point to worm
        points.push(WormPoint {
            position: current_pos,
            radius,
            tangent: current_dir,
            humidity,
            depth_from_surface: depth,
        });

        max_depth = max_depth.max(depth);

        // Check for branching (only after initial tunnel length)
        if total_length > config.min_tunnel_length * 0.3 {
            let branch_roll = noise_util::hash(branch_seed.wrapping_add(step as u32));
            if branch_roll < config.branch_probability {
                // Create branch with perpendicular-ish direction
                let up = Vec3::Y;
                let right = current_dir.cross(up).normalize();

                // Alternate branch direction
                let branch_angle = if step % 2 == 0 { 0.7 } else { -0.7 };
                let branch_dir = (current_dir + right * branch_angle).normalize();

                let branch_config = WormConfig {
                    seed: config.seed.wrapping_add(step as u32 * 1000),
                    max_tunnel_length: config.max_tunnel_length * 0.4,
                    branch_probability: config.branch_probability * 0.3, // Reduce branching in branches
                    ..config.clone()
                };

                let branch = generate_perlin_worm(
                    current_pos,
                    branch_dir,
                    &branch_config,
                    terrain_height_fn,
                );
                branches.push(branch);
            }
        }

        // Step forward
        current_pos += current_dir * config.step_size;
        total_length += config.step_size;

        // Termination conditions
        if total_length >= config.max_tunnel_length {
            break;
        }
        if depth > config.max_tunnel_length * 0.5 {
            break; // Too deep
        }
        if radius < config.min_radius * 0.5 {
            break; // Tunnel pinched off
        }
        // Don't go above surface
        if current_pos.y > surface_height - 2.0 {
            current_dir.y = -0.3; // Force downward
        }
    }

    WormTunnel {
        seed: config.seed,
        entrance,
        points,
        branches,
        total_length,
        max_depth,
    }
}

/// Convert a WormTunnel to a CaveSystem for compatibility with existing code
pub fn worm_to_cave_system(worm: &WormTunnel, config: &CaveGenConfig) -> CaveSystem {
    let mut sections = Vec::new();
    let mut features = Vec::new();

    // Create entrance section
    if let Some(first) = worm.points.first() {
        sections.push(CaveSection {
            section_type: CaveSectionType::Entrance,
            center: first.position,
            radius: first.radius,
            length: 0.0,
            direction: first.tangent,
            connections: if worm.points.len() > 1 { vec![1] } else { vec![] },
            has_water: first.humidity > 0.7,
            light_level: 1.0,
        });
    }

    // Convert worm points to sections (sample every few points)
    let sample_interval = 5.max(worm.points.len() / 20);
    for (i, point) in worm.points.iter().enumerate().skip(1) {
        if i % sample_interval != 0 && i != worm.points.len() - 1 {
            continue;
        }

        // Determine section type based on radius and depth
        let section_type = if point.radius > config.passage_width * 2.5 {
            if point.humidity > 0.8 && point.depth_from_surface > 20.0 {
                CaveSectionType::SacredChamber
            } else {
                CaveSectionType::Chamber
            }
        } else if point.depth_from_surface > worm.max_depth * 0.8 {
            CaveSectionType::DeadEnd
        } else {
            CaveSectionType::Passage
        };

        let light_level = (1.0 - point.depth_from_surface / 30.0).max(0.0);

        sections.push(CaveSection {
            section_type,
            center: point.position,
            radius: point.radius,
            length: config.passage_width,
            direction: point.tangent,
            connections: vec![],
            has_water: point.humidity > 0.7,
            light_level,
        });

        // Generate features for high-humidity areas
        if point.humidity > 0.6 && point.depth_from_surface > 10.0 {
            features.push(CaveFeatureInstance {
                feature: CaveFeature::GlowingMoss,
                position: point.position,
                rotation: 0.0,
                scale: 1.0 + point.humidity * 0.5,
                section_index: sections.len() as u32 - 1,
                rarity: ArtifactRarity::Common,
            });
        }
    }

    // Count chambers
    let num_chambers = sections
        .iter()
        .filter(|s| {
            matches!(
                s.section_type,
                CaveSectionType::Chamber | CaveSectionType::SacredChamber
            )
        })
        .count() as u32;

    CaveSystem {
        entrance_pos: worm.entrance,
        seed: worm.seed,
        total_length: worm.total_length,
        max_depth: worm.max_depth,
        num_chambers,
        sections,
        features,
    }
}

// ============================================================================
// BIOLUMINESCENT ORB GENERATION
// ============================================================================

/// Generate bioluminescent orbs for a Perlin worm tunnel
///
/// Orbs are placed on cave walls and ceilings in humid, deep areas
pub fn generate_bio_orbs(worm: &WormTunnel, density: f32) -> Vec<BioOrb> {
    let mut orbs = Vec::new();
    let mut rng_seed = worm.seed.wrapping_add(10000);

    generate_orbs_recursive(worm, density, &mut orbs, &mut rng_seed);

    orbs
}

fn generate_orbs_recursive(worm: &WormTunnel, density: f32, orbs: &mut Vec<BioOrb>, rng_seed: &mut u32) {
    for (_i, point) in worm.points.iter().enumerate() {
        // Only spawn in humid and deep areas
        if point.humidity < 0.4 || point.depth_from_surface < 8.0 {
            continue;
        }

        // Spawn probability based on humidity and depth
        let spawn_chance = point.humidity * 0.6 + (point.depth_from_surface / 50.0).min(0.4);

        let roll = noise_util::hash(*rng_seed);
        *rng_seed = rng_seed.wrapping_add(1);

        if roll > spawn_chance * density {
            continue;
        }

        // Position on wall/ceiling (random angle around tunnel axis)
        let angle = noise_util::hash(*rng_seed) * std::f32::consts::TAU;
        *rng_seed = rng_seed.wrapping_add(1);

        // Prefer ceiling and upper walls (adjust angle to favor top)
        let adjusted_angle = angle * 0.6 + std::f32::consts::PI * 0.2;

        // Calculate position on tunnel wall
        let tangent = point.tangent;
        let up = Vec3::Y;
        let right = tangent.cross(up);
        let right = if right.length() > 0.001 {
            right.normalize()
        } else {
            Vec3::X
        };
        let local_up = right.cross(tangent).normalize();

        let offset =
            (right * adjusted_angle.cos() + local_up * adjusted_angle.sin()) * (point.radius - 0.3);
        let position = point.position + offset;
        let normal = -offset.normalize(); // Points into tunnel

        // Select orb type and color
        let type_roll = noise_util::hash(*rng_seed);
        *rng_seed = rng_seed.wrapping_add(1);

        let (orb_type, color) = if type_roll < 0.4 {
            // Blue-green fungi (most common)
            (BioOrbType::Fungi, [0.2, 0.8, 0.6])
        } else if type_roll < 0.7 {
            // Cyan moss
            (BioOrbType::Moss, [0.3, 0.9, 0.9])
        } else if type_roll < 0.9 {
            // Purple crystal
            (BioOrbType::Crystal, [0.6, 0.3, 0.9])
        } else {
            // Warm yellow pool (only on floor-ish areas)
            if adjusted_angle > std::f32::consts::PI * 0.8 {
                (BioOrbType::Pool, [0.9, 0.8, 0.3])
            } else {
                (BioOrbType::Moss, [0.3, 0.9, 0.9])
            }
        };

        // Deeper = more intense (darker surroundings need brighter orbs)
        let depth_factor = (point.depth_from_surface / 40.0).min(1.0);
        let intensity = 0.5 + depth_factor * 1.0 + noise_util::hash(*rng_seed) * 0.5;
        *rng_seed = rng_seed.wrapping_add(1);

        let pulse_phase = noise_util::hash(*rng_seed) * std::f32::consts::TAU;
        *rng_seed = rng_seed.wrapping_add(1);

        let pulse_speed = 0.3 + noise_util::hash(*rng_seed) * 0.5;
        *rng_seed = rng_seed.wrapping_add(1);

        let cluster_size = 0.5 + noise_util::hash(*rng_seed) * 1.5;
        *rng_seed = rng_seed.wrapping_add(1);

        orbs.push(BioOrb {
            position,
            normal,
            color,
            intensity,
            pulse_phase,
            pulse_speed,
            cluster_size,
            orb_type,
        });

        // Sometimes spawn a cluster of smaller orbs nearby
        if noise_util::hash(*rng_seed) < 0.3 {
            *rng_seed = rng_seed.wrapping_add(1);
            let cluster_count = 2 + (noise_util::hash(*rng_seed) * 3.0) as usize;
            *rng_seed = rng_seed.wrapping_add(1);

            for _ in 0..cluster_count {
                let offset_dist = 0.3 + noise_util::hash(*rng_seed) * 0.5;
                *rng_seed = rng_seed.wrapping_add(1);
                let offset_angle = noise_util::hash(*rng_seed) * std::f32::consts::TAU;
                *rng_seed = rng_seed.wrapping_add(1);

                let cluster_offset = Vec3::new(
                    offset_angle.cos() * offset_dist,
                    (noise_util::hash(*rng_seed) - 0.5) * offset_dist,
                    offset_angle.sin() * offset_dist,
                );
                *rng_seed = rng_seed.wrapping_add(1);

                orbs.push(BioOrb {
                    position: position + cluster_offset,
                    normal,
                    color,
                    intensity: intensity * 0.6,
                    pulse_phase: pulse_phase + noise_util::hash(*rng_seed) * 0.5,
                    pulse_speed,
                    cluster_size: cluster_size * 0.4,
                    orb_type,
                });
                *rng_seed = rng_seed.wrapping_add(1);
            }
        }
    }

    // Process branches
    for branch in &worm.branches {
        generate_orbs_recursive(branch, density * 0.8, orbs, rng_seed);
    }
}

/// Sample the worm tunnel as a Signed Distance Field
/// Returns negative values inside the cave, positive outside
pub fn sample_worm_sdf(pos: Vec3, worm: &WormTunnel) -> f32 {
    let mut min_dist = f32::MAX;

    // Check distance to each worm segment
    for i in 0..worm.points.len().saturating_sub(1) {
        let p0 = &worm.points[i];
        let p1 = &worm.points[i + 1];

        // Capsule SDF: distance to line segment minus interpolated radius
        let segment = p1.position - p0.position;
        let seg_len_sq = segment.length_squared();

        let t = if seg_len_sq > 0.0001 {
            ((pos - p0.position).dot(segment) / seg_len_sq).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let closest = p0.position + segment * t;
        let radius = lerp(p0.radius, p1.radius, t);
        let dist = (pos - closest).length() - radius;

        min_dist = min_dist.min(dist);
    }

    // Handle single-point case
    if worm.points.len() == 1 {
        let p = &worm.points[0];
        min_dist = (pos - p.position).length() - p.radius;
    }

    // Recursively check branches (smooth union)
    for branch in &worm.branches {
        let branch_dist = sample_worm_sdf(pos, branch);
        // Smooth minimum for organic blending at junctions
        let k = 2.0; // Smoothing factor
        let h = (0.5 + 0.5 * (branch_dist - min_dist) / k).clamp(0.0, 1.0);
        min_dist = lerp(branch_dist, min_dist, h) - k * h * (1.0 - h);
    }

    min_dist
}

/// Get the bounds of a worm tunnel (for chunk culling)
pub fn get_worm_bounds(worm: &WormTunnel) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    fn update_bounds(worm: &WormTunnel, min: &mut Vec3, max: &mut Vec3) {
        for point in &worm.points {
            let r = point.radius;
            *min = min.min(point.position - Vec3::splat(r));
            *max = max.max(point.position + Vec3::splat(r));
        }
        for branch in &worm.branches {
            update_bounds(branch, min, max);
        }
    }

    update_bounds(worm, &mut min, &mut max);

    (min, max)
}

// ============================================================================
// WORM CAVE BONE/ARTIFACT/FOSSIL GENERATION
// ============================================================================

/// Generate bones, fossils, and artifacts scattered throughout a Perlin worm cave
pub fn generate_worm_cave_items(worm: &WormTunnel, config: &CaveGenConfig) -> (Vec<BoneInstance>, Vec<ArtifactInstance>) {
    let mut bones = Vec::new();
    let mut artifacts = Vec::new();
    let mut rng_seed = worm.seed.wrapping_add(20000);

    generate_items_recursive(worm, config, &mut bones, &mut artifacts, &mut rng_seed, 1.0);

    (bones, artifacts)
}

fn generate_items_recursive(
    worm: &WormTunnel,
    config: &CaveGenConfig,
    bones: &mut Vec<BoneInstance>,
    artifacts: &mut Vec<ArtifactInstance>,
    rng_seed: &mut u32,
    density_mult: f32,
) {
    // Sample points for item placement (not every point, creates clumps)
    let sample_interval = 8.max(worm.points.len() / 30);

    for (i, point) in worm.points.iter().enumerate() {
        // Only check at intervals or at large chambers
        let is_chamber = point.radius > config.passage_width * 2.0;
        if i % sample_interval != 0 && !is_chamber {
            continue;
        }

        // More items deeper in the cave and in larger areas
        let depth_factor = (point.depth_from_surface / 30.0).min(1.0);
        let size_factor = (point.radius / 10.0).min(1.0);
        let base_density = (config.bone_density + config.artifact_density) * density_mult;

        // Bones are more common than artifacts
        let bone_chance = base_density * 1.5 * (0.5 + depth_factor * 0.5) * size_factor;
        let artifact_chance = base_density * 0.5 * depth_factor * size_factor;

        // Roll for bones
        let bone_roll = noise_util::hash(*rng_seed);
        *rng_seed = rng_seed.wrapping_add(1);

        if bone_roll < bone_chance {
            let num_bones = if is_chamber {
                2 + (noise_util::hash(*rng_seed) * 4.0) as usize
            } else {
                1 + (noise_util::hash(*rng_seed) * 2.0) as usize
            };
            *rng_seed = rng_seed.wrapping_add(1);

            for _ in 0..num_bones {
                let bone = generate_single_bone(point, *rng_seed, depth_factor);
                *rng_seed = rng_seed.wrapping_add(10);
                bones.push(bone);
            }
        }

        // Roll for artifacts (rarer, more valuable)
        let artifact_roll = noise_util::hash(*rng_seed);
        *rng_seed = rng_seed.wrapping_add(1);

        if artifact_roll < artifact_chance {
            let artifact = generate_single_artifact(point, *rng_seed, depth_factor);
            *rng_seed = rng_seed.wrapping_add(10);
            artifacts.push(artifact);

            // Deep sacred chambers can have multiple artifacts
            if is_chamber && depth_factor > 0.6 && noise_util::hash(*rng_seed) < 0.3 {
                *rng_seed = rng_seed.wrapping_add(1);
                let extra_count = 1 + (noise_util::hash(*rng_seed) * 2.0) as usize;
                *rng_seed = rng_seed.wrapping_add(1);

                for _ in 0..extra_count {
                    let extra = generate_single_artifact(point, *rng_seed, depth_factor);
                    *rng_seed = rng_seed.wrapping_add(10);
                    artifacts.push(extra);
                }
            }
        }
    }

    // Process branches with reduced density
    for branch in &worm.branches {
        generate_items_recursive(branch, config, bones, artifacts, rng_seed, density_mult * 0.7);
    }
}

fn generate_single_bone(point: &WormPoint, seed: u32, depth_factor: f32) -> BoneInstance {
    // Position on cave floor (bottom of tunnel)
    let floor_offset = -point.radius * 0.9;
    let scatter_r = noise_util::hash(seed) * point.radius * 0.6;
    let scatter_angle = noise_util::hash(seed + 1) * std::f32::consts::TAU;

    let pos = point.position + Vec3::new(
        scatter_angle.cos() * scatter_r,
        floor_offset + noise_util::hash(seed + 2) * 0.3,
        scatter_angle.sin() * scatter_r,
    );

    // Select bone type - deeper = more ancient/prehistoric
    let type_roll = noise_util::hash(seed + 3);
    let bone_type = if depth_factor > 0.7 && type_roll < 0.3 {
        // Deep caves have prehistoric remains
        let prehistoric_roll = noise_util::hash(seed + 4);
        if prehistoric_roll < 0.3 {
            BoneType::MastodonBone
        } else if prehistoric_roll < 0.5 {
            BoneType::GiantSlothClaw
        } else if prehistoric_roll < 0.7 {
            BoneType::SabertoothSkull
        } else {
            BoneType::AncientTurtleShell
        }
    } else if type_roll < 0.4 {
        // Human remains
        let human_roll = noise_util::hash(seed + 5);
        if human_roll < 0.2 {
            BoneType::HumanSkull
        } else if human_roll < 0.4 {
            BoneType::HumanRibcage
        } else if human_roll < 0.6 {
            BoneType::HumanSpine
        } else if human_roll < 0.8 {
            BoneType::HumanLimb
        } else {
            BoneType::ScatteredHumanBones
        }
    } else {
        // Animal remains (most common)
        let animal_roll = noise_util::hash(seed + 6);
        if animal_roll < 0.3 {
            BoneType::DeerSkull
        } else if animal_roll < 0.5 {
            BoneType::BearSkull
        } else if animal_roll < 0.75 {
            BoneType::SmallAnimalSkeleton
        } else {
            BoneType::LargeAnimalRibcage
        }
    };

    // Deeper bones are more fossilized
    let age_factor = depth_factor * 0.7 + noise_util::hash(seed + 7) * 0.3;

    BoneInstance {
        bone_type,
        position: pos,
        rotation: Vec3::new(
            noise_util::hash(seed + 8) * std::f32::consts::PI * 0.3,
            noise_util::hash(seed + 9) * std::f32::consts::TAU,
            noise_util::hash(seed + 10) * std::f32::consts::PI * 0.3,
        ),
        scale: 0.6 + noise_util::hash(seed + 11) * 0.8,
        age_factor,
        completeness: 0.2 + noise_util::hash(seed + 12) * 0.8,
    }
}

fn generate_single_artifact(point: &WormPoint, seed: u32, depth_factor: f32) -> ArtifactInstance {
    // Position near walls or on floor
    let wall_or_floor = noise_util::hash(seed) < 0.7; // 70% on floor
    let pos = if wall_or_floor {
        let floor_offset = -point.radius * 0.85;
        let scatter_r = noise_util::hash(seed + 1) * point.radius * 0.5;
        let scatter_angle = noise_util::hash(seed + 2) * std::f32::consts::TAU;
        point.position + Vec3::new(
            scatter_angle.cos() * scatter_r,
            floor_offset,
            scatter_angle.sin() * scatter_r,
        )
    } else {
        // Against wall (cached supplies, hidden)
        let wall_angle = noise_util::hash(seed + 1) * std::f32::consts::TAU;
        point.position + Vec3::new(
            wall_angle.cos() * point.radius * 0.8,
            -point.radius * 0.7,
            wall_angle.sin() * point.radius * 0.8,
        )
    };

    // Deeper = rarer artifacts
    let rarity_roll = noise_util::hash(seed + 3);
    let rarity = if depth_factor > 0.8 && rarity_roll > 0.9 {
        ArtifactRarity::Legendary
    } else if depth_factor > 0.5 && rarity_roll > 0.8 {
        ArtifactRarity::Rare
    } else if depth_factor > 0.3 && rarity_roll > 0.6 {
        ArtifactRarity::Uncommon
    } else {
        ArtifactRarity::Common
    };

    let artifact_type = select_artifact_by_rarity(rarity, seed + 4);

    // Cultural period based on depth (deeper = older)
    let cultural_period = (depth_factor * 5.0) as u32;

    ArtifactInstance {
        artifact_type,
        position: pos,
        rotation: Vec3::new(
            noise_util::hash(seed + 5) * std::f32::consts::PI * 0.2,
            noise_util::hash(seed + 6) * std::f32::consts::TAU,
            noise_util::hash(seed + 7) * std::f32::consts::PI * 0.2,
        ),
        scale: 0.7 + noise_util::hash(seed + 8) * 0.5,
        rarity,
        condition: 0.1 + noise_util::hash(seed + 9) * 0.9,
        cultural_period,
    }
}

// ============================================================================
// BONE AND ARTIFACT GENERATION
// ============================================================================

/// Types of bones that can spawn
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoneType {
    // Human remains
    HumanSkull,
    HumanRibcage,
    HumanLimb,
    HumanSpine,
    ScatteredHumanBones,

    // Animal remains
    DeerSkull,
    BearSkull,
    SmallAnimalSkeleton,
    LargeAnimalRibcage,

    // Ancient/Prehistoric
    MastodonBone,
    GiantSlothClaw,
    SabertoothSkull,
    AncientTurtleShell,
}

/// Archaeological artifact types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactType {
    // Stone Age
    FlintKnife,
    StoneAxe,
    Hammerstone,
    ScrapingTool,
    FlintCore,

    // Projectiles
    SpearPoint,
    AtlatlWeight,
    ArrowheadObsidian,
    ArrowheadFlint,
    ArrowheadBone,

    // Pottery
    PotteryShard,
    ClayVessel,
    CeramicBowl,
    StorageJar,

    // Ornamental
    ShellNecklace,
    BoneBeads,
    CarvedPendant,
    PaintedShell,

    // Ceremonial
    CeremonialPipe,
    EffigyFigurine,
    MedicineBundle,
    SacredBundle,

    // Tools/Utility
    BoneNeedle,
    AntlerTool,
    FishHook,
    WeavingWeight,
    GroundstoneAxe,
}

/// Instance of a bone spawn
#[derive(Debug, Clone)]
pub struct BoneInstance {
    pub bone_type: BoneType,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: f32,
    pub age_factor: f32,       // 0.0 = fresh, 1.0 = ancient/fossilized
    pub completeness: f32,     // 0.0 = fragments, 1.0 = complete
}

/// Instance of an artifact spawn
#[derive(Debug, Clone)]
pub struct ArtifactInstance {
    pub artifact_type: ArtifactType,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: f32,
    pub rarity: ArtifactRarity,
    pub condition: f32,        // 0.0 = heavily damaged, 1.0 = pristine
    pub cultural_period: u32,  // Rough age indicator
}

/// Generate bones for a cave section
pub fn generate_bones_for_section(
    section: &CaveSection,
    seed: u32,
    density: f32,
) -> Vec<BoneInstance> {
    let mut bones = Vec::new();

    // More bones in certain section types
    let type_multiplier = match section.section_type {
        CaveSectionType::SacredChamber => 3.0,
        CaveSectionType::Chamber => 1.5,
        CaveSectionType::DeadEnd => 2.0,
        CaveSectionType::Passage => 0.5,
        _ => 1.0,
    };

    let count = (section.radius * density * type_multiplier) as usize;

    for i in 0..count {
        let bone_seed = seed.wrapping_add(i as u32 * 100);

        // Position within section
        let r = noise_util::hash(bone_seed) * section.radius * 0.85;
        let angle = noise_util::hash(bone_seed + 1) * std::f32::consts::PI * 2.0;
        let pos = section.center + Vec3::new(
            angle.cos() * r,
            -section.radius * 0.4 + noise_util::hash(bone_seed + 2) * 0.5, // On floor
            angle.sin() * r
        );

        // Select bone type
        let type_roll = noise_util::hash(bone_seed + 3);
        let bone_type = if type_roll < 0.3 {
            // Human bones (less common)
            let human_roll = noise_util::hash(bone_seed + 4);
            if human_roll < 0.2 {
                BoneType::HumanSkull
            } else if human_roll < 0.4 {
                BoneType::HumanRibcage
            } else if human_roll < 0.6 {
                BoneType::HumanSpine
            } else if human_roll < 0.8 {
                BoneType::HumanLimb
            } else {
                BoneType::ScatteredHumanBones
            }
        } else if type_roll < 0.7 {
            // Animal bones (common)
            let animal_roll = noise_util::hash(bone_seed + 5);
            if animal_roll < 0.3 {
                BoneType::DeerSkull
            } else if animal_roll < 0.5 {
                BoneType::BearSkull
            } else if animal_roll < 0.7 {
                BoneType::SmallAnimalSkeleton
            } else {
                BoneType::LargeAnimalRibcage
            }
        } else {
            // Ancient/prehistoric (rare)
            let ancient_roll = noise_util::hash(bone_seed + 6);
            if ancient_roll < 0.3 {
                BoneType::MastodonBone
            } else if ancient_roll < 0.5 {
                BoneType::GiantSlothClaw
            } else if ancient_roll < 0.7 {
                BoneType::SabertoothSkull
            } else {
                BoneType::AncientTurtleShell
            }
        };

        bones.push(BoneInstance {
            bone_type,
            position: pos,
            rotation: Vec3::new(
                noise_util::hash(bone_seed + 10) * std::f32::consts::PI * 0.3,
                noise_util::hash(bone_seed + 11) * std::f32::consts::PI * 2.0,
                noise_util::hash(bone_seed + 12) * std::f32::consts::PI * 0.3,
            ),
            scale: 0.7 + noise_util::hash(bone_seed + 13) * 0.6,
            age_factor: noise_util::hash(bone_seed + 14),
            completeness: 0.3 + noise_util::hash(bone_seed + 15) * 0.7,
        });
    }

    bones
}

/// Generate artifacts for a cave section
pub fn generate_artifacts_for_section(
    section: &CaveSection,
    seed: u32,
    density: f32,
) -> Vec<ArtifactInstance> {
    let mut artifacts = Vec::new();

    // More artifacts in sacred chambers
    let type_multiplier = match section.section_type {
        CaveSectionType::SacredChamber => 5.0,
        CaveSectionType::Chamber => 1.5,
        CaveSectionType::DeadEnd => 1.2, // Hidden caches
        _ => 0.3,
    };

    let count = (section.radius * density * type_multiplier * 0.3) as usize;

    for i in 0..count {
        let artifact_seed = seed.wrapping_add(i as u32 * 200);

        // Position
        let r = noise_util::hash(artifact_seed) * section.radius * 0.8;
        let angle = noise_util::hash(artifact_seed + 1) * std::f32::consts::PI * 2.0;
        let pos = section.center + Vec3::new(
            angle.cos() * r,
            -section.radius * 0.4 + noise_util::hash(artifact_seed + 2) * 0.3,
            angle.sin() * r
        );

        // Select artifact based on rarity distribution
        let rarity_roll = noise_util::hash(artifact_seed + 3);
        let rarity = if rarity_roll > 0.98 {
            ArtifactRarity::Legendary
        } else if rarity_roll > 0.9 {
            ArtifactRarity::Rare
        } else if rarity_roll > 0.7 {
            ArtifactRarity::Uncommon
        } else {
            ArtifactRarity::Common
        };

        let artifact_type = select_artifact_by_rarity(rarity, artifact_seed);

        artifacts.push(ArtifactInstance {
            artifact_type,
            position: pos,
            rotation: Vec3::new(
                noise_util::hash(artifact_seed + 10) * std::f32::consts::PI * 0.2,
                noise_util::hash(artifact_seed + 11) * std::f32::consts::PI * 2.0,
                noise_util::hash(artifact_seed + 12) * std::f32::consts::PI * 0.2,
            ),
            scale: 0.8 + noise_util::hash(artifact_seed + 13) * 0.4,
            rarity,
            condition: 0.2 + noise_util::hash(artifact_seed + 14) * 0.8,
            cultural_period: (noise_util::hash(artifact_seed + 15) * 5.0) as u32,
        });
    }

    artifacts
}

/// Select artifact type based on rarity
fn select_artifact_by_rarity(rarity: ArtifactRarity, seed: u32) -> ArtifactType {
    let roll = noise_util::hash(seed);

    match rarity {
        ArtifactRarity::Common => {
            if roll < 0.3 { ArtifactType::PotteryShard }
            else if roll < 0.5 { ArtifactType::FlintCore }
            else if roll < 0.7 { ArtifactType::Hammerstone }
            else if roll < 0.85 { ArtifactType::ArrowheadFlint }
            else { ArtifactType::ScrapingTool }
        }
        ArtifactRarity::Uncommon => {
            if roll < 0.2 { ArtifactType::StoneAxe }
            else if roll < 0.4 { ArtifactType::SpearPoint }
            else if roll < 0.55 { ArtifactType::BoneNeedle }
            else if roll < 0.7 { ArtifactType::FishHook }
            else if roll < 0.85 { ArtifactType::ShellNecklace }
            else { ArtifactType::ClayVessel }
        }
        ArtifactRarity::Rare => {
            if roll < 0.2 { ArtifactType::ArrowheadObsidian }
            else if roll < 0.4 { ArtifactType::CarvedPendant }
            else if roll < 0.55 { ArtifactType::CeramicBowl }
            else if roll < 0.7 { ArtifactType::AtlatlWeight }
            else if roll < 0.85 { ArtifactType::GroundstoneAxe }
            else { ArtifactType::CeremonialPipe }
        }
        ArtifactRarity::Legendary => {
            if roll < 0.25 { ArtifactType::EffigyFigurine }
            else if roll < 0.5 { ArtifactType::MedicineBundle }
            else if roll < 0.75 { ArtifactType::SacredBundle }
            else { ArtifactType::PaintedShell }
        }
    }
}

/// Utility: Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cave_generation() {
        let config = CaveGenConfig::default();
        let gen = CaveGenerator::new(config);

        let entrance = Vec3::new(100.0, 50.0, 100.0);
        let cave = gen.generate_cave_system(entrance, 12345);

        assert!(cave.sections.len() > 0);
        assert!(cave.features.len() > 0);
        assert_eq!(cave.sections[0].section_type, CaveSectionType::Entrance);
    }

    #[test]
    fn test_cave_determinism() {
        let config = CaveGenConfig::default();
        let gen = CaveGenerator::new(config);

        let entrance = Vec3::new(100.0, 50.0, 100.0);
        let cave1 = gen.generate_cave_system(entrance, 12345);
        let cave2 = gen.generate_cave_system(entrance, 12345);

        assert_eq!(cave1.sections.len(), cave2.sections.len());
        assert_eq!(cave1.features.len(), cave2.features.len());
    }

    #[test]
    fn test_bone_generation() {
        let section = CaveSection {
            section_type: CaveSectionType::Chamber,
            center: Vec3::ZERO,
            radius: 10.0,
            length: 0.0,
            direction: Vec3::NEG_Z,
            connections: vec![],
            has_water: false,
            light_level: 0.0,
        };

        let bones = generate_bones_for_section(&section, 12345, 1.0);
        assert!(bones.len() > 0);
    }
}
