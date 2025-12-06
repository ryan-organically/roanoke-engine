//! River and Waterfall Generation System
//!
//! This module generates realistic river networks with:
//! - Perlin-based river path generation
//! - Tributary systems
//! - Waterfalls at elevation changes
//! - River deltas and estuaries
//! - Canyon formation in mountain rivers
//!
//! All generation is seed-based and deterministic.

use crate::noise_util::{self, fbm, turbulence};
use glam::{Vec2, Vec3};

// ============================================================================
// RIVER TYPES AND STRUCTURES
// ============================================================================

/// Types of river segments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiverSegmentType {
    /// Mountain headwaters - narrow, fast
    Headwater,
    /// Main river channel
    MainChannel,
    /// Tributary joining main river
    Tributary,
    /// Wide, slow lowland section
    Lowland,
    /// Waterfall/cascade
    Waterfall,
    /// Rapids section
    Rapids,
    /// Deep canyon section
    Canyon,
    /// River delta/estuary
    Delta,
    /// River mouth at ocean
    Mouth,
    /// River fork/split
    Fork,
}

/// A complete river system from source to mouth
#[derive(Debug, Clone)]
pub struct RiverSystem {
    pub seed: u32,
    pub source: Vec3,           // Headwaters location
    pub mouth: Vec3,            // Where it meets ocean/lake
    pub total_length: f32,
    pub segments: Vec<RiverSegment>,
    pub tributaries: Vec<RiverSystem>,
    pub waterfalls: Vec<WaterfallData>,
}

/// A segment of river
#[derive(Debug, Clone)]
pub struct RiverSegment {
    pub segment_type: RiverSegmentType,
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    pub depth: f32,
    pub flow_speed: f32,
    pub meander_amount: f32,    // How much the river curves
    pub control_points: Vec<Vec3>, // For smooth curves
}

/// Data for a waterfall
#[derive(Debug, Clone)]
pub struct WaterfallData {
    pub position: Vec3,
    pub width: f32,
    pub height: f32,           // Vertical drop
    pub flow_rate: f32,
    pub mist_radius: f32,
    pub pool_depth: f32,       // Plunge pool at bottom
    pub spray_particles: bool,
}

/// Configuration for river generation
#[derive(Debug, Clone)]
pub struct RiverGenConfig {
    pub seed: u32,
    pub min_river_length: f32,
    pub max_river_length: f32,
    pub base_width: f32,
    pub width_growth: f32,     // How much width increases downstream
    pub meander_scale: f32,
    pub tributary_chance: f32,
    pub waterfall_threshold: f32, // Slope that triggers waterfall
    pub canyon_depth_factor: f32,
}

impl Default for RiverGenConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            min_river_length: 200.0,
            max_river_length: 3000.0,
            base_width: 3.0,
            width_growth: 1.5,
            meander_scale: 0.003,
            tributary_chance: 0.3,
            waterfall_threshold: 0.3,
            canyon_depth_factor: 2.0,
        }
    }
}

// ============================================================================
// RIVER GENERATOR
// ============================================================================

/// Main river system generator
#[allow(dead_code)]
pub struct RiverGenerator {
    config: RiverGenConfig,
    path_seed: u32,
    meander_seed: u32,
    width_seed: u32,
}

impl RiverGenerator {
    pub fn new(config: RiverGenConfig) -> Self {
        let seed = config.seed;
        Self {
            config,
            path_seed: seed,
            meander_seed: seed.wrapping_add(1000),
            width_seed: seed.wrapping_add(2000),
        }
    }

    /// Check if a river should start at this position
    /// Rivers start in highlands and flow toward lowlands/ocean
    pub fn should_have_river_source(&self, x: f32, z: f32, height: f32, moisture: f32) -> bool {
        // Rivers need:
        // 1. Elevated terrain (mountains/hills)
        // 2. Sufficient moisture
        // 3. Noise check for distribution

        if height < 30.0 {
            return false; // Not high enough for headwaters
        }

        if moisture < 0.4 {
            return false; // Too dry
        }

        // Use noise to distribute river sources
        let source_noise = turbulence(
            Vec2::new(x * 0.0008, z * 0.0008),
            3, 2.0, 0.5, self.path_seed
        );

        source_noise > 0.7 && height > 40.0
    }

