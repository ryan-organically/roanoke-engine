use glam::{Vec3, Vec2};
use std::collections::HashMap;

/// Architectural style for the building
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchStyle {
    Colonial,
    Modern,
    Rustic,
}

/// Parameters for procedural building generation
#[derive(Debug, Clone)]
pub struct BuildingRecipe {
    pub style: ArchStyle,
    pub floors: u32,
    pub width: f32,
    pub depth: f32,
    pub seed: u32,
    pub floor_height: f32,
    pub roof_height: f32,
}

impl Default for BuildingRecipe {
    fn default() -> Self {
        Self::colonial_house()
    }
}

impl BuildingRecipe {
    pub fn colonial_house() -> Self {
        BuildingRecipe {
            style: ArchStyle::Colonial,
            floors: 2,
            width: 8.0,
            depth: 6.0,
            seed: 0,
            floor_height: 3.0,
            roof_height: 2.5,
        }
    }

    pub fn small_shack() -> Self {
        BuildingRecipe {
            style: ArchStyle::Rustic,
            floors: 1,
            width: 5.0,
            depth: 4.0,
            seed: 0,
            floor_height: 2.5,
            roof_height: 1.5,
        }
    }
}

/// Vertex data for building mesh
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct BuildingVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3], // Added color for simple material differentiation
}

/// Generated building mesh
#[derive(Debug, Clone)]
pub struct BuildingMesh {
    pub vertices: Vec<BuildingVertex>,
    pub indices: Vec<u32>,
}

