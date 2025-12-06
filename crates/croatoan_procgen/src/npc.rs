//! # NPC (Non-Player Character) Generation
//!
//! This module provides procedural generation of Native American village inhabitants,
//! including their physical appearance, clothing, adornments, and names.
//!
//! ## Quick Start
//!
//! ```rust
//! use croatoan_procgen::{NpcRecipe, NpcId, Gender, generate_npc, generate_npc_mesh};
//!
//! // Generate a village chief
//! let recipe = NpcRecipe::chief(42);
//! let npc = generate_npc(&recipe, NpcId(1));
//!
//! println!("{} - {} {:?}",
//!     npc.name,
//!     match npc.appearance.gender {
//!         Gender::Male => "Male",
//!         Gender::Female => "Female",
//!     },
//!     npc.role
//! );
//!
//! // Generate a renderable mesh
//! let mesh = generate_npc_mesh(&npc.appearance);
//! ```
//!
//! ## NPC Roles
//!
//! - **Chief**: Village leader (elder male, feather headdress)
//! - **Shaman**: Spiritual leader (elder, any gender, tattoos)
//! - **Warrior**: Combat-ready (adult male, mohawk, war paint)
//! - **Farmer**: Tends crops (any adult)
//! - **Villager**: General population
//!
//! ## Appearance System
//!
//! NPCs have procedurally generated:
//! - **Physical traits**: Height, build, skin tone
//! - **Hair**: Style (long, braided, mohawk, etc.) and color
//! - **Clothing**: Breechcloth, dress, leggings, robes, moccasins
//! - **Adornments**: Feathers, beads, war paint, tattoos, jewelry
//!
//! ## Name Generation
//!
//! Names use authentic syllable patterns from Iroquoian languages:
//! - 2-3 syllables per name
//! - Gender-specific syllable pools
//! - Examples: "Tawenho", "Awakoya", "Moheda"

use glam::Vec3;

/// NPC unique identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NpcId(pub u64);

/// Gender for appearance generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

/// Age category affects appearance and behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeCategory {
    Child,      // 5-12
    Youth,      // 13-18
    Adult,      // 19-50
    Elder,      // 51+
}

/// Body build type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyBuild {
    Slim,
    Average,
    Stocky,
}

/// NPC role in the village
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcRole {
    Chief,
    Shaman,
    Warrior,
    Hunter,
    Farmer,
    Craftsperson,
    Elder,
    Child,
    Villager,
}

/// Hair style options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HairStyle {
    Long,           // Loose long hair
    Braided,        // Single or double braids
    Mohawk,         // Traditional roach
    Shaved,         // Shaved sides
    Topknot,        // Tied up top
    Short,          // Short cut
}

/// Clothing types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClothingType {
    Breechcloth,    // Male basic
    Leggings,       // Both
    Tunic,          // Both
    Dress,          // Female
    Robe,           // Elder/ceremonial
    Moccasins,      // Footwear
}

/// Adornment items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adornment {
    Feather,
    Beads,
    WarPaint,
    Tattoo,
    Necklace,
    Bracelet,
    Earring,
    Headband,
}

/// Physical appearance parameters
#[derive(Debug, Clone)]
pub struct NpcAppearance {
    pub gender: Gender,
    pub age_category: AgeCategory,
    pub height: f32,
    pub build: BodyBuild,
    pub skin_tone: [f32; 3],
    pub hair_style: HairStyle,
    pub hair_color: [f32; 3],
    pub face_seed: u32,
    pub clothing: Vec<ClothingType>,
    pub adornments: Vec<Adornment>,
}

/// Recipe for generating an NPC
#[derive(Debug, Clone)]
pub struct NpcRecipe {
    pub role: NpcRole,
    pub gender: Option<Gender>,
    pub age: Option<AgeCategory>,
    pub seed: u32,
}

impl Default for NpcRecipe {
    fn default() -> Self {
        NpcRecipe {
            role: NpcRole::Villager,
            gender: None,
            age: None,
            seed: 0,
        }
    }
}

impl NpcRecipe {
    pub fn villager(seed: u32) -> Self {
        NpcRecipe { role: NpcRole::Villager, seed, ..Default::default() }
    }

    pub fn farmer(seed: u32) -> Self {
        NpcRecipe { role: NpcRole::Farmer, seed, ..Default::default() }
    }

