use std::collections::HashMap;
use glam::{Vec3, Quat};

/// Tree species with different growth characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSpecies {
    Oak,
    Pine,
    Willow,
    Birch,
    Palm,
    Maple,
    Spruce,
    Custom,
}

/// L-System rule for tree generation
#[derive(Debug, Clone)]
pub struct LSystemRule {
    pub from: char,
    pub to: String,
}

/// Complete tree recipe using L-System parameters
#[derive(Debug, Clone)]
pub struct TreeRecipe {
    pub axiom: String,
    pub rules: HashMap<char, String>,
    pub iterations: u32,
    pub angle: f32,
    pub length_decay: f32,
    pub thickness_decay: f32,
    pub initial_length: f32,
    pub initial_thickness: f32,
    pub leaf_probability: f32,
    pub gravity: f32,
    pub species: TreeSpecies,
    pub branch_segments: u32,
    pub radial_segments: u32,
}

impl Default for TreeRecipe {
    fn default() -> Self {
        TreeRecipe::oak()
    }
}

impl TreeRecipe {
    /// Create a generic oak tree recipe
    pub fn oak() -> Self {
        let mut rules = HashMap::new();
        rules.insert('F', "FF-[-F+F+F]+[+F-F-F]".to_string());

        TreeRecipe {
            axiom: "F".to_string(),
            rules,
            iterations: 2,  // Reduced from 3 to 2 for performance
            angle: 22.5_f32.to_radians(),
            length_decay: 0.7,
            thickness_decay: 0.6,
            initial_length: 2.0,
            initial_thickness: 0.3,
            leaf_probability: 0.3,
            gravity: 0.0,
            species: TreeSpecies::Oak,
            branch_segments: 3,
            radial_segments: 4,
        }
    }

    /// Create a pine tree recipe (conical, narrow)
    pub fn pine() -> Self {
        let mut rules = HashMap::new();
        rules.insert('F', "FF[-F][+F]F".to_string());

        TreeRecipe {
            axiom: "F".to_string(),
            rules,
            iterations: 3,  // Reduced from 4 to 3 for performance
            angle: 15.0_f32.to_radians(),
            length_decay: 0.75,
            thickness_decay: 0.65,
            initial_length: 2.5,
            initial_thickness: 0.25,
            leaf_probability: 0.4,
            gravity: 0.0,
            species: TreeSpecies::Pine,
            branch_segments: 2,
            radial_segments: 4,
        }
    }

    /// Create a willow tree recipe (drooping branches)
    pub fn willow() -> Self {
        let mut rules = HashMap::new();
        rules.insert('F', "F[--F][++F]F".to_string());

        TreeRecipe {
            axiom: "F".to_string(),
            rules,
            iterations: 5,
            angle: 25.0_f32.to_radians(),
            length_decay: 0.8,
            thickness_decay: 0.55,
            initial_length: 1.8,
            initial_thickness: 0.28,
            leaf_probability: 0.5,
            gravity: -0.5,
            species: TreeSpecies::Willow,
            branch_segments: 3,
            radial_segments: 4,
        }
    }

    /// Create a birch tree recipe (tall, slender)
    pub fn birch() -> Self {
        let mut rules = HashMap::new();
        rules.insert('F', "FF[-F+F][+F-F]".to_string());

        TreeRecipe {
            axiom: "F".to_string(),
            rules,
            iterations: 5,
            angle: 20.0_f32.to_radians(),
            length_decay: 0.65,
            thickness_decay: 0.7,
            initial_length: 2.2,
            initial_thickness: 0.2,
            leaf_probability: 0.35,
            gravity: 0.0,
            species: TreeSpecies::Birch,
            branch_segments: 3,
            radial_segments: 5,
        }
    }

    /// Create a palm tree recipe (single trunk, terminal fronds)
    pub fn palm() -> Self {
        let mut rules = HashMap::new();
        rules.insert('F', "FF".to_string());
        rules.insert('L', "[++++L][----L][++L][--L]".to_string());

        TreeRecipe {
            axiom: "FFFFFFL".to_string(),
            rules,
            iterations: 2,
            angle: 35.0_f32.to_radians(),
            length_decay: 1.0,
            thickness_decay: 0.9,
            initial_length: 3.0,
            initial_thickness: 0.35,
            leaf_probability: 1.0,
            gravity: 0.0,
            species: TreeSpecies::Palm,
            branch_segments: 2,
            radial_segments: 5,
        }
    }

    /// Create a maple tree recipe (broad, dense canopy)
    pub fn maple() -> Self {
        let mut rules = HashMap::new();
        rules.insert('F', "F[-F+F][+F-F]F".to_string());

        TreeRecipe {
            axiom: "F".to_string(),
            rules,
            iterations: 3,
            angle: 28.0_f32.to_radians(),
            length_decay: 0.68,
            thickness_decay: 0.58,
            initial_length: 2.0,
            initial_thickness: 0.32,
            leaf_probability: 0.4,
            gravity: 0.0,
            species: TreeSpecies::Maple,
            branch_segments: 3,
            radial_segments: 4,
        }
    }

