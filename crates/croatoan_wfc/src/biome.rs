//! Comprehensive Biome System
//!
//! This module defines multiple biome types with unique terrain characteristics,
//! flora, fauna, and environmental features. All generation is seed-based and
//! uses Perlin noise for deterministic procedural generation.

use crate::noise_util::{fbm, ridged, turbulence};
use crate::trees;
use glam::Vec2;

/// Major biome categories covering square miles of terrain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    // Coastal biomes
    Ocean,
    Beach,
    SaltMarsh,
    CoastalScrub,

    // Lowland biomes
    Grassland,
    DeciduousForest,
    Wetland,
    River,

    // Highland biomes
    Foothills,
    RollingMountains,
    MountainPeak,
    AlpineMeadow,

    // Special biomes
    Cave,
    Waterfall,
    CanyonRiver,
}

/// Biome data at a specific world position
#[derive(Debug, Clone)]
pub struct BiomeData {
    pub primary_biome: BiomeType,
    pub secondary_biome: Option<BiomeType>,
    pub blend_factor: f32,  // 0.0 = fully primary, 1.0 = fully secondary
    pub height: f32,
    pub moisture: f32,      // 0.0 = arid, 1.0 = wet
    pub temperature: f32,   // 0.0 = cold, 1.0 = hot
    pub cave_depth: f32,    // 0.0 = surface, >0 = underground depth
    pub river_proximity: f32, // Distance to nearest river
    pub color: [f32; 3],
}

/// Configuration for world generation
#[derive(Debug, Clone)]
pub struct WorldGenConfig {
    pub seed: u32,
    pub world_size: f32,          // World size in meters (e.g., 16000 for ~10 square miles)
    pub sea_level: f32,
    pub mountain_scale: f32,
    pub river_frequency: f32,
    pub cave_frequency: f32,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            world_size: 16000.0,  // ~10 square miles
            sea_level: 0.0,
            mountain_scale: 150.0,
            river_frequency: 0.0008,
            cave_frequency: 0.003,
        }
    }
}

/// Main biome generator - the heart of world generation
pub struct BiomeGenerator {
    pub config: WorldGenConfig,
    // Cached noise seeds for different layers
    continental_seed: u32,
    moisture_seed: u32,
    temperature_seed: u32,
    mountain_seed: u32,
    river_seed: u32,
    cave_seed: u32,
    detail_seed: u32,
}

impl BiomeGenerator {
    pub fn new(config: WorldGenConfig) -> Self {
        let seed = config.seed;
        Self {
            config,
            continental_seed: seed.wrapping_add(1000),
            moisture_seed: seed.wrapping_add(2000),
            temperature_seed: seed.wrapping_add(3000),
            mountain_seed: seed.wrapping_add(4000),
            river_seed: seed.wrapping_add(5000),
            cave_seed: seed.wrapping_add(6000),
            detail_seed: seed.wrapping_add(7000),
        }
    }

    /// Get complete biome data at a world position
    pub fn get_biome_at(&self, x: f32, z: f32) -> BiomeData {
        let pos = Vec2::new(x, z);

        // Layer 1: Continental/base terrain shape (very low frequency)
        let continental = self.sample_continental(pos);

        // Layer 2: Moisture (affects biome selection)
        let moisture = self.sample_moisture(pos);

        // Layer 3: Temperature (latitude-based with noise)
        let temperature = self.sample_temperature(pos);

        // Layer 4: Mountain/elevation
        let mountain_factor = self.sample_mountains(pos);

        // Layer 5: River systems
        let (river_proximity, is_river, is_waterfall) = self.sample_rivers(pos);

        // Layer 6: Cave systems (3D, but we check surface entrance here)
        let cave_entrance = self.sample_cave_entrance(pos);

        // Determine base height before biome selection
        let base_height = self.calculate_base_height(continental, mountain_factor, moisture);

        // Determine primary biome
        let primary_biome = self.classify_biome(
            base_height, moisture, temperature,
            river_proximity, is_river, is_waterfall, cave_entrance
        );

        // Find secondary biome for blending
        let (secondary_biome, blend_factor) = self.find_blend_biome(
            pos, primary_biome, base_height, moisture, temperature
        );

        // Calculate final height with biome-specific modifications
        let height = self.calculate_final_height(
            pos, base_height, primary_biome, secondary_biome, blend_factor,
            is_river, is_waterfall, river_proximity
        );

        // Calculate color
        let color = self.calculate_biome_color(
            primary_biome, secondary_biome, blend_factor,
            height, moisture, temperature
        );

        BiomeData {
            primary_biome,
            secondary_biome,
            blend_factor,
            height,
            moisture,
            temperature,
            cave_depth: if cave_entrance > 0.7 { cave_entrance } else { 0.0 },
            river_proximity,
            color,
        }
    }

