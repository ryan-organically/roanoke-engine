//! # Village World Integration
//!
//! This module handles village placement in the world and provides chunk-based
//! structure streaming for efficient rendering.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use croatoan_wfc::{find_village_sites, generate_world_village, get_village_structures_for_chunk};
//! use glam::Vec3;
//!
//! // 1. Find suitable village locations
//! let sites = find_village_sites(
//!     world_seed,
//!     Vec3::new(-1000.0, 0.0, -1000.0),
//!     Vec3::new(1000.0, 0.0, 1000.0),
//!     10, // max villages
//! );
//!
//! // 2. Generate villages at each site
//! let villages: Vec<_> = sites.iter().enumerate()
//!     .map(|(i, &pos)| generate_world_village(pos, world_seed, i as u64))
//!     .collect();
//!
//! // 3. When rendering a chunk, get its structures
//! for village in &villages {
//!     let structures = get_village_structures_for_chunk(
//!         village,
//!         chunk_x, chunk_z, chunk_size,
//!         world_seed,
//!     );
//!
//!     for s in structures {
//!         // s.mesh_vertices, s.mesh_indices, s.transform
//!     }
//! }
//! ```
//!
//! ## Site Selection Criteria
//!
//! Villages are placed based on terrain analysis:
//! - **Elevation**: 3-60m (above water, below mountains)
//! - **Flatness**: < 8m height variation across 80m
//! - **Spacing**: 400m minimum between villages
//!
//! ## Chunk Integration
//!
//! `get_village_structures_for_chunk()` returns all village structures
//! that fall within a given chunk, with flattened mesh data ready for
//! GPU buffer creation.
//!
//! ## Structure Types
//!
//! - `Longhouse` - Main dwelling structures
//! - `FirePit` - Ceremonial and domestic fires
//! - `CornPlant` - Agricultural elements at various growth stages
//! - `PrayerSite` - Sacred locations (future)

use crate::mesh_gen::get_height_at;
use noise::{NoiseFn, Perlin};
use glam::{Mat4, Vec3, Quat};
use croatoan_procgen::{
    VillageId, VillageRecipe, VillageLayout, generate_village,
    LonghouseMesh, FirePitMesh, generate_fire_pit,
    CornGrowthStage, generate_corn_plant,
};