    /// Generate a complete river system from a source point
    pub fn generate_river_system(
        &self,
        source: Vec3,
        target_height: f32, // Sea level or lake level
        terrain_sampler: &dyn Fn(f32, f32) -> f32,
        local_seed: u32,
    ) -> RiverSystem {
        let combined_seed = self.config.seed.wrapping_add(local_seed);
        let mut segments = Vec::new();
        let mut waterfalls = Vec::new();

        let mut current_pos = source;
        let mut current_width = self.config.base_width;
        let mut total_length = 0.0;
        let mut segment_index = 0;

        // Determine river length based on source height
        let height_factor = (source.y - target_height) / 100.0;
        let target_length = lerp(
            self.config.min_river_length,
            self.config.max_river_length,
            height_factor.clamp(0.0, 1.0)
        );

        // Generate main river path
        while current_pos.y > target_height && total_length < target_length * 1.5 {
            let segment_seed = combined_seed.wrapping_add(segment_index * 1000);

            // Calculate flow direction using noise-based pathfinding
            let flow_dir = self.calculate_flow_direction(current_pos, target_height, segment_seed);

            // Segment length varies with terrain
            let base_segment_len = 20.0 + noise_util::hash(segment_seed) * 30.0;

            // Sample terrain ahead to determine segment type
            let next_pos = current_pos + flow_dir * base_segment_len;
            let current_terrain_height = terrain_sampler(current_pos.x, current_pos.z);
            let next_terrain_height = terrain_sampler(next_pos.x, next_pos.z);

            // Calculate slope
            let slope = (current_terrain_height - next_terrain_height) / base_segment_len;

            // Determine segment type based on conditions
            let segment_type = self.classify_segment(
                current_pos.y, next_pos.y, slope, current_width, segment_index
            );

            // Adjust width based on downstream distance
            let width_noise = noise_util::hash(segment_seed + 100);
            current_width += self.config.width_growth * 0.1 * (0.8 + width_noise * 0.4);
            current_width = current_width.min(50.0); // Cap max width

            // Generate meandering control points
            let control_points = self.generate_meander_points(
                current_pos, next_pos, current_width, segment_seed
            );

            // Calculate depth based on width and type
            let depth = self.calculate_depth(current_width, segment_type);

            // Flow speed based on slope and type
            let flow_speed = self.calculate_flow_speed(slope, segment_type);

            // Check for waterfall
            if slope > self.config.waterfall_threshold {
                let waterfall = self.create_waterfall(
                    current_pos, current_width, slope, segment_seed
                );
                waterfalls.push(waterfall);
            }

            let segment = RiverSegment {
                segment_type,
                start: current_pos,
                end: next_pos,
                width: current_width,
                depth,
                flow_speed,
                meander_amount: self.config.meander_scale * 100.0,
                control_points,
            };
            segments.push(segment);

            // Update position
            current_pos = next_pos;
            current_pos.y = next_terrain_height.max(target_height);
            total_length += base_segment_len;
            segment_index += 1;

            // Safety limit
            if segment_index > 200 {
                break;
            }
        }

        // Add final mouth segment
        if current_pos.y <= target_height + 2.0 {
            let mouth_segment = RiverSegment {
                segment_type: RiverSegmentType::Mouth,
                start: current_pos,
                end: current_pos + Vec3::new(0.0, 0.0, -current_width),
                width: current_width * 1.5,
                depth: self.calculate_depth(current_width * 1.5, RiverSegmentType::Mouth),
                flow_speed: 0.5,
                meander_amount: 0.0,
                control_points: vec![],
            };
            segments.push(mouth_segment);
        }

        // Generate tributaries
        let tributaries = self.generate_tributaries(
            &segments, target_height, terrain_sampler, combined_seed
        );

        RiverSystem {
            seed: combined_seed,
            source,
            mouth: current_pos,
            total_length,
            segments,
            tributaries,
            waterfalls,
        }
    }

    /// Calculate flow direction using gradient descent with noise
    fn calculate_flow_direction(&self, pos: Vec3, _target_height: f32, seed: u32) -> Vec3 {
        // Base direction: toward lower terrain (approximated by toward ocean = +X)
        let base_dir = Vec3::new(0.5, 0.0, 0.0).normalize();

        // Add noise-based meandering
        let meander_x = fbm(
            Vec2::new(pos.x * self.config.meander_scale, pos.z * self.config.meander_scale),
            3, 2.0, 0.5, seed
        );
        let meander_z = fbm(
            Vec2::new(pos.z * self.config.meander_scale, pos.x * self.config.meander_scale),
            3, 2.0, 0.5, seed + 100
        );

        // Combine base direction with meander
        let dir = Vec3::new(
            base_dir.x + meander_x * 0.5,
            -0.1, // Slight downward bias
            base_dir.z + meander_z * 0.8
        );

        dir.normalize()
    }