    pub fn chief(seed: u32) -> Self {
        NpcRecipe {
            role: NpcRole::Chief,
            age: Some(AgeCategory::Elder),
            gender: Some(Gender::Male),
            seed,
        }
    }

    pub fn shaman(seed: u32) -> Self {
        NpcRecipe {
            role: NpcRole::Shaman,
            age: Some(AgeCategory::Elder),
            gender: None, // Shaman can be any gender
            seed,
        }
    }

    pub fn warrior(seed: u32) -> Self {
        NpcRecipe {
            role: NpcRole::Warrior,
            age: Some(AgeCategory::Adult),
            gender: Some(Gender::Male),
            seed,
        }
    }
}

/// Generated NPC data
#[derive(Debug, Clone)]
pub struct NpcData {
    pub id: NpcId,
    pub name: String,
    pub appearance: NpcAppearance,
    pub role: NpcRole,
    pub position: glam::Vec2, // World position (x, z) - height determined at render time
}

/// Skin tone palette (warm brown tones)
const SKIN_TONES: [[f32; 3]; 5] = [
    [0.72, 0.55, 0.42],  // Light copper
    [0.65, 0.48, 0.36],  // Medium copper
    [0.58, 0.42, 0.32],  // Bronze
    [0.52, 0.38, 0.28],  // Dark bronze
    [0.45, 0.32, 0.24],  // Deep brown
];

/// Hair color palette
const HAIR_COLORS: [[f32; 3]; 3] = [
    [0.08, 0.06, 0.05],  // Black
    [0.15, 0.10, 0.08],  // Dark brown
    [0.25, 0.18, 0.12],  // Brown (rare, elder gray added separately)
];

/// Name syllables for procedural names
const NAME_SYLLABLES_MALE: [&str; 20] = [
    "Ta", "Wa", "Ki", "Mo", "Ha", "Ne", "So", "Ke", "O", "A",
    "hon", "kan", "wen", "ta", "da", "ko", "na", "wa", "he", "yo",
];

const NAME_SYLLABLES_FEMALE: [&str; 20] = [
    "A", "O", "Ka", "Te", "Wa", "Ya", "Ne", "Hi", "Sa", "Mi",
    "wen", "da", "na", "ko", "ya", "wa", "ni", "ta", "he", "la",
];

/// Simple LCG random number generator
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 32) as f32 / u32::MAX as f32
    }

    fn next_int(&mut self, max: u32) -> usize {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.state >> 32) as u32 % max) as usize
    }
}

