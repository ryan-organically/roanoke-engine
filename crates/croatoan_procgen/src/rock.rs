use glam::Vec3;

/// Types of rock formations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RockType {
    Boulder,
    RiverStone,
    SharpRock,
    CliffFace,
}

/// Parameters for procedural rock generation
#[derive(Debug, Clone)]
pub struct RockRecipe {
    pub rock_type: RockType,
    pub base_size: Vec3,
    pub seed: u32,
    pub subdivision_levels: u32,
    pub roughness: f32,
    pub deformation: f32,
}

impl Default for RockRecipe {
    fn default() -> Self {
        Self::boulder()
    }
}

impl RockRecipe {
    pub fn boulder() -> Self {
        RockRecipe {
            rock_type: RockType::Boulder,
            base_size: Vec3::new(1.0, 0.8, 1.0),
            seed: 0,
            subdivision_levels: 2,
            roughness: 0.1,
            deformation: 0.2,
        }
    }

    pub fn river_stone() -> Self {
        RockRecipe {
            rock_type: RockType::RiverStone,
            base_size: Vec3::new(0.5, 0.3, 0.5),
            seed: 0,
            subdivision_levels: 3, // More smooth
            roughness: 0.05,
            deformation: 0.1,
        }
    }

    pub fn sharp_rock() -> Self {
        RockRecipe {
            rock_type: RockType::SharpRock,
            base_size: Vec3::new(0.8, 1.2, 0.8),
            seed: 0,
            subdivision_levels: 1, // Angular
            roughness: 0.4,
            deformation: 0.5,
        }
    }
}

/// Vertex data for rock mesh
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RockVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Generated rock mesh
#[derive(Debug, Clone)]
pub struct RockMesh {
    pub vertices: Vec<RockVertex>,
    pub indices: Vec<u32>,
}

/// Generate a rock mesh from a recipe
pub fn generate_rock(recipe: &RockRecipe) -> RockMesh {
    // Start with a simple icosahedron or cube
    let (mut vertices, mut indices) = create_base_icosphere();

    // Subdivide and displace
    for _ in 0..recipe.subdivision_levels {
        subdivide(&mut vertices, &mut indices);
    }

    // Displace vertices based on noise and recipe parameters
    displace_vertices(&mut vertices, recipe);

    // Recalculate normals
    recalculate_normals(&mut vertices, &indices);

    RockMesh {
        vertices,
        indices,
    }
}

// --- Helper Functions ---

fn create_base_icosphere() -> (Vec<RockVertex>, Vec<u32>) {
    // Golden ratio
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;

    let positions = vec![
        Vec3::new(-1.0,  t,  0.0).normalize(),
        Vec3::new( 1.0,  t,  0.0).normalize(),
        Vec3::new(-1.0, -t,  0.0).normalize(),
        Vec3::new( 1.0, -t,  0.0).normalize(),

        Vec3::new( 0.0, -1.0,  t).normalize(),
        Vec3::new( 0.0,  1.0,  t).normalize(),
        Vec3::new( 0.0, -1.0, -t).normalize(),
        Vec3::new( 0.0,  1.0, -t).normalize(),

        Vec3::new( t,  0.0, -1.0).normalize(),
        Vec3::new( t,  0.0,  1.0).normalize(),
        Vec3::new(-t,  0.0, -1.0).normalize(),
        Vec3::new(-t,  0.0,  1.0).normalize(),
    ];

    let mut vertices = Vec::new();
    for pos in positions {
        vertices.push(RockVertex {
            position: pos.to_array(),
            normal: pos.to_array(), // Initial normal is just position for sphere
            uv: [0.0, 0.0], // Todo: Spherical UV mapping
        });
    }

    let indices = vec![
        0, 11, 5,  0, 5, 1,  0, 1, 7,  0, 7, 10,  0, 10, 11,
        1, 5, 9,  5, 11, 4,  11, 10, 2,  10, 7, 6,  7, 1, 8,
        3, 9, 4,  3, 4, 2,  3, 2, 6,  3, 6, 8,  3, 8, 9,
        4, 9, 5,  2, 4, 11,  6, 2, 10,  8, 6, 7,  9, 8, 1
    ];

    (vertices, indices)
}

