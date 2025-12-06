//! # Village Layout Generation
//!
//! This module generates complete Native American village layouts including longhouses,
//! fire pits, corn fields, prayer sites, and NPCs.
//!
//! ## Quick Start
//!
//! ```rust
//! use croatoan_procgen::{VillageRecipe, VillageId, generate_village};
//! use glam::Vec3;
//!
//! let recipe = VillageRecipe::medium_village(42);
//! let village = generate_village(Vec3::new(100.0, 10.0, 100.0), &recipe, VillageId(1));
//!
//! println!("Village '{}' has:", village.name);
//! println!("  {} longhouses", village.longhouses.len());
//! println!("  {} fire pits", village.fire_pits.len());
//! println!("  {} corn fields", village.corn_fields.len());
//! println!("  {} NPCs", village.npcs.len());
//! ```
//!
//! ## Village Layout
//!
//! Villages are organized in concentric zones:
//!
//! 1. **Center**: Ceremonial fire pit with dance circle
//! 2. **Inner Ring**: Longhouses arranged in an oval (25-50m radius)
//! 3. **Middle Ring**: Domestic fire pits near each longhouse
//! 4. **Outer Ring**: Corn fields (Three Sisters agriculture)
//! 5. **Cardinal Points**: Prayer sites (sunrise knoll, ancestor shrines)
//!
//! ## Village Sizes
//!
//! | Size | Population | Longhouses | Corn Fields |
//! |------|------------|------------|-------------|
//! | Small Camp | 15 | 3 | 2 |
//! | Medium Village | 35 | 4-5 | 3-4 |
//! | Large Village | 60 | 6-8 | 4-5 |
//!
//! ## Corn Fields
//!
//! Fields use the Three Sisters agricultural method:
//! - Corn, beans, and squash grown together on mounds
//! - Mounds spaced ~3m apart in rows
//! - 5 growth stages: Sprout → Young → Growing → Tasseling → Mature
//!
//! ## Fire Pits
//!
//! - **Ceremonial** (central): Large (1.5m), 16 stones, log pile, dance circle
//! - **Domestic** (per longhouse): Small (0.8m), 10 stones, simple

use glam::{Vec2, Vec3};
use crate::longhouse::{LonghouseRecipe, LonghouseStyle, LonghouseMesh, generate_longhouse};
use crate::npc::{NpcId, NpcRecipe, NpcData, generate_npc};

/// Unique identifier for villages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VillageId(pub u64);

/// Fire pit for ceremonies and warmth
#[derive(Debug, Clone)]
pub struct FirePit {
    pub position: Vec3,
    pub radius: f32,
    pub is_ceremonial: bool,
    pub dance_circle_radius: f32,
}

/// Corn field for farming
#[derive(Debug, Clone)]
pub struct CornField {
    pub position: Vec3,
    pub size: Vec2,
    pub rows: u32,
    pub mounds: Vec<Vec3>,
}

/// Prayer site location
#[derive(Debug, Clone)]
pub struct PrayerSite {
    pub position: Vec3,
    pub facing_direction: Vec3,
    pub site_type: PrayerSiteType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrayerSiteType {
    SunriseKnoll,
    AncestorShrine,
    WaterEdge,
    SacredTree,
}

/// Placement of a longhouse in the village
#[derive(Debug, Clone)]
pub struct LonghousePlacement {
    pub position: Vec3,
    pub rotation: f32,
    pub recipe: LonghouseRecipe,
    pub clan_name: String,
    pub mesh: LonghouseMesh,
}

/// Complete village layout
#[derive(Debug, Clone)]
pub struct VillageLayout {
    pub id: VillageId,
    pub center: Vec3,
    pub name: String,
    pub longhouses: Vec<LonghousePlacement>,
    pub fire_pits: Vec<FirePit>,
    pub corn_fields: Vec<CornField>,
    pub prayer_sites: Vec<PrayerSite>,
    pub npcs: Vec<NpcData>,
    pub bounds_radius: f32,
}

/// Recipe for generating a village
#[derive(Debug, Clone)]
pub struct VillageRecipe {
    pub population: u32,
    pub seed: u32,
    pub style: LonghouseStyle,
}

impl Default for VillageRecipe {
    fn default() -> Self {
        VillageRecipe {
            population: 30,
            seed: 0,
            style: LonghouseStyle::Iroquoian,
        }
    }
}

impl VillageRecipe {
    pub fn small_camp(seed: u32) -> Self {
        VillageRecipe {
            population: 15,
            seed,
            style: LonghouseStyle::Iroquoian,
        }
    }