    /// Sample continental shelf / base terrain
    fn sample_continental(&self, pos: Vec2) -> f32 {
        // Very low frequency for continent-scale features
        let scale = 0.0003;
        let noise = fbm(pos * scale, 4, 2.0, 0.5, self.continental_seed);

        // Eastern sea gradient (positive X = ocean)
        let gradient = -pos.x * 0.00008;

        // Combine noise and gradient
        let combined = noise * 0.6 + gradient + 0.3;
        combined.clamp(0.0, 1.0)
    }

    /// Sample moisture levels
    fn sample_moisture(&self, pos: Vec2) -> f32 {
        let scale = 0.0008;
        let base = fbm(pos * scale, 3, 2.0, 0.5, self.moisture_seed);

        // Rivers increase local moisture
        let (river_dist, _, _) = self.sample_rivers(pos);
        let river_moisture = (1.0 - (river_dist / 100.0).min(1.0)) * 0.4;

        // Coast increases moisture
        let continental = self.sample_continental(pos);
        let coastal_moisture = if continental < 0.5 { (0.5 - continental) * 0.5 } else { 0.0 };

        ((base + 1.0) * 0.5 + river_moisture + coastal_moisture).clamp(0.0, 1.0)
    }

    /// Sample temperature (latitude + altitude based)
    fn sample_temperature(&self, pos: Vec2) -> f32 {
        let scale = 0.0005;
        let noise = fbm(pos * scale, 2, 2.0, 0.5, self.temperature_seed);

        // Z-axis acts as latitude (negative Z = colder)
        let latitude_temp = (pos.y / self.config.world_size + 0.5).clamp(0.0, 1.0);

        // Combine
        (latitude_temp * 0.7 + (noise + 1.0) * 0.15).clamp(0.0, 1.0)
    }

    /// Sample mountain/highland terrain
    fn sample_mountains(&self, pos: Vec2) -> f32 {
        // Mountain ranges use ridged noise for dramatic peaks
        let ridge_scale = 0.0006;
        let ridge_noise = ridged(pos * ridge_scale, 5, 2.2, 0.5, self.mountain_seed);

        // Rolling hills use FBM
        let hill_scale = 0.001;
        let hill_noise = fbm(pos * hill_scale, 4, 2.0, 0.5, self.mountain_seed + 100);

        // Western side has more mountains (negative X)
        let mountain_gradient = (-pos.x / self.config.world_size).clamp(0.0, 1.0) * 0.5;

        // Combine ridges and hills based on position
        let ridge_factor = ridge_noise * mountain_gradient;
        let hill_factor = ((hill_noise + 1.0) * 0.5) * 0.6;

        (ridge_factor + hill_factor).clamp(0.0, 1.0)
    }

    /// Sample river systems - returns (distance_to_river, is_river, is_waterfall)
    fn sample_rivers(&self, pos: Vec2) -> (f32, bool, bool) {
        // River path noise - creates winding paths
        let river_scale = self.config.river_frequency;

        // Multiple river "channels" at different frequencies
        let channel1 = self.river_channel(pos, river_scale, self.river_seed);
        let channel2 = self.river_channel(pos, river_scale * 0.5, self.river_seed + 500);
        let channel3 = self.river_channel(pos, river_scale * 2.0, self.river_seed + 1000);

        // Take the minimum distance to any channel
        let min_dist = channel1.min(channel2).min(channel3);

        // River width varies with terrain
        let river_width = 8.0 + fbm(pos * 0.01, 2, 2.0, 0.5, self.river_seed + 200).abs() * 12.0;

        let is_river = min_dist < river_width;

        // Waterfalls occur where rivers meet steep terrain changes
        let terrain_slope = self.calculate_slope(pos);
        let is_waterfall = is_river && terrain_slope > 0.4 && min_dist < river_width * 0.5;

        (min_dist, is_river, is_waterfall)
    }

