//! Campfire system - Player-placeable campfires with ember particles and flickering light
//!
//! Campfires provide warmth and light in the wilderness. They consist of:
//! - A ring of pebbles (~1m diameter, 8-10 stones)
//! - Glowing ember bed in the center
//! - Rising ember particles
//! - Flickering point light that illuminates surroundings

use glam::{Vec3, Mat4};
use std::collections::HashMap;

/// Unique identifier for a campfire
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CampfireId(pub u64);

/// A placed campfire in the world
#[derive(Debug, Clone)]
pub struct Campfire {
    pub id: CampfireId,
    pub position: Vec3,        // World position (center of fire pit)
    pub rotation: f32,         // Y-axis rotation in radians
    pub radius: f32,           // Pebble ring radius (~0.5m for 1m diameter)
    pub ember_intensity: f32,  // 0.0-1.0, controls glow + particle spawn rate
    pub is_lit: bool,          // Can be extinguished
    pub light_phase: f32,      // For flicker animation (accumulates over time)
    pub base_intensity: f32,   // Base intensity before flicker applied
}

impl Campfire {
    /// Create a new campfire at the given position
    pub fn new(id: CampfireId, position: Vec3, rotation: f32) -> Self {
        Self {
            id,
            position,
            rotation,
            radius: 0.5,           // 1m diameter ring
            ember_intensity: 1.0,  // Full brightness
            is_lit: true,
            light_phase: 0.0,
            base_intensity: 1.0,
        }
    }

    /// Get the current flickering light intensity
    pub fn light_intensity(&self) -> f32 {
        if !self.is_lit {
            return 0.0;
        }
        self.ember_intensity
    }

    /// Get the warm orange color for the campfire light
    pub fn light_color() -> [f32; 3] {
        [1.0, 0.5, 0.15]  // Warm orange, slightly more saturated than muzzle flash
    }

    /// Get the effective light radius
    pub fn light_radius() -> f32 {
        12.0  // Campfire light reaches ~12 units
    }
}

/// Light data for shader uniforms
#[derive(Debug, Clone, Copy)]
pub struct CampfireLightData {
    pub position: Vec3,
    pub intensity: f32,
}

/// Manages all campfires in the world
#[derive(Debug, Clone, Default)]
pub struct CampfireManager {
    campfires: HashMap<CampfireId, Campfire>,
    next_id: u64,
}

impl CampfireManager {
    /// Create a new empty campfire manager
    pub fn new() -> Self {
        Self {
            campfires: HashMap::new(),
            next_id: 1,
        }
    }

    /// Place a new campfire at the given position
    pub fn place_campfire(&mut self, position: Vec3, rotation: f32) -> CampfireId {
        let id = CampfireId(self.next_id);
        self.next_id += 1;

        let campfire = Campfire::new(id, position, rotation);
        log::info!("[CAMPFIRE] Placed campfire {} at {:?}", id.0, position);
        self.campfires.insert(id, campfire);
        id
    }

    /// Remove a campfire by ID
    pub fn remove_campfire(&mut self, id: CampfireId) -> Option<Campfire> {
        let result = self.campfires.remove(&id);
        if result.is_some() {
            log::info!("[CAMPFIRE] Removed campfire {}", id.0);
        }
        result
    }

    /// Check if a campfire can be placed at the given position
    /// Returns false if too close to another campfire
    pub fn can_place_at(&self, position: Vec3) -> bool {
        const MIN_SPACING: f32 = 3.0;  // Minimum 3m between campfires

        for campfire in self.campfires.values() {
            let dist = (campfire.position - position).length();
            if dist < MIN_SPACING {
                return false;
            }
        }
        true
    }

    /// Get all campfires near a position within a given radius
    pub fn campfires_near(&self, position: Vec3, radius: f32) -> Vec<&Campfire> {
        self.campfires
            .values()
            .filter(|c| (c.position - position).length() <= radius)
            .collect()
    }

    /// Get mutable reference to all campfires near a position
    pub fn campfires_near_mut(&mut self, position: Vec3, radius: f32) -> Vec<&mut Campfire> {
        self.campfires
            .values_mut()
            .filter(|c| (c.position - position).length() <= radius)
            .collect()
    }

