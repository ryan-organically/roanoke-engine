//! # Village Manager
//!
//! Manages village placement and structure streaming for the game world.
//! Villages are discovered once at world initialization and then structures
//! are streamed per-chunk as the player explores.
//!
//! # FPS Optimization: Cached NPC Instances
//!
//! NPC orb instances are cached and only recomputed when positions change.
//! This eliminates per-frame matrix generation for 80+ NPCs.

use glam::{Vec3, Mat4};
use croatoan_wfc::{
    find_village_sites, generate_world_village, get_village_structures_for_chunk,
    WorldVillage, VillageStructure, VillageStructureType, get_height_at,
};
use croatoan_procgen::{VillageRecipe, VillageId, generate_village, LonghouseStyle};

/// NPC orb instance data type for GPU upload
pub type NpcOrbInstance = ([f32; 4], [f32; 4], [f32; 4], [f32; 4], [f32; 3], f32);

/// NPC orb data for rendering
#[derive(Debug, Clone)]
pub struct NpcOrb {
    pub position: Vec3,
    pub color: [f32; 3],
    pub name: String,
    pub role: String,
}

/// Manages all villages in the world
pub struct VillageManager {
    /// All villages discovered in the world
    pub villages: Vec<WorldVillage>,
    /// World seed for deterministic generation
    seed: u32,
    /// Whether initial village discovery has been done
    initialized: bool,
    /// NPC orbs for visualization
    pub npc_orbs: Vec<NpcOrb>,

    // ========================================================================
    // FPS OPTIMIZATION: Instance Cache System
    // ========================================================================
    /// Cached NPC orb instances for GPU upload
    /// Only regenerated when dirty flag is set
    cached_instances: Vec<NpcOrbInstance>,
    /// Dirty flag - set when NPC positions change
    instances_dirty: bool,
}

impl VillageManager {
    pub fn new(seed: u32) -> Self {
        Self {
            villages: Vec::new(),
            seed,
            initialized: false,
            npc_orbs: Vec::new(),
            cached_instances: Vec::new(),
            instances_dirty: true,
        }
    }

    /// Mark instances as needing regeneration
    /// Call when NPC positions change
    #[inline]
    pub fn mark_instances_dirty(&mut self) {
        self.instances_dirty = true;
    }

    /// Discover villages in a region around the player spawn
    /// Call this once when the world is created/loaded
    pub fn discover_villages(&mut self, center: Vec3, radius: f32, max_villages: u32) {
        if self.initialized {
            return;
        }

        println!("\n[VILLAGE] ========== VILLAGE DISCOVERY ==========");

        // ALWAYS create the main Croatoan village near spawn first
        self.create_croatoan_village(center);

        // Then find additional village sites
        let region_min = Vec3::new(center.x - radius, 0.0, center.z - radius);
        let region_max = Vec3::new(center.x + radius, 0.0, center.z + radius);

        println!("[VILLAGE] Searching for additional sites in ({:.0}, {:.0}) to ({:.0}, {:.0})",
            region_min.x, region_min.z, region_max.x, region_max.z);

        let sites = find_village_sites(self.seed, region_min, region_max, max_villages.saturating_sub(1));

        println!("[VILLAGE] Found {} additional suitable sites", sites.len());

        for (i, site) in sites.iter().enumerate() {
            // Skip sites too close to Croatoan village
            let croatoan_center = Vec3::new(center.x + 150.0, 0.0, center.z + 150.0);
            let dist = ((site.x - croatoan_center.x).powi(2) + (site.z - croatoan_center.z).powi(2)).sqrt();
            if dist < 300.0 {
                println!("[VILLAGE] Skipping site at ({:.1}, {:.1}) - too close to Croatoan", site.x, site.z);
                continue;
            }

            let village = generate_world_village(*site, self.seed, (i + 1) as u64);
            self.add_village_npcs_as_orbs(&village);
            println!("[VILLAGE] Generated '{}' at ({:.1}, {:.1}) with {} longhouses, {} NPCs",
                village.layout.name,
                site.x, site.z,
                village.layout.longhouses.len(),
                village.layout.npcs.len());
            self.villages.push(village);
        }

        self.initialized = true;
        println!("[VILLAGE] Discovery complete: {} villages, {} total NPCs",
            self.villages.len(), self.npc_orbs.len());
        println!("[VILLAGE] ==========================================\n");
    }

