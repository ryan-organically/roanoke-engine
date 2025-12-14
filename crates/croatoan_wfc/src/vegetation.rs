use croatoan_procgen::{
    GrassBladeRecipe,
    generate_grass_blade,
    GrassSpecies,
    get_species_config,
    get_species_for_conditions,
    should_spawn_grass,
    calculate_density,
};
use crate::mesh_gen::get_height_at;
use glam::Vec3;
use noise::{NoiseFn, Perlin};

/// Derive biome name from terrain height
/// This maps height to biome strings for species selection
/// Heights matched to actual terrain generation in mesh_gen.rs
fn height_to_biome(height: f32) -> &'static str {
    if height < 0.0 {
        "Ocean"
    } else if height < 0.8 {
        "Beach" // Wet sand - no grass
    } else if height < 2.5 {
        "Beach" // Dunes - Sea Oats territory
    } else if height < 5.0 {
        "CoastalScrub"
    } else if height < 8.0 {
        "Grassland"
    } else if height < 40.0 {
        "DeciduousForest" // Forest starts at height 8+ (matches terrain gen)
    } else if height < 70.0 {
        "Foothills"
    } else {
        "AlpineMeadow"
    }
}

/// Estimate moisture from height and noise
fn estimate_moisture(height: f32, noise_val: f32) -> f32 {
    // Lower areas tend to be wetter, with noise variation
    let base_moisture = (1.0 - (height / 100.0).clamp(0.0, 1.0)) * 0.6;
    (base_moisture + noise_val * 0.2 + 0.3).clamp(0.0, 1.0)
}

/// Generate vegetation (grass) for a terrain chunk based on biome
///
/// Uses species-differentiated grass based on biome conditions:
/// - Beach: Sea Oats (wispy, sparse, tan-colored)
/// - Salt Marsh: Cordgrass (dense, tall, dark green)
/// - Grassland/Scrub: Sawgrass (medium, flowing)
/// - Forest: Forest Floor grass (tall, dark, shade-adapted)
/// - Alpine: Alpine grass (short, hardy tufts)
///
/// Returns (positions, colors, local_heights, indices) for grass mesh
pub fn generate_vegetation_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<f32>, Vec<u32>) {
    let noise = Perlin::new(seed + 999);

    // Maximum density for sampling positions
    // Higher density for bushier grass - must stay under 500K vertex limit
    // 2.5 * 256 * 256 = ~164K potential blades × 10 verts = ~1.6M, filtered to ~400K
    let max_density = 2.5;
    let blade_count = (chunk_size * chunk_size * max_density) as u32;

    let mut all_positions = Vec::new();
    let mut all_colors = Vec::new();
    let mut all_local_heights = Vec::new();
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

        // Derive biome from height
        let biome = height_to_biome(height);

        // Estimate moisture for species selection
        let moisture_noise = noise.get([world_x as f64 * 0.05, world_z as f64 * 0.05]) as f32;
        let moisture = estimate_moisture(height, moisture_noise);

        // Check if grass should spawn at this location
        let is_water = height < 0.0;
        let is_rock = false; // TODO: integrate with rock spawning system
        if !should_spawn_grass(biome, height, moisture, is_water, is_rock) {
            continue;
        }

        // Select species based on biome conditions
        let species = get_species_for_conditions(biome, height, moisture);
        let species_config = get_species_config(species);

        // Calculate actual density for this species at this location
        let slope = 0.0; // TODO: calculate from terrain
        let temperature = 0.6; // Colonial Virginia average
        let effective_density = calculate_density(species, moisture, temperature, slope);

        // Density check using species-specific density
        let density_roll = noise.get([world_x as f64 * 3.7, world_z as f64 * 3.7]) as f32;
        let normalized_roll = (density_roll + 1.0) * 0.5;

        // Compare against effective density (normalized to max_density)
        let density_threshold = effective_density / max_density;
        if normalized_roll > density_threshold.clamp(0.1, 1.0) {
            continue; // Skip this blade based on density
        }

        // Clumping: species with high clumping_factor create patches
        if species_config.clumping_factor > 0.3 {
            let clump_noise = noise.get([
                world_x as f64 * 0.3 * (1.0 + species_config.clumping_factor as f64),
                world_z as f64 * 0.3 * (1.0 + species_config.clumping_factor as f64),
            ]) as f32;
            if clump_noise < -0.3 * species_config.clumping_factor {
                continue; // Skip - in a gap between clumps
            }
        }

        // Patch Noise: Create patches of different sizes/heights
        let patch_noise = noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]) as f32;

        // Modulate height with patch noise for natural variation
        let height_mod = 1.0 + patch_noise * 0.3; // +/- 30% height variation

        // Create recipe from species config with environmental modulation
        let base_recipe = &species_config.recipe;
        let recipe = GrassBladeRecipe {
            height_range: (
                base_recipe.height_range.0 * height_mod,
                base_recipe.height_range.1 * height_mod,
            ),
            blade_segments: base_recipe.blade_segments.max(3), // Ensure minimum for FPS
            curve_factor: base_recipe.curve_factor,
            width_base: base_recipe.width_base,
            width_tip: base_recipe.width_tip,
            color_base: base_recipe.color_base,
            color_tip: base_recipe.color_tip,
        };

        // Small Y offset (5cm) to prevent Z-fighting with terrain
        let base_pos = Vec3::new(world_x, height + 0.05, world_z);
        let blade = generate_grass_blade(&recipe, seed + i, base_pos);

        // Append to combined mesh
        let vertex_offset = all_positions.len() as u32;
        all_positions.extend(blade.positions);
        all_colors.extend(blade.colors);
        all_local_heights.extend(blade.local_heights);
        all_indices.extend(blade.indices.iter().map(|idx| idx + vertex_offset));
    }

    println!("[GRASS] Chunk ({}, {}): {} verts, {} indices",
        offset_x, offset_z, all_positions.len(), all_indices.len());

    (all_positions, all_colors, all_local_heights, all_indices)
}

