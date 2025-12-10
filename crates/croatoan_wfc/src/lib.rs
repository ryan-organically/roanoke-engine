#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

//! # Croatoan WFC - Procedural Generation Library
//!
//! This crate provides comprehensive procedural generation for terrain, vegetation,
//! structures, and environmental features across square miles of diverse biomes.
//!
//! ## Key Systems
//!
//! ### Biome System (`biome.rs`)
//! - 15+ distinct biome types (ocean, beach, salt marsh, forest, mountains, etc.)
//! - Biome blending and transitions
//! - Flora and fauna weight tables per biome
//! - Environmental factors (moisture, temperature, elevation)
//!
//! ### Terrain Generation
//! - `TerrainGenerator`: Master terrain system integrating all features
//! - `generate_terrain_chunk`: Creates heightmap-based terrain meshes with biome coloring
//! - `get_height_at`: Sample terrain height at any world position
//! - Rolling mountains with ridged Perlin noise
//! - River valleys carved into terrain
//!
//! ### Cave System (`caves.rs`)
//! - Complete cave networks from entrance to depths
//! - Multiple section types (chambers, passages, shafts, sacred chambers)
//! - Bone and artifact spawning
//! - Stalactites, stalagmites, and mineral formations
//!
//! ### River System (`rivers.rs`)
//! - Perlin-based river path generation
//! - Tributary networks
//! - Waterfalls at elevation changes
//! - River deltas and estuaries
//! - Canyon formation
//!
//! ### Salt Marsh System (`terrain.rs`)
//! - Tidal channels
//! - Salt pans
//! - Cordgrass and vegetation
//! - Realistic marsh coloring
//!
//! ### Vegetation System
//! - **LowlandBunch**: Clustered vegetation units (rocks + pebbles + bushes + trees)
//! - **Treeline**: Trees only spawn 40+ yards from shoreline
//! - `generate_trees_for_chunk`: Full tree/bush generation with bunch integration
//! - Biome-specific flora spawning
//!
//! ### Fauna Spawning (`biome_spawner.rs`)
//! - Habitat-based animal spawning
//! - Cross-biome overlap zones
//! - Pack behavior for social animals
//! - Environmental suitability modifiers
//!
//! ### Rock/Pebble System
//! - 10x density rocks and pebbles for natural ground coverage
//! - Bunch-integrated anchor rocks and pebble clusters
//! - Beach pebble strips along tide line
//!
//! ### Village System
//! - Site selection based on terrain
//! - Longhouses and structures
//! - Agricultural areas

// Core modules
pub mod noise_util;
pub mod seed;
pub mod mesh_gen;

// New comprehensive systems
pub mod biome;
pub mod caves;
pub mod rivers;
pub mod terrain;
pub mod biome_spawner;

// Existing systems
pub mod vegetation;
pub mod trees;
pub mod foliage_gen;
pub mod rocks;
pub mod buildings;
pub mod villages;

// Re-export commonly used items
pub use noise_util::{fbm, ridged, turbulence, hash};
pub use seed::WorldSeed;

// Legacy terrain (for backward compatibility)
pub use mesh_gen::generate_terrain_chunk;
pub use mesh_gen::get_height_at;
pub use mesh_gen::distance_to_shoreline;
pub use mesh_gen::get_biome_t;

// New terrain system
pub use terrain::{
    TerrainGenerator,
    TerrainConfig,
    TerrainData,
    TerrainFeature,
    TerrainChunkData,
    SaltMarshGenerator,
    SaltMarshDetail,
    MountainGenerator,
    MountainDetail,
    GrassInstance,
    RockInstance,
    MountainRockType,
};

// Biome system
pub use biome::{
    BiomeType,
    BiomeData,
    BiomeGenerator,
    WorldGenConfig,
    FloraType,
    FaunaType,
    get_flora_weights,
    get_fauna_weights,
};

// Cave system
pub use caves::{
    CaveGenerator,
    CaveGenConfig,
    CaveSystem,
    CaveSection,
    CaveSectionType,
    CaveFeature,
    CaveFeatureInstance,
    ArtifactRarity,
    BoneType,
    ArtifactType,
    BoneInstance,
    ArtifactInstance,
    generate_bones_for_section,
    generate_artifacts_for_section,
};

// River system
pub use rivers::{
    RiverGenerator,
    RiverGenConfig,
    RiverSystem,
    RiverSegment,
    RiverSegmentType,
    WaterfallData,
    RiverbankFeature,
    generate_waterfall_mesh,
    generate_plunge_pool,
    generate_riverbank_features,
};

// Biome-aware spawning
pub use biome_spawner::{
    SpawnConfig,
    FloraSpawner,
    FaunaSpawner,
    BiomeSpawner,
    FloraInstance,
    FaunaInstance,
    FaunaAge,
    BehaviorState,
    ChunkSpawns,
};

// Vegetation
pub use vegetation::generate_vegetation_for_chunk;
pub use vegetation::generate_detritus_for_chunk;

// Trees & Bunches
pub use trees::generate_trees_for_chunk;
pub use trees::generate_bunches_for_chunk;
pub use trees::TreeTemplate;
pub use trees::LowlandBunch;
pub use trees::BunchInstances;
// Foliage (multi-model trees + shrubs)
pub use foliage_gen::generate_foliage_for_chunk;
pub use foliage_gen::FoliageInstance;
pub use foliage_gen::FoliageInstances;

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