    /// Update all campfires (flicker animation)
    pub fn update(&mut self, delta: f32) {
        for campfire in self.campfires.values_mut() {
            if campfire.is_lit {
                // Advance flicker phase
                campfire.light_phase += delta * 5.0;

                // Multi-frequency flicker for organic feel
                let flicker = 0.8
                    + 0.1 * campfire.light_phase.sin()
                    + 0.05 * (campfire.light_phase * 2.3).sin()
                    + 0.05 * (campfire.light_phase * 3.7).sin();

                campfire.ember_intensity = (campfire.base_intensity * flicker).clamp(0.3, 1.0);
            }
        }
    }

    /// Get light data for all visible campfires (for shader uniforms)
    /// Returns up to max_count campfires, sorted by distance
    pub fn get_light_data(&self, camera_pos: Vec3, max_distance: f32, max_count: usize) -> Vec<CampfireLightData> {
        let mut lights: Vec<_> = self.campfires
            .values()
            .filter(|c| c.is_lit && (c.position - camera_pos).length() <= max_distance)
            .map(|c| (c, (c.position - camera_pos).length()))
            .collect();

        // Sort by distance (closest first for priority)
        lights.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        lights
            .into_iter()
            .take(max_count)
            .map(|(c, _)| CampfireLightData {
                position: c.position,
                intensity: c.light_intensity(),
            })
            .collect()
    }

    /// Get all campfires (for rendering)
    pub fn all_campfires(&self) -> impl Iterator<Item = &Campfire> {
        self.campfires.values()
    }

    /// Get campfire count
    pub fn count(&self) -> usize {
        self.campfires.len()
    }

    /// Toggle a campfire's lit state
    pub fn toggle_campfire(&mut self, id: CampfireId) {
        if let Some(campfire) = self.campfires.get_mut(&id) {
            campfire.is_lit = !campfire.is_lit;
            log::info!("[CAMPFIRE] Campfire {} is now {}", id.0, if campfire.is_lit { "lit" } else { "extinguished" });
        }
    }
}

// ============================================================================
// CAMPFIRE MESH GENERATION
// ============================================================================

/// Vertex data for campfire mesh (pebble ring + ember bed)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CampfireVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

/// Generated mesh data for a campfire
pub struct CampfireMesh {
    pub vertices: Vec<CampfireVertex>,
    pub indices: Vec<u32>,
}

impl CampfireMesh {
    /// Get mesh data in format compatible with TreePipeline
    pub fn to_tree_mesh_data(&self) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
        let positions: Vec<[f32; 3]> = self.vertices.iter().map(|v| v.position).collect();
        let normals: Vec<[f32; 3]> = self.vertices.iter().map(|v| v.normal).collect();
        // Generate simple UVs (not used for solid color, but required by pipeline)
        let uvs: Vec<[f32; 2]> = self.vertices.iter().map(|_| [0.5, 0.5]).collect();
        (positions, normals, uvs, self.indices.clone())
    }

    /// Generate mesh for a campfire
    pub fn generate(campfire: &Campfire) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Determine number of pebbles (8-10 based on ID for variation)
        let pebble_count = 8 + (campfire.id.0 % 3) as usize;
        let ring_radius = campfire.radius;

        // Pebble colors (gray stone variations)
        let stone_colors = [
            [0.35, 0.32, 0.30],
            [0.38, 0.35, 0.32],
            [0.32, 0.30, 0.28],
            [0.40, 0.36, 0.33],
        ];

        // Generate pebbles in a ring
        for i in 0..pebble_count {
            let angle = (i as f32 / pebble_count as f32) * std::f32::consts::TAU + campfire.rotation;
            let x = ring_radius * angle.cos();
            let z = ring_radius * angle.sin();

            // Vary pebble size (0.06-0.10m)
            let size_variation = ((campfire.id.0 + i as u64) % 100) as f32 / 2500.0;
            let pebble_size = 0.06 + size_variation;

            // Slight height variation
            let height_var = ((campfire.id.0 * 7 + i as u64 * 13) % 50) as f32 / 500.0;

            let pebble_pos = Vec3::new(
                campfire.position.x + x,
                campfire.position.y + pebble_size * 0.4 + height_var,
                campfire.position.z + z,
            );

            let color = stone_colors[i % stone_colors.len()];

            // Add box for each pebble
            add_box(
                &mut vertices,
                &mut indices,
                pebble_pos,
                Vec3::new(pebble_size, pebble_size * 0.7, pebble_size),
                color,
            );
        }

        // Add ember bed in center (dark, flat disc)
        let ember_bed_radius = ring_radius - 0.12;
        let ember_color = [0.12, 0.08, 0.06];  // Very dark brown/charcoal
        add_disc(
            &mut vertices,
            &mut indices,
            campfire.position + Vec3::new(0.0, 0.02, 0.0),  // Slightly above ground
            ember_bed_radius,
            ember_color,
            12,  // 12 segments for smooth circle
        );

        Self { vertices, indices }
    }
}