/// Village instance in the world
#[derive(Debug, Clone)]
pub struct WorldVillage {
    pub id: VillageId,
    pub center: Vec3,
    pub layout: VillageLayout,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

/// Structure instance for rendering
#[derive(Debug)]
pub struct VillageStructure {
    pub structure_type: VillageStructureType,
    pub transform: Mat4,
    pub mesh_vertices: Vec<f32>,
    pub mesh_indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VillageStructureType {
    Longhouse,
    FirePit,
    CornPlant,
    PrayerSite,
}

/// Find suitable village locations in a region
pub fn find_village_sites(
    world_seed: u32,
    region_min: Vec3,
    region_max: Vec3,
    max_villages: u32,
) -> Vec<Vec3> {
    let noise = Perlin::new(world_seed + 7777); // Village placement noise
    let mut sites = Vec::new();

    // Sample grid for potential village sites
    let sample_step = 200.0; // Check every 200 units
    let min_village_spacing = 400.0; // Minimum distance between villages

    let mut x = region_min.x;
    while x < region_max.x {
        let mut z = region_min.z;
        while z < region_max.z {
            // Check if this is a good village site
            if let Some(score) = evaluate_village_site(x, z, world_seed, &noise) {
                if score > 0.7 { // High quality site threshold
                    let pos = Vec3::new(x, 0.0, z);

                    // Check spacing from existing sites
                    let too_close = sites.iter().any(|s: &Vec3| {
                        let dx = s.x - x;
                        let dz = s.z - z;
                        (dx * dx + dz * dz).sqrt() < min_village_spacing
                    });

                    if !too_close {
                        let (height, _) = get_height_at(x, z, world_seed);
                        sites.push(Vec3::new(x, height, z));

                        if sites.len() >= max_villages as usize {
                            return sites;
                        }
                    }
                }
            }
            z += sample_step;
        }
        x += sample_step;
    }

    sites
}

fn evaluate_village_site(x: f32, z: f32, seed: u32, noise: &Perlin) -> Option<f32> {
    let mut score = 0.5;

    // Get terrain height
    let (height, biome) = get_height_at(x, z, seed);

    // Must be above water
    if height < 3.0 {
        return None;
    }

    // Prefer mid-elevation (not too high, not beach)
    if height > 10.0 && height < 60.0 {
        score += 0.2;
    }

    // Check flatness (sample 4 corners of potential village area)
    let check_radius = 40.0;
    let samples = [
        get_height_at(x - check_radius, z - check_radius, seed).0,
        get_height_at(x + check_radius, z - check_radius, seed).0,
        get_height_at(x - check_radius, z + check_radius, seed).0,
        get_height_at(x + check_radius, z + check_radius, seed).0,
    ];

    let max_slope = samples.iter()
        .map(|h| (h - height).abs())
        .fold(0.0_f32, |a, b| a.max(b));

    if max_slope > 8.0 {
        return None; // Too steep
    }

    score += (8.0 - max_slope) / 8.0 * 0.3; // Flatter is better

    // Use noise for some randomness
    let noise_val = noise.get([x as f64 * 0.005, z as f64 * 0.005]) as f32;
    score += noise_val * 0.2;

    Some(score)
}

/// Generate a village at a given location
pub fn generate_world_village(center: Vec3, seed: u32, village_id: u64) -> WorldVillage {
    let id = VillageId(village_id);

    // Determine village size based on noise
    let noise = Perlin::new(seed);
    let size_roll = noise.get([center.x as f64 * 0.01, center.z as f64 * 0.01]) as f32;

    let recipe = if size_roll < -0.3 {
        VillageRecipe::small_camp(seed)
    } else if size_roll > 0.3 {
        VillageRecipe::large_village(seed)
    } else {
        VillageRecipe::medium_village(seed)
    };

    let layout = generate_village(center, &recipe, id);

    let bounds_radius = layout.bounds_radius;
    let bounds_min = Vec3::new(center.x - bounds_radius, center.y - 5.0, center.z - bounds_radius);
    let bounds_max = Vec3::new(center.x + bounds_radius, center.y + 20.0, center.z + bounds_radius);

    WorldVillage {
        id,
        center,
        layout,
        bounds_min,
        bounds_max,
    }
}

/// Get village structures that fall within a chunk
pub fn get_village_structures_for_chunk(
    village: &WorldVillage,
    chunk_min_x: f32,
    chunk_min_z: f32,
    chunk_size: f32,
    world_seed: u32,
) -> Vec<VillageStructure> {
    let mut structures = Vec::new();

    let chunk_max_x = chunk_min_x + chunk_size;
    let chunk_max_z = chunk_min_z + chunk_size;

    // Check if village overlaps this chunk at all
    if village.bounds_max.x < chunk_min_x || village.bounds_min.x > chunk_max_x ||
       village.bounds_max.z < chunk_min_z || village.bounds_min.z > chunk_max_z {
        return structures;
    }

    // Add longhouses that are in this chunk
    for longhouse in &village.layout.longhouses {
        let pos = longhouse.position;

        // Check if longhouse center is in chunk (with some margin for the structure extent)
        let margin = longhouse.recipe.length() * 0.6;
        if pos.x >= chunk_min_x - margin && pos.x <= chunk_max_x + margin &&
           pos.z >= chunk_min_z - margin && pos.z <= chunk_max_z + margin {

            // Get terrain height at longhouse position
            let (height, _) = get_height_at(pos.x, pos.z, world_seed);

            let transform = Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_y(longhouse.rotation),
                Vec3::new(pos.x, height, pos.z),
            );

            // Convert mesh to flat vertex data
            let (vertices, indices) = flatten_longhouse_mesh(&longhouse.mesh);

            structures.push(VillageStructure {
                structure_type: VillageStructureType::Longhouse,
                transform,
                mesh_vertices: vertices,
                mesh_indices: indices,
            });
        }
    }

    // Add fire pits
    for fire_pit in &village.layout.fire_pits {
        let pos = fire_pit.position;

        if pos.x >= chunk_min_x && pos.x <= chunk_max_x &&
           pos.z >= chunk_min_z && pos.z <= chunk_max_z {

            let (height, _) = get_height_at(pos.x, pos.z, world_seed);

            let transform = Mat4::from_translation(Vec3::new(pos.x, height, pos.z));

            let mesh = generate_fire_pit(fire_pit);
            let (vertices, indices) = flatten_fire_pit_mesh(&mesh);

            structures.push(VillageStructure {
                structure_type: VillageStructureType::FirePit,
                transform,
                mesh_vertices: vertices,
                mesh_indices: indices,
            });
        }
    }