    /// Calculate a single river channel distance
    fn river_channel(&self, pos: Vec2, scale: f32, seed: u32) -> f32 {
        // Use domain warping for more natural river paths
        let warp_amount = 200.0;
        let warp_x = fbm(pos * scale * 0.5, 3, 2.0, 0.5, seed + 10) * warp_amount;
        let warp_z = fbm(pos * scale * 0.5, 3, 2.0, 0.5, seed + 20) * warp_amount;
        let warped_pos = pos + Vec2::new(warp_x, warp_z);

        // River follows noise "valleys"
        let river_noise = fbm(warped_pos * scale, 4, 2.0, 0.5, seed);

        // Convert noise to distance-like value
        // Rivers form where noise is near 0
        river_noise.abs() * 500.0
    }

    /// Calculate terrain slope at a position
    fn calculate_slope(&self, pos: Vec2) -> f32 {
        let delta = 5.0;
        let h_center = self.sample_mountains(pos);
        let h_x = self.sample_mountains(pos + Vec2::new(delta, 0.0));
        let h_z = self.sample_mountains(pos + Vec2::new(0.0, delta));

        let dx = (h_x - h_center) / delta;
        let dz = (h_z - h_center) / delta;

        (dx * dx + dz * dz).sqrt()
    }

    /// Sample cave entrance locations
    fn sample_cave_entrance(&self, pos: Vec2) -> f32 {
        // Caves are more common in mountainous/hilly areas
        let mountain_factor = self.sample_mountains(pos);

        // Use high-frequency noise for cave entrance locations
        let cave_noise = turbulence(pos * self.config.cave_frequency, 4, 2.5, 0.5, self.cave_seed);

        // Caves need minimum elevation and rougher terrain
        if mountain_factor > 0.3 {
            cave_noise * mountain_factor
        } else {
            0.0
        }
    }

    /// Calculate base height from continental and mountain data
    fn calculate_base_height(&self, continental: f32, mountain_factor: f32, moisture: f32) -> f32 {
        // Sea level threshold
        if continental < 0.35 {
            // Ocean depth
            let depth_factor = continental / 0.35;
            lerp(-15.0, self.config.sea_level, depth_factor)
        } else if continental < 0.45 {
            // Coastal/beach zone
            let blend = (continental - 0.35) / 0.1;
            lerp(self.config.sea_level, 3.0, blend)
        } else {
            // Land - varies with mountain factor
            let land_blend = (continental - 0.45) / 0.55;
            let base_land = lerp(3.0, 30.0, land_blend);

            // Add mountain elevation
            let mountain_height = mountain_factor * self.config.mountain_scale;

            // Wetlands are lower
            let wetland_depression = if moisture > 0.7 { (moisture - 0.7) * 10.0 } else { 0.0 };

            base_land + mountain_height - wetland_depression
        }
    }

    /// Classify the biome type based on environmental factors
    fn classify_biome(
        &self,
        height: f32,
        moisture: f32,
        temperature: f32,
        _river_proximity: f32,
        is_river: bool,
        is_waterfall: bool,
        cave_entrance: f32,
    ) -> BiomeType {
        // Priority checks
        if is_waterfall {
            return BiomeType::Waterfall;
        }

        if is_river {
            if height > 50.0 {
                return BiomeType::CanyonRiver;
            }
            return BiomeType::River;
        }

        if cave_entrance > 0.7 {
            return BiomeType::Cave;
        }

        // Height-based primary classification
        if height < self.config.sea_level {
            return BiomeType::Ocean;
        }

        if height < 3.0 {
            // Coastal zone
            if moisture > 0.65 {
                return BiomeType::SaltMarsh;
            }
            return BiomeType::Beach;
        }

        if height < 15.0 {
            // Lowlands
            if moisture > 0.75 {
                return BiomeType::Wetland;
            }
            if moisture < 0.35 {
                return BiomeType::CoastalScrub;
            }
            if moisture > 0.5 {
                return BiomeType::DeciduousForest;
            }
            return BiomeType::Grassland;
        }

        if height < 50.0 {
            // Foothills
            if moisture > 0.6 {
                return BiomeType::DeciduousForest;
            }
            return BiomeType::Foothills;
        }

        if height < 100.0 {
            // Mountain slopes
            if temperature > 0.5 && moisture > 0.4 {
                return BiomeType::AlpineMeadow;
            }
            return BiomeType::RollingMountains;
        }

        // High peaks
        BiomeType::MountainPeak
    }

