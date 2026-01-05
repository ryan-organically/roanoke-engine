//! Pond and Lake Water Surface Mesh Generation
//!
//! Generates circular/organic water surface meshes for inland water bodies.
//! These are rendered separately from the ocean water system with calmer waves.

use glam::{Vec2, Vec3};
use crate::noise_util;

/// Water body type affects rendering style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterBodyType {
    Pond,       // Small, calm
    Lake,       // Larger, gentle ripples
    Wetland,    // Murky, very calm
    MarshPool,  // Tidal, brackish
}

impl From<u32> for WaterBodyType {
    fn from(v: u32) -> Self {
        match v {
            0 => WaterBodyType::Pond,
            1 => WaterBodyType::Lake,
            2 => WaterBodyType::Wetland,
            3 => WaterBodyType::MarshPool,
            _ => WaterBodyType::Pond,
        }
    }
}

/// A water body definition
#[derive(Debug, Clone)]
pub struct WaterBody {
    pub center: Vec2,
    pub radius: f32,
    pub depth: f32,
    pub water_type: WaterBodyType,
    pub water_level: f32, // Y position of water surface
}

/// Generated water mesh data
#[derive(Debug, Clone)]
pub struct WaterMeshData {
    pub positions: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
    pub center: Vec2,
    pub water_type: WaterBodyType,
}

/// Generate a circular water surface mesh for a pond/lake
pub fn generate_pond_mesh(body: &WaterBody, segments: u32, seed: u32) -> WaterMeshData {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices in concentric rings
    let rings = (segments / 2).max(4);
    let points_per_ring = segments;

    // Center vertex
    positions.push(Vec3::new(body.center.x, body.water_level, body.center.y));
    uvs.push(Vec2::new(0.5, 0.5));

    // Concentric rings from center to edge
    for ring in 1..=rings {
        let ring_t = ring as f32 / rings as f32;
        let ring_radius = body.radius * ring_t;

        for i in 0..points_per_ring {
            let angle = (i as f32 / points_per_ring as f32) * std::f32::consts::TAU;

            // Add organic edge variation using noise
            let noise_input = Vec2::new(
                body.center.x + angle.cos() * 100.0,
                body.center.y + angle.sin() * 100.0,
            );
            let edge_noise = if ring == rings {
                noise_util::fbm(noise_input * 0.05, 2, 2.0, 0.5, seed + 100) * (body.radius * 0.15)
            } else {
                0.0
            };

            let effective_radius = ring_radius + edge_noise;

            let x = body.center.x + angle.cos() * effective_radius;
            let z = body.center.y + angle.sin() * effective_radius;

            // Slight height variation for natural look (very subtle)
            let height_noise = noise_util::fbm(
                Vec2::new(x * 0.1, z * 0.1),
                1, 2.0, 0.5, seed + 200
            ) * 0.02;

            positions.push(Vec3::new(x, body.water_level + height_noise, z));

            // UVs for texture mapping (0-1 range centered)
            let u = 0.5 + (angle.cos() * ring_t * 0.5);
            let v = 0.5 + (angle.sin() * ring_t * 0.5);
            uvs.push(Vec2::new(u, v));
        }
    }

    // Generate triangle indices
    // Center fan
    for i in 0..points_per_ring {
        let next = (i + 1) % points_per_ring;
        indices.push(0); // Center
        indices.push(1 + i);
        indices.push(1 + next);
    }

    // Ring strips
    for ring in 0..(rings - 1) {
        let inner_start = 1 + ring * points_per_ring;
        let outer_start = 1 + (ring + 1) * points_per_ring;

        for i in 0..points_per_ring {
            let next = (i + 1) % points_per_ring;

            // Two triangles per quad
            indices.push(inner_start + i);
            indices.push(outer_start + i);
            indices.push(outer_start + next);

            indices.push(inner_start + i);
            indices.push(outer_start + next);
            indices.push(inner_start + next);
        }
    }

    WaterMeshData {
        positions,
        uvs,
        indices,
        center: body.center,
        water_type: body.water_type,
    }
}

