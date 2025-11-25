pub mod noise_util;
pub mod seed;
pub mod mesh_gen;
pub mod vegetation;
pub mod trees;
pub mod rocks;
pub mod buildings;

// Re-export commonly used items
pub use noise_util::{fbm, ridged, turbulence};
pub use seed::WorldSeed;
pub use mesh_gen::generate_terrain_chunk;
pub use vegetation::generate_vegetation_for_chunk;
pub use vegetation::generate_detritus_for_chunk;
pub use trees::generate_trees_for_chunk;
pub use trees::TreeTemplate;
pub use rocks::generate_rocks_for_chunk;
pub use buildings::generate_buildings_for_chunk;
