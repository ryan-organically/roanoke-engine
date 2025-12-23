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
//!
//! # NPC Movement System
//!
//! NPCs follow schedule-based paths within their village, creating a
//! living, breathing settlement experience.

use glam::{Vec3, Mat4};
use croatoan_wfc::{
    find_village_sites, generate_world_village, get_village_structures_for_chunk,
    WorldVillage, VillageStructure, VillageStructureType, get_height_at,
};
use croatoan_procgen::{VillageRecipe, VillageId, generate_village, LonghouseStyle};

/// NPC orb instance data type for GPU upload
pub type NpcOrbInstance = ([f32; 4], [f32; 4], [f32; 4], [f32; 4], [f32; 3], f32);

/// NPC schedule activity
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NpcActivity {
    Sleeping,
    Working,
    Eating,
    Socializing,
    Patrolling,
    Praying,
    Gathering,
}

impl NpcActivity {
    /// Get movement speed multiplier for activity
    pub fn speed_mult(&self) -> f32 {
        match self {
            Self::Sleeping => 0.0,
            Self::Working => 0.3,
            Self::Eating => 0.0,
            Self::Socializing => 0.5,
            Self::Patrolling => 0.8,
            Self::Praying => 0.0,
            Self::Gathering => 0.6,
        }
    }
}

/// NPC orb data for rendering with movement
#[derive(Debug, Clone)]
pub struct NpcOrb {
    pub position: Vec3,
    pub color: [f32; 3],
    pub name: String,
    pub role: String,
    // Movement data
    pub home_position: Vec3,
    pub target_position: Vec3,
    pub velocity: Vec3,
    pub current_activity: NpcActivity,
    pub activity_time: f32,
    pub walk_timer: f32,
    // For communication beams
    pub awareness_target: Option<usize>, // Index of NPC they're aware of
    pub emissive: f32,
}

/// Communication beam between NPCs (visual dialogue)
#[derive(Debug, Clone)]
pub struct CommunicationBeam {
    pub from_idx: usize,
    pub to_idx: usize,
    pub color: [f32; 3],
    pub intensity: f32,
    pub lifetime: f32,
}

/// Player-focused NPC data (when player looks at an NPC)
#[derive(Debug, Clone)]
pub struct FocusedNpc {
    pub index: usize,
    pub name: String,
    pub role: String,
    pub position: Vec3,
    pub distance: f32,
    pub color: [f32; 3],
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
    /// Communication beams between NPCs
    pub communication_beams: Vec<CommunicationBeam>,
    /// Current game hour (0-24) for schedules
    current_hour: f32,
    /// Village center for reference
    village_center: Vec3,
    /// Currently focused NPC (player is looking at)
    pub focused_npc: Option<FocusedNpc>,
    /// Player focus beam (from player to focused NPC)
    pub player_focus_beam: Option<([f32; 3], [f32; 3], [f32; 3], f32)>,

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
            communication_beams: Vec::new(),
            current_hour: 8.0,
            village_center: Vec3::ZERO,
            focused_npc: None,
            player_focus_beam: None,
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
        // Place village close to spawn (40 units in positive X/Z direction)
        let village_x = spawn.x + 40.0;
        let village_z = spawn.z + 40.0;
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
        // Store village center for movement calculations
        self.village_center = village.center;

        for (idx, npc) in village.layout.npcs.iter().enumerate() {
            // Use NPC's stored position (Vec2 with x, y representing world x, z)
            let npc_x = npc.position.x;
            let npc_z = npc.position.y; // Vec2.y is world Z coordinate
            let (height, _) = get_height_at(npc_x, npc_z, self.seed);
            let pos = Vec3::new(npc_x, height + 1.5, npc_z);

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

            // Determine initial activity based on role
            let initial_activity = match npc.role {
                croatoan_procgen::NpcRole::Warrior => NpcActivity::Patrolling,
                croatoan_procgen::NpcRole::Hunter => NpcActivity::Gathering,
                croatoan_procgen::NpcRole::Farmer => NpcActivity::Working,
                croatoan_procgen::NpcRole::Chief | croatoan_procgen::NpcRole::Elder => NpcActivity::Socializing,
                croatoan_procgen::NpcRole::Shaman => NpcActivity::Praying,
                croatoan_procgen::NpcRole::Child => NpcActivity::Socializing,
                _ => NpcActivity::Working,
            };

            self.npc_orbs.push(NpcOrb {
                position: pos,
                color,
                name: npc.name.clone(),
                role: format!("{:?}", npc.role),
                home_position: pos,
                target_position: pos,
                velocity: Vec3::ZERO,
                current_activity: initial_activity,
                activity_time: (idx as f32 * 0.7) % 30.0, // Stagger activity timers
                walk_timer: idx as f32 * 0.3, // Stagger walking
                awareness_target: None,
                emissive: 0.3,
            });
        }