    /// Classify river segment type based on conditions
    fn classify_segment(
        &self,
        current_height: f32,
        _next_height: f32,
        slope: f32,
        width: f32,
        index: u32,
    ) -> RiverSegmentType {
        // High elevation = headwaters
        if current_height > 80.0 && index < 5 {
            return RiverSegmentType::Headwater;
        }

        // Very steep = waterfall or rapids
        if slope > self.config.waterfall_threshold {
            return RiverSegmentType::Waterfall;
        }
        if slope > self.config.waterfall_threshold * 0.5 {
            return RiverSegmentType::Rapids;
        }

        // Canyon conditions
        if current_height > 40.0 && width < 15.0 && slope > 0.1 {
            return RiverSegmentType::Canyon;
        }

        // Low elevation, wide = lowland
        if current_height < 20.0 && width > 15.0 {
            return RiverSegmentType::Lowland;
        }

        // Near sea level = delta
        if current_height < 5.0 {
            return RiverSegmentType::Delta;
        }

        RiverSegmentType::MainChannel
    }

    /// Generate meandering control points between two positions
    fn generate_meander_points(
        &self,
        start: Vec3,
        end: Vec3,
        width: f32,
        seed: u32,
    ) -> Vec<Vec3> {
        let mut points = Vec::new();
        let segment_dir = (end - start).normalize();
        let perpendicular = Vec3::new(-segment_dir.z, 0.0, segment_dir.x);
        let length = (end - start).length();

        // Number of control points based on length
        let num_points = (length / 15.0).max(2.0) as usize;

        for i in 0..num_points {
            let t = (i as f32 + 1.0) / (num_points as f32 + 1.0);
            let base_pos = start.lerp(end, t);

            // Meander offset using noise
            let noise_val = fbm(
                Vec2::new(base_pos.x * 0.02, base_pos.z * 0.02),
                2, 2.0, 0.5, seed + i as u32
            );

            // Meander amplitude scales with width and t (more in middle)
            let amplitude = width * 2.0 * (1.0 - (t - 0.5).abs() * 2.0);
            let offset = perpendicular * noise_val * amplitude;

            points.push(base_pos + offset);
        }

        points
    }

    /// Calculate river depth based on width and type
    fn calculate_depth(&self, width: f32, segment_type: RiverSegmentType) -> f32 {
        let base_depth = width * 0.2;

        let type_mult = match segment_type {
            RiverSegmentType::Headwater => 0.3,
            RiverSegmentType::MainChannel => 1.0,
            RiverSegmentType::Tributary => 0.6,
            RiverSegmentType::Lowland => 0.8,
            RiverSegmentType::Waterfall => 0.5,
            RiverSegmentType::Rapids => 0.4,
            RiverSegmentType::Canyon => 1.5, // Deep canyons
            RiverSegmentType::Delta => 0.6,
            RiverSegmentType::Mouth => 1.2,
            RiverSegmentType::Fork => 0.9,
        };

        (base_depth * type_mult).max(0.5)
    }

    /// Calculate flow speed based on slope and type
    fn calculate_flow_speed(&self, slope: f32, segment_type: RiverSegmentType) -> f32 {
        let base_speed = 1.0 + slope * 10.0;

        let type_mult = match segment_type {
            RiverSegmentType::Headwater => 1.2,
            RiverSegmentType::MainChannel => 1.0,
            RiverSegmentType::Tributary => 0.9,
            RiverSegmentType::Lowland => 0.5,
            RiverSegmentType::Waterfall => 3.0,
            RiverSegmentType::Rapids => 2.5,
            RiverSegmentType::Canyon => 1.8,
            RiverSegmentType::Delta => 0.3,
            RiverSegmentType::Mouth => 0.4,
            RiverSegmentType::Fork => 0.8,
        };

        (base_speed * type_mult).clamp(0.2, 10.0)
    }

    /// Create waterfall data at a steep section
    fn create_waterfall(&self, pos: Vec3, width: f32, slope: f32, seed: u32) -> WaterfallData {
        let height = slope * 30.0 + noise_util::hash(seed) * 20.0;
        let height = height.clamp(3.0, 50.0);

        WaterfallData {
            position: pos,
            width: width * (0.8 + noise_util::hash(seed + 1) * 0.4),
            height,
            flow_rate: width * 0.5,
            mist_radius: height * 0.8,
            pool_depth: height * 0.3 + 1.0,
            spray_particles: height > 5.0,
        }
    }

