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
            radial_segments: 6, // Increased from 4 for smoother cylinders
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
            radial_segments: 6,
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
            radial_segments: 6,
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
            radial_segments: 6,
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
            radial_segments: 6,
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
            radial_segments: 6,
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
            radial_segments: 6,
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

                // Place leaves on thin terminal branches
                if random() < recipe.leaf_probability && turtle.thickness < 0.08 {
                    leaves.push(LeafInstance {
                        position: end,
                        normal: turtle.direction,
                        size: 0.3 + random() * 0.4, // Slightly larger leaves
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
                // Explicit leaf command (used by palm trees)
                leaves.push(LeafInstance {
                    position: turtle.position,
                    normal: turtle.direction,
                    size: 0.6 + random() * 0.5,
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
    // Leaves use UV.y > 1.0 as a flag for the shader to render them green
    for (i, leaf) in tree.leaves.iter().enumerate() {
        let base_index = vertices.len() as u32;

        // Create billboard oriented along branch direction with some randomization
        // Use leaf index for pseudo-random variation
        let seed = i as f32 * 0.7;
        let angle = seed * 2.5; // Rotation around branch axis

        // Build basis from leaf normal (branch direction)
        let forward = leaf.normal.normalize();
        let arbitrary = if forward.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
        let right = forward.cross(arbitrary).normalize();
        let up = right.cross(forward).normalize();

        // Rotate the basis for variety
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let rotated_right = right * cos_a + up * sin_a;
        let rotated_up = up * cos_a - right * sin_a;

        let half_size = leaf.size * 0.5;

        // Offset slightly along branch direction so leaves extend outward
        let offset = forward * half_size * 0.3;

        let positions = [
            leaf.position + offset + (-rotated_right - rotated_up) * half_size,
            leaf.position + offset + (rotated_right - rotated_up) * half_size,
            leaf.position + offset + (rotated_right + rotated_up) * half_size,
            leaf.position + offset + (-rotated_right + rotated_up) * half_size,
        ];

        // UV.y = 2.0 marks this as a leaf for the shader
        let uvs = [
            [0.0, 2.0],
            [1.0, 2.0],
            [1.0, 3.0],
            [0.0, 3.0],
        ];

        // Normal points mostly up with slight variation for lighting
        let leaf_normal = (Vec3::Y * 0.7 + forward * 0.3).normalize();

        for j in 0..4 {
            vertices.push(TreeVertex {
                position: positions[j].to_array(),
                normal: leaf_normal.to_array(),
                uv: uvs[j],
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

    TreeMesh {
        vertices,
        indices,
    }
}

//=============================================================================
// SIMPLE LOW-POLY TREE GENERATOR
//=============================================================================
// Generates a simple cylinder trunk + icosphere canopy tree
// Total: ~200-300 triangles vs 94K from complex OBJ
// Perfect for LOD1 or as primary tree mesh for performance

/// Generate a simple low-poly tree mesh (cylinder trunk + icosphere canopy)
///
/// This creates a tree with approximately 200-300 triangles:
/// - Trunk: cylinder with 8 sides = ~48 triangles
/// - Canopy: icosphere with 1 subdivision = ~80 triangles
/// - Total: ~130 triangles per tree
///
/// # Arguments
/// * `trunk_height` - Height of the trunk (default ~3.0)
/// * `trunk_radius` - Radius of the trunk (default ~0.3)
/// * `canopy_radius` - Radius of the canopy sphere (default ~2.5)
/// * `seed` - Seed for slight variation
pub fn generate_simple_tree_mesh(
    trunk_height: f32,
    trunk_radius: f32,
    canopy_radius: f32,
    seed: u64,
) -> TreeMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Simple RNG for variation
    let mut rng_state = seed;
    let mut random = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((rng_state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0 // -1 to 1
    };

    // Add slight variation based on seed
    let height_var = 1.0 + random() * 0.15;
    let canopy_var = 1.0 + random() * 0.1;
    let trunk_h = trunk_height * height_var;
    let canopy_r = canopy_radius * canopy_var;

    //=========================================================================
    // TRUNK (Cylinder with 8 sides)
    //=========================================================================
    let trunk_segments = 8;
    let trunk_rings = 2; // Just top and bottom

    // Generate trunk vertices
    for ring in 0..trunk_rings {
        let y = if ring == 0 { 0.0 } else { trunk_h };
        let radius = if ring == 0 { trunk_radius * 1.1 } else { trunk_radius * 0.85 }; // Slight taper

        for i in 0..trunk_segments {
            let angle = (i as f32 / trunk_segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            let normal = Vec3::new(angle.cos(), 0.0, angle.sin()).normalize();

            vertices.push(TreeVertex {
                position: [x, y, z],
                normal: normal.to_array(),
                uv: [i as f32 / trunk_segments as f32, y / trunk_h],
            });
        }
    }

    // Generate trunk indices (connect rings)
    for i in 0..trunk_segments {
        let next = (i + 1) % trunk_segments;
        let i0 = i as u32;
        let i1 = next as u32;
        let i2 = (trunk_segments + i) as u32;
        let i3 = (trunk_segments + next) as u32;

        indices.push(i0);
        indices.push(i2);
        indices.push(i1);

        indices.push(i1);
        indices.push(i2);
        indices.push(i3);
    }

    //=========================================================================
    // CANOPY (Icosphere with subdivision level 1)
    //=========================================================================
    let canopy_center_y = trunk_h + canopy_r * 0.4; // Slightly above trunk
    let base_vertex_count = vertices.len() as u32;

    // Golden ratio for icosahedron
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let icosa_scale = canopy_r / (1.0 + phi * phi).sqrt();

    // 12 vertices of icosahedron
    let icosa_verts = [
        Vec3::new(-1.0, phi, 0.0) * icosa_scale,
        Vec3::new(1.0, phi, 0.0) * icosa_scale,
        Vec3::new(-1.0, -phi, 0.0) * icosa_scale,
        Vec3::new(1.0, -phi, 0.0) * icosa_scale,
        Vec3::new(0.0, -1.0, phi) * icosa_scale,
        Vec3::new(0.0, 1.0, phi) * icosa_scale,
        Vec3::new(0.0, -1.0, -phi) * icosa_scale,
        Vec3::new(0.0, 1.0, -phi) * icosa_scale,
        Vec3::new(phi, 0.0, -1.0) * icosa_scale,
        Vec3::new(phi, 0.0, 1.0) * icosa_scale,
        Vec3::new(-phi, 0.0, -1.0) * icosa_scale,
        Vec3::new(-phi, 0.0, 1.0) * icosa_scale,
    ];

    // Add icosahedron vertices (offset to canopy position)
    for v in &icosa_verts {
        let pos = *v + Vec3::new(0.0, canopy_center_y, 0.0);
        let normal = v.normalize();

        vertices.push(TreeVertex {
            position: pos.to_array(),
            normal: normal.to_array(),
            uv: [0.5 + normal.x * 0.5, 2.0 + normal.y * 0.5], // UV.y > 1 marks as leaf
        });
    }

    // 20 faces of icosahedron
    let icosa_faces: [[u32; 3]; 20] = [
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ];

    for face in &icosa_faces {
        indices.push(base_vertex_count + face[0]);
        indices.push(base_vertex_count + face[1]);
        indices.push(base_vertex_count + face[2]);
    }

    TreeMesh {
        vertices,
        indices,
    }
}

/// Generate a default simple tree with standard proportions
pub fn generate_default_simple_tree(seed: u64) -> TreeMesh {
    generate_simple_tree_mesh(3.5, 0.25, 2.2, seed)
}

//=============================================================================
// ENHANCED PROCEDURAL TREE GENERATOR
//=============================================================================
// Creates more natural-looking trees with:
// - Tapered trunk with optional curve
// - Multi-layer canopy clusters for fuller appearance
// - Branch stubs for visual depth
// - Noise-based organic variation
// Total: ~400-800 triangles depending on settings

/// Tree style presets for different visual appearances
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProceduralTreeStyle {
    /// Classic deciduous tree - round, full canopy (oak-like)
    Deciduous,
    /// Conifer - tall, conical canopy (pine/spruce-like)
    Conifer,
    /// Spreading tree - wide, flattened canopy (acacia-like)
    Spreading,
    /// Columnar - tall, narrow canopy (cypress-like)
    Columnar,
}

/// Configuration for enhanced procedural tree generation
#[derive(Debug, Clone)]
pub struct ProceduralTreeConfig {
    /// Overall height of the tree (trunk + canopy)
    pub height: f32,
    /// Trunk radius at base
    pub trunk_radius: f32,
    /// How much the trunk tapers (0.0 = no taper, 1.0 = full taper to point)
    pub trunk_taper: f32,
    /// Number of canopy layers/clusters
    pub canopy_layers: u32,
    /// Base radius of canopy
    pub canopy_radius: f32,
    /// Vertical stretch of canopy (1.0 = sphere, <1.0 = flat, >1.0 = tall)
    pub canopy_vertical_scale: f32,
    /// Number of branch stubs to add
    pub branch_count: u32,
    /// Trunk curve amount (0.0 = straight)
    pub trunk_curve: f32,
    /// Polygon detail (segments around trunk/canopy)
    pub detail_level: u32,
    /// Tree style preset
    pub style: ProceduralTreeStyle,
}

impl Default for ProceduralTreeConfig {
    fn default() -> Self {
        Self::deciduous()
    }
}

impl ProceduralTreeConfig {
    /// Classic deciduous tree (oak/maple style) - FULL CANOPY from mid-trunk to top
    pub fn deciduous() -> Self {
        Self {
            height: 8.0,           // Taller tree
            trunk_radius: 0.35,
            trunk_taper: 0.5,
            canopy_layers: 8,      // MORE layers for fuller canopy
            canopy_radius: 3.0,    // Wider canopy
            canopy_vertical_scale: 0.7, // Flatter clusters
            branch_count: 6,       // More branch stubs
            trunk_curve: 0.15,
            detail_level: 8,
            style: ProceduralTreeStyle::Deciduous,
        }
    }

    /// Conifer tree (pine/spruce style) - tiered branches from low to high
    pub fn conifer() -> Self {
        Self {
            height: 12.0,          // Taller
            trunk_radius: 0.3,
            trunk_taper: 0.35,
            canopy_layers: 10,     // Many tiers
            canopy_radius: 2.5,    // Wider base
            canopy_vertical_scale: 0.5, // Flatter layers (tiers)
            branch_count: 0,       // Conifers have branch tiers instead
            trunk_curve: 0.0,
            detail_level: 6,
            style: ProceduralTreeStyle::Conifer,
        }
    }

    /// Spreading tree (acacia/oak style) - wide horizontal canopy
    pub fn spreading() -> Self {
        Self {
            height: 6.0,
            trunk_radius: 0.4,
            trunk_taper: 0.6,
            canopy_layers: 6,      // More layers spread horizontally
            canopy_radius: 5.0,    // Very wide
            canopy_vertical_scale: 0.35, // Very flat
            branch_count: 8,       // Lots of visible branches
            trunk_curve: 0.25,
            detail_level: 8,
            style: ProceduralTreeStyle::Spreading,
        }
    }

    /// Columnar tree (cypress/poplar style) - tall narrow
    pub fn columnar() -> Self {
        Self {
            height: 14.0,
            trunk_radius: 0.2,
            trunk_taper: 0.15,
            canopy_layers: 8,
            canopy_radius: 1.0,
            canopy_vertical_scale: 3.0,
            branch_count: 0,
            trunk_curve: 0.0,
            detail_level: 6,
            style: ProceduralTreeStyle::Columnar,
        }
    }

    /// Dense forest tree - medium size, very full canopy for forest interiors
    pub fn forest_dense() -> Self {
        Self {
            height: 7.0,
            trunk_radius: 0.3,
            trunk_taper: 0.45,
            canopy_layers: 10,     // LOTS of layers
            canopy_radius: 2.8,
            canopy_vertical_scale: 0.8,
            branch_count: 5,
            trunk_curve: 0.1,
            detail_level: 6,
            style: ProceduralTreeStyle::Deciduous,
        }
    }
}

/// Generate an enhanced procedural tree with natural-looking multi-layer canopy
///
/// Creates trees with ~400-800 triangles featuring:
/// - Tapered, optionally curved trunk
/// - Multiple overlapping canopy spheroids for fuller appearance
/// - Branch stubs emerging from trunk
/// - Noise-based vertex displacement for organic variation
///
/// # Arguments
/// * `config` - Tree configuration parameters
/// * `seed` - Seed for deterministic variation
///
/// # Returns
/// TreeMesh with vertices and indices ready for GPU upload
pub fn generate_enhanced_tree(config: &ProceduralTreeConfig, seed: u64) -> TreeMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // PCG-style RNG - returns -1.0 to 1.0
    let mut rng_state = seed;
    let mut random = || -> f32 {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((rng_state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0
    };

    // Apply seed-based variation to config (convert -1..1 to 0..1 inline)
    let height = config.height * (0.85 + (random() + 1.0) * 0.5 * 0.3);
    let trunk_radius = config.trunk_radius * (0.9 + (random() + 1.0) * 0.5 * 0.2);
    let canopy_radius = config.canopy_radius * (0.9 + (random() + 1.0) * 0.5 * 0.2);

    // Calculate trunk height (canopy starts partway up)
    let trunk_height = match config.style {
        ProceduralTreeStyle::Deciduous => height * 0.45,
        ProceduralTreeStyle::Conifer => height * 0.25,
        ProceduralTreeStyle::Spreading => height * 0.55,
        ProceduralTreeStyle::Columnar => height * 0.15,
    };

    let segments = config.detail_level as usize;
    let trunk_rings = 4;

    //=========================================================================
    // TRUNK - Tapered cylinder with optional curve
    //=========================================================================
    for ring in 0..trunk_rings {
        let t = ring as f32 / (trunk_rings - 1) as f32;
        let y = t * trunk_height;

        // Taper radius from base to top
        let radius = trunk_radius * (1.0 - t * config.trunk_taper);

        // Optional trunk curve (lean)
        let curve_offset = config.trunk_curve * t * t * 0.5;

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius + curve_offset;
            let z = angle.sin() * radius;

            // Add slight noise for organic feel
            let noise = random() * 0.02 * trunk_radius;

            let normal = Vec3::new(angle.cos(), 0.0, angle.sin()).normalize();

            vertices.push(TreeVertex {
                position: [x + noise, y, z + noise],
                normal: normal.to_array(),
                uv: [i as f32 / segments as f32, t], // UV.y < 1.0 = bark
            });
        }
    }

    // Connect trunk rings
    for ring in 0..(trunk_rings - 1) {
        for i in 0..segments {
            let next = (i + 1) % segments;
            let base = ring * segments;
            let i0 = (base + i) as u32;
            let i1 = (base + next) as u32;
            let i2 = (base + segments + i) as u32;
            let i3 = (base + segments + next) as u32;

            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    //=========================================================================
    // BRANCH STUBS - Short cylinders emerging from trunk
    //=========================================================================
    for b in 0..config.branch_count {
        let branch_base = vertices.len() as u32;

        // Position along trunk (upper half)
        let branch_t = 0.4 + (b as f32 / config.branch_count as f32) * 0.5;
        let branch_y = branch_t * trunk_height;
        let branch_angle = (b as f32 / config.branch_count as f32) * std::f32::consts::TAU
            + random() * 0.5;

        // Branch direction (outward and slightly up)
        let branch_dir = Vec3::new(
            branch_angle.cos(),
            0.3 + random() * 0.2,
            branch_angle.sin(),
        ).normalize();

        let branch_length = trunk_radius * 3.0 * (0.8 + (random() + 1.0) * 0.5 * 0.4);
        let branch_radius = trunk_radius * 0.3;

        // Branch start and end
        let trunk_surface_offset = trunk_radius * (1.0 - branch_t * config.trunk_taper);
        let start = Vec3::new(
            branch_angle.cos() * trunk_surface_offset,
            branch_y,
            branch_angle.sin() * trunk_surface_offset,
        );
        let end = start + branch_dir * branch_length;

        // Simple 4-sided branch
        let up = Vec3::Y;
        let right = branch_dir.cross(up).normalize();
        let branch_up = right.cross(branch_dir).normalize();

        // Start cap vertices
        for i in 0..4 {
            let a = (i as f32 / 4.0) * std::f32::consts::TAU;
            let offset = right * a.cos() * branch_radius + branch_up * a.sin() * branch_radius;
            let pos = start + offset;
            let normal = offset.normalize();

            vertices.push(TreeVertex {
                position: pos.to_array(),
                normal: normal.to_array(),
                uv: [i as f32 / 4.0, 0.5], // Bark UV
            });
        }

        // End cap vertices (tapered)
        for i in 0..4 {
            let a = (i as f32 / 4.0) * std::f32::consts::TAU;
            let offset = right * a.cos() * branch_radius * 0.5 + branch_up * a.sin() * branch_radius * 0.5;
            let pos = end + offset;
            let normal = offset.normalize();

            vertices.push(TreeVertex {
                position: pos.to_array(),
                normal: normal.to_array(),
                uv: [i as f32 / 4.0, 0.8], // Bark UV
            });
        }

        // Connect branch cylinder
        for i in 0..4u32 {
            let next = (i + 1) % 4;
            indices.push(branch_base + i);
            indices.push(branch_base + 4 + i);
            indices.push(branch_base + next);

            indices.push(branch_base + next);
            indices.push(branch_base + 4 + i);
            indices.push(branch_base + 4 + next);
        }
    }

    //=========================================================================
    // CANOPY - Multiple overlapping spheroid clusters filling the canopy volume
    //=========================================================================
    // Start canopy LOWER on trunk for fuller appearance
    let canopy_base_y = match config.style {
        ProceduralTreeStyle::Deciduous => trunk_height * 0.35,  // Start at 35% for full canopy
        ProceduralTreeStyle::Spreading => trunk_height * 0.5,
        ProceduralTreeStyle::Conifer => trunk_height * 0.2,     // Conifers start very low
        ProceduralTreeStyle::Columnar => trunk_height * 0.1,
    };

    // Total canopy height (from base to top)
    let canopy_total_height = height - canopy_base_y;

    for layer in 0..config.canopy_layers {
        let layer_base = vertices.len() as u32;
        let layer_t = layer as f32 / config.canopy_layers.max(1) as f32;

        // Position this canopy cluster - distribute throughout the canopy volume
        let (cluster_y, cluster_radius, cluster_height, offset_x, offset_z) = match config.style {
            ProceduralTreeStyle::Deciduous => {
                // FULL CANOPY: Distribute clusters throughout 3D canopy volume
                // Use golden angle for horizontal distribution, varied heights
                let golden_angle = 2.399963; // Golden angle in radians
                let layer_angle = layer as f32 * golden_angle;

                // Vertical position: spread from bottom to top of canopy
                // Use sqrt for more clusters at edges (shell-like distribution)
                let height_factor = (layer_t * 0.8 + 0.1).sqrt(); // 0.3 to 0.95 of canopy height
                let y = canopy_base_y + height_factor * canopy_total_height;

                // Horizontal offset: larger at bottom, smaller at top (tree shape)
                let horizontal_extent = canopy_radius * (1.0 - height_factor * 0.4);
                let dist_from_center = horizontal_extent * (0.3 + layer_t * 0.5);

                let ox = layer_angle.cos() * dist_from_center + random() * 0.3;
                let oz = layer_angle.sin() * dist_from_center + random() * 0.3;

                // Cluster size: varies for natural look
                let r = canopy_radius * (0.4 + random() * 0.15 + (1.0 - height_factor) * 0.3);
                let h = r * config.canopy_vertical_scale;

                (y, r, h, ox, oz)
            }
            ProceduralTreeStyle::Conifer => {
                // CONE SHAPE: Tiered layers getting smaller toward top
                // Each layer is a ring of clusters around the trunk
                let tier = layer / 2; // Two clusters per tier
                let tier_t = tier as f32 / (config.canopy_layers / 2).max(1) as f32;
                let is_offset = layer % 2 == 1; // Alternate clusters

                let y = canopy_base_y + tier_t * canopy_total_height;

                // Radius decreases with height (cone shape)
                let tier_radius = canopy_radius * (1.0 - tier_t * 0.8);

                // Horizontal position: either center or edge
                let angle = if is_offset {
                    tier as f32 * 2.399963 + std::f32::consts::PI
                } else {
                    tier as f32 * 2.399963
                };
                let dist = if is_offset { tier_radius * 0.7 } else { tier_radius * 0.3 };

                let ox = angle.cos() * dist;
                let oz = angle.sin() * dist;

                let r = tier_radius * (0.5 + random() * 0.2);
                let h = r * config.canopy_vertical_scale;

                (y, r, h, ox, oz)
            }
            ProceduralTreeStyle::Spreading => {
                // FLAT UMBRELLA: Clusters spread horizontally at similar heights
                let angle = layer as f32 * 2.399963; // Golden angle
                let dist = canopy_radius * (0.2 + layer_t * 0.7);

                // Height varies slightly but mostly flat
                let y = canopy_base_y + canopy_total_height * 0.3 + random() * canopy_total_height * 0.4;

                let ox = angle.cos() * dist + random() * 0.5;
                let oz = angle.sin() * dist + random() * 0.5;

                let r = canopy_radius * (0.35 + random() * 0.2);
                let h = r * config.canopy_vertical_scale;

                (y, r, h, ox, oz)
            }
            ProceduralTreeStyle::Columnar => {
                // VERTICAL COLUMN: Stack clusters along the trunk
                let y = canopy_base_y + layer_t * canopy_total_height;

                // Slight bulge in middle
                let bulge = 1.0 - (layer_t - 0.5).abs() * 1.5;
                let r = canopy_radius * (0.7 + bulge * 0.4);
                let h = r * config.canopy_vertical_scale;

                let ox = random() * r * 0.2;
                let oz = random() * r * 0.2;

                (y, r, h, ox, oz)
            }
        };

        // Generate deformed sphere for this cluster
        let lat_segments = (segments / 2).max(4);
        let lon_segments = segments;

        for lat in 0..=lat_segments {
            let lat_t = lat as f32 / lat_segments as f32;
            let phi = lat_t * std::f32::consts::PI;

            for lon in 0..=lon_segments {
                let lon_t = lon as f32 / lon_segments as f32;
                let theta = lon_t * std::f32::consts::TAU;

                // Spheroid coordinates
                let mut x = phi.sin() * theta.cos() * cluster_radius;
                let mut y_local = phi.cos() * cluster_height;
                let mut z = phi.sin() * theta.sin() * cluster_radius;

                // Add noise-based displacement for organic shape
                let noise_scale = 0.15;
                let noise_freq = 3.0;
                let noise = ((x * noise_freq + seed as f32 * 0.01).sin()
                    * (z * noise_freq).cos()
                    * (y_local * noise_freq * 0.5).sin()) * noise_scale;

                let displacement = 1.0 + noise + random() * 0.05;
                x *= displacement;
                z *= displacement;

                // Flatten bottom slightly
                if y_local < 0.0 {
                    y_local *= 0.7;
                }

                let pos = Vec3::new(
                    x + offset_x,
                    y_local + cluster_y,
                    z + offset_z,
                );

                let normal = Vec3::new(x, y_local / config.canopy_vertical_scale, z).normalize();

                vertices.push(TreeVertex {
                    position: pos.to_array(),
                    normal: normal.to_array(),
                    uv: [lon_t, 2.0 + lat_t], // UV.y > 1.0 = canopy/leaf
                });
            }
        }

        // Generate indices for this cluster
        for lat in 0..lat_segments {
            for lon in 0..lon_segments {
                let row_size = (lon_segments + 1) as u32;
                let i0 = layer_base + lat as u32 * row_size + lon as u32;
                let i1 = i0 + 1;
                let i2 = i0 + row_size;
                let i3 = i2 + 1;

                if lat > 0 {
                    indices.push(i0);
                    indices.push(i2);
                    indices.push(i1);
                }
                if lat < lat_segments - 1 {
                    indices.push(i1);
                    indices.push(i2);
                    indices.push(i3);
                }
            }
        }
    }

    TreeMesh { vertices, indices }
}

/// Generate a deciduous tree with default settings
pub fn generate_deciduous_tree(seed: u64) -> TreeMesh {
    generate_enhanced_tree(&ProceduralTreeConfig::deciduous(), seed)
}

/// Generate a conifer tree with default settings
pub fn generate_conifer_tree(seed: u64) -> TreeMesh {
    generate_enhanced_tree(&ProceduralTreeConfig::conifer(), seed)
}

/// Generate a spreading tree with default settings
pub fn generate_spreading_tree(seed: u64) -> TreeMesh {
    generate_enhanced_tree(&ProceduralTreeConfig::spreading(), seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tree_generation() {
        let mesh = generate_simple_tree_mesh(3.0, 0.3, 2.0, 12345);
        println!("Simple tree: {} vertices, {} triangles",
            mesh.vertices.len(), mesh.indices.len() / 3);
        assert!(mesh.vertices.len() < 100, "Simple tree should have < 100 vertices");
        assert!(mesh.indices.len() / 3 < 200, "Simple tree should have < 200 triangles");
    }

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

    #[test]
    fn test_enhanced_deciduous_tree() {
        let mesh = generate_deciduous_tree(12345);
        println!("Enhanced deciduous: {} vertices, {} triangles",
            mesh.vertices.len(), mesh.indices.len() / 3);
        assert!(mesh.vertices.len() > 100, "Enhanced tree should have > 100 vertices");
        assert!(mesh.indices.len() / 3 > 200, "Enhanced tree should have > 200 triangles");
        assert_eq!(mesh.indices.len() % 3, 0, "Must be valid triangles");
    }

    #[test]
    fn test_enhanced_conifer_tree() {
        let mesh = generate_conifer_tree(54321);
        println!("Enhanced conifer: {} vertices, {} triangles",
            mesh.vertices.len(), mesh.indices.len() / 3);
        assert!(!mesh.vertices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn test_enhanced_spreading_tree() {
        let mesh = generate_spreading_tree(99999);
        println!("Enhanced spreading: {} vertices, {} triangles",
            mesh.vertices.len(), mesh.indices.len() / 3);
        assert!(!mesh.vertices.is_empty());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn test_all_enhanced_styles() {
        let configs = vec![
            ProceduralTreeConfig::deciduous(),
            ProceduralTreeConfig::conifer(),
            ProceduralTreeConfig::spreading(),
            ProceduralTreeConfig::columnar(),
        ];

        for config in configs {
            let mesh = generate_enhanced_tree(&config, 11111);
            println!("{:?} tree: {} verts, {} tris",
                config.style, mesh.vertices.len(), mesh.indices.len() / 3);
            assert!(!mesh.vertices.is_empty());
            assert!(!mesh.indices.is_empty());
            assert_eq!(mesh.indices.len() % 3, 0);
        }
    }

    #[test]
    fn test_tree_variation_with_seed() {
        // Different seeds should produce different trees
        let mesh1 = generate_deciduous_tree(100);
        let mesh2 = generate_deciduous_tree(200);

        // Same structure but positions should differ slightly
        assert_eq!(mesh1.vertices.len(), mesh2.vertices.len());

        // At least some vertices should be different
        let mut different_count = 0;
        for (v1, v2) in mesh1.vertices.iter().zip(mesh2.vertices.iter()) {
            if v1.position != v2.position {
                different_count += 1;
            }
        }
        assert!(different_count > 0, "Different seeds should produce different trees");
    }
}