    // Add corn plants from fields
    for field in &village.layout.corn_fields {
        for mound_pos in &field.mounds {
            if mound_pos.x >= chunk_min_x && mound_pos.x <= chunk_max_x &&
               mound_pos.z >= chunk_min_z && mound_pos.z <= chunk_max_z {

                let (height, _) = get_height_at(mound_pos.x, mound_pos.z, world_seed);

                // Determine growth stage based on position (deterministic)
                let stage_seed = (mound_pos.x as u32).wrapping_mul(31).wrapping_add(mound_pos.z as u32);
                let stage = match stage_seed % 5 {
                    0 => CornGrowthStage::Sprout,
                    1 => CornGrowthStage::Young,
                    2 => CornGrowthStage::Growing,
                    3 => CornGrowthStage::Tasseling,
                    _ => CornGrowthStage::Mature,
                };

                let transform = Mat4::from_translation(Vec3::new(mound_pos.x, height, mound_pos.z));

                let mesh = generate_corn_plant(stage, stage_seed);
                let (vertices, indices) = flatten_corn_mesh(&mesh);

                structures.push(VillageStructure {
                    structure_type: VillageStructureType::CornPlant,
                    transform,
                    mesh_vertices: vertices,
                    mesh_indices: indices,
                });
            }
        }
    }

    structures
}

fn flatten_longhouse_mesh(mesh: &LonghouseMesh) -> (Vec<f32>, Vec<u32>) {
    let mut vertices = Vec::with_capacity(mesh.vertices.len() * 11);

    for v in &mesh.vertices {
        vertices.push(v.position[0]);
        vertices.push(v.position[1]);
        vertices.push(v.position[2]);
        vertices.push(v.normal[0]);
        vertices.push(v.normal[1]);
        vertices.push(v.normal[2]);
        vertices.push(v.uv[0]);
        vertices.push(v.uv[1]);
        vertices.push(v.color[0]);
        vertices.push(v.color[1]);
        vertices.push(v.color[2]);
    }

    (vertices, mesh.indices.clone())
}

fn flatten_fire_pit_mesh(mesh: &FirePitMesh) -> (Vec<f32>, Vec<u32>) {
    let mut vertices = Vec::with_capacity(mesh.vertices.len() * 11);

    for v in &mesh.vertices {
        vertices.push(v.position[0]);
        vertices.push(v.position[1]);
        vertices.push(v.position[2]);
        vertices.push(v.normal[0]);
        vertices.push(v.normal[1]);
        vertices.push(v.normal[2]);
        vertices.push(v.uv[0]);
        vertices.push(v.uv[1]);
        vertices.push(v.color[0]);
        vertices.push(v.color[1]);
        vertices.push(v.color[2]);
    }

    (vertices, mesh.indices.clone())
}

fn flatten_corn_mesh(mesh: &croatoan_procgen::CornPlantMesh) -> (Vec<f32>, Vec<u32>) {
    let mut vertices = Vec::with_capacity(mesh.vertices.len() * 11);

    for v in &mesh.vertices {
        vertices.push(v.position[0]);
        vertices.push(v.position[1]);
        vertices.push(v.position[2]);
        vertices.push(v.normal[0]);
        vertices.push(v.normal[1]);
        vertices.push(v.normal[2]);
        vertices.push(v.uv[0]);
        vertices.push(v.uv[1]);
        vertices.push(v.color[0]);
        vertices.push(v.color[1]);
        vertices.push(v.color[2]);
    }

    (vertices, mesh.indices.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_village_sites() {
        let sites = find_village_sites(
            12345,
            Vec3::new(-500.0, 0.0, -500.0),
            Vec3::new(500.0, 0.0, 500.0),
            5,
        );

        println!("Found {} village sites", sites.len());
        for site in &sites {
            println!("  Site at ({:.1}, {:.1}, {:.1})", site.x, site.y, site.z);
        }
    }

    #[test]
    fn test_generate_village() {
        let center = Vec3::new(100.0, 10.0, 100.0);
        let village = generate_world_village(center, 12345, 1);

        println!("Village '{}' with {} longhouses, {} NPCs",
                 village.layout.name,
                 village.layout.longhouses.len(),
                 village.layout.npcs.len());

        assert!(!village.layout.longhouses.is_empty());
        assert!(!village.layout.fire_pits.is_empty());
    }

    #[test]
    fn test_village_structures_for_chunk() {
        let center = Vec3::new(100.0, 10.0, 100.0);
        let village = generate_world_village(center, 12345, 1);

        let structures = get_village_structures_for_chunk(
            &village,
            64.0,
            64.0,
            64.0,
            12345,
        );

        println!("Found {} structures in chunk", structures.len());
        for s in &structures {
            println!("  {:?}", s.structure_type);
        }
    }
}