    /// Generate tributary rivers that join the main river
    fn generate_tributaries(
        &self,
        main_segments: &[RiverSegment],
        _target_height: f32,
        terrain_sampler: &dyn Fn(f32, f32) -> f32,
        seed: u32,
    ) -> Vec<RiverSystem> {
        let mut tributaries = Vec::new();

        // Check each segment for potential tributary join points
        for (i, segment) in main_segments.iter().enumerate() {
            // Skip certain segment types
            if matches!(segment.segment_type,
                RiverSegmentType::Waterfall |
                RiverSegmentType::Mouth |
                RiverSegmentType::Delta
            ) {
                continue;
            }

            let trib_seed = seed.wrapping_add(i as u32 * 5000);

            // Check if tributary should spawn here
            if noise_util::hash(trib_seed) < self.config.tributary_chance {
                // Determine tributary source direction (perpendicular to main river)
                let main_dir = (segment.end - segment.start).normalize();
                let side = if noise_util::hash(trib_seed + 1) > 0.5 { 1.0 } else { -1.0 };
                let perp_dir = Vec3::new(-main_dir.z * side, 0.0, main_dir.x * side);

                // Tributary source position
                let trib_length = 50.0 + noise_util::hash(trib_seed + 2) * 150.0;
                let trib_height_gain = 10.0 + noise_util::hash(trib_seed + 3) * 30.0;

                let trib_source = segment.start
                    + perp_dir * trib_length
                    + Vec3::new(0.0, trib_height_gain, 0.0);

                // Generate tributary (smaller config)
                let trib_config = RiverGenConfig {
                    seed: trib_seed,
                    min_river_length: trib_length * 0.8,
                    max_river_length: trib_length * 1.2,
                    base_width: self.config.base_width * 0.6,
                    width_growth: self.config.width_growth * 0.5,
                    tributary_chance: 0.0, // No sub-tributaries
                    ..self.config.clone()
                };

                let trib_gen = RiverGenerator::new(trib_config);
                let tributary = trib_gen.generate_river_system(
                    trib_source,
                    segment.start.y,
                    terrain_sampler,
                    trib_seed
                );

                tributaries.push(tributary);
            }
        }

        tributaries
    }