/// Generate a building mesh from a recipe using a simple Shape Grammar
pub fn generate_building(recipe: &BuildingRecipe) -> BuildingMesh {
    let mut builder = MeshBuilder::new();
    
    // RNG (Linear Congruential Generator)
    let mut rng_state = recipe.seed as u64;
    let mut random = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (rng_state >> 32) as f32 / u32::MAX as f32
    };

    let half_w = recipe.width * 0.5;
    let half_d = recipe.depth * 0.5;

    // 1. Foundation
    builder.add_box(
        Vec3::new(0.0, 0.2, 0.0), // Center (slightly raised)
        Vec3::new(recipe.width + 0.2, 0.4, recipe.depth + 0.2), // Size
        [0.4, 0.4, 0.4], // Stone gray
    );

    // Porch (Colonial/Rustic only)
    let has_porch = (recipe.style == ArchStyle::Colonial || recipe.style == ArchStyle::Rustic) && random() > 0.3;
    if has_porch {
        let porch_depth = 2.0;
        let porch_z = half_d + porch_depth * 0.5;
        // Porch floor
        builder.add_box(
            Vec3::new(0.0, 0.2, porch_z),
            Vec3::new(recipe.width, 0.4, porch_depth),
            [0.45, 0.35, 0.25], // Wood deck
        );
        // Porch roof (extension of main roof or separate)
        let porch_roof_y = 0.4 + recipe.floor_height * 0.8;
        builder.add_box(
            Vec3::new(0.0, porch_roof_y, porch_z),
            Vec3::new(recipe.width + 0.2, 0.2, porch_depth + 0.2),
            [0.35, 0.2, 0.15], // Dark wood roof
        );
        // Columns
        let col_x = half_w - 0.2;
        builder.add_box(Vec3::new(-col_x, porch_roof_y * 0.5, porch_z + porch_depth * 0.4), Vec3::new(0.3, porch_roof_y, 0.3), [0.9, 0.9, 0.9]);
        builder.add_box(Vec3::new( col_x, porch_roof_y * 0.5, porch_z + porch_depth * 0.4), Vec3::new(0.3, porch_roof_y, 0.3), [0.9, 0.9, 0.9]);
    }

    // 2. Floors (Walls)
    for i in 0..recipe.floors {
        let y_base = 0.4 + i as f32 * recipe.floor_height;
        
        // Main box for the floor
        builder.add_box(
            Vec3::new(0.0, y_base + recipe.floor_height * 0.5, 0.0),
            Vec3::new(recipe.width, recipe.floor_height, recipe.depth),
            match recipe.style {
                ArchStyle::Colonial => [0.9, 0.9, 0.85], // White/Cream clapboard
                ArchStyle::Rustic => [0.55, 0.4, 0.25], // Wood
                ArchStyle::Modern => [0.8, 0.8, 0.85], // Concrete/Glass
            }
        );

        // Add Windows/Doors
        // Front face (Z+)
        let window_spacing = 2.0;
        let num_windows = (recipe.width / window_spacing).floor() as i32 - 1;
        
        for w in 0..num_windows {
             let x_offset = -half_w + window_spacing + (w as f32 * window_spacing);
             
             // Ground floor center = Door
             if i == 0 && (x_offset).abs() < 1.0 {
                 // Door Frame
                 builder.add_box(
                     Vec3::new(x_offset, y_base + 1.0, half_d + 0.05),
                     Vec3::new(1.4, 2.2, 0.15),
                     [0.3, 0.2, 0.1], // Dark wood frame
                 );
                 // Door
                 builder.add_box(
                     Vec3::new(x_offset, y_base + 1.0, half_d + 0.08),
                     Vec3::new(1.0, 2.0, 0.1),
                     [0.4, 0.25, 0.15], // Door panel
                 );
             } else {
                 // Window Frame
                 builder.add_box(
                     Vec3::new(x_offset, y_base + 1.5, half_d + 0.05),
                     Vec3::new(1.2, 1.4, 0.1),
                     [0.8, 0.8, 0.8], // White frame
                 );
                 // Window Glass
                 builder.add_box(
                     Vec3::new(x_offset, y_base + 1.5, half_d + 0.06),
                     Vec3::new(1.0, 1.2, 0.1),
                     [0.2, 0.3, 0.5], // Blueish glass
                 );
                 // Sill
                 builder.add_box(
                     Vec3::new(x_offset, y_base + 0.9, half_d + 0.1),
                     Vec3::new(1.3, 0.1, 0.2),
                     [0.8, 0.8, 0.8], // White sill
                 );
             }
        }
    }

    // 3. Roof
    let roof_base_y = 0.4 + recipe.floors as f32 * recipe.floor_height;
    match recipe.style {
        ArchStyle::Colonial | ArchStyle::Rustic => {
            // Pitched Roof (Triangular prism)
            // Overhang
            let overhang = 0.6;
            builder.add_prism(
                Vec3::new(0.0, roof_base_y, 0.0),
                recipe.width + overhang * 2.0, 
                recipe.depth + overhang * 2.0,
                recipe.roof_height,
                [0.35, 0.15, 0.15], // Red/Brown shingles
            );
        }
        ArchStyle::Modern => {
            // Flat roof with parapet
            builder.add_box(
                Vec3::new(0.0, roof_base_y + 0.1, 0.0),
                Vec3::new(recipe.width + 0.4, 0.2, recipe.depth + 0.4),
                [0.2, 0.2, 0.2], // Dark gray trim
            );
            // Skylight
            builder.add_box(
                 Vec3::new(0.0, roof_base_y + 0.2, 0.0),
                 Vec3::new(recipe.width * 0.5, 0.2, recipe.depth * 0.5),
                 [0.4, 0.5, 0.6], // Glass
            );
        }
    }

    // 4. Chimney (if Colonial/Rustic)
    if recipe.style != ArchStyle::Modern {
        let chimney_pos = Vec3::new(half_w - 1.0, 0.0, 0.0);
        let chimney_height = roof_base_y + recipe.roof_height + 0.5;
        builder.add_box(
            Vec3::new(chimney_pos.x, chimney_height * 0.5, chimney_pos.z),
            Vec3::new(0.8, chimney_height, 0.8),
            [0.5, 0.25, 0.2], // Brick red
        );
        // Chimney Cap
        builder.add_box(
            Vec3::new(chimney_pos.x, chimney_height, chimney_pos.z),
            Vec3::new(1.0, 0.2, 1.0),
            [0.3, 0.3, 0.3], // Stone cap
        );
    }

    BuildingMesh {
        vertices: builder.vertices,
        indices: builder.indices,
    }
}

// --- Mesh Builder Helper ---

