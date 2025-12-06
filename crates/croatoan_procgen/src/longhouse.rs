//! # Longhouse Procedural Generation
//!
//! This module generates Native American longhouse structures using procedural mesh generation.
//! Longhouses were the traditional dwellings of Iroquoian peoples, constructed from bark over
//! bent-pole frames.
//!
//! ## Quick Start
//!
//! ```rust
//! use croatoan_procgen::{LonghouseRecipe, generate_longhouse};
//!
//! let recipe = LonghouseRecipe::iroquoian_medium();
//! let mesh = generate_longhouse(&recipe);
//!
//! println!("Generated {} vertices, {} indices",
//!     mesh.vertices.len(),
//!     mesh.indices.len()
//! );
//! ```
//!
//! ## Architectural Styles
//!
//! - **Iroquoian**: Bark-covered with rounded ends (default)
//! - **Algonquian**: Similar but with dome-shaped ends
//! - **Coastal**: Plank-covered with flat ends (Pacific Northwest)
//!
//! ## Mesh Components
//!
//! Each longhouse mesh includes:
//! - Bent-pole frame arches (every 1.5m)
//! - Ridge pole and horizontal stringers
//! - Bark shell covering
//! - End walls with doorways
//! - Smoke holes in roof
//! - Interior stone hearths
//!
//! ## Sizing
//!
//! Longhouse length is determined by `family_units * 6.0` meters:
//! - Small (3 units): 18m long
//! - Medium (5 units): 30m long
//! - Large (8 units): 48m long

use glam::Vec3;

/// Longhouse architectural style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LonghouseStyle {
    Iroquoian,      // Bark-covered, rounded ends
    Algonquian,     // Bark/mat-covered, dome ends
    Coastal,        // Plank-covered, flat ends
}

/// Parameters for procedural longhouse generation
#[derive(Debug, Clone)]
pub struct LonghouseRecipe {
    pub style: LonghouseStyle,
    pub family_units: u32,          // 2-10, determines length
    pub width: f32,                 // 6.0-7.0 meters typical
    pub height: f32,                // 5.0-6.0 meters at ridge
    pub seed: u32,
}

impl Default for LonghouseRecipe {
    fn default() -> Self {
        Self::iroquoian_medium()
    }
}

impl LonghouseRecipe {
    pub fn iroquoian_medium() -> Self {
        LonghouseRecipe {
            style: LonghouseStyle::Iroquoian,
            family_units: 5,
            width: 6.5,
            height: 5.5,
            seed: 0,
        }
    }

    pub fn small_clan_house() -> Self {
        LonghouseRecipe {
            style: LonghouseStyle::Iroquoian,
            family_units: 3,
            width: 6.0,
            height: 5.0,
            seed: 0,
        }
    }

    pub fn large_council_house() -> Self {
        LonghouseRecipe {
            style: LonghouseStyle::Iroquoian,
            family_units: 8,
            width: 7.0,
            height: 6.0,
            seed: 0,
        }
    }

    /// Calculate total length based on family units
    pub fn length(&self) -> f32 {
        self.family_units as f32 * 6.0
    }

    /// Number of doorways (ends + middle for large houses)
    pub fn door_count(&self) -> u32 {
        if self.family_units > 5 { 3 } else { 2 }
    }

    /// Number of smoke holes in roof
    pub fn smoke_hole_count(&self) -> u32 {
        (self.family_units / 2).max(1)
    }

    /// Interior hearth count
    pub fn hearth_count(&self) -> u32 {
        self.family_units / 2
    }
}

/// Vertex data for longhouse mesh
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LonghouseVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
}

/// Generated longhouse mesh with metadata
#[derive(Debug, Clone)]
pub struct LonghouseMesh {
    pub vertices: Vec<LonghouseVertex>,
    pub indices: Vec<u32>,
    pub smoke_hole_positions: Vec<Vec3>,
    pub door_positions: Vec<Vec3>,
    pub hearth_positions: Vec<Vec3>,
    pub bounds: (Vec3, Vec3), // AABB min/max
}