    pub fn medium_village(seed: u32) -> Self {
        VillageRecipe {
            population: 35,
            seed,
            style: LonghouseStyle::Iroquoian,
        }
    }

    pub fn large_village(seed: u32) -> Self {
        VillageRecipe {
            population: 60,
            seed,
            style: LonghouseStyle::Iroquoian,
        }
    }
}

/// Clan name components
const CLAN_PREFIXES: [&str; 10] = [
    "Bear", "Wolf", "Turtle", "Deer", "Hawk", "Beaver", "Eel", "Heron", "Snipe", "Eagle"
];

const VILLAGE_NAMES: [&str; 12] = [
    "Kanata", "Ossernenon", "Onondaga", "Cayuga", "Seneca", "Mohawk",
    "Stadacona", "Hochelaga", "Ganondagan", "Tenochtitlan", "Caughnawaga", "Kahnawake"
];

/// Simple deterministic RNG for village generation
struct VillageRng {
    state: u64,
}

impl VillageRng {
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

/// Generate a complete village layout
pub fn generate_village(center: Vec3, recipe: &VillageRecipe, village_id: VillageId) -> VillageLayout {
    let mut rng = VillageRng::new(recipe.seed as u64 ^ (village_id.0 << 24));

    let mut layout = VillageLayout {
        id: village_id,
        center,
        name: VILLAGE_NAMES[rng.next_int(VILLAGE_NAMES.len() as u32)].to_string(),
        longhouses: Vec::new(),
        fire_pits: Vec::new(),
        corn_fields: Vec::new(),
        prayer_sites: Vec::new(),
        npcs: Vec::new(),
        bounds_radius: 0.0,
    };

    // 1. Central ceremonial fire pit
    layout.fire_pits.push(FirePit {
        position: center,
        radius: 1.5,
        is_ceremonial: true,
        dance_circle_radius: 8.0,
    });

    // 2. Calculate longhouse count and arrange in oval
    let longhouse_count = ((recipe.population as f32 / 10.0).ceil() as u32).max(3).min(8);
    let oval_radius = 25.0 + longhouse_count as f32 * 4.0;

    for i in 0..longhouse_count {
        let angle = (i as f32 / longhouse_count as f32) * std::f32::consts::TAU;

        // Oval positioning (wider on X axis)
        let offset = Vec3::new(
            oval_radius * angle.cos() * (1.0 + rng.next() * 0.15),
            0.0,
            oval_radius * angle.sin() * 0.7 * (1.0 + rng.next() * 0.15),
        );

        // Orient longhouse perpendicular to center
        let rotation = angle + std::f32::consts::FRAC_PI_2;

        // Vary size based on position (first is council house, larger)
        let family_units = if i == 0 {
            6 + rng.next_int(3) as u32
        } else {
            3 + rng.next_int(3) as u32
        };

        let longhouse_recipe = LonghouseRecipe {
            style: recipe.style,
            family_units,
            width: 6.0 + rng.next() * 1.0,
            height: 5.0 + rng.next() * 1.0,
            seed: recipe.seed.wrapping_add(i),
        };

        let mesh = generate_longhouse(&longhouse_recipe);

        layout.longhouses.push(LonghousePlacement {
            position: center + offset,
            rotation,
            recipe: longhouse_recipe,
            clan_name: format!("{} Clan", CLAN_PREFIXES[rng.next_int(CLAN_PREFIXES.len() as u32)]),
            mesh,
        });

        // Add small fire pit near each longhouse
        let fire_offset = Vec3::new(
            offset.x * 0.6,
            0.0,
            offset.z * 0.6,
        );
        layout.fire_pits.push(FirePit {
            position: center + fire_offset,
            radius: 0.8,
            is_ceremonial: false,
            dance_circle_radius: 0.0,
        });
    }

    // 3. Corn fields outside the longhouse ring
    let field_count = ((recipe.population as f32 / 12.0).ceil() as u32).max(2).min(5);
    let field_radius = oval_radius + 35.0;

    for i in 0..field_count {
        let angle = (i as f32 / field_count as f32) * std::f32::consts::TAU
                  + std::f32::consts::FRAC_PI_4; // Offset from longhouses

        let offset = Vec3::new(
            field_radius * angle.cos(),
            0.0,
            field_radius * angle.sin() * 0.8,
        );

        let field_width = 18.0 + rng.next() * 12.0;
        let field_depth = 12.0 + rng.next() * 8.0;
        let rows = 6 + rng.next_int(4) as u32;

        // Generate mound positions (Three Sisters style)
        let mut mounds = Vec::new();
        let mound_spacing = field_width / rows as f32;

        for row in 0..rows {
            let mounds_in_row = 4 + rng.next_int(3);
            for col in 0..mounds_in_row {
                let mx = -field_width * 0.5 + row as f32 * mound_spacing + rng.next() * 0.5;
                let mz = -field_depth * 0.5 + col as f32 * (field_depth / mounds_in_row as f32) + rng.next() * 0.5;
                mounds.push(center + offset + Vec3::new(mx, 0.0, mz));
            }
        }

        layout.corn_fields.push(CornField {
            position: center + offset,
            size: Vec2::new(field_width, field_depth),
            rows,
            mounds,
        });
    }

    // 4. Prayer sites at cardinal directions
    let prayer_radius = oval_radius + 15.0;

    // East - sunrise prayers
    layout.prayer_sites.push(PrayerSite {
        position: center + Vec3::new(prayer_radius, 0.0, 0.0),
        facing_direction: Vec3::X,
        site_type: PrayerSiteType::SunriseKnoll,
    });

    // West - ancestor shrine
    layout.prayer_sites.push(PrayerSite {
        position: center + Vec3::new(-prayer_radius, 0.0, 0.0),
        facing_direction: Vec3::NEG_X,
        site_type: PrayerSiteType::AncestorShrine,
    });

    // North - sacred tree
    layout.prayer_sites.push(PrayerSite {
        position: center + Vec3::new(0.0, 0.0, -prayer_radius * 0.7),
        facing_direction: Vec3::NEG_Z,
        site_type: PrayerSiteType::SacredTree,
    });

    // South - secondary shrine
    layout.prayer_sites.push(PrayerSite {
        position: center + Vec3::new(0.0, 0.0, prayer_radius * 0.7),
        facing_direction: Vec3::Z,
        site_type: PrayerSiteType::AncestorShrine,
    });

    // 5. Generate NPCs with positions distributed around village
    let mut npc_id_counter = village_id.0 * 1000;
    let village_x = center.x;
    let village_z = center.z;

    // Helper to generate NPC position in village
    let npc_position = |idx: u32, seed: u32, radius: f32| -> glam::Vec2 {
        let angle = (idx as f32 * 2.399) + (seed as f32 * 0.01); // Golden angle distribution
        let r = radius * (0.3 + (((seed + idx) % 100) as f32 / 100.0) * 0.7);
        glam::Vec2::new(
            village_x + angle.cos() * r,
            village_z + angle.sin() * r,
        )
    };

    // Chief (near center)
    let mut chief = generate_npc(&NpcRecipe::chief(recipe.seed), NpcId(npc_id_counter));
    chief.position = npc_position(0, recipe.seed, 5.0);
    layout.npcs.push(chief);
    npc_id_counter += 1;

    // Shaman (near prayer area)
    let mut shaman = generate_npc(&NpcRecipe::shaman(recipe.seed + 1), NpcId(npc_id_counter));
    shaman.position = npc_position(1, recipe.seed + 1, prayer_radius * 0.5);
    layout.npcs.push(shaman);
    npc_id_counter += 1;

    // Warriors (10% of population) - near edges
    let warrior_count = (recipe.population as f32 * 0.1) as u32;
    for i in 0..warrior_count {
        let mut warrior = generate_npc(
            &NpcRecipe::warrior(recipe.seed + 100 + i),
            NpcId(npc_id_counter)
        );
        warrior.position = npc_position(i + 2, recipe.seed + 100 + i, oval_radius * 1.2);
        layout.npcs.push(warrior);
        npc_id_counter += 1;
    }

    // Farmers (30% of population) - near corn fields
    let farmer_count = (recipe.population as f32 * 0.3) as u32;
    for i in 0..farmer_count {
        let mut farmer = generate_npc(
            &NpcRecipe::farmer(recipe.seed + 200 + i),
            NpcId(npc_id_counter)
        );
        farmer.position = npc_position(i + 2 + warrior_count, recipe.seed + 200 + i, field_radius * 0.8);
        layout.npcs.push(farmer);
        npc_id_counter += 1;
    }

    // Remaining villagers - distributed throughout
    let remaining = recipe.population.saturating_sub(2 + warrior_count + farmer_count);
    for i in 0..remaining {
        let mut villager = generate_npc(
            &NpcRecipe::villager(recipe.seed + 500 + i),
            NpcId(npc_id_counter)
        );
        villager.position = npc_position(i + 2 + warrior_count + farmer_count, recipe.seed + 500 + i, oval_radius);
        layout.npcs.push(villager);
        npc_id_counter += 1;
    }

    // Calculate bounds
    layout.bounds_radius = field_radius + 30.0;

    layout
}

/// Fire pit mesh generation
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct FirePitVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct FirePitMesh {
    pub vertices: Vec<FirePitVertex>,
    pub indices: Vec<u32>,
}

/// Generate fire pit mesh
pub fn generate_fire_pit(fire_pit: &FirePit) -> FirePitMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let stone_count = if fire_pit.is_ceremonial { 16 } else { 10 };
    let stone_radius = fire_pit.radius;

