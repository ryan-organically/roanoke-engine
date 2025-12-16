use std::collections::{HashMap, HashSet};
use glam::Vec3;
use croatoan_render::{TerrainPipeline, GrassPipeline, TreePipeline, DetritusPipeline, BuildingPipeline, ChunkBounds};
use croatoan_wfc::CornFieldBounds;

/// Coordinates for a chunk in chunk space (not world space)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn from_world_pos(world_pos: Vec3, chunk_size: f32) -> Self {
        Self {
            x: (world_pos.x / chunk_size).floor() as i32,
            z: (world_pos.z / chunk_size).floor() as i32,
        }
    }

    pub fn world_offset(&self, chunk_size: f32) -> (f32, f32) {
        (self.x as f32 * chunk_size, self.z as f32 * chunk_size)
    }
}

/// Data for a loaded chunk
pub struct LoadedChunk {
    pub terrain: TerrainPipeline,
    pub grass: Option<GrassPipeline>,
    pub trees: Vec<TreePipeline>, // Foliage: trees + shrubs (LOD0 - full detail)
    pub trees_lod1: Vec<TreePipeline>, // LOD1 simplified trees for distant rendering
    pub ferns: Vec<TreePipeline>, // Forest understory ferns
    pub detritus: Option<DetritusPipeline>,
    pub rocks: Vec<TreePipeline>, // Non-boulder rocks (pebble, small, medium, flat, mossy)
    pub boulders_lod0: Vec<TreePipeline>, // Boulder LOD0 (0-200 units)
    pub boulders_lod1: Vec<TreePipeline>, // Boulder LOD1 (150-500 units)
    pub boulders_lod2: Vec<TreePipeline>, // Boulder LOD2 (450-1200 units)
    pub dead_logs_lod0: Vec<TreePipeline>, // Dead log LOD0 (0-150 units)
    pub dead_logs_lod1: Vec<TreePipeline>, // Dead log LOD1 (100-400 units)
    pub dead_logs_lod2: Vec<TreePipeline>, // Dead log LOD2 (350-800 units)
    pub conifer_shrubs_lod0: Vec<TreePipeline>, // Conifer shrub LOD0 (0-100 units)
    pub conifer_shrubs_lod1: Vec<TreePipeline>, // Conifer shrub LOD1 (80-250 units)
    pub conifer_shrubs_lod2: Vec<TreePipeline>, // Conifer shrub LOD2 (200-500 units)
    pub buildings: Vec<BuildingPipeline>, // List of pipelines for different building types in this chunk
    pub river_water: Vec<TreePipeline>, // Flat calm water quads for rivers
    pub bounds: ChunkBounds,
}

/// Request to generate a chunk
#[derive(Clone)]
pub struct ChunkRequest {
    pub coord: ChunkCoord,
    pub seed: u32,
    /// Village center positions for tree clearing (forest clearing around settlements)
    pub village_centers: Vec<Vec3>,
    /// Corn field exclusion zones (no rocks spawn in these areas)
    pub corn_field_exclusions: Vec<CornFieldBounds>,
}

/// Manages chunk loading/unloading based on player position
pub struct ChunkManager {
    pub loaded_chunks: HashMap<ChunkCoord, LoadedChunk>,
    pub loading_chunks: HashSet<ChunkCoord>,
    pub chunk_size: f32,
    pub load_radius: i32,
    pub unload_radius: i32,
    player_chunk: ChunkCoord,
}

impl ChunkManager {
    pub fn new(chunk_size: f32, load_radius: i32, unload_radius: i32) -> Self {
        Self {
            loaded_chunks: HashMap::new(),
            loading_chunks: HashSet::new(),
            chunk_size,
            load_radius,
            unload_radius,
            player_chunk: ChunkCoord { x: 0, z: 0 },
        }
    }