// Color palette
const FRAME_POLE_COLOR: [f32; 3] = [0.55, 0.40, 0.25];
const ELM_BARK_COLOR: [f32; 3] = [0.45, 0.35, 0.28];
const BIRCH_BARK_COLOR: [f32; 3] = [0.75, 0.72, 0.65];
const INTERIOR_BARK_COLOR: [f32; 3] = [0.50, 0.38, 0.30];
const SMOKE_STAIN_COLOR: [f32; 3] = [0.30, 0.27, 0.25];
const DOOR_FRAME_COLOR: [f32; 3] = [0.40, 0.30, 0.22];

/// Generate a longhouse mesh from a recipe
pub fn generate_longhouse(recipe: &LonghouseRecipe) -> LonghouseMesh {
    let mut builder = MeshBuilder::new();

    let length = recipe.length();
    let half_length = length * 0.5;
    let half_width = recipe.width * 0.5;

    // RNG for variation
    let mut rng_state = recipe.seed as u64;
    let mut random = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (rng_state >> 32) as f32 / u32::MAX as f32
    };

    // 1. Generate frame poles (bent saplings forming arches)
    let arch_spacing = 1.5;
    let arch_count = (length / arch_spacing) as u32 + 1;

    for i in 0..arch_count {
        let z = -half_length + i as f32 * arch_spacing;
        generate_arch_frame(&mut builder, z, recipe.width, recipe.height, &mut random);
    }

    // 2. Ridge pole (horizontal along top)
    builder.add_cylinder(
        Vec3::new(0.0, recipe.height - 0.1, -half_length),
        Vec3::new(0.0, recipe.height - 0.1, half_length),
        0.06,
        FRAME_POLE_COLOR,
    );

    // 3. Horizontal stringers (3 on each side)
    for side in [-1.0_f32, 1.0] {
        for h in [0.3, 0.5, 0.7] {
            let y = recipe.height * h;
            let x = half_width * side * (1.0 - (h - 0.5).abs() * 0.3);
            builder.add_cylinder(
                Vec3::new(x, y, -half_length),
                Vec3::new(x, y, half_length),
                0.04,
                FRAME_POLE_COLOR,
            );
        }
    }

    // 4. Bark covering (curved shell panels)
    generate_bark_shell(&mut builder, recipe, &mut random);

    // 5. End walls
    match recipe.style {
        LonghouseStyle::Iroquoian => {
            generate_rounded_end(&mut builder, -half_length, recipe, false);
            generate_rounded_end(&mut builder, half_length, recipe, true);
        }
        LonghouseStyle::Algonquian => {
            generate_dome_end(&mut builder, -half_length, recipe, false);
            generate_dome_end(&mut builder, half_length, recipe, true);
        }
        LonghouseStyle::Coastal => {
            generate_flat_end(&mut builder, -half_length, recipe, false);
            generate_flat_end(&mut builder, half_length, recipe, true);
        }
    }

    // 6. Doorways
    let mut door_positions = Vec::new();

    // End doors
    door_positions.push(Vec3::new(0.0, 0.0, -half_length));
    door_positions.push(Vec3::new(0.0, 0.0, half_length));

    // Center door for large houses
    if recipe.door_count() > 2 {
        door_positions.push(Vec3::new(half_width + 0.1, 0.0, 0.0));
        generate_side_doorway(&mut builder, Vec3::new(half_width, 0.0, 0.0), recipe);
    }

    // 7. Smoke holes
    let mut smoke_positions = Vec::new();
    let hole_spacing = length / (recipe.smoke_hole_count() + 1) as f32;

    for i in 0..recipe.smoke_hole_count() {
        let z = -half_length + hole_spacing * (i + 1) as f32;
        smoke_positions.push(Vec3::new(0.0, recipe.height, z));
        generate_smoke_hole(&mut builder, z, recipe);
    }

    // 8. Interior hearths (flat stone circles on ground)
    let mut hearth_positions = Vec::new();
    let hearth_spacing = length / (recipe.hearth_count() + 1) as f32;

    for i in 0..recipe.hearth_count() {
        let z = -half_length + hearth_spacing * (i + 1) as f32;
        hearth_positions.push(Vec3::new(0.0, 0.05, z));
        generate_hearth(&mut builder, Vec3::new(0.0, 0.0, z));
    }

    // Calculate bounds
    let bounds = (
        Vec3::new(-half_width - 0.5, 0.0, -half_length - 0.5),
        Vec3::new(half_width + 0.5, recipe.height + 0.5, half_length + 0.5),
    );

    LonghouseMesh {
        vertices: builder.vertices,
        indices: builder.indices,
        smoke_hole_positions: smoke_positions,
        door_positions,
        hearth_positions,
        bounds,
    }
}