/// Generate vegetation with explicit biome override
///
/// Use this when you have actual biome data from BiomeGenerator
pub fn generate_vegetation_for_chunk_with_biome(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
    biome_name: &str,
    moisture: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let noise = Perlin::new(seed + 999);
    let max_density = 2.5;  // Same as primary function
    let blade_count = (chunk_size * chunk_size * max_density) as u32;

    let mut all_positions = Vec::new();
    let mut all_colors = Vec::new();
    let mut all_indices = Vec::new();

    let chunk_hash = ((offset_x as i32).wrapping_mul(73856093) ^ (offset_z as i32).wrapping_mul(19349663)) as u32;

    // Pre-select species for this biome
    let species = get_species_for_conditions(biome_name, 5.0, moisture);
    let species_config = get_species_config(species);

    for i in 0..blade_count {
        let combined_seed = seed.wrapping_add(chunk_hash).wrapping_add(i);
        let hash1 = (combined_seed.wrapping_mul(2654435761)) as f32 / u32::MAX as f32;
        let hash2 = (combined_seed.wrapping_mul(1597334677)) as f32 / u32::MAX as f32;

        let local_x = hash1 * chunk_size;
        let local_z = hash2 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Use provided biome for spawn check
        let is_water = height < 0.0;
        if !should_spawn_grass(biome_name, height, moisture, is_water, false) {
            continue;
        }

        // Density check
        let effective_density = calculate_density(species, moisture, 0.6, 0.0);
        let density_roll = noise.get([world_x as f64 * 3.7, world_z as f64 * 3.7]) as f32;
        let normalized_roll = (density_roll + 1.0) * 0.5;
        if normalized_roll > (effective_density / max_density).clamp(0.1, 1.0) {
            continue;
        }

        // Clumping check
        if species_config.clumping_factor > 0.3 {
            let clump_noise = noise.get([
                world_x as f64 * 0.3 * (1.0 + species_config.clumping_factor as f64),
                world_z as f64 * 0.3 * (1.0 + species_config.clumping_factor as f64),
            ]) as f32;
            if clump_noise < -0.3 * species_config.clumping_factor {
                continue;
            }
        }

        let patch_noise = noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]) as f32;
        let height_mod = 1.0 + patch_noise * 0.3;

        let base_recipe = &species_config.recipe;
        let recipe = GrassBladeRecipe {
            height_range: (
                base_recipe.height_range.0 * height_mod,
                base_recipe.height_range.1 * height_mod,
            ),
            blade_segments: base_recipe.blade_segments.max(3),
            curve_factor: base_recipe.curve_factor,
            width_base: base_recipe.width_base,
            width_tip: base_recipe.width_tip,
            color_base: base_recipe.color_base,
            color_tip: base_recipe.color_tip,
        };

        let base_pos = Vec3::new(world_x, height, world_z);
        let blade = generate_grass_blade(&recipe, seed + i, base_pos);

        let vertex_offset = all_positions.len() as u32;
        all_positions.extend(blade.positions);
        all_colors.extend(blade.colors);
        all_indices.extend(blade.indices.iter().map(|idx| idx + vertex_offset));
    }

    (all_positions, all_colors, all_indices)
}

