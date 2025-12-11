//! World Features Manager
//!
//! Manages world-level terrain features that span multiple chunks:
//! - River systems with waterfalls and tributaries
//! - Cave systems with artifacts and bones
//!
//! These are generated once at world creation and cached for terrain queries.

use croatoan_wfc::{
    RiverGenerator, RiverGenConfig, RiverSystem, RiverSegment, RiverSegmentType,
    WaterfallData, RiverbankFeature, generate_riverbank_features,
    CaveGenerator, CaveGenConfig, CaveSystem, CaveSection, CaveSectionType,
    CaveFeature, CaveFeatureInstance, ArtifactRarity,
    get_height_at,
};
use glam::{Vec2, Vec3};
use std::collections::HashMap;

/// Manages all world-level terrain features
pub struct WorldFeatures {
    seed: u32,
    river_generator: RiverGenerator,
    cave_generator: CaveGenerator,

    /// All river systems in the world
    pub rivers: Vec<RiverSystem>,

    /// All cave systems in the world (keyed by entrance chunk)
    pub caves: Vec<CaveSystem>,

    /// Spatial index of cave entrances for fast lookup
    cave_spatial: HashMap<(i32, i32), Vec<usize>>,

    /// Spatial index of river segments for fast lookup
    river_spatial: HashMap<(i32, i32), Vec<(usize, usize)>>, // (river_idx, segment_idx)
}

impl WorldFeatures {
    const SPATIAL_CELL_SIZE: f32 = 256.0; // Match chunk size
    const MAX_RIVERS: usize = 12;
    const MAX_CAVES: usize = 20;

    /// Create a new world features manager (lazy initialization - call generate() to populate)
    pub fn new(seed: u32) -> Self {
        let river_config = RiverGenConfig {
            seed,
            min_river_length: 300.0,
            max_river_length: 2000.0,
            base_width: 4.0,
            width_growth: 1.8,
            meander_scale: 0.004,
            tributary_chance: 0.25,
            waterfall_threshold: 0.25,
            canyon_depth_factor: 2.5,
        };

        let cave_config = CaveGenConfig {
            seed,
            min_cave_length: 80.0,
            max_cave_length: 400.0,
            min_depth: 15.0,
            max_depth: 60.0,
            passage_width: 4.0,
            chamber_probability: 0.3,
            branch_probability: 0.2,
            water_probability: 0.25,
            artifact_density: 0.06,
            bone_density: 0.1,
        };

        // Create without generating - call generate_world_features() explicitly when ready
        Self {
            seed,
            river_generator: RiverGenerator::new(river_config),
            cave_generator: CaveGenerator::new(cave_config),
            rivers: Vec::new(),
            caves: Vec::new(),
            cave_spatial: HashMap::new(),
            river_spatial: HashMap::new(),
        }
    }

    /// Create and generate all features immediately
    pub fn new_with_generation(seed: u32) -> Self {
        let mut features = Self::new(seed);
        features.generate_world_features();
        features
    }

    /// Generate all rivers and caves for the world
    pub fn generate_world_features(&mut self) {
        println!("[WORLD] Generating world features...");

        // Generate rivers from highland sources
        self.generate_rivers();

        // Generate caves in hilly/mountainous terrain
        self.generate_caves();

        println!("[WORLD] Generated {} rivers, {} caves",
            self.rivers.len(), self.caves.len());
    }

    /// Generate river systems from highland areas
    fn generate_rivers(&mut self) {
        // Sample potential river source locations in highlands
        // Rivers flow from negative X (inland mountains) toward positive X (ocean)
        let source_candidates = vec![
            Vec3::new(-800.0, 80.0, -400.0),
            Vec3::new(-600.0, 70.0, 200.0),
            Vec3::new(-900.0, 90.0, 0.0),
            Vec3::new(-700.0, 75.0, -200.0),
            Vec3::new(-500.0, 60.0, 400.0),
            Vec3::new(-1000.0, 100.0, -100.0),
            Vec3::new(-850.0, 85.0, 300.0),
            Vec3::new(-650.0, 65.0, -500.0),
            Vec3::new(-750.0, 72.0, 150.0),
            Vec3::new(-950.0, 95.0, -300.0),
            Vec3::new(-550.0, 55.0, 250.0),
            Vec3::new(-1100.0, 105.0, 50.0),
        ];

        // Terrain sampler using existing height function
        let seed = self.seed;
        let terrain_sampler = |x: f32, z: f32| -> f32 {
            get_height_at(x, z, seed).0
        };

        for (i, source) in source_candidates.iter().enumerate() {
            if i >= Self::MAX_RIVERS {
                break;
            }

            // Check if this is a valid river source
            let height = terrain_sampler(source.x, source.z);
            if height < 30.0 {
                continue; // Too low for headwaters
            }

            // Adjust source to actual terrain height
            let adjusted_source = Vec3::new(source.x, height, source.z);

            let river = self.river_generator.generate_river_system(
                adjusted_source,
                0.0, // Sea level
                &terrain_sampler,
                self.seed.wrapping_add(i as u32 * 7919),
            );

            // Only keep rivers with meaningful length
            if river.segments.len() >= 3 && river.total_length > 100.0 {
                // Add to spatial index
                self.index_river_segments(self.rivers.len(), &river);
                self.rivers.push(river);
            }
        }
    }