/// Add a box (cuboid) to the mesh
fn add_box(
    vertices: &mut Vec<CampfireVertex>,
    indices: &mut Vec<u32>,
    center: Vec3,
    half_extents: Vec3,
    color: [f32; 3],
) {
    let base_idx = vertices.len() as u32;

    // 8 corners of the box
    let corners = [
        Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new( half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new( half_extents.x, -half_extents.y,  half_extents.z),
        Vec3::new(-half_extents.x, -half_extents.y,  half_extents.z),
        Vec3::new(-half_extents.x,  half_extents.y, -half_extents.z),
        Vec3::new( half_extents.x,  half_extents.y, -half_extents.z),
        Vec3::new( half_extents.x,  half_extents.y,  half_extents.z),
        Vec3::new(-half_extents.x,  half_extents.y,  half_extents.z),
    ];

    // 6 faces with normals
    let faces = [
        ([0, 1, 2, 3], [0.0, -1.0, 0.0]),  // Bottom
        ([4, 7, 6, 5], [0.0,  1.0, 0.0]),  // Top
        ([0, 4, 5, 1], [0.0, 0.0, -1.0]),  // Front
        ([2, 6, 7, 3], [0.0, 0.0,  1.0]),  // Back
        ([0, 3, 7, 4], [-1.0, 0.0, 0.0]),  // Left
        ([1, 5, 6, 2], [ 1.0, 0.0, 0.0]),  // Right
    ];

    for (face_indices, normal) in faces.iter() {
        let face_base = vertices.len() as u32;

        for &idx in face_indices.iter() {
            let pos = center + corners[idx];
            vertices.push(CampfireVertex {
                position: pos.to_array(),
                normal: *normal,
                color,
            });
        }

        // Two triangles per face
        indices.extend_from_slice(&[
            face_base, face_base + 1, face_base + 2,
            face_base, face_base + 2, face_base + 3,
        ]);
    }
}

/// Add a flat disc to the mesh
fn add_disc(
    vertices: &mut Vec<CampfireVertex>,
    indices: &mut Vec<u32>,
    center: Vec3,
    radius: f32,
    color: [f32; 3],
    segments: u32,
) {
    let base_idx = vertices.len() as u32;
    let normal = [0.0, 1.0, 0.0];  // Pointing up

    // Center vertex
    vertices.push(CampfireVertex {
        position: center.to_array(),
        normal,
        color,
    });

    // Ring vertices
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let pos = center + Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin());
        vertices.push(CampfireVertex {
            position: pos.to_array(),
            normal,
            color,
        });
    }

    // Triangle fan
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.extend_from_slice(&[
            base_idx,          // Center
            base_idx + 1 + i,  // Current
            base_idx + 1 + next,  // Next
        ]);
    }
}

/// Get the model matrix for a campfire (for instanced rendering)
pub fn campfire_model_matrix(campfire: &Campfire) -> Mat4 {
    Mat4::from_translation(campfire.position)
        * Mat4::from_rotation_y(campfire.rotation)
}