    /// Find secondary biome for blending at transitions
    fn find_blend_biome(
        &self,
        pos: Vec2,
        primary: BiomeType,
        _height: f32,
        _moisture: f32,
        _temperature: f32,
    ) -> (Option<BiomeType>, f32) {
        // Sample nearby points to find transition zones
        let sample_dist = 20.0;
        let samples = [
            Vec2::new(sample_dist, 0.0),
            Vec2::new(-sample_dist, 0.0),
            Vec2::new(0.0, sample_dist),
            Vec2::new(0.0, -sample_dist),
        ];

        let mut other_biome: Option<BiomeType> = None;
        let mut transition_count = 0;

        for offset in &samples {
            let sample_pos = pos + *offset;
            let sample_data = BiomeData {
                primary_biome: self.classify_biome(
                    self.calculate_base_height(
                        self.sample_continental(sample_pos),
                        self.sample_mountains(sample_pos),
                        self.sample_moisture(sample_pos),
                    ),
                    self.sample_moisture(sample_pos),
                    self.sample_temperature(sample_pos),
                    self.sample_rivers(sample_pos).0,
                    self.sample_rivers(sample_pos).1,
                    self.sample_rivers(sample_pos).2,
                    self.sample_cave_entrance(sample_pos),
                ),
                secondary_biome: None,
                blend_factor: 0.0,
                height: 0.0,
                moisture: 0.0,
                temperature: 0.0,
                cave_depth: 0.0,
                river_proximity: 0.0,
                color: [0.0; 3],
            };

            if sample_data.primary_biome != primary {
                other_biome = Some(sample_data.primary_biome);
                transition_count += 1;
            }
        }

        if transition_count > 0 {
            // Use noise for smooth blending
            let blend_noise = fbm(pos * 0.05, 2, 2.0, 0.5, self.detail_seed);
            let blend_factor = ((blend_noise + 1.0) * 0.25 + transition_count as f32 * 0.1).clamp(0.0, 0.5);
            (other_biome, blend_factor)
        } else {
            (None, 0.0)
        }
    }

    /// Calculate final height with all biome-specific modifiers
    fn calculate_final_height(
        &self,
        pos: Vec2,
        base_height: f32,
        primary: BiomeType,
        _secondary: Option<BiomeType>,
        _blend: f32,
        is_river: bool,
        is_waterfall: bool,
        river_proximity: f32,
    ) -> f32 {
        // Detail noise for micro-terrain
        let detail = fbm(pos * 0.05, 4, 2.0, 0.5, self.detail_seed);

        // Biome-specific height modifiers
        let height_mod = match primary {
            BiomeType::Ocean => {
                // Underwater terrain with sandbars
                let sandbar = if detail > 0.6 { 0.5 } else { 0.0 };
                detail * 0.3 + sandbar
            }
            BiomeType::Beach => {
                // Gentle dunes
                detail * 0.5
            }
            BiomeType::SaltMarsh => {
                // Very flat with subtle channels
                let channel = if detail.abs() < 0.2 { -0.3 } else { 0.0 };
                detail * 0.2 + channel
            }
            BiomeType::Grassland | BiomeType::CoastalScrub => {
                // Rolling terrain
                detail * 1.5
            }
            BiomeType::DeciduousForest => {
                // Moderate undulation
                detail * 2.0
            }
            BiomeType::Wetland => {
                // Very flat, occasional hummocks
                let hummock = if detail > 0.7 { 0.5 } else { 0.0 };
                detail * 0.3 + hummock
            }
            BiomeType::River => {
                // River bed depression
                -2.0 + detail * 0.5
            }
            BiomeType::Waterfall => {
                // Dramatic drop
                -5.0 + detail * 2.0
            }
            BiomeType::CanyonRiver => {
                // Deep canyon
                -8.0 + detail * 1.0
            }
            BiomeType::Foothills => {
                // More dramatic terrain
                detail * 4.0 + ridged(pos * 0.01, 3, 2.0, 0.5, self.mountain_seed + 300) * 5.0
            }
            BiomeType::RollingMountains => {
                // Large rolling peaks
                ridged(pos * 0.005, 4, 2.2, 0.5, self.mountain_seed) * 20.0 + detail * 5.0
            }
            BiomeType::MountainPeak => {
                // Sharp peaks with ridges
                ridged(pos * 0.003, 5, 2.5, 0.5, self.mountain_seed) * 40.0 + detail * 3.0
            }
            BiomeType::AlpineMeadow => {
                // Gentler mountain terrain
                detail * 3.0 + ridged(pos * 0.008, 3, 2.0, 0.5, self.mountain_seed + 400) * 8.0
            }
            BiomeType::Cave => {
                // Surface entrance dip
                -1.0 + detail * 0.5
            }
        };

        // River carving effect
        let river_carve = if river_proximity < 50.0 {
            let factor = 1.0 - (river_proximity / 50.0);
            factor * factor * -3.0  // Gradual descent toward river
        } else {
            0.0
        };

        base_height + height_mod + river_carve
    }