        // Mark instance cache as dirty since we added NPCs
        self.instances_dirty = true;
    }

    /// Get village data for spawning tame animals
    /// Returns a list of (center, bounds_radius, name) for each village
    pub fn get_village_spawn_data(&self) -> Vec<(Vec3, f32, String)> {
        self.villages
            .iter()
            .map(|v| (v.center, v.layout.bounds_radius, v.layout.name.clone()))
            .collect()
    }

    /// Get the seed for terrain height queries
    pub fn get_seed(&self) -> u32 {
        self.seed
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
                orb.emissive, // Dynamic emissive from NPC state
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

    /// Get all village center positions for tree clearing calculations
    /// Used by tree generation to create forest clearings around settlements
    pub fn get_village_centers(&self) -> Vec<Vec3> {
        self.villages.iter().map(|v| v.center).collect()
    }

    /// Get all corn field boundaries for rock exclusion
    /// Rocks won't spawn inside these areas (tilled/cultivated ground)
    pub fn get_corn_field_bounds(&self) -> Vec<croatoan_wfc::CornFieldBounds> {
        let mut bounds = Vec::new();
        for village in &self.villages {
            for field in &village.layout.corn_fields {
                bounds.push(croatoan_wfc::CornFieldBounds {
                    center: glam::Vec2::new(field.position.x, field.position.z),
                    half_size: glam::Vec2::new(field.size.x * 0.5, field.size.y * 0.5),
                });
            }
        }
        bounds
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

    // ========================================================================
    // NPC Movement and Communication System
    // ========================================================================

    /// Update NPC positions, activities, and communication beams
    /// Call this every frame with delta time and current game hour
    pub fn update(&mut self, dt: f32, game_hour: f32, player_pos: Vec3, player_look_dir: Vec3) {
        self.current_hour = game_hour;

        // Update communication beam lifetimes
        self.communication_beams.retain_mut(|beam| {
            beam.lifetime -= dt;
            beam.intensity = (beam.lifetime / 3.0).clamp(0.0, 1.0);
            beam.lifetime > 0.0
        });

        // Update player focus - which NPC is the player looking at
        self.update_player_focus(player_pos, player_look_dir);

        // Skip if no NPCs
        if self.npc_orbs.is_empty() {
            return;
        }

        let npc_count = self.npc_orbs.len();
        let base_speed = 2.5; // Units per second

        // First pass: Update activity and pick new targets
        for idx in 0..npc_count {
            // Get needed values first (avoiding borrow issues)
            let role = self.npc_orbs[idx].role.clone();
            let home_position = self.npc_orbs[idx].home_position;
            let position = self.npc_orbs[idx].position;
            let target_position = self.npc_orbs[idx].target_position;
            let current_activity = self.npc_orbs[idx].current_activity;
            let walk_timer = self.npc_orbs[idx].walk_timer;

            // Update activity timer
            self.npc_orbs[idx].activity_time += dt;
            self.npc_orbs[idx].walk_timer += dt;

            // Determine activity based on time of day
            let new_activity = self.get_activity_for_hour(game_hour, &role);
            if current_activity != new_activity {
                self.npc_orbs[idx].current_activity = new_activity;
                self.npc_orbs[idx].activity_time = 0.0;
            }

            // Pick new target periodically or when arrived
            let dist_to_target = position.distance(target_position);
            let should_pick_target = dist_to_target < 1.0 || walk_timer > 8.0;
            let activity_for_target = self.npc_orbs[idx].current_activity;

            if should_pick_target && activity_for_target.speed_mult() > 0.0 {
                self.npc_orbs[idx].walk_timer = 0.0;

                // Pick target based on activity
                let new_target = self.pick_target_for_activity(
                    idx,
                    activity_for_target,
                    home_position,
                );
                self.npc_orbs[idx].target_position = new_target;
            }

            // Calculate movement toward target
            let to_target = self.npc_orbs[idx].target_position - position;
            let dist = to_target.length();
            let speed_mult = self.npc_orbs[idx].current_activity.speed_mult();

            if dist > 0.5 && speed_mult > 0.0 {
                let direction = to_target / dist;
                let speed = base_speed * speed_mult;
                self.npc_orbs[idx].velocity = direction * speed;
            } else {
                self.npc_orbs[idx].velocity = Vec3::ZERO;
            }
        }

        // Second pass: Apply movement and check for nearby interactions
        let mut new_beams: Vec<CommunicationBeam> = Vec::new();

        for idx in 0..npc_count {
            // Apply velocity
            let vel = self.npc_orbs[idx].velocity;
            if vel.length_squared() > 0.01 {
                self.npc_orbs[idx].position += vel * dt;

                // Keep height correct
                let (h, _) = get_height_at(
                    self.npc_orbs[idx].position.x,
                    self.npc_orbs[idx].position.z,
                    self.seed,
                );
                self.npc_orbs[idx].position.y = h + 1.5;
            }

            // Check for nearby NPCs to create communication beams
            // Only check every few frames to save performance (use walk_timer as proxy)
            if self.npc_orbs[idx].walk_timer < 0.5 {
                for other_idx in (idx + 1)..npc_count {
                    let pos_a = self.npc_orbs[idx].position;
                    let pos_b = self.npc_orbs[other_idx].position;
                    let dist = pos_a.distance(pos_b);

                    // Close NPCs may interact
                    if dist < 8.0 && dist > 1.0 {
                        // Both must be in social activities
                        let act_a = self.npc_orbs[idx].current_activity;
                        let act_b = self.npc_orbs[other_idx].current_activity;

                        let can_interact = matches!(
                            (act_a, act_b),
                            (NpcActivity::Socializing, NpcActivity::Socializing)
                                | (NpcActivity::Working, NpcActivity::Working)
                                | (NpcActivity::Eating, NpcActivity::Eating)
                        );

                        if can_interact {
                            // Blend colors for beam
                            let color_a = self.npc_orbs[idx].color;
                            let color_b = self.npc_orbs[other_idx].color;
                            let beam_color = [
                                (color_a[0] + color_b[0]) * 0.5,
                                (color_a[1] + color_b[1]) * 0.5,
                                (color_a[2] + color_b[2]) * 0.5,
                            ];

                            // Check if beam already exists
                            let beam_exists = self.communication_beams.iter().any(|b| {
                                (b.from_idx == idx && b.to_idx == other_idx)
                                    || (b.from_idx == other_idx && b.to_idx == idx)
                            });

                            if !beam_exists {
                                new_beams.push(CommunicationBeam {
                                    from_idx: idx,
                                    to_idx: other_idx,
                                    color: beam_color,
                                    intensity: 1.0,
                                    lifetime: 3.0,
                                });

                                // Update awareness targets
                                self.npc_orbs[idx].awareness_target = Some(other_idx);
                                self.npc_orbs[other_idx].awareness_target = Some(idx);

                                // Increase emissive when communicating
                                self.npc_orbs[idx].emissive = 0.6;
                                self.npc_orbs[other_idx].emissive = 0.6;
                            }
                        }
                    }
                }
            }

            // Decay emissive
            self.npc_orbs[idx].emissive = (self.npc_orbs[idx].emissive - dt * 0.1).max(0.3);
        }

        // Add new beams
        self.communication_beams.extend(new_beams);

        // Player proximity awareness - NPCs notice player
        for idx in 0..npc_count {
            let dist_to_player = self.npc_orbs[idx].position.distance(player_pos);
            if dist_to_player < 15.0 {
                // NPC notices player - glow slightly
                self.npc_orbs[idx].emissive = 0.5;
            }
        }

        // Mark instances dirty so rendering updates
        self.instances_dirty = true;
    }

    /// Get appropriate activity for the hour based on role
    fn get_activity_for_hour(&self, hour: f32, role: &str) -> NpcActivity {
        // Night time (22:00 - 6:00)
        if hour >= 22.0 || hour < 6.0 {
            return NpcActivity::Sleeping;
        }

        // Early morning (6:00 - 8:00)
        if hour < 8.0 {
            return NpcActivity::Eating;
        }

        // Work hours (8:00 - 17:00)
        if hour < 17.0 {
            return match role {
                "Warrior" => NpcActivity::Patrolling,
                "Hunter" => NpcActivity::Gathering,
                "Shaman" => NpcActivity::Praying,
                "Child" => NpcActivity::Socializing,
                _ => NpcActivity::Working,
            };
        }

        // Evening (17:00 - 20:00)
        if hour < 20.0 {
            return NpcActivity::Eating;
        }

        // Late evening (20:00 - 22:00)
        NpcActivity::Socializing
    }

    /// Pick a target position based on current activity
    fn pick_target_for_activity(&self, _npc_idx: usize, activity: NpcActivity, home: Vec3) -> Vec3 {
        // Simple hash for pseudo-random offset
        let time_hash = (self.current_hour * 1000.0) as u64;
        let hash = time_hash.wrapping_mul(0x517cc1b727220a95);
        let rand_x = ((hash >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let rand_z = ((hash & 0xFFFFFFFF) as f32 / u32::MAX as f32) * 2.0 - 1.0;

        match activity {
            NpcActivity::Sleeping => home, // Stay at home
            NpcActivity::Working => {
                // Move around near home
                home + Vec3::new(rand_x * 10.0, 0.0, rand_z * 10.0)
            }
            NpcActivity::Eating => {
                // Move toward village center (fire pit area)
                let toward_center = (self.village_center - home).normalize_or_zero();
                home + toward_center * 15.0 + Vec3::new(rand_x * 5.0, 0.0, rand_z * 5.0)
            }
            NpcActivity::Socializing => {
                // Move toward village center
                let toward_center = (self.village_center - home).normalize_or_zero();
                home + toward_center * 20.0 + Vec3::new(rand_x * 8.0, 0.0, rand_z * 8.0)
            }
            NpcActivity::Patrolling => {
                // Circle around village perimeter
                let angle = self.current_hour * 0.3;
                let radius = 40.0;
                self.village_center + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
            }
            NpcActivity::Praying => {
                // Stay near prayer site (offset from center)
                self.village_center + Vec3::new(30.0 + rand_x * 5.0, 0.0, 30.0 + rand_z * 5.0)
            }
            NpcActivity::Gathering => {
                // Wander outward from village
                let angle = self.current_hour * 0.5 + rand_x;
                let radius = 50.0 + rand_z * 20.0;
                self.village_center + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
            }
        }
    }

    /// Get communication beam render data
    pub fn get_beam_render_data(&self) -> Vec<([f32; 3], [f32; 3], [f32; 3], f32)> {
        let mut beams: Vec<_> = self.communication_beams
            .iter()
            .filter_map(|beam| {
                if beam.from_idx < self.npc_orbs.len() && beam.to_idx < self.npc_orbs.len() {
                    let from = self.npc_orbs[beam.from_idx].position;
                    let to = self.npc_orbs[beam.to_idx].position;
                    Some((
                        [from.x, from.y, from.z],
                        [to.x, to.y, to.z],
                        beam.color,
                        beam.intensity,
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Add player focus beam if present
        if let Some(beam) = &self.player_focus_beam {
            beams.push(*beam);
        }

        beams
    }

    /// Update which NPC the player is looking at
    fn update_player_focus(&mut self, player_pos: Vec3, look_dir: Vec3) {
        self.focused_npc = None;
        self.player_focus_beam = None;

        if self.npc_orbs.is_empty() || look_dir.length_squared() < 0.01 {
            return;
        }

        let look_dir = look_dir.normalize();
        let mut best_score = 0.0f32;
        let mut best_idx: Option<usize> = None;

        // Find the NPC closest to the look direction
        for (idx, orb) in self.npc_orbs.iter().enumerate() {
            let to_npc = orb.position - player_pos;
            let dist = to_npc.length();

            // Must be within reasonable distance (50 units)
            if dist < 2.0 || dist > 50.0 {
                continue;
            }

            let to_npc_norm = to_npc / dist;

            // Calculate how aligned the look direction is with the NPC
            let dot = look_dir.dot(to_npc_norm);

            // Must be looking roughly toward the NPC (within ~30 degree cone)
            if dot < 0.85 {
                continue;
            }

            // Score based on alignment and distance (prefer closer and more aligned)
            let score = dot * (1.0 - dist / 50.0);

            if score > best_score {
                best_score = score;
                best_idx = Some(idx);
            }
        }

        // If we found an NPC being looked at
        if let Some(idx) = best_idx {
            let orb = &self.npc_orbs[idx];

            self.focused_npc = Some(FocusedNpc {
                index: idx,
                name: orb.name.clone(),
                role: orb.role.clone(),
                position: orb.position,
                distance: (orb.position - player_pos).length(),
                color: orb.color,
            });

            // Create a beam from player to focused NPC
            // Use a cyan/white color for the focus beam
            let beam_color = [0.5, 0.8, 1.0]; // Light blue/cyan
            self.player_focus_beam = Some((
                [player_pos.x, player_pos.y + 1.5, player_pos.z], // From player eye level
                [orb.position.x, orb.position.y, orb.position.z], // To NPC
                beam_color,
                0.7, // Intensity
            ));

            // Make the focused NPC glow brighter
            self.npc_orbs[idx].emissive = 0.8;
            self.instances_dirty = true;
        }
    }

    /// Get the name and role of the focused NPC for UI display
    pub fn get_focused_npc_info(&self) -> Option<(String, String, f32)> {
        self.focused_npc.as_ref().map(|f| {
            (f.name.clone(), f.role.clone(), f.distance)
        })
    }
}

/// Convert VillageStructureType to a registry key name
pub fn structure_type_to_name(structure_type: VillageStructureType) -> &'static str {
    match structure_type {
        VillageStructureType::Longhouse => "village_longhouse",
        VillageStructureType::FirePit => "village_fire_pit",
        VillageStructureType::CornPlant => "village_corn",
        VillageStructureType::PrayerSite => "village_prayer_site",
        VillageStructureType::TilledGround => "village_tilled_ground",
        VillageStructureType::FencePost => "village_fence_post",
    }
}
