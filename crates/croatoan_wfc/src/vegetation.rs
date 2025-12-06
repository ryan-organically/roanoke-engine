use croatoan_procgen::{GrassBladeRecipe, generate_grass_blade};
use crate::mesh_gen::get_height_at;
use glam::Vec3;
use noise::{NoiseFn, Perlin};

/// Generate vegetation (grass) for a terrain chunk based on biome
///
/// Grass density and height increase toward forest edge
/// Returns (positions, colors, indices) for grass mesh
pub fn generate_vegetation_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let noise = Perlin::new(seed + 999);

    // Maximum density for sampling positions
    // DRASTICALLY REDUCED for FPS: was 8.0, now 0.3
    // 0.3 * 256 * 256 = ~20K potential blades, filtering reduces to ~2-5K actual
    // This is 26x fewer grass blades than before
    let max_density = 0.3;
    let blade_count = (chunk_size * chunk_size * max_density) as u32;

    let mut all_positions = Vec::new();
    let mut all_colors = Vec::new();
    let mut all_indices = Vec::new();

    // Create chunk-specific seed from chunk coordinates
    let chunk_hash = ((offset_x as i32).wrapping_mul(73856093) ^ (offset_z as i32).wrapping_mul(19349663)) as u32;

    for i in 0..blade_count {
        // Generate positions using deterministic hash based on chunk + seed + index
        // Each chunk gets unique grass placement
        let combined_seed = seed.wrapping_add(chunk_hash).wrapping_add(i);
        let hash1 = (combined_seed.wrapping_mul(2654435761)) as f32 / u32::MAX as f32;
        let hash2 = (combined_seed.wrapping_mul(1597334677)) as f32 / u32::MAX as f32;

        let local_x = hash1 * chunk_size;
        let local_z = hash2 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        // Get terrain height and determine biome
        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Beach: height < 0.8 (no grass - pure sand)
        // Transition: height 0.8-2.0 (sparse dune grass)
        // Scrub: height 2.0-6.0 (moderate grass)
        // Forest edge: height 6.0-12.0 (dense, tall grass)
        // Deep forest: height 12.0+ (very dense, very tall grass)

        if height < 0.8 {
            continue; // No grass on beach/wet sand
        }

        // Calculate biome factor (0.0 = beach edge, 1.0 = deep forest)
        let biome_factor = ((height - 0.8) / 12.0).clamp(0.0, 1.0);

        // Density increases with height (scrub = 10%, forest = 100%)
        let density_threshold = 0.1 + biome_factor * 0.9;
        let density_roll = noise.get([world_x as f64 * 3.7, world_z as f64 * 3.7]) as f32;
        if (density_roll + 1.0) * 0.5 > density_threshold {
            continue; // Skip this blade based on density
        }

        // Patch Noise: Create patches of different sizes/heights
        let patch_noise = noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]) as f32; // Low frequency
        
        // Height range increases toward forest
        // Modulate with patch noise for variety
        let height_mod = 1.0 + patch_noise * 0.3; // +/- 30% height variation
        
        // Scrub: 0.4-0.8m
        // Forest edge: 0.8-1.6m
        // Deep forest: 1.2-2.4m
        let min_height = (0.4 + biome_factor * 0.8) * height_mod;
        let max_height = (0.8 + biome_factor * 1.6) * height_mod;

        let recipe = GrassBladeRecipe {
            height_range: (min_height, max_height),
            blade_segments: 3, // Reduced from 5 for FPS (40% fewer verts per blade)
            curve_factor: 0.4 + biome_factor * 0.3, // More curve in forest
            width_base: 0.06 + biome_factor * 0.04,
            width_tip: 0.01,
            // Brighter grass colors - vibrant greens
            color_base: [
                0.25 - biome_factor * 0.08,  // Slightly darker base in forest
                0.55 + biome_factor * 0.15,  // Rich green
                0.15,
            ],
            color_tip: [
                0.45 - biome_factor * 0.10,  // Yellow-green tips
                0.75 + biome_factor * 0.10,  // Bright green
                0.20,
            ],
        };

        let base_pos = Vec3::new(world_x, height, world_z);
        let blade = generate_grass_blade(&recipe, seed + i, base_pos);

        // Append to combined mesh
        let vertex_offset = all_positions.len() as u32;
        all_positions.extend(blade.positions);
        all_colors.extend(blade.colors);
        all_indices.extend(blade.indices.iter().map(|idx| idx + vertex_offset));
    }

    (all_positions, all_colors, all_indices)
}