    /// Create the main Croatoan village - a large settlement near spawn
    fn create_croatoan_village(&mut self, spawn: Vec3) {
        // Place village 150 units from spawn in positive X/Z direction
        let village_x = spawn.x + 150.0;
        let village_z = spawn.z + 150.0;
        let (height, _) = get_height_at(village_x, village_z, self.seed);
        let village_center = Vec3::new(village_x, height.max(5.0), village_z);

        println!("[VILLAGE] Creating CROATOAN main village at ({:.1}, {:.1}, {:.1})",
            village_center.x, village_center.y, village_center.z);

        // Create a LARGE village with many structures
        let recipe = VillageRecipe {
            population: 80, // Very large population = more structures
            seed: self.seed,
            style: LonghouseStyle::Iroquoian,
        };

        let layout = generate_village(village_center, &recipe, VillageId(0));

        // Override the name to "Croatoan"
        let mut layout = layout;
        layout.name = "Croatoan".to_string();

        println!("[VILLAGE] CROATOAN village generated:");
        println!("  - {} longhouses", layout.longhouses.len());
        println!("  - {} fire pits", layout.fire_pits.len());
        println!("  - {} corn fields with {} mounds total",
            layout.corn_fields.len(),
            layout.corn_fields.iter().map(|f| f.mounds.len()).sum::<usize>());
        println!("  - {} prayer sites", layout.prayer_sites.len());
        println!("  - {} NPCs", layout.npcs.len());

        // Print NPC details
        for npc in &layout.npcs {
            println!("    NPC: {} ({:?}) at ({:.1}, {:.1})",
                npc.name, npc.role, npc.position.x, npc.position.y);
        }

        let bounds_radius = layout.bounds_radius;
        let bounds_min = Vec3::new(village_center.x - bounds_radius, village_center.y - 5.0, village_center.z - bounds_radius);
        let bounds_max = Vec3::new(village_center.x + bounds_radius, village_center.y + 20.0, village_center.z + bounds_radius);

        let village = WorldVillage {
            id: VillageId(0),
            center: village_center,
            layout,
            bounds_min,
            bounds_max,
        };

        // Add NPC orbs for this village
        self.add_village_npcs_as_orbs(&village);

        self.villages.push(village);
    }

    /// Add NPC orbs from a village for visualization
    /// Uses NPC positions stored in NpcData.position (Vec2: x, z)
    fn add_village_npcs_as_orbs(&mut self, village: &WorldVillage) {
        for npc in &village.layout.npcs {
            // Use NPC's stored position (Vec2 with x, y representing world x, z)
            let npc_x = npc.position.x;
            let npc_z = npc.position.y; // Vec2.y is world Z coordinate
            let (height, _) = get_height_at(npc_x, npc_z, self.seed);

            // Color based on role
            let color = match npc.role {
                croatoan_procgen::NpcRole::Chief => [1.0, 0.8, 0.0],      // Gold
                croatoan_procgen::NpcRole::Shaman => [0.6, 0.0, 0.8],    // Purple
                croatoan_procgen::NpcRole::Warrior => [0.8, 0.2, 0.2],   // Red
                croatoan_procgen::NpcRole::Hunter => [0.4, 0.6, 0.2],    // Green
                croatoan_procgen::NpcRole::Farmer => [0.6, 0.4, 0.2],    // Brown
                croatoan_procgen::NpcRole::Craftsperson => [0.3, 0.5, 0.7], // Blue
                croatoan_procgen::NpcRole::Elder => [0.7, 0.7, 0.7],     // Silver
                croatoan_procgen::NpcRole::Child => [1.0, 0.6, 0.8],     // Pink
                croatoan_procgen::NpcRole::Villager => [0.5, 0.5, 0.4],  // Tan
            };

            self.npc_orbs.push(NpcOrb {
                position: Vec3::new(npc_x, height + 1.5, npc_z),
                color,
                name: npc.name.clone(),
                role: format!("{:?}", npc.role),
            });
        }

        // Mark instance cache as dirty since we added NPCs
        self.instances_dirty = true;
    }