    // Stone ring
    for i in 0..stone_count {
        let angle = (i as f32 / stone_count as f32) * std::f32::consts::TAU;
        let x = stone_radius * angle.cos();
        let z = stone_radius * angle.sin();

        // Each stone is a small box
        let stone_size = if fire_pit.is_ceremonial { 0.25 } else { 0.18 };
        add_box(
            &mut vertices,
            &mut indices,
            Vec3::new(x, stone_size * 0.5, z),
            Vec3::new(stone_size, stone_size, stone_size * 0.8),
            [0.45, 0.42, 0.40], // Gray stone
        );
    }

    // Ash bed in center
    let ash_segments = 12;
    let center_base = vertices.len() as u32;

    vertices.push(FirePitVertex {
        position: [0.0, 0.02, 0.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.5, 0.5],
        color: [0.12, 0.10, 0.08], // Dark ash
    });

    for i in 0..ash_segments {
        let angle = (i as f32 / ash_segments as f32) * std::f32::consts::TAU;
        let x = (stone_radius - 0.15) * angle.cos();
        let z = (stone_radius - 0.15) * angle.sin();

        vertices.push(FirePitVertex {
            position: [x, 0.02, z],
            normal: [0.0, 1.0, 0.0],
            uv: [(x + 1.0) * 0.5, (z + 1.0) * 0.5],
            color: [0.15, 0.12, 0.10],
        });

        let next = (i + 1) % ash_segments;
        indices.extend_from_slice(&[
            center_base,
            center_base + 1 + i as u32,
            center_base + 1 + next as u32,
        ]);
    }