    /// Sample river presence at a world position
    /// Returns (is_river, distance_to_center, river_data)
    pub fn sample_river_at<'a>(&self, x: f32, z: f32, rivers: &'a [RiverSystem]) -> (bool, f32, Option<&'a RiverSegment>) {
        let pos = Vec2::new(x, z);
        let mut min_dist = f32::MAX;
        let mut closest_segment: Option<&RiverSegment> = None;

        for river in rivers {
            for segment in &river.segments {
                let seg_start = Vec2::new(segment.start.x, segment.start.z);
                let seg_end = Vec2::new(segment.end.x, segment.end.z);

                // Distance to line segment
                let dist = distance_to_line_segment(pos, seg_start, seg_end);

                if dist < min_dist {
                    min_dist = dist;
                    closest_segment = Some(segment);
                }
            }

            // Check tributaries too
            for tributary in &river.tributaries {
                for segment in &tributary.segments {
                    let seg_start = Vec2::new(segment.start.x, segment.start.z);
                    let seg_end = Vec2::new(segment.end.x, segment.end.z);
                    let dist = distance_to_line_segment(pos, seg_start, seg_end);

                    if dist < min_dist {
                        min_dist = dist;
                        closest_segment = Some(segment);
                    }
                }
            }
        }

        if let Some(segment) = closest_segment {
            let is_river = min_dist < segment.width * 0.5;
            (is_river, min_dist, Some(segment))
        } else {
            (false, min_dist, None)
        }
    }

    /// Get river height modifier at a position (for carving river bed)
    pub fn get_river_carve_depth(&self, x: f32, z: f32, rivers: &[RiverSystem]) -> f32 {
        let (is_river, dist, segment_opt) = self.sample_river_at(x, z, rivers);

        if let Some(segment) = segment_opt {
            let half_width = segment.width * 0.5;

            if dist < half_width {
                // Inside river - carve bed
                let center_factor = 1.0 - (dist / half_width);
                -segment.depth * center_factor
            } else if dist < half_width + 5.0 {
                // River bank - gradual slope
                let bank_factor = (dist - half_width) / 5.0;
                -segment.depth * 0.3 * (1.0 - bank_factor)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

// ============================================================================
// WATERFALL MESH GENERATION
// ============================================================================

/// Generate mesh data for a waterfall
pub fn generate_waterfall_mesh(
    waterfall: &WaterfallData,
    seed: u32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Waterfall is a curved sheet of water
    let segments_horizontal = 8;
    let segments_vertical = 12;

    let _half_width = waterfall.width * 0.5;

    for v in 0..=segments_vertical {
        let v_t = v as f32 / segments_vertical as f32;
        let y_offset = -waterfall.height * v_t;

        // Add curve to waterfall (parabolic)
        let z_curve = waterfall.height * 0.3 * v_t * v_t;

        // Add noise for turbulence
        let turbulence_factor = fbm(
            Vec2::new(v_t * 5.0, seed as f32 * 0.001),
            2, 2.0, 0.5, seed
        ) * 0.3;

        for h in 0..=segments_horizontal {
            let h_t = h as f32 / segments_horizontal as f32;
            let x_offset = (h_t - 0.5) * waterfall.width;

            // Add horizontal variation
            let x_variation = fbm(
                Vec2::new(h_t * 3.0, v_t * 3.0),
                2, 2.0, 0.5, seed + 100
            ) * 0.2 * waterfall.width;

            let pos = [
                waterfall.position.x + x_offset + x_variation,
                waterfall.position.y + y_offset,
                waterfall.position.z - z_curve + turbulence_factor,
            ];
            positions.push(pos);

            // Normal points outward from waterfall
            let normal = [0.0, 0.0, 1.0];
            normals.push(normal);

            // UV for water texture animation
            uvs.push([h_t, v_t * 2.0]); // Repeat V for flow
        }
    }

    // Generate indices
    for v in 0..segments_vertical {
        for h in 0..segments_horizontal {
            let row = segments_horizontal + 1;
            let i = v * row + h;

            indices.push(i as u32);
            indices.push((i + row) as u32);
            indices.push((i + 1) as u32);

            indices.push((i + 1) as u32);
            indices.push((i + row) as u32);
            indices.push((i + row + 1) as u32);
        }
    }

    (positions, normals, uvs, indices)
}

/// Generate plunge pool mesh at waterfall base
pub fn generate_plunge_pool(
    waterfall: &WaterfallData,
    seed: u32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Pool is an elliptical depression
    let pool_radius = waterfall.width * 0.8;
    let pool_length = waterfall.height * 0.4;
    let segments = 16;

    let pool_center = Vec3::new(
        waterfall.position.x,
        waterfall.position.y - waterfall.height - waterfall.pool_depth * 0.5,
        waterfall.position.z - pool_length * 0.5,
    );

    // Center vertex
    positions.push([pool_center.x, pool_center.y - waterfall.pool_depth, pool_center.z]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    // Ring vertices
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
        let x = pool_center.x + angle.cos() * pool_radius;
        let z = pool_center.z + angle.sin() * pool_length;

        // Add noise to edge
        let noise_val = noise_util::hash(seed + i) * 0.2;
        let y = pool_center.y + noise_val;

        positions.push([x, y, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([
            0.5 + angle.cos() * 0.5,
            0.5 + angle.sin() * 0.5,
        ]);
    }

    // Indices (fan from center)
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(0);
        indices.push((i + 1) as u32);
        indices.push((next + 1) as u32);
    }

    (positions, normals, uvs, indices)
}

// ============================================================================
// RIVER BANK AND DETAIL GENERATION
// ============================================================================

/// Types of riverbank features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiverbankFeature {
    GravelBar,
    SandBar,
    MuddyBank,
    RockyShore,
    Reeds,
    WillowOverhang,
    BeaverDam,
    LogJam,
    FishingSpot,
}

/// Generate riverbank features for a segment
pub fn generate_riverbank_features(
    segment: &RiverSegment,
    seed: u32,
) -> Vec<(RiverbankFeature, Vec3, f32)> {
    let mut features = Vec::new();

    let segment_length = (segment.end - segment.start).length();
    let num_features = (segment_length / 20.0) as usize;

    for i in 0..num_features {
        let feature_seed = seed.wrapping_add(i as u32 * 100);
        let t = (i as f32 + 0.5) / num_features as f32;

        // Position along segment
        let base_pos = segment.start.lerp(segment.end, t);

        // Offset to bank (alternating sides)
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let segment_dir = (segment.end - segment.start).normalize();
        let perp = Vec3::new(-segment_dir.z, 0.0, segment_dir.x);

        let bank_offset = segment.width * 0.5 + noise_util::hash(feature_seed) * 3.0;
        let pos = base_pos + perp * side * bank_offset;

        // Select feature type based on segment type and random
        let feature = select_bank_feature(segment.segment_type, feature_seed);

        // Scale
        let scale = 0.8 + noise_util::hash(feature_seed + 1) * 0.4;

        features.push((feature, pos, scale));
    }

    features
}

fn select_bank_feature(segment_type: RiverSegmentType, seed: u32) -> RiverbankFeature {
    let roll = noise_util::hash(seed);

    match segment_type {
        RiverSegmentType::Headwater | RiverSegmentType::Canyon => {
            if roll < 0.4 { RiverbankFeature::RockyShore }
            else if roll < 0.7 { RiverbankFeature::GravelBar }
            else { RiverbankFeature::LogJam }
        }
        RiverSegmentType::Lowland | RiverSegmentType::Delta => {
            if roll < 0.3 { RiverbankFeature::MuddyBank }
            else if roll < 0.5 { RiverbankFeature::Reeds }
            else if roll < 0.7 { RiverbankFeature::WillowOverhang }
            else if roll < 0.85 { RiverbankFeature::BeaverDam }
            else { RiverbankFeature::FishingSpot }
        }
        RiverSegmentType::MainChannel => {
            if roll < 0.25 { RiverbankFeature::SandBar }
            else if roll < 0.45 { RiverbankFeature::GravelBar }
            else if roll < 0.65 { RiverbankFeature::Reeds }
            else if roll < 0.8 { RiverbankFeature::WillowOverhang }
            else { RiverbankFeature::FishingSpot }
        }
        RiverSegmentType::Rapids => {
            if roll < 0.5 { RiverbankFeature::RockyShore }
            else if roll < 0.8 { RiverbankFeature::GravelBar }
            else { RiverbankFeature::LogJam }
        }
        _ => {
            if roll < 0.5 { RiverbankFeature::GravelBar }
            else { RiverbankFeature::SandBar }
        }
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

/// Distance from point to line segment
fn distance_to_line_segment(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line = line_end - line_start;
    let len_sq = line.length_squared();

    if len_sq < 0.001 {
        return (point - line_start).length();
    }

    let t = ((point - line_start).dot(line) / len_sq).clamp(0.0, 1.0);
    let projection = line_start + line * t;

    (point - projection).length()
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_river_generation() {
        let config = RiverGenConfig::default();
        let gen = RiverGenerator::new(config);

        let source = Vec3::new(-500.0, 80.0, 0.0);
        let terrain_sampler = |x: f32, z: f32| -> f32 {
            80.0 - x * 0.05 // Simple slope toward +X
        };

        let river = gen.generate_river_system(source, 0.0, &terrain_sampler, 12345);

        assert!(river.segments.len() > 0);
        assert!(river.total_length > 0.0);
    }

    #[test]
    fn test_waterfall_mesh() {
        let waterfall = WaterfallData {
            position: Vec3::ZERO,
            width: 10.0,
            height: 20.0,
            flow_rate: 5.0,
            mist_radius: 15.0,
            pool_depth: 3.0,
            spray_particles: true,
        };

        let (positions, normals, uvs, indices) = generate_waterfall_mesh(&waterfall, 12345);

        assert!(positions.len() > 0);
        assert_eq!(positions.len(), normals.len());
        assert_eq!(positions.len(), uvs.len());
        assert!(indices.len() > 0);
    }

    #[test]
    fn test_river_determinism() {
        let config = RiverGenConfig::default();
        let gen = RiverGenerator::new(config);

        let source = Vec3::new(-500.0, 80.0, 0.0);
        let terrain_sampler = |x: f32, z: f32| -> f32 { 80.0 - x * 0.05 };

        let river1 = gen.generate_river_system(source, 0.0, &terrain_sampler, 12345);
        let river2 = gen.generate_river_system(source, 0.0, &terrain_sampler, 12345);

        assert_eq!(river1.segments.len(), river2.segments.len());
        assert_eq!(river1.waterfalls.len(), river2.waterfalls.len());
    }
}
