use crate::noise_util;
use glam::{Vec2, Vec3};

/// Generate a procedural terrain chunk mesh
/// Returns (positions, colors, normals, indices)
pub fn generate_terrain_chunk(
    seed: u32,
    size: u32,
    offset_x: i32,
    offset_z: i32,
    scale: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
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

    // Calculate smooth normals
    let normals = calculate_smooth_normals(&positions, &indices, grid_size);

    // VERIFICATION OUTPUT
    if offset_x == 0 && offset_z == 0 {
        println!("[VERIFY] Generated Terrain Chunk: {}x{} (Scale {}) at ({}, {})", size, size, scale, offset_x, offset_z);
        println!("[VERIFY] Vertex Count: {}", positions.len());
        println!("[VERIFY] Triangle Count: {}", indices.len() / 3);
    }

    (positions, colors, normals, indices)
}

/// Calculate smooth vertex normals by averaging face normals
fn calculate_smooth_normals(positions: &[[f32; 3]], indices: &[u32], _grid_size: u32) -> Vec<[f32; 3]> {
    let vertex_count = positions.len();
    let mut normals = vec![[0.0f32; 3]; vertex_count];

    // Accumulate face normals for each vertex
    for triangle in indices.chunks(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        let p0 = Vec3::from_array(positions[i0]);
        let p1 = Vec3::from_array(positions[i1]);
        let p2 = Vec3::from_array(positions[i2]);

        // Calculate face normal
        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let face_normal = edge1.cross(edge2);

        // Add to each vertex
        normals[i0][0] += face_normal.x;
        normals[i0][1] += face_normal.y;
        normals[i0][2] += face_normal.z;

        normals[i1][0] += face_normal.x;
        normals[i1][1] += face_normal.y;
        normals[i1][2] += face_normal.z;

        normals[i2][0] += face_normal.x;
        normals[i2][1] += face_normal.y;
        normals[i2][2] += face_normal.z;
    }

    // Normalize all normals
    for normal in &mut normals {
        let n = Vec3::from_array(*normal);
        let normalized = n.normalize();
        *normal = normalized.to_array();
    }

    normals
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
        // Warm Sandy Brown (darker, less white)
        let c = [0.76, 0.60, 0.35];
        (h, m, c)
    } else if t < 0.65 {
        // Subtropical Scrub
        // Shortened from 0.75 to 0.65 to reduce middle ground
        let blend = (t - 0.55) / 0.1; // Adjusted divisor for new range (0.1 width)
        let h = lerp(2.0, 6.0, blend);
        let m = 1.0; // Rougher
        // Olive Green - Darkened significantly
        // Old: [0.92, 0.90, 0.85] -> [0.4, 0.5, 0.2]
        // New: [0.55, 0.55, 0.45] -> [0.25, 0.35, 0.15]
        let c = lerp_color([0.55, 0.55, 0.45], [0.25, 0.35, 0.15], blend);
        (h, m, c)
    } else {
        // Coastal Forest
        let blend = (t - 0.65) / 0.35; // Adjusted start and divisor (remainder of 1.0)
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
        let (positions, colors, normals, indices) = generate_terrain_chunk(1587, 64, 0, 0, 1.0);

        // Verify dimensions
        assert_eq!(positions.len(), 65 * 65);
        assert_eq!(colors.len(), 65 * 65);
        assert_eq!(normals.len(), 65 * 65);
        assert_eq!(indices.len(), 64 * 64 * 2 * 3);
    }

    #[test]
    fn test_small_mesh() {
        let (positions, colors, normals, indices) = generate_terrain_chunk(42, 4, 0, 0, 1.0);

        // 5x5 grid = 25 vertices
        assert_eq!(positions.len(), 25);
        assert_eq!(colors.len(), 25);
        assert_eq!(normals.len(), 25);

        // 4x4 quads = 32 triangles = 96 indices
        assert_eq!(indices.len(), 96);
    }

    #[test]
    fn test_eastern_sea_gradient() {
        // Generate West Chunk (Spawn)
        let (west_pos, _, _, _) = generate_terrain_chunk(12345, 64, 0, 0, 1.0);

        // Generate East Chunk (Far East)
        let (east_pos, _, _, _) = generate_terrain_chunk(12345, 64, 1000, 0, 1.0);
        
        // Calculate average height
        let west_avg: f32 = west_pos.iter().map(|p| p[1]).sum::<f32>() / west_pos.len() as f32;
        let east_avg: f32 = east_pos.iter().map(|p| p[1]).sum::<f32>() / east_pos.len() as f32;

        println!("West Avg Height: {}, East Avg Height: {}", west_avg, east_avg);

        // The East side should be lower (Ocean)
        assert!(east_avg < west_avg, "East side should be lower than West side due to gradient");
    }
}