    // Log pile (3 logs in triangle)
    if fire_pit.is_ceremonial {
        for i in 0..3 {
            let angle = (i as f32 / 3.0) * std::f32::consts::TAU + 0.3;
            let log_x = 0.4 * angle.cos();
            let log_z = 0.4 * angle.sin();

            add_cylinder(
                &mut vertices,
                &mut indices,
                Vec3::new(log_x - 0.3, 0.08, log_z),
                Vec3::new(log_x + 0.3, 0.08, log_z),
                0.06,
                [0.35, 0.22, 0.12], // Wood
            );
        }
    }

    FirePitMesh { vertices, indices }
}

/// Corn plant mesh
#[derive(Debug, Clone)]
pub struct CornPlantMesh {
    pub vertices: Vec<FirePitVertex>, // Reusing vertex type
    pub indices: Vec<u32>,
}

/// Growth stage for corn
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornGrowthStage {
    Sprout,
    Young,
    Growing,
    Tasseling,
    Mature,
}

/// Generate corn plant mesh at a growth stage
pub fn generate_corn_plant(stage: CornGrowthStage, seed: u32) -> CornPlantMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let mut rng_state = seed as u64;
    let mut random = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (rng_state >> 32) as f32 / u32::MAX as f32
    };

    let (stalk_height, leaf_count, has_tassel, has_ear) = match stage {
        CornGrowthStage::Sprout => (0.15, 2, false, false),
        CornGrowthStage::Young => (0.5, 4, false, false),
        CornGrowthStage::Growing => (1.2, 6, false, false),
        CornGrowthStage::Tasseling => (1.8, 8, true, false),
        CornGrowthStage::Mature => (2.2, 10, true, true),
    };

    let stalk_color = [0.35, 0.55, 0.25]; // Green stalk
    let leaf_color = [0.40, 0.60, 0.28];  // Slightly brighter leaves

    // Main stalk
    add_cylinder(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, stalk_height, 0.0),
        0.02 + stage as u32 as f32 * 0.005,
        stalk_color,
    );

    // Leaves spiraling up stalk
    for i in 0..leaf_count {
        let t = i as f32 / leaf_count as f32;
        let y = stalk_height * 0.2 + t * stalk_height * 0.7;
        let angle = t * std::f32::consts::TAU * 2.5 + random() * 0.5;

        let leaf_length = 0.3 + t * 0.4;
        let leaf_droop = 0.1 + t * 0.15;

        let base = Vec3::new(0.0, y, 0.0);
        let tip = Vec3::new(
            leaf_length * angle.cos(),
            y - leaf_droop,
            leaf_length * angle.sin(),
        );

        // Simplified leaf as flat quad
        let right = Vec3::new(-angle.sin(), 0.0, angle.cos()) * 0.04;
        let v0 = base - right;
        let v1 = base + right;
        let v2 = tip + right * 0.3;
        let v3 = tip - right * 0.3;

        let leaf_base = vertices.len() as u32;
        let normal = (v1 - v0).cross(v2 - v0).normalize();

        vertices.push(FirePitVertex {
            position: v0.to_array(), normal: normal.to_array(), uv: [0.0, 0.0], color: leaf_color
        });
        vertices.push(FirePitVertex {
            position: v1.to_array(), normal: normal.to_array(), uv: [1.0, 0.0], color: leaf_color
        });
        vertices.push(FirePitVertex {
            position: v2.to_array(), normal: normal.to_array(), uv: [1.0, 1.0], color: leaf_color
        });
        vertices.push(FirePitVertex {
            position: v3.to_array(), normal: normal.to_array(), uv: [0.0, 1.0], color: leaf_color
        });

        indices.extend_from_slice(&[leaf_base, leaf_base + 1, leaf_base + 2, leaf_base, leaf_base + 2, leaf_base + 3]);
    }

    // Tassel at top
    if has_tassel {
        let tassel_color = [0.85, 0.75, 0.45]; // Golden tassel
        for i in 0..5 {
            let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
            let tip = Vec3::new(
                0.08 * angle.cos(),
                stalk_height + 0.15,
                0.08 * angle.sin(),
            );
            add_cylinder(
                &mut vertices,
                &mut indices,
                Vec3::new(0.0, stalk_height, 0.0),
                tip,
                0.008,
                tassel_color,
            );
        }
    }

    // Ear of corn
    if has_ear {
        let ear_y = stalk_height * 0.6;
        let ear_color = [0.90, 0.82, 0.55]; // Yellow corn

        add_cylinder(
            &mut vertices,
            &mut indices,
            Vec3::new(0.03, ear_y, 0.0),
            Vec3::new(0.12, ear_y + 0.02, 0.0),
            0.025,
            ear_color,
        );
    }

    CornPlantMesh { vertices, indices }
}

