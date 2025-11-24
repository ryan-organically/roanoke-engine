use croatoan_procgen::{TreeRecipe, generate_tree, generate_tree_mesh};
use crate::mesh_gen::get_height_at;
use noise::{NoiseFn, Perlin};

/// Generate trees for a terrain chunk based on biome
///
/// Trees appear at forest edge and become denser in deep forest
/// Returns combined mesh data (positions, normals, uvs, indices)
pub fn generate_trees_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let noise = Perlin::new(seed + 777);

    // Sample potential tree positions
    let tree_density = 0.001; // Trees per square unit (very sparse - was 0.02)
    let potential_trees = (chunk_size * chunk_size * tree_density) as u32;

    let mut all_positions = Vec::new();
    let mut all_normals = Vec::new();
    let mut all_uvs = Vec::new();
    let mut all_indices = Vec::new();

    for i in 0..potential_trees {
        // Pseudo-random position within chunk
        let rand_x = noise.get([i as f64 * 0.1, 0.0]) as f32;
        let rand_z = noise.get([i as f64 * 0.1, 100.0]) as f32;

        let local_x = (rand_x + 1.0) * 0.5 * chunk_size;
        let local_z = (rand_z + 1.0) * 0.5 * chunk_size;

        let world_x = offset_x + local_x;
        let world_z = offset_z + local_z;

        // Get terrain height and determine biome
        let (height, _color) = get_height_at(world_x, world_z, seed);

        // Beach: height < 1.5 (no trees)
        // Scrub: height 1.5-6.0 (no trees)
        // Forest edge: height 6.0-12.0 (sparse trees start appearing)
        // Forest: height 12.0+ (dense trees)

        if height < 6.0 {
            continue; // No trees in beach or scrub
        }

        // Calculate biome factor (0.0 = forest edge start, 1.0 = deep forest)
        let biome_factor = ((height - 6.0) / 10.0).clamp(0.0, 1.0);

        // Density increases with height (forest edge = 20%, deep forest = 100%)
        let density_threshold = 0.2 + biome_factor * 0.8;
        let density_roll = noise.get([world_x as f64 * 5.1, world_z as f64 * 5.1]) as f32;
        if (density_roll + 1.0) * 0.5 > density_threshold {
            continue; // Skip this tree based on density
        }

        // Choose tree species based on biome
        // Forest edge: Oak, Birch (40% each), Maple (20%)
        // Deep forest: Pine, Spruce (40% each), Oak (20%)
        let species_roll = noise.get([world_x as f64 * 7.3, world_z as f64 * 7.3]) as f32;
        let species_roll_norm = (species_roll + 1.0) * 0.5;

        let recipe = if biome_factor < 0.5 {
            // Forest edge - deciduous trees
            if species_roll_norm < 0.4 {
                TreeRecipe::oak()
            } else if species_roll_norm < 0.8 {
                TreeRecipe::birch()
            } else {
                TreeRecipe::maple()
            }
        } else {
            // Deep forest - mix of deciduous and coniferous
            if species_roll_norm < 0.4 {
                TreeRecipe::pine()
            } else if species_roll_norm < 0.8 {
                TreeRecipe::spruce()
            } else {
                TreeRecipe::oak()
            }
        };

        // Generate tree with position-based seed for variation
        let tree_seed = seed.wrapping_add((world_x * 1000.0) as u32).wrapping_add((world_z * 100.0) as u32);
        let tree = generate_tree(&recipe, tree_seed as u64);
        let mesh = generate_tree_mesh(&tree);

        // Offset tree to world position
        let vertex_offset = all_positions.len() as u32;

        for vertex in &mesh.vertices {
            let mut pos = vertex.position;
            pos[0] += world_x;
            pos[1] += height; // Base of tree at terrain height
            pos[2] += world_z;

            all_positions.push(pos);
            all_normals.push(vertex.normal);
            all_uvs.push(vertex.uv);
        }

        all_indices.extend(mesh.indices.iter().map(|idx| idx + vertex_offset));
    }

    (all_positions, all_normals, all_uvs, all_indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_generation() {
        let (positions, normals, uvs, indices) = generate_trees_for_chunk(
            12345,
            256.0,
            0.0,
            0.0,
        );

        // Should generate some trees (depends on seed and chunk)
        println!("Generated {} tree vertices", positions.len());
        println!("Generated {} triangles", indices.len() / 3);

        assert_eq!(positions.len(), normals.len());
        assert_eq!(positions.len(), uvs.len());
        assert!(indices.len() % 3 == 0);
    }
}
