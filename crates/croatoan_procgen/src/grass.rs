use glam::{Vec2, Vec3};
use noise::{NoiseFn, Perlin};
use std::f32::consts::PI;

/// Recipe for generating grass blades
#[derive(Debug, Clone)]
pub struct GrassBladeRecipe {
    pub height_range: (f32, f32),
    pub blade_segments: u32,
    pub curve_factor: f32,
    pub width_base: f32,
    pub width_tip: f32,
    pub color_base: [f32; 3],
    pub color_tip: [f32; 3],
}

impl Default for GrassBladeRecipe {
    fn default() -> Self {
        Self {
            height_range: (0.3, 0.8),
            blade_segments: 4,
            curve_factor: 0.3,
            width_base: 0.04,
            width_tip: 0.01,
            color_base: [0.2, 0.5, 0.1],
            color_tip: [0.3, 0.7, 0.2],
        }
    }
}

/// Single grass blade mesh data
pub struct GrassBlade {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Generates a single procedural grass blade
///
/// Uses a curved ribbon with segments, tapering from base to tip
/// seed controls variation in height, curve direction, etc.
pub fn generate_grass_blade(recipe: &GrassBladeRecipe, seed: u32, base_pos: Vec3) -> GrassBlade {
    let noise = Perlin::new(seed);

    // Use noise to vary blade properties
    let noise_val = noise.get([base_pos.x as f64 * 10.0, base_pos.z as f64 * 10.0]) as f32;
    let height = lerp(recipe.height_range.0, recipe.height_range.1, (noise_val + 1.0) * 0.5);

    // Random curve direction
    let curve_angle = noise.get([base_pos.x as f64 * 7.3, base_pos.z as f64 * 7.3]) as f32 * PI;
    let curve_dir = Vec2::new(curve_angle.cos(), curve_angle.sin());

    let segments = recipe.blade_segments;
    let vertex_count = ((segments + 1) * 2) as usize;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut indices = Vec::new();

    // Generate vertices along the blade
    for i in 0..=segments {
        let t = i as f32 / segments as f32;

        // Curve the blade (quadratic)
        let curve_offset = t * t * recipe.curve_factor;
        let local_x = curve_dir.x * curve_offset;
        let local_z = curve_dir.y * curve_offset;
        let local_y = t * height;

        // Width tapers from base to tip
        let width = lerp(recipe.width_base, recipe.width_tip, t);

        // Color gradient from base to tip
        let color = lerp_color(recipe.color_base, recipe.color_tip, t);

        // Create two vertices (left and right edge)
        let perpendicular = Vec2::new(-curve_dir.y, curve_dir.x);
        let left_offset = perpendicular * width * 0.5;
        let right_offset = perpendicular * -width * 0.5;

        // Left vertex
        positions.push([
            base_pos.x + local_x + left_offset.x,
            base_pos.y + local_y,
            base_pos.z + local_z + left_offset.y,
        ]);
        colors.push(color);

        // Right vertex
        positions.push([
            base_pos.x + local_x + right_offset.x,
            base_pos.y + local_y,
            base_pos.z + local_z + right_offset.y,
        ]);
        colors.push(color);
    }

    // Generate indices for triangles
    for i in 0..segments {
        let base_idx = i * 2;

        // First triangle
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 1);

        // Second triangle
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    GrassBlade {
        positions,
        colors,
        indices,
    }
}

/// Generate a patch of grass blades for a terrain chunk
///
/// density: blades per square unit
/// biome_filter: function to determine if grass should spawn at location
pub fn generate_grass_patch(
    recipe: &GrassBladeRecipe,
    seed: u32,
    chunk_offset: (f32, f32),
    chunk_size: f32,
    density: f32,
    terrain_height_fn: impl Fn(f32, f32) -> f32,
    biome_filter: impl Fn(f32, f32) -> bool,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let noise = Perlin::new(seed + 999);

    let blade_count = (chunk_size * chunk_size * density) as u32;
    let mut all_positions = Vec::new();
    let mut all_colors = Vec::new();
    let mut all_indices = Vec::new();

    for i in 0..blade_count {
        // Pseudo-random position within chunk
        let rand_x = noise.get([i as f64 * 0.1, 0.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 100.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = chunk_offset.0 + local_x;
        let world_z = chunk_offset.1 + local_z;

        // Check if this biome supports grass
        if !biome_filter(world_x, world_z) {
            continue;
        }

        // Get terrain height
        let world_y = terrain_height_fn(world_x, world_z);

        let base_pos = Vec3::new(world_x, world_y, world_z);
        let blade = generate_grass_blade(recipe, seed + i, base_pos);

        // Append to combined mesh
        let vertex_offset = all_positions.len() as u32;
        all_positions.extend(blade.positions);
        all_colors.extend(blade.colors);
        all_indices.extend(blade.indices.iter().map(|idx| idx + vertex_offset));
    }

    (all_positions, all_colors, all_indices)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_blade_generation() {
        let recipe = GrassBladeRecipe::default();
        let blade = generate_grass_blade(&recipe, 42, Vec3::ZERO);

        // Should have (segments + 1) * 2 vertices
        assert_eq!(blade.positions.len(), 10);
        assert_eq!(blade.colors.len(), 10);

        // Should have segments * 2 triangles * 3 indices
        assert_eq!(blade.indices.len(), 24);
    }

    #[test]
    fn test_grass_patch() {
        let recipe = GrassBladeRecipe::default();
        let (positions, colors, indices) = generate_grass_patch(
            &recipe,
            1587,
            (0.0, 0.0),
            10.0,
            0.5, // 0.5 blades per square unit = 50 blades
            |_x, _z| 0.0, // flat terrain
            |_x, _z| true, // allow everywhere
        );

        assert!(!positions.is_empty());
        assert_eq!(positions.len(), colors.len());
        assert!(indices.len() % 3 == 0);
    }
}