    /// Generate cave systems in suitable terrain
    fn generate_caves(&mut self) {
        // Sample potential cave locations
        let seed = self.seed;
        let terrain_sampler = |x: f32, z: f32| -> f32 {
            get_height_at(x, z, seed).0
        };

        // Grid search for cave locations
        let search_step = 300.0;
        let search_range = 1500.0;

        let mut cave_count = 0;
        let mut x = -search_range;

        while x < search_range && cave_count < Self::MAX_CAVES {
            let mut z = -search_range;
            while z < search_range && cave_count < Self::MAX_CAVES {
                let height = terrain_sampler(x, z);

                // Calculate slope by sampling nearby
                let dx = terrain_sampler(x + 5.0, z) - terrain_sampler(x - 5.0, z);
                let dz = terrain_sampler(x, z + 5.0) - terrain_sampler(x, z - 5.0);
                let slope = ((dx * dx + dz * dz) / 100.0).sqrt();

                // Check if cave should exist here
                let local_seed = self.seed.wrapping_add(
                    ((x / search_step) as i32 * 1000 + (z / search_step) as i32) as u32
                );

                if self.cave_generator.should_have_cave(x, z, height, slope) {
                    let entrance = Vec3::new(x, height, z);
                    let cave = self.cave_generator.generate_cave_system(entrance, local_seed);

                    // Only keep caves with meaningful content
                    if cave.sections.len() >= 2 {
                        // Add to spatial index
                        let cell = self.pos_to_cell(x, z);
                        self.cave_spatial.entry(cell).or_default().push(self.caves.len());
                        self.caves.push(cave);
                        cave_count += 1;
                    }
                }

                z += search_step;
            }
            x += search_step;
        }
    }

    /// Add river segments to spatial index
    fn index_river_segments(&mut self, river_idx: usize, river: &RiverSystem) {
        for (seg_idx, segment) in river.segments.iter().enumerate() {
            // Index both start and end cells
            let start_cell = self.pos_to_cell(segment.start.x, segment.start.z);
            let end_cell = self.pos_to_cell(segment.end.x, segment.end.z);

            self.river_spatial.entry(start_cell).or_default().push((river_idx, seg_idx));
            if end_cell != start_cell {
                self.river_spatial.entry(end_cell).or_default().push((river_idx, seg_idx));
            }
        }

        // Also index tributaries
        for tributary in &river.tributaries {
            for (seg_idx, segment) in tributary.segments.iter().enumerate() {
                let cell = self.pos_to_cell(segment.start.x, segment.start.z);
                self.river_spatial.entry(cell).or_default().push((river_idx, seg_idx));
            }
        }
    }

    /// Convert world position to spatial cell
    fn pos_to_cell(&self, x: f32, z: f32) -> (i32, i32) {
        (
            (x / Self::SPATIAL_CELL_SIZE).floor() as i32,
            (z / Self::SPATIAL_CELL_SIZE).floor() as i32,
        )
    }