// Helper functions for mesh building
fn add_box(
    vertices: &mut Vec<FirePitVertex>,
    indices: &mut Vec<u32>,
    center: Vec3,
    size: Vec3,
    color: [f32; 3],
) {
    let half = size * 0.5;
    let corners = [
        Vec3::new(-half.x, -half.y,  half.z),
        Vec3::new( half.x, -half.y,  half.z),
        Vec3::new( half.x,  half.y,  half.z),
        Vec3::new(-half.x,  half.y,  half.z),
        Vec3::new(-half.x, -half.y, -half.z),
        Vec3::new( half.x, -half.y, -half.z),
        Vec3::new( half.x,  half.y, -half.z),
        Vec3::new(-half.x,  half.y, -half.z),
    ];

    let faces = [
        ([0, 1, 2, 3], Vec3::Z),
        ([5, 4, 7, 6], Vec3::NEG_Z),
        ([3, 2, 6, 7], Vec3::Y),
        ([4, 5, 1, 0], Vec3::NEG_Y),
        ([1, 5, 6, 2], Vec3::X),
        ([4, 0, 3, 7], Vec3::NEG_X),
    ];

    for (corner_indices, normal) in faces {
        let base = vertices.len() as u32;
        for &ci in &corner_indices {
            vertices.push(FirePitVertex {
                position: (center + corners[ci]).to_array(),
                normal: normal.to_array(),
                uv: [0.0, 0.0],
                color,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn add_cylinder(
    vertices: &mut Vec<FirePitVertex>,
    indices: &mut Vec<u32>,
    start: Vec3,
    end: Vec3,
    radius: f32,
    color: [f32; 3],
) {
    let segments = 6;
    let direction = (end - start).normalize();

    let up = if direction.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let right = direction.cross(up).normalize();
    let forward = right.cross(direction).normalize();

    for i in 0..segments {
        let angle0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let angle1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

        let offset0 = right * angle0.cos() * radius + forward * angle0.sin() * radius;
        let offset1 = right * angle1.cos() * radius + forward * angle1.sin() * radius;

        let v0 = start + offset0;
        let v1 = start + offset1;
        let v2 = end + offset1;
        let v3 = end + offset0;

        let normal = (offset0 + offset1).normalize();
        let base = vertices.len() as u32;

        vertices.push(FirePitVertex { position: v0.to_array(), normal: normal.to_array(), uv: [0.0, 0.0], color });
        vertices.push(FirePitVertex { position: v1.to_array(), normal: normal.to_array(), uv: [1.0, 0.0], color });
        vertices.push(FirePitVertex { position: v2.to_array(), normal: normal.to_array(), uv: [1.0, 1.0], color });
        vertices.push(FirePitVertex { position: v3.to_array(), normal: normal.to_array(), uv: [0.0, 1.0], color });

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_village_generation() {
        let recipe = VillageRecipe::medium_village(42);
        let village = generate_village(Vec3::ZERO, &recipe, VillageId(1));

        assert!(!village.name.is_empty());
        assert!(!village.longhouses.is_empty());
        assert!(!village.fire_pits.is_empty());
        assert!(!village.corn_fields.is_empty());
        assert!(!village.npcs.is_empty());
        assert!(village.bounds_radius > 0.0);
    }

    #[test]
    fn test_fire_pit_mesh() {
        let fire_pit = FirePit {
            position: Vec3::ZERO,
            radius: 1.5,
            is_ceremonial: true,
            dance_circle_radius: 8.0,
        };
        let mesh = generate_fire_pit(&fire_pit);

        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn test_corn_plant_stages() {
        for stage in [
            CornGrowthStage::Sprout,
            CornGrowthStage::Young,
            CornGrowthStage::Growing,
            CornGrowthStage::Tasseling,
            CornGrowthStage::Mature,
        ] {
            let mesh = generate_corn_plant(stage, 42);
            assert!(!mesh.vertices.is_empty());
        }
    }
}