    /// Calculate biome color with blending
    fn calculate_biome_color(
        &self,
        primary: BiomeType,
        secondary: Option<BiomeType>,
        blend: f32,
        height: f32,
        moisture: f32,
        temperature: f32,
    ) -> [f32; 3] {
        let primary_color = self.get_biome_base_color(primary, height, moisture, temperature);

        if let Some(sec) = secondary {
            let secondary_color = self.get_biome_base_color(sec, height, moisture, temperature);
            lerp_color(primary_color, secondary_color, blend)
        } else {
            primary_color
        }
    }

    /// Get base color for a biome type
    fn get_biome_base_color(
        &self,
        biome: BiomeType,
        height: f32,
        moisture: f32,
        _temperature: f32,
    ) -> [f32; 3] {
        match biome {
            BiomeType::Ocean => {
                // Deep blue to turquoise based on depth
                let depth_factor = ((-height) / 15.0).clamp(0.0, 1.0);
                lerp_color([0.1, 0.6, 0.7], [0.02, 0.15, 0.3], depth_factor)
            }
            BiomeType::Beach => {
                // Sandy tan
                [0.82, 0.72, 0.55]
            }
            BiomeType::SaltMarsh => {
                // Muddy green-brown
                [0.45, 0.50, 0.35]
            }
            BiomeType::CoastalScrub => {
                // Dry olive green
                [0.55, 0.55, 0.40]
            }
            BiomeType::Grassland => {
                // Vibrant green, varies with moisture
                let green_intensity = 0.4 + moisture * 0.3;
                [0.35, green_intensity, 0.20]
            }
            BiomeType::DeciduousForest => {
                // Rich forest green
                [0.15, 0.40, 0.12]
            }
            BiomeType::Wetland => {
                // Dark murky green
                [0.25, 0.35, 0.25]
            }
            BiomeType::River => {
                // River blue
                [0.15, 0.45, 0.55]
            }
            BiomeType::Waterfall => {
                // White-blue water
                [0.70, 0.85, 0.95]
            }
            BiomeType::CanyonRiver => {
                // Dark blue-green
                [0.10, 0.35, 0.45]
            }
            BiomeType::Foothills => {
                // Earthy green-brown
                [0.45, 0.50, 0.30]
            }
            BiomeType::RollingMountains => {
                // Gray-green stone
                let gray_blend = ((height - 50.0) / 50.0).clamp(0.0, 1.0);
                lerp_color([0.40, 0.45, 0.35], [0.55, 0.55, 0.50], gray_blend)
            }
            BiomeType::MountainPeak => {
                // Snow-capped gray
                let snow_blend = ((height - 100.0) / 50.0).clamp(0.0, 1.0);
                lerp_color([0.50, 0.50, 0.48], [0.95, 0.95, 0.98], snow_blend)
            }
            BiomeType::AlpineMeadow => {
                // Bright alpine grass
                [0.50, 0.65, 0.35]
            }
            BiomeType::Cave => {
                // Dark rock
                [0.25, 0.23, 0.22]
            }
        }
    }
}

/// Utility: Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Utility: Color interpolation
fn lerp_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

// ============================================================================
// BIOME-SPECIFIC FLORA AND FAUNA DEFINITIONS
// ============================================================================

/// Flora types that can spawn in biomes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloraType {
    // Trees
    LiveOak,
    Cypress,
    Pine,
    Willow,
    Birch,
    Spruce,
    DeadTree,

    // Shrubs
    Palmetto,
    Waxmyrtle,
    Yaupon,
    Blueberry,
    Azalea,
    MountainLaurel,

    // Ground cover
    SawGrass,
    Cordgrass,
    SeaOats,
    Fern,
    Moss,
    Lichen,

    // Marsh plants
    Cattail,
    Bulrush,
    Pickerelweed,

    // Aquatic
    Seaweed,
    WaterLily,
}