/// Get the grass species that would be used at a given world position
/// Useful for debugging and encyclopedia integration
pub fn get_grass_species_at(x: f32, z: f32, seed: u32) -> Option<GrassSpecies> {
    let (height, _) = get_height_at(x, z, seed);
    let biome = height_to_biome(height);

    let noise = Perlin::new(seed + 999);
    let moisture_noise = noise.get([x as f64 * 0.05, z as f64 * 0.05]) as f32;
    let moisture = estimate_moisture(height, moisture_noise);

    if !should_spawn_grass(biome, height, moisture, height < 0.0, false) {
        return None;
    }

    Some(get_species_for_conditions(biome, height, moisture))
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
        let (positions, colors, _heights, indices) = generate_vegetation_for_chunk(
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

    #[test]
    fn test_height_to_biome() {
        assert_eq!(height_to_biome(-1.0), "Ocean");
        assert_eq!(height_to_biome(0.5), "Beach");
        assert_eq!(height_to_biome(2.0), "Beach"); // Dunes
        assert_eq!(height_to_biome(4.0), "CoastalScrub"); // 2.5 - 5.0
        assert_eq!(height_to_biome(6.0), "Grassland");    // 5.0 - 8.0
        assert_eq!(height_to_biome(10.0), "DeciduousForest"); // 8.0 - 40.0
        assert_eq!(height_to_biome(30.0), "DeciduousForest");
        assert_eq!(height_to_biome(60.0), "Foothills");   // 40.0 - 70.0
        assert_eq!(height_to_biome(90.0), "AlpineMeadow"); // 70.0+
    }

    #[test]
    fn test_species_selection_by_biome() {
        // Beach should get Sea Oats
        let beach_species = get_species_for_conditions("Beach", 2.0, 0.3);
        assert_eq!(beach_species, GrassSpecies::SeaOats);

        // Forest should get Forest Floor
        let forest_species = get_species_for_conditions("DeciduousForest", 30.0, 0.5);
        assert_eq!(forest_species, GrassSpecies::ForestFloor);

        // Alpine should get Alpine Grass
        let alpine_species = get_species_for_conditions("AlpineMeadow", 80.0, 0.4);
        assert_eq!(alpine_species, GrassSpecies::AlpineGrass);
    }

    #[test]
    fn test_biome_specific_generation() {
        // Generate specifically for salt marsh
        let (positions, colors, indices) = generate_vegetation_for_chunk_with_biome(
            42,
            16.0,
            0.0,
            0.0,
            "SaltMarsh",
            0.8,
        );

        // Salt marsh should have dense grass
        assert!(!positions.is_empty());
        assert_eq!(positions.len(), colors.len());
    }

    #[test]
    fn test_get_grass_species_at() {
        // At height 2.0 (beach dunes), should get Sea Oats
        let species = get_grass_species_at(100.0, 100.0, 42);
        // Species depends on actual terrain generation, just verify it returns something
        // or None for non-grass areas
        println!("Species at (100, 100): {:?}", species);
    }

    #[test]
    fn test_terrain_heights_for_grass() {
        use crate::mesh_gen::get_height_at;

        let seed = 42u32;
        let mut height_counts = std::collections::HashMap::new();

        // Sample heights across a typical chunk
        for x in (0..256).step_by(16) {
            for z in (0..256).step_by(16) {
                let (height, _) = get_height_at(x as f32, z as f32, seed);
                let biome = height_to_biome(height);
                *height_counts.entry(biome).or_insert(0) += 1;

                if x == 0 || x == 128 || x == 240 {
                    println!("({}, {}): height={:.2}, biome={}", x, z, height, biome);
                }
            }
        }

        println!("\nBiome distribution in chunk:");
        for (biome, count) in &height_counts {
            println!("  {}: {} samples", biome, count);
        }
    }
}
