use crate::noise_util;
use glam::Vec2;

/// Generate a procedural terrain chunk mesh
/// Returns (positions, colors, indices)
pub fn generate_terrain_chunk(
    seed: u32,
    size: u32,
    offset_x: i32,
    offset_z: i32,
    scale: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let grid_size = size + 1; // Number of vertices per dimension
    let vertex_count = (grid_size * grid_size) as usize;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);

    // Generate vertices with biome-based height and coloring
    for z in 0..grid_size {
        for x in 0..grid_size {
            // Global coordinates
            // Scale determines the distance between vertices
            let global_x = (x as f32 * scale) + offset_x as f32;
            let global_z = (z as f32 * scale) + offset_z as f32;

            let (height, base_color) = get_height_at(global_x, global_z, seed);

            // Global position for the mesh
            // We use global coordinates so the chunks align perfectly without needing model matrices
            positions.push([global_x, height, global_z]);
            colors.push(base_color);
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

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    // VERIFICATION OUTPUT
    if offset_x == 0 && offset_z == 0 {
        println!("[VERIFY] Generated Terrain Chunk: {}x{} (Scale {}) at ({}, {})", size, size, scale, offset_x, offset_z);
        println!("[VERIFY] Vertex Count: {}", positions.len());
        println!("[VERIFY] Triangle Count: {}", indices.len() / 3);
    }

    (positions, colors, indices)
}

/// Calculate height and color at a specific global position
pub fn get_height_at(x: f32, z: f32, seed: u32) -> (f32, [f32; 3]) {
    // 1. Biome Noise (Low Frequency)
    let biome_scale = 0.002; // Slower transitions
    let biome_noise = noise_util::fbm(
        Vec2::new(x * biome_scale, z * biome_scale),
        3, 2.0, 0.5, seed + 100
    );
    let noise_norm = (biome_noise + 1.0) * 0.5;

    // 2. Eastern Sea Gradient (Global X based)
    // We want a gentle curve.
    // Positive X -> Ocean. Negative X -> Inland.
    // Transition zone ~1000 units.
    let gradient = -x * 0.001; 
    
    // Combined 't' value determines "Land vs Sea"
    let t = noise_norm * 0.3 + gradient + 0.5; // Bias to 0.5 at x=0
    let t = t.clamp(0.0, 1.0);

    // 3. Detail Noise
    let detail_noise = noise_util::fbm(
        Vec2::new(x * 0.05, z * 0.05),
        4, 2.0, 0.5, seed
    );

    // 4. Biome Definitions (Roanoke Spec)
    let (base_height, height_mult, base_color) = if t < 0.45 {
        // Ocean / Shallow Water
        // Add sandbars using detail noise
        let sandbar = if detail_noise > 0.5 { 0.5 } else { 0.0 };
        let water_depth = lerp(-5.0, -0.5, t / 0.45);
        let h = water_depth + sandbar;
        
        // Color: Turquoise at shore, Teal deep
        let depth_factor = (t / 0.45).clamp(0.0, 1.0);
        let c = lerp_color([0.05, 0.3, 0.4], [0.2, 0.8, 0.8], depth_factor);
        (h, 0.1, c)
    } else if t < 0.55 {
        // Beach / Dunes
        let blend = (t - 0.45) / 0.1;
        let h = lerp(0.0, 2.0, blend);
        let m = 0.2; // Soft dunes
        // Golden Beige Sand
        let c = [0.94, 0.85, 0.65]; 
        (h, m, c)
    } else if t < 0.75 {
        // Subtropical Scrub
        let blend = (t - 0.55) / 0.2;
        let h = lerp(2.0, 6.0, blend);
        let m = 1.0; // Rougher
        // Olive Green
        let c = lerp_color([0.92, 0.90, 0.85], [0.4, 0.5, 0.2], blend);
        (h, m, c)
    } else {
        // Coastal Forest
        let blend = (t - 0.75) / 0.25;
        let h = lerp(6.0, 15.0, blend);
        let m = 2.0;
        // Deep Green
        let c = lerp_color([0.4, 0.5, 0.2], [0.1, 0.35, 0.1], blend);
        (h, m, c)
    };

    // Apply height
    let height = base_height + detail_noise * height_mult;

    (height, base_color)
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
    fn test_mesh_generation() {
        let (positions, colors, indices) = generate_terrain_chunk(1587, 64, 0, 0, 1.0);

        // Verify dimensions
        assert_eq!(positions.len(), 65 * 65);
        assert_eq!(colors.len(), 65 * 65);
        assert_eq!(indices.len(), 64 * 64 * 2 * 3);
    }

    #[test]
    fn test_small_mesh() {
        let (positions, colors, indices) = generate_terrain_chunk(42, 4, 0, 0, 1.0);

        // 5x5 grid = 25 vertices
        assert_eq!(positions.len(), 25);
        assert_eq!(colors.len(), 25);

        // 4x4 quads = 32 triangles = 96 indices
        assert_eq!(indices.len(), 96);
    }

    #[test]
    fn test_eastern_sea_gradient() {
        // Generate West Chunk (Spawn)
        let (west_pos, _, _) = generate_terrain_chunk(12345, 64, 0, 0, 1.0);
        
        // Generate East Chunk (Far East)
        let (east_pos, _, _) = generate_terrain_chunk(12345, 64, 1000, 0, 1.0);
        
        // Calculate average height
        let west_avg: f32 = west_pos.iter().map(|p| p[1]).sum::<f32>() / west_pos.len() as f32;
        let east_avg: f32 = east_pos.iter().map(|p| p[1]).sum::<f32>() / east_pos.len() as f32;

        println!("West Avg Height: {}, East Avg Height: {}", west_avg, east_avg);

        // The East side should be lower (Ocean)
        assert!(east_avg < west_avg, "East side should be lower than West side due to gradient");
    }
}