/// Fauna types for different biomes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaunaType {
    // Large mammals
    BlackBear,
    WhitetailDeer,
    WildBoar,
    EasternCougar,

    // Medium mammals
    GrayFox,
    RedWolf,
    Bobcat,
    Raccoon,
    Opossum,

    // Small mammals
    EasternCottontail,
    GraySquirrel,
    Muskrat,
    Beaver,

    // Reptiles
    AmericanAlligator,
    TimberRattlesnake,
    Copperhead,
    Cottonmouth,
    BoxTurtle,

    // Birds
    WildTurkey,
    GreatBlueHeron,
    Osprey,
    BarredOwl,

    // Aquatic
    LargemouthBass,
    Catfish,
    BlueGill,
}

/// Get flora spawning weights for a biome
pub fn get_flora_weights(biome: BiomeType) -> Vec<(FloraType, f32)> {
    match biome {
        BiomeType::Ocean => vec![
            (FloraType::Seaweed, 0.8),
        ],
        BiomeType::Beach => vec![
            (FloraType::SeaOats, 0.6),
            (FloraType::Palmetto, 0.2),
        ],
        BiomeType::SaltMarsh => vec![
            (FloraType::Cordgrass, 0.9),
            (FloraType::SawGrass, 0.7),
            (FloraType::Cattail, 0.4),
            (FloraType::Bulrush, 0.3),
        ],
        BiomeType::CoastalScrub => vec![
            (FloraType::Palmetto, 0.8),
            (FloraType::Waxmyrtle, 0.6),
            (FloraType::Yaupon, 0.5),
            (FloraType::LiveOak, 0.3),
        ],
        BiomeType::Grassland => vec![
            (FloraType::SawGrass, 0.7),
            (FloraType::Blueberry, 0.3),
            (FloraType::Pine, 0.1),
        ],
        BiomeType::DeciduousForest => vec![
            (FloraType::LiveOak, 0.8),
            (FloraType::Pine, 0.6),
            (FloraType::Birch, 0.3),
            (FloraType::Fern, 0.7),
            (FloraType::Azalea, 0.4),
            (FloraType::Moss, 0.5),
        ],
        BiomeType::Wetland => vec![
            (FloraType::Cypress, 0.9),
            (FloraType::Willow, 0.6),
            (FloraType::Cattail, 0.8),
            (FloraType::Pickerelweed, 0.5),
            (FloraType::WaterLily, 0.4),
            (FloraType::Moss, 0.7),
        ],
        BiomeType::River | BiomeType::CanyonRiver => vec![
            (FloraType::Willow, 0.7),
            (FloraType::WaterLily, 0.5),
            (FloraType::Bulrush, 0.4),
        ],
        BiomeType::Waterfall => vec![
            (FloraType::Moss, 0.9),
            (FloraType::Fern, 0.8),
        ],
        BiomeType::Foothills => vec![
            (FloraType::Pine, 0.7),
            (FloraType::Birch, 0.5),
            (FloraType::MountainLaurel, 0.4),
            (FloraType::Blueberry, 0.3),
            (FloraType::Fern, 0.4),
        ],
        BiomeType::RollingMountains => vec![
            (FloraType::Spruce, 0.8),
            (FloraType::Pine, 0.6),
            (FloraType::MountainLaurel, 0.3),
            (FloraType::Lichen, 0.5),
        ],
        BiomeType::MountainPeak => vec![
            (FloraType::Lichen, 0.7),
            (FloraType::Moss, 0.3),
        ],
        BiomeType::AlpineMeadow => vec![
            (FloraType::SawGrass, 0.6),
            (FloraType::MountainLaurel, 0.4),
            (FloraType::Blueberry, 0.5),
        ],
        BiomeType::Cave => vec![
            (FloraType::Moss, 0.3),
            (FloraType::Lichen, 0.4),
        ],
    }
}