    /// Create a spruce tree recipe (tall, conical)
    pub fn spruce() -> Self {
        let mut rules = HashMap::new();
        rules.insert('F', "FF[--F][+F][++F]".to_string());

        TreeRecipe {
            axiom: "F".to_string(),
            rules,
            iterations: 3,  // Reduced from 4 to 3 for performance
            angle: 18.0_f32.to_radians(),
            length_decay: 0.73,
            thickness_decay: 0.68,
            initial_length: 2.8,
            initial_thickness: 0.22,
            leaf_probability: 0.5,
            gravity: 0.0,
            species: TreeSpecies::Spruce,
            branch_segments: 2,
            radial_segments: 4,
        }
    }

    /// Generate the L-System string after N iterations
    pub fn generate_string(&self) -> String {
        let mut current = self.axiom.clone();

        for _ in 0..self.iterations {
            let mut next = String::new();

            for ch in current.chars() {
                if let Some(replacement) = self.rules.get(&ch) {
                    next.push_str(replacement);
                } else {
                    next.push(ch);
                }
            }

            current = next;
        }

        current
    }
}

/// Turtle state for interpreting L-System commands
#[derive(Debug, Clone)]
struct TurtleState {
    position: Vec3,
    direction: Vec3,
    up: Vec3,
    right: Vec3,
    length: f32,
    thickness: f32,
}

impl TurtleState {
    fn new(recipe: &TreeRecipe) -> Self {
        TurtleState {
            position: Vec3::ZERO,
            direction: Vec3::Y,
            up: Vec3::Z,
            right: Vec3::X,
            length: recipe.initial_length,
            thickness: recipe.initial_thickness,
        }
    }

    fn rotate_right(&mut self, angle: f32) {
        let rotation = Quat::from_axis_angle(self.up, angle);
        self.direction = rotation * self.direction;
        self.right = rotation * self.right;
    }

    fn rotate_up(&mut self, angle: f32) {
        let rotation = Quat::from_axis_angle(self.right, angle);
        self.direction = rotation * self.direction;
        self.up = rotation * self.up;
    }

    fn rotate_roll(&mut self, angle: f32) {
        let rotation = Quat::from_axis_angle(self.direction, angle);
        self.up = rotation * self.up;
        self.right = rotation * self.right;
    }
}

/// A single branch segment with position and thickness
#[derive(Debug, Clone)]
pub struct BranchSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub start_thickness: f32,
    pub end_thickness: f32,
}

/// Generated tree structure
#[derive(Debug, Clone)]
pub struct GeneratedTree {
    pub branches: Vec<BranchSegment>,
    pub leaves: Vec<LeafInstance>,
    pub recipe: TreeRecipe,
}

/// Leaf instance position and orientation
#[derive(Debug, Clone)]
pub struct LeafInstance {
    pub position: Vec3,
    pub normal: Vec3,
    pub size: f32,
}

/// Generate a tree from a recipe
pub fn generate_tree(recipe: &TreeRecipe, seed: u64) -> GeneratedTree {
    let lsystem_string = recipe.generate_string();
    let mut turtle = TurtleState::new(recipe);
    let mut state_stack: Vec<TurtleState> = Vec::new();
    let mut branches = Vec::new();
    let mut leaves = Vec::new();

    // Simple RNG using seed
    let mut rng_state = seed;
    let mut random = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (rng_state >> 32) as f32 / u32::MAX as f32
    };

    for ch in lsystem_string.chars() {
        match ch {
            'F' | 'G' => {
                // Move forward and draw branch
                let start = turtle.position;
                let end = turtle.position + turtle.direction * turtle.length;

                // Apply gravity effect
                let gravity_offset = Vec3::new(0.0, recipe.gravity * turtle.length, 0.0);
                let end = end + gravity_offset;

                branches.push(BranchSegment {
                    start,
                    end,
                    start_thickness: turtle.thickness,
                    end_thickness: turtle.thickness * recipe.thickness_decay,
                });

                turtle.position = end;
                turtle.length *= recipe.length_decay;
                turtle.thickness *= recipe.thickness_decay;

                // Possibly place a leaf
                if random() < recipe.leaf_probability && turtle.thickness < 0.05 {
                    leaves.push(LeafInstance {
                        position: end,
                        normal: turtle.direction,
                        size: 0.2 + random() * 0.3,
                    });
                }
            }
            'f' => {
                // Move forward without drawing
                turtle.position += turtle.direction * turtle.length;
            }
            '+' => {
                // Rotate right (yaw)
                turtle.rotate_right(recipe.angle);
            }
            '-' => {
                // Rotate left (yaw)
                turtle.rotate_right(-recipe.angle);
            }
            '&' => {
                // Pitch down
                turtle.rotate_up(-recipe.angle);
            }
            '^' => {
                // Pitch up
                turtle.rotate_up(recipe.angle);
            }
            '\\' => {
                // Roll left
                turtle.rotate_roll(-recipe.angle);
            }
            '/' => {
                // Roll right
                turtle.rotate_roll(recipe.angle);
            }
            '[' => {
                // Push state
                state_stack.push(turtle.clone());
            }
            ']' => {
                // Pop state
                if let Some(state) = state_stack.pop() {
                    turtle = state;
                }
            }
            'L' => {
                // Explicit leaf command
                leaves.push(LeafInstance {
                    position: turtle.position,
                    normal: turtle.direction,
                    size: 0.5 + random() * 0.5,
                });
            }
            _ => {
                // Ignore unknown characters
            }
        }
    }

    GeneratedTree {
        branches,
        leaves,
        recipe: recipe.clone(),
    }
}