fn generate_arch_frame(builder: &mut MeshBuilder, z: f32, width: f32, height: f32, random: &mut impl FnMut() -> f32) {
    let half_width = width * 0.5;
    let segments = 8;
    let pole_radius = 0.04 + random() * 0.02;

    // Create arch from ground on one side, over top, to ground on other side
    let mut points = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = std::f32::consts::PI * t;

        // Parabolic arch shape
        let x = -half_width + width * t;
        let y = height * (1.0 - (2.0 * t - 1.0).powi(2)) * 0.95 + 0.1;

        // Add slight variation
        let y = y + (random() - 0.5) * 0.1;

        points.push(Vec3::new(x, y.max(0.0), z));
    }

    // Connect points with cylinders
    for i in 0..points.len() - 1 {
        builder.add_cylinder(points[i], points[i + 1], pole_radius, FRAME_POLE_COLOR);
    }
}

fn generate_bark_shell(builder: &mut MeshBuilder, recipe: &LonghouseRecipe, random: &mut impl FnMut() -> f32) {
    let length = recipe.length();
    let half_length = length * 0.5;
    let half_width = recipe.width * 0.5;

    let segments_around = 12;
    let segments_along = (length / 2.0) as u32;

    // Generate curved surface
    for j in 0..segments_along {
        let z0 = -half_length + (j as f32 / segments_along as f32) * length;
        let z1 = -half_length + ((j + 1) as f32 / segments_along as f32) * length;

        for i in 0..segments_around {
            let t0 = i as f32 / segments_around as f32;
            let t1 = (i + 1) as f32 / segments_around as f32;

            // Parabolic cross-section
            let (x0, y0) = arch_profile(t0, half_width, recipe.height);
            let (x1, y1) = arch_profile(t1, half_width, recipe.height);

            // Slightly vary colors for bark panel effect
            let color_var = 0.9 + random() * 0.2;
            let color = [
                ELM_BARK_COLOR[0] * color_var,
                ELM_BARK_COLOR[1] * color_var,
                ELM_BARK_COLOR[2] * color_var,
            ];

            // Create quad
            let v0 = Vec3::new(x0, y0, z0);
            let v1 = Vec3::new(x1, y1, z0);
            let v2 = Vec3::new(x1, y1, z1);
            let v3 = Vec3::new(x0, y0, z1);

            // Calculate normal (pointing outward)
            let center_x = (x0 + x1) * 0.5;
            let center_y = (y0 + y1) * 0.5;
            let normal = Vec3::new(center_x, center_y - recipe.height * 0.5, 0.0).normalize();

            builder.add_quad(v0, v1, v2, v3, normal, color);
        }
    }
}

fn arch_profile(t: f32, half_width: f32, height: f32) -> (f32, f32) {
    let x = -half_width + half_width * 2.0 * t;
    let normalized_x = (t - 0.5) * 2.0; // -1 to 1
    let y = height * (1.0 - normalized_x.powi(2)) * 0.95;
    (x, y.max(0.1))
}

fn generate_rounded_end(builder: &mut MeshBuilder, z: f32, recipe: &LonghouseRecipe, flip: bool) {
    let half_width = recipe.width * 0.5;
    let segments = 8;
    let normal = if flip { Vec3::Z } else { Vec3::NEG_Z };

    // Create rounded end wall with door opening
    let door_width = 0.8;
    let door_height = 1.6;

    for i in 0..segments {
        let t0 = i as f32 / segments as f32;
        let t1 = (i + 1) as f32 / segments as f32;

        let (x0, y0) = arch_profile(t0, half_width, recipe.height);
        let (x1, y1) = arch_profile(t1, half_width, recipe.height);

        // Skip door area at center bottom
        let in_door = x0.abs() < door_width && y0 < door_height && x1.abs() < door_width && y1 < door_height;

        if !in_door {
            // Triangle from center bottom to edge
            let center = Vec3::new(0.0, 0.1, z);
            let p0 = Vec3::new(x0, y0, z);
            let p1 = Vec3::new(x1, y1, z);

            if flip {
                builder.add_tri(center, p1, p0, normal, ELM_BARK_COLOR);
            } else {
                builder.add_tri(center, p0, p1, normal, ELM_BARK_COLOR);
            }
        }
    }

    // Door frame
    let door_z = if flip { z + 0.05 } else { z - 0.05 };
    builder.add_box(
        Vec3::new(0.0, door_height * 0.5, door_z),
        Vec3::new(door_width + 0.2, door_height + 0.1, 0.15),
        DOOR_FRAME_COLOR,
    );
}