fn subdivide(vertices: &mut Vec<RockVertex>, indices: &mut Vec<u32>) {
    let mut new_indices = Vec::new();
    let mut midpoints = std::collections::HashMap::new();

    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i];
        let i1 = indices[i+1];
        let i2 = indices[i+2];

        let a = get_midpoint(i0, i1, vertices, &mut midpoints);
        let b = get_midpoint(i1, i2, vertices, &mut midpoints);
        let c = get_midpoint(i2, i0, vertices, &mut midpoints);

        new_indices.extend_from_slice(&[i0, a, c]);
        new_indices.extend_from_slice(&[i1, b, a]);
        new_indices.extend_from_slice(&[i2, c, b]);
        new_indices.extend_from_slice(&[a, b, c]);
    }

    *indices = new_indices;
}

fn get_midpoint(p1: u32, p2: u32, vertices: &mut Vec<RockVertex>, midpoints: &mut std::collections::HashMap<(u32, u32), u32>) -> u32 {
    let key = if p1 < p2 { (p1, p2) } else { (p2, p1) };

    if let Some(&index) = midpoints.get(&key) {
        return index;
    }

    let v1 = vertices[p1 as usize];
    let v2 = vertices[p2 as usize];

    let p1_vec = Vec3::from_array(v1.position);
    let p2_vec = Vec3::from_array(v2.position);
    let middle = (p1_vec + p2_vec).normalize(); // Keep it spherical for now

    let index = vertices.len() as u32;
    vertices.push(RockVertex {
        position: middle.to_array(),
        normal: middle.to_array(),
        uv: [0.0, 0.0],
    });

    midpoints.insert(key, index);
    index
}

fn displace_vertices(vertices: &mut Vec<RockVertex>, recipe: &RockRecipe) {
    use noise::{NoiseFn, Perlin};
    let perlin = Perlin::new(recipe.seed);

    for v in vertices.iter_mut() {
        let pos = Vec3::from_array(v.position);
        
        // 1. Base shape deformation (scaling)
        let mut deformed_pos = pos * recipe.base_size;

        // 2. Noise displacement
        let noise_val = perlin.get([pos.x as f64 * 2.0, pos.y as f64 * 2.0, pos.z as f64 * 2.0]) as f32;
        let displacement = noise_val * recipe.roughness;
        
        // 3. Voronoi-like flattening (simple approximation)
        // If we want sharp rocks, we can clamp noise or use abs()
        if recipe.rock_type == RockType::SharpRock {
             // Flatten bottom
             if deformed_pos.y < -0.2 {
                 deformed_pos.y *= 0.3;
             }
        }

        let final_pos = deformed_pos + (pos.normalize() * displacement * recipe.deformation);
        v.position = final_pos.to_array();
        
        // Simple UV mapping (spherical projection)
        let u = 0.5 + (final_pos.z.atan2(final_pos.x) / (2.0 * std::f32::consts::PI));
        let v_coord = 0.5 - (final_pos.y.asin() / std::f32::consts::PI);
        v.uv = [u, v_coord];
    }
}

fn recalculate_normals(vertices: &mut Vec<RockVertex>, indices: &Vec<u32>) {
    // Reset normals
    for v in vertices.iter_mut() {
        v.normal = [0.0, 0.0, 0.0];
    }

    // Accumulate face normals
    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i] as usize;
        let i1 = indices[i+1] as usize;
        let i2 = indices[i+2] as usize;

        let v0 = Vec3::from_array(vertices[i0].position);
        let v1 = Vec3::from_array(vertices[i1].position);
        let v2 = Vec3::from_array(vertices[i2].position);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();

        // Add to vertices (smooth shading by averaging)
        let n_array = normal.to_array();
        for idx in [i0, i1, i2] {
            vertices[idx].normal[0] += n_array[0];
            vertices[idx].normal[1] += n_array[1];
            vertices[idx].normal[2] += n_array[2];
        }
    }

    // Normalize
    for v in vertices.iter_mut() {
        let n = Vec3::from_array(v.normal).normalize_or_zero();
        v.normal = n.to_array();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rock_generation() {
        let recipe = RockRecipe::boulder();
        let mesh = generate_rock(&recipe);
        
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0); // Triangles
        
        // Check bounds
        for v in &mesh.vertices {
            let pos = Vec3::from_array(v.position);
            assert!(pos.length() > 0.1); // Should have some size
        }
    }

    #[test]
    fn test_different_types() {
        let types = [
            RockRecipe::boulder(),
            RockRecipe::river_stone(),
            RockRecipe::sharp_rock(),
        ];

        for recipe in types {
            let mesh = generate_rock(&recipe);
            assert!(!mesh.vertices.is_empty());
        }
    }
}