/// Vertex data for tree mesh
#[derive(Debug, Clone, Copy)]
pub struct TreeVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Generated tree mesh with vertex and index data
#[derive(Debug, Clone)]
pub struct TreeMesh {
    pub vertices: Vec<TreeVertex>,
    pub indices: Vec<u32>,
}

/// Generate a cylindrical mesh from tree branches
pub fn generate_tree_mesh(tree: &GeneratedTree) -> TreeMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let radial_segments = tree.recipe.radial_segments as usize;

    for branch in &tree.branches {
        let base_index = vertices.len() as u32;

        // Direction and perpendicular vectors
        let direction = (branch.end - branch.start).normalize();

        // Find perpendicular vector
        let arbitrary = if direction.y.abs() > 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let tangent = direction.cross(arbitrary).normalize();
        let bitangent = direction.cross(tangent).normalize();

        // Generate ring of vertices at start and end
        for ring in 0..2 {
            let position = if ring == 0 { branch.start } else { branch.end };
            let thickness = if ring == 0 { branch.start_thickness } else { branch.end_thickness };
            let v_coord = ring as f32;

            for i in 0..radial_segments {
                let angle = (i as f32 / radial_segments as f32) * std::f32::consts::TAU;
                let cos = angle.cos();
                let sin = angle.sin();

                let normal = (tangent * cos + bitangent * sin).normalize();
                let vertex_pos = position + normal * thickness;

                vertices.push(TreeVertex {
                    position: vertex_pos.to_array(),
                    normal: normal.to_array(),
                    uv: [i as f32 / radial_segments as f32, v_coord],
                });
            }
        }

        // Generate triangles connecting the rings
        for i in 0..radial_segments {
            let next = (i + 1) % radial_segments;

            let i0 = base_index + i as u32;
            let i1 = base_index + next as u32;
            let i2 = base_index + radial_segments as u32 + i as u32;
            let i3 = base_index + radial_segments as u32 + next as u32;

            // Two triangles per quad
            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    // Generate leaf billboards
    // DISABLED for performance/style
    /*
    for leaf in &tree.leaves {
        let base_index = vertices.len() as u32;

        // Create billboard facing up
        let right = Vec3::X;
        let up = Vec3::Z;
        let half_size = leaf.size * 0.5;

        let positions = [
            leaf.position + (-right - up) * half_size,
            leaf.position + (right - up) * half_size,
            leaf.position + (right + up) * half_size,
            leaf.position + (-right + up) * half_size,
        ];

        let uvs = [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ];

        for i in 0..4 {
            vertices.push(TreeVertex {
                position: positions[i].to_array(),
                normal: leaf.normal.to_array(),
                uv: uvs[i],
            });
        }

        // Two triangles for the leaf quad
        indices.push(base_index);
        indices.push(base_index + 1);
        indices.push(base_index + 2);

        indices.push(base_index);
        indices.push(base_index + 2);
        indices.push(base_index + 3);
    }
    */

    TreeMesh {
        vertices,
        indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsystem_generation() {
        let recipe = TreeRecipe::oak();
        let lsystem = recipe.generate_string();
        assert!(!lsystem.is_empty());
        assert!(lsystem.len() > recipe.axiom.len());
    }

    #[test]
    fn test_tree_generation() {
        let recipe = TreeRecipe::pine();
        let tree = generate_tree(&recipe, 12345);
        assert!(!tree.branches.is_empty());
    }

    #[test]
    fn test_mesh_generation() {
        let recipe = TreeRecipe::oak();
        let tree = generate_tree(&recipe, 54321);
        let mesh = generate_tree_mesh(&tree);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0); // Must be triangles
    }

    #[test]
    fn test_all_species() {
        let recipes = vec![
            TreeRecipe::oak(),
            TreeRecipe::pine(),
            TreeRecipe::willow(),
            TreeRecipe::birch(),
            TreeRecipe::palm(),
            TreeRecipe::maple(),
            TreeRecipe::spruce(),
        ];

        for recipe in recipes {
            let tree = generate_tree(&recipe, 99999);
            assert!(!tree.branches.is_empty());
            let mesh = generate_tree_mesh(&tree);
            assert!(!mesh.vertices.is_empty());
        }
    }
}