    /// Update which chunks should be loaded based on player position
    /// Returns chunks to request for generation
    /// `village_centers` is used for tree clearing around settlements
    /// `corn_field_exclusions` prevents rocks from spawning in crop fields
    pub fn update(&mut self, player_pos: Vec3, seed: u32, village_centers: &[Vec3], corn_field_exclusions: &[CornFieldBounds]) -> Vec<ChunkRequest> {
        let new_player_chunk = ChunkCoord::from_world_pos(player_pos, self.chunk_size);

        // Debug: Track calls to update
        static mut CALL_COUNT: u32 = 0;
        unsafe {
            CALL_COUNT += 1;
            if CALL_COUNT % 300 == 1 {
                println!("[CHUNK DEBUG] update called: player_chunk=({},{}), loaded={}, loading={}",
                    new_player_chunk.x, new_player_chunk.z,
                    self.loaded_chunks.len(), self.loading_chunks.len());
            }
        }

        // Only update if player moved to a different chunk
        if new_player_chunk == self.player_chunk && !self.loaded_chunks.is_empty() {
            return Vec::new();
        }

        self.player_chunk = new_player_chunk;
        let mut requests = Vec::new();

        // Unload distant chunks
        let chunks_to_unload: Vec<ChunkCoord> = self.loaded_chunks
            .keys()
            .filter(|coord| {
                let dx = (coord.x - new_player_chunk.x).abs();
                let dz = (coord.z - new_player_chunk.z).abs();
                dx > self.unload_radius || dz > self.unload_radius
            })
            .cloned()
            .collect();

        for coord in chunks_to_unload {
            self.loaded_chunks.remove(&coord);
            println!("[CHUNK] Unloaded chunk ({}, {})", coord.x, coord.z);
        }

        // Request new chunks that should be loaded, sorted by distance (closest first)
        let mut pending_coords: Vec<(ChunkCoord, i32)> = Vec::new();

        for dz in -self.load_radius..=self.load_radius {
            for dx in -self.load_radius..=self.load_radius {
                let coord = ChunkCoord {
                    x: new_player_chunk.x + dx,
                    z: new_player_chunk.z + dz,
                };

                // Skip if already loaded or loading
                if self.loaded_chunks.contains_key(&coord) || self.loading_chunks.contains(&coord) {
                    continue;
                }

                // Calculate squared distance for sorting (no sqrt needed)
                let dist_sq = dx * dx + dz * dz;
                pending_coords.push((coord, dist_sq));
            }
        }

        // Sort by distance (closest chunks first)
        pending_coords.sort_by_key(|(_, dist_sq)| *dist_sq);

        // Mark as loading and create requests
        for (coord, _) in pending_coords {
            self.loading_chunks.insert(coord);
            requests.push(ChunkRequest {
                coord,
                seed,
                village_centers: village_centers.to_vec(),
                corn_field_exclusions: corn_field_exclusions.to_vec(),
            });
        }

        if !requests.is_empty() {
            println!("[CHUNK] Requesting {} new chunks around ({}, {})",
                     requests.len(), new_player_chunk.x, new_player_chunk.z);
        }

        requests
    }

    /// Called when a chunk has been generated and is ready to be added
    pub fn add_chunk(&mut self, coord: ChunkCoord, chunk: LoadedChunk) {
        self.loading_chunks.remove(&coord);
        self.loaded_chunks.insert(coord, chunk);
    }

    /// Get the number of chunks in each radius tier (for stats)
    pub fn get_stats(&self) -> (usize, usize) {
        (self.loaded_chunks.len(), self.loading_chunks.len())
    }

    /// Iterator over all loaded chunks for rendering
    pub fn iter_chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &LoadedChunk)> {
        self.loaded_chunks.iter()
    }

    /// Get total counts
    pub fn chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    /// Update load/unload radii based on render distance
    /// Trees render at 1.5x render_distance with LOD system, so we need chunks loaded for that
    pub fn update_radius_for_render_distance(&mut self, render_distance: f32) {
        // Validate input - clamp to sane range
        let render_distance = render_distance.clamp(100.0, 1000.0);

        // Trees visible at 1.5x render distance (LOD1 extends further)
        // Buildings at 1.0x, so trees are the furthest-visible objects
        let max_visible_distance = render_distance * 1.5;
        // Convert to chunk units and round up, ensure chunk_size is valid
        let chunk_size = self.chunk_size.max(1.0);
        let needed_radius = (max_visible_distance / chunk_size).ceil() as i32;
        // Allow up to 4 chunks for extended render distance with LOD system
        // 1-2 chunks: 150-400 render distance
        // 3-4 chunks: 400-1000 render distance (LOD1 trees fill the extra distance)
        self.load_radius = needed_radius.clamp(1, 4);
        // Unload radius should be 1 chunk beyond load radius for hysteresis
        self.unload_radius = self.load_radius + 1;

        println!("[CHUNK] Updated radii for render_distance={:.0}: load={}, unload={} ({} max chunks)",
                 render_distance, self.load_radius, self.unload_radius,
                 (self.load_radius * 2 + 1) * (self.load_radius * 2 + 1));
    }
}