/// Get fauna spawning weights for a biome
pub fn get_fauna_weights(biome: BiomeType) -> Vec<(FaunaType, f32)> {
    match biome {
        BiomeType::Ocean => vec![
            (FaunaType::LargemouthBass, 0.5),
            (FaunaType::Catfish, 0.4),
        ],
        BiomeType::Beach => vec![
            (FaunaType::GrayFox, 0.1),
            (FaunaType::EasternCottontail, 0.2),
        ],
        BiomeType::SaltMarsh => vec![
            (FaunaType::AmericanAlligator, 0.6),
            (FaunaType::Cottonmouth, 0.5),
            (FaunaType::GreatBlueHeron, 0.4),
            (FaunaType::Muskrat, 0.5),
            (FaunaType::BlueGill, 0.6),
        ],
        BiomeType::CoastalScrub => vec![
            (FaunaType::WhitetailDeer, 0.3),
            (FaunaType::WildBoar, 0.2),
            (FaunaType::Bobcat, 0.1),
            (FaunaType::EasternCottontail, 0.5),
            (FaunaType::BoxTurtle, 0.3),
        ],
        BiomeType::Grassland => vec![
            (FaunaType::WhitetailDeer, 0.5),
            (FaunaType::WildTurkey, 0.4),
            (FaunaType::EasternCottontail, 0.6),
            (FaunaType::GrayFox, 0.2),
            (FaunaType::TimberRattlesnake, 0.2),
        ],
        BiomeType::DeciduousForest => vec![
            (FaunaType::BlackBear, 0.3),
            (FaunaType::WhitetailDeer, 0.6),
            (FaunaType::WildBoar, 0.4),
            (FaunaType::GraySquirrel, 0.7),
            (FaunaType::Raccoon, 0.4),
            (FaunaType::BarredOwl, 0.3),
            (FaunaType::Copperhead, 0.2),
        ],
        BiomeType::Wetland => vec![
            (FaunaType::AmericanAlligator, 0.7),
            (FaunaType::Cottonmouth, 0.6),
            (FaunaType::Beaver, 0.5),
            (FaunaType::Muskrat, 0.6),
            (FaunaType::GreatBlueHeron, 0.5),
            (FaunaType::Catfish, 0.7),
        ],
        BiomeType::River | BiomeType::CanyonRiver => vec![
            (FaunaType::Beaver, 0.6),
            (FaunaType::Osprey, 0.4),
            (FaunaType::LargemouthBass, 0.7),
            (FaunaType::Catfish, 0.6),
            (FaunaType::BlueGill, 0.5),
        ],
        BiomeType::Waterfall => vec![
            (FaunaType::Osprey, 0.3),
        ],
        BiomeType::Foothills => vec![
            (FaunaType::BlackBear, 0.4),
            (FaunaType::WhitetailDeer, 0.5),
            (FaunaType::EasternCougar, 0.2),
            (FaunaType::WildTurkey, 0.4),
            (FaunaType::TimberRattlesnake, 0.3),
        ],
        BiomeType::RollingMountains => vec![
            (FaunaType::BlackBear, 0.5),
            (FaunaType::EasternCougar, 0.3),
            (FaunaType::RedWolf, 0.2),
            (FaunaType::Bobcat, 0.3),
        ],
        BiomeType::MountainPeak => vec![
            (FaunaType::Bobcat, 0.2),
        ],
        BiomeType::AlpineMeadow => vec![
            (FaunaType::WhitetailDeer, 0.3),
            (FaunaType::GrayFox, 0.2),
            (FaunaType::EasternCottontail, 0.4),
        ],
        BiomeType::Cave => vec![
            // Cave-specific fauna handled by cave system
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_generator() {
        let config = WorldGenConfig::default();
        let gen = BiomeGenerator::new(config);

        // Test ocean area (positive X)
        let ocean_data = gen.get_biome_at(5000.0, 0.0);
        assert!(ocean_data.height < 0.0 || ocean_data.primary_biome == BiomeType::Beach);

        // Test inland area (negative X)
        let land_data = gen.get_biome_at(-3000.0, 0.0);
        assert!(land_data.height > 0.0);
    }

    #[test]
    fn test_biome_determinism() {
        let config = WorldGenConfig::default();
        let gen = BiomeGenerator::new(config.clone());

        let data1 = gen.get_biome_at(100.0, 200.0);
        let data2 = gen.get_biome_at(100.0, 200.0);

        assert_eq!(data1.height, data2.height);
        assert_eq!(data1.primary_biome, data2.primary_biome);
    }
}