    /// Collect village faction data for registration
    /// Returns vectors of (village_id, VillageFaction) and (npc_id, NpcFactionData)
    pub fn collect_faction_data(&self) -> (
        Vec<(u32, crate::progression::faction_integration::VillageFaction)>,
        Vec<(u32, crate::progression::faction_integration::NpcFactionData)>
    ) {
        use crate::progression::faction_integration::{VillageFaction, VillageStatus, NpcFactionData, determine_village_faction};

        let mut village_factions = Vec::new();
        let mut npc_factions = Vec::new();

        for village in &self.villages {
            let village_id = village.id.0 as u32;

            // Determine faction based on village name and location
            let faction = determine_village_faction(
                &village.layout.name,
                village.center,
                self.seed,
            );

            // Croatoan is special - starts with bonus reputation
            let (status, local_rep) = if village.layout.name == "Croatoan" {
                (VillageStatus::Capital, 50) // Player starts with good standing
            } else {
                (VillageStatus::Normal, 0)
            };

            let village_faction = VillageFaction {
                primary_faction: faction,
                influences: Vec::new(),
                local_reputation: local_rep,
                independent: false,
                status,
                clan_name: Some(village.layout.name.clone()),
            };

            village_factions.push((village_id, village_faction));

            // Also collect NPC faction data for each NPC in the village
            for (npc_idx, npc) in village.layout.npcs.iter().enumerate() {
                let npc_id = (village_id * 1000) + npc_idx as u32;
                let npc_role = match npc.role {
                    croatoan_procgen::NpcRole::Elder => crate::npc::npc_manager::NpcRole::Elder,
                    croatoan_procgen::NpcRole::Chief => crate::npc::npc_manager::NpcRole::Chief,
                    croatoan_procgen::NpcRole::Shaman => crate::npc::npc_manager::NpcRole::Shaman,
                    croatoan_procgen::NpcRole::Warrior => crate::npc::npc_manager::NpcRole::Warrior,
                    croatoan_procgen::NpcRole::Hunter => crate::npc::npc_manager::NpcRole::Hunter,
                    croatoan_procgen::NpcRole::Farmer => crate::npc::npc_manager::NpcRole::Farmer,
                    croatoan_procgen::NpcRole::Craftsperson => crate::npc::npc_manager::NpcRole::Craftsperson,
                    croatoan_procgen::NpcRole::Child => crate::npc::npc_manager::NpcRole::Child,
                    croatoan_procgen::NpcRole::Villager => crate::npc::npc_manager::NpcRole::Villager,
                };

                let npc_faction_data = NpcFactionData::from_role(
                    npc_id,
                    npc_role,
                    faction,
                );

                npc_factions.push((npc_id, npc_faction_data));
            }
        }

        println!("[FACTION] Collected faction data for {} villages, {} NPCs",
            village_factions.len(), npc_factions.len());

        (village_factions, npc_factions)
    }

    /// Get all village structures that fall within a chunk
    pub fn get_structures_for_chunk(
        &self,
        chunk_min_x: f32,
        chunk_min_z: f32,
        chunk_size: f32,
    ) -> Vec<VillageStructure> {
        let mut all_structures = Vec::new();

        for village in &self.villages {
            let structures = get_village_structures_for_chunk(
                village,
                chunk_min_x,
                chunk_min_z,
                chunk_size,
                self.seed,
            );
            all_structures.extend(structures);
        }

        all_structures
    }