    /// Get height modification for rivers at a position (negative = carve)
    pub fn get_river_height_mod(&self, x: f32, z: f32) -> f32 {
        let cell = self.pos_to_cell(x, z);
        let pos = Vec2::new(x, z);

        // Check current cell and neighbors
        let mut min_dist = f32::MAX;
        let mut closest_segment: Option<&RiverSegment> = None;

        for dx in -1..=1 {
            for dz in -1..=1 {
                let check_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(segments) = self.river_spatial.get(&check_cell) {
                    for &(river_idx, seg_idx) in segments {
                        if let Some(river) = self.rivers.get(river_idx) {
                            if let Some(segment) = river.segments.get(seg_idx) {
                                let dist = self.distance_to_segment(pos, segment);
                                if dist < min_dist {
                                    min_dist = dist;
                                    closest_segment = Some(segment);
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(segment) = closest_segment {
            let half_width = segment.width * 0.5;

            if min_dist < half_width {
                // Inside river - carve bed
                let center_factor = 1.0 - (min_dist / half_width);
                -segment.depth * center_factor
            } else if min_dist < half_width + 8.0 {
                // River bank - gradual slope
                let bank_factor = (min_dist - half_width) / 8.0;
                -segment.depth * 0.3 * (1.0 - bank_factor)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Check if a position is in water (river)
    pub fn is_in_river(&self, x: f32, z: f32) -> bool {
        let cell = self.pos_to_cell(x, z);
        let pos = Vec2::new(x, z);

        for dx in -1..=1 {
            for dz in -1..=1 {
                let check_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(segments) = self.river_spatial.get(&check_cell) {
                    for &(river_idx, seg_idx) in segments {
                        if let Some(river) = self.rivers.get(river_idx) {
                            if let Some(segment) = river.segments.get(seg_idx) {
                                let dist = self.distance_to_segment(pos, segment);
                                if dist < segment.width * 0.5 {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Get cave entrances near a position
    pub fn get_caves_near(&self, x: f32, z: f32, radius: f32) -> Vec<&CaveSystem> {
        let mut result = Vec::new();
        let cell = self.pos_to_cell(x, z);
        let cell_radius = (radius / Self::SPATIAL_CELL_SIZE).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dz in -cell_radius..=cell_radius {
                let check_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(cave_indices) = self.cave_spatial.get(&check_cell) {
                    for &cave_idx in cave_indices {
                        if let Some(cave) = self.caves.get(cave_idx) {
                            let dx = cave.entrance_pos.x - x;
                            let dz = cave.entrance_pos.z - z;
                            if (dx * dx + dz * dz).sqrt() <= radius {
                                result.push(cave);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// Get all waterfalls in the world
    pub fn get_waterfalls(&self) -> Vec<&WaterfallData> {
        self.rivers.iter()
            .flat_map(|r| r.waterfalls.iter())
            .collect()
    }

    /// Get river segment info at position (for debugging/UI)
    pub fn get_river_info_at(&self, x: f32, z: f32) -> Option<RiverInfo> {
        let cell = self.pos_to_cell(x, z);
        let pos = Vec2::new(x, z);

        for dx in -1..=1 {
            for dz in -1..=1 {
                let check_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(segments) = self.river_spatial.get(&check_cell) {
                    for &(river_idx, seg_idx) in segments {
                        if let Some(river) = self.rivers.get(river_idx) {
                            if let Some(segment) = river.segments.get(seg_idx) {
                                let dist = self.distance_to_segment(pos, segment);
                                if dist < segment.width {
                                    return Some(RiverInfo {
                                        segment_type: segment.segment_type,
                                        width: segment.width,
                                        depth: segment.depth,
                                        flow_speed: segment.flow_speed,
                                        distance_to_center: dist,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Distance from point to river segment
    fn distance_to_segment(&self, point: Vec2, segment: &RiverSegment) -> f32 {
        let seg_start = Vec2::new(segment.start.x, segment.start.z);
        let seg_end = Vec2::new(segment.end.x, segment.end.z);

        let line = seg_end - seg_start;
        let len_sq = line.length_squared();

        if len_sq < 0.001 {
            return (point - seg_start).length();
        }

        let t = ((point - seg_start).dot(line) / len_sq).clamp(0.0, 1.0);
        let projection = seg_start + line * t;

        (point - projection).length()
    }

    /// Get statistics about generated features
    pub fn stats(&self) -> WorldFeatureStats {
        let total_river_length: f32 = self.rivers.iter()
            .map(|r| r.total_length)
            .sum();

        let total_waterfalls: usize = self.rivers.iter()
            .map(|r| r.waterfalls.len())
            .sum();

        let total_cave_sections: usize = self.caves.iter()
            .map(|c| c.sections.len())
            .sum();

        let total_cave_features: usize = self.caves.iter()
            .map(|c| c.features.len())
            .sum();

        WorldFeatureStats {
            river_count: self.rivers.len(),
            total_river_length,
            waterfall_count: total_waterfalls,
            cave_count: self.caves.len(),
            cave_section_count: total_cave_sections,
            cave_feature_count: total_cave_features,
        }
    }
}

/// Information about a river at a position
#[derive(Debug, Clone)]
pub struct RiverInfo {
    pub segment_type: RiverSegmentType,
    pub width: f32,
    pub depth: f32,
    pub flow_speed: f32,
    pub distance_to_center: f32,
}

/// Statistics about world features
#[derive(Debug, Clone)]
pub struct WorldFeatureStats {
    pub river_count: usize,
    pub total_river_length: f32,
    pub waterfall_count: usize,
    pub cave_count: usize,
    pub cave_section_count: usize,
    pub cave_feature_count: usize,
}