/// Generate an NPC from a recipe
pub fn generate_npc(recipe: &NpcRecipe, id: NpcId) -> NpcData {
    let mut rng = Rng::new(recipe.seed as u64 ^ (id.0 << 16));

    // Determine gender
    let gender = recipe.gender.unwrap_or_else(|| {
        if rng.next() < 0.5 { Gender::Male } else { Gender::Female }
    });

    // Determine age
    let age_category = recipe.age.unwrap_or_else(|| {
        match rng.next_int(10) {
            0..=1 => AgeCategory::Child,
            2..=3 => AgeCategory::Youth,
            4..=7 => AgeCategory::Adult,
            _ => AgeCategory::Elder,
        }
    });

    // Generate height based on age and gender
    let base_height = match age_category {
        AgeCategory::Child => 1.0 + rng.next() * 0.3,
        AgeCategory::Youth => 1.4 + rng.next() * 0.2,
        AgeCategory::Adult => match gender {
            Gender::Male => 1.65 + rng.next() * 0.2,
            Gender::Female => 1.55 + rng.next() * 0.15,
        },
        AgeCategory::Elder => match gender {
            Gender::Male => 1.60 + rng.next() * 0.15,
            Gender::Female => 1.50 + rng.next() * 0.12,
        },
    };

    // Generate build
    let build = match rng.next_int(3) {
        0 => BodyBuild::Slim,
        1 => BodyBuild::Average,
        _ => BodyBuild::Stocky,
    };

    // Select skin tone
    let skin_tone = SKIN_TONES[rng.next_int(SKIN_TONES.len() as u32)];

    // Select hair color (gray for elders)
    let hair_color = if age_category == AgeCategory::Elder && rng.next() < 0.6 {
        [0.5, 0.5, 0.5] // Gray
    } else {
        HAIR_COLORS[rng.next_int(HAIR_COLORS.len() as u32)]
    };

    // Select hair style based on gender and role
    let hair_style = match (gender, recipe.role) {
        (Gender::Male, NpcRole::Warrior) => HairStyle::Mohawk,
        (Gender::Male, NpcRole::Chief) => HairStyle::Topknot,
        (Gender::Female, _) => match rng.next_int(3) {
            0 => HairStyle::Long,
            1 => HairStyle::Braided,
            _ => HairStyle::Braided,
        },
        (Gender::Male, _) => match rng.next_int(4) {
            0 => HairStyle::Long,
            1 => HairStyle::Shaved,
            2 => HairStyle::Topknot,
            _ => HairStyle::Short,
        },
    };

    // Generate clothing
    let mut clothing = Vec::new();
    match gender {
        Gender::Male => {
            clothing.push(ClothingType::Breechcloth);
            if rng.next() < 0.6 {
                clothing.push(ClothingType::Leggings);
            }
            if age_category == AgeCategory::Elder || recipe.role == NpcRole::Chief {
                clothing.push(ClothingType::Robe);
            }
        }
        Gender::Female => {
            clothing.push(ClothingType::Dress);
            if rng.next() < 0.4 {
                clothing.push(ClothingType::Leggings);
            }
        }
    }
    clothing.push(ClothingType::Moccasins);

    // Generate adornments based on role
    let mut adornments = Vec::new();
    match recipe.role {
        NpcRole::Chief => {
            adornments.push(Adornment::Feather);
            adornments.push(Adornment::Beads);
            adornments.push(Adornment::Necklace);
        }
        NpcRole::Shaman => {
            adornments.push(Adornment::Feather);
            adornments.push(Adornment::Tattoo);
            adornments.push(Adornment::Necklace);
        }
        NpcRole::Warrior => {
            adornments.push(Adornment::WarPaint);
            if rng.next() < 0.5 {
                adornments.push(Adornment::Feather);
            }
        }
        _ => {
            if rng.next() < 0.3 {
                adornments.push(Adornment::Beads);
            }
            if rng.next() < 0.2 {
                adornments.push(Adornment::Earring);
            }
        }
    }

    // Generate name
    let name = generate_name(gender, &mut rng);

    NpcData {
        id,
        name,
        appearance: NpcAppearance {
            gender,
            age_category,
            height: base_height,
            build,
            skin_tone,
            hair_style,
            hair_color,
            face_seed: recipe.seed,
            clothing,
            adornments,
        },
        role: recipe.role,
        position: glam::Vec2::ZERO, // Will be set when placed in village
    }
}

fn generate_name(gender: Gender, rng: &mut Rng) -> String {
    let syllables = match gender {
        Gender::Male => &NAME_SYLLABLES_MALE,
        Gender::Female => &NAME_SYLLABLES_FEMALE,
    };

    let syllable_count = 2 + rng.next_int(2); // 2-3 syllables
    let mut name = String::new();

    for i in 0..syllable_count {
        let idx = rng.next_int(syllables.len() as u32);
        let syllable = syllables[idx];

        if i == 0 {
            name.push_str(syllable);
        } else {
            // Lowercase for subsequent syllables
            name.push_str(&syllable.to_lowercase());
        }
    }

    name
}

/// Vertex data for NPC mesh (simplified capsule/billboard)
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct NpcVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
}

/// Simple NPC mesh (capsule representation)
#[derive(Debug, Clone)]
pub struct NpcMesh {
    pub vertices: Vec<NpcVertex>,
    pub indices: Vec<u32>,
}