/// Generate detritus (logs, driftwood, dead trees) for a chunk
/// Returns (positions, normals, uvs, indices)
pub fn generate_detritus_for_chunk(
    seed: u32,
    size: f32,
    offset_x: f32,
    offset_z: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut index_offset = 0;

    // Use a fixed grid for potential spawn points
    let grid_step = 4.0; // Check every 4 meters
    let steps = (size / grid_step) as i32;

    for z in 0..steps {
        for x in 0..steps {
            let global_x = offset_x + (x as f32 * grid_step);
            let global_z = offset_z + (z as f32 * grid_step);

            // Add some jitter to position
            let jitter_x = noise_util::hash(seed + (global_x as u32) * 73856093) * 3.0;
            let jitter_z = noise_util::hash(seed + (global_z as u32) * 19349663) * 3.0;
            let px = global_x + jitter_x;
            let pz = global_z + jitter_z;

            // Get biome info
            // Replicating get_height_at logic partially to get 't'
            let biome_scale = 0.002;
            let biome_noise = noise_util::fbm(
                Vec2::new(px * biome_scale, pz * biome_scale),
                3, 2.0, 0.5, seed + 100
            );
            let noise_norm = (biome_noise + 1.0) * 0.5;
            let gradient = -px * 0.001; 
            let t = (noise_norm * 0.3 + gradient + 0.5).clamp(0.0, 1.0);

            let (terrain_height, _) = get_height_at(px, pz, seed);

            // Spawn Logic based on Biome
            let spawn_chance = noise_util::hash(seed + (px as u32) ^ (pz as u32));
            
            if t < 0.45 {
                // Ocean / Shallow Water (Inlets)
                // Spawn dead trees in shallow water
                if terrain_height > -2.0 && terrain_height < 0.5 && spawn_chance > 0.95 {
                    // Dead Tree (Vertical)
                    add_cylinder(
                        &mut positions, &mut normals, &mut uvs, &mut indices, &mut index_offset,
                        Vec3::new(px, terrain_height, pz),
                        0.3, // Radius
                        4.0 + spawn_chance * 3.0, // Height
                        Vec3::Y, // Up
                        8 // Segments
                    );
                }
            } else if t < 0.55 {
                // Beach
                // Spawn driftwood (scattered sticks)
                if spawn_chance > 0.92 {
                    // Driftwood (Small, random orientation)
                    let rot_x = (spawn_chance * 10.0).sin();
                    let rot_z = (spawn_chance * 10.0).cos();
                    let axis = Vec3::new(rot_x, 0.1, rot_z).normalize();
                    
                    add_cylinder(
                        &mut positions, &mut normals, &mut uvs, &mut indices, &mut index_offset,
                        Vec3::new(px, terrain_height + 0.1, pz),
                        0.1, // Radius
                        1.5, // Length
                        axis,
                        6 // Segments
                    );
                }
            } else if t > 0.75 {
                // Forest
                // Spawn fallen logs
                if spawn_chance > 0.97 {
                    // Fallen Log (Horizontal)
                    let angle = spawn_chance * std::f32::consts::PI * 2.0;
                    let axis = Vec3::new(angle.cos(), 0.0, angle.sin());
                    
                    add_cylinder(
                        &mut positions, &mut normals, &mut uvs, &mut indices, &mut index_offset,
                        Vec3::new(px, terrain_height + 0.3, pz),
                        0.4, // Radius
                        3.0 + spawn_chance * 2.0, // Length
                        axis,
                        8 // Segments
                    );
                }
            }
        }
    }

    (positions, normals, uvs, indices)
}

/// Helper to add a cylinder mesh
fn add_cylinder(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    index_offset: &mut u32,
    center: Vec3,
    radius: f32,
    length: f32,
    axis: Vec3,
    segments: u32,
) {
    // Basis vectors for the cylinder cap
    let up = axis.normalize();
    let arbitrary = if up.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
    let right = up.cross(arbitrary).normalize();
    let forward = up.cross(right).normalize();

    let half_len = length * 0.5;
    let start = center - up * half_len;
    let end = center + up * half_len;

    // Generate vertices for the side
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
        let x = angle.cos();
        let z = angle.sin();

        let normal = (right * x + forward * z).normalize();
        let offset = normal * radius;

        // Bottom vertex
        positions.push((start + offset).to_array());
        normals.push(normal.to_array());
        uvs.push([i as f32 / segments as f32, 0.0]);

        // Top vertex
        positions.push((end + offset).to_array());
        normals.push(normal.to_array());
        uvs.push([i as f32 / segments as f32, 1.0]);
    }

    // Generate indices
    for i in 0..segments {
        let base = *index_offset + i * 2;
        
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);

        indices.push(base + 1);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    *index_offset += (segments + 1) * 2;
}