fn generate_dome_end(builder: &mut MeshBuilder, z: f32, recipe: &LonghouseRecipe, flip: bool) {
    // Similar to rounded but more spherical
    generate_rounded_end(builder, z, recipe, flip);
}

fn generate_flat_end(builder: &mut MeshBuilder, z: f32, recipe: &LonghouseRecipe, flip: bool) {
    let half_width = recipe.width * 0.5;
    let normal = if flip { Vec3::Z } else { Vec3::NEG_Z };
    let door_width = 0.8;
    let door_height = 1.6;

    // Simple flat wall with door
    // Left panel
    builder.add_quad(
        Vec3::new(-half_width, 0.0, z),
        Vec3::new(-door_width * 0.5, 0.0, z),
        Vec3::new(-door_width * 0.5, recipe.height, z),
        Vec3::new(-half_width, recipe.height, z),
        normal,
        ELM_BARK_COLOR,
    );

    // Right panel
    builder.add_quad(
        Vec3::new(door_width * 0.5, 0.0, z),
        Vec3::new(half_width, 0.0, z),
        Vec3::new(half_width, recipe.height, z),
        Vec3::new(door_width * 0.5, recipe.height, z),
        normal,
        ELM_BARK_COLOR,
    );

    // Above door
    builder.add_quad(
        Vec3::new(-door_width * 0.5, door_height, z),
        Vec3::new(door_width * 0.5, door_height, z),
        Vec3::new(door_width * 0.5, recipe.height, z),
        Vec3::new(-door_width * 0.5, recipe.height, z),
        normal,
        ELM_BARK_COLOR,
    );
}

fn generate_side_doorway(builder: &mut MeshBuilder, pos: Vec3, recipe: &LonghouseRecipe) {
    let door_width = 0.8;
    let door_height = 1.6;

    // Frame around door
    builder.add_box(
        Vec3::new(pos.x + 0.05, door_height * 0.5, pos.z),
        Vec3::new(0.15, door_height + 0.1, door_width + 0.2),
        DOOR_FRAME_COLOR,
    );
}

fn generate_smoke_hole(builder: &mut MeshBuilder, z: f32, recipe: &LonghouseRecipe) {
    // Smoke hole is a rectangular opening in the roof
    // We don't actually cut geometry, just add a dark frame
    let hole_width = 0.6;
    let hole_length = 0.8;

    builder.add_box(
        Vec3::new(0.0, recipe.height + 0.02, z),
        Vec3::new(hole_width + 0.1, 0.08, hole_length + 0.1),
        SMOKE_STAIN_COLOR,
    );
}

fn generate_hearth(builder: &mut MeshBuilder, pos: Vec3) {
    // Stone ring hearth
    let radius = 0.6;
    let stone_count = 12;

    for i in 0..stone_count {
        let angle = (i as f32 / stone_count as f32) * std::f32::consts::TAU;
        let x = pos.x + radius * angle.cos();
        let z = pos.z + radius * angle.sin();

        // Individual stones
        builder.add_box(
            Vec3::new(x, 0.08, z),
            Vec3::new(0.15, 0.16, 0.12),
            [0.4, 0.4, 0.42], // Gray stone
        );
    }

    // Ash/char center
    builder.add_box(
        Vec3::new(pos.x, 0.02, pos.z),
        Vec3::new(radius * 1.5, 0.04, radius * 1.5),
        [0.15, 0.12, 0.10], // Dark ash
    );
}

// --- Mesh Builder ---