/// Generate detritus (fallen logs, dead branches, etc.) for a terrain chunk.
///
/// # Detritus Types
/// - **Fallen Logs**: Horizontal cylinders in forest areas (height > 4m), 3-5m long
/// - **Dead Branches**: Smaller debris scattered in scrub/open areas
///
/// # Density
/// Increased 10x from original for much denser ground clutter.
/// - Previous: 0.008 items/sq meter
/// - Current: 0.08 items/sq meter
///
/// # Spawn Zones
/// - Logs: Forest only (height > 4m), more common in deep forest
/// - Branches: Scrub and forest edge (height 2-6m)
///
/// Returns (positions, normals, uvs, indices) for the combined mesh.
pub fn generate_detritus_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let noise = Perlin::new(seed + 555);

    // Detritus DISABLED for FPS - was generating thousands of extra vertices per chunk
    // Re-enable once we have instanced rendering
    let detritus_density = 0.0; // DISABLED
    let potential_items = (chunk_size * chunk_size * detritus_density) as u32;

    let mut all_positions = Vec::new();
    let mut all_normals = Vec::new();
    let mut all_uvs = Vec::new();
    let mut all_indices = Vec::new();

    for i in 0..potential_items {
        // Pseudo-random position within chunk
        let rand_x = noise.get([i as f64 * 0.2, 0.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.2, 100.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        // Get terrain height and determine biome
        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Only place detritus on land (above beach)
        if height < 2.0 {
            continue;
        }

        // Determine type: Rock or Log
        // Rocks more common in scrub/open areas, Logs in forest
        let type_roll = noise.get([world_x as f64 * 1.3, world_z as f64 * 1.3]) as f32;
        // Logs appear at forest edge (height > 4.0) and become more common deeper in
        let log_threshold = if height > 10.0 { -0.2 } else if height > 6.0 { 0.1 } else { 0.4 };
        let is_log = height > 4.0 && type_roll > log_threshold;

        let vertex_offset = all_positions.len() as u32;

        if is_log {
            // Generate a fallen log (horizontal cylinder)
            // 6-sided cylinder on its side - made larger and more visible
            let radius = 0.4 + (noise.get([world_x as f64, world_z as f64]) as f32 * 0.2);
            let length = 3.0 + (noise.get([world_x as f64 + 10.0, world_z as f64]) as f32 * 2.0);
            let angle = noise.get([world_x as f64 * 0.5, world_z as f64 * 0.5]) as f32 * 3.14; // Random rotation

            let segments = 6;
            for s in 0..=segments {
                let theta = (s as f32 / segments as f32) * std::f32::consts::TAU;
                let y = theta.sin() * radius;
                let z = theta.cos() * radius;

                // Rotate around Y axis (vertical) for orientation
                let cos_rot = angle.cos();
                let sin_rot = angle.sin();

                // Start cap
                let x_start = -length * 0.5;
                let rx_start = x_start * cos_rot - z * sin_rot;
                let rz_start = x_start * sin_rot + z * cos_rot;
                
                // End cap
                let x_end = length * 0.5;
                let rx_end = x_end * cos_rot - z * sin_rot;
                let rz_end = x_end * sin_rot + z * cos_rot;

                // Add vertices (simplified, no end caps for now)
                // Start
                all_positions.push([world_x + rx_start, height + y + radius * 0.8, world_z + rz_start]);
                all_normals.push([0.0, 1.0, 0.0]); // Approximate normal
                all_uvs.push([0.0, s as f32 / segments as f32]);

                // End
                all_positions.push([world_x + rx_end, height + y + radius * 0.8, world_z + rz_end]);
                all_normals.push([0.0, 1.0, 0.0]);
                all_uvs.push([1.0, s as f32 / segments as f32]);
            }

            // Indices for cylinder
            for s in 0..segments {
                let base = vertex_offset + (s * 2);
                all_indices.push(base);
                all_indices.push(base + 1);
                all_indices.push(base + 2);

                all_indices.push(base + 1);
                all_indices.push(base + 3);
                all_indices.push(base + 2);
            }

        } else {
            // Generate a simple rock (distorted tetrahedron/pyramid)
            let scale = 0.5 + (noise.get([world_x as f64, world_z as f64]) as f32 * 0.3);
            
            // 4 vertices for a tetrahedron
            let v0 = [world_x, height + scale, world_z]; // Top
            let v1 = [world_x - scale, height, world_z - scale];
            let v2 = [world_x + scale, height, world_z - scale];
            let v3 = [world_x, height, world_z + scale];

            all_positions.push(v0); all_normals.push([0.0, 1.0, 0.0]); all_uvs.push([0.5, 0.0]);
            all_positions.push(v1); all_normals.push([-0.5, 0.5, -0.5]); all_uvs.push([0.0, 1.0]);
            all_positions.push(v2); all_normals.push([0.5, 0.5, -0.5]); all_uvs.push([1.0, 1.0]);
            all_positions.push(v3); all_normals.push([0.0, 0.5, 0.5]); all_uvs.push([0.5, 1.0]);

            // Faces
            all_indices.push(vertex_offset); all_indices.push(vertex_offset + 1); all_indices.push(vertex_offset + 2);
            all_indices.push(vertex_offset); all_indices.push(vertex_offset + 2); all_indices.push(vertex_offset + 3);
            all_indices.push(vertex_offset); all_indices.push(vertex_offset + 3); all_indices.push(vertex_offset + 1);
        }
    }

    (all_positions, all_normals, all_uvs, all_indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vegetation_generation() {
        let (positions, colors, indices) = generate_vegetation_for_chunk(
            1587,
            32.0,
            0.0,
            0.0,
        );

        // Should generate some grass
        assert!(!positions.is_empty());
        assert_eq!(positions.len(), colors.len());
        assert!(indices.len() % 3 == 0);

        println!("Generated {} grass blades", positions.len() / 10); // ~10 verts per blade
    }
}