    /// Get NPC orb instances for rendering with AnimalOrbPipeline
    ///
    /// # FPS OPTIMIZATION: Cached Instance Generation
    ///
    /// Previous: Generated 80+ matrices every frame
    /// New: Cached, only regenerates when dirty flag is set
    ///
    /// Performance gain: Eliminates per-frame allocation and matrix math
    pub fn get_npc_orb_instances(&mut self) -> &[NpcOrbInstance] {
        // Only regenerate if dirty
        if self.instances_dirty {
            self.regenerate_instance_cache();
            self.instances_dirty = false;
        }

        &self.cached_instances
    }

    /// Internal: Regenerate the instance cache
    /// Called only when dirty flag is set
    fn regenerate_instance_cache(&mut self) {
        self.cached_instances.clear();
        self.cached_instances.reserve(self.npc_orbs.len());

        for orb in &self.npc_orbs {
            // Create model matrix (translation + scale)
            let scale = 0.8; // NPC orb size
            let model = Mat4::from_scale_rotation_translation(
                Vec3::splat(scale),
                glam::Quat::IDENTITY,
                orb.position,
            );
            let cols = model.to_cols_array_2d();

            self.cached_instances.push((
                cols[0],
                cols[1],
                cols[2],
                cols[3],
                orb.color,
                0.5, // Emissive intensity
            ));
        }
    }

    /// Check if any village overlaps with the given chunk bounds
    pub fn has_village_in_chunk(&self, chunk_min_x: f32, chunk_min_z: f32, chunk_size: f32) -> bool {
        let chunk_max_x = chunk_min_x + chunk_size;
        let chunk_max_z = chunk_min_z + chunk_size;

        self.villages.iter().any(|v| {
            v.bounds_max.x >= chunk_min_x && v.bounds_min.x <= chunk_max_x &&
            v.bounds_max.z >= chunk_min_z && v.bounds_min.z <= chunk_max_z
        })
    }

    /// Get village count
    pub fn village_count(&self) -> usize {
        self.villages.len()
    }

    /// Get total NPC count across all villages
    pub fn total_npc_count(&self) -> usize {
        self.npc_orbs.len()
    }

    /// Check if the player is inside any village bounds
    /// Returns (is_in_village, village_population)
    pub fn is_player_in_village(&self, player_pos: Vec3) -> (bool, u32) {
        for village in &self.villages {
            if player_pos.x >= village.bounds_min.x && player_pos.x <= village.bounds_max.x &&
               player_pos.z >= village.bounds_min.z && player_pos.z <= village.bounds_max.z {
                // Count NPCs in this village
                let npc_count = village.layout.npcs.len() as u32;
                return (true, npc_count);
            }
        }
        (false, 0)
    }

    /// Get the name of the village the player is in, if any
    pub fn get_current_village_name(&self, player_pos: Vec3) -> Option<String> {
        for village in &self.villages {
            if player_pos.x >= village.bounds_min.x && player_pos.x <= village.bounds_max.x &&
               player_pos.z >= village.bounds_min.z && player_pos.z <= village.bounds_max.z {
                return Some(village.layout.name.clone());
            }
        }
        None
    }

    /// Get statistics string for debug display
    pub fn stats_string(&self) -> String {
        let total_longhouses: usize = self.villages.iter()
            .map(|v| v.layout.longhouses.len())
            .sum();
        let total_fire_pits: usize = self.villages.iter()
            .map(|v| v.layout.fire_pits.len())
            .sum();

        format!("{} villages, {} longhouses, {} fire pits, {} NPCs",
            self.villages.len(), total_longhouses, total_fire_pits, self.npc_orbs.len())
    }
}

/// Convert VillageStructureType to a registry key name
pub fn structure_type_to_name(structure_type: VillageStructureType) -> &'static str {
    match structure_type {
        VillageStructureType::Longhouse => "village_longhouse",
        VillageStructureType::FirePit => "village_fire_pit",
        VillageStructureType::CornPlant => "village_corn",
        VillageStructureType::PrayerSite => "village_prayer_site",
    }
}