struct MeshBuilder {
    vertices: Vec<LonghouseVertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_box(&mut self, center: Vec3, size: Vec3, color: [f32; 3]) {
        let half = size * 0.5;

        let corners = [
            Vec3::new(-half.x, -half.y,  half.z),
            Vec3::new( half.x, -half.y,  half.z),
            Vec3::new( half.x,  half.y,  half.z),
            Vec3::new(-half.x,  half.y,  half.z),
            Vec3::new(-half.x, -half.y, -half.z),
            Vec3::new( half.x, -half.y, -half.z),
            Vec3::new( half.x,  half.y, -half.z),
            Vec3::new(-half.x,  half.y, -half.z),
        ];

        // Front
        self.add_quad(center + corners[0], center + corners[1], center + corners[2], center + corners[3], Vec3::Z, color);
        // Back
        self.add_quad(center + corners[5], center + corners[4], center + corners[7], center + corners[6], Vec3::NEG_Z, color);
        // Top
        self.add_quad(center + corners[3], center + corners[2], center + corners[6], center + corners[7], Vec3::Y, color);
        // Bottom
        self.add_quad(center + corners[4], center + corners[5], center + corners[1], center + corners[0], Vec3::NEG_Y, color);
        // Right
        self.add_quad(center + corners[1], center + corners[5], center + corners[6], center + corners[2], Vec3::X, color);
        // Left
        self.add_quad(center + corners[4], center + corners[0], center + corners[3], center + corners[7], Vec3::NEG_X, color);
    }

    fn add_quad(&mut self, v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3, color: [f32; 3]) {
        let base = self.vertices.len() as u32;

        self.vertices.push(LonghouseVertex {
            position: v0.to_array(),
            normal: normal.to_array(),
            uv: [0.0, 1.0],
            color
        });
        self.vertices.push(LonghouseVertex {
            position: v1.to_array(),
            normal: normal.to_array(),
            uv: [1.0, 1.0],
            color
        });
        self.vertices.push(LonghouseVertex {
            position: v2.to_array(),
            normal: normal.to_array(),
            uv: [1.0, 0.0],
            color
        });
        self.vertices.push(LonghouseVertex {
            position: v3.to_array(),
            normal: normal.to_array(),
            uv: [0.0, 0.0],
            color
        });

        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn add_tri(&mut self, v0: Vec3, v1: Vec3, v2: Vec3, normal: Vec3, color: [f32; 3]) {
        let base = self.vertices.len() as u32;

        self.vertices.push(LonghouseVertex {
            position: v0.to_array(),
            normal: normal.to_array(),
            uv: [0.5, 0.0],
            color
        });
        self.vertices.push(LonghouseVertex {
            position: v1.to_array(),
            normal: normal.to_array(),
            uv: [0.0, 1.0],
            color
        });
        self.vertices.push(LonghouseVertex {
            position: v2.to_array(),
            normal: normal.to_array(),
            uv: [1.0, 1.0],
            color
        });

        self.indices.extend_from_slice(&[base, base + 1, base + 2]);
    }

    fn add_cylinder(&mut self, start: Vec3, end: Vec3, radius: f32, color: [f32; 3]) {
        let segments = 6;
        let direction = (end - start).normalize();

        // Find perpendicular vectors
        let up = if direction.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let right = direction.cross(up).normalize();
        let forward = right.cross(direction).normalize();

        // Generate cylinder vertices
        let mut ring_start = Vec::with_capacity(segments);
        let mut ring_end = Vec::with_capacity(segments);

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let offset = right * angle.cos() * radius + forward * angle.sin() * radius;
            ring_start.push(start + offset);
            ring_end.push(end + offset);
        }

        // Create quads for cylinder surface
        for i in 0..segments {
            let next = (i + 1) % segments;
            let normal = ((ring_start[i] - start) + (ring_start[next] - start)).normalize();

            self.add_quad(
                ring_start[i],
                ring_start[next],
                ring_end[next],
                ring_end[i],
                normal,
                color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longhouse_generation() {
        let recipe = LonghouseRecipe::iroquoian_medium();
        let mesh = generate_longhouse(&recipe);

        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.door_positions.len(), 2); // Two end doors
        assert!(mesh.smoke_hole_positions.len() >= 1);
        assert!(mesh.hearth_positions.len() >= 1);
    }

    #[test]
    fn test_large_longhouse() {
        let recipe = LonghouseRecipe::large_council_house();
        let mesh = generate_longhouse(&recipe);

        assert_eq!(recipe.door_count(), 3); // Has side door
        assert!(mesh.door_positions.len() >= 2);
    }
}