/// Generate a simple capsule mesh for NPC representation
pub fn generate_npc_mesh(appearance: &NpcAppearance) -> NpcMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let height = appearance.height;
    let radius = match appearance.build {
        BodyBuild::Slim => 0.18,
        BodyBuild::Average => 0.22,
        BodyBuild::Stocky => 0.28,
    };

    // Body color (clothing)
    let body_color = match appearance.gender {
        Gender::Male => [0.55, 0.45, 0.35],   // Tan deerskin
        Gender::Female => [0.50, 0.40, 0.32], // Slightly different
    };

    // Generate capsule
    let segments = 8;
    let rings = 6;

    // Cylinder body
    for ring in 0..rings {
        let y0 = radius + (ring as f32 / rings as f32) * (height - radius * 2.0);
        let y1 = radius + ((ring + 1) as f32 / rings as f32) * (height - radius * 2.0);

        for seg in 0..segments {
            let angle0 = (seg as f32 / segments as f32) * std::f32::consts::TAU;
            let angle1 = ((seg + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let x0 = radius * angle0.cos();
            let z0 = radius * angle0.sin();
            let x1 = radius * angle1.cos();
            let z1 = radius * angle1.sin();

            let base = vertices.len() as u32;
            let normal0 = Vec3::new(angle0.cos(), 0.0, angle0.sin());
            let normal1 = Vec3::new(angle1.cos(), 0.0, angle1.sin());

            vertices.push(NpcVertex {
                position: [x0, y0, z0],
                normal: normal0.to_array(),
                uv: [seg as f32 / segments as f32, ring as f32 / rings as f32],
                color: body_color,
            });
            vertices.push(NpcVertex {
                position: [x1, y0, z1],
                normal: normal1.to_array(),
                uv: [(seg + 1) as f32 / segments as f32, ring as f32 / rings as f32],
                color: body_color,
            });
            vertices.push(NpcVertex {
                position: [x1, y1, z1],
                normal: normal1.to_array(),
                uv: [(seg + 1) as f32 / segments as f32, (ring + 1) as f32 / rings as f32],
                color: body_color,
            });
            vertices.push(NpcVertex {
                position: [x0, y1, z0],
                normal: normal0.to_array(),
                uv: [seg as f32 / segments as f32, (ring + 1) as f32 / rings as f32],
                color: body_color,
            });

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    // Head (sphere at top)
    let head_y = height - radius * 0.8;
    let head_radius = radius * 0.7;

    for lat in 0..4 {
        let lat0 = (lat as f32 / 4.0) * std::f32::consts::FRAC_PI_2;
        let lat1 = ((lat + 1) as f32 / 4.0) * std::f32::consts::FRAC_PI_2;

        for lon in 0..segments {
            let lon0 = (lon as f32 / segments as f32) * std::f32::consts::TAU;
            let lon1 = ((lon + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let base = vertices.len() as u32;

            let positions = [
                Vec3::new(
                    head_radius * lat0.cos() * lon0.cos(),
                    head_y + head_radius * lat0.sin(),
                    head_radius * lat0.cos() * lon0.sin(),
                ),
                Vec3::new(
                    head_radius * lat0.cos() * lon1.cos(),
                    head_y + head_radius * lat0.sin(),
                    head_radius * lat0.cos() * lon1.sin(),
                ),
                Vec3::new(
                    head_radius * lat1.cos() * lon1.cos(),
                    head_y + head_radius * lat1.sin(),
                    head_radius * lat1.cos() * lon1.sin(),
                ),
                Vec3::new(
                    head_radius * lat1.cos() * lon0.cos(),
                    head_y + head_radius * lat1.sin(),
                    head_radius * lat1.cos() * lon0.sin(),
                ),
            ];

            for pos in &positions {
                let normal = (*pos - Vec3::new(0.0, head_y, 0.0)).normalize();
                vertices.push(NpcVertex {
                    position: pos.to_array(),
                    normal: normal.to_array(),
                    uv: [0.0, 0.0],
                    color: appearance.skin_tone,
                });
            }

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    NpcMesh { vertices, indices }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_generation() {
        let recipe = NpcRecipe::villager(42);
        let npc = generate_npc(&recipe, NpcId(1));

        assert!(!npc.name.is_empty());
        assert!(npc.appearance.height > 1.0);
        assert!(npc.appearance.height < 2.0);
    }

    #[test]
    fn test_chief_generation() {
        let recipe = NpcRecipe::chief(123);
        let npc = generate_npc(&recipe, NpcId(2));

        assert_eq!(npc.appearance.gender, Gender::Male);
        assert_eq!(npc.appearance.age_category, AgeCategory::Elder);
        assert!(npc.appearance.adornments.contains(&Adornment::Feather));
    }

    #[test]
    fn test_mesh_generation() {
        let recipe = NpcRecipe::villager(42);
        let npc = generate_npc(&recipe, NpcId(1));
        let mesh = generate_npc_mesh(&npc.appearance);

        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }
}
