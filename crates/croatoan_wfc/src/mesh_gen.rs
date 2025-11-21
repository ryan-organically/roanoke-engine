use crate::noise_util;
use glam::Vec2;

/// Generate a procedural terrain chunk mesh
/// Returns (positions, colors, indices)
pub fn generate_terrain_chunk(
    seed: u32,
    size: u32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let grid_size = size + 1; // Number of vertices per dimension
    let vertex_count = (grid_size * grid_size) as usize;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);

    // Generate vertices with noise-based height and coloring
    for z in 0..grid_size {
        for x in 0..grid_size {
            let fx = x as f32;
            let fz = z as f32;

            // Normalized coordinates (0.0 to 1.0)
            // We use size (grid cells) for normalization to match the gradient across the chunk
            let _u = fx / size as f32;
            let v = fz / size as f32;

            // The Gradient: slope increases with z (v)
            let slope = v;

            // Sample noise at this position
            // Frequency reduced to 0.03 for wider features
            let sample_point = Vec2::new(fx * 0.03, fz * 0.03);
            let noise = noise_util::fbm(
                sample_point,
                4,    // octaves
                2.0,  // lacunarity
                0.4,  // persistence (reduced from 0.5)
                seed,
            );

            // The Composition
            // Base height rises from -5.0 to +15.0
            let base_height = (slope * 20.0) - 5.0;
            // Detail is small rolling dunes
            let detail = noise * 2.0;
            
            let height = base_height + detail;

            // Position
            positions.push([fx, height, fz]);

            // Color based on height
            let color = if height < 0.0 {
                // Deep Blue - Ocean
                [0.0, 0.0, 0.5]
            } else if height < 1.5 {
                // Sand Yellow - Beach
                [0.9, 0.8, 0.3]
            } else if height < 5.0 {
                // Scrub Green - Dunes
                [0.4, 0.6, 0.3]
            } else {
                // Pine Dark Green - Forest
                [0.1, 0.4, 0.1]
            };

            colors.push(color);
        }
    }

    // Generate indices for triangles
    let triangle_count = (size * size * 2) as usize;
    let mut indices = Vec::with_capacity(triangle_count * 3);

    for z in 0..size {
        for x in 0..size {
            let top_left = z * grid_size + x;
            let top_right = top_left + 1;
            let bottom_left = (z + 1) * grid_size + x;
            let bottom_right = bottom_left + 1;

            // First triangle (top-left, bottom-left, top-right)
            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            // Second triangle (top-right, bottom-left, bottom-right)
            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    // VERIFICATION OUTPUT
    println!("[VERIFY] Generated Terrain Chunk: {}x{}", size, size);
    println!("[VERIFY] Vertex Count: {}", positions.len());
    println!("[VERIFY] Triangle Count: {}", indices.len() / 3);

    // Sanity check
    assert_eq!(
        positions.len(),
        vertex_count,
        "Position count mismatch"
    );
    assert_eq!(
        colors.len(),
        vertex_count,
        "Color count mismatch"
    );
    assert_eq!(
        indices.len(),
        triangle_count * 3,
        "Index count mismatch"
    );

    (positions, colors, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_generation() {
        let (positions, colors, indices) = generate_terrain_chunk(1587, 64);

        // Verify dimensions
        assert_eq!(positions.len(), 65 * 65);
        assert_eq!(colors.len(), 65 * 65);
        assert_eq!(indices.len(), 64 * 64 * 2 * 3);
    }

    #[test]
    fn test_small_mesh() {
        let (positions, colors, indices) = generate_terrain_chunk(42, 4);

        // 5x5 grid = 25 vertices
        assert_eq!(positions.len(), 25);
        assert_eq!(colors.len(), 25);

        // 4x4 quads = 32 triangles = 96 indices
        assert_eq!(indices.len(), 96);
    }
}
