//! # Croatoan WFC - Procedural Generation Library
//!
//! This crate provides procedural generation for terrain, vegetation, and structures.
//!
//! ## Key Systems
//!
//! ### Terrain Generation
//! - `generate_terrain_chunk`: Creates heightmap-based terrain meshes with biome coloring
//! - `get_height_at`: Sample terrain height at any world position
//! - `distance_to_shoreline`: Calculate distance from any point to the ocean
//!
//! ### Vegetation System
//! - **LowlandBunch**: Clustered vegetation units (rocks + pebbles + bushes + trees)
//! - **Treeline**: Trees only spawn 40+ yards from shoreline
//! - `generate_trees_for_chunk`: Full tree/bush generation with bunch integration
//! - `generate_bunches_for_chunk`: Get bunch data for coordinated rock placement
//!
//! ### Rock/Pebble System
//! - 10x density rocks and pebbles for natural ground coverage
//! - Bunch-integrated anchor rocks and pebble clusters
//! - Beach pebble strips along tide line
//!
//! ### Detritus System
//! - Fallen logs in forest areas
//! - Dead branches in scrub zones
//! - 10x density for visible ground clutter

pub mod noise_util;
pub mod seed;
pub mod mesh_gen;
pub mod vegetation;
pub mod trees;
pub mod rocks;
pub mod buildings;
pub mod villages;

// Re-export commonly used items
pub use noise_util::{fbm, ridged, turbulence};
pub use seed::WorldSeed;

// Terrain
pub use mesh_gen::generate_terrain_chunk;
pub use mesh_gen::get_height_at;
pub use mesh_gen::distance_to_shoreline;
pub use mesh_gen::get_biome_t;

// Vegetation
pub use vegetation::generate_vegetation_for_chunk;
pub use vegetation::generate_detritus_for_chunk;

// Trees & Bunches
pub use trees::generate_trees_for_chunk;
pub use trees::generate_bunches_for_chunk;
pub use trees::TreeTemplate;
pub use trees::LowlandBunch;
pub use trees::BunchInstances;

// Rocks
pub use rocks::generate_rocks_for_chunk;
pub use rocks::RockType;

// Buildings
pub use buildings::generate_buildings_for_chunk;

// Villages
pub use villages::{
    find_village_sites,
    generate_world_village,
    get_village_structures_for_chunk,
    WorldVillage,
    VillageStructure,
    VillageStructureType,
};