struct MeshBuilder {
    vertices: Vec<BuildingVertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_box(&mut self, center: Vec3, size: Vec3, color: [f32; 3]) {
        let half = size * 0.5;
        let base_idx = self.vertices.len() as u32;

        // 8 corners
        let p = [
            Vec3::new(-half.x, -half.y,  half.z), // 0: Front Bottom Left
            Vec3::new( half.x, -half.y,  half.z), // 1: Front Bottom Right
            Vec3::new( half.x,  half.y,  half.z), // 2: Front Top Right
            Vec3::new(-half.x,  half.y,  half.z), // 3: Front Top Left
            Vec3::new(-half.x, -half.y, -half.z), // 4: Back Bottom Left
            Vec3::new( half.x, -half.y, -half.z), // 5: Back Bottom Right
            Vec3::new( half.x,  half.y, -half.z), // 6: Back Top Right
            Vec3::new(-half.x,  half.y, -half.z), // 7: Back Top Left
        ];

        // Normals
        let n = [
            Vec3::Z,  // Front
            Vec3::NEG_Z, // Back
            Vec3::Y,  // Top
            Vec3::NEG_Y, // Bottom
            Vec3::X,  // Right
            Vec3::NEG_X, // Left
        ];

        // Add faces (duplicated vertices for sharp normals)
        // Front
        self.add_quad(center + p[0], center + p[1], center + p[2], center + p[3], n[0], color);
        // Back
        self.add_quad(center + p[5], center + p[4], center + p[7], center + p[6], n[1], color);
        // Top
        self.add_quad(center + p[3], center + p[2], center + p[6], center + p[7], n[2], color);
        // Bottom
        self.add_quad(center + p[4], center + p[5], center + p[1], center + p[0], n[3], color);
        // Right
        self.add_quad(center + p[1], center + p[5], center + p[6], center + p[2], n[4], color);
        // Left
        self.add_quad(center + p[4], center + p[0], center + p[3], center + p[7], n[5], color);
    }

    fn add_prism(&mut self, base_center: Vec3, width: f32, depth: f32, height: f32, color: [f32; 3]) {
        let half_w = width * 0.5;
        let half_d = depth * 0.5;
        
        // Vertices relative to base_center
        let v_front_left = base_center + Vec3::new(-half_w, 0.0, half_d);
        let v_front_right = base_center + Vec3::new(half_w, 0.0, half_d);
        let v_back_left = base_center + Vec3::new(-half_w, 0.0, -half_d);
        let v_back_right = base_center + Vec3::new(half_w, 0.0, -half_d);
        let v_top_front = base_center + Vec3::new(0.0, height, half_d);
        let v_top_back = base_center + Vec3::new(0.0, height, -half_d);

        // Slopes
        let slope_left_normal = Vec3::new(-height, half_w, 0.0).normalize();
        let slope_right_normal = Vec3::new(height, half_w, 0.0).normalize();

        // Left Slope
        self.add_quad(v_front_left, v_back_left, v_top_back, v_top_front, slope_left_normal, color);
        // Right Slope
        self.add_quad(v_back_right, v_front_right, v_top_front, v_top_back, slope_right_normal, color);
        // Front Gable (Triangle)
        self.add_tri(v_front_left, v_front_right, v_top_front, Vec3::Z, color);
        // Back Gable (Triangle)
        self.add_tri(v_back_right, v_back_left, v_top_back, Vec3::NEG_Z, color);
        // Bottom (optional, usually hidden)
        self.add_quad(v_back_left, v_back_right, v_front_right, v_front_left, Vec3::NEG_Y, color);
    }

    fn add_quad(&mut self, v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3, color: [f32; 3]) {
        let base = self.vertices.len() as u32;
        
        self.vertices.push(BuildingVertex { position: v0.to_array(), normal: normal.to_array(), uv: [0.0, 1.0], color });
        self.vertices.push(BuildingVertex { position: v1.to_array(), normal: normal.to_array(), uv: [1.0, 1.0], color });
        self.vertices.push(BuildingVertex { position: v2.to_array(), normal: normal.to_array(), uv: [1.0, 0.0], color });
        self.vertices.push(BuildingVertex { position: v3.to_array(), normal: normal.to_array(), uv: [0.0, 0.0], color });

        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn add_tri(&mut self, v0: Vec3, v1: Vec3, v2: Vec3, normal: Vec3, color: [f32; 3]) {
        let base = self.vertices.len() as u32;

        self.vertices.push(BuildingVertex { position: v0.to_array(), normal: normal.to_array(), uv: [0.0, 0.0], color });
        self.vertices.push(BuildingVertex { position: v1.to_array(), normal: normal.to_array(), uv: [1.0, 0.0], color });
        self.vertices.push(BuildingVertex { position: v2.to_array(), normal: normal.to_array(), uv: [0.5, 1.0], color });

        self.indices.extend_from_slice(&[base, base + 1, base + 2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_gen() {
        let recipe = BuildingRecipe::colonial_house();
        let mesh = generate_building(&recipe);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }
}