/// Generate water meshes for all defined water bodies
pub fn generate_all_water_meshes(seed: u32) -> Vec<WaterMeshData> {
    let bodies = crate::mesh_gen::get_water_bodies();
    let mut meshes = Vec::with_capacity(bodies.len());

    for (center, radius, depth, water_type_id) in bodies {
        let water_type = WaterBodyType::from(water_type_id);

        // Water level is slightly below ground level at center
        // This is calculated from the pond carving depth
        let water_level = match water_type {
            WaterBodyType::Pond => -0.3,
            WaterBodyType::Lake => -0.5,
            WaterBodyType::Wetland => 0.2, // Almost at ground level
            WaterBodyType::MarshPool => 0.1,
        };

        let body = WaterBody {
            center,
            radius,
            depth,
            water_type,
            water_level,
        };

        // More segments for larger water bodies
        let segments = if radius > 25.0 { 48 } else if radius > 15.0 { 32 } else { 24 };

        meshes.push(generate_pond_mesh(&body, segments, seed));
    }

    meshes
}

/// Get water shader parameters for a water body type
pub fn get_water_params(water_type: WaterBodyType) -> WaterParams {
    match water_type {
        WaterBodyType::Pond => WaterParams {
            wave_amplitude: 0.02,
            wave_frequency: 0.5,
            deep_color: [0.08, 0.18, 0.25],     // Lighter, more refined
            shallow_color: [0.20, 0.40, 0.38],  // Crystal aqua
            foam_color: [0.85, 0.90, 0.88],
            turbidity: 0.1,                     // Crystal clear pond
            transparency_depth: 3.0,            // Deeper visibility
        },
        WaterBodyType::Lake => WaterParams {
            wave_amplitude: 0.05,
            wave_frequency: 0.3,
            deep_color: [0.04, 0.14, 0.22],     // Refined lake blue
            shallow_color: [0.18, 0.38, 0.42],  // Clear teal
            foam_color: [0.92, 0.94, 0.93],
            turbidity: 0.08,                    // Very clear alpine lake
            transparency_depth: 5.0,            // Deep visibility
        },
        WaterBodyType::Wetland => WaterParams {
            wave_amplitude: 0.01,
            wave_frequency: 0.2,
            deep_color: [0.10, 0.12, 0.08],
            shallow_color: [0.20, 0.25, 0.15],
            foam_color: [0.6, 0.55, 0.45],
            turbidity: 0.7,
            transparency_depth: 0.8,
        },
        WaterBodyType::MarshPool => WaterParams {
            wave_amplitude: 0.015,
            wave_frequency: 0.25,
            deep_color: [0.12, 0.15, 0.10],
            shallow_color: [0.25, 0.30, 0.20],
            foam_color: [0.7, 0.65, 0.55],
            turbidity: 0.6,
            transparency_depth: 1.0,
        },
    }
}

/// Water rendering parameters
#[derive(Debug, Clone, Copy)]
pub struct WaterParams {
    pub wave_amplitude: f32,
    pub wave_frequency: f32,
    pub deep_color: [f32; 3],
    pub shallow_color: [f32; 3],
    pub foam_color: [f32; 3],
    pub turbidity: f32,
    pub transparency_depth: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pond_mesh() {
        let body = WaterBody {
            center: Vec2::new(0.0, 0.0),
            radius: 10.0,
            depth: 2.0,
            water_type: WaterBodyType::Pond,
            water_level: -0.3,
        };

        let mesh = generate_pond_mesh(&body, 16, 12345);

        assert!(!mesh.positions.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.positions.len(), mesh.uvs.len());
    }

    #[test]
    fn test_generate_all_water_meshes() {
        let meshes = generate_all_water_meshes(12345);
        assert!(!meshes.is_empty(), "Should generate water meshes for all water bodies");
    }
}
