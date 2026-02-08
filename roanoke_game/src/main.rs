// Allow dead code for planned features not yet integrated
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use croatoan_core::{App, CursorGrabMode, DeviceEvent, ElementState, KeyCode, MouseButton, PhysicalKey, WinitEvent as Event, WinitWindowEvent as WindowEvent};
use croatoan_wfc::{generate_terrain_chunk, generate_vegetation_for_chunk, generate_detritus_for_chunk, generate_rocks_for_chunk_with_exclusions, generate_deadwood_for_chunk, generate_buildings_for_chunk, generate_foliage_for_chunk, generate_ferns_for_chunk};
use croatoan_wfc::{WormConfig, WormTunnel, generate_perlin_worm, generate_bio_orbs, generate_worm_cave_items, generate_cave_mesh_for_chunk, CaveMeshConfig, CaveGenConfig, sample_worm_sdf, get_worm_bounds, get_biome_t};
use croatoan_render::{Camera, TerrainPipeline, TerrainTextures, ShadowMap, ShadowPipeline, InstancedShadowPipeline, GrassPipeline, TreePipeline, TreeMesh, TreeLODConfig, LODFadeMode, DetritusPipeline, BuildingPipeline, BuildingMesh, BuildingVertex, Frustum, ChunkBounds, SunPipeline, MoonPipeline, SkyPipeline, ViewModelPipeline, WeaponViewModelPipeline, WeaponVertex, LightShaftPipeline, AnimalOrbPipeline, OrbInstance, AnimalModelPipeline, AnimalVertex, AnimalInstance, FoliagePipeline, FoliageVertex, RainPipeline, EmberPipeline, BioOrbPipeline, BioOrbInstance};
use croatoan_procgen::{generate_simple_tree_mesh, generate_deciduous_tree, generate_conifer_tree, ProceduralTreeConfig, generate_enhanced_tree, generate_default_lod1_tree, generate_lod1_tree_mesh, RockRecipe, generate_rock, BuildingRecipe, generate_building};
use glam::{Vec2, Vec3, Mat4, Quat};
use wgpu;
use image; // Added image crate
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};
use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod player;
mod biped_ik;
mod chunk_manager;
mod asset_loader;
use player::Player;
use biped_ik::{BipedIK, PlayerMoveState};
use chunk_manager::{ChunkManager, ChunkCoord, ChunkRequest, LoadedChunk};

// Extend LoadedChunk to include buildings (we can't modify the struct definition in chunk_manager.rs from here easily without replacing the file, 
// but wait, LoadedChunk is defined in chunk_manager.rs. I need to modify chunk_manager.rs FIRST or define a wrapper.
// Actually, I should modify chunk_manager.rs to add buildings field.
// But for now, I will modify main.rs to import the struct and I will modify chunk_manager.rs in a separate step.
// Wait, I can't modify main.rs to use a field that doesn't exist yet.
// I will assume I will modify chunk_manager.rs in the next step.


mod water_system;
mod pond_water_system;
mod atmosphere;
mod audio_system;
mod procedural_synth;
mod animals;
mod gltf_loader;
mod village_manager;
mod progression;
mod npc;
mod game_state;
mod economy;
mod audio_events;
mod data_pipeline;
mod safe_ops;

// New systems for discovery, ecology, flora, naval, and weather
mod encyclopedia;
mod flora;
mod inventory_icons;
mod ecology;
mod naval;
mod weather;
mod systems_manager;
mod character_agent;
mod world_features;
mod ui;
mod network;
mod campfire;

use water_system::WaterSystem;
use pond_water_system::PondWaterSystem;
use world_features::WorldFeatures;
mod weather_system;
use weather_system::{WeatherSystem, WeatherType};
use atmosphere::AtmosphereEngine;
use audio_system::{AudioSystem, MusicState};
use audio_events::{AudioEventProcessor, AudioEvent, AudioBiome, ThreatLevel, species_threat_profile, wfc_biome_to_audio};
use data_pipeline::{DataPipeline, GameEvent, NpcAudioIntegration, ProgressionAudioBridge, FactionAudioBridge};
use animals::{AnimalManager, AnimalSpawner, Difficulty, TimeOfDay as AnimalTimeOfDay, BehaviorState};
use village_manager::VillageManager;
use game_state::GameProgression;
use safe_ops::{SafeMutex, saturating_range_area}; // Safe mutex and overflow protection

// ... (Existing structs remain same) ...



// --- Game State & Save System ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameState {
    Menu,
    Loading,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PauseMenuPage {
    Main,
    Settings,
    Controls,
    LoadGame,
    CharacterSheet,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterSheetTab {
    Inventory,
    SkillTree,
    Commendations,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveData {
    seed: u32,
    player_pos: [f32; 3],
    player_rot: [f32; 2], // Yaw, Pitch
    inventory: Vec<String>,
    // NPC System persistence
    #[serde(default)]
    npc_relationships: Option<npc::relationships::RelationshipManager>,
}

struct LoadingProgress {
    total_chunks: usize,
    chunks_generated: usize,
    chunks_uploaded: usize,
    current_status: String,
}

/// Slideshow state for loading screen with Ken Burns panning effect
struct LoadingSlideshow {
    textures: Vec<egui::TextureHandle>,
    image_sizes: Vec<[usize; 2]>,  // Original image dimensions
    current_index: usize,
    next_index: usize,
    pan_time: f32,           // Time into current pan (0.0 - PAN_DURATION)
    transition_time: f32,    // Time into crossfade (0.0 - TRANSITION_DURATION, negative = not transitioning)
    pan_directions: Vec<(f32, f32)>,  // Random pan direction per image (dx, dy normalized)
    start_time: Instant,
}

impl LoadingSlideshow {
    const PAN_DURATION: f32 = 12.0;      // Seconds per image
    const TRANSITION_DURATION: f32 = 3.0; // Crossfade duration (longer for smoother blend)
    const ZOOM_SCALE: f32 = 1.2;        // How much to zoom in (for pan headroom)
    const PAN_AMOUNT: f32 = 0.08;       // How far to pan (as fraction of image) - gentle drift

    fn new() -> Self {
        Self {
            textures: Vec::new(),
            image_sizes: Vec::new(),
            current_index: 0,
            next_index: 1,
            pan_time: 0.0,
            transition_time: -1.0,
            pan_directions: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn load_images(&mut self, ctx: &egui::Context) {
        let loading_dir = "assets/ui/loading";
        if let Ok(entries) = std::fs::read_dir(loading_dir) {
            let mut image_paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png")
                })
                .map(|e| e.path())
                .collect();

            // Sort for consistent ordering
            image_paths.sort();

            for (i, path) in image_paths.iter().enumerate() {
                if let Ok(bytes) = std::fs::read(path) {
                    if let Ok(image) = image::load_from_memory(&bytes) {
                        let size = [image.width() as usize, image.height() as usize];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            pixels.as_slice(),
                        );
                        let texture = ctx.load_texture(
                            format!("loading_slide_{}", i),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.textures.push(texture);
                        self.image_sizes.push(size);

                        // Random pan direction for this image
                        let angle = (i as f32 * 2.37) % std::f32::consts::TAU; // Pseudo-random angle
                        self.pan_directions.push((angle.cos(), angle.sin()));

                        println!("[UI] Loaded slideshow image: {:?}", path.file_name().unwrap_or_default());
                    }
                }
            }

            if !self.textures.is_empty() {
                self.next_index = if self.textures.len() > 1 { 1 } else { 0 };
                println!("[UI] Loaded {} slideshow images", self.textures.len());
            }
        }
    }

    fn update(&mut self, dt: f32) {
        if self.textures.is_empty() {
            return;
        }

        self.pan_time += dt;

        // Start transition near end of pan
        if self.pan_time > Self::PAN_DURATION - Self::TRANSITION_DURATION && self.transition_time < 0.0 {
            self.transition_time = 0.0;
        }

        // Update transition
        if self.transition_time >= 0.0 {
            self.transition_time += dt;

            // Transition complete - switch to next image
            if self.transition_time >= Self::TRANSITION_DURATION {
                self.current_index = self.next_index;
                self.next_index = (self.next_index + 1) % self.textures.len();
                self.pan_time = 0.0;
                self.transition_time = -1.0;
            }
        }
    }

    fn calculate_uv(&self, index: usize, pan_progress: f32, screen_aspect: f32) -> egui::Rect {
        if index >= self.image_sizes.len() {
            return egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        }

        let [img_w, img_h] = self.image_sizes[index];
        let img_aspect = img_w as f32 / img_h as f32;

        // Calculate base UV to fill screen (cover mode)
        let (mut u_size, mut v_size) = if screen_aspect > img_aspect {
            // Screen is wider - fit width, crop height
            (1.0, img_aspect / screen_aspect)
        } else {
            // Screen is taller - fit height, crop width
            (screen_aspect / img_aspect, 1.0)
        };

        // Apply zoom for pan headroom
        u_size /= Self::ZOOM_SCALE;
        v_size /= Self::ZOOM_SCALE;

        // Get pan direction for this image
        let (dx, dy) = self.pan_directions.get(index).copied().unwrap_or((1.0, 0.0));

        // Calculate pan offset (ease in-out)
        let t = pan_progress.clamp(0.0, 1.0);
        let eased = t * t * (3.0 - 2.0 * t); // Smoothstep
        let pan_offset = (eased - 0.5) * Self::PAN_AMOUNT;

        // Center the UV with pan offset
        let u_center = 0.5 + dx * pan_offset;
        let v_center = 0.5 + dy * pan_offset;

        let u_min = (u_center - u_size / 2.0).clamp(0.0, 1.0 - u_size);
        let v_min = (v_center - v_size / 2.0).clamp(0.0, 1.0 - v_size);

        egui::Rect::from_min_max(
            egui::pos2(u_min, v_min),
            egui::pos2(u_min + u_size, v_min + v_size),
        )
    }

    fn render(&self, ui: &mut egui::Ui) {
        if self.textures.is_empty() {
            return;
        }

        let screen_rect = ui.ctx().screen_rect();
        let screen_aspect = screen_rect.width() / screen_rect.height();
        let pan_progress = self.pan_time / Self::PAN_DURATION;

        // Draw current image
        let current_uv = self.calculate_uv(self.current_index, pan_progress, screen_aspect);
        let current_alpha = if self.transition_time >= 0.0 {
            let t = (self.transition_time / Self::TRANSITION_DURATION).clamp(0.0, 1.0);
            ((1.0 - t) * 255.0) as u8
        } else {
            255
        };

        ui.painter().image(
            self.textures[self.current_index].id(),
            screen_rect,
            current_uv,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, current_alpha),
        );

        // Draw next image (fading in) during transition
        if self.transition_time >= 0.0 && self.textures.len() > 1 {
            let next_pan_progress = self.transition_time / Self::PAN_DURATION; // Start fresh pan
            let next_uv = self.calculate_uv(self.next_index, next_pan_progress, screen_aspect);
            let next_alpha = ((self.transition_time / Self::TRANSITION_DURATION).clamp(0.0, 1.0) * 255.0) as u8;

            ui.painter().image(
                self.textures[self.next_index].id(),
                screen_rect,
                next_uv,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, next_alpha),
            );
        }
    }
}

// Shot/recoil animation state for viewmodel
struct SwingAnimation {
    is_swinging: bool,
    swing_progress: f32,  // 0.0 to 1.0
    swing_duration: f32,  // Total animation duration in seconds
    hit_processed: bool,  // Whether hit was processed this swing
    muzzle_flash: f32,    // Muzzle flash intensity (0.0 to 1.0, decays quickly)
}

struct SharedState {
    camera: Camera,
    game_state: GameState,
    seed: u32,
    seed_input: String,
    inventory: Vec<String>,
    egui_state: Option<egui_winit::State>,
    egui_ctx: egui::Context,
    // FPS & Save System
    fps: f32,
    last_frame_time: Instant,
    save_name_input: String,
    // Player
    player: Player,
    keys: std::collections::HashMap<KeyCode, ElementState>,
    // Time
    time_of_day: f32, // 0.0 - 24.0
    // Loading Progress
    loading_progress: LoadingProgress,
    // Asset Registry
    mesh_registry: std::collections::HashMap<String, TreeMesh>, // For Trees/Rocks
    building_registry: std::collections::HashMap<String, Arc<BuildingMesh>>, // For Buildings
    terrain_textures: Option<Arc<TerrainTextures>>, // Shared terrain textures for all chunks
    background_texture: Option<egui::TextureHandle>, // For Home Screen
    loading_slideshow: LoadingSlideshow, // For Loading Screen
    weather: WeatherSystem,
    // Audio System
    audio_system: AudioSystem,
    audio_event_processor: AudioEventProcessor,
    // Pause Menu
    pause_menu_page: PauseMenuPage,
    show_save_popup: bool,
    // Game Settings
    mouse_sensitivity: f32,
    movement_speed: f32,
    render_distance: f32,
    dither_distance_ratio: f32,  // 0.5-1.0, controls when LOD dither begins (relative to render_distance)
    dither_fade_width: f32,      // Width of dither transition zone in units
    master_volume: f32,
    // Swing Animation
    swing_animation: SwingAnimation,
    atmosphere: AtmosphereEngine,
    show_load_menu: bool, // For Load Game submenu
    show_debug_ui: bool,  // F12 toggle for performance stats
    // Animal System
    animal_manager: AnimalManager,
    animal_spawner: AnimalSpawner,
    // Village System
    village_manager: VillageManager,
    // NPC System (integrated with unified agent manager)
    npc_manager: npc::NpcManager,
    // Progression System
    game_progression: GameProgression,
    // Economy System
    economy_manager: economy::EconomyManager,
    player_economy: economy::PlayerEconomy,
    dropped_items: economy::DroppedItemManager,
    // Storage Containers (chests, crates in the world)
    storage_manager: economy::StorageManager,
    // Inventory Icon Cache (3D model icons for items)
    icon_cache: inventory_icons::InventoryIconCache,
    // Hotbar (quick-slot for inventory access)
    active_hotbar_slot: usize, // 0-9, maps to first 10 inventory slots
    // Currently loaded weapon viewmodel (template_id)
    loaded_weapon_viewmodel: Option<String>,
    // Combat state
    combat_kill_time: f32, // Tracks time spent fighting current target
    // Debug
    debug_timer: f32,
    fog_level: u8, // 0=Off, 1=Light, 2=Medium, 3=Heavy, 4=Dense
    // Audio state tracking
    was_in_village: bool,
    coastal_sounds_loaded: bool,
    // Data Pipeline
    data_pipeline: DataPipeline,
    npc_audio: NpcAudioIntegration,
    progression_audio: ProgressionAudioBridge,
    faction_audio: FactionAudioBridge,
    // Systems Manager (encyclopedia, flora, ecology, weather coordination)
    systems_manager: systems_manager::SystemsManager,
    // Forageable Plants (world flora instances for harvesting)
    forageable_plants: flora::growth::FloraManager,
    // Unified Agent Manager (NPC/Animal coordination, orb visuals, communication)
    unified_agents: character_agent::unified_manager::UnifiedAgentManager,
    // Naval System (ships, sailing, water travel)
    ship_manager: naval::ships::ShipManager,
    // World Features (rivers, caves)
    world_features: WorldFeatures,
    // Worm tunnels for cave collision (generated per-chunk, stored for player traversal)
    worm_tunnels: Vec<WormTunnel>,
    // Dialogue UI state
    current_dialogue: Option<npc::interaction::DialogueUIData>,
    // Character Sheet (Tab menu)
    character_sheet_tab: CharacterSheetTab,
    character_preview_rotation: f32,  // Y-axis rotation for 3D model preview
    character_preview_dragging: bool, // Is user dragging to rotate?
    // Perks Journal
    perks_journal: ui::PerksJournalState,
    journal_textures: ui::JournalTextures,
    // Campfire System
    campfire_manager: campfire::CampfireManager,
    // Currently open chest (for UI interaction)
    open_chest_id: Option<economy::ContainerId>,
    // Multiplayer Networking
    network: network::NetworkManager,
    // Network UI state
    network_host_port: String,
    network_join_address: String,
    network_player_name: String,
}

impl SharedState {
    /// Helper method to process audio event with proper split borrow
    fn process_audio_event(&mut self, event: AudioEvent) {
        self.audio_event_processor.process_event(event, &mut self.audio_system);
    }

    /// Helper method to update audio event processor with proper split borrow
    fn update_audio_processor(&mut self, delta: f32) {
        self.audio_event_processor.update(delta, &mut self.audio_system);
    }

    /// Helper method to update systems manager with proper split borrow
    fn update_systems_manager(&mut self, delta: f32) {
        let player_pos = self.player.position;
        let look_dir = self.camera.forward();
        self.systems_manager.update(delta, player_pos, look_dir, &self.animal_manager);
    }

    /// Helper method to process combat loot with proper split borrow
    fn process_combat_loot(
        &mut self,
        result: &crate::animals::combat::AnimalAttackResult,
        weapon_used: &str,
    ) -> economy::CombatLootResult {
        use economy::CombatEconomyExt;
        let combat_kill_time = self.combat_kill_time;
        let hunting_level = self.game_progression.player_progression.get_hunting_level();
        result.process_loot(
            &mut self.economy_manager,
            &mut self.player_economy,
            weapon_used,
            combat_kill_time,
            100.0, // Player health
            hunting_level,
            0.0, // Luck stat
        )
    }

    /// Helper method to update systems manager with proper split borrow
    fn update_systems(&mut self, delta: f32) {
        let player_pos = self.player.position;
        let look_dir = self.camera.forward();
        // Split borrow: systems_manager and animal_manager are separate fields
        // Within a method on SharedState, Rust can see they're distinct
        self.systems_manager.update(delta, player_pos, look_dir, &self.animal_manager);
    }
}

/// Spawn tame animals (horses, donkeys) in villages
/// Each village gets 4 tame horses and 1 tame donkey that stay within village bounds
fn spawn_village_animals(
    animal_manager: &mut animals::AnimalManager,
    village_data: &[(Vec3, f32, String)],
    seed: u32,
) {
    use croatoan_wfc::get_height_at;

    for (center, bounds_radius, name) in village_data {
        // Spawn 4 tame horses spread around the village
        for i in 0..4 {
            let angle = (i as f32 / 4.0) * std::f32::consts::TAU + 0.3;
            let radius = bounds_radius * 0.4 + (i as f32 * 5.0);
            let x = center.x + angle.cos() * radius;
            let z = center.z + angle.sin() * radius;
            let (height, _) = get_height_at(x, z, seed);
            let pos = Vec3::new(x, height, z);

            // Spawn tame horse with village as home
            let id = animal_manager.spawn(animals::AnimalSpecies::Horse, pos, (0, 0), None);
            if let Some(horse) = animal_manager.get_mut(id) {
                horse.home_position = *center;
                horse.territory_radius = bounds_radius * 0.8;
                horse.taming_progress = 1.0; // Fully tamed
            }
        }

        // Spawn 1 tame donkey
        let donkey_angle: f32 = 2.5;
        let donkey_radius = bounds_radius * 0.3;
        let x = center.x + donkey_angle.cos() * donkey_radius;
        let z = center.z + donkey_angle.sin() * donkey_radius;
        let (height, _) = get_height_at(x, z, seed);
        let pos = Vec3::new(x, height, z);

        let id = animal_manager.spawn(animals::AnimalSpecies::Donkey, pos, (0, 0), None);
        if let Some(donkey) = animal_manager.get_mut(id) {
            donkey.home_position = *center;
            donkey.territory_radius = bounds_radius * 0.7;
            donkey.taming_progress = 1.0; // Fully tamed
        }

        println!("[VILLAGE] Spawned 4 horses and 1 donkey in '{}'", name);
    }
}

/// Spawn wild horse herds on beaches throughout the world
/// Horses spawn in groups of 2-7, flee from player, and roam beach areas
fn spawn_beach_horses(
    animal_manager: &mut animals::AnimalManager,
    seed: u32,
) {
    use croatoan_wfc::{get_height_at, get_biome_t};
    use rand::SeedableRng;
    use rand::Rng;

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);

    // Scan for beach areas and spawn horse herds
    // We sample points in a large grid and look for beach biome (t between 0.45-0.55)
    let world_size = 4000.0; // Scan radius from origin
    let sample_step = 200.0; // Check every 200 units
    let mut herds_spawned = 0;
    let max_herds = 20; // Limit total beach horse herds

    let mut x = -world_size;
    while x < world_size && herds_spawned < max_herds {
        let mut z = -world_size;
        while z < world_size && herds_spawned < max_herds {
            // Check if this is a beach biome
            let biome_t = get_biome_t(x, z, seed);

            // Beach is biome_t between 0.45 and 0.55
            if biome_t >= 0.45 && biome_t <= 0.55 {
                // Use deterministic randomness based on position
                let pos_hash = ((x as i32).wrapping_mul(73856093)) ^ ((z as i32).wrapping_mul(19349663));
                let spawn_chance = ((pos_hash as u32) % 100) as f32 / 100.0;

                // 15% chance to spawn a herd at each beach sample point
                if spawn_chance < 0.15 {
                    // Determine herd size (2-7 horses)
                    let herd_size = 2 + (rng.gen::<u32>() % 6) as usize;

                    // Create a pack for this herd
                    let pack_id = animal_manager.create_pack(animals::AnimalSpecies::Horse);

                    // Spawn horses in a loose group
                    for i in 0..herd_size {
                        let angle = (i as f32 / herd_size as f32) * std::f32::consts::TAU + rng.gen::<f32>() * 0.5;
                        let radius = 5.0 + rng.gen::<f32>() * 10.0; // 5-15m spread
                        let hx = x + angle.cos() * radius;
                        let hz = z + angle.sin() * radius;
                        let (height, _) = get_height_at(hx, hz, seed);

                        // Only spawn if above water
                        if height > 0.5 {
                            let pos = glam::Vec3::new(hx, height, hz);
                            let id = animal_manager.spawn(
                                animals::AnimalSpecies::Horse,
                                pos,
                                (0, 0),
                                Some(pack_id),
                            );

                            // Set home position and territory for the herd
                            if let Some(horse) = animal_manager.get_mut(id) {
                                horse.home_position = glam::Vec3::new(x, height, z);
                                horse.territory_radius = 150.0; // Roam within 150m of spawn
                            }
                        }
                    }

                    herds_spawned += 1;
                }
            }

            z += sample_step;
        }
        x += sample_step;
    }

    if herds_spawned > 0 {
        println!("[ANIMALS] Spawned {} wild horse herds on beaches", herds_spawned);
    }
}

/// Find a beach spawn position at ground level
/// Searches for a low beach area close to the water line
fn find_beach_spawn_position(seed: u32) -> Vec3 {
    use croatoan_wfc::{get_height_at, get_biome_t};

    // Beach zone biome_t is roughly 0.45-0.65
    // Search a grid and find the best low beach spot above water
    let search_x_start = 200.0;
    let search_x_end = 450.0;
    let search_z_range = 200.0;

    let mut best_pos = None;
    let mut best_score = f32::MIN;

    // Search for a good beach spot
    let step = 10.0;
    let mut x = search_x_start;
    while x < search_x_end {
        let mut z = -search_z_range;
        while z < search_z_range {
            let biome_t = get_biome_t(x, z, seed);
            let (height, _) = get_height_at(x, z, seed);

            // Must be above water (height > 1.0) but on low beach
            // Beach biome_t is 0.45-0.65, prefer closer to water (lower biome_t)
            if height > 1.0 && height < 8.0 && biome_t >= 0.45 && biome_t <= 0.60 {
                // Score: prefer low height and low biome_t (closer to water)
                let score = -height - (biome_t - 0.45) * 10.0;
                if score > best_score {
                    best_score = score;
                    best_pos = Some(Vec3::new(x, height, z));
                }
            }
            z += step;
        }
        x += step;
    }

    // Fallback: search for ANY spot above water if no beach found
    let spawn_pos = match best_pos {
        Some(pos) => {
            println!("[SPAWN] Found beach at ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);
            pos
        }
        None => {
            // Search for any above-water spot
            let mut fallback_pos = Vec3::new(250.0, 5.0, 0.0);
            for fx in (100..400).step_by(20) {
                for fz in (-100..100).step_by(20) {
                    let (h, _) = get_height_at(fx as f32, fz as f32, seed);
                    if h > 2.0 && h < 20.0 {
                        fallback_pos = Vec3::new(fx as f32, h, fz as f32);
                        break;
                    }
                }
            }
            println!("[SPAWN] No beach found, fallback at ({:.1}, {:.1}, {:.1})",
                fallback_pos.x, fallback_pos.y, fallback_pos.z);
            fallback_pos
        }
    };

    // Add player eye height (1.8m) to spawn at ground level
    Vec3::new(spawn_pos.x, spawn_pos.y + 1.8, spawn_pos.z)
}

/// Spawn ring-necked pheasants near player spawn location
/// Creates a flock of 30-40 pheasants in the area around the beach spawn for immediate wildlife encounters
fn spawn_pheasants_at_spawn(
    animal_manager: &mut animals::AnimalManager,
    seed: u32,
) {
    use croatoan_wfc::get_height_at;
    use rand::SeedableRng;
    use rand::Rng;

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64 + 7777); // Different seed offset for variety

    // Spawn 30-40 pheasants in a 200m radius around beach spawn point
    let beach_spawn = find_beach_spawn_position(seed);
    let spawn_center = glam::Vec3::new(beach_spawn.x, 0.0, beach_spawn.z);
    let spawn_radius = 200.0;
    let num_pheasants = 30 + rng.gen_range(0..11); // 30-40 pheasants

    let mut spawned_count = 0;
    for _ in 0..num_pheasants {
        // Random position within spawn radius
        let angle: f32 = rng.gen::<f32>() * std::f32::consts::TAU;
        let distance: f32 = rng.gen::<f32>().sqrt() * spawn_radius; // sqrt for uniform distribution

        let x = spawn_center.x + angle.cos() * distance;
        let z = spawn_center.z + angle.sin() * distance;
        let height = get_height_at(x, z, seed).0;

        // Only spawn on reasonable terrain (not underwater, not too steep)
        if height > 1.0 {
            let pos = glam::Vec3::new(x, height, z);

            // Create small groups (flocks of 3-6)
            let pack_id = if rng.gen_bool(0.3) {
                Some(animal_manager.create_pack(animals::AnimalSpecies::RingNeckedPheasant))
            } else {
                None
            };

            let id = animal_manager.spawn(
                animals::AnimalSpecies::RingNeckedPheasant,
                pos,
                (0, 0),
                pack_id,
            );

            // Set home position and small territory
            if let Some(pheasant) = animal_manager.get_mut(id) {
                pheasant.home_position = pos;
                pheasant.territory_radius = 50.0 + rng.gen::<f32>() * 50.0; // 50-100m territory
            }

            spawned_count += 1;
        }
    }

    println!("[ANIMALS] Spawned {} ring-necked pheasants near spawn point", spawned_count);
}

/// Register village factions with player progression
/// Uses collect_faction_data to avoid borrow conflicts
fn register_village_factions(state: &mut SharedState) {
    // Collect data first (immutable borrow of village_manager)
    let (village_factions, npc_factions) = state.village_manager.collect_faction_data();

    // Then register (mutable borrow of player_progression)
    for (id, faction) in village_factions {
        state.game_progression.player_progression.register_village_faction(id, faction);
    }
    for (id, npc_data) in npc_factions {
        state.game_progression.player_progression.register_npc_faction(id, npc_data);
    }
}

fn save_game(name: &str, data: &SaveData) {
    let _ = fs::create_dir_all("saves");
    let path = format!("saves/{}.json", name);
    if let Ok(json) = serde_json::to_string_pretty(data) {
        if let Ok(mut file) = File::create(&path) {
            let _ = file.write_all(json.as_bytes());
            println!("[SAVE] Game saved to {}", path);
        }
    }
}

fn load_game(name: &str) -> Option<SaveData> {
    let path = format!("saves/{}.json", name);
    if let Ok(mut file) = File::open(&path) {
        let mut json = String::new();
        if file.read_to_string(&mut json).is_ok() {
            if let Ok(data) = serde_json::from_str::<SaveData>(&json) {
                println!("[LOAD] Game loaded: Seed {}", data.seed);
                return Some(data);
            }
        }
    }
    println!("[LOAD] Save file '{}' not found or invalid.", name);
    None
}

fn list_saves() -> Vec<String> {
    let mut saves = Vec::new();
    if let Ok(entries) = fs::read_dir("saves") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(name) = entry.path().file_stem() {
                        if let Some(name_str) = name.to_str() {
                            saves.push(name_str.to_string());
                        }
                    }
                }
            }
        }
    }
    saves
}

// --- Character Sheet UI (Book Layout) ---

fn render_character_sheet(ui_ctx: &egui::Context, state: &mut SharedState) {
    // Book-style colors
    let paper_color = egui::Color32::from_rgb(244, 228, 188);      // Aged paper
    let leather_color = egui::Color32::from_rgb(101, 67, 33);      // Leather binding
    let ink_color = egui::Color32::from_rgb(40, 30, 20);           // Dark ink
    let accent_color = egui::Color32::from_rgb(139, 90, 43);       // Warm brown
    let tab_active_color = egui::Color32::from_rgb(210, 180, 140); // Active tab
    let tab_inactive_color = egui::Color32::from_rgb(180, 150, 110); // Inactive tab

    let screen_rect = ui_ctx.screen_rect();
    let screen_width = screen_rect.width();
    let screen_height = screen_rect.height();

    // Book dimensions (centered, takes 80% of screen)
    let book_width = (screen_width * 0.85).min(1200.0);
    let book_height = (screen_height * 0.85).min(800.0);
    let book_left = (screen_width - book_width) / 2.0;
    let book_top = (screen_height - book_height) / 2.0;

    // Draw dark overlay behind book
    egui::Area::new(egui::Id::new("character_sheet_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ui_ctx, |ui| {
            let overlay_rect = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(screen_width, screen_height),
            );
            ui.painter().rect_filled(
                overlay_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
        });

    // Main book area
    egui::Area::new(egui::Id::new("character_sheet_book"))
        .fixed_pos(egui::pos2(book_left, book_top))
        .order(egui::Order::Foreground)
        .show(ui_ctx, |ui| {
            // Book frame (leather binding effect)
            let book_rect = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(book_width, book_height),
            );

            // Outer leather border
            ui.painter().rect_filled(
                book_rect,
                egui::Rounding::same(12.0),
                leather_color,
            );

            // Inner paper area (with margin for binding)
            let paper_margin = 15.0;
            let paper_rect = book_rect.shrink(paper_margin);
            ui.painter().rect_filled(
                paper_rect,
                egui::Rounding::same(6.0),
                paper_color,
            );

            // Center spine line
            let spine_x = book_width / 2.0;
            ui.painter().line_segment(
                [egui::pos2(spine_x, paper_margin + 10.0), egui::pos2(spine_x, book_height - paper_margin - 10.0)],
                egui::Stroke::new(3.0, leather_color),
            );

            // === TAB BAR (Top of book) ===
            let tab_area_top = paper_margin + 10.0;
            let tab_height = 35.0;
            let tab_width = 140.0;
            let tab_spacing = 10.0;
            let tabs_total_width = 3.0 * tab_width + 2.0 * tab_spacing;
            let tabs_start_x = (book_width - tabs_total_width) / 2.0;

            let tabs = [
                (CharacterSheetTab::Inventory, "Inventory"),
                (CharacterSheetTab::SkillTree, "Skill Tree"),
                (CharacterSheetTab::Commendations, "Commendations"),
            ];

            for (i, (tab_type, tab_name)) in tabs.iter().enumerate() {
                let tab_x = tabs_start_x + (i as f32) * (tab_width + tab_spacing);
                let tab_rect = egui::Rect::from_min_size(
                    egui::pos2(tab_x, tab_area_top),
                    egui::vec2(tab_width, tab_height),
                );

                let is_active = state.character_sheet_tab == *tab_type;
                let tab_color = if is_active { tab_active_color } else { tab_inactive_color };

                // Tab button
                let tab_response = ui.allocate_rect(tab_rect, egui::Sense::click());
                ui.painter().rect_filled(
                    tab_rect,
                    egui::Rounding::same(6.0),
                    tab_color,
                );
                ui.painter().rect_stroke(
                    tab_rect,
                    egui::Rounding::same(6.0),
                    egui::Stroke::new(2.0, leather_color),
                );
                ui.painter().text(
                    tab_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *tab_name,
                    egui::FontId::proportional(16.0),
                    ink_color,
                );

                if tab_response.clicked() {
                    state.character_sheet_tab = *tab_type;
                }

                // Hover effect
                if tab_response.hovered() && !is_active {
                    ui.painter().rect_stroke(
                        tab_rect,
                        egui::Rounding::same(6.0),
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                }
            }

            // === CONTENT AREA ===
            let content_top = tab_area_top + tab_height + 20.0;
            let content_height = book_height - content_top - paper_margin - 20.0;
            let left_page_width = (book_width / 2.0) - paper_margin - 15.0;
            let right_page_width = left_page_width;
            let left_page_x = paper_margin + 10.0;
            let right_page_x = book_width / 2.0 + 15.0;

            // === LEFT PAGE: Character Preview ===
            let left_page_rect = egui::Rect::from_min_size(
                egui::pos2(left_page_x, content_top),
                egui::vec2(left_page_width, content_height),
            );

            // Character preview frame
            ui.painter().rect_stroke(
                left_page_rect.shrink(5.0),
                egui::Rounding::same(4.0),
                egui::Stroke::new(2.0, accent_color),
            );

            // "CHARACTER" header
            ui.painter().text(
                egui::pos2(left_page_rect.center().x, content_top + 25.0),
                egui::Align2::CENTER_CENTER,
                "CHARACTER",
                egui::FontId::proportional(22.0),
                ink_color,
            );

            // Character silhouette placeholder (will be replaced with 3D model)
            let preview_rect = egui::Rect::from_min_size(
                egui::pos2(left_page_x + 30.0, content_top + 50.0),
                egui::vec2(left_page_width - 60.0, content_height - 180.0),
            );
            ui.painter().rect_filled(
                preview_rect,
                egui::Rounding::same(8.0),
                egui::Color32::from_rgb(60, 50, 40),
            );

            // Rotation indicator
            let rotation_degrees = (state.character_preview_rotation * 180.0 / std::f32::consts::PI) % 360.0;
            ui.painter().text(
                egui::pos2(preview_rect.center().x, preview_rect.center().y),
                egui::Align2::CENTER_CENTER,
                format!("3D Model\nRotation: {:.0}°\n\nDrag to rotate", rotation_degrees),
                egui::FontId::proportional(14.0),
                egui::Color32::GRAY,
            );

            // Handle rotation drag
            let preview_response = ui.allocate_rect(preview_rect, egui::Sense::drag());
            if preview_response.dragged() {
                state.character_preview_rotation += preview_response.drag_delta().x * 0.01;
            }

            // Character stats summary at bottom of left page
            let stats_y = preview_rect.max.y + 15.0;
            let stats = [
                ("Level", "1"),
                ("Health", "100/100"),
                ("Stamina", "100/100"),
            ];
            for (i, (label, value)) in stats.iter().enumerate() {
                let y = stats_y + (i as f32) * 22.0;
                ui.painter().text(
                    egui::pos2(left_page_x + 25.0, y),
                    egui::Align2::LEFT_CENTER,
                    format!("{}: {}", label, value),
                    egui::FontId::proportional(14.0),
                    ink_color,
                );
            }

            // === RIGHT PAGE: Tab Content ===
            let right_page_rect = egui::Rect::from_min_size(
                egui::pos2(right_page_x, content_top),
                egui::vec2(right_page_width, content_height),
            );

            match state.character_sheet_tab {
                CharacterSheetTab::Inventory => {
                    render_inventory_tab(ui, right_page_rect, state, ink_color, accent_color);
                }
                CharacterSheetTab::SkillTree => {
                    render_skill_tree_tab(ui, right_page_rect, state, ink_color, accent_color);
                }
                CharacterSheetTab::Commendations => {
                    render_commendations_tab(ui, right_page_rect, state, ink_color, accent_color);
                }
            }

            // === CLOSE BUTTON (Top right corner) ===
            let close_btn_rect = egui::Rect::from_min_size(
                egui::pos2(book_width - 50.0, 5.0),
                egui::vec2(40.0, 40.0),
            );
            let close_response = ui.allocate_rect(close_btn_rect, egui::Sense::click());
            ui.painter().text(
                close_btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                "X",
                egui::FontId::proportional(24.0),
                if close_response.hovered() { egui::Color32::WHITE } else { paper_color },
            );
            if close_response.clicked() {
                state.game_state = GameState::Playing;
            }
        });
}

// --- Journal Settings UI (renders inside journal book when Settings tab active) ---

fn render_journal_settings(ui_ctx: &egui::Context, state: &mut SharedState) {
    let screen_rect = ui_ctx.screen_rect();
    let screen_width = screen_rect.width();
    let screen_height = screen_rect.height();

    // Match journal layout dimensions exactly
    let journal_width = (screen_width * 0.75).min(1000.0);
    let journal_height = (screen_height * 0.80).min(700.0);
    // Use screen_rect min position for proper centering
    let journal_left = screen_rect.min.x + (screen_width - journal_width) / 2.0;
    let journal_top = screen_rect.min.y + (screen_height - journal_height) / 2.0;

    // Left navigation tab width
    let tab_width = 140.0;
    let content_width = journal_width - tab_width;
    let margin = 20.0;

    // Content positioning (matches journal render)
    let content_top = journal_top + 90.0; // Below header and tabs
    let content_height = journal_height - 110.0;
    let half_width = (content_width - margin * 3.0) / 2.0;

    // Three columns for settings: Game, Audio, Developer
    let col_width = (content_width - margin * 4.0) / 3.0;
    let col1_x = journal_left + tab_width + margin;
    let col2_x = col1_x + col_width + margin;
    let col3_x = col2_x + col_width + margin;

    let ink_color = egui::Color32::from_rgb(40, 30, 20);
    let slider_width = col_width - 20.0;

    // Column 1: Game Settings
    egui::Area::new(egui::Id::new("settings_game"))
        .fixed_pos(egui::pos2(col1_x, content_top))
        .order(egui::Order::Foreground)
        .show(ui_ctx, |ui| {
            ui.set_max_width(col_width);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("GAME").size(14.0).color(ink_color).strong());
                ui.add_space(12.0);

                ui.label(egui::RichText::new("Mouse Sensitivity").size(11.0).color(ink_color));
                ui.add(egui::Slider::new(&mut state.mouse_sensitivity, 1.0..=100.0)
                    .custom_formatter(|n, _| format!("{:.0}", n)));
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Movement Speed").size(11.0).color(ink_color));
                ui.add(egui::Slider::new(&mut state.movement_speed, 1.0..=1000.0)
                    .logarithmic(true)
                    .custom_formatter(|n, _| format!("{:.0}x", n / 10.0)));
                state.player.speed = state.movement_speed;
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Render Distance").size(11.0).color(ink_color));
                ui.add(egui::Slider::new(&mut state.render_distance, 150.0..=600.0)
                    .custom_formatter(|n, _| format!("{:.0}m", n)));
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Dither Distance").size(11.0).color(ink_color));
                let effective_dist = state.render_distance * state.dither_distance_ratio;
                ui.add(egui::Slider::new(&mut state.dither_distance_ratio, 0.5..=1.0)
                    .custom_formatter(move |n, _| format!("{:.0}%", n * 100.0)));
                ui.label(egui::RichText::new(format!("Effective: {:.0}m", effective_dist))
                    .size(9.0).color(egui::Color32::DARK_GRAY));
            });
        });

    // Column 2: Audio Settings
    egui::Area::new(egui::Id::new("settings_audio"))
        .fixed_pos(egui::pos2(col2_x, content_top))
        .order(egui::Order::Foreground)
        .show(ui_ctx, |ui| {
            ui.set_max_width(col_width);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("AUDIO").size(14.0).color(ink_color).strong());
                ui.add_space(12.0);

                ui.label(egui::RichText::new("Master Volume").size(11.0).color(ink_color));
                ui.add(egui::Slider::new(&mut state.master_volume, 0.0..=100.0)
                    .custom_formatter(|n, _| format!("{:.0}", n)));
                state.audio_system.set_master_volume(state.master_volume / 100.0);
                ui.add_space(8.0);

                let mut music_vol = state.audio_system.music_volume * 100.0;
                ui.label(egui::RichText::new("Music Volume").size(11.0).color(ink_color));
                if ui.add(egui::Slider::new(&mut music_vol, 0.0..=100.0)
                    .custom_formatter(|n, _| format!("{:.0}", n))).changed() {
                    state.audio_system.set_music_volume(music_vol / 100.0);
                }
                ui.add_space(8.0);

                let mut amb_vol = state.audio_system.ambience_volume * 100.0;
                ui.label(egui::RichText::new("Ambience Volume").size(11.0).color(ink_color));
                if ui.add(egui::Slider::new(&mut amb_vol, 0.0..=100.0)
                    .custom_formatter(|n, _| format!("{:.0}", n))).changed() {
                    state.audio_system.set_ambience_volume(amb_vol / 100.0);
                }
            });
        });

    // Column 3: Developer Settings
    egui::Area::new(egui::Id::new("settings_developer"))
        .fixed_pos(egui::pos2(col3_x, content_top))
        .order(egui::Order::Foreground)
        .show(ui_ctx, |ui| {
            ui.set_max_width(col_width);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("DEVELOPER").size(14.0).color(ink_color).strong());
                ui.add_space(12.0);

                ui.label(egui::RichText::new("Time of Day").size(11.0).color(ink_color));
                ui.add(egui::Slider::new(&mut state.time_of_day, 0.0..=24.0)
                    .custom_formatter(|n, _| {
                        let h = n as i32;
                        let m = ((n - h as f64) * 60.0) as i32;
                        format!("{:02}:{:02}", h % 24, m)
                    }));
                ui.add_space(12.0);

                ui.label(egui::RichText::new("Weather").size(11.0).color(ink_color));
                ui.horizontal_wrapped(|ui| {
                    if ui.small_button("Clear").clicked() {
                        state.weather.set_weather(WeatherType::Clear, false);
                    }
                    if ui.small_button("Cloudy").clicked() {
                        state.weather.set_weather(WeatherType::PartlyCloudy, false);
                    }
                    if ui.small_button("Overcast").clicked() {
                        state.weather.set_weather(WeatherType::Overcast, false);
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.small_button("Stormy").clicked() {
                        state.weather.set_weather(WeatherType::Stormy, false);
                    }
                    if ui.small_button("Foggy").clicked() {
                        state.weather.set_weather(WeatherType::Foggy, false);
                    }
                });
                ui.add_space(8.0);
                ui.checkbox(&mut state.weather.auto_weather_enabled, "Auto Weather");
            });
        });
}

fn render_inventory_tab(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    state: &SharedState,
    ink_color: egui::Color32,
    accent_color: egui::Color32,
) {
    // Header
    ui.painter().text(
        egui::pos2(rect.center().x, rect.min.y + 25.0),
        egui::Align2::CENTER_CENTER,
        "INVENTORY",
        egui::FontId::proportional(22.0),
        ink_color,
    );

    // Currency display
    let currency_y = rect.min.y + 55.0;
    let wampum = state.player_economy.wallet.wampum;
    let tobacco = state.player_economy.wallet.tobacco;
    ui.painter().text(
        egui::pos2(rect.min.x + 20.0, currency_y),
        egui::Align2::LEFT_CENTER,
        format!("Wampum: {}    Tobacco: {}", wampum, tobacco),
        egui::FontId::proportional(14.0),
        ink_color,
    );

    // Inventory grid
    let grid_top = currency_y + 30.0;
    let grid_left = rect.min.x + 15.0;
    let slot_size = 50.0;
    let slot_spacing = 5.0;
    let cols = 6;
    let rows = 6;

    for row in 0..rows {
        for col in 0..cols {
            let slot_index = row * cols + col;
            let slot_x = grid_left + (col as f32) * (slot_size + slot_spacing);
            let slot_y = grid_top + (row as f32) * (slot_size + slot_spacing);
            let slot_rect = egui::Rect::from_min_size(
                egui::pos2(slot_x, slot_y),
                egui::vec2(slot_size, slot_size),
            );

            // Slot background
            let slot_color = if slot_index < 10 {
                // Hotbar slots highlighted
                egui::Color32::from_rgb(180, 160, 130)
            } else {
                egui::Color32::from_rgb(200, 180, 150)
            };
            ui.painter().rect_filled(slot_rect, egui::Rounding::same(4.0), slot_color);
            ui.painter().rect_stroke(
                slot_rect,
                egui::Rounding::same(4.0),
                egui::Stroke::new(1.0, accent_color),
            );

            // Item in slot (if any)
            if let Some(item) = state.player_economy.inventory.get_slot(slot_index) {
                // Rarity color border
                let rarity_color = match item.rarity {
                    economy::Rarity::Crude => egui::Color32::GRAY,
                    economy::Rarity::Common => egui::Color32::WHITE,
                    economy::Rarity::Uncommon => egui::Color32::from_rgb(30, 255, 30),
                    economy::Rarity::Rare => egui::Color32::from_rgb(30, 144, 255),
                    economy::Rarity::Epic => egui::Color32::from_rgb(138, 43, 226),
                    economy::Rarity::Legendary => egui::Color32::from_rgb(255, 165, 0),
                    economy::Rarity::Mythic => egui::Color32::from_rgb(255, 20, 147),
                    economy::Rarity::Primordial => egui::Color32::from_rgb(255, 215, 0),
                };
                ui.painter().rect_stroke(
                    slot_rect.shrink(2.0),
                    egui::Rounding::same(3.0),
                    egui::Stroke::new(2.0, rarity_color),
                );

                // Try to show 3D model icon if available
                if let Some(tex) = state.icon_cache.textures.get(&item.template_id) {
                    let icon_size = slot_size - 8.0;
                    let icon_rect = egui::Rect::from_center_size(
                        slot_rect.center(),
                        egui::vec2(icon_size, icon_size),
                    );
                    ui.painter().image(
                        tex.id(),
                        icon_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        rarity_color,
                    );
                } else {
                    // Fallback to item name (truncated)
                    let name: String = item.name.chars().take(6).collect();
                    ui.painter().text(
                        slot_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        name,
                        egui::FontId::proportional(10.0),
                        ink_color,
                    );
                }

                // Stack count
                if item.stack_size > 1 {
                    ui.painter().text(
                        egui::pos2(slot_rect.max.x - 5.0, slot_rect.max.y - 5.0),
                        egui::Align2::RIGHT_BOTTOM,
                        format!("{}", item.stack_size),
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                }
            }

            // Hotbar number indicator
            if slot_index < 10 {
                ui.painter().text(
                    egui::pos2(slot_x + 5.0, slot_y + 5.0),
                    egui::Align2::LEFT_TOP,
                    format!("{}", (slot_index + 1) % 10),
                    egui::FontId::proportional(9.0),
                    egui::Color32::from_rgb(100, 80, 60),
                );
            }
        }
    }

    // Equipment slots label
    let equip_y = grid_top + (rows as f32) * (slot_size + slot_spacing) + 15.0;
    ui.painter().text(
        egui::pos2(rect.min.x + 20.0, equip_y),
        egui::Align2::LEFT_CENTER,
        "Equipment: Coming Soon",
        egui::FontId::proportional(12.0),
        egui::Color32::DARK_GRAY,
    );
}

fn render_skill_tree_tab(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    state: &SharedState,
    ink_color: egui::Color32,
    accent_color: egui::Color32,
) {
    // Header
    ui.painter().text(
        egui::pos2(rect.center().x, rect.min.y + 25.0),
        egui::Align2::CENTER_CENTER,
        "SKILL TREES",
        egui::FontId::proportional(22.0),
        ink_color,
    );

    // Skill tree sections
    let sections = [
        ("Hunting", vec![
            ("Basic Tracker", true),
            ("Boar Hunter", false),
            ("Deer Stalker", false),
            ("Wolf Tracker", false),
            ("Wilderness Scout", false),
        ]),
        ("Archaeology", vec![
            ("Novice Digger", true),
            ("Field Scholar", false),
            ("Fossil Expert", false),
        ]),
        ("Mining", vec![
            ("Surface Collector", false),
            ("Prospector", false),
            ("Iron Seeker", false),
        ]),
        ("Pond Hockey", vec![
            ("Ice Legs", false),
            ("Steady Stride", false),
        ]),
    ];

    let mut y = rect.min.y + 55.0;
    for (section_name, skills) in sections.iter() {
        // Section header
        ui.painter().text(
            egui::pos2(rect.min.x + 20.0, y),
            egui::Align2::LEFT_CENTER,
            *section_name,
            egui::FontId::proportional(16.0),
            accent_color,
        );
        y += 22.0;

        // Skills
        for (skill_name, unlocked) in skills.iter() {
            let icon = if *unlocked { "[+]" } else { "[ ]" };
            let color = if *unlocked { ink_color } else { egui::Color32::DARK_GRAY };
            ui.painter().text(
                egui::pos2(rect.min.x + 35.0, y),
                egui::Align2::LEFT_CENTER,
                format!("{} {}", icon, skill_name),
                egui::FontId::proportional(13.0),
                color,
            );
            y += 18.0;
        }
        y += 10.0;
    }

    // Points display
    ui.painter().text(
        egui::pos2(rect.min.x + 20.0, rect.max.y - 30.0),
        egui::Align2::LEFT_CENTER,
        "Skill Points: 25",
        egui::FontId::proportional(14.0),
        ink_color,
    );
}

fn render_commendations_tab(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    _state: &SharedState,
    ink_color: egui::Color32,
    accent_color: egui::Color32,
) {
    // Header
    ui.painter().text(
        egui::pos2(rect.center().x, rect.min.y + 25.0),
        egui::Align2::CENTER_CENTER,
        "COMMENDATIONS",
        egui::FontId::proportional(22.0),
        ink_color,
    );

    // Achievement categories
    let categories = [
        ("Exploration", vec![
            ("First Steps", "Travel 100 meters", true),
            ("Wanderer", "Discover 10 locations", false),
            ("Cartographer", "Map every region", false),
        ]),
        ("Combat", vec![
            ("First Blood", "Defeat an enemy", false),
            ("Hunter's Mark", "Kill 10 animals", false),
            ("Apex Predator", "Defeat a legendary beast", false),
        ]),
        ("Survival", vec![
            ("Well Fed", "Eat 10 different foods", false),
            ("Night Owl", "Survive 3 nights", true),
            ("Weather the Storm", "Survive a hurricane", false),
        ]),
        ("Social", vec![
            ("First Contact", "Talk to an NPC", true),
            ("Diplomat", "Befriend 5 NPCs", false),
            ("Blood Bond", "Achieve max faction standing", false),
        ]),
    ];

    let mut y = rect.min.y + 55.0;
    for (category_name, achievements) in categories.iter() {
        // Category header
        ui.painter().text(
            egui::pos2(rect.min.x + 20.0, y),
            egui::Align2::LEFT_CENTER,
            *category_name,
            egui::FontId::proportional(16.0),
            accent_color,
        );
        y += 22.0;

        // Achievements
        for (name, desc, completed) in achievements.iter() {
            let icon = if *completed { "[*]" } else { "[ ]" };
            let color = if *completed { ink_color } else { egui::Color32::DARK_GRAY };
            ui.painter().text(
                egui::pos2(rect.min.x + 35.0, y),
                egui::Align2::LEFT_CENTER,
                format!("{} {}", icon, name),
                egui::FontId::proportional(13.0),
                color,
            );
            y += 16.0;
            ui.painter().text(
                egui::pos2(rect.min.x + 55.0, y),
                egui::Align2::LEFT_CENTER,
                *desc,
                egui::FontId::proportional(11.0),
                egui::Color32::from_rgb(120, 100, 80),
            );
            y += 18.0;
        }
        y += 8.0;
    }
}

// --- Offscreen Render Target for Post-Process Effects ---

struct OffscreenTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl OffscreenTarget {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Render Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view, width, height }
    }

    fn needs_resize(&self, width: u32, height: u32) -> bool {
        self.width != width || self.height != height
    }
}

// --- Main Entry Point ---

fn main() {
    println!("=== ROANOKE ENGINE: HOME SCREEN & SAVE SYSTEM ===\n");

    // Initialize App
    let mut app = App::new("Roanoke Engine", 1280, 720);


    
    // Re-thinking strategy: SharedState needs to hold `Option<TreeMesh>` or similar created in render loop.
    // But we want a registry.
    // Let's make SharedState hold `Option<HashMap<String, TreeMesh>>` which is populated in the first render pass.
    
    // Shared State
    let shared_state = Arc::new(Mutex::new(SharedState {
        camera: Camera::new(
            Vec3::new(32.0, 50.0, -30.0),
            Vec3::new(32.0, 0.0, 32.0),
            1280.0 / 720.0,
        ),
        game_state: GameState::Menu,
        seed: 12345,
        seed_input: "12345".to_string(),
        inventory: Vec::new(),
        egui_state: None,
        egui_ctx: egui::Context::default(),
        fps: 0.0,
        last_frame_time: Instant::now(),
        save_name_input: String::new(),
        player: Player::new(Vec3::new(0.0, 50.0, 0.0)), // Start high up
        keys: std::collections::HashMap::new(),
        time_of_day: 12.0, // Start at noon
        loading_progress: LoadingProgress {
            total_chunks: 0,
            chunks_generated: 0,
            chunks_uploaded: 0,
            current_status: String::new(),
        },
        mesh_registry: std::collections::HashMap::new(),
        building_registry: std::collections::HashMap::new(),
        terrain_textures: None, // Loaded on first GPU access
        background_texture: None,
        loading_slideshow: LoadingSlideshow::new(),
        weather: WeatherSystem::new(),
        // Audio System
        audio_system: AudioSystem::new(),
        audio_event_processor: AudioEventProcessor::new(),
        // Pause Menu
        pause_menu_page: PauseMenuPage::Main,
        show_save_popup: false,
        // Game Settings (default values)
        mouse_sensitivity: 50.0, // 0-100 scale, 50 = default
        movement_speed: 10.0,
        render_distance: 400.0, // Extended for long-distance fidelity testing
        dither_distance_ratio: 0.85, // 85% of render distance before dithering begins
        dither_fade_width: 100.0,    // 100 unit transition zone for smooth dithering
        master_volume: 80.0, // 0-100 scale, 80 = default
        swing_animation: SwingAnimation {
            is_swinging: false,
            swing_progress: 0.0,
            swing_duration: 0.4, // 400ms for recoil animation
            hit_processed: false,
            muzzle_flash: 0.0,
        },
        atmosphere: AtmosphereEngine::new(),
        show_load_menu: false,
        show_debug_ui: false,  // Hidden by default, F12 to toggle
        // Animal System
        animal_manager: AnimalManager::new(Difficulty::Normal),
        animal_spawner: AnimalSpawner::new(12345), // Will be re-seeded when game starts
        // Village System
        village_manager: VillageManager::new(12345), // Will be re-seeded when game starts
        // NPC System
        npc_manager: npc::NpcManager::new(),
        // Progression System
        game_progression: GameProgression::new(),
        // Economy System
        economy_manager: economy::EconomyManager::new(),
        player_economy: economy::PlayerEconomy::new(),
        dropped_items: economy::DroppedItemManager::new(),
        // Storage Containers
        storage_manager: economy::StorageManager::new(),
        // Inventory Icon Cache
        icon_cache: inventory_icons::InventoryIconCache::new(),
        // Hotbar
        active_hotbar_slot: 0,
        loaded_weapon_viewmodel: None,
        // Combat state
        combat_kill_time: 0.0,
        // Debug
        debug_timer: 0.0,
        fog_level: 0, // Start with fog off
        // Audio state tracking
        was_in_village: false,
        coastal_sounds_loaded: false,
        // Data Pipeline
        data_pipeline: DataPipeline::new(),
        npc_audio: NpcAudioIntegration::new(),
        progression_audio: ProgressionAudioBridge::new(),
        faction_audio: FactionAudioBridge::new(),
        // Systems Manager (will be re-seeded when game starts)
        systems_manager: systems_manager::SystemsManager::new(12345),
        // Forageable Plants manager
        forageable_plants: flora::growth::FloraManager::new(),
        // Unified Agent Manager for NPC/Animal coordination
        unified_agents: character_agent::unified_manager::UnifiedAgentManager::new(),
        // Naval System for ships and water travel
        ship_manager: naval::ships::ShipManager::new(),
        // World Features (rivers and caves - will be re-seeded when game starts)
        world_features: WorldFeatures::new(12345),
        // Worm tunnels (populated during chunk generation)
        worm_tunnels: Vec::new(),
        // Dialogue state
        current_dialogue: None,
        // Character Sheet (Tab menu)
        character_sheet_tab: CharacterSheetTab::Inventory,
        character_preview_rotation: 0.0,
        character_preview_dragging: false,
        // Perks Journal
        perks_journal: ui::PerksJournalState::default(),
        journal_textures: ui::JournalTextures::new(),
        // Campfire System
        campfire_manager: campfire::CampfireManager::new(),
        // Currently open chest
        open_chest_id: None,
        // Multiplayer Networking - initialize based on CLI args
        network: {
            let net_mode = network::NetworkLaunchMode::from_args();
            println!("[NET] Launch mode: {}", net_mode.description());
            match net_mode {
                network::NetworkLaunchMode::Offline => network::NetworkManager::offline(12345),
                network::NetworkLaunchMode::Host { port } => {
                    let seed: u32 = rand::random();
                    match network::NetworkManager::host(port, seed) {
                        Ok(nm) => nm,
                        Err(e) => {
                            eprintln!("[NET] Failed to start host: {}", e);
                            network::NetworkManager::offline(12345)
                        }
                    }
                }
                network::NetworkLaunchMode::Join { address, player_name } => {
                    match network::NetworkManager::join(&address, &player_name) {
                        Ok(nm) => nm,
                        Err(e) => {
                            eprintln!("[NET] Failed to join: {}", e);
                            network::NetworkManager::offline(12345)
                        }
                    }
                }
            }
        },
        // Network UI state
        network_host_port: "7878".to_string(),
        network_join_address: "127.0.0.1:7878".to_string(),
        network_player_name: format!("Player_{}", rand::random::<u16>()),
    }));

    // ... (Channel setup) ...
    // Response Data: (Terrain, Trees, Shrubs, Detritus, Rocks, Buildings, Ferns, Coord X, Coord Z)
    // NOTE: Grass data removed - using grass2/grass3 model LOD system instead
    type ChunkData = (
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>, // Terrain
        std::collections::HashMap<String, Vec<Mat4>>, // Trees (Named + Grouped)
        std::collections::HashMap<String, Vec<Mat4>>, // Shrubs (Named + Grouped)
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>, // Detritus
        Vec<(String, Mat4)>, // Rocks (Named Instances)
        Vec<(String, Mat4)>, // Buildings (Named Instances)
        Vec<(String, Mat4)>, // Ferns (Named Instances)
        i32, i32 // Offsets (World Space)
    );
    
    // Channel for requesting chunks
    let (request_tx, request_rx): (Sender<ChunkRequest>, Receiver<ChunkRequest>) = channel();
    // Channel for receiving generated chunks
    let (chunk_tx, chunk_rx): (Sender<ChunkData>, Receiver<ChunkData>) = channel();

    let chunk_rx = Arc::new(Mutex::new(chunk_rx));

    // Wrap request receiver in Arc<Mutex<>> for multi-threaded access
    let request_rx = Arc::new(Mutex::new(request_rx));

    // Spawn Multiple Chunk Generation Worker Threads
    // Using 6 workers for better parallel chunk generation at high speeds
    let num_workers = 6;
    println!("[GEN] Spawning {} chunk generation workers...", num_workers);

    for worker_id in 0..num_workers {
        let request_rx = Arc::clone(&request_rx);
        let chunk_tx = chunk_tx.clone();

        thread::spawn(move || {
            loop {
                // Try to get a request from the shared queue
                let req = {
                    let rx = request_rx.lock().unwrap();
                    rx.recv()
                };

                let req = match req {
                    Ok(r) => r,
                    Err(_) => break,
                };
                let chunk_world_size = 256.0;
                let chunk_resolution = 64;
                let scale = 4.0;
                let (offset_x, offset_z) = req.coord.world_offset(chunk_world_size);
                let offset_x = offset_x as i32;
                let offset_z = offset_z as i32;

                // Generate terrain
                let (terrain_pos, terrain_col, terrain_nrm, terrain_idx) =
                    generate_terrain_chunk(req.seed, chunk_resolution, offset_x, offset_z, scale);

                // Procedural grass DISABLED - replaced by grass2/grass3 model LOD system
                // See docs/archive/PROCEDURAL_GRASS_SYSTEM.md for details
                // let (grass_pos, grass_col, grass_heights, grass_idx) = generate_vegetation_for_chunk(...);

                // Generate trees (with birch/pine communities via foliage system)
                let foliage = generate_foliage_for_chunk(
                    req.seed,
                    chunk_world_size,
                    offset_x as f32,
                    offset_z as f32,
                    5, // tree model count: birch_0, pine_0, dead_conifer_0, fir_0, fir_1
                    1, // shrub model count: conifer_shrub_0 (beach_grass_0 disabled - using grass3 LOD)
                );
                let tree_groups = foliage.trees_by_model(5);
                let shrub_groups = foliage.shrubs_by_model(1);

                // Generate detritus
                let (det_pos, det_nrm, det_uv, det_idx) = generate_detritus_for_chunk(
                    req.seed,
                    chunk_world_size,
                    offset_x as f32,
                    offset_z as f32,
                );

                // Generate rocks (excluding corn field areas)
                let mut rock_instances = generate_rocks_for_chunk_with_exclusions(
                    req.seed,
                    chunk_world_size,
                    offset_x as f32,
                    offset_z as f32,
                    &req.corn_field_exclusions,
                );

                // Generate deadwood (fallen logs) and merge with rock instances
                let deadwood_instances = generate_deadwood_for_chunk(
                    req.seed,
                    chunk_world_size,
                    offset_x as f32,
                    offset_z as f32,
                );
                rock_instances.extend(deadwood_instances);

                // Generate buildings
                let building_instances = generate_buildings_for_chunk(
                    req.seed,
                    chunk_world_size,
                    offset_x as f32,
                    offset_z as f32,
                );

                // Generate ferns (forest understory)
                let fern_result = generate_ferns_for_chunk(
                    req.seed,
                    chunk_world_size,
                    offset_x as f32,
                    offset_z as f32,
                    1, // Currently 1 fern model variant (fern_01)
                );
                // Convert to named instances
                let fern_instances: Vec<(String, Mat4)> = fern_result.by_model(1)
                    .into_iter()
                    .flat_map(|(name, transforms)| {
                        transforms.into_iter().map(move |t| (name.clone(), t))
                    })
                    .collect();

                // Send result (grass data removed - using grass2/grass3 models instead)
                if chunk_tx.send((
                    terrain_pos, terrain_col, terrain_nrm, terrain_idx,
                    tree_groups,
                    shrub_groups,
                    det_pos, det_nrm, det_uv, det_idx,
                    rock_instances,
                    building_instances,
                    fern_instances,
                    offset_x, offset_z
                )).is_err() {
                    break;
                }
            }
        });
    }

    // Drop original chunk_tx so channel closes when all workers finish
    drop(chunk_tx);

    // Terrain Data (Protected by Mutex to allow regeneration)
    let _terrain_data = Arc::new(Mutex::new(None::<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>)>));
    
    // Time tracking
    let start_time = Instant::now();

    // --- Input Callback ---
    let input_state = Arc::clone(&shared_state);
    app.set_input_callback(move |event, window| {
        let mut state = input_state.safe_lock();

        // Initialize egui state if needed
        if state.egui_state.is_none() {
            let viewport_id = state.egui_ctx.viewport_id();
            state.egui_state = Some(egui_winit::State::new(
                state.egui_ctx.clone(),
                viewport_id,
                window,
                Some(window.scale_factor() as f32),
                None,
            ));
        }

        // Handle CloseRequested before egui can consume it
        if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
            println!("[EXIT] Window close requested");
            std::process::exit(0);
        }

        // Handle Tab key BEFORE egui (so it doesn't consume it for focus navigation)
        // Tab now opens the unified journal menu as an overlay (game continues rendering behind)
        if let Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } = event {
            if let PhysicalKey::Code(KeyCode::Tab) = key_event.physical_key {
                if key_event.state == ElementState::Pressed {
                    // Toggle journal overlay (stays in Playing state)
                    state.perks_journal.is_open = !state.perks_journal.is_open;
                    return; // Don't pass Tab to egui or other handlers
                }
            }
            // J key also toggles journal (alternative binding)
            if let PhysicalKey::Code(KeyCode::KeyJ) = key_event.physical_key {
                if key_event.state == ElementState::Pressed && state.game_state == GameState::Playing {
                    state.perks_journal.is_open = !state.perks_journal.is_open;
                    return;
                }
            }
        }

        // Pass event to egui
        if let Some(egui_state) = &mut state.egui_state {
            if let Event::WindowEvent { event, .. } = event {
                let response = egui_state.on_window_event(window, event);
                if response.consumed {
                    return;
                }
            }
        }

        // Handle Game Input (only if Playing, not during Loading, and journal not open)
        if state.game_state == GameState::Playing {
            match event {
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    // Skip mouse look when UI overlay is open (journal or chest)
                    let ui_open = state.perks_journal.is_open || state.open_chest_id.is_some();
                    if !ui_open {
                        // Mouse Look - convert 0-100 scale to actual sensitivity (50 = 0.002)
                        let sensitivity = state.mouse_sensitivity / 25000.0;
                        state.player.yaw += delta.0 as f32 * sensitivity;
                        state.player.pitch -= delta.1 as f32 * sensitivity;
                        state.player.pitch = state.player.pitch.clamp(-1.5, 1.5);
                    }
                }
                Event::WindowEvent { event: WindowEvent::MouseInput { state: button_state, button, .. }, .. } => {
                    // Left mouse click triggers attack/shot animation
                    if *button == MouseButton::Left && *button_state == ElementState::Pressed {
                        if !state.swing_animation.is_swinging {
                            state.swing_animation.is_swinging = true;
                            state.swing_animation.swing_progress = 0.0;
                            state.swing_animation.hit_processed = false;

                            // Only trigger muzzle flash if holding a flintlock
                            let is_flintlock = state.player_economy.inventory
                                .get_slot(state.active_hotbar_slot)
                                .map(|item| item.template_id == "flintlock_pistol")
                                .unwrap_or(false);
                            if is_flintlock {
                                state.swing_animation.muzzle_flash = 1.0;
                            }
                        }
                    }
                }
                Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } => {
                    if let PhysicalKey::Code(keycode) = key_event.physical_key {
                        state.keys.insert(keycode, key_event.state);

                        // ESC key: close dialogue first, then toggle pause
                        if key_event.state == ElementState::Pressed && keycode == KeyCode::Escape {
                            if state.game_state == GameState::Playing {
                                // If in dialogue, close it instead of pausing
                                if state.current_dialogue.is_some() {
                                    state.current_dialogue = None;
                                    state.game_progression.interaction_system.interacting_npc = None;
                                    log::info!("[DIALOGUE] Closed with ESC");
                                } else {
                                    state.game_state = GameState::Paused;
                                    state.pause_menu_page = PauseMenuPage::Main;
                                    println!("[PAUSE] Game paused");
                                }
                            } else if state.game_state == GameState::Paused {
                                state.game_state = GameState::Playing;
                                println!("[PAUSE] Game resumed");
                            }
                        }

                        if key_event.state == ElementState::Pressed && state.game_state == GameState::Playing {
                            match keycode {
                                KeyCode::Space => state.player.jump(),
                                // Time controls: T = advance time, Y = reverse time
                                KeyCode::KeyT => {
                                    state.time_of_day = (state.time_of_day + 1.0) % 24.0;
                                    println!("[TIME] {:.1}:00", state.time_of_day);
                                }
                                KeyCode::KeyY => {
                                    state.time_of_day = (state.time_of_day - 1.0 + 24.0) % 24.0;
                                    println!("[TIME] {:.1}:00", state.time_of_day);
                                }
                                KeyCode::BracketLeft => {
                                    // Cycle weather backward: Clear <- Cloudy <- Overcast <- Stormy <- Foggy
                                    let prev = match state.weather.current_weather {
                                        WeatherType::Clear => WeatherType::Foggy,
                                        WeatherType::PartlyCloudy => WeatherType::Clear,
                                        WeatherType::Overcast => WeatherType::PartlyCloudy,
                                        WeatherType::Stormy => WeatherType::Overcast,
                                        WeatherType::Foggy => WeatherType::Stormy,
                                    };
                                    state.weather.set_weather(prev, true); // Instant transition for manual control
                                    state.weather.auto_weather_enabled = false; // Disable auto when manual
                                    println!("[WEATHER] << Set to {:?} (auto disabled)", prev);
                                }
                                KeyCode::BracketRight => {
                                    // Cycle weather forward: Clear -> Cloudy -> Overcast -> Stormy -> Foggy
                                    let next = match state.weather.current_weather {
                                        WeatherType::Clear => WeatherType::PartlyCloudy,
                                        WeatherType::PartlyCloudy => WeatherType::Overcast,
                                        WeatherType::Overcast => WeatherType::Stormy,
                                        WeatherType::Stormy => WeatherType::Foggy,
                                        WeatherType::Foggy => WeatherType::Clear,
                                    };
                                    state.weather.set_weather(next, true); // Instant transition for manual control
                                    state.weather.auto_weather_enabled = false; // Disable auto when manual
                                    println!("[WEATHER] >> Set to {:?} (auto disabled)", next);
                                }
                                KeyCode::KeyM => {
                                    state.audio_system.toggle_enabled();
                                }
                                // Backslash = Cycle fog level: Off -> Light -> Medium -> Heavy -> Dense
                                KeyCode::Backslash => {
                                    state.fog_level = (state.fog_level + 1) % 5;
                                    let fog_name = match state.fog_level {
                                        0 => "Off",
                                        1 => "Light",
                                        2 => "Medium",
                                        3 => "Heavy",
                                        _ => "Dense",
                                    };
                                    println!("[FOG] Level: {} ({})", state.fog_level, fog_name);
                                }
                                // C = Place campfire in front of player (requires sticks + pebbles)
                                KeyCode::KeyC => {
                                    use croatoan_wfc::mesh_gen::get_height_at;

                                    // Find ground position in front of player (3 meters ahead)
                                    let forward = state.camera.forward();
                                    let placement_offset = forward * 3.0;
                                    let target_x = state.player.position.x + placement_offset.x;
                                    let target_z = state.player.position.z + placement_offset.z;

                                    // Get terrain height at target position
                                    let (ground_height, _biome) = get_height_at(target_x, target_z, state.seed);
                                    let placement_pos = Vec3::new(target_x, ground_height, target_z);

                                    // Check for required materials
                                    let requirements = campfire::campfire_requirements();
                                    let sticks = state.player_economy.inventory.count_items("stick");
                                    let pebbles = state.player_economy.inventory.count_items("pebble");

                                    if sticks < campfire::CAMPFIRE_STICKS_REQUIRED {
                                        println!("[CAMPFIRE] Need {} sticks (have {})",
                                            campfire::CAMPFIRE_STICKS_REQUIRED, sticks);
                                    } else if pebbles < campfire::CAMPFIRE_PEBBLES_REQUIRED {
                                        println!("[CAMPFIRE] Need {} pebbles (have {})",
                                            campfire::CAMPFIRE_PEBBLES_REQUIRED, pebbles);
                                    } else if ground_height <= 0.5 {
                                        println!("[CAMPFIRE] Cannot place campfire in water!");
                                    } else if !state.campfire_manager.can_place_at(placement_pos) {
                                        println!("[CAMPFIRE] Too close to another campfire!");
                                    } else {
                                        // Consume materials and place campfire
                                        let consumed = state.player_economy.inventory.consume_recipe_materials(&requirements);
                                        if consumed {
                                            let rotation = state.player.yaw;
                                            let id = state.campfire_manager.place_campfire(placement_pos, rotation);
                                            println!("[CAMPFIRE] Placed campfire {} at ({:.1}, {:.1}, {:.1})",
                                                id.0, placement_pos.x, placement_pos.y, placement_pos.z);
                                            println!("[CAMPFIRE] Used {} sticks and {} pebbles",
                                                campfire::CAMPFIRE_STICKS_REQUIRED, campfire::CAMPFIRE_PEBBLES_REQUIRED);
                                        }
                                    }
                                }
                                // G = Drop item from current hotbar slot onto the ground
                                KeyCode::KeyG => {
                                    let slot = state.active_hotbar_slot;
                                    if let Some(item) = state.player_economy.inventory.get_slot(slot).cloned() {
                                        // Remove from inventory
                                        let item_name = item.name.clone();
                                        let item_id = item.id;
                                        if let Some(removed) = state.player_economy.inventory.remove_item(item_id) {
                                            // Drop in front of player
                                            let drop_pos = state.player.position + state.camera.forward() * 2.0 + Vec3::new(0.0, 1.0, 0.0);
                                            state.dropped_items.spawn_drop(removed, drop_pos);
                                            println!("[DROP] Dropped {} from hotbar slot {}", item_name, slot + 1);
                                            // Clear the loaded viewmodel if we dropped a weapon
                                            if item.item_type == economy::ItemType::Weapon {
                                                state.loaded_weapon_viewmodel = None;
                                            }
                                        }
                                    }
                                }
                                // F5 = Spawn debug animals in a circle around player
                                // Only spawns species that have 3D models
                                KeyCode::F5 => {
                                    use animals::AnimalSpecies;
                                    let player_pos = state.player.position;
                                    let species_list = [
                                        AnimalSpecies::GrayWolf,
                                        AnimalSpecies::RedWolf,
                                        AnimalSpecies::WhitetailDeer,
                                        AnimalSpecies::Stag,
                                        AnimalSpecies::Horse,
                                        AnimalSpecies::Donkey,
                                        AnimalSpecies::Fox,
                                        AnimalSpecies::Husky,
                                        AnimalSpecies::Bobcat, // Uses Fox model
                                    ];
                                    let num_species = species_list.len();
                                    for (i, species) in species_list.iter().enumerate() {
                                        let angle = (i as f32 / num_species as f32) * std::f32::consts::TAU;
                                        let distance = 15.0; // 15 meters from player
                                        let spawn_pos = Vec3::new(
                                            player_pos.x + angle.cos() * distance,
                                            player_pos.y,
                                            player_pos.z + angle.sin() * distance,
                                        );
                                        state.animal_manager.spawn(*species, spawn_pos, (0, 0), None);
                                    }
                                    println!("[DEBUG] Spawned {} animals around player at {:?}", num_species, player_pos);
                                }
                                // F6 = Clear all animals
                                KeyCode::F6 => {
                                    let count = state.animal_manager.animal_count();
                                    // Get all animal IDs first
                                    let ids: Vec<_> = state.animal_manager.animals_iter()
                                        .map(|a| a.id)
                                        .collect();
                                    for id in ids {
                                        state.animal_manager.despawn(id);
                                    }
                                    println!("[DEBUG] Cleared {} animals", count);
                                }
                                // E = Interact with NPC or pickup closest item
                                KeyCode::KeyE => {
                                    // Check if we're in an active dialogue - select choice 0 (or close if no choices)
                                    if let Some(dialogue) = &state.current_dialogue {
                                        if dialogue.choices.is_empty() {
                                            // End dialogue
                                            state.current_dialogue = None;
                                            state.game_progression.interaction_system.interacting_npc = None;
                                            log::info!("[DIALOGUE] Ended");
                                        } else {
                                            // Make choice 0 by pressing E (1-4 for specific choices)
                                            let game_time = state.game_progression.game_time;
                                            if let Some(next) = state.game_progression.interaction_system.make_choice(0, game_time) {
                                                state.current_dialogue = Some(next);
                                                log::info!("[DIALOGUE] Selected choice 1");
                                            } else {
                                                // Dialogue ended
                                                state.current_dialogue = None;
                                                state.game_progression.interaction_system.interacting_npc = None;
                                                log::info!("[DIALOGUE] Ended");
                                            }
                                        }
                                    }
                                    // Check if looking at an NPC to start dialogue
                                    else if let Some(focused) = state.village_manager.focused_npc.clone() {
                                        let game_time = state.game_progression.game_time;
                                        if let Some(dialogue_data) = state.game_progression.interaction_system.start_interaction(
                                            focused.index,
                                            &focused.name,
                                            &focused.role,
                                            focused.position,
                                            game_time,
                                        ) {
                                            state.current_dialogue = Some(dialogue_data);
                                            log::info!("[DIALOGUE] Started with {} ({})", focused.name, focused.role);
                                        }
                                    }
                                    // Check if near a harvestable plant
                                    else if let Some((plant_id, species, dist, can_harvest)) = state.forageable_plants.get_closest_harvestable(
                                        [state.player.position.x, state.player.position.y, state.player.position.z],
                                        4.0, // Harvest range
                                    ) {
                                        if can_harvest && dist < 4.0 {
                                            // Harvest the plant
                                            if let Some((species, result)) = state.forageable_plants.harvest_plant(plant_id) {
                                                match result {
                                                    flora::growth::HarvestResult::Success { quality, quantity } => {
                                                        // Get harvest items
                                                        let items = flora::harvest::get_harvest_drops(species, quality);

                                                        // Notify systems
                                                        let player_pos = state.player.position;
                                                        state.systems_manager.record_harvest(species, player_pos);

                                                        // Add items to inventory
                                                        for item in &items {
                                                            let item_type = if item.properties.is_food {
                                                                economy::ItemType::Food
                                                            } else if item.properties.is_medicine {
                                                                economy::ItemType::Medicine
                                                            } else {
                                                                economy::ItemType::Material
                                                            };

                                                            let mut inv_item = economy::Item::new(
                                                                item.item_id,
                                                                item.name,
                                                                item_type,
                                                                item.properties.value,
                                                            );
                                                            inv_item.stack_size = item.quantity;
                                                            inv_item.max_stack = item.properties.max_stack;
                                                            inv_item.rarity = match quality {
                                                                flora::growth::HarvestQuality::Poor => economy::Rarity::Crude,
                                                                flora::growth::HarvestQuality::Average => economy::Rarity::Common,
                                                                flora::growth::HarvestQuality::Good => economy::Rarity::Uncommon,
                                                                flora::growth::HarvestQuality::Prime => economy::Rarity::Rare,
                                                            };
                                                            let _ = state.player_economy.inventory.add_item(inv_item);
                                                        }

                                                        log::info!("[FORAGE] Harvested {} - {} items ({:?} quality)",
                                                            species.name(), items.len(), quality);
                                                    }
                                                    flora::growth::HarvestResult::Failed => {
                                                        log::info!("[FORAGE] Cannot harvest {} yet", species.name());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    // Check if chest UI is open - close it
                                    else if state.open_chest_id.is_some() {
                                        // Close chest
                                        if let Some(chest_id) = state.open_chest_id.take() {
                                            if let Some(chest) = state.storage_manager.get_mut(chest_id) {
                                                chest.close();
                                            }
                                        }
                                        log::info!("[CHEST] Closed");
                                    }
                                    // Check if near a chest to open it
                                    else if let Some(chest) = state.storage_manager.nearest_interactable(
                                        state.player.position,
                                        None, // player_id for lock check
                                        3.5,  // interaction range
                                    ) {
                                        let chest_id = chest.id;
                                        let chest_name = chest.display_name().to_string();
                                        // Open the chest
                                        if let Some(chest) = state.storage_manager.get_mut(chest_id) {
                                            chest.open();
                                        }
                                        state.open_chest_id = Some(chest_id);
                                        // Play chest creak sound
                                        if let Ok(sfx) = state.audio_system.load_sound("assets/audio/sfx/chest open creak.wav") {
                                            state.audio_system.play_sfx(sfx);
                                        }
                                        log::info!("[CHEST] Opened {}", chest_name);
                                    }
                                    // Otherwise pickup closest item
                                    else {
                                        let pickup_range = 3.0;
                                        if let Some(closest) = state.dropped_items.closest_drop(
                                            state.player.position,
                                            pickup_range,
                                        ) {
                                            let item_name = closest.item.name.clone();
                                            let rarity = closest.item.rarity;
                                            let drop_id = closest.id;

                                            // Pick up the item
                                            if let Some(picked_up) = state.dropped_items.pickup(drop_id) {
                                                // Add to player inventory
                                                if let Err(e) = state.player_economy.inventory.add_item(picked_up.item) {
                                                    log::warn!("Failed to add item to inventory: {:?}", e);
                                                } else {
                                                    log::info!("[PICKUP] {} ({:?})", item_name, rarity);
                                                }
                                            }
                                        }
                                    }
                                }
                                // Q = Drop item from current hotbar slot
                                KeyCode::KeyQ => {
                                    let slot = state.active_hotbar_slot;
                                    if let Some(item) = state.player_economy.inventory.get_slot(slot).cloned() {
                                        // Remove from inventory
                                        let item_id = item.id;
                                        let is_weapon = item.item_type == economy::ItemType::Weapon;
                                        if let Some(removed) = state.player_economy.inventory.remove_item(item_id) {
                                            // Drop in front of player
                                            let drop_pos = state.player.position + state.camera.forward() * 1.5 + Vec3::new(0.0, 0.5, 0.0);
                                            state.dropped_items.spawn_drop(removed, drop_pos);
                                            log::info!("[DROP] {} from hotbar slot {}", item.name, slot + 1);
                                            // Clear the loaded viewmodel if we dropped a weapon
                                            if is_weapon {
                                                state.loaded_weapon_viewmodel = None;
                                            }
                                        }
                                    }
                                }
                                // Number keys 1-4 = Dialogue choices (when in dialogue), otherwise hotbar
                                KeyCode::Digit1 => {
                                    if state.current_dialogue.is_some() {
                                        let game_time = state.game_progression.game_time;
                                        if let Some(next) = state.game_progression.interaction_system.make_choice(0, game_time) {
                                            state.current_dialogue = Some(next);
                                            log::info!("[DIALOGUE] Selected choice 1");
                                        } else {
                                            state.current_dialogue = None;
                                            state.game_progression.interaction_system.interacting_npc = None;
                                        }
                                    } else {
                                        state.active_hotbar_slot = 0;
                                    }
                                }
                                KeyCode::Digit2 => {
                                    if state.current_dialogue.is_some() {
                                        let game_time = state.game_progression.game_time;
                                        if let Some(next) = state.game_progression.interaction_system.make_choice(1, game_time) {
                                            state.current_dialogue = Some(next);
                                            log::info!("[DIALOGUE] Selected choice 2");
                                        } else {
                                            state.current_dialogue = None;
                                            state.game_progression.interaction_system.interacting_npc = None;
                                        }
                                    } else {
                                        state.active_hotbar_slot = 1;
                                    }
                                }
                                KeyCode::Digit3 => {
                                    if state.current_dialogue.is_some() {
                                        let game_time = state.game_progression.game_time;
                                        if let Some(next) = state.game_progression.interaction_system.make_choice(2, game_time) {
                                            state.current_dialogue = Some(next);
                                            log::info!("[DIALOGUE] Selected choice 3");
                                        } else {
                                            state.current_dialogue = None;
                                            state.game_progression.interaction_system.interacting_npc = None;
                                        }
                                    } else {
                                        state.active_hotbar_slot = 2;
                                    }
                                }
                                KeyCode::Digit4 => {
                                    if state.current_dialogue.is_some() {
                                        let game_time = state.game_progression.game_time;
                                        if let Some(next) = state.game_progression.interaction_system.make_choice(3, game_time) {
                                            state.current_dialogue = Some(next);
                                            log::info!("[DIALOGUE] Selected choice 4");
                                        } else {
                                            state.current_dialogue = None;
                                            state.game_progression.interaction_system.interacting_npc = None;
                                        }
                                    } else {
                                        state.active_hotbar_slot = 3;
                                    }
                                }
                                KeyCode::Digit5 => state.active_hotbar_slot = 4,
                                KeyCode::Digit6 => state.active_hotbar_slot = 5,
                                KeyCode::Digit7 => state.active_hotbar_slot = 6,
                                KeyCode::Digit8 => state.active_hotbar_slot = 7,
                                KeyCode::Digit9 => state.active_hotbar_slot = 8,
                                KeyCode::Digit0 => state.active_hotbar_slot = 9,
                                // F12 = Toggle debug/performance UI
                                KeyCode::F12 => {
                                    state.show_debug_ui = !state.show_debug_ui;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    // --- Render Callback ---
    let render_state = Arc::clone(&shared_state);
    let render_rx = Arc::clone(&chunk_rx);
    
    app.set_render_callback(move |ctx| {
        // Shadow System (initialized early so it's available for asset loading)
        static SHADOW_SYSTEM: OnceLock<(Mutex<ShadowMap>, Mutex<ShadowPipeline>, Mutex<InstancedShadowPipeline>)> = OnceLock::new();
        let (shadow_map_mutex, _shadow_pipeline_mutex_early, instanced_shadow_pipeline_mutex) = SHADOW_SYSTEM.get_or_init(|| {
            let shadow_map = ShadowMap::new(ctx.device(), 2048);
            let shadow_pipeline = ShadowPipeline::new(ctx.device());
            let instanced_shadow_pipeline = InstancedShadowPipeline::new(ctx.device());
            (Mutex::new(shadow_map), Mutex::new(shadow_pipeline), Mutex::new(instanced_shadow_pipeline))
        });

        // Initialize Asset Registry if empty
        {
            let mut state = render_state.safe_lock();
            if state.mesh_registry.is_empty() {
                println!("[GPU] Initializing Mesh Registry...");

                // ========================================================================
                // TREE SYSTEM - FRAMEWORK ONLY
                // ========================================================================
                // Current status: NO TREE MODELS AVAILABLE
                //
                // The assets/models/foliage/scene.gltf contains only billboard grass planes,
                // NOT actual 3D tree models. The tree system requires proper tree GLTFs.
                //
                // To add trees:
                // 1. Acquire tree GLTF models (e.g., from Poly Haven, Sketchfab)
                // 2. Place in assets/models/trees/
                // 3. Load here using gltf_loader::load_gltf()
                // 4. Register as "tree_oak", "tree_pine", etc. in mesh_registry
                //
                // The TreePipeline infrastructure is ready:
                // - TreePipeline::create_mesh() for GPU mesh creation
                // - TreePipeline::new() for rendering pipeline
                // - tree_groups from generate_foliage_for_chunk() for placement
                // ========================================================================
                // Create a temporary TreePipeline to get access to texture_bind_group_layout
                let shadow_map = shadow_map_mutex.safe_lock();
                let texture_helper = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map);
                drop(shadow_map);

                // Load tree models from assets/models/trees/
                // Each tree GLB contains multiple meshes (bark + leaves) with different textures.
                // We create TWO meshes per tree: one for bark (OPAQUE), one for leaves (BLEND).
                // NOTE: tree_0, tree_1 are single-LOD placeholders - DISABLED
                // Species names map to their file names; some LOD0 models have special names with y-offset
                let tree_models: Vec<(&str, &str)> = vec![
                    // (registry_name, file_name) - file_name is what to load from disk
                    ("birch_0", "birch_0_lod0_highpoly_yoffset"), // Use highpoly version with y-offset fix
                    ("pine_0", "pine_0"),
                    ("dead_conifer_0", "dead_conifer_0"),
                    ("fir_0", "fir_0"),
                    ("fir_1", "fir_1"),
                ];
                // OPTIMIZED: Using models_optimized for reduced texture/poly models
                let mut tree_cache = gltf_loader::ModelCache::new("assets/models_optimized/trees");
                for (registry_name, file_name) in &tree_models {
                    if let Some(model) = tree_cache.load(file_name) {
                        // Separate meshes into bark (OPAQUE) and leaves (BLEND)
                        let mut bark_positions: Vec<[f32; 3]> = Vec::new();
                        let mut bark_normals: Vec<[f32; 3]> = Vec::new();
                        let mut bark_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut bark_indices: Vec<u32> = Vec::new();
                        let mut bark_texture: Option<&gltf_loader::LoadedTexture> = None;

                        let mut leaf_positions: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_normals: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut leaf_indices: Vec<u32> = Vec::new();
                        let mut leaf_texture: Option<&gltf_loader::LoadedTexture> = None;

                        // First pass: find the height range of the model
                        let mut min_y = f32::MAX;
                        let mut max_y = f32::MIN;
                        for mesh in &model.meshes {
                            for pos in &mesh.positions {
                                min_y = min_y.min(pos[1]);
                                max_y = max_y.max(pos[1]);
                            }
                        }
                        let height_range = max_y - min_y;
                        // Leaves below this threshold will be culled
                        // 0.25 = leaves start at 25% of tree height (full canopy, clear trunk base)
                        // Was 0.75 which culled 75% of leaves - way too aggressive!
                        let leaf_cull_height = min_y + height_range * 0.25;

                        for mesh in &model.meshes {
                            let is_leaves = mesh.material.alpha_mode == "BLEND" || mesh.material.alpha_mode == "MASK";

                            if is_leaves {
                                // For leaves: filter out triangles near the bottom
                                let base_idx = leaf_positions.len() as u32;

                                // Add all vertices first
                                let vert_offset = leaf_positions.len();
                                leaf_positions.extend_from_slice(&mesh.positions);
                                leaf_normals.extend_from_slice(&mesh.normals);
                                leaf_uvs.extend_from_slice(&mesh.uvs);

                                // Filter triangles - only keep those above cull height
                                for tri in mesh.indices.chunks(3) {
                                    if tri.len() == 3 {
                                        let i0 = tri[0] as usize;
                                        let i1 = tri[1] as usize;
                                        let i2 = tri[2] as usize;

                                        // Get average Y of triangle
                                        let avg_y = (mesh.positions[i0][1] + mesh.positions[i1][1] + mesh.positions[i2][1]) / 3.0;

                                        // Keep triangle if it's above the cull threshold
                                        if avg_y > leaf_cull_height {
                                            leaf_indices.push(tri[0] + base_idx);
                                            leaf_indices.push(tri[1] + base_idx);
                                            leaf_indices.push(tri[2] + base_idx);
                                        }
                                    }
                                }

                                // Use first leaf texture we find
                                if leaf_texture.is_none() {
                                    leaf_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            } else {
                                // Combine into bark mesh (no filtering)
                                let base_idx = bark_positions.len() as u32;
                                bark_positions.extend_from_slice(&mesh.positions);
                                bark_normals.extend_from_slice(&mesh.normals);
                                bark_uvs.extend_from_slice(&mesh.uvs);
                                bark_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                // Use first bark texture we find
                                if bark_texture.is_none() {
                                    bark_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            }
                        }

                        // Create bark mesh
                        if !bark_positions.is_empty() {
                            let texture_bind_group = bark_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_bark_texture", registry_name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_bark_bind", registry_name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &bark_positions, &bark_normals, &bark_uvs, &bark_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_bark", registry_name), gpu_mesh);
                        }

                        // Create leaves mesh
                        if !leaf_positions.is_empty() {
                            let texture_bind_group = leaf_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_leaf_texture", registry_name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_leaf_bind", registry_name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &leaf_positions, &leaf_normals, &leaf_uvs, &leaf_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_leaves", registry_name), gpu_mesh);
                        }

                        // Also register combined mesh for backwards compatibility
                        // (uses bark texture - procedural shader handles leaves via UV heuristic)
                        if !bark_positions.is_empty() {
                            let all_positions: Vec<[f32; 3]> = bark_positions.iter().chain(leaf_positions.iter()).cloned().collect();
                            let all_normals: Vec<[f32; 3]> = bark_normals.iter().chain(leaf_normals.iter()).cloned().collect();
                            let all_uvs: Vec<[f32; 2]> = bark_uvs.iter().chain(leaf_uvs.iter()).cloned().collect();
                            let bark_count = bark_indices.len();
                            let all_indices: Vec<u32> = bark_indices.iter().cloned()
                                .chain(leaf_indices.iter().map(|i| i + bark_positions.len() as u32))
                                .collect();

                            let texture_bind_group = bark_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_combined_texture", registry_name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_combined_bind", registry_name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &all_positions, &all_normals, &all_uvs, &all_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(registry_name.to_string(), gpu_mesh);
                        }
                    } else {
                        // FALLBACK: Generate procedural tree when GLB not found
                        println!("[WARN] Tree '{}' (file: {}) NOT FOUND - using procedural fallback!", registry_name, file_name);

                        // Choose tree style based on name
                        let proc_mesh = if registry_name.contains("pine") || registry_name.contains("1") {
                            generate_conifer_tree(registry_name.as_bytes().iter().map(|&b| b as u64).sum())
                        } else {
                            generate_deciduous_tree(registry_name.as_bytes().iter().map(|&b| b as u64).sum())
                        };

                        let positions: Vec<[f32; 3]> = proc_mesh.vertices.iter().map(|v| v.position).collect();
                        let normals: Vec<[f32; 3]> = proc_mesh.vertices.iter().map(|v| v.normal).collect();
                        let uvs: Vec<[f32; 2]> = proc_mesh.vertices.iter().map(|v| v.uv).collect();

                        // Create combined mesh (bark + leaves in one) with no texture (procedural colors)
                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(),
                            &positions,
                            &normals,
                            &uvs,
                            &proc_mesh.indices,
                            None, // No texture - uses procedural coloring in shader
                        );
                        state.mesh_registry.insert(registry_name.to_string(), gpu_mesh);
                    }
                }

                // Load or generate LOD1 meshes for distant tree rendering
                // First try to load artist-created GLB, fall back to procedural generation
                let lod1_tree_models = [
                    // ("tree_0_lod1", "tree_0"), // DISABLED
                    // ("tree_1_lod1", "tree_1"), // DISABLED
                    ("birch_0_lod1", "birch_0"),
                    ("pine_0_lod1", "pine_0"),
                    ("dead_conifer_0_lod1", "dead_conifer_0"),
                    ("fir_0_lod1", "fir_0"),
                    ("fir_1_lod1", "fir_1"),
                ];
                // OPTIMIZED: Using models_optimized for reduced texture/poly models
                let mut lod1_cache = gltf_loader::ModelCache::new("assets/models_optimized/trees");

                for (i, (lod1_name, base_name)) in lod1_tree_models.iter().enumerate() {
                    // Try to load GLB first
                    if let Some(model) = lod1_cache.load(lod1_name) {
                        // Load from GLB - separate bark (OPAQUE) and leaves (BLEND) like LOD0
                        let mut bark_positions: Vec<[f32; 3]> = Vec::new();
                        let mut bark_normals: Vec<[f32; 3]> = Vec::new();
                        let mut bark_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut bark_indices: Vec<u32> = Vec::new();
                        let mut bark_texture: Option<&gltf_loader::LoadedTexture> = None;

                        let mut leaf_positions: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_normals: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut leaf_indices: Vec<u32> = Vec::new();
                        let mut leaf_texture: Option<&gltf_loader::LoadedTexture> = None;

                        for mesh in model.meshes.iter() {
                            let is_leaves = mesh.material.alpha_mode == "BLEND" || mesh.material.alpha_mode == "MASK";
                            if is_leaves {
                                let base_idx = leaf_positions.len() as u32;
                                leaf_positions.extend_from_slice(&mesh.positions);
                                leaf_normals.extend_from_slice(&mesh.normals);
                                leaf_uvs.extend_from_slice(&mesh.uvs);
                                leaf_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                if leaf_texture.is_none() {
                                    leaf_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            } else {
                                let base_idx = bark_positions.len() as u32;
                                bark_positions.extend_from_slice(&mesh.positions);
                                bark_normals.extend_from_slice(&mesh.normals);
                                bark_uvs.extend_from_slice(&mesh.uvs);
                                bark_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                if bark_texture.is_none() {
                                    bark_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            }
                        }

                        // Create bark mesh for LOD1
                        if !bark_positions.is_empty() {
                            let texture_bind_group = bark_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_lod1_bark_texture", base_name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_lod1_bark_bind", base_name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &bark_positions, &bark_normals, &bark_uvs, &bark_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_lod1_bark", base_name), gpu_mesh);
                        }

                        // Create leaves mesh for LOD1
                        if !leaf_positions.is_empty() {
                            let texture_bind_group = leaf_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_lod1_leaf_texture", base_name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_lod1_leaf_bind", base_name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &leaf_positions, &leaf_normals, &leaf_uvs, &leaf_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_lod1_leaves", base_name), gpu_mesh);
                        }
                    } else {
                        // Fall back to procedural generation
                        let lod1_mesh = if base_name.contains("birch") {
                            generate_lod1_tree_mesh(
                                4.5, 0.18, 5.5, 1.6, // Taller, thinner birch shape
                                (i as u64).wrapping_mul(12345),
                            )
                        } else if base_name.contains("pine") {
                            generate_lod1_tree_mesh(
                                6.0, 0.25, 4.0, 2.5, // Tall conical pine shape
                                (i as u64).wrapping_mul(12345),
                            )
                        } else {
                            generate_default_lod1_tree((i as u64).wrapping_mul(12345))
                        };
                        let positions: Vec<[f32; 3]> = lod1_mesh.vertices.iter().map(|v| v.position).collect();
                        let normals: Vec<[f32; 3]> = lod1_mesh.vertices.iter().map(|v| v.normal).collect();
                        let uvs: Vec<[f32; 2]> = lod1_mesh.vertices.iter().map(|v| v.uv).collect();

                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(), &positions, &normals, &uvs, &lod1_mesh.indices, None,
                        );
                        state.mesh_registry.insert(lod1_name.to_string(), gpu_mesh);
                    }
                }

                // Load LOD2 meshes for very distant tree rendering
                // Separate bark (OPAQUE) and leaves (BLEND) like LOD0/LOD1 for proper texturing
                let lod2_tree_models = [
                    // ("tree_0_lod2", "tree_0"), // DISABLED
                    // ("tree_1_lod2", "tree_1"), // DISABLED
                    ("birch_0_lod2", "birch_0"),
                    ("pine_0_lod2", "pine_0"),
                    ("dead_conifer_0_lod2", "dead_conifer_0"),
                    ("fir_0_lod2", "fir_0"),
                    ("fir_1_lod2", "fir_1"),
                ];
                // OPTIMIZED: Using models_optimized for reduced texture/poly models
                let mut lod2_cache = gltf_loader::ModelCache::new("assets/models_optimized/trees");

                for (_i, (lod2_name, base_name)) in lod2_tree_models.iter().enumerate() {
                    if let Some(model) = lod2_cache.load(lod2_name) {
                        // Separate meshes into bark (OPAQUE) and leaves (BLEND) like LOD0/LOD1
                        let mut bark_positions: Vec<[f32; 3]> = Vec::new();
                        let mut bark_normals: Vec<[f32; 3]> = Vec::new();
                        let mut bark_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut bark_indices: Vec<u32> = Vec::new();
                        let mut bark_texture: Option<&gltf_loader::LoadedTexture> = None;

                        let mut leaf_positions: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_normals: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut leaf_indices: Vec<u32> = Vec::new();
                        let mut leaf_texture: Option<&gltf_loader::LoadedTexture> = None;

                        for mesh in &model.meshes {
                            let is_leaves = mesh.material.alpha_mode == "BLEND" || mesh.material.alpha_mode == "MASK";
                            if is_leaves {
                                let base_idx = leaf_positions.len() as u32;
                                leaf_positions.extend_from_slice(&mesh.positions);
                                leaf_normals.extend_from_slice(&mesh.normals);
                                leaf_uvs.extend_from_slice(&mesh.uvs);
                                leaf_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                if leaf_texture.is_none() {
                                    leaf_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            } else {
                                let base_idx = bark_positions.len() as u32;
                                bark_positions.extend_from_slice(&mesh.positions);
                                bark_normals.extend_from_slice(&mesh.normals);
                                bark_uvs.extend_from_slice(&mesh.uvs);
                                bark_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                if bark_texture.is_none() {
                                    bark_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            }
                        }

                        // Create bark mesh with texture
                        if !bark_positions.is_empty() {
                            let bark_bind_group = bark_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_lod2_bark_texture", base_name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_lod2_bark_bind", base_name)),
                                ))
                            });
                            let bark_mesh = TreePipeline::create_mesh(
                                ctx.device(), &bark_positions, &bark_normals, &bark_uvs, &bark_indices, bark_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_lod2_bark", base_name), bark_mesh);
                        }

                        // Create leaves mesh with texture
                        if !leaf_positions.is_empty() {
                            let leaf_bind_group = leaf_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_lod2_leaves_texture", base_name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_lod2_leaves_bind", base_name)),
                                ))
                            });
                            let leaf_mesh = TreePipeline::create_mesh(
                                ctx.device(), &leaf_positions, &leaf_normals, &leaf_uvs, &leaf_indices, leaf_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_lod2_leaves", base_name), leaf_mesh);
                        }
                    }
                }

                // Load shrub/bush models from assets/models/shrubs/
                // Same pattern as trees: separate bark and leaves meshes
                // Includes beach grass (low-poly clumps for upper beach)
                // NOTE: shrub_0, bush_0, grass_0, beach_grass_0 disabled - using grass3 LOD system
                let shrub_models = [
                    // "shrub_0", "bush_0", "grass_0", // DISABLED - single LOD placeholders
                    // "beach_grass_0", // DISABLED - replaced by grass3 LOD system
                    "conifer_shrub_0", // New 3-LOD shrub
                ];
                // OPTIMIZED: Using models_optimized for reduced texture/poly models
                let mut shrub_cache = gltf_loader::ModelCache::new("assets/models_optimized/shrubs");
                for name in &shrub_models {
                    if let Some(model) = shrub_cache.load(name) {
                        // Debug: show mesh breakdown
                        println!("[SHRUB DEBUG] '{}' has {} meshes:", name, model.meshes.len());
                        for (i, mesh) in model.meshes.iter().enumerate() {
                            let has_tex = mesh.material.base_color_texture_data.is_some();
                            println!("  mesh[{}]: alpha_mode='{}', has_texture={}, verts={}",
                                i, mesh.material.alpha_mode, has_tex, mesh.positions.len());
                        }
                        // Separate meshes into bark (OPAQUE) and leaves (BLEND)
                        let mut bark_positions: Vec<[f32; 3]> = Vec::new();
                        let mut bark_normals: Vec<[f32; 3]> = Vec::new();
                        let mut bark_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut bark_indices: Vec<u32> = Vec::new();
                        let mut bark_texture: Option<&gltf_loader::LoadedTexture> = None;

                        let mut leaf_positions: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_normals: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut leaf_indices: Vec<u32> = Vec::new();
                        let mut leaf_texture: Option<&gltf_loader::LoadedTexture> = None;

                        for mesh in &model.meshes {
                            let is_leaves = mesh.material.alpha_mode == "BLEND" || mesh.material.alpha_mode == "MASK";

                            if is_leaves {
                                let base_idx = leaf_positions.len() as u32;
                                leaf_positions.extend_from_slice(&mesh.positions);
                                leaf_normals.extend_from_slice(&mesh.normals);
                                leaf_uvs.extend_from_slice(&mesh.uvs);
                                leaf_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                if leaf_texture.is_none() {
                                    leaf_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            } else {
                                let base_idx = bark_positions.len() as u32;
                                bark_positions.extend_from_slice(&mesh.positions);
                                bark_normals.extend_from_slice(&mesh.normals);
                                bark_uvs.extend_from_slice(&mesh.uvs);
                                bark_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                if bark_texture.is_none() {
                                    bark_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            }
                        }

                        // Create bark mesh
                        if !bark_positions.is_empty() {
                            let texture_bind_group = bark_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_bark_texture", name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_bark_bind", name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &bark_positions, &bark_normals, &bark_uvs, &bark_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_bark", name), gpu_mesh);
                        }

                        // Create leaves mesh
                        if !leaf_positions.is_empty() {
                            let texture_bind_group = leaf_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_leaf_texture", name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_leaf_bind", name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &leaf_positions, &leaf_normals, &leaf_uvs, &leaf_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_leaves", name), gpu_mesh);
                        }
                    } else {
                        println!("[WARN] Shrub '{}' NOT FOUND!", name);
                    }
                }

                // Load conifer_shrub_0 LOD1 and LOD2 with bark/leaves separation
                for lod in 1..=2 {
                    let lod_name = format!("conifer_shrub_0_lod{}", lod);
                    if let Some(model) = shrub_cache.load(&lod_name) {
                        // Separate bark (OPAQUE) and leaves (BLEND) like LOD0
                        let mut bark_positions: Vec<[f32; 3]> = Vec::new();
                        let mut bark_normals: Vec<[f32; 3]> = Vec::new();
                        let mut bark_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut bark_indices: Vec<u32> = Vec::new();
                        let mut bark_texture: Option<&gltf_loader::LoadedTexture> = None;

                        let mut leaf_positions: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_normals: Vec<[f32; 3]> = Vec::new();
                        let mut leaf_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut leaf_indices: Vec<u32> = Vec::new();
                        let mut leaf_texture: Option<&gltf_loader::LoadedTexture> = None;

                        for mesh in &model.meshes {
                            let is_leaves = mesh.material.alpha_mode == "BLEND" || mesh.material.alpha_mode == "MASK";

                            if is_leaves {
                                let base_idx = leaf_positions.len() as u32;
                                leaf_positions.extend_from_slice(&mesh.positions);
                                leaf_normals.extend_from_slice(&mesh.normals);
                                leaf_uvs.extend_from_slice(&mesh.uvs);
                                leaf_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                if leaf_texture.is_none() {
                                    leaf_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            } else {
                                let base_idx = bark_positions.len() as u32;
                                bark_positions.extend_from_slice(&mesh.positions);
                                bark_normals.extend_from_slice(&mesh.normals);
                                bark_uvs.extend_from_slice(&mesh.uvs);
                                bark_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                                if bark_texture.is_none() {
                                    bark_texture = mesh.material.base_color_texture_data.as_ref();
                                }
                            }
                        }

                        // Create bark mesh
                        if !bark_positions.is_empty() {
                            let texture_bind_group = bark_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_bark_texture", lod_name)),
                                );
                                Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_bark_bind", lod_name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &bark_positions, &bark_normals, &bark_uvs, &bark_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_bark", lod_name), gpu_mesh);
                        }

                        // Create leaves mesh
                        if !leaf_positions.is_empty() {
                            let texture_bind_group = leaf_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_leaf_texture", lod_name)),
                                );
                                Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_leaf_bind", lod_name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &leaf_positions, &leaf_normals, &leaf_uvs, &leaf_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_leaves", lod_name), gpu_mesh);
                        }
                    }
                }

                // Load fern models from assets/models/shrubs/
                // Ferns are single mesh with MASK alpha mode for leaf cutout
                // NOTE: fern_01.glb was corrupted (empty scene), using fern_02
                let fern_models = ["fern_02"];
                // OPTIMIZED: Using models_optimized for reduced texture/poly models
                let mut fern_cache = gltf_loader::ModelCache::new("assets/models_optimized/shrubs");
                for name in &fern_models {
                    if let Some(model) = fern_cache.load(name) {
                        // Ferns are single combined mesh - no bark/leaf separation
                        let mut all_positions: Vec<[f32; 3]> = Vec::new();
                        let mut all_normals: Vec<[f32; 3]> = Vec::new();
                        let mut all_uvs: Vec<[f32; 2]> = Vec::new();
                        let mut all_indices: Vec<u32> = Vec::new();
                        let mut fern_texture: Option<&gltf_loader::LoadedTexture> = None;

                        for mesh in &model.meshes {
                            let base_idx = all_positions.len() as u32;
                            all_positions.extend_from_slice(&mesh.positions);
                            all_normals.extend_from_slice(&mesh.normals);
                            all_uvs.extend_from_slice(&mesh.uvs);
                            all_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                            if fern_texture.is_none() {
                                fern_texture = mesh.material.base_color_texture_data.as_ref();
                            }
                        }

                        if !all_positions.is_empty() {
                            let texture_bind_group = fern_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_texture", name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_bind", name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(),
                                &all_positions,
                                &all_normals,
                                &all_uvs,
                                &all_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(name.to_string(), gpu_mesh);
                            println!("[FERN] Registered '{}': {} verts, {} tris",
                                name, all_positions.len(), all_indices.len() / 3);
                        }
                    } else {
                        println!("[FERN] WARNING: Model '{}' not found in assets/models_optimized/shrubs/", name);
                    }
                }

                // Load dead grass chunk models from assets/models/grass/
                // dead_grass DISABLED - replaced by grass2/grass3 system

                // Load grass2 LOD models (inland/meadow/forest ground cover)
                // OPTIMIZED: Using models_optimized/grass with separate LOD models
                // LOD0: grass2.glb (2.2MB, 2160px texture) - close range
                // LOD1: grass2_lod1.glb (83KB, 256px texture) - distant
                let mut grass2_cache = gltf_loader::ModelCache::new("assets/models_optimized/grass");

                // Load LOD0 (high quality)
                if let Some(model) = grass2_cache.load("grass2") {
                    let mut all_positions: Vec<[f32; 3]> = Vec::new();
                    let mut all_normals: Vec<[f32; 3]> = Vec::new();
                    let mut all_uvs: Vec<[f32; 2]> = Vec::new();
                    let mut all_indices: Vec<u32> = Vec::new();
                    let mut grass_texture: Option<&gltf_loader::LoadedTexture> = None;

                    for mesh in &model.meshes {
                        let base_idx = all_positions.len() as u32;
                        all_positions.extend_from_slice(&mesh.positions);
                        all_normals.extend_from_slice(&mesh.normals);
                        all_uvs.extend_from_slice(&mesh.uvs);
                        all_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                        if grass_texture.is_none() {
                            grass_texture = mesh.material.base_color_texture_data.as_ref();
                        }
                    }

                    if !all_positions.is_empty() {
                        let texture_bind_group = grass_texture.map(|tex_data| {
                            let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                ctx.device(), ctx.queue(), tex_data,
                                Some("grass2_lod0_texture"),
                            );
                            std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                ctx.device(), &tex_view, Some("grass2_lod0_bind"),
                            ))
                        });
                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(), &all_positions, &all_normals, &all_uvs, &all_indices,
                            texture_bind_group,
                        );
                        state.mesh_registry.insert("grass2_lod0".to_string(), gpu_mesh);
                        println!("[GRASS2] LOD0: {} verts, {} tris", all_positions.len(), all_indices.len() / 3);
                    }
                } else {
                    println!("[GRASS2] LOD0 model 'grass2' not found");
                }

                // Load LOD1 (low quality - 96% smaller)
                if let Some(model) = grass2_cache.load("grass2_lod1") {
                    let mut all_positions: Vec<[f32; 3]> = Vec::new();
                    let mut all_normals: Vec<[f32; 3]> = Vec::new();
                    let mut all_uvs: Vec<[f32; 2]> = Vec::new();
                    let mut all_indices: Vec<u32> = Vec::new();
                    let mut grass_texture: Option<&gltf_loader::LoadedTexture> = None;

                    for mesh in &model.meshes {
                        let base_idx = all_positions.len() as u32;
                        all_positions.extend_from_slice(&mesh.positions);
                        all_normals.extend_from_slice(&mesh.normals);
                        all_uvs.extend_from_slice(&mesh.uvs);
                        all_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                        if grass_texture.is_none() {
                            grass_texture = mesh.material.base_color_texture_data.as_ref();
                        }
                    }

                    if !all_positions.is_empty() {
                        let texture_bind_group = grass_texture.map(|tex_data| {
                            let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                ctx.device(), ctx.queue(), tex_data,
                                Some("grass2_lod1_texture"),
                            );
                            std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                ctx.device(), &tex_view, Some("grass2_lod1_bind"),
                            ))
                        });
                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(), &all_positions, &all_normals, &all_uvs, &all_indices,
                            texture_bind_group,
                        );
                        // Use LOD1 for both lod1 and lod2 (same low-poly model)
                        state.mesh_registry.insert("grass2_lod1".to_string(), gpu_mesh.clone());
                        state.mesh_registry.insert("grass2_lod2".to_string(), gpu_mesh);
                        println!("[GRASS2] LOD1/2: {} verts, {} tris (96% smaller)", all_positions.len(), all_indices.len() / 3);
                    }
                } else {
                    println!("[GRASS2] LOD1 model 'grass2_lod1' not found");
                }

                // Load grass3 LOD models (beach/riverbank grass)
                // OPTIMIZED: Using models_optimized/grass with separate LOD models
                // LOD0: grass3.glb (2.2MB, 2160px texture) - close range
                // LOD1: grass3_lod1.glb (82KB, 256px texture) - distant
                let mut grass3_cache = gltf_loader::ModelCache::new("assets/models_optimized/grass");

                // Load LOD0 (high quality)
                if let Some(model) = grass3_cache.load("grass3") {
                    let mut all_positions: Vec<[f32; 3]> = Vec::new();
                    let mut all_normals: Vec<[f32; 3]> = Vec::new();
                    let mut all_uvs: Vec<[f32; 2]> = Vec::new();
                    let mut all_indices: Vec<u32> = Vec::new();
                    let mut grass_texture: Option<&gltf_loader::LoadedTexture> = None;

                    for mesh in &model.meshes {
                        let base_idx = all_positions.len() as u32;
                        all_positions.extend_from_slice(&mesh.positions);
                        all_normals.extend_from_slice(&mesh.normals);
                        all_uvs.extend_from_slice(&mesh.uvs);
                        all_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                        if grass_texture.is_none() {
                            grass_texture = mesh.material.base_color_texture_data.as_ref();
                        }
                    }

                    if !all_positions.is_empty() {
                        let texture_bind_group = grass_texture.map(|tex_data| {
                            let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                ctx.device(), ctx.queue(), tex_data,
                                Some("grass3_lod0_texture"),
                            );
                            std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                ctx.device(), &tex_view, Some("grass3_lod0_bind"),
                            ))
                        });
                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(), &all_positions, &all_normals, &all_uvs, &all_indices,
                            texture_bind_group,
                        );
                        state.mesh_registry.insert("grass3_lod0".to_string(), gpu_mesh);
                        println!("[GRASS3] LOD0: {} verts, {} tris", all_positions.len(), all_indices.len() / 3);
                    }
                } else {
                    println!("[GRASS3] LOD0 model 'grass3' not found");
                }

                // Load LOD1 (low quality - 96% smaller)
                if let Some(model) = grass3_cache.load("grass3_lod1") {
                    let mut all_positions: Vec<[f32; 3]> = Vec::new();
                    let mut all_normals: Vec<[f32; 3]> = Vec::new();
                    let mut all_uvs: Vec<[f32; 2]> = Vec::new();
                    let mut all_indices: Vec<u32> = Vec::new();
                    let mut grass_texture: Option<&gltf_loader::LoadedTexture> = None;

                    for mesh in &model.meshes {
                        let base_idx = all_positions.len() as u32;
                        all_positions.extend_from_slice(&mesh.positions);
                        all_normals.extend_from_slice(&mesh.normals);
                        all_uvs.extend_from_slice(&mesh.uvs);
                        all_indices.extend(mesh.indices.iter().map(|i| i + base_idx));
                        if grass_texture.is_none() {
                            grass_texture = mesh.material.base_color_texture_data.as_ref();
                        }
                    }

                    if !all_positions.is_empty() {
                        let texture_bind_group = grass_texture.map(|tex_data| {
                            let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                ctx.device(), ctx.queue(), tex_data,
                                Some("grass3_lod1_texture"),
                            );
                            std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                ctx.device(), &tex_view, Some("grass3_lod1_bind"),
                            ))
                        });
                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(), &all_positions, &all_normals, &all_uvs, &all_indices,
                            texture_bind_group,
                        );
                        // Use LOD1 for both lod1 and lod2 (same low-poly model)
                        state.mesh_registry.insert("grass3_lod1".to_string(), gpu_mesh.clone());
                        state.mesh_registry.insert("grass3_lod2".to_string(), gpu_mesh);
                        println!("[GRASS3] LOD1/2: {} verts, {} tris (96% smaller)", all_positions.len(), all_indices.len() / 3);
                    }
                } else {
                    println!("[GRASS3] LOD1 model 'grass3_lod1' not found");
                }

                // Load groundcover models (clover, daisy) for forest floor scatter
                // OPTIMIZED: models_optimized/groundcover with LOD0/LOD1
                let mut groundcover_cache = gltf_loader::ModelCache::new("assets/models_optimized/groundcover");

                // Clover (10 tris, LOD0: 1.3MB, LOD1: 34KB)
                for (file_name, registry_name) in [
                    ("clover0lod0", "clover_lod0"),
                    ("clover0_lod1", "clover_lod1"),
                ] {
                    if let Some(model) = groundcover_cache.load(file_name) {
                        let mut positions: Vec<[f32; 3]> = Vec::new();
                        let mut normals: Vec<[f32; 3]> = Vec::new();
                        let mut uvs: Vec<[f32; 2]> = Vec::new();
                        let mut indices: Vec<u32> = Vec::new();
                        let mut texture_bind_group = None;

                        for mesh in &model.meshes {
                            let base_idx = positions.len() as u32;
                            positions.extend_from_slice(&mesh.positions);
                            normals.extend_from_slice(&mesh.normals);
                            uvs.extend_from_slice(&mesh.uvs);
                            indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                            if texture_bind_group.is_none() {
                                if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                    let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                        ctx.device(), ctx.queue(), tex_data,
                                        Some(&format!("{}_texture", registry_name)),
                                    );
                                    let shadow_map_gc = shadow_map_mutex.safe_lock();
                                    let helper = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_gc);
                                    texture_bind_group = Some(Arc::new(
                                        helper.create_texture_bind_group(ctx.device(), &tex_view, Some(&format!("{}_bind", registry_name)))
                                    ));
                                    drop(shadow_map_gc);
                                }
                            }
                        }

                        let gpu_mesh = TreePipeline::create_mesh(ctx.device(), &positions, &normals, &uvs, &indices, texture_bind_group);
                        println!("[GROUNDCOVER] {}: {} verts, {} tris", registry_name, positions.len(), indices.len() / 3);
                        state.mesh_registry.insert(registry_name.to_string(), gpu_mesh);
                    }
                }

                // Sparse clover (8 tris, LOD0: 653KB, LOD1: 15KB)
                for (file_name, registry_name) in [
                    ("sparseclover_lod0", "sparseclover_lod0"),
                    ("sparseclover_lod1", "sparseclover_lod1"),
                ] {
                    if let Some(model) = groundcover_cache.load(file_name) {
                        let mut positions: Vec<[f32; 3]> = Vec::new();
                        let mut normals: Vec<[f32; 3]> = Vec::new();
                        let mut uvs: Vec<[f32; 2]> = Vec::new();
                        let mut indices: Vec<u32> = Vec::new();
                        let mut texture_bind_group = None;

                        for mesh in &model.meshes {
                            let base_idx = positions.len() as u32;
                            positions.extend_from_slice(&mesh.positions);
                            normals.extend_from_slice(&mesh.normals);
                            uvs.extend_from_slice(&mesh.uvs);
                            indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                            if texture_bind_group.is_none() {
                                if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                    let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                        ctx.device(), ctx.queue(), tex_data,
                                        Some(&format!("{}_texture", registry_name)),
                                    );
                                    let shadow_map_gc = shadow_map_mutex.safe_lock();
                                    let helper = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_gc);
                                    texture_bind_group = Some(Arc::new(
                                        helper.create_texture_bind_group(ctx.device(), &tex_view, Some(&format!("{}_bind", registry_name)))
                                    ));
                                    drop(shadow_map_gc);
                                }
                            }
                        }

                        let gpu_mesh = TreePipeline::create_mesh(ctx.device(), &positions, &normals, &uvs, &indices, texture_bind_group);
                        println!("[GROUNDCOVER] {}: {} verts, {} tris", registry_name, positions.len(), indices.len() / 3);
                        state.mesh_registry.insert(registry_name.to_string(), gpu_mesh);
                    }
                }

                // Sparse daisy (28 tris, LOD0: 766KB, LOD1: 19KB)
                for (file_name, registry_name) in [
                    ("sparsedaisy_lod0", "daisy_lod0"),
                    ("sparsedaisy_lod1", "daisy_lod1"),
                ] {
                    if let Some(model) = groundcover_cache.load(file_name) {
                        let mut positions: Vec<[f32; 3]> = Vec::new();
                        let mut normals: Vec<[f32; 3]> = Vec::new();
                        let mut uvs: Vec<[f32; 2]> = Vec::new();
                        let mut indices: Vec<u32> = Vec::new();
                        let mut texture_bind_group = None;

                        for mesh in &model.meshes {
                            let base_idx = positions.len() as u32;
                            positions.extend_from_slice(&mesh.positions);
                            normals.extend_from_slice(&mesh.normals);
                            uvs.extend_from_slice(&mesh.uvs);
                            indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                            if texture_bind_group.is_none() {
                                if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                    let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                        ctx.device(), ctx.queue(), tex_data,
                                        Some(&format!("{}_texture", registry_name)),
                                    );
                                    let shadow_map_gc = shadow_map_mutex.safe_lock();
                                    let helper = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_gc);
                                    texture_bind_group = Some(Arc::new(
                                        helper.create_texture_bind_group(ctx.device(), &tex_view, Some(&format!("{}_bind", registry_name)))
                                    ));
                                    drop(shadow_map_gc);
                                }
                            }
                        }

                        let gpu_mesh = TreePipeline::create_mesh(ctx.device(), &positions, &normals, &uvs, &indices, texture_bind_group);
                        println!("[GROUNDCOVER] {}: {} verts, {} tris", registry_name, positions.len(), indices.len() / 3);
                        state.mesh_registry.insert(registry_name.to_string(), gpu_mesh);
                    }
                }

                // Load plants (chamomile, genericbush, hedge)
                let mut plants_cache = gltf_loader::ModelCache::new("assets/models_optimized/plants");
                let plant_models = [
                    // Chamomile (360 tris, LOD0: 2.8MB, LOD1: 107KB)
                    ("chamomile_lod0", "chamomile_lod0"),
                    ("chamomile_lod1", "chamomile_lod1"),
                    // Generic bush 0 (512 tris, LOD0: 1.3MB, LOD1: 55KB)
                    ("genericbush0_lod0", "genericbush0_lod0"),
                    ("genericbush0_lod1", "genericbush0_lod1"),
                    // Generic bush 1 (216 tris, LOD0: 2.8MB, LOD1: 71KB)
                    ("genericbush1_lod0", "genericbush1_lod0"),
                    ("genericbush1_lod1", "genericbush1_lod1"),
                    // Hedge (976 tris, LOD0: 1.8MB, LOD1: 97KB)
                    ("hedge0_lod0", "hedge0_lod0"),
                    ("hedge0_lod1", "hedge0_lod1"),
                ];
                for (file_name, registry_name) in plant_models {
                    if let Some(model) = plants_cache.load(file_name) {
                        let mut positions: Vec<[f32; 3]> = Vec::new();
                        let mut normals: Vec<[f32; 3]> = Vec::new();
                        let mut uvs: Vec<[f32; 2]> = Vec::new();
                        let mut indices: Vec<u32> = Vec::new();
                        let mut texture_bind_group = None;

                        for mesh in &model.meshes {
                            let base_idx = positions.len() as u32;
                            positions.extend_from_slice(&mesh.positions);
                            normals.extend_from_slice(&mesh.normals);
                            uvs.extend_from_slice(&mesh.uvs);
                            indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                            if texture_bind_group.is_none() {
                                if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                    let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                        ctx.device(), ctx.queue(), tex_data,
                                        Some(&format!("{}_texture", registry_name)),
                                    );
                                    let shadow_map_gc = shadow_map_mutex.safe_lock();
                                    let helper = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_gc);
                                    texture_bind_group = Some(Arc::new(
                                        helper.create_texture_bind_group(ctx.device(), &tex_view, Some(&format!("{}_bind", registry_name)))
                                    ));
                                    drop(shadow_map_gc);
                                }
                            }
                        }

                        let gpu_mesh = TreePipeline::create_mesh(ctx.device(), &positions, &normals, &uvs, &indices, texture_bind_group);
                        println!("[PLANTS] {}: {} verts, {} tris", registry_name, positions.len(), indices.len() / 3);
                        state.mesh_registry.insert(registry_name.to_string(), gpu_mesh);
                    }
                }

                // Load spikegrass groundcover (40 tris, LOD0: 1.8MB, LOD1: 39KB, Z offset -0.3)
                for (file_name, registry_name) in [
                    ("spikegrass0offset_lod0", "spikegrass_lod0"),
                    ("spikegrass0offset_lod1", "spikegrass_lod1"),
                ] {
                    if let Some(model) = groundcover_cache.load(file_name) {
                        let mut positions: Vec<[f32; 3]> = Vec::new();
                        let mut normals: Vec<[f32; 3]> = Vec::new();
                        let mut uvs: Vec<[f32; 2]> = Vec::new();
                        let mut indices: Vec<u32> = Vec::new();
                        let mut texture_bind_group = None;

                        for mesh in &model.meshes {
                            let base_idx = positions.len() as u32;
                            positions.extend_from_slice(&mesh.positions);
                            normals.extend_from_slice(&mesh.normals);
                            uvs.extend_from_slice(&mesh.uvs);
                            indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                            if texture_bind_group.is_none() {
                                if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                    let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                        ctx.device(), ctx.queue(), tex_data,
                                        Some(&format!("{}_texture", registry_name)),
                                    );
                                    let shadow_map_gc = shadow_map_mutex.safe_lock();
                                    let helper = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_gc);
                                    texture_bind_group = Some(Arc::new(
                                        helper.create_texture_bind_group(ctx.device(), &tex_view, Some(&format!("{}_bind", registry_name)))
                                    ));
                                    drop(shadow_map_gc);
                                }
                            }
                        }

                        let gpu_mesh = TreePipeline::create_mesh(ctx.device(), &positions, &normals, &uvs, &indices, texture_bind_group);
                        println!("[GROUNDCOVER] {}: {} verts, {} tris", registry_name, positions.len(), indices.len() / 3);
                        state.mesh_registry.insert(registry_name.to_string(), gpu_mesh);
                    }
                }

                // 2. Rocks - Procedural types (pebble, small, medium, flat, mossy)
                let rock_types: Vec<(RockRecipe, &str)> = vec![
                    (RockRecipe::pebble(), "rock_pebble"),
                    (RockRecipe::small_rock(), "rock_small"),
                    (RockRecipe::medium_rock(), "rock_medium"),
                    (RockRecipe::flat_rock(), "rock_flat"),
                    (RockRecipe::mossy_rock(), "rock_mossy"),
                ];

                for (recipe, name) in rock_types {
                    let mesh = generate_rock(&recipe);
                    let positions: Vec<[f32; 3]> = mesh.vertices.iter().map(|v| v.position).collect();
                    let normals: Vec<[f32; 3]> = mesh.vertices.iter().map(|v| v.normal).collect();
                    let uvs: Vec<[f32; 2]> = mesh.vertices.iter().map(|v| v.uv).collect();

                    let gpu_mesh = TreePipeline::create_mesh(
                        ctx.device(),
                        &positions,
                        &normals,
                        &uvs,
                        &mesh.indices,
                        None,
                    );
                    state.mesh_registry.insert(name.to_string(), gpu_mesh);
                    println!("[ASSET] Registered rock: {} ({} verts)", name, positions.len());
                }

                // 3. Boulder LODs - loaded from GLB models
                // OPTIMIZED: 16.3MB -> 6.2MB (5.5MB LOD0, 560KB LOD1, 150KB LOD2)
                let mut boulder_cache = gltf_loader::ModelCache::new("assets/models_optimized/rocks");
                let shadow_map_for_boulders = shadow_map_mutex.safe_lock();
                let boulder_texture_helper = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_for_boulders);
                drop(shadow_map_for_boulders);

                for lod in 0..=2 {
                    let name = format!("boulder_lod{}", lod);
                    if let Some(model) = boulder_cache.load(&name) {
                        let mut positions: Vec<[f32; 3]> = Vec::new();
                        let mut normals: Vec<[f32; 3]> = Vec::new();
                        let mut uvs: Vec<[f32; 2]> = Vec::new();
                        let mut indices: Vec<u32> = Vec::new();
                        let mut texture_bind_group = None;

                        for mesh in &model.meshes {
                            let base_idx = positions.len() as u32;
                            positions.extend_from_slice(&mesh.positions);
                            normals.extend_from_slice(&mesh.normals);
                            uvs.extend_from_slice(&mesh.uvs);
                            indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                            // Get texture from first mesh that has one
                            if texture_bind_group.is_none() {
                                if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                    let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                        ctx.device(), ctx.queue(), tex_data,
                                        Some(&format!("{}_texture", name)),
                                    );
                                    texture_bind_group = Some(Arc::new(
                                        boulder_texture_helper.create_texture_bind_group(
                                            ctx.device(), &tex_view, Some(&format!("{}_bind", name)),
                                        )
                                    ));
                                }
                            }
                        }

                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(),
                            &positions,
                            &normals,
                            &uvs,
                            &indices,
                            texture_bind_group,
                        );
                        println!("[ASSET] Registered boulder: {} ({} verts) from GLB", name, positions.len());
                        state.mesh_registry.insert(name, gpu_mesh);
                    } else {
                        println!("[ASSET] WARNING: boulder_lod{}.glb not found, using procedural fallback", lod);
                        let mesh = generate_rock(&RockRecipe::boulder());
                        let positions: Vec<[f32; 3]> = mesh.vertices.iter().map(|v| v.position).collect();
                        let normals: Vec<[f32; 3]> = mesh.vertices.iter().map(|v| v.normal).collect();
                        let uvs: Vec<[f32; 2]> = mesh.vertices.iter().map(|v| v.uv).collect();
                        let gpu_mesh = TreePipeline::create_mesh(ctx.device(), &positions, &normals, &uvs, &mesh.indices, None);
                        state.mesh_registry.insert(format!("boulder_lod{}", lod), gpu_mesh);
                    }
                }
                // Keep rock_boulder as alias to boulder_lod0 for backwards compat
                if let Some(lod0) = state.mesh_registry.get("boulder_lod0").cloned() {
                    state.mesh_registry.insert("rock_boulder".to_string(), lod0);
                    println!("[ASSET] Aliased rock_boulder -> boulder_lod0");
                }

                // 4. Dead log LODs - fallen logs for forest floor and beach driftwood
                // OPTIMIZED: 36MB -> 3.8MB (2.9MB LOD0, 724KB LOD1, 192KB LOD2)
                let mut dead_log_cache = gltf_loader::ModelCache::new("assets/models_optimized/trees");
                for lod in 0..=2 {
                    let name = if lod == 0 {
                        "dead_log_0".to_string()
                    } else {
                        format!("dead_log_0_lod{}", lod)
                    };

                    if let Some(model) = dead_log_cache.load(&name) {
                        let mut positions: Vec<[f32; 3]> = Vec::new();
                        let mut normals: Vec<[f32; 3]> = Vec::new();
                        let mut uvs: Vec<[f32; 2]> = Vec::new();
                        let mut indices: Vec<u32> = Vec::new();
                        let mut texture_bind_group = None;

                        for mesh in &model.meshes {
                            let base_idx = positions.len() as u32;
                            positions.extend_from_slice(&mesh.positions);
                            normals.extend_from_slice(&mesh.normals);
                            uvs.extend_from_slice(&mesh.uvs);
                            indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                            if texture_bind_group.is_none() {
                                if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                    let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                        ctx.device(), ctx.queue(), tex_data,
                                        Some(&format!("{}_texture", name)),
                                    );
                                    texture_bind_group = Some(Arc::new(
                                        boulder_texture_helper.create_texture_bind_group(
                                            ctx.device(), &tex_view, Some(&format!("{}_bind", name)),
                                        )
                                    ));
                                }
                            }
                        }

                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(), &positions, &normals, &uvs, &indices, texture_bind_group,
                        );
                        println!("[ASSET] Registered dead_log: {} ({} verts, {} tris) from GLB",
                            name, positions.len(), indices.len() / 3);
                        state.mesh_registry.insert(name, gpu_mesh);
                    } else {
                        println!("[ASSET] Dead log {} not found - will skip", name);
                    }
                }

                // 5. Storage Containers - Chests (all LODs from optimized folder)
                // OPTIMIZED: Using models_optimized/containers for reduced texture/poly models
                let mut container_cache = gltf_loader::ModelCache::new("assets/models_optimized/containers");
                let chest_variants = ["chest_closed", "chest_open"];
                let chest_lods = [0, 1, 2]; // All LODs now available

                for variant in &chest_variants {
                    for &lod in &chest_lods {
                        let name = format!("{}_lod{}", variant, lod);
                        if let Some(model) = container_cache.load(&name) {
                            let mut positions: Vec<[f32; 3]> = Vec::new();
                            let mut normals: Vec<[f32; 3]> = Vec::new();
                            let mut uvs: Vec<[f32; 2]> = Vec::new();
                            let mut indices: Vec<u32> = Vec::new();
                            let mut texture_bind_group = None;

                            for mesh in &model.meshes {
                                let base_idx = positions.len() as u32;
                                positions.extend_from_slice(&mesh.positions);
                                normals.extend_from_slice(&mesh.normals);
                                uvs.extend_from_slice(&mesh.uvs);
                                indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                                if texture_bind_group.is_none() {
                                    if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                        let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                            ctx.device(), ctx.queue(), tex_data,
                                            Some(&format!("{}_texture", name)),
                                        );
                                        texture_bind_group = Some(Arc::new(
                                            boulder_texture_helper.create_texture_bind_group(
                                                ctx.device(), &tex_view, Some(&format!("{}_bind", name)),
                                            )
                                        ));
                                    }
                                }
                            }

                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &positions, &normals, &uvs, &indices, texture_bind_group,
                            );
                            println!("[CONTAINER] Registered '{}': {} verts, {} tris",
                                name, positions.len(), indices.len() / 3);
                            state.mesh_registry.insert(name, gpu_mesh);
                        } else {
                            println!("[CONTAINER] Model '{}' not found in assets/models_optimized/containers/", name);
                        }
                    }
                }

                // 6. Weapon Models - For dropped items in world
                let mut weapon_cache = gltf_loader::ModelCache::new("assets/models/weapons");
                let weapon_models = ["dagger_lod0", "flintlock_lod0", "hatchet_lod0"];

                for weapon_name in &weapon_models {
                    if let Some(model) = weapon_cache.load(weapon_name) {
                        let mut positions: Vec<[f32; 3]> = Vec::new();
                        let mut normals: Vec<[f32; 3]> = Vec::new();
                        let mut uvs: Vec<[f32; 2]> = Vec::new();
                        let mut indices: Vec<u32> = Vec::new();
                        let mut texture_bind_group = None;

                        for mesh in &model.meshes {
                            let base_idx = positions.len() as u32;
                            positions.extend_from_slice(&mesh.positions);
                            normals.extend_from_slice(&mesh.normals);
                            uvs.extend_from_slice(&mesh.uvs);
                            indices.extend(mesh.indices.iter().map(|i| i + base_idx));

                            if texture_bind_group.is_none() {
                                if let Some(tex_data) = &mesh.material.base_color_texture_data {
                                    let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                        ctx.device(), ctx.queue(), tex_data,
                                        Some(&format!("{}_texture", weapon_name)),
                                    );
                                    texture_bind_group = Some(Arc::new(
                                        boulder_texture_helper.create_texture_bind_group(
                                            ctx.device(), &tex_view, Some(&format!("{}_bind", weapon_name)),
                                        )
                                    ));
                                }
                            }
                        }

                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(), &positions, &normals, &uvs, &indices, texture_bind_group,
                        );
                        println!("[WEAPON] Registered '{}': {} verts, {} tris",
                            weapon_name, positions.len(), indices.len() / 3);
                        state.mesh_registry.insert(weapon_name.to_string(), gpu_mesh);
                    } else {
                        println!("[WEAPON] Model '{}' not found in assets/models/weapons/", weapon_name);
                    }
                }

                println!("[GPU] Assets registered: {:?}", state.mesh_registry.keys());
            }

            // Load terrain textures if not already loaded
            if state.terrain_textures.is_none() {
                println!("[GPU] Loading terrain textures...");
                match TerrainTextures::load(ctx.device(), ctx.queue(), "assets") {
                    Ok(textures) => {
                        state.terrain_textures = Some(Arc::new(textures));
                        println!("[GPU] Terrain textures loaded successfully");
                    }
                    Err(e) => {
                        println!("[GPU] WARNING: Failed to load terrain textures: {}", e);
                        println!("[GPU] Using fallback placeholder texture");
                        let fallback = TerrainTextures::create_fallback(ctx.device(), ctx.queue());
                        state.terrain_textures = Some(Arc::new(fallback));
                    }
                }
            }

            if state.building_registry.is_empty() {
                println!("[GPU] Initializing Building Registry...");
                
                // 1. Colonial House
                {
                    let recipe = BuildingRecipe::colonial_house();
                    let mesh = generate_building(&recipe);
                    
                    // Convert to BuildingVertex
                    let vertices: Vec<BuildingVertex> = mesh.vertices.iter().map(|v| BuildingVertex {
                        position: v.position,
                        normal: v.normal,
                        uv: v.uv,
                        color: v.color,
                    }).collect();

                    let gpu_mesh = BuildingPipeline::create_mesh(
                        ctx.device(),
                        &vertices,
                        &mesh.indices,
                    );
                    state.building_registry.insert("building_colonial".to_string(), gpu_mesh);
                }

                // 2. Small Shack
                {
                    let recipe = BuildingRecipe::small_shack();
                    let mesh = generate_building(&recipe);
                    
                    let vertices: Vec<BuildingVertex> = mesh.vertices.iter().map(|v| BuildingVertex {
                        position: v.position,
                        normal: v.normal,
                        uv: v.uv,
                        color: v.color,
                    }).collect();

                    let gpu_mesh = BuildingPipeline::create_mesh(
                        ctx.device(),
                        &vertices,
                        &mesh.indices,
                    );
                    state.building_registry.insert("building_cabin".to_string(), gpu_mesh); // Matches "building_cabin" from buildings.rs
                }
                
                println!("[GPU] Buildings registered: {:?}", state.building_registry.keys());
            }
        }

        // Initialize egui renderer
        static EGUI_RENDERER: OnceLock<Mutex<egui_wgpu::Renderer>> = OnceLock::new();
        let egui_renderer_mutex = EGUI_RENDERER.get_or_init(|| {
            Mutex::new(egui_wgpu::Renderer::new(
                ctx.device(),
                ctx.surface_format(),
                None,
                1,
            ))
        });

        // Chunk Manager (Stores all loaded chunks and manages streaming)
        // Load/unload radii are scaled dynamically based on render_distance
        static CHUNK_MANAGER: OnceLock<Mutex<ChunkManager>> = OnceLock::new();
        let chunk_manager = CHUNK_MANAGER.get_or_init(|| {
            let mut manager = ChunkManager::new(256.0, 2, 4);
            // Initialize radius based on default render_distance (400)
            manager.update_radius_for_render_distance(400.0);
            Mutex::new(manager)
        });

        // Get shadow pipeline reference (SHADOW_SYSTEM was initialized at start of render callback)
        let shadow_pipeline_mutex = &SHADOW_SYSTEM.get().expect("SHADOW_SYSTEM should be initialized").1;

        // Grass System (requires shadow map)
        static GRASS_PIPELINE: OnceLock<Mutex<GrassPipeline>> = OnceLock::new();
        let _grass_pipeline_mutex = GRASS_PIPELINE.get_or_init(|| {
            let shadow_map = shadow_map_mutex.safe_lock();
            let grass_pipeline = GrassPipeline::new(ctx.device(), ctx.surface_format(), &shadow_map);
            drop(shadow_map);  // Release lock
            Mutex::new(grass_pipeline)
        });

        // Tree System
        static TREE_PIPELINE: OnceLock<Mutex<TreePipeline>> = OnceLock::new();
        let _tree_pipeline_mutex = TREE_PIPELINE.get_or_init(|| {
            let shadow_map = shadow_map_mutex.safe_lock();
            let tree_pipeline = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map);
            drop(shadow_map);
            Mutex::new(tree_pipeline)
        });

        // Sun Billboard
        static SUN_PIPELINE: OnceLock<Mutex<SunPipeline>> = OnceLock::new();
        let sun_pipeline_mutex = SUN_PIPELINE.get_or_init(|| {
            Mutex::new(SunPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // Sky Pipeline
        static SKY_PIPELINE: OnceLock<Mutex<SkyPipeline>> = OnceLock::new();
        let sky_pipeline_mutex = SKY_PIPELINE.get_or_init(|| {
            Mutex::new(SkyPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // Viewmodel Pipeline (First-person arms and weapon)
        static VIEWMODEL_PIPELINE: OnceLock<Mutex<ViewModelPipeline>> = OnceLock::new();
        let viewmodel_pipeline_mutex = VIEWMODEL_PIPELINE.get_or_init(|| {
            Mutex::new(ViewModelPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // Weapon Viewmodel Pipeline (GLB weapon models)
        static WEAPON_VIEWMODEL_PIPELINE: OnceLock<Mutex<WeaponViewModelPipeline>> = OnceLock::new();
        let weapon_viewmodel_mutex = WEAPON_VIEWMODEL_PIPELINE.get_or_init(|| {
            Mutex::new(WeaponViewModelPipeline::new(ctx.device(), ctx.queue(), ctx.surface_format()))
        });

        // Water System
        static WATER_SYSTEM: OnceLock<Mutex<WaterSystem>> = OnceLock::new();
        let water_system_mutex = WATER_SYSTEM.get_or_init(|| {
            Mutex::new(WaterSystem::new(ctx.device(), ctx.surface_format()))
        });

        // Pond Water System (inland lakes, ponds, wetlands)
        static POND_WATER_SYSTEM: OnceLock<Mutex<PondWaterSystem>> = OnceLock::new();
        let pond_water_system_mutex = POND_WATER_SYSTEM.get_or_init(|| {
            Mutex::new(PondWaterSystem::new(ctx.device(), ctx.surface_format(), 12345))
        });

        // Light Shaft Pipeline (God Rays Post-Process)
        static LIGHT_SHAFT_PIPELINE: OnceLock<Mutex<LightShaftPipeline>> = OnceLock::new();
        let light_shaft_pipeline_mutex = LIGHT_SHAFT_PIPELINE.get_or_init(|| {
            Mutex::new(LightShaftPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // Animal Orb Pipeline (Visual representation of animals - fallback for species without models)
        static ANIMAL_ORB_PIPELINE: OnceLock<Mutex<AnimalOrbPipeline>> = OnceLock::new();
        let animal_orb_pipeline_mutex = ANIMAL_ORB_PIPELINE.get_or_init(|| {
            Mutex::new(AnimalOrbPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // Animal Model Pipeline (3D models for animals)
        static ANIMAL_MODEL_PIPELINE: OnceLock<Mutex<AnimalModelPipeline>> = OnceLock::new();
        static ANIMAL_MODEL_SHADOW_BOUND: OnceLock<std::sync::atomic::AtomicBool> = OnceLock::new();
        let animal_model_pipeline_mutex = ANIMAL_MODEL_PIPELINE.get_or_init(|| {
            Mutex::new(AnimalModelPipeline::new_with_queue(ctx.device(), Some(ctx.queue()), ctx.surface_format()))
        });
        let shadow_bound_flag = ANIMAL_MODEL_SHADOW_BOUND.get_or_init(|| std::sync::atomic::AtomicBool::new(false));

        // Bind shadow map to animal model pipeline once
        if !shadow_bound_flag.load(std::sync::atomic::Ordering::Relaxed) {
            let shadow_map = shadow_map_mutex.safe_lock();
            let mut model_pipeline = animal_model_pipeline_mutex.safe_lock();
            model_pipeline.bind_shadow_map(ctx.device(), &shadow_map.view, &shadow_map.sampler);
            drop(model_pipeline);
            drop(shadow_map);
            shadow_bound_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Rain Particle Pipeline
        static RAIN_PIPELINE: OnceLock<Mutex<RainPipeline>> = OnceLock::new();
        let rain_pipeline_mutex = RAIN_PIPELINE.get_or_init(|| {
            Mutex::new(RainPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // Ember Particle Pipeline (campfire embers)
        static EMBER_PIPELINE: OnceLock<Mutex<EmberPipeline>> = OnceLock::new();
        let ember_pipeline_mutex = EMBER_PIPELINE.get_or_init(|| {
            Mutex::new(EmberPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // BioOrb Pipeline (bioluminescent cave orbs)
        static BIO_ORB_PIPELINE: OnceLock<Mutex<BioOrbPipeline>> = OnceLock::new();
        let bio_orb_pipeline_mutex = BIO_ORB_PIPELINE.get_or_init(|| {
            Mutex::new(BioOrbPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // Container Pipelines created fresh each frame (same pattern as rocks/trees)

        // Animal Model Cache (loads GLTF models)
        static ANIMAL_MODEL_CACHE: OnceLock<Mutex<gltf_loader::ModelCache>> = OnceLock::new();
        let animal_model_cache_mutex = ANIMAL_MODEL_CACHE.get_or_init(|| {
            let mut cache = gltf_loader::ModelCache::new("assets/models/animals");
            // Preload available models
            cache.preload(&["Wolf", "Deer", "Stag", "Horse", "Donkey", "Fox", "Husky", "ring_necked_pheasant"]);
            Mutex::new(cache)
        });

        // Offscreen Render Target (for post-process effects)
        static OFFSCREEN_TARGET: OnceLock<Mutex<Option<OffscreenTarget>>> = OnceLock::new();
        let offscreen_target_mutex = OFFSCREEN_TARGET.get_or_init(|| Mutex::new(None));

        // Check if offscreen target needs to be created/resized
        {
            let mut offscreen_opt = offscreen_target_mutex.safe_lock();
            let needs_create = match &*offscreen_opt {
                None => true,
                Some(target) => target.needs_resize(ctx.config().width, ctx.config().height),
            };
            if needs_create {
                *offscreen_opt = Some(OffscreenTarget::new(
                    ctx.device(),
                    ctx.surface_format(),
                    ctx.config().width,
                    ctx.config().height,
                ));
                // Invalidate cached light shaft bind group since scene texture changed
                light_shaft_pipeline_mutex.safe_lock().invalidate_bind_group();
            }
        }

        let mut state = render_state.safe_lock();

        // Dynamic weapon viewmodel loading based on active hotbar slot
        {
            let current_hotbar_weapon = state.player_economy.inventory.get_slot(state.active_hotbar_slot)
                .filter(|item| item.item_type == economy::ItemType::Weapon)
                .map(|item| item.template_id.clone());

            if current_hotbar_weapon != state.loaded_weapon_viewmodel {
                let (model_path, position, rotation, scale) = match current_hotbar_weapon.as_deref() {
                    // Standard right-hand weapon position for all weapons
                    // Position: (0.40, -0.30, -0.55) - right side, below center, in front
                    // Scale: 1.2 for good visibility
                    Some("flintlock_pistol") => (
                        "assets/models/weapons/flintlock_lod0.glb",
                        Vec3::new(0.40, -0.30, -0.55), // Right hand position
                        Vec3::new(0.0, -1.57, 0.0),    // Point straight forward (-90 degrees)
                        1.2f32,                        // Larger for visibility
                    ),
                    Some("dagger") => (
                        "assets/models/weapons/dagger_lod0.glb",
                        Vec3::new(0.40, -0.30, -0.55), // Right hand position (same as others)
                        Vec3::new(0.0, 1.57, 0.0),     // Rotated 180° - dagger model faces opposite direction
                        0.4f32,                        // Dagger model is larger, reduced scale
                    ),
                    Some("hatchet") => (
                        "assets/models/weapons/hatchet_lod0.glb",
                        Vec3::new(0.40, -0.30, -0.55), // Right hand position
                        Vec3::new(0.0, -1.57, 0.0),    // Point straight forward
                        1.2f32,                        // Larger for visibility
                    ),
                    _ => {
                        // No weapon or unknown weapon - clear viewmodel
                        state.loaded_weapon_viewmodel = None;
                        ("", Vec3::ZERO, Vec3::ZERO, 0.0f32)
                    }
                };

                if !model_path.is_empty() {
                    if let Ok(model) = gltf_loader::load_gltf(model_path) {
                        let mut all_vertices: Vec<WeaponVertex> = Vec::new();
                        let mut all_indices: Vec<u32> = Vec::new();
                        let mut texture_data: Option<(Vec<u8>, u32, u32)> = None;

                        for mesh in &model.meshes {
                            let base_vertex = all_vertices.len() as u32;

                            for (i, pos) in mesh.positions.iter().enumerate() {
                                all_vertices.push(WeaponVertex::new(
                                    *pos,
                                    mesh.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                                    mesh.uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                                ));
                            }

                            for idx in &mesh.indices {
                                all_indices.push(base_vertex + idx);
                            }

                            if texture_data.is_none() {
                                if let Some(tex) = mesh.material.get_or_create_texture() {
                                    texture_data = Some((tex.data.clone(), tex.width, tex.height));
                                }
                            }
                        }

                        if !all_vertices.is_empty() {
                            let mut weapon_pipeline = weapon_viewmodel_mutex.safe_lock();

                            weapon_pipeline.position_offset = position;
                            weapon_pipeline.rotation_offset = rotation;
                            weapon_pipeline.scale = scale;

                            if let Some((data, w, h)) = texture_data {
                                weapon_pipeline.upload_weapon_mesh(
                                    ctx.device(),
                                    ctx.queue(),
                                    &all_vertices,
                                    &all_indices,
                                    Some((&data, w, h)),
                                );
                            } else {
                                weapon_pipeline.upload_weapon_mesh(
                                    ctx.device(),
                                    ctx.queue(),
                                    &all_vertices,
                                    &all_indices,
                                    None,
                                );
                            }

                            let weapon_name = current_hotbar_weapon.as_deref().unwrap_or("unknown");
                            println!("[Weapon] Loaded {}: {} vertices, {} indices",
                                weapon_name, all_vertices.len(), all_indices.len());
                            state.loaded_weapon_viewmodel = current_hotbar_weapon.clone();
                        }
                    } else {
                        // Model file not found - keep current or clear
                        if current_hotbar_weapon.is_some() {
                            log::debug!("[Weapon] Model not found: {}", model_path);
                        }
                    }
                } else {
                    // No weapon in slot - clear the viewmodel mesh
                    let mut weapon_pipeline = weapon_viewmodel_mutex.safe_lock();
                    weapon_pipeline.clear_weapon();
                    log::debug!("[Weapon] Cleared viewmodel - empty hand");
                }
            }
        }

        // Calculate FPS
        let now = Instant::now();
        let delta = now.duration_since(state.last_frame_time).as_secs_f32();
        state.last_frame_time = now;
        if delta > 0.0 {
            // Simple smoothing
            state.fps = state.fps * 0.9 + (1.0 / delta) * 0.1;
        }

        // Update Time of Day - cycles automatically, can be adjusted with T/Y keys
        if state.game_state == GameState::Playing {
            // Auto-advance time (1 real second = 0.5 game minutes = 1/120 hour)
            state.time_of_day += delta * (1.0 / 120.0);
            if state.time_of_day >= 24.0 {
                state.time_of_day -= 24.0;
            }
            if state.time_of_day >= 24.0 {
                state.time_of_day -= 24.0;
            }
            // Time is no longer clamped to allow night cycle
            
            // Update Weather
            state.weather.update(delta);

            // Update Network (multiplayer sync)
            let game_time = state.game_progression.game_time;
            state.network.update(delta, game_time);

            // Send player position to network
            state.network.send_position(
                state.player.position,
                state.player.velocity,
                state.player.yaw,
                state.player.pitch,
                state.player.on_ground,
            );

            // Update Campfires (flickering animation)
            state.campfire_manager.update(delta);

            // Update dropped items (despawn timer)
            state.dropped_items.update(delta);

            // Update Systems Manager (encyclopedia, flora, ecology pipelines)
            // This coordinates data flow between weather, ecology, observations
            state.update_systems(delta);

            // Update Audio System (responds to weather, time, game state)
            let time_normalized = state.time_of_day / 24.0; // Normalize to 0.0-1.0
            let current_weather = state.weather.current_weather;
            state.audio_system.update(delta, current_weather, time_normalized);

            // Load coastal sounds and footsteps once when playing starts
            if !state.coastal_sounds_loaded {
                state.audio_system.load_coastal_sounds();
                state.audio_system.load_footsteps();
                state.coastal_sounds_loaded = true;
            }

            // Update coastal ambience based on player distance to shoreline
            {
                let player_x = state.player.position.x;
                let player_z = state.player.position.z;
                let seed = state.seed;

                // Calculate distance to nearest shoreline
                let dist_to_surf = croatoan_wfc::distance_to_shoreline(player_x, player_z, seed);

                // Get biome_t for gradient-based forest wind and overlap zones
                let biome_t = croatoan_wfc::get_biome_t(player_x, player_z, seed);

                state.audio_system.update_coastal(delta, dist_to_surf, biome_t);
            }

            // Sync audio music state with game state
            let audio_music_state = match state.game_state {
                GameState::Menu => MusicState::MainMenu,
                GameState::Loading => MusicState::MainMenu,
                GameState::Playing => MusicState::Exploration,
                GameState::Paused => MusicState::Peaceful,
            };
            state.audio_system.set_music_state(audio_music_state);

            // Update Swing Animation & Combat
            if state.swing_animation.is_swinging {
                state.swing_animation.swing_progress += delta / state.swing_animation.swing_duration;

                // Process hit at midpoint of swing (when weapon makes contact)
                if state.swing_animation.swing_progress >= 0.4 && !state.swing_animation.hit_processed {
                    state.swing_animation.hit_processed = true;
                    state.combat_kill_time += delta;

                    // Get equipped weapon from active hotbar slot
                    let hotbar_weapon = state.player_economy.inventory.get_slot(state.active_hotbar_slot)
                        .filter(|item| item.item_type == economy::ItemType::Weapon)
                        .map(|item| item.template_id.as_str());

                    // Determine attack type based on equipped weapon
                    let (target_id, weapon_name, base_damage) = match hotbar_weapon {
                        // Dagger: melee-only, fast, moderate damage
                        Some("dagger") => {
                            let melee_range = 3.5; // Slightly longer than fists
                            if let Some((id, _dist)) = animals::combat::find_closest_animal(
                                &state.animal_manager,
                                state.player.position,
                                melee_range,
                            ) {
                                (Some(id), "dagger", 30.0)
                            } else {
                                (None, "", 0.0)
                            }
                        }
                        // Flintlock: ranged-only
                        Some("flintlock_pistol") => {
                            let look_dir = state.camera.forward();
                            let ranged_distance = 50.0;
                            let aim_cone = 0.05;
                            if let Some((id, _dist, _species, _behavior)) = state.animal_manager.get_focused_animal(
                                state.player.position,
                                look_dir,
                                ranged_distance,
                                aim_cone,
                            ) {
                                (Some(id), "flintlock_pistol", 40.0)
                            } else {
                                (None, "", 0.0)
                            }
                        }
                        // Hatchet: melee, high damage, good range
                        Some("hatchet") => {
                            let melee_range = 4.0; // Longer reach than dagger
                            if let Some((id, _dist)) = animals::combat::find_closest_animal(
                                &state.animal_manager,
                                state.player.position,
                                melee_range,
                            ) {
                                (Some(id), "hatchet", 45.0) // High damage
                            } else {
                                (None, "", 0.0)
                            }
                        }
                        // Default/unarmed: basic melee
                        _ => {
                            let melee_range = 3.0;
                            if let Some((id, _dist)) = animals::combat::find_closest_animal(
                                &state.animal_manager,
                                state.player.position,
                                melee_range,
                            ) {
                                (Some(id), "hunter_knife", 25.0)
                            } else {
                                (None, "", 0.0)
                            }
                        }
                    };

                    if let Some(animal_id) = target_id {
                        let combat_ctx = animals::combat::CombatContext::default();

                        // Process the attack
                        if let Some(result) = animals::combat::player_attack_animal(
                            &mut state.animal_manager,
                            animal_id,
                            base_damage,
                            Some(weapon_name),
                            &combat_ctx,
                        ) {
                            // Process loot if killed
                            if result.killed {
                                let loot_result = state.process_combat_loot(&result, weapon_name);

                                // Spawn dropped items in the world
                                let drop_pos = result.position + Vec3::new(0.0, 0.5, 0.0);
                                for notification in &loot_result.loot {
                                    state.dropped_items.spawn_drop(
                                        notification.item.clone(),
                                        drop_pos + Vec3::new(
                                            (rand::random::<f32>() - 0.5) * 1.0,
                                            0.0,
                                            (rand::random::<f32>() - 0.5) * 1.0,
                                        ),
                                    );
                                }

                                // If killed a pheasant, also spawn a pheasant carcass item for pickup
                                if result.species == animals::AnimalSpecies::RingNeckedPheasant {
                                    let carcass = economy::Item::new(
                                        "pheasant_carcass",
                                        "Pheasant Carcass",
                                        economy::ItemType::Food,
                                        15,
                                    );
                                    state.dropped_items.spawn_drop(carcass, drop_pos);
                                    log::info!("[HUNT] Shot a pheasant! Carcass dropped at {:?}", drop_pos);
                                }

                                // Spawn currency drops if any
                                if loot_result.total_wampum > 0 || loot_result.total_tobacco > 0 {
                                    state.dropped_items.spawn_currency(
                                        loot_result.total_wampum,
                                        loot_result.total_tobacco,
                                        drop_pos,
                                    );
                                }

                                // Log loot for debugging
                                if !loot_result.loot.is_empty() || loot_result.total_wampum > 0 {
                                    log::info!(
                                        "Killed {:?}! Spawned {} drops, {} wampum",
                                        loot_result.species,
                                        loot_result.loot.len(),
                                        loot_result.total_wampum
                                    );
                                }

                                // Notify ecology system of animal kill (affects ecosystem health)
                                state.systems_manager.record_hunt(
                                    loot_result.species,
                                    result.position,
                                );

                                // Reset combat timer
                                state.combat_kill_time = 0.0;
                            }
                        }
                    }
                }

                if state.swing_animation.swing_progress >= 1.0 {
                    state.swing_animation.is_swinging = false;
                    state.swing_animation.swing_progress = 0.0;
                }
            }

            // Decay muzzle flash quickly (fades in ~0.1 seconds)
            if state.swing_animation.muzzle_flash > 0.0 {
                state.swing_animation.muzzle_flash -= delta * 10.0;
                if state.swing_animation.muzzle_flash < 0.0 {
                    state.swing_animation.muzzle_flash = 0.0;
                }
            }

            // Update Atmosphere (fog, light shafts based on time/weather)
            // Pass max_visible_distance so fog_end hides object pop-in at LOD boundaries
            // Trees render at 3.5x render_distance with LOD system, so fog must extend there
            let weather_fog = match state.weather.current_weather {
                WeatherType::Foggy => 0.8,
                WeatherType::Overcast => 0.3,
                WeatherType::Stormy => 0.5,
                _ => 0.0,
            };
            let time_of_day = state.time_of_day;
            let cloud_coverage = state.weather.cloud_coverage;
            let max_visible_dist = state.render_distance * state.dither_distance_ratio * 3.0; // Match LOD2 max distance
            state.atmosphere.update(time_of_day, weather_fog, cloud_coverage, max_visible_dist);

            // Override fog density based on manual fog_level (\ key)
            // 0=Off, 1=Light, 2=Medium, 3=Heavy, 4=Dense
            let fog_multiplier = match state.fog_level {
                0 => 0.0,   // Off
                1 => 0.3,   // Light
                2 => 0.6,   // Medium
                3 => 1.0,   // Heavy
                _ => 1.5,   // Dense
            };
            state.atmosphere.state.fog_density = fog_multiplier;

            // Debug: Print fog and weather values every 5 seconds
            state.debug_timer += delta;
            if state.debug_timer > 5.0 {
                state.debug_timer = 0.0;
                let fog = state.atmosphere.fog_params();
                println!("[DEBUG] Weather: {:?} | Fog: level={} density={:.2}, start={:.1}, end={:.1} | Clouds: {:.2}",
                    state.weather.current_weather, state.fog_level, fog[0], fog[1], fog[2], state.weather.cloud_coverage);
            }

            // Update Animal System
            // Sync time of day with animal manager
            let animal_time = AnimalTimeOfDay::from_hour(state.time_of_day as u8);
            state.animal_manager.set_time_of_day(animal_time);
            // Update animal AI and movement
            let player_pos = state.player.position;
            let player_vel = state.player.velocity;
            let terrain_seed = state.seed;
            state.animal_manager.update(delta, player_pos, player_vel, |x, z| {
                croatoan_wfc::get_height_at(x, z, terrain_seed).0
            });

            // Update quadruped IK for ground adaptation (horses, wolves, deer, etc.)
            let ik_seed = state.seed;
            state.animal_manager.update_ik(
                |x, z| croatoan_wfc::get_height_at(x, z, ik_seed).0,
                player_pos,
                50.0, // Max IK distance from player
            );

            // === AUDIO EVENTS INTEGRATION ===
            // Collect audio events to avoid borrow conflicts
            let audio_events: Vec<AudioEvent> = {
                let mut events = Vec::new();

                // Process animal encounters for audio
                let mut in_combat = false;
                for animal in state.animal_manager.animals_near(player_pos, 50.0) {
                    let distance = (animal.position - player_pos).length();
                    let species_name = animal.species.name().to_string();
                    let threat = species_threat_profile(&species_name);

                    match animal.behavior_state {
                        BehaviorState::Attack(_) => {
                            if !in_combat {
                                events.push(AudioEvent::AnimalCombatStart { species: species_name });
                                in_combat = true;
                            }
                        }
                        BehaviorState::Pursue(_) | BehaviorState::Alert(_) => {
                            events.push(AudioEvent::AnimalDetected {
                                species: species_name,
                                threat,
                                distance
                            });
                        }
                        BehaviorState::Flee(_) => {
                            events.push(AudioEvent::AnimalFleeing { species: species_name });
                        }
                        _ => {}
                    }
                }

                // Time-of-day audio events
                let prev_hour = ((state.time_of_day - delta * 0.1) + 24.0) % 24.0;
                let curr_hour = state.time_of_day;

                if prev_hour < 6.0 && curr_hour >= 6.0 {
                    events.push(AudioEvent::SunriseBegins);
                }
                if prev_hour < 18.0 && curr_hour >= 18.0 {
                    events.push(AudioEvent::SunsetBegins);
                }

                // Storm approaching
                if state.weather.target_weather == WeatherType::Stormy &&
                   state.weather.transition_progress < 0.1 {
                    events.push(AudioEvent::StormApproaching);
                }

                events
            };

            // Collect village events
            let (is_in_village, population) = state.village_manager.is_player_in_village(player_pos);
            let village_event = if is_in_village && !state.was_in_village {
                Some(AudioEvent::VillageEntered { population })
            } else if !is_in_village && state.was_in_village {
                Some(AudioEvent::VillageExited)
            } else {
                None
            };
            state.was_in_village = is_in_village;

            // Process all audio events - use std::mem::take to avoid borrow conflicts
            {
                let mut audio_processor = std::mem::take(&mut state.audio_event_processor);
                for event in audio_events {
                    audio_processor.process_event(event, &mut state.audio_system);
                }
                if let Some(event) = village_event {
                    audio_processor.process_event(event, &mut state.audio_system);
                }
                audio_processor.update(delta, &mut state.audio_system);
                state.audio_event_processor = audio_processor;
            }

            // === DATA PIPELINE PROCESSING ===
            // Process game events through the unified data pipeline
            {
                // Process data pipeline and collect audio events
                let mut pipeline = std::mem::take(&mut state.data_pipeline);
                let pipeline_audio_events = pipeline.process(delta);
                state.data_pipeline = pipeline;

                // Send pipeline-generated audio events to audio processor
                if !pipeline_audio_events.is_empty() {
                    let mut audio_processor = std::mem::take(&mut state.audio_event_processor);
                    for event in pipeline_audio_events {
                        audio_processor.process_event(event, &mut state.audio_system);
                    }
                    state.audio_event_processor = audio_processor;
                }

                // Cleanup NPC audio integration periodically
                let cleanup_time = state.data_pipeline.stats().events_processed as f32 * 0.016;
                state.npc_audio.cleanup(cleanup_time);
            }

            // Update Game Progression System
            {
                use crate::progression::reputation::Faction;
                use std::collections::HashMap;

                // Collect faction reputation for NPC behavior
                let faction_rep: HashMap<Faction, i32> = state.game_progression.player_progression.reputation
                    .iter()
                    .map(|(k, v)| (*k, v.value))
                    .collect();

                // Update game progression (quests, events, NPC schedules)
                state.game_progression.update(delta, player_pos, player_vel, &faction_rep);

                // Update village NPC movement and communication
                let game_hour = (state.game_progression.game_time % 24.0) as f32;
                let player_look_dir = state.camera.forward();
                state.village_manager.update(delta, game_hour, player_pos, player_look_dir);

                // Update NPC manager (schedules, behaviors, relationships)
                state.npc_manager.update(delta, player_pos, &faction_rep);

                // Update unified agent manager (cross-system coordination)
                // Note: We use split borrows to avoid borrow checker conflicts
                let ua_game_time = state.game_progression.game_time;
                let ua_world_phase = state.game_progression.world_phase();
                let SharedState {
                    ref mut unified_agents,
                    ref mut npc_manager,
                    ref mut animal_manager,
                    ..
                } = *state;
                unified_agents.world_phase = ua_world_phase;
                let mut adapter = character_agent::unified_manager::CombinedAgentAdapter {
                    npcs: npc_manager,
                    animals: animal_manager,
                };
                unified_agents.update(delta, player_pos, player_vel, ua_game_time, &mut adapter);

                // Check for achievements
                let game_time = state.game_progression.game_time;
                let new_achievements = state.game_progression.player_progression.check_all_achievements(game_time);
                for achievement in new_achievements {
                    println!("[ACHIEVEMENT] Unlocked: {}", achievement);
                }

                // Validate progression state periodically
                state.game_progression.player_progression.validate();

                // Update faction system and process faction events
                state.game_progression.player_progression.update_faction_system(game_time);

                // Display faction notifications
                let notifications = state.game_progression.player_progression.get_faction_notifications();
                for notification in notifications {
                    if notification.importance >= crate::progression::faction_integration::NotificationImportance::Medium {
                        println!("[FACTION] {}", notification.message);
                    }
                }

                // Process pending dialogue effects
                let pending_effects = std::mem::take(&mut state.game_progression.interaction_system.pending_effects);
                for effect in pending_effects {
                    match effect {
                        npc::interaction::PendingEffect::ModifyReputation { faction, delta } => {
                            state.game_progression.player_progression.modify_reputation(faction, delta);
                            log::info!("[DIALOGUE EFFECT] Reputation {:?} {:+}", faction, delta);
                        }
                        npc::interaction::PendingEffect::GiveItem { item, count } => {
                            // Create item and add to inventory
                            let new_item = economy::Item::new(
                                &item,
                                &item,
                                economy::item::ItemType::Material,
                                10, // Base value
                            );
                            if let Err(e) = state.player_economy.inventory.add_item(new_item) {
                                log::warn!("[DIALOGUE EFFECT] Failed to give item {}: {:?}", item, e);
                            } else {
                                log::info!("[DIALOGUE EFFECT] Received {} x{}", item, count);
                            }
                        }
                        npc::interaction::PendingEffect::TakeItem { item, count } => {
                            // Find and remove item from inventory
                            if let Some(item_in_inv) = state.player_economy.inventory.slots.iter()
                                .filter_map(|s| s.as_ref())
                                .find(|i| i.name == item)
                                .map(|i| i.id) {
                                state.player_economy.inventory.remove_item(item_in_inv);
                                log::info!("[DIALOGUE EFFECT] Gave away {} x{}", item, count);
                            }
                        }
                        npc::interaction::PendingEffect::StartQuest(quest_id) => {
                            log::info!("[DIALOGUE EFFECT] Quest started: {}", quest_id);
                            // Quest system integration would go here
                        }
                        npc::interaction::PendingEffect::CompleteObjective { quest, objective } => {
                            log::info!("[DIALOGUE EFFECT] Objective completed: {} - {}", quest, objective);
                        }
                        npc::interaction::PendingEffect::SetFlag { flag, value } => {
                            log::info!("[DIALOGUE EFFECT] Flag set: {} = {}", flag, value);
                        }
                        npc::interaction::PendingEffect::UnlockTrading(npc_id) => {
                            log::info!("[DIALOGUE EFFECT] Trading unlocked with NPC #{}", npc_id);
                        }
                        npc::interaction::PendingEffect::TeachSkill(skill) => {
                            log::info!("[DIALOGUE EFFECT] Learned skill: {}", skill);
                        }
                        npc::interaction::PendingEffect::Heal(amount) => {
                            log::info!("[DIALOGUE EFFECT] Healed for {}", amount);
                            // Player health system integration would go here
                        }
                        npc::interaction::PendingEffect::ModifyRelationship { npc_id, affinity, trust, respect } => {
                            // Already handled by interaction system
                            log::info!("[DIALOGUE EFFECT] Relationship modified: NPC#{} A{:+} T{:+} R{:+}",
                                npc_id, affinity, trust, respect);
                        }
                        npc::interaction::PendingEffect::SpreadRumor { positive, radius } => {
                            log::info!("[DIALOGUE EFFECT] Rumor spread ({}) in {}m radius",
                                if positive { "positive" } else { "negative" }, radius);
                        }
                    }
                }
            }

            // Update animal visual representation - only render nearby animals
            // Animals with 3D models use the model pipeline, others use orbs as fallback
            let nearby_animals = state.animal_manager.animals_near(player_pos, state.render_distance * 0.5);

            // Separate animals by whether they have 3D models
            let mut orb_instances: Vec<OrbInstance> = Vec::new();
            let mut model_instances: std::collections::HashMap<&'static str, Vec<AnimalInstance>> = std::collections::HashMap::new();
            // Track animation state counts per species to pick dominant animation
            let mut species_anim_states: std::collections::HashMap<&'static str, std::collections::HashMap<animals::AnimationState, usize>> = std::collections::HashMap::new();

            for animal in &nearby_animals {
                let base_color = animal.species.orb_color();
                let scale = animal.species.orb_scale();

                // Modify color based on behavior state
                let (mut color, mut emissive) = match &animal.behavior_state {
                    animals::BehaviorState::Attack(_) => {
                        // Red tint and strong glow when attacking
                        ([base_color[0] * 1.5, base_color[1] * 0.5, base_color[2] * 0.5], 0.8)
                    }
                    animals::BehaviorState::Pursue(_) => {
                        // Orange-ish tint and moderate glow when pursuing
                        ([base_color[0] * 1.3, base_color[1] * 0.8, base_color[2] * 0.6], 0.5)
                    }
                    animals::BehaviorState::Alert(_) => {
                        // Slight yellow tint when alert
                        ([base_color[0] * 1.1, base_color[1] * 1.1, base_color[2] * 0.8], 0.2)
                    }
                    animals::BehaviorState::Flee(_) => {
                        // Pale/washed out when fleeing
                        ([base_color[0] * 0.7, base_color[1] * 0.7, base_color[2] * 0.7], 0.0)
                    }
                    animals::BehaviorState::Dead => {
                        // Dark gray when dead
                        ([0.2, 0.2, 0.2], 0.0)
                    }
                    _ => {
                        // Normal color for idle/patrol
                        (base_color, 0.0)
                    }
                };

                // Apply damage flash effect (bright white/red flash when hit)
                let (flash_tint, flash_emissive) = animal.damage_flash_effect();
                color = [
                    (color[0] * flash_tint[0]).min(2.0),
                    (color[1] * flash_tint[1]).min(2.0),
                    (color[2] * flash_tint[2]).min(2.0),
                ];
                emissive = (emissive + flash_emissive).min(2.0);

                // Check if this species has a 3D model
                if let Some(model_name) = animal.species.model_name() {
                    // Use 3D model pipeline
                    let model_scale = animal.species.model_scale();
                    // Apply Y offset to correct model anchor points (e.g., stag antlers)
                    let y_offset = animal.species.model_y_offset();

                    // Apply IK tilt for slope adaptation (position.y already at ground from manager update)
                    let ik_tilt = animal.get_ik_pelvis_tilt()
                        .unwrap_or(glam::Quat::IDENTITY);

                    // Animation is now handled via GPU skeletal animation (joint matrices)
                    // No procedural transform modifications needed - shader handles it

                    // Combine base rotation with IK pelvis tilt
                    let final_rotation = animal.rotation * ik_tilt;

                    // Model position - animal.position.y is already ground height
                    let model_position = animal.position + Vec3::new(0.0, y_offset, 0.0);

                    // Model scale (animation via GPU skinning, not transform scale)
                    let final_scale = Vec3::splat(model_scale);

                    let model_matrix = Mat4::from_scale_rotation_translation(
                        final_scale,
                        final_rotation,
                        model_position,
                    );
                    let instance = AnimalInstance::new(model_matrix, color, emissive);
                    model_instances.entry(model_name).or_insert_with(Vec::new).push(instance);
                    // Track animation state for this species
                    *species_anim_states.entry(model_name).or_default().entry(animal.animation_state).or_insert(0) += 1;
                } else {
                    // Fall back to orb rendering
                    let pos = animal.position + Vec3::new(0.0, scale * 0.5 + 0.5, 0.0);
                    let model_matrix = Mat4::from_scale_rotation_translation(
                        Vec3::splat(scale),
                        glam::Quat::IDENTITY,
                        pos,
                    );
                    orb_instances.push(OrbInstance {
                        model_matrix: model_matrix.to_cols_array_2d(),
                        color,
                        emissive,
                    });
                }
            }

            // Upload model instances to model pipeline
            {
                let mut model_pipeline = animal_model_pipeline_mutex.safe_lock();
                let model_cache = animal_model_cache_mutex.safe_lock();

                // Upload meshes for any new species (only once per species)
                for (model_name, instances) in &model_instances {
                    if !model_pipeline.has_mesh(model_name) {
                        if let Some(loaded_model) = model_cache.get(model_name) {
                            // Combine all meshes into one for this species
                            let mut all_vertices: Vec<AnimalVertex> = Vec::new();
                            let mut all_indices: Vec<u32> = Vec::new();

                            // Find texture: first try embedded, then create from baseColorFactor
                            let mut texture_data: Option<gltf_loader::LoadedTexture> = None;

                            // Check if this model has a skeleton (is animated)
                            let model_is_animated = loaded_model.is_animated();

                            for mesh in &loaded_model.meshes {
                                // Skip non-skinned meshes in animated models - they're often
                                // broken/mispositioned helper objects (e.g. pheasant tail quads)
                                if model_is_animated && !mesh.is_skinned() {
                                    log::debug!("[AnimalModel] Skipping non-skinned mesh '{}' in animated model '{}'",
                                        mesh.name, model_name);
                                    continue;
                                }

                                let vertex_offset = all_vertices.len() as u32;
                                for i in 0..mesh.positions.len() {
                                    // Get joint indices and weights if available (for skinned meshes)
                                    let joint_indices = mesh.joint_indices.get(i)
                                        .map(|j| [j[0] as u32, j[1] as u32, j[2] as u32, j[3] as u32])
                                        .unwrap_or([0, 0, 0, 0]);
                                    let joint_weights = mesh.joint_weights.get(i)
                                        .copied()
                                        .unwrap_or([0.0, 0.0, 0.0, 0.0]);

                                    all_vertices.push(AnimalVertex {
                                        position: mesh.positions[i],
                                        normal: mesh.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                                        uv: mesh.uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                                        color: mesh.colors.get(i).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]),
                                        joints: joint_indices,
                                        weights: joint_weights,
                                    });
                                }
                                for idx in &mesh.indices {
                                    all_indices.push(*idx + vertex_offset);
                                }

                                // Prefer OPAQUE textures over BLEND (body over feathers/fur)
                                // Uses get_or_create_texture() which creates synthetic texture
                                // from baseColorFactor if no embedded texture exists
                                if let Some(tex) = mesh.material.get_or_create_texture() {
                                    if texture_data.is_none() || mesh.material.alpha_mode == "OPAQUE" {
                                        texture_data = Some(tex);
                                    }
                                }
                            }

                            model_pipeline.upload_species_mesh(
                                ctx.device(),
                                model_name,
                                &all_vertices,
                                &all_indices,
                            );

                            // Upload texture if available (embedded or from baseColorFactor)
                            if let Some(ref tex) = texture_data {
                                model_pipeline.upload_species_texture(
                                    ctx.device(),
                                    ctx.queue(),
                                    model_name,
                                    &tex.data,
                                    tex.width,
                                    tex.height,
                                );
                            }

                            // Upload skeleton and animations if available
                            if loaded_model.is_animated() {
                                if let Some(ref skeleton) = loaded_model.skeleton {
                                    // Convert gltf_loader skeleton to pipeline format
                                    let skeleton_gpu = croatoan_render::SkeletonGpu {
                                        inverse_bind_matrices: skeleton.inverse_bind_matrices.clone(),
                                        parents: skeleton.joints.iter().map(|j| j.parent).collect(),
                                        local_transforms: skeleton.joints.iter().map(|j| {
                                            // Build local transform matrix from components
                                            let t = j.local_translation;
                                            let r = j.local_rotation;
                                            let s = j.local_scale;
                                            let mat = glam::Mat4::from_scale_rotation_translation(
                                                Vec3::from_array(s),
                                                glam::Quat::from_xyzw(r[0], r[1], r[2], r[3]),
                                                Vec3::from_array(t),
                                            );
                                            (mat.to_cols_array_2d(), r, s)
                                        }).collect(),
                                        roots: skeleton.roots.clone(),
                                    };

                                    // Convert animations
                                    let animations_gpu: Vec<croatoan_render::AnimationGpu> = loaded_model.animations.iter().map(|anim| {
                                        let joint_count = skeleton.joints.len();
                                        let mut joint_keyframes = vec![croatoan_render::JointKeyframes::default(); joint_count];

                                        for channel in &anim.channels {
                                            if channel.joint_index < joint_count {
                                                let kf = &mut joint_keyframes[channel.joint_index];
                                                match channel.property {
                                                    gltf_loader::AnimationProperty::Translation => {
                                                        kf.translation_times = channel.times.clone();
                                                        kf.translations = channel.values.iter()
                                                            .map(|v| [v[0], v[1], v[2]])
                                                            .collect();
                                                    }
                                                    gltf_loader::AnimationProperty::Rotation => {
                                                        kf.rotation_times = channel.times.clone();
                                                        kf.rotations = channel.values.clone();
                                                    }
                                                    gltf_loader::AnimationProperty::Scale => {
                                                        kf.scale_times = channel.times.clone();
                                                        kf.scales = channel.values.iter()
                                                            .map(|v| [v[0], v[1], v[2]])
                                                            .collect();
                                                    }
                                                }
                                            }
                                        }

                                        croatoan_render::AnimationGpu {
                                            name: anim.name.clone(),
                                            duration: anim.duration,
                                            joint_keyframes,
                                        }
                                    }).collect();

                                    model_pipeline.upload_species_animation(
                                        ctx.device(),
                                        model_name,
                                        skeleton_gpu,
                                        animations_gpu,
                                    );

                                    println!("[Animal] Uploaded {} animations for {}: {:?}",
                                        loaded_model.animations.len(),
                                        model_name,
                                        loaded_model.animation_names()
                                    );
                                }
                            }
                        }
                    }

                    // Upload instances for this species
                    model_pipeline.upload_instances(ctx.device(), model_name, instances);

                    // Update skeletal animation for animated species
                    // Skip pheasants - their animation is broken and causes mesh stretching
                    if model_pipeline.has_animations(model_name) && *model_name != "ring_necked_pheasant" {
                        // Find dominant animation state for this species
                        let dominant_state = species_anim_states.get(model_name)
                            .and_then(|states| states.iter().max_by_key(|(_, count)| *count).map(|(state, _)| *state))
                            .unwrap_or(animals::AnimationState::Idle);

                        // Map AnimationState to GLTF animation name
                        let anim_name = match dominant_state {
                            animals::AnimationState::Idle => "Idle",
                            animals::AnimationState::Walking => "Walk",
                            animals::AnimationState::Running => "Gallop",
                            animals::AnimationState::Attacking => "Attack",
                            animals::AnimationState::TakingDamage => "Idle_HitReact1",
                            animals::AnimationState::Dying | animals::AnimationState::Dead => "Death",
                        };

                        // Animation speed varies by state
                        let anim_speed = match dominant_state {
                            animals::AnimationState::Idle => 1.0,
                            animals::AnimationState::Walking => 1.5,
                            animals::AnimationState::Running => 2.0,
                            animals::AnimationState::Attacking => 2.5,
                            _ => 1.0,
                        };
                        let anim_time = state.time_of_day * 150.0 * anim_speed;

                        if let Some(joint_matrices) = model_pipeline.compute_animation_matrices(
                            model_name,
                            anim_name,
                            anim_time,
                        ) {
                            model_pipeline.update_joint_matrices(
                                ctx.queue(),
                                model_name,
                                &joint_matrices,
                            );
                        }
                    }
                }
            }

            // Add village NPC orbs
            for npc_orb in &state.village_manager.npc_orbs {
                let scale = 1.2; // NPCs slightly larger than animals
                let model_matrix = Mat4::from_scale_rotation_translation(
                    Vec3::splat(scale),
                    glam::Quat::IDENTITY,
                    npc_orb.position,
                );
                orb_instances.push(OrbInstance {
                    model_matrix: model_matrix.to_cols_array_2d(),
                    color: npc_orb.color,
                    emissive: 0.6, // NPCs glow more
                });
            }

            // Add dropped item orbs (skip weapons - they're rendered as 3D models)
            let game_time = state.time_of_day; // For bounce animation
            for drop in state.dropped_items.all_drops() {
                // Skip weapons - they're rendered as actual GLB models
                if drop.item.item_type == economy::ItemType::Weapon {
                    continue;
                }
                let scale = drop.scale();
                let bounce = drop.bounce_offset(game_time);
                let pos = drop.position + Vec3::new(0.0, scale * 0.5 + bounce + 0.2, 0.0);
                let model_matrix = Mat4::from_scale_rotation_translation(
                    Vec3::splat(scale),
                    glam::Quat::from_rotation_y(drop.rotation),
                    pos,
                );
                orb_instances.push(OrbInstance {
                    model_matrix: model_matrix.to_cols_array_2d(),
                    color: drop.color(),
                    emissive: drop.glow_intensity + if drop.is_highlighted { 0.5 } else { 0.0 },
                });
            }

            // Upload instances to GPU
            let mut orb_pipeline = animal_orb_pipeline_mutex.safe_lock();
            orb_pipeline.upload_instances(ctx.device(), &orb_instances);
        }

        // Handle Input (Player Controller)
        if state.game_state == GameState::Playing {
            let mut input_dir = Vec3::ZERO;
            if state.keys.get(&KeyCode::KeyW) == Some(&ElementState::Pressed) { input_dir.z += 1.0; }
            if state.keys.get(&KeyCode::KeyS) == Some(&ElementState::Pressed) { input_dir.z -= 1.0; }
            if state.keys.get(&KeyCode::KeyA) == Some(&ElementState::Pressed) { input_dir.x -= 1.0; }
            if state.keys.get(&KeyCode::KeyD) == Some(&ElementState::Pressed) { input_dir.x += 1.0; }

            // Sprint with Shift (70% speed boost)
            let is_sprinting = state.keys.get(&KeyCode::ShiftLeft) == Some(&ElementState::Pressed) ||
                               state.keys.get(&KeyCode::ShiftRight) == Some(&ElementState::Pressed);
            let speed_multiplier = if is_sprinting { 1.7 } else { 1.0 };

            // Temporarily increase player speed for sprinting
            let original_speed = state.player.speed;
            state.player.speed = original_speed * speed_multiplier;

            let seed = state.seed; // Copy seed to avoid borrow error
            let player_y = state.player.position.y;

            // Clone worm tunnels to avoid borrow conflict with player
            let worm_tunnels: Vec<WormTunnel> = state.worm_tunnels.clone();

            // Create cave-aware height function
            // Check if player is inside any worm tunnel - if so, allow traversal below terrain
            state.player.update_with_height_fn(delta, input_dir, seed, |x, z| {
                let (terrain_height, _) = croatoan_wfc::get_height_at(x, z, seed);

                // Check if we're inside or near any worm tunnel at this XZ position
                for tunnel in worm_tunnels.iter() {
                    // First check if there's a tunnel at terrain level (entrance detection)
                    let surface_test = Vec3::new(x, terrain_height - 1.0, z);
                    let surface_sdf = sample_worm_sdf(surface_test, tunnel);

                    // If tunnel exists near surface, check if we should be in it
                    if surface_sdf < 2.0 {
                        // Sample at current player Y position
                        let test_pos = Vec3::new(x, player_y, z);
                        let sdf = sample_worm_sdf(test_pos, tunnel);

                        if sdf < 0.0 {
                            // Inside tunnel - find tunnel floor
                            let mut floor_y = player_y;
                            for step in 0..50 {
                                let check_y = player_y - step as f32 * 0.5;
                                let check_pos = Vec3::new(x, check_y, z);
                                let check_sdf = sample_worm_sdf(check_pos, tunnel);
                                if check_sdf >= 0.0 {
                                    floor_y = check_y + 0.5;
                                    break;
                                }
                                floor_y = check_y;
                            }
                            return floor_y;
                        } else if surface_sdf < 0.0 {
                            // At cave entrance but above tunnel - allow falling in
                            // Find where the tunnel air space starts
                            let mut cave_floor = terrain_height;
                            for step in 0..30 {
                                let check_y = terrain_height - step as f32 * 0.5;
                                let check_pos = Vec3::new(x, check_y, z);
                                let check_sdf = sample_worm_sdf(check_pos, tunnel);
                                if check_sdf < 0.0 {
                                    // Found tunnel air - now find the floor
                                    for floor_step in 0..50 {
                                        let floor_check_y = check_y - floor_step as f32 * 0.5;
                                        let floor_check_pos = Vec3::new(x, floor_check_y, z);
                                        let floor_sdf = sample_worm_sdf(floor_check_pos, tunnel);
                                        if floor_sdf >= 0.0 {
                                            cave_floor = floor_check_y + 0.5;
                                            break;
                                        }
                                        cave_floor = floor_check_y;
                                    }
                                    return cave_floor;
                                }
                            }
                        }
                    }
                }

                // Not in any tunnel, use terrain height
                terrain_height
            });

            // Restore original speed
            state.player.speed = original_speed;

            // Update footsteps audio based on horizontal movement speed and terrain
            let horizontal_speed = Vec2::new(state.player.velocity.x, state.player.velocity.z).length();
            // Check if near water: biome_t < 0.5 is beach/shoreline (within ~5 yards of surf)
            let biome_t = get_biome_t(state.player.position.x, state.player.position.z, state.seed);
            let near_water = biome_t < 0.5;
            state.audio_system.update_footsteps(horizontal_speed, near_water);

            // Sync Camera to Player
            state.camera.position = state.player.position;
            state.camera.yaw = state.player.yaw;
            state.camera.pitch = state.player.pitch;
            state.camera.update_vectors();
        } else {
            // Menu Camera (Orbit)
            state.camera.yaw += 0.1 * delta;
            state.camera.update_vectors();
        }

        // Sun Billboard


        // Moon Billboard (Proper MoonPipeline with silver/white colors)
        static MOON_PIPELINE: OnceLock<Mutex<MoonPipeline>> = OnceLock::new();
        let moon_pipeline_mutex = MOON_PIPELINE.get_or_init(|| {
            Mutex::new(MoonPipeline::new(ctx.device(), ctx.surface_format()))
        });

        // Egui Input
        let raw_input = if let Some(egui_state) = &mut state.egui_state {
            egui_state.take_egui_input(&ctx.window)
        } else {
            egui::RawInput::default()
        };

        let egui_ctx = state.egui_ctx.clone();
        let full_output = egui_ctx.run(raw_input, |ui_ctx| {
            // UI Styling
            let mut style = (*ui_ctx.style()).clone();
            style.visuals.window_fill = egui::Color32::from_rgb(244, 228, 188); // Paper Color
            style.visuals.panel_fill = egui::Color32::from_rgb(244, 228, 188);
            // Remove window stroke/shadow to prevent outline artifacts
            style.visuals.window_stroke = egui::Stroke::NONE;
            style.visuals.window_shadow = egui::epaint::Shadow::NONE;
            ui_ctx.set_style(style);

            // Sync Cursor State with Game State (show cursor when UI overlays are open)
            let journal_open = state.perks_journal.is_open;
            let chest_open = state.open_chest_id.is_some();
            match state.game_state {
                GameState::Menu | GameState::Loading | GameState::Paused => {
                    ctx.window.set_cursor_visible(true);
                    let _ = ctx.window.set_cursor_grab(CursorGrabMode::None);
                }
                GameState::Playing => {
                    if journal_open || chest_open {
                        // Show cursor when journal or chest UI is open
                        ctx.window.set_cursor_visible(true);
                        let _ = ctx.window.set_cursor_grab(CursorGrabMode::None);
                    } else {
                        // Lock cursor to window and hide it during gameplay
                        ctx.window.set_cursor_visible(false);
                        let _ = ctx.window.set_cursor_grab(CursorGrabMode::Confined);
                    }
                }
            }

            match state.game_state {
                GameState::Loading => {
                    // Use transparent frame for loading screen
                    let frame = egui::Frame::none();
                    egui::CentralPanel::default().frame(frame).show(ui_ctx, |ui| {
                        // Load slideshow images if not loaded
                        if state.loading_slideshow.textures.is_empty() {
                            state.loading_slideshow.load_images(ui.ctx());
                        }

                        // Update slideshow animation using actual elapsed time
                        let slideshow_dt = state.loading_slideshow.start_time.elapsed().as_secs_f32();
                        state.loading_slideshow.start_time = Instant::now();
                        // Clamp dt to avoid huge jumps (first frame or lag spikes)
                        let slideshow_dt = slideshow_dt.clamp(0.0, 0.1);
                        state.loading_slideshow.update(slideshow_dt);

                        // Draw slideshow background with Ken Burns effect
                        state.loading_slideshow.render(ui);

                        // Request continuous repaints for smooth animation
                        ui.ctx().request_repaint();

                        // Semi-transparent overlay for readability
                        let screen_rect = ui.ctx().screen_rect();
                        ui.painter().rect_filled(
                            screen_rect,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100),
                        );

                        ui.vertical_centered(|ui| {
                            ui.add_space(150.0);
                            ui.heading(egui::RichText::new("Loading World").size(40.0).color(egui::Color32::WHITE));
                            ui.add_space(30.0);

                            // Progress Bar
                            let progress = if state.loading_progress.total_chunks > 0 {
                                state.loading_progress.chunks_uploaded as f32 / state.loading_progress.total_chunks as f32
                            } else {
                                0.0
                            };

                            ui.add(egui::ProgressBar::new(progress)
                                .text(format!("{} / {}", state.loading_progress.chunks_uploaded, state.loading_progress.total_chunks))
                                .desired_width(400.0));

                            ui.add_space(20.0);

                            // Detailed Status
                            ui.label(egui::RichText::new(&state.loading_progress.current_status)
                                .size(16.0)
                                .color(egui::Color32::LIGHT_GRAY));

                            ui.add_space(10.0);

                            // Additional Progress Info
                            ui.label(egui::RichText::new(format!(
                                "Generated: {} | Uploaded: {}",
                                state.loading_progress.chunks_generated,
                                state.loading_progress.chunks_uploaded
                            )).color(egui::Color32::LIGHT_GRAY));
                        });
                    });
                }
                GameState::Menu => {
                    // Transparent panel for background
                    let frame = egui::Frame::none();
                    egui::CentralPanel::default().frame(frame).show(ui_ctx, |ui| {
                        // Load background texture if not loaded
                        if state.background_texture.is_none() {
                            let path = "assets/ui/roanoke1.png";
                            if let Ok(bytes) = std::fs::read(path) {
                                if let Ok(image) = image::load_from_memory(&bytes) {
                                    let size = [image.width() as usize, image.height() as usize];
                                    let image_buffer = image.to_rgba8();
                                    let pixels = image_buffer.as_flat_samples();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        pixels.as_slice(),
                                    );
                                    state.background_texture = Some(ui.ctx().load_texture(
                                        "background",
                                        color_image,
                                        egui::TextureOptions::LINEAR,
                                    ));
                                    println!("[UI] Loaded background image from {}", path);
                                } else {
                                    println!("[UI] Failed to decode background image");
                                }
                            }
                        }

                        // Draw Background
                        let screen_rect = ui.ctx().screen_rect();
                        if let Some(texture) = &state.background_texture {
                            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                            ui.painter().image(
                                texture.id(),
                                screen_rect,
                                uv,
                                egui::Color32::WHITE,
                            );
                        }

                        // Menu styling - right side, serif-style
                        let menu_width = 350.0;
                        let menu_x = screen_rect.max.x - menu_width - 80.0;
                        let menu_y = screen_rect.center().y - 60.0;

                        // Colors - rich brown tones
                        let title_color = egui::Color32::from_rgb(61, 43, 31); // Deep brown
                        let menu_color = egui::Color32::from_rgb(101, 67, 33); // Dark brown
                        let hover_color = egui::Color32::from_rgb(160, 100, 50); // Warm brown
                        let disabled_color = egui::Color32::from_rgb(140, 130, 115); // Muted

                        // Big bold ROANOKE title
                        ui.painter().text(
                            egui::pos2(menu_x + menu_width - 10.0, menu_y - 100.0),
                            egui::Align2::RIGHT_CENTER,
                            "ROANOKE",
                            egui::FontId::new(96.0, egui::FontFamily::Proportional),
                            title_color,
                        );
                        // Version subtitle
                        ui.painter().text(
                            egui::pos2(menu_x + menu_width - 10.0, menu_y - 45.0),
                            egui::Align2::RIGHT_CENTER,
                            "v0.0.3",
                            egui::FontId::new(28.0, egui::FontFamily::Proportional),
                            egui::Color32::from_rgb(120, 90, 60),
                        );

                        // Menu items
                        let menu_items = [
                            ("Continue", false), // Disabled if no saves
                            ("New Game", true),
                            ("Load Game", true),
                            ("Settings", false), // Not implemented
                            ("Exit", true),
                        ];

                        let saves = list_saves();
                        let has_saves = !saves.is_empty();

                        let mut y_offset = menu_y;
                        for (label, base_enabled) in menu_items {
                            let enabled = match label {
                                "Continue" => has_saves,
                                "Load Game" => has_saves,
                                _ => base_enabled,
                            };

                            let btn_rect = egui::Rect::from_min_size(
                                egui::pos2(menu_x, y_offset),
                                egui::vec2(menu_width, 65.0),
                            );

                            let response = ui.allocate_rect(btn_rect, egui::Sense::click());
                            let is_hovered = response.hovered() && enabled;

                            // Draw text - big bold brown
                            let text_color = if !enabled {
                                disabled_color
                            } else if is_hovered {
                                hover_color
                            } else {
                                menu_color
                            };

                            let font_size = if is_hovered { 48.0 } else { 42.0 };
                            ui.painter().text(
                                egui::pos2(menu_x + menu_width - 10.0, y_offset + 32.0),
                                egui::Align2::RIGHT_CENTER,
                                label,
                                egui::FontId::new(font_size, egui::FontFamily::Proportional),
                                text_color,
                            );

                            // Handle clicks
                            if response.clicked() && enabled {
                                match label {
                                    "Continue" => {
                                        // Load most recent save
                                        if let Some(save_name) = saves.first() {
                                            if let Some(data) = load_game(save_name) {
                                                state.seed = data.seed;
                                                state.inventory = data.inventory;
                                                state.player.position = Vec3::from_array(data.player_pos);
                                                state.player.yaw = data.player_rot[0];
                                                state.player.pitch = data.player_rot[1];
                                                // Restore NPC relationships if present
                                                if let Some(relationships) = data.npc_relationships {
                                                    state.npc_manager.relationships = relationships;
                                                }
                                                state.game_state = GameState::Loading;
                                                state.save_name_input = save_name.clone();
                                                let range = 3;
                                                let total = saturating_range_area(range);
                                                state.loading_progress = LoadingProgress {
                                                    total_chunks: total,
                                                    chunks_generated: 0,
                                                    chunks_uploaded: 0,
                                                    current_status: "Loading saved world...".to_string(),
                                                };
                                                if let Some(manager) = CHUNK_MANAGER.get() {
                                                    let mut mgr = manager.safe_lock();
                                                    mgr.loaded_chunks.clear();
                                                    mgr.loading_chunks.clear();
                                                }
                                                // Village system disabled for now
                                                state.village_manager = VillageManager::new(data.seed);
                                                // state.village_manager.discover_villages(
                                                //     state.player.position,
                                                //     2000.0,
                                                //     10,
                                                // );
                                                // Initialize world features (rivers and caves)
                                                state.world_features = WorldFeatures::new(data.seed);
                                                // Village animals disabled for now
                                                // {
                                                //     let village_data = state.village_manager.get_village_spawn_data();
                                                //     let seed = state.village_manager.get_seed();
                                                //     spawn_village_animals(&mut state.animal_manager, &village_data, seed);
                                                // }
                                                // Spawn wild horse herds on beaches
                                                spawn_beach_horses(&mut state.animal_manager, data.seed);
                                                // Spawn pheasants near player spawn for early wildlife encounters
                                                spawn_pheasants_at_spawn(&mut state.animal_manager, data.seed);
                                                // register_village_factions(&mut *state);
                                            }
                                        }
                                    }
                                    "New Game" => {
                                        // Generate random seed
                                        let seed: u32 = rand::random();
                                        state.seed = seed;
                                        state.seed_input = seed.to_string();
                                        state.game_state = GameState::Loading;
                                        state.save_name_input = format!("seed_{}", seed);
                                        // Spawn on beach at ground level
                                        let spawn_pos = find_beach_spawn_position(seed);
                                        state.player = Player::new(spawn_pos);
                                        state.player.yaw = std::f32::consts::PI; // Face west (inland)
                                        println!("[GAME] Starting new game with seed: {} at beach ({:.1}, {:.1}, {:.1})",
                                            seed, spawn_pos.x, spawn_pos.y, spawn_pos.z);
                                        let range = 3;
                                        let total = saturating_range_area(range);
                                        state.loading_progress = LoadingProgress {
                                            total_chunks: total,
                                            chunks_generated: 0,
                                            chunks_uploaded: 0,
                                            current_status: "Initializing world generation...".to_string(),
                                        };
                                        if let Some(manager) = CHUNK_MANAGER.get() {
                                            let mut mgr = manager.safe_lock();
                                            mgr.loaded_chunks.clear();
                                            mgr.loading_chunks.clear();
                                        }
                                        // Village system disabled for now
                                        state.village_manager = VillageManager::new(seed);
                                        // state.village_manager.discover_villages(
                                        //     state.player.position,
                                        //     2000.0,
                                        //     10,
                                        // );
                                        // Initialize world features (rivers and caves)
                                        state.world_features = WorldFeatures::new(seed);
                                        let stats = state.world_features.stats();
                                        println!("[WORLD] Rivers: {}, Caves: {}, Waterfalls: {}",
                                            stats.river_count, stats.cave_count, stats.waterfall_count);
                                        // Village animals disabled for now
                                        // {
                                        //     let village_data = state.village_manager.get_village_spawn_data();
                                        //     let seed = state.village_manager.get_seed();
                                        //     spawn_village_animals(&mut state.animal_manager, &village_data, seed);
                                        // }
                                        // Spawn wild horse herds on beaches
                                        spawn_beach_horses(&mut state.animal_manager, seed);
                                        // Spawn pheasants near player spawn for early wildlife encounters
                                        spawn_pheasants_at_spawn(&mut state.animal_manager, seed);
                                        // Spawn starter chests near beach spawn (scattered along shoreline)
                                        {
                                            use croatoan_wfc::get_height_at;
                                            let base_x = spawn_pos.x;
                                            let base_z = spawn_pos.z;

                                            // Spawn chests scattered along the beach and in shallow water
                                            let chest_offsets = [
                                                (-3.0, 2.0, 0.0),      // Near player, facing east
                                                (5.0, -8.0, 0.5),      // South along beach
                                                (-2.0, 15.0, -0.3),    // North along beach
                                                (8.0, -20.0, 0.8),     // Further south
                                                (-5.0, 25.0, -0.6),    // Further north
                                                (-15.0, 5.0, 0.2),     // In shallow water (west)
                                                (-12.0, -10.0, -0.4),  // In shallow water (southwest)
                                            ];

                                            for (i, (dx, dz, rot)) in chest_offsets.iter().enumerate() {
                                                let x = base_x + dx;
                                                let z = base_z + dz;
                                                let (height, _) = get_height_at(x, z, seed);
                                                // Use terrain height, but if underwater, place at water surface
                                                // Water level is ~0.0, so chests in water float at surface
                                                let chest_y = if height < 0.5 { 0.3 } else { height };
                                                let chest_pos = Vec3::new(x, chest_y, z);
                                                let chest_id = state.storage_manager.spawn_container(
                                                    economy::ContainerType::WoodenChest,
                                                    chest_pos,
                                                    *rot,
                                                );

                                                // Add weapons to each chest - varied loadout
                                                if let Some(chest) = state.storage_manager.get_mut(chest_id) {
                                                    // Every chest gets a dagger
                                                    let mut dagger = economy::Item::new(
                                                        "dagger",
                                                        "Dagger",
                                                        economy::ItemType::Weapon,
                                                        35,
                                                    );
                                                    dagger.rarity = economy::Rarity::Common;
                                                    let _ = chest.add_item(dagger);

                                                    // First two chests get flintlocks
                                                    if i < 2 {
                                                        let mut flintlock = economy::Item::new(
                                                            "flintlock_pistol",
                                                            "Flintlock Pistol",
                                                            economy::ItemType::Weapon,
                                                            50,
                                                        );
                                                        flintlock.rarity = economy::Rarity::Uncommon;
                                                        let _ = chest.add_item(flintlock);
                                                    }

                                                    // Last two chests get hatchets
                                                    if i >= 2 {
                                                        let mut hatchet = economy::Item::new(
                                                            "hatchet",
                                                            "Hatchet",
                                                            economy::ItemType::Weapon,
                                                            40,
                                                        );
                                                        hatchet.rarity = economy::Rarity::Common;
                                                        let _ = chest.add_item(hatchet);
                                                    }
                                                }
                                            }

                                            // Scatter weapons throughout spawn area
                                            let weapon_spawns = [
                                                // Daggers scattered around spawn
                                                (base_x + 2.0, base_z + 3.0, "dagger"),
                                                (base_x - 4.0, base_z - 5.0, "dagger"),
                                                (base_x + 7.0, base_z + 10.0, "dagger"),
                                                (base_x - 6.0, base_z + 12.0, "dagger"),
                                                // Flintlocks
                                                (base_x + 5.0, base_z - 3.0, "flintlock_pistol"),
                                                (base_x - 3.0, base_z + 8.0, "flintlock_pistol"),
                                                // Hatchets
                                                (base_x + 10.0, base_z - 8.0, "hatchet"),
                                                (base_x - 8.0, base_z - 15.0, "hatchet"),
                                                (base_x + 12.0, base_z + 18.0, "hatchet"),
                                                (base_x - 10.0, base_z + 20.0, "hatchet"),
                                            ];

                                            for (wx, wz, weapon_type) in weapon_spawns {
                                                let (height, _) = get_height_at(wx, wz, seed);
                                                // Use actual terrain height (same as player gravity)
                                                // Skip if underwater
                                                if height < 1.0 { continue; }
                                                let drop_pos = Vec3::new(wx, height + 0.05, wz); // Tiny offset for z-fighting

                                                let weapon = match weapon_type {
                                                    "dagger" => {
                                                        let mut d = economy::Item::new(
                                                            "dagger",
                                                            "Dagger",
                                                            economy::ItemType::Weapon,
                                                            35,
                                                        );
                                                        d.rarity = economy::Rarity::Common;
                                                        d
                                                    }
                                                    "hatchet" => {
                                                        let mut h = economy::Item::new(
                                                            "hatchet",
                                                            "Hatchet",
                                                            economy::ItemType::Weapon,
                                                            40,
                                                        );
                                                        h.rarity = economy::Rarity::Uncommon;
                                                        h
                                                    }
                                                    _ => {
                                                        let mut f = economy::Item::new(
                                                            "flintlock_pistol",
                                                            "Flintlock Pistol",
                                                            economy::ItemType::Weapon,
                                                            50,
                                                        );
                                                        f.rarity = economy::Rarity::Uncommon;
                                                        f
                                                    }
                                                };
                                                state.dropped_items.spawn_drop(weapon, drop_pos);
                                                println!("[WEAPON] Spawned {} at ({:.1}, {:.1}, {:.1})", weapon_type, wx, drop_pos.y, wz);
                                            }
                                            println!("[STORAGE] Spawned {} starter chests and {} weapons on beach", chest_offsets.len(), weapon_spawns.len());
                                        }
                                        // register_village_factions(&mut *state);
                                    }
                                    "Load Game" => {
                                        state.show_load_menu = true;
                                    }
                                    "Exit" => {
                                        println!("[EXIT] User requested exit from menu");
                                        std::process::exit(0);
                                    }
                                    _ => {}
                                }
                            }

                            y_offset += 70.0;
                        }

                        // Load Game submenu (if active)
                        if state.show_load_menu && has_saves {
                            let submenu_x = menu_x - 250.0;
                            let submenu_y = menu_y + 140.0; // Align with Load Game (3rd item, 70px spacing)

                            // Semi-transparent background for submenu
                            let submenu_bg = egui::Rect::from_min_size(
                                egui::pos2(submenu_x - 15.0, submenu_y - 15.0),
                                egui::vec2(230.0, saves.len() as f32 * 45.0 + 30.0),
                            );
                            ui.painter().rect_filled(
                                submenu_bg,
                                8.0,
                                egui::Color32::from_rgba_unmultiplied(255, 248, 230, 230),
                            );

                            let mut save_y = submenu_y;
                            for save_name in &saves {
                                let save_rect = egui::Rect::from_min_size(
                                    egui::pos2(submenu_x, save_y),
                                    egui::vec2(200.0, 40.0),
                                );
                                let response = ui.allocate_rect(save_rect, egui::Sense::click());
                                let text_color = if response.hovered() {
                                    hover_color
                                } else {
                                    menu_color
                                };

                                ui.painter().text(
                                    egui::pos2(submenu_x, save_y + 20.0),
                                    egui::Align2::LEFT_CENTER,
                                    save_name,
                                    egui::FontId::new(22.0, egui::FontFamily::Proportional),
                                    text_color,
                                );

                                if response.clicked() {
                                    if let Some(data) = load_game(save_name) {
                                        state.seed = data.seed;
                                        state.inventory = data.inventory;
                                        state.player.position = Vec3::from_array(data.player_pos);
                                        state.player.yaw = data.player_rot[0];
                                        state.player.pitch = data.player_rot[1];
                                        // Restore NPC relationships if present
                                        if let Some(relationships) = data.npc_relationships {
                                            state.npc_manager.relationships = relationships;
                                        }
                                        state.game_state = GameState::Loading;
                                        state.save_name_input = save_name.clone();
                                        state.show_load_menu = false;
                                        let range = 3;
                                        let total = saturating_range_area(range);
                                        state.loading_progress = LoadingProgress {
                                            total_chunks: total,
                                            chunks_generated: 0,
                                            chunks_uploaded: 0,
                                            current_status: "Loading saved world...".to_string(),
                                        };
                                        if let Some(manager) = CHUNK_MANAGER.get() {
                                            let mut mgr = manager.safe_lock();
                                            mgr.loaded_chunks.clear();
                                            mgr.loading_chunks.clear();
                                        }
                                        // Village system disabled for now
                                        state.village_manager = VillageManager::new(data.seed);
                                        // state.village_manager.discover_villages(
                                        //     state.player.position,
                                        //     2000.0,
                                        //     10,
                                        // );
                                        // Initialize world features (rivers and caves)
                                        state.world_features = WorldFeatures::new(data.seed);
                                        // register_village_factions(&mut *state);
                                    }
                                }
                                save_y += 45.0;
                            }

                            // Close submenu if clicking elsewhere
                            if ui.input(|i| i.pointer.any_click()) && !submenu_bg.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default())) {
                                let load_btn_rect = egui::Rect::from_min_size(
                                    egui::pos2(menu_x, menu_y + 140.0),
                                    egui::vec2(menu_width, 65.0),
                                );
                                if !load_btn_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default())) {
                                    state.show_load_menu = false;
                                }
                            }
                        }
                    });
                }
                GameState::Playing => {
                    // Initialize inventory icon textures if not already done
                    if !state.icon_cache.initialized {
                        inventory_icons::initialize_icons(
                            &mut state.icon_cache,
                            ctx.device(),
                            ctx.queue(),
                            ui_ctx,
                        );
                    }

                    // Update icon rotation each frame for animation
                    inventory_icons::update_icons(
                        &mut state.icon_cache,
                        ctx.device(),
                        ctx.queue(),
                        ui_ctx,
                        delta,
                    );

                    // === HUD: Top-left player stats ===
                    egui::Area::new(egui::Id::new("hud_stats"))
                        .fixed_pos(egui::pos2(10.0, 10.0))
                        .show(ui_ctx, |ui| {
                            ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                            let bg = egui::Frame::none()
                                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150))
                                .rounding(egui::Rounding::same(5.0))
                                .inner_margin(egui::Margin::same(8.0));
                            bg.show(ui, |ui| {
                                // Health bar (placeholder - use progression's player_health when available)
                                let health = 100.0_f32; // TODO: Wire to actual player health
                                let max_health = 100.0_f32;
                                let health_pct = health / max_health;
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("♥").color(egui::Color32::RED).size(16.0));
                                    let bar_size = egui::vec2(100.0, 12.0);
                                    let (rect, _) = ui.allocate_exact_size(bar_size, egui::Sense::hover());
                                    ui.painter().rect_filled(rect, 2.0, egui::Color32::DARK_RED);
                                    let mut filled = rect;
                                    filled.set_right(rect.left() + rect.width() * health_pct);
                                    ui.painter().rect_filled(filled, 2.0, egui::Color32::RED);
                                    ui.label(format!("{:.0}/{:.0}", health, max_health));
                                });
                                // Currency (access wallet fields directly)
                                ui.horizontal(|ui| {
                                    let wampum = state.player_economy.wallet.wampum;
                                    let tobacco = state.player_economy.wallet.tobacco;
                                    ui.label(egui::RichText::new("◎").color(egui::Color32::from_rgb(180, 180, 255)).size(14.0));
                                    ui.label(format!("{}", wampum));
                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new("⚘").color(egui::Color32::from_rgb(139, 90, 43)).size(14.0));
                                    ui.label(format!("{}", tobacco));
                                });
                            });
                        });

                    // === HUD: Minimal center crosshair (dot) ===
                    {
                        // Use screen rect center for true center positioning
                        let screen_center = ui_ctx.screen_rect().center();
                        let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180);

                        // Minimal dot crosshair - just a small circle
                        ui_ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("crosshair")))
                            .circle_filled(screen_center, 2.0, color);
                    }

                    // === HUD: Bottom-center hotbar ===
                    egui::Area::new(egui::Id::new("hud_hotbar"))
                        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
                        .show(ui_ctx, |ui| {
                            let bg = egui::Frame::none()
                                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150))
                                .rounding(egui::Rounding::same(5.0))
                                .inner_margin(egui::Margin::same(5.0));
                            bg.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for slot in 0..10 {
                                        let is_active = slot == state.active_hotbar_slot;
                                        let slot_bg = if is_active {
                                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)
                                        } else {
                                            egui::Color32::from_rgba_unmultiplied(50, 50, 50, 200)
                                        };
                                        let frame = egui::Frame::none()
                                            .fill(slot_bg)
                                            .stroke(if is_active {
                                                egui::Stroke::new(2.0, egui::Color32::GOLD)
                                            } else {
                                                egui::Stroke::new(1.0, egui::Color32::GRAY)
                                            })
                                            .rounding(egui::Rounding::same(3.0))
                                            .inner_margin(egui::Margin::same(4.0));
                                        frame.show(ui, |ui| {
                                            ui.set_min_size(egui::vec2(36.0, 36.0));
                                            if let Some(item) = state.player_economy.inventory.get_slot(slot) {
                                                let rarity_color = match item.rarity {
                                                    economy::Rarity::Crude => egui::Color32::GRAY,
                                                    economy::Rarity::Common => egui::Color32::WHITE,
                                                    economy::Rarity::Uncommon => egui::Color32::GREEN,
                                                    economy::Rarity::Rare => egui::Color32::from_rgb(0, 112, 221),
                                                    economy::Rarity::Epic => egui::Color32::from_rgb(163, 53, 238),
                                                    economy::Rarity::Legendary => egui::Color32::from_rgb(255, 128, 0),
                                                    economy::Rarity::Mythic => egui::Color32::from_rgb(230, 30, 30),
                                                    economy::Rarity::Primordial => egui::Color32::from_rgb(255, 215, 0),
                                                };
                                                // Try to show 3D model icon if available
                                                if let Some(tex) = state.icon_cache.textures.get(&item.template_id) {
                                                    let size = egui::vec2(28.0, 28.0);
                                                    ui.add(egui::Image::new((tex.id(), size)).tint(rarity_color));
                                                } else {
                                                    // Fallback to text icon
                                                    let icon = item.name.chars().next().unwrap_or('?');
                                                    ui.label(egui::RichText::new(icon.to_string()).color(rarity_color).size(18.0));
                                                }
                                                // Stack count
                                                if item.stack_size > 1 {
                                                    ui.label(egui::RichText::new(format!("x{}", item.stack_size)).size(10.0).color(egui::Color32::LIGHT_GRAY));
                                                }
                                            } else {
                                                // Empty slot - show slot number
                                                let key = if slot == 9 { "0".to_string() } else { (slot + 1).to_string() };
                                                ui.label(egui::RichText::new(key).color(egui::Color32::DARK_GRAY).size(12.0));
                                            }
                                        });
                                    }
                                });
                            });
                        });

                    // === "E to interact" prompt when looking at NPC ===
                    // Glassmorphic futuristic tile style
                    if state.current_dialogue.is_none() {
                        if let Some((name, role, distance)) = state.village_manager.get_focused_npc_info() {
                            egui::Area::new(egui::Id::new("npc_interact_prompt"))
                                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 80.0))
                                .show(ui_ctx, |ui| {
                                    // Glassmorphic frame: low opacity, subtle border, rounded
                                    let bg = egui::Frame::none()
                                        .fill(egui::Color32::from_rgba_unmultiplied(10, 20, 30, 140))
                                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 180, 255, 100)))
                                        .rounding(egui::Rounding::same(12.0))
                                        .inner_margin(egui::Margin::symmetric(16.0, 10.0));
                                    bg.show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Icon placeholder (diamond shape)
                                            ui.label(egui::RichText::new("◇")
                                                .color(egui::Color32::from_rgb(100, 180, 255))
                                                .size(16.0));
                                            ui.add_space(6.0);
                                            ui.vertical(|ui| {
                                                ui.label(egui::RichText::new(format!("{}", name))
                                                    .color(egui::Color32::WHITE)
                                                    .size(14.0));
                                                ui.label(egui::RichText::new(format!("{} · {:.1}m", role, distance))
                                                    .color(egui::Color32::from_rgba_unmultiplied(180, 180, 180, 200))
                                                    .size(10.0));
                                            });
                                            ui.add_space(12.0);
                                            ui.label(egui::RichText::new("[E]")
                                                .color(egui::Color32::from_rgb(100, 200, 255))
                                                .size(12.0)
                                                .strong());
                                        });
                                    });
                                });
                        }
                    }

                    // === "E to open" prompt for nearby chest ===
                    if state.open_chest_id.is_none() && state.current_dialogue.is_none() {
                        if let Some(chest) = state.storage_manager.nearest_interactable(
                            state.player.position,
                            None,
                            3.5,
                        ) {
                            let chest_name = chest.display_name().to_string();
                            let distance = chest.distance_from(state.player.position);
                            egui::Area::new(egui::Id::new("chest_interact_prompt"))
                                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 80.0))
                                .show(ui_ctx, |ui| {
                                    // Glassmorphic frame: warm amber tint for chests
                                    let bg = egui::Frame::none()
                                        .fill(egui::Color32::from_rgba_unmultiplied(40, 30, 15, 140))
                                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 180, 80, 100)))
                                        .rounding(egui::Rounding::same(12.0))
                                        .inner_margin(egui::Margin::symmetric(16.0, 10.0));
                                    bg.show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Chest icon
                                            ui.label(egui::RichText::new("▣")
                                                .color(egui::Color32::from_rgb(255, 200, 100))
                                                .size(16.0));
                                            ui.add_space(6.0);
                                            ui.vertical(|ui| {
                                                ui.label(egui::RichText::new(&chest_name)
                                                    .color(egui::Color32::from_rgb(255, 230, 180))
                                                    .size(14.0));
                                                ui.label(egui::RichText::new(format!("{:.1}m", distance))
                                                    .color(egui::Color32::from_rgba_unmultiplied(180, 160, 140, 200))
                                                    .size(10.0));
                                            });
                                            ui.add_space(12.0);
                                            ui.label(egui::RichText::new("[E]")
                                                .color(egui::Color32::from_rgb(255, 200, 100))
                                                .size(12.0)
                                                .strong());
                                        });
                                    });
                                });
                        }
                    }

                    // === Chest UI Window (when open) ===
                    if let Some(chest_id) = state.open_chest_id {
                        // Close chest if player moves too far away
                        let should_close = state.storage_manager.get(chest_id)
                            .map(|c| c.distance_from(state.player.position) > 5.0)
                            .unwrap_or(true);

                        if should_close {
                            if let Some(chest) = state.storage_manager.get_mut(chest_id) {
                                chest.close();
                            }
                            state.open_chest_id = None;
                        } else {
                            // Track which slot was clicked (if any)
                            let mut clicked_slot: Option<usize> = None;

                            egui::Window::new("Chest")
                                .collapsible(false)
                                .resizable(false)
                                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                                .show(ui_ctx, |ui| {
                                    if let Some(chest) = state.storage_manager.get(chest_id) {
                                        ui.label(egui::RichText::new(chest.display_name())
                                            .size(16.0)
                                            .color(egui::Color32::from_rgb(255, 220, 150)));
                                        ui.separator();

                                        // Grid of chest contents
                                        let slot_size = 48.0;
                                        let cols = 5;
                                        let rows = (chest.slots.len() + cols - 1) / cols;

                                        egui::Grid::new("chest_grid")
                                            .spacing([4.0, 4.0])
                                            .show(ui, |ui| {
                                                for row in 0..rows {
                                                    for col in 0..cols {
                                                        let idx = row * cols + col;
                                                        if idx < chest.slots.len() {
                                                            let has_item = chest.slots[idx].is_some();

                                                            // Use allocate_rect with Sense::click for clickable slots
                                                            let (rect, response) = ui.allocate_exact_size(
                                                                egui::vec2(slot_size, slot_size),
                                                                if has_item { egui::Sense::click() } else { egui::Sense::hover() }
                                                            );

                                                            // Check for click
                                                            if has_item && response.clicked() {
                                                                clicked_slot = Some(idx);
                                                            }

                                                            // Slot background (highlight on hover if has item)
                                                            let bg_color = if has_item && response.hovered() {
                                                                egui::Color32::from_rgba_unmultiplied(120, 90, 60, 220)
                                                            } else if has_item {
                                                                egui::Color32::from_rgba_unmultiplied(80, 60, 40, 200)
                                                            } else {
                                                                egui::Color32::from_rgba_unmultiplied(40, 30, 20, 150)
                                                            };
                                                            ui.painter().rect_filled(rect, 4.0, bg_color);
                                                            ui.painter().rect_stroke(rect, 4.0,
                                                                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 80, 60)));

                                                            // Item in slot
                                                            if let Some(item) = &chest.slots[idx] {
                                                                let color = egui::Color32::from_rgb(
                                                                    (item.rarity.color()[0] * 255.0) as u8,
                                                                    (item.rarity.color()[1] * 255.0) as u8,
                                                                    (item.rarity.color()[2] * 255.0) as u8,
                                                                );
                                                                // Try to show 3D model icon if available
                                                                if let Some(tex) = state.icon_cache.textures.get(&item.template_id) {
                                                                    let icon_rect = rect.shrink(4.0);
                                                                    ui.painter().image(
                                                                        tex.id(),
                                                                        icon_rect,
                                                                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                                                        color,
                                                                    );
                                                                } else {
                                                                    // Fallback to item initial
                                                                    let initial = item.name.chars().next().unwrap_or('?');
                                                                    ui.painter().text(
                                                                        rect.center(),
                                                                        egui::Align2::CENTER_CENTER,
                                                                        initial.to_string(),
                                                                        egui::FontId::new(20.0, egui::FontFamily::Monospace),
                                                                        color,
                                                                    );
                                                                }
                                                                // Stack count
                                                                if item.stack_size > 1 {
                                                                    ui.painter().text(
                                                                        rect.right_bottom() - egui::vec2(4.0, 4.0),
                                                                        egui::Align2::RIGHT_BOTTOM,
                                                                        item.stack_size.to_string(),
                                                                        egui::FontId::new(11.0, egui::FontFamily::Monospace),
                                                                        egui::Color32::WHITE,
                                                                    );
                                                                }

                                                                // Tooltip on hover
                                                                if response.hovered() {
                                                                    egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("chest_item_tooltip"), |ui| {
                                                                        ui.label(egui::RichText::new(&item.name).color(color).size(14.0));
                                                                        ui.label(egui::RichText::new(format!("{:?}", item.rarity))
                                                                            .color(egui::Color32::GRAY).size(11.0));
                                                                        if item.stack_size > 1 {
                                                                            ui.label(egui::RichText::new(format!("x{}", item.stack_size))
                                                                                .color(egui::Color32::WHITE).size(11.0));
                                                                        }
                                                                        ui.label(egui::RichText::new("Click to take")
                                                                            .color(egui::Color32::from_rgb(255, 200, 100)).size(10.0));
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ui.end_row();
                                                }
                                            });

                                        ui.separator();
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("[E] Close")
                                                .color(egui::Color32::GRAY)
                                                .size(12.0));
                                            ui.label(egui::RichText::new("  [Click item] Take")
                                                .color(egui::Color32::GRAY)
                                                .size(12.0));
                                        });
                                    }
                                });

                            // Transfer clicked item from chest to player inventory
                            if let Some(slot_idx) = clicked_slot {
                                if let Some(chest) = state.storage_manager.get_mut(chest_id) {
                                    if let Some(item) = chest.remove_item(slot_idx) {
                                        let item_name = item.name.clone();
                                        let rarity = item.rarity;
                                        if let Err(e) = state.player_economy.inventory.add_item(item) {
                                            log::warn!("[CHEST] Failed to add item to inventory: {:?}", e);
                                        } else {
                                            log::info!("[CHEST] Took {} ({:?})", item_name, rarity);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // === Animal Observation HUD (Encyclopedia integration) ===
                    // Show info when player is looking at an animal
                    let look_dir = state.camera.forward();
                    let player_pos_for_obs = state.player.position;
                    let mut focused_animal_for_obs: Option<animals::AnimalSpecies> = None;

                    if let Some((_id, distance, species, behavior)) = state.animal_manager.get_focused_animal(
                        player_pos_for_obs,
                        look_dir,
                        50.0, // Max observation distance
                        0.15, // ~8.5 degree cone (tight focus)
                    ) {
                        // Save species for observation update after UI
                        focused_animal_for_obs = Some(species);

                        // Get encyclopedia info about this species
                        let discovery_tier = state.systems_manager.encyclopedia.get_fauna_tier(species);
                        let observation_count = state.systems_manager.encyclopedia.get_observation_count(species);

                        egui::Area::new(egui::Id::new("animal_observation_hud"))
                            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 60.0))
                            .show(ui_ctx, |ui| {
                                // Glassmorphic frame: green-tinted for fauna
                                let bg = egui::Frame::none()
                                    .fill(egui::Color32::from_rgba_unmultiplied(15, 30, 20, 140))
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(80, 180, 100, 100)))
                                    .rounding(egui::Rounding::same(12.0))
                                    .inner_margin(egui::Margin::symmetric(16.0, 10.0));
                                bg.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Fauna icon
                                        let icon_color = match discovery_tier {
                                            encyclopedia::DiscoveryTier::Unknown => egui::Color32::GRAY,
                                            encyclopedia::DiscoveryTier::Sighted => egui::Color32::from_rgb(120, 150, 120),
                                            encyclopedia::DiscoveryTier::Observed => egui::Color32::from_rgb(100, 200, 100),
                                            encyclopedia::DiscoveryTier::Studied => egui::Color32::from_rgb(150, 220, 150),
                                            encyclopedia::DiscoveryTier::Mastered => egui::Color32::GOLD,
                                        };
                                        ui.label(egui::RichText::new("●")
                                            .color(icon_color)
                                            .size(14.0));
                                        ui.add_space(6.0);
                                        ui.vertical(|ui| {
                                            // Species name
                                            let name_color = match discovery_tier {
                                                encyclopedia::DiscoveryTier::Unknown => egui::Color32::GRAY,
                                                encyclopedia::DiscoveryTier::Sighted => egui::Color32::from_rgb(180, 180, 180),
                                                encyclopedia::DiscoveryTier::Observed => egui::Color32::WHITE,
                                                encyclopedia::DiscoveryTier::Studied => egui::Color32::from_rgb(150, 230, 150),
                                                encyclopedia::DiscoveryTier::Mastered => egui::Color32::GOLD,
                                            };
                                            let display_name = if discovery_tier == encyclopedia::DiscoveryTier::Unknown {
                                                "???".to_string()
                                            } else {
                                                species.name().to_string()
                                            };
                                            ui.label(egui::RichText::new(display_name).color(name_color).size(14.0));

                                            // Compact info line
                                            let behavior_str = if discovery_tier >= encyclopedia::DiscoveryTier::Observed {
                                                format!("{:?} · {:.1}m", behavior, distance)
                                            } else {
                                                format!("{:.1}m", distance)
                                            };
                                            ui.label(egui::RichText::new(behavior_str)
                                                .color(egui::Color32::from_rgba_unmultiplied(150, 170, 150, 200))
                                                .size(10.0));
                                        });
                                    });
                                });
                            });
                    }

                    // Auto-observe focused animal (after UI code to avoid borrow conflict)
                    if let Some(species) = focused_animal_for_obs {
                        state.systems_manager.encyclopedia.on_animal_sighted(species, player_pos_for_obs, false);
                    }

                    // === Flora Foraging HUD ===
                    // Show prompt when near a harvestable plant
                    let player_pos_arr = [state.player.position.x, state.player.position.y, state.player.position.z];
                    let mut focused_flora_for_obs: Option<flora::FloraSpecies> = None;

                    if let Some((_, species, distance, can_harvest)) = state.forageable_plants.get_closest_harvestable(
                        player_pos_arr,
                        4.0,
                    ) {
                        // Save species for observation update after UI
                        focused_flora_for_obs = Some(species);

                        // Get encyclopedia knowledge level
                        let discovery_tier = state.systems_manager.encyclopedia.get_flora_tier(species);

                        egui::Area::new(egui::Id::new("flora_forage_hud"))
                            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 80.0))
                            .show(ui_ctx, |ui| {
                                // Glassmorphic frame: earthy green for flora
                                let bg = egui::Frame::none()
                                    .fill(egui::Color32::from_rgba_unmultiplied(20, 35, 15, 140))
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 180, 80, 100)))
                                    .rounding(egui::Rounding::same(12.0))
                                    .inner_margin(egui::Margin::symmetric(16.0, 10.0));
                                bg.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Flora icon (leaf)
                                        let icon_color = if can_harvest {
                                            egui::Color32::from_rgb(100, 220, 80)
                                        } else {
                                            egui::Color32::from_rgb(150, 120, 80)
                                        };
                                        ui.label(egui::RichText::new("❧")
                                            .color(icon_color)
                                            .size(16.0));
                                        ui.add_space(6.0);
                                        ui.vertical(|ui| {
                                            // Plant name
                                            let name_color = match discovery_tier {
                                                encyclopedia::DiscoveryTier::Unknown => egui::Color32::GRAY,
                                                encyclopedia::DiscoveryTier::Sighted => egui::Color32::from_rgb(180, 180, 180),
                                                encyclopedia::DiscoveryTier::Observed => egui::Color32::WHITE,
                                                encyclopedia::DiscoveryTier::Studied => egui::Color32::from_rgb(150, 230, 150),
                                                encyclopedia::DiscoveryTier::Mastered => egui::Color32::GOLD,
                                            };
                                            let display_name = if discovery_tier == encyclopedia::DiscoveryTier::Unknown {
                                                "???".to_string()
                                            } else {
                                                species.name().to_string()
                                            };
                                            ui.label(egui::RichText::new(display_name).color(name_color).size(14.0));

                                            // Status line
                                            ui.label(egui::RichText::new(format!("{:.1}m", distance))
                                                .color(egui::Color32::from_rgba_unmultiplied(150, 170, 150, 200))
                                                .size(10.0));
                                        });
                                        ui.add_space(12.0);
                                        // Action hint
                                        if can_harvest {
                                            ui.label(egui::RichText::new("[E]")
                                                .color(egui::Color32::from_rgb(100, 220, 80))
                                                .size(12.0)
                                                .strong());
                                        } else {
                                            ui.label(egui::RichText::new("◌")
                                                .color(egui::Color32::from_rgb(150, 100, 80))
                                                .size(12.0));
                                        }
                                    });
                                });
                            });
                    }

                    // Auto-observe focused plant (after UI code to avoid borrow conflict)
                    if let Some(species) = focused_flora_for_obs {
                        state.systems_manager.encyclopedia.on_plant_sighted(species, player_pos_for_obs, false);
                    }

                    // === Dialogue UI ===
                    if let Some(dialogue) = &state.current_dialogue {
                        egui::Area::new(egui::Id::new("dialogue_window"))
                            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -100.0))
                            .show(ui_ctx, |ui| {
                                let bg = egui::Frame::none()
                                    .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 240))
                                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(139, 90, 43)))
                                    .rounding(egui::Rounding::same(10.0))
                                    .inner_margin(egui::Margin::same(16.0));
                                bg.show(ui, |ui| {
                                    ui.set_min_width(500.0);
                                    ui.set_max_width(600.0);

                                    // Speaker name
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(&dialogue.speaker_name)
                                            .color(egui::Color32::from_rgb(255, 220, 150))
                                            .size(20.0)
                                            .strong());
                                        // Relationship status if available
                                        if let Some(rel) = &dialogue.relationship_status {
                                            ui.label(egui::RichText::new(format!("({})", rel.relationship_type))
                                                .color(egui::Color32::GRAY)
                                                .size(12.0));
                                        }
                                    });

                                    ui.add_space(8.0);

                                    // Dialogue text
                                    ui.label(egui::RichText::new(&dialogue.text)
                                        .color(egui::Color32::WHITE)
                                        .size(16.0));

                                    ui.add_space(12.0);
                                    ui.separator();
                                    ui.add_space(8.0);

                                    // Dialogue choices
                                    if dialogue.choices.is_empty() {
                                        ui.label(egui::RichText::new("[E] Continue")
                                            .color(egui::Color32::from_rgb(100, 200, 255))
                                            .size(14.0));
                                    } else {
                                        for choice in &dialogue.choices {
                                            let color = if choice.locked {
                                                egui::Color32::DARK_GRAY
                                            } else if choice.has_effect {
                                                egui::Color32::from_rgb(180, 255, 180)
                                            } else {
                                                egui::Color32::WHITE
                                            };
                                            let key = choice.index + 1;
                                            let prefix = format!("[{}] ", key);
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new(&prefix)
                                                    .color(egui::Color32::from_rgb(100, 200, 255))
                                                    .size(14.0));
                                                ui.label(egui::RichText::new(&choice.text)
                                                    .color(color)
                                                    .size(14.0));
                                                if choice.locked {
                                                    if let Some(reason) = &choice.lock_reason {
                                                        ui.label(egui::RichText::new(format!("({})", reason))
                                                            .color(egui::Color32::from_rgb(200, 100, 100))
                                                            .size(11.0));
                                                    }
                                                }
                                            });
                                        }
                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("[ESC] Leave conversation")
                                            .color(egui::Color32::GRAY)
                                            .size(12.0));
                                    }
                                });
                            });
                    }

                    // === Debug window (F12 to toggle) ===
                    if state.show_debug_ui {
                        egui::Window::new("Debug").show(ui_ctx, |ui| {
                            ui.label(format!("FPS: {:.1}", state.fps));
                            let hours = state.time_of_day as u32;
                            let minutes = ((state.time_of_day - hours as f32) * 60.0) as u32;
                            ui.label(format!("Time: {:02}:{:02}", hours, minutes));
                            ui.label("T/Y keys: Change time");
                            ui.separator();

                            // Dev Stats - Weather/Fog/Render/Network
                            ui.collapsing("Dev Stats", |ui| {
                                // Network Status
                                if state.network.is_online() {
                                    ui.label(format!("Network: {}", state.network.status_string()));
                                    for (_id, player) in state.network.remote_players() {
                                        ui.label(format!("  {} @ ({:.0}, {:.0}, {:.0})",
                                            player.name,
                                            player.position.x,
                                            player.position.y,
                                            player.position.z
                                        ));
                                    }
                                    ui.separator();
                                }

                                ui.label(format!("Weather: {:?}", state.weather.current_weather));
                                ui.label(format!("Render Dist: {:.0}", state.render_distance));
                                let fog = state.atmosphere.fog_params();
                                ui.label(format!("FOG: density={:.2} start={:.0} end={:.0}", fog[0], fog[1], fog[2]));
                                let fog_color = state.atmosphere.fog_color();
                                ui.label(format!("FOG COLOR: ({:.2}, {:.2}, {:.2})", fog_color[0], fog_color[1], fog_color[2]));
                                if let Some(manager) = CHUNK_MANAGER.get() {
                                    if let Ok(mgr) = manager.lock() {
                                        let (loaded, loading) = mgr.get_stats();
                                        ui.label(format!("Chunks: {} loaded, {} loading", loaded, loading));
                                        ui.label(format!("Load radius: {}", mgr.load_radius));
                                    }
                                }
                                // Village System Debug
                                ui.separator();
                                ui.label(format!("VILLAGES: {}", state.village_manager.stats_string()));
                                // Show nearest village
                                for village in &state.village_manager.villages {
                                    let dist = ((village.center.x - state.player.position.x).powi(2)
                                              + (village.center.z - state.player.position.z).powi(2)).sqrt();
                                    ui.label(format!("  {} @ {:.0}m ({} longhouses)",
                                        village.layout.name, dist, village.layout.longhouses.len()));
                                }
                                // Animal System Debug
                                ui.separator();
                                ui.label(state.animal_manager.debug_info());
                                // List nearby animals
                                let nearby = state.animal_manager.animals_near(state.player.position, 50.0);
                                if !nearby.is_empty() {
                                    ui.label(format!("Nearby (50m): {}", nearby.len()));
                                    for animal in nearby.iter().take(5) {
                                        let dist = animal.position.distance(state.player.position);
                                        ui.label(format!("  {} @ {:.0}m - {:?}",
                                            animal.species.name(), dist, animal.behavior_state));
                                    }
                                }
                            });
                            ui.separator();

                            ui.label("Save Name:");
                            ui.text_edit_singleline(&mut state.save_name_input);

                            if ui.button("Save Game").clicked() {
                                let data = SaveData {
                                    seed: state.seed,
                                    player_pos: state.player.position.to_array(),
                                    player_rot: [state.player.yaw, state.player.pitch],
                                    inventory: state.inventory.clone(),
                                    npc_relationships: Some(state.npc_manager.relationships.clone()),
                                };
                                save_game(&state.save_name_input, &data);
                            }
                            if ui.button("Back to Menu").clicked() {
                                state.game_state = GameState::Menu;
                            }
                            ui.label(format!("Camera: {:.1?}", state.camera.position));
                        });
                    }

                    // === Journal Overlay (Tab to toggle) ===
                    if state.perks_journal.is_open {
                        let SharedState { perks_journal, journal_textures, .. } = &mut *state;
                        ui::render_perks_journal(ui_ctx, perks_journal, journal_textures);

                        // Render settings content when Settings tab is active
                        if perks_journal.active_section == ui::JournalSection::Settings {
                            render_journal_settings(ui_ctx, &mut *state);
                        }
                    }
                }
                GameState::Paused => {
                    // Perks Journal also available when paused
                    if state.perks_journal.is_open {
                        let SharedState { perks_journal, journal_textures, .. } = &mut *state;
                        ui::render_perks_journal(ui_ctx, perks_journal, journal_textures);

                        // Render settings content when Settings tab is active
                        if perks_journal.active_section == ui::JournalSection::Settings {
                            render_journal_settings(ui_ctx, &mut *state);
                        }
                    }
                    // Character Sheet uses different layout than normal pause menu
                    else if state.pause_menu_page == PauseMenuPage::CharacterSheet {
                        // Book-style character sheet UI
                        render_character_sheet(ui_ctx, &mut *state);
                    } else {
                    // Pause Menu - Centered Panel
                    egui::CentralPanel::default().show(ui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);
                            ui.heading(egui::RichText::new("PAUSED").size(60.0));
                            ui.add_space(40.0);

                            match state.pause_menu_page {
                                PauseMenuPage::Main => {
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Resume")).clicked() {
                                        state.game_state = GameState::Playing;
                                    }
                                    ui.add_space(10.0);
                                    // Settings moved to Journal (Tab key)
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Journal (Settings)")).clicked() {
                                        state.perks_journal.is_open = true;
                                        state.perks_journal.active_section = ui::JournalSection::Settings;
                                    }
                                    ui.add_space(10.0);
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Controls")).clicked() {
                                        state.pause_menu_page = PauseMenuPage::Controls;
                                    }
                                    ui.add_space(10.0);
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Save Game")).clicked() {
                                        state.show_save_popup = true;
                                    }
                                    ui.add_space(10.0);
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Load Game")).clicked() {
                                        state.pause_menu_page = PauseMenuPage::LoadGame;
                                    }
                                    ui.add_space(10.0);
                                    // Multiplayer button - shows current status
                                    let mp_label = if state.network.is_online() {
                                        format!("Multiplayer ({})", state.network.player_count())
                                    } else {
                                        "Multiplayer".to_string()
                                    };
                                    if ui.add_sized([200.0, 40.0], egui::Button::new(mp_label)).clicked() {
                                        state.pause_menu_page = PauseMenuPage::Network;
                                    }
                                    ui.add_space(10.0);
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Exit to Main Menu")).clicked() {
                                        state.game_state = GameState::Menu;
                                    }
                                }
                                PauseMenuPage::Settings => {
                                    ui.heading("Game Settings");
                                    ui.add_space(30.0);

                                    // Customize slider style for vertical line handles
                                    let mut style = (*ui.ctx().style()).clone();
                                    style.spacing.slider_width = 300.0;
                                    style.visuals.widgets.inactive.bg_stroke.width = 2.0;
                                    style.visuals.widgets.active.bg_stroke.width = 2.0;
                                    ui.ctx().set_style(style);

                                    // Mouse Sensitivity (1-100 scale, minimum 1 so player can always look around)
                                    ui.label(egui::RichText::new("Mouse Sensitivity:").color(egui::Color32::BLACK));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        ui.add(egui::Slider::new(&mut state.mouse_sensitivity, 1.0..=100.0)
                                            .text("Sensitivity")
                                            .custom_formatter(|n, _| format!("{:.0}", n)));
                                    });
                                    ui.add_space(15.0);

                                    // Movement Speed
                                    ui.label(egui::RichText::new("Movement Speed:").color(egui::Color32::BLACK));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        ui.add(egui::Slider::new(&mut state.movement_speed, 1.0..=1000.0)
                                            .text("Speed")
                                            .logarithmic(true)
                                            .custom_formatter(|n, _| format!("{:.0}x", n / 10.0)));
                                    });
                                    ui.add_space(15.0);

                                    // Render Distance - extended range for long-distance fidelity
                                    ui.label(egui::RichText::new("Render Distance:").color(egui::Color32::BLACK));
                                    let old_render_dist = state.render_distance;
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        ui.add(egui::Slider::new(&mut state.render_distance, 150.0..=600.0)
                                            .text("Distance")
                                            .custom_formatter(|n, _| format!("{:.0}", n)));
                                    });
                                    // Update chunk load radius when render distance changes
                                    if (state.render_distance - old_render_dist).abs() > 0.1 {
                                        if let Some(manager) = CHUNK_MANAGER.get() {
                                            if let Ok(mut mgr) = manager.lock() {
                                                mgr.update_radius_for_render_distance(state.render_distance);
                                            }
                                        }
                                    }
                                    ui.add_space(15.0);

                                    // Dither Distance - controls when LOD fading begins
                                    ui.label(egui::RichText::new("Dither Distance:").color(egui::Color32::BLACK));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        let effective_dist = state.render_distance * state.dither_distance_ratio;
                                        ui.add(egui::Slider::new(&mut state.dither_distance_ratio, 0.5..=1.0)
                                            .text("Quality")
                                            .custom_formatter(move |n, _| format!("{:.0}% ({:.0}m)", n * 100.0, effective_dist)));
                                    });
                                    ui.label(egui::RichText::new("Lower = better FPS, shorter sight distance")
                                        .small()
                                        .color(egui::Color32::GRAY));
                                    ui.add_space(15.0);

                                    // Audio Settings Header
                                    ui.label(egui::RichText::new("Audio").size(18.0).strong().color(egui::Color32::BLACK));
                                    ui.add_space(10.0);

                                    // Master Volume (0-100 scale)
                                    ui.label(egui::RichText::new("Master Volume:").color(egui::Color32::BLACK));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        ui.add(egui::Slider::new(&mut state.master_volume, 0.0..=100.0)
                                            .text("Volume")
                                            .custom_formatter(|n, _| format!("{:.0}", n)));
                                    });
                                    // Sync with audio system (copy value first to avoid borrow conflict)
                                    let master_vol = state.master_volume / 100.0;
                                    state.audio_system.set_master_volume(master_vol);
                                    ui.add_space(15.0);

                                    // Music Volume
                                    let mut music_vol = state.audio_system.music_volume * 100.0;
                                    ui.label(egui::RichText::new("Music Volume:").color(egui::Color32::BLACK));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        if ui.add(egui::Slider::new(&mut music_vol, 0.0..=100.0)
                                            .text("Music")
                                            .custom_formatter(|n, _| format!("{:.0}", n))).changed() {
                                            state.audio_system.set_music_volume(music_vol / 100.0);
                                        }
                                    });
                                    ui.add_space(15.0);

                                    // Ambience Volume
                                    let mut amb_vol = state.audio_system.ambience_volume * 100.0;
                                    ui.label(egui::RichText::new("Ambience Volume:").color(egui::Color32::BLACK));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        if ui.add(egui::Slider::new(&mut amb_vol, 0.0..=100.0)
                                            .text("Ambience")
                                            .custom_formatter(|n, _| format!("{:.0}", n))).changed() {
                                            state.audio_system.set_ambience_volume(amb_vol / 100.0);
                                        }
                                    });
                                    ui.add_space(30.0);

                                    // Developer Controls Header
                                    ui.label(egui::RichText::new("Developer").size(18.0).strong().color(egui::Color32::BLACK));
                                    ui.add_space(10.0);

                                    // Time of Day
                                    ui.label(egui::RichText::new("Time of Day:").color(egui::Color32::BLACK));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        ui.add(egui::Slider::new(&mut state.time_of_day, 0.0..=24.0)
                                            .text("Hour")
                                            .custom_formatter(|n, _| {
                                                let h = n as i32;
                                                let m = ((n - h as f64) * 60.0) as i32;
                                                format!("{:02}:{:02}", h % 24, m)
                                            }));
                                    });
                                    ui.add_space(10.0);

                                    // Weather Controls
                                    ui.label(egui::RichText::new("Weather:").color(egui::Color32::BLACK));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        if ui.button("Clear").clicked() {
                                            state.weather.set_weather(WeatherType::Clear, false);
                                        }
                                        if ui.button("Cloudy").clicked() {
                                            state.weather.set_weather(WeatherType::PartlyCloudy, false);
                                        }
                                        if ui.button("Overcast").clicked() {
                                            state.weather.set_weather(WeatherType::Overcast, false);
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        if ui.button("Stormy").clicked() {
                                            state.weather.set_weather(WeatherType::Stormy, false);
                                        }
                                        if ui.button("Foggy").clicked() {
                                            state.weather.set_weather(WeatherType::Foggy, false);
                                        }
                                    });
                                    ui.add_space(5.0);
                                    let current_weather = format!("Current: {:?}", state.weather.current_weather);
                                    ui.label(egui::RichText::new(current_weather).size(12.0).color(egui::Color32::DARK_GRAY));
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        ui.checkbox(&mut state.weather.auto_weather_enabled, "Auto Weather Changes");
                                    });
                                    ui.add_space(30.0);

                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Back")).clicked() {
                                        state.pause_menu_page = PauseMenuPage::Main;
                                        // Apply settings
                                        state.player.speed = state.movement_speed;
                                        let vol = state.master_volume / 100.0;
                                        state.audio_system.set_master_volume(vol);
                                    }
                                }
                                PauseMenuPage::Controls => {
                                    ui.heading("Controls");
                                    ui.add_space(20.0);

                                    ui.label(egui::RichText::new("Movement:").size(18.0).strong().color(egui::Color32::BLACK));
                                    ui.label("W/A/S/D - Move Forward/Left/Back/Right");
                                    ui.label("Shift - Sprint (70% faster)");
                                    ui.label("Space - Jump");
                                    ui.label("Mouse - Look Around");
                                    ui.add_space(15.0);

                                    ui.label(egui::RichText::new("Game Controls:").size(18.0).strong().color(egui::Color32::BLACK));
                                    ui.label("ESC - Pause Menu");
                                    ui.label("Tab/J - Journal (Perks/Stats/Encyclopedia/Settings)");
                                    ui.label("F12 - Toggle Debug UI");
                                    ui.label("T/Y - Change Time of Day (+/- 1 hour)");
                                    ui.label("M - Toggle Audio On/Off");
                                    ui.add_space(10.0);

                                    ui.label(egui::RichText::new("Weather Controls:").size(18.0).strong().color(egui::Color32::BLACK));
                                    ui.label("[ / ] - Cycle Weather");
                                    ui.label("\\ - Cycle Fog Level (Off/Light/Medium/Heavy/Dense)");
                                    ui.add_space(30.0);

                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Back")).clicked() {
                                        state.pause_menu_page = PauseMenuPage::Main;
                                    }
                                }
                                PauseMenuPage::LoadGame => {
                                    ui.heading("Load Game");
                                    ui.add_space(20.0);

                                    // Get list of saves
                                    let saves = list_saves();

                                    if saves.is_empty() {
                                        ui.label(egui::RichText::new("No saved games found").size(18.0).color(egui::Color32::DARK_GRAY));
                                    } else {
                                        ui.label(egui::RichText::new("Select a save to load:").size(16.0).color(egui::Color32::BLACK));
                                        ui.add_space(15.0);

                                        // Display each save as a button
                                        for save_name in saves {
                                            if ui.add_sized([250.0, 40.0], egui::Button::new(&save_name)).clicked() {
                                                // Load the selected save
                                                if let Some(save_data) = load_game(&save_name) {
                                                    println!("[LOAD] Loading save: {}", save_name);
                                                    state.seed = save_data.seed;
                                                    state.player.position = Vec3::from_array(save_data.player_pos);
                                                    state.player.yaw = save_data.player_rot[0];
                                                    state.player.pitch = save_data.player_rot[1];
                                                    state.inventory = save_data.inventory;
                                                    // Restore NPC relationships if present
                                                    if let Some(relationships) = save_data.npc_relationships {
                                                        state.npc_manager.relationships = relationships;
                                                    }

                                                    // Reset camera to match loaded player rotation
                                                    state.camera.yaw = state.player.yaw;
                                                    state.camera.pitch = state.player.pitch;
                                                    state.camera.position = state.player.position + Vec3::new(0.0, 1.6, 0.0);

                                                    // Return to game
                                                    state.game_state = GameState::Playing;
                                                    state.pause_menu_page = PauseMenuPage::Main;
                                                    println!("[LOAD] Save loaded successfully!");
                                                } else {
                                                    println!("[LOAD] Failed to load save: {}", save_name);
                                                }
                                            }
                                            ui.add_space(10.0);
                                        }
                                    }

                                    ui.add_space(20.0);
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Back")).clicked() {
                                        state.pause_menu_page = PauseMenuPage::Main;
                                    }
                                }
                                PauseMenuPage::Network => {
                                    ui.heading("Multiplayer");
                                    ui.add_space(20.0);

                                    // Show current status
                                    let status = state.network.status_string();
                                    ui.label(egui::RichText::new(format!("Status: {}", status)).size(16.0).color(egui::Color32::BLACK));
                                    ui.add_space(20.0);

                                    if state.network.is_online() {
                                        // Show connected players
                                        ui.label(egui::RichText::new("Connected Players:").size(14.0).color(egui::Color32::DARK_GRAY));
                                        ui.add_space(10.0);

                                        // Show self
                                        ui.label("  You (Host)" .to_string());

                                        // Show remote players
                                        for (_id, player) in state.network.remote_players() {
                                            ui.label(format!("  {} at ({:.0}, {:.0}, {:.0})",
                                                player.name,
                                                player.position.x,
                                                player.position.y,
                                                player.position.z
                                            ));
                                        }

                                        ui.add_space(20.0);

                                        // Disconnect button
                                        if ui.add_sized([200.0, 40.0], egui::Button::new("Disconnect")).clicked() {
                                            // Create new offline manager
                                            state.network = network::NetworkManager::offline(state.seed);
                                            println!("[NET] Disconnected from multiplayer");
                                        }
                                    } else {
                                        // Not connected - show host/join options
                                        ui.separator();
                                        ui.add_space(10.0);

                                        // === HOST SECTION ===
                                        ui.label(egui::RichText::new("Host a Game (Open to LAN)").size(18.0).color(egui::Color32::BLACK));
                                        ui.add_space(10.0);

                                        ui.horizontal(|ui| {
                                            ui.label("Port:");
                                            ui.add(egui::TextEdit::singleline(&mut state.network_host_port).desired_width(80.0));
                                        });
                                        ui.add_space(5.0);

                                        if ui.add_sized([200.0, 40.0], egui::Button::new("🌐 Open to LAN")).clicked() {
                                            let port: u16 = state.network_host_port.parse().unwrap_or(7878);
                                            match network::NetworkManager::host(port, state.seed) {
                                                Ok(nm) => {
                                                    state.network = nm;
                                                    println!("[NET] Now hosting on port {}", port);
                                                    println!("[NET] Others can join with: --join <your-ip>:{}", port);
                                                }
                                                Err(e) => {
                                                    eprintln!("[NET] Failed to host: {}", e);
                                                }
                                            }
                                        }

                                        ui.add_space(20.0);
                                        ui.separator();
                                        ui.add_space(10.0);

                                        // === JOIN SECTION ===
                                        ui.label(egui::RichText::new("Join a Game").size(18.0).color(egui::Color32::BLACK));
                                        ui.add_space(10.0);

                                        ui.horizontal(|ui| {
                                            ui.label("Address:");
                                            ui.add(egui::TextEdit::singleline(&mut state.network_join_address).desired_width(150.0));
                                        });
                                        ui.add_space(5.0);

                                        ui.horizontal(|ui| {
                                            ui.label("Name:");
                                            ui.add(egui::TextEdit::singleline(&mut state.network_player_name).desired_width(120.0));
                                        });
                                        ui.add_space(10.0);

                                        if ui.add_sized([200.0, 40.0], egui::Button::new("🔗 Join Game")).clicked() {
                                            let address = state.network_join_address.clone();
                                            let name = state.network_player_name.clone();
                                            match network::NetworkManager::join(&address, &name) {
                                                Ok(nm) => {
                                                    // Update seed from server
                                                    state.seed = nm.seed();
                                                    state.network = nm;
                                                    println!("[NET] Connected to {}", address);
                                                    // Return to game with new seed
                                                    state.game_state = GameState::Playing;
                                                    state.pause_menu_page = PauseMenuPage::Main;
                                                }
                                                Err(e) => {
                                                    eprintln!("[NET] Failed to join: {}", e);
                                                }
                                            }
                                        }
                                    }

                                    ui.add_space(30.0);
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Back")).clicked() {
                                        state.pause_menu_page = PauseMenuPage::Main;
                                    }
                                }
                                PauseMenuPage::CharacterSheet => {
                                    // Handled above, this case won't be reached
                                }
                            }
                        });
                    });

                    // Save Game Popup (appears on top of pause menu)
                    if state.show_save_popup {
                        egui::Window::new("Save Game")
                            .collapsible(false)
                            .resizable(false)
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .show(ui_ctx, |ui| {
                                ui.set_min_width(300.0);
                                ui.vertical_centered(|ui| {
                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new("Enter save name:").size(16.0));
                                    ui.add_space(10.0);

                                    ui.add(egui::TextEdit::singleline(&mut state.save_name_input)
                                        .hint_text("Save name...")
                                        .desired_width(250.0));

                                    ui.add_space(15.0);

                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 220.0) / 2.0);

                                        if ui.add_sized([100.0, 35.0], egui::Button::new("Save")).clicked() {
                                            // Save the current game state with custom name
                                            let save_name = if state.save_name_input.trim().is_empty() {
                                                "quicksave"
                                            } else {
                                                state.save_name_input.trim()
                                            };

                                            let save_data = SaveData {
                                                seed: state.seed,
                                                player_pos: state.player.position.to_array(),
                                                player_rot: [state.player.yaw, state.player.pitch],
                                                inventory: state.inventory.clone(),
                                                npc_relationships: Some(state.npc_manager.relationships.clone()),
                                            };
                                            save_game(save_name, &save_data);
                                            println!("[SAVE] Game saved to {}", save_name);
                                            state.save_name_input.clear();
                                            state.show_save_popup = false;
                                        }

                                        ui.add_space(20.0);

                                        if ui.add_sized([100.0, 35.0], egui::Button::new("Cancel")).clicked() {
                                            state.show_save_popup = false;
                                            state.save_name_input.clear();
                                        }
                                    });

                                    ui.add_space(10.0);
                                });
                            });
                    }
                    } // end else (not CharacterSheet)
                }
            }
        });

        // Handle Pipeline Updates (scoped to release locks early)
        {
            let mut manager = chunk_manager.safe_lock();

            // Update Chunk Streaming (Request new chunks / Unload old ones)
            if state.game_state == GameState::Loading || state.game_state == GameState::Playing {
                let village_centers = state.village_manager.get_village_centers();
                let corn_field_exclusions = state.village_manager.get_corn_field_bounds();
                let camera_forward = state.camera.forward();
                let requests = manager.update(state.player.position, camera_forward, state.seed, &village_centers, &corn_field_exclusions);
                for req in requests {
                    let _ = request_tx.send(req);
                }
                
                // Update Loading Progress stats
                state.loading_progress.chunks_generated = manager.chunk_count(); // Approximation
            }

            // Check for new chunks from background thread
            if let Ok(rx) = render_rx.try_lock() {
                // During Loading: Process ALL available chunks for faster load
                // During Playing: Process 1 chunk per frame to avoid stutter
                let chunks_per_frame = if state.game_state == GameState::Loading { 100 } else { 1 };
                for _ in 0..chunks_per_frame {
                    match rx.try_recv() {
                        Ok((terrain_pos, terrain_col, terrain_nrm, terrain_idx,
                            tree_groups,
                            shrub_groups,
                            det_pos, det_nrm, det_uv, det_idx,
                            rock_instances,
                            building_instances,
                            fern_instances,
                            offset_x, offset_z)) => {

                            // Update status
                            state.loading_progress.current_status = format!(
                                "Uploading chunk at ({}, {})...",
                                offset_x, offset_z
                            );

                            // Calculate bounds
                            let chunk_size = 256.0;
                            let bounds = ChunkBounds::new(
                                offset_x as f32,
                                offset_z as f32,
                                chunk_size,
                                -10.0,
                                50.0,
                            );

                            // Create Pipelines
                            let terrain_pipeline = {
                                let shadow_map = shadow_map_mutex.safe_lock();
                                // Get terrain textures (should be loaded by now)
                                let terrain_textures = state.terrain_textures.as_ref()
                                    .expect("Terrain textures should be loaded before chunks");
                                TerrainPipeline::new(
                                    ctx.device(),
                                    ctx.surface_format(),
                                    &terrain_pos, &terrain_col, &terrain_nrm, &terrain_idx,
                                    &shadow_map,
                                    terrain_textures.as_ref()
                                )
                            };

                            // Procedural grass pipeline DISABLED - using grass2/grass3 models instead
                            let grass_pipeline: Option<GrassPipeline> = None;

                            // FOLIAGE: Create pipelines for trees and shrubs
                            let mut foliage_pipelines: Vec<TreePipeline> = Vec::new();
                            // tree_groups is already HashMap<String, Vec<Mat4>> from foliage_gen.rs
                            // Debug: show registry contents when processing first chunk
                            if state.mesh_registry.len() < 20 {
                                log::debug!("mesh_registry has {} entries: {:?}",
                                    state.mesh_registry.len(), state.mesh_registry.keys().collect::<Vec<_>>());
                            }
                            // Acquire shadow map once for all tree pipelines
                            let shadow_map = shadow_map_mutex.safe_lock();
                            for (name, transforms) in &tree_groups {
                                // Render bark mesh (OPAQUE)
                                let bark_name = format!("{}_bark", name);
                                if let Some(mesh) = state.mesh_registry.get(&bark_name) {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map);
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), transforms);
                                    foliage_pipelines.push(tp);
                                }
                                // Render leaves mesh (BLEND) with same transforms
                                let leaves_name = format!("{}_leaves", name);
                                if let Some(mesh) = state.mesh_registry.get(&leaves_name) {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map);
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), transforms);
                                    foliage_pipelines.push(tp);
                                }
                            }
                            drop(shadow_map);

                            // SHRUBS: Separate conifer_shrub_0 for LOD handling, others go to foliage_pipelines
                            let shadow_map_shrubs = shadow_map_mutex.safe_lock();
                            let mut conifer_shrub_transforms: Vec<Mat4> = Vec::new();

                            for (name, transforms) in &shrub_groups {
                                if name == "conifer_shrub_0" {
                                    // Collect conifer shrub transforms for LOD handling
                                    conifer_shrub_transforms.extend(transforms.iter().cloned());
                                } else {
                                    // Other shrubs (beach_grass_0) render without LOD
                                    let bark_name = format!("{}_bark", name);
                                    if let Some(mesh) = state.mesh_registry.get(&bark_name) {
                                        let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_shrubs);
                                        tp.set_mesh(mesh.clone());
                                        tp.upload_instances(ctx.device(), transforms);
                                        foliage_pipelines.push(tp);
                                    }
                                    let leaves_name = format!("{}_leaves", name);
                                    if let Some(mesh) = state.mesh_registry.get(&leaves_name) {
                                        let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_shrubs);
                                        tp.set_mesh(mesh.clone());
                                        tp.upload_instances(ctx.device(), transforms);
                                        foliage_pipelines.push(tp);
                                    }
                                }
                            }

                            // Create conifer_shrub LOD pipelines
                            let mut conifer_shrubs_lod0 = Vec::new();
                            let mut conifer_shrubs_lod1 = Vec::new();
                            let mut conifer_shrubs_lod2 = Vec::new();

                            if !conifer_shrub_transforms.is_empty() {
                                for (lod, pipelines) in [(0, &mut conifer_shrubs_lod0), (1, &mut conifer_shrubs_lod1), (2, &mut conifer_shrubs_lod2)] {
                                    let mesh_name = if lod == 0 {
                                        "conifer_shrub_0".to_string()
                                    } else {
                                        format!("conifer_shrub_0_lod{}", lod)
                                    };
                                    // Try bark mesh first, then leaves
                                    let bark_name = format!("{}_bark", mesh_name);
                                    let leaves_name = format!("{}_leaves", mesh_name);

                                    if let Some(mesh) = state.mesh_registry.get(&bark_name) {
                                        let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_shrubs);
                                        tp.set_mesh(mesh.clone());
                                        tp.upload_instances(ctx.device(), &conifer_shrub_transforms);
                                        pipelines.push(tp);
                                    }
                                    if let Some(mesh) = state.mesh_registry.get(&leaves_name) {
                                        let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_shrubs);
                                        tp.set_mesh(mesh.clone());
                                        tp.upload_instances(ctx.device(), &conifer_shrub_transforms);
                                        pipelines.push(tp);
                                    }
                                    // Try single mesh (no bark/leaves split)
                                    if let Some(mesh) = state.mesh_registry.get(&mesh_name) {
                                        let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_shrubs);
                                        tp.set_mesh(mesh.clone());
                                        tp.upload_instances(ctx.device(), &conifer_shrub_transforms);
                                        pipelines.push(tp);
                                    }
                                }
                            }

                            drop(shadow_map_shrubs);

                            // Create LOD1 pipelines (scaled down transforms for reduced visual volume)
                            // LOD1 now uses separate bark/leaves like LOD0
                            let mut foliage_pipelines_lod1: Vec<TreePipeline> = Vec::new();
                            let shadow_map_lod1 = shadow_map_mutex.safe_lock();
                            for (name, transforms) in &tree_groups {
                                // Scale down LOD1 trees to 70% size for reduced poly volume at distance
                                let scaled_transforms: Vec<Mat4> = transforms.iter().map(|t| {
                                    let (scale, rot, trans) = t.to_scale_rotation_translation();
                                    Mat4::from_scale_rotation_translation(scale * 0.7, rot, trans)
                                }).collect();

                                // Try bark mesh first
                                let bark_name = format!("{}_lod1_bark", name);
                                if let Some(mesh) = state.mesh_registry.get(&bark_name) {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_lod1);
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), &scaled_transforms);
                                    foliage_pipelines_lod1.push(tp);
                                }

                                // Then leaves mesh
                                let leaves_name = format!("{}_lod1_leaves", name);
                                if let Some(mesh) = state.mesh_registry.get(&leaves_name) {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_lod1);
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), &scaled_transforms);
                                    foliage_pipelines_lod1.push(tp);
                                }
                            }
                            drop(shadow_map_lod1);

                            // Create LOD2 pipelines (billboard/simplified for distant rendering)
                            // LOD2 now uses separate bark/leaves meshes like LOD0/LOD1 for proper texturing
                            let mut foliage_pipelines_lod2: Vec<TreePipeline> = Vec::new();
                            let shadow_map_lod2 = shadow_map_mutex.safe_lock();
                            for (name, transforms) in &tree_groups {
                                // Scale down LOD2 trees to 50% size for distant silhouettes
                                let scaled_transforms: Vec<Mat4> = transforms.iter().map(|t| {
                                    let (scale, rot, trans) = t.to_scale_rotation_translation();
                                    Mat4::from_scale_rotation_translation(scale * 0.5, rot, trans)
                                }).collect();

                                // LOD2 uses separate bark and leaves meshes (e.g., "birch_0_lod2_bark", "birch_0_lod2_leaves")
                                let bark_name = format!("{}_lod2_bark", name);
                                let leaves_name = format!("{}_lod2_leaves", name);

                                // Create bark pipeline if mesh exists
                                if let Some(mesh) = state.mesh_registry.get(&bark_name) {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_lod2);
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), &scaled_transforms);
                                    foliage_pipelines_lod2.push(tp);
                                }

                                // Create leaves pipeline if mesh exists
                                if let Some(mesh) = state.mesh_registry.get(&leaves_name) {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_lod2);
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), &scaled_transforms);
                                    foliage_pipelines_lod2.push(tp);
                                }
                            }
                            drop(shadow_map_lod2);

                            let mut detritus_pipeline = None;
                            if !det_pos.is_empty() {
                                let mut dp = DetritusPipeline::new(ctx.device(), ctx.surface_format());
                                dp.upload_mesh(ctx.device(), ctx.queue(), &det_pos, &det_nrm, &det_uv, &det_idx);
                                detritus_pipeline = Some(dp);
                            }

                            // Group rocks by type, separating boulders and dead_logs for LOD handling
                            let mut rock_groups: std::collections::HashMap<String, Vec<Mat4>> = std::collections::HashMap::new();
                            let mut boulder_transforms: Vec<Mat4> = Vec::new();
                            let mut dead_log_transforms: Vec<Mat4> = Vec::new();

                            for (name, transform) in rock_instances {
                                if name == "rock_boulder" {
                                    boulder_transforms.push(transform);
                                } else if name == "dead_log_0" {
                                    dead_log_transforms.push(transform);
                                } else {
                                    rock_groups.entry(name).or_default().push(transform);
                                }
                            }

                            // Debug: Show rock type breakdown
                            // Create pipelines for non-boulder rocks
                            let mut rock_pipelines = Vec::new();
                            let shadow_map = shadow_map_mutex.safe_lock();
                            for (name, transforms) in rock_groups {
                                if let Some(mesh) = state.mesh_registry.get(&name) {
                                    let mut rp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map);
                                    rp.set_mesh(mesh.clone());
                                    rp.upload_instances(ctx.device(), &transforms);
                                    rock_pipelines.push(rp);
                                }
                            }

                            // Create boulder LOD pipelines (all share same transforms, different meshes)
                            let mut boulders_lod0 = Vec::new();
                            let mut boulders_lod1 = Vec::new();
                            let mut boulders_lod2 = Vec::new();

                            if !boulder_transforms.is_empty() {
                                for (lod, pipelines) in [(0, &mut boulders_lod0), (1, &mut boulders_lod1), (2, &mut boulders_lod2)] {
                                    let mesh_name = format!("boulder_lod{}", lod);
                                    if let Some(mesh) = state.mesh_registry.get(&mesh_name) {
                                        let mut bp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map);
                                        bp.set_mesh(mesh.clone());
                                        bp.upload_instances(ctx.device(), &boulder_transforms);
                                        pipelines.push(bp);
                                    }
                                }
                            }

                            // Create dead_log LOD pipelines
                            let mut dead_logs_lod0 = Vec::new();
                            let mut dead_logs_lod1 = Vec::new();
                            let mut dead_logs_lod2 = Vec::new();

                            if !dead_log_transforms.is_empty() {
                                for (lod, pipelines) in [(0, &mut dead_logs_lod0), (1, &mut dead_logs_lod1), (2, &mut dead_logs_lod2)] {
                                    let mesh_name = if lod == 0 {
                                        "dead_log_0".to_string()
                                    } else {
                                        format!("dead_log_0_lod{}", lod)
                                    };
                                    if let Some(mesh) = state.mesh_registry.get(&mesh_name) {
                                        let mut dlp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map);
                                        dlp.set_mesh(mesh.clone());
                                        dlp.upload_instances(ctx.device(), &dead_log_transforms);
                                        pipelines.push(dlp);
                                    }
                                }
                            }

                            drop(shadow_map);

                            // Process Buildings
                            let mut building_pipelines = Vec::new();
                            let mut buildings_by_type: std::collections::HashMap<String, Vec<Mat4>> = std::collections::HashMap::new();
                            for (name, transform) in building_instances {
                                buildings_by_type.entry(name).or_default().push(transform);
                            }

                            // Get shadow map for binding to building pipelines
                            let shadow_map_for_buildings = shadow_map_mutex.safe_lock();
                            for (name, transforms) in buildings_by_type {
                                if let Some(mesh) = state.building_registry.get(&name) {
                                    let mut pipeline = BuildingPipeline::new(ctx.device(), ctx.surface_format());
                                    pipeline.set_mesh(mesh.clone());
                                    pipeline.upload_instances(ctx.device(), &transforms);
                                    // Bind shadow map for shadow rendering
                                    pipeline.bind_shadow_map(ctx.device(), &shadow_map_for_buildings.view, &shadow_map_for_buildings.sampler);
                                    building_pipelines.push(pipeline);
                                } else {
                                    println!("[WARN] Building mesh '{}' not found in registry", name);
                                }
                            }
                            drop(shadow_map_for_buildings);

                            // Process Village Structures
                            let village_structures = state.village_manager.get_structures_for_chunk(
                                offset_x as f32,
                                offset_z as f32,
                                chunk_size,
                            );

                            if !village_structures.is_empty() {
                                println!("[VILLAGE] Processing {} structures for chunk ({}, {})",
                                    village_structures.len(), offset_x, offset_z);
                            }

                            for structure in village_structures {
                                // Convert flattened vertex data back to BuildingVertex
                                // Each vertex is 11 floats: pos[3], normal[3], uv[2], color[3]
                                let num_vertices = structure.mesh_vertices.len() / 11;
                                let mut vertices = Vec::with_capacity(num_vertices);

                                for i in 0..num_vertices {
                                    let base = i * 11;
                                    vertices.push(BuildingVertex {
                                        position: [
                                            structure.mesh_vertices[base],
                                            structure.mesh_vertices[base + 1],
                                            structure.mesh_vertices[base + 2],
                                        ],
                                        normal: [
                                            structure.mesh_vertices[base + 3],
                                            structure.mesh_vertices[base + 4],
                                            structure.mesh_vertices[base + 5],
                                        ],
                                        uv: [
                                            structure.mesh_vertices[base + 6],
                                            structure.mesh_vertices[base + 7],
                                        ],
                                        color: [
                                            structure.mesh_vertices[base + 8],
                                            structure.mesh_vertices[base + 9],
                                            structure.mesh_vertices[base + 10],
                                        ],
                                    });
                                }

                                // Create mesh and pipeline for this structure
                                let mesh = BuildingPipeline::create_mesh(
                                    ctx.device(),
                                    &vertices,
                                    &structure.mesh_indices,
                                );

                                let mut pipeline = BuildingPipeline::new(ctx.device(), ctx.surface_format());
                                pipeline.set_mesh(mesh);
                                pipeline.upload_instances(ctx.device(), &[structure.transform]);
                                // Bind shadow map for village structures
                                let shadow_map_village = shadow_map_mutex.safe_lock();
                                pipeline.bind_shadow_map(ctx.device(), &shadow_map_village.view, &shadow_map_village.sampler);
                                drop(shadow_map_village);
                                building_pipelines.push(pipeline);
                            }

                            // Process Ferns (forest understory)
                            let mut fern_pipelines = Vec::new();
                            let mut ferns_by_type: std::collections::HashMap<String, Vec<Mat4>> = std::collections::HashMap::new();
                            for (name, transform) in fern_instances {
                                ferns_by_type.entry(name).or_default().push(transform);
                            }

                            let shadow_map_for_ferns = shadow_map_mutex.safe_lock();
                            for (name, transforms) in ferns_by_type {
                                if let Some(mesh) = state.mesh_registry.get(&name) {
                                    let mut fp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_for_ferns);
                                    fp.set_mesh(mesh.clone());
                                    fp.upload_instances(ctx.device(), &transforms);
                                    fern_pipelines.push(fp);
                                }
                            }
                            drop(shadow_map_for_ferns);

                            // ================================================================
                            // GRASS2: Inland/meadow/forest ground cover
                            // ================================================================
                            let mut grass2_lod0_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut grass2_lod1_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut grass2_lod2_pipelines: Vec<TreePipeline> = Vec::new();

                            let mut grass2_transforms: Vec<Mat4> = Vec::new();

                            // Use deterministic seed based on chunk position
                            let grass2_seed = state.seed.wrapping_add(5555) ^ (offset_x as u32) ^ ((offset_z as u32) << 16);
                            use rand::SeedableRng;
                            let mut rng_grass2 = rand::rngs::StdRng::seed_from_u64(grass2_seed as u64);

                            // Ground cover spacing (wider = fewer instances = better perf)
                            // Increased from 4.0 to 25.0 = 90% instance reduction
                            let grass2_spacing = 25.0;

                            let grass2_steps = (chunk_size / grass2_spacing) as i32;
                            for gz in 0..grass2_steps {
                                for gx in 0..grass2_steps {
                                    use rand::Rng;
                                    let jitter_x = (rng_grass2.gen::<f32>() - 0.5) * grass2_spacing * 0.8;
                                    let jitter_z = (rng_grass2.gen::<f32>() - 0.5) * grass2_spacing * 0.8;

                                    let world_x = offset_x as f32 + (gx as f32 * grass2_spacing) + jitter_x;
                                    let world_z = offset_z as f32 + (gz as f32 * grass2_spacing) + jitter_z;

                                    let (height, _color) = croatoan_wfc::get_height_at(world_x, world_z, state.seed);
                                    let biome_t = croatoan_wfc::get_biome_t(world_x, world_z, state.seed);

                                    // Skip underwater and beach (grass3 handles beach)
                                    if height < 2.5 || biome_t < 0.60 {
                                        continue;
                                    }

                                    // Density by zone - sparse near beach, denser inland:
                                    // Near beach (0.60-0.68): gradient 20-60%
                                    // Meadow/treeline (0.68-0.72): 70% density
                                    // Forest floor (0.72+): 50% density (shade)
                                    let spawn_chance = if biome_t < 0.68 {
                                        // Gradient: 20% at 0.60, 60% at 0.68
                                        let t = (biome_t - 0.60) / 0.08;
                                        0.20 + t * 0.40
                                    } else if biome_t < 0.72 {
                                        0.70
                                    } else {
                                        0.50
                                    };

                                    if rng_grass2.gen::<f32>() > spawn_chance {
                                        continue;
                                    }

                                    let rotation = rng_grass2.gen::<f32>() * std::f32::consts::TAU;
                                    let scale = 1.4 + rng_grass2.gen::<f32>() * 0.8; // 1.4-2.2 (larger clumps)

                                    // DEBUG: +0.3 offset to test - remove once model issue found
                                    let transform = Mat4::from_scale_rotation_translation(
                                        Vec3::splat(scale),
                                        glam::Quat::from_rotation_y(rotation),
                                        Vec3::new(world_x, height + 0.3, world_z),
                                    );
                                    grass2_transforms.push(transform);
                                }
                            }

                            // Create LOD pipelines for grass2
                            if !grass2_transforms.is_empty() {
                                let shadow_map_g2 = shadow_map_mutex.safe_lock();

                                if let Some(lod0_mesh) = state.mesh_registry.get("grass2_lod0") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_g2);
                                    p.set_mesh(lod0_mesh.clone());
                                    p.upload_instances(ctx.device(), &grass2_transforms);
                                    grass2_lod0_pipelines.push(p);
                                }

                                if let Some(lod1_mesh) = state.mesh_registry.get("grass2_lod1") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_g2);
                                    p.set_mesh(lod1_mesh.clone());
                                    p.upload_instances(ctx.device(), &grass2_transforms);
                                    grass2_lod1_pipelines.push(p);
                                }

                                if let Some(lod2_mesh) = state.mesh_registry.get("grass2_lod2") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_g2);
                                    p.set_mesh(lod2_mesh.clone());
                                    p.upload_instances(ctx.device(), &grass2_transforms);
                                    grass2_lod2_pipelines.push(p);
                                }

                                drop(shadow_map_g2);
                            }

                            // ================================================================
                            // GRASS3: Tall wispy grass (beach + riverbanks)
                            // ================================================================
                            let mut grass3_lod0_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut grass3_lod1_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut grass3_lod2_pipelines: Vec<TreePipeline> = Vec::new();

                            let mut grass3_transforms: Vec<Mat4> = Vec::new();

                            // Use different seed for grass3
                            let grass3_seed = state.seed.wrapping_add(7777) ^ (offset_x as u32) ^ ((offset_z as u32) << 16);
                            let mut rng_grass3 = rand::rngs::StdRng::seed_from_u64(grass3_seed as u64);

                            // Sparse spacing - larger clumps, fewer instances
                            // Increased to 25.0 = 90% instance reduction
                            let grass3_spacing = 25.0;

                            // Clumping noise for natural clustering
                            let clump_noise = noise::Perlin::new(grass3_seed);

                            let grass3_steps = (chunk_size / grass3_spacing) as i32;
                            for gz in 0..grass3_steps {
                                for gx in 0..grass3_steps {
                                    use rand::Rng;
                                    use noise::NoiseFn;

                                    let jitter_x = (rng_grass3.gen::<f32>() - 0.5) * grass3_spacing * 0.9;
                                    let jitter_z = (rng_grass3.gen::<f32>() - 0.5) * grass3_spacing * 0.9;

                                    let world_x = offset_x as f32 + (gx as f32 * grass3_spacing) + jitter_x;
                                    let world_z = offset_z as f32 + (gz as f32 * grass3_spacing) + jitter_z;

                                    let (height, _color) = croatoan_wfc::get_height_at(world_x, world_z, state.seed);
                                    let biome_t = croatoan_wfc::get_biome_t(world_x, world_z, state.seed);

                                    // Skip if underwater
                                    if height < 1.0 {
                                        continue;
                                    }

                                    // Check river proximity for inland spawning
                                    let river_depth = croatoan_wfc::calculate_river_depth(world_x, world_z, state.seed);
                                    let near_river = river_depth > 0.05 && river_depth < 0.6; // Near but not in river

                                    // Spawn on beach (0.48-0.62) OR near rivers inland
                                    let is_beach = biome_t >= 0.48 && biome_t <= 0.62;
                                    if !is_beach && !near_river {
                                        continue;
                                    }

                                    // Clumping: use noise to create sparse clusters
                                    let clump_val = clump_noise.get([world_x as f64 * 0.08, world_z as f64 * 0.08]);
                                    let clump_threshold = if near_river { 0.1 } else { 0.25 }; // Denser near rivers
                                    if clump_val < clump_threshold {
                                        continue;
                                    }

                                    // Low base density, boosted by clump strength
                                    let base_chance = if near_river { 0.45 } else { 0.30 };
                                    let clump_boost = ((clump_val - clump_threshold) * 0.5).min(0.3) as f32;
                                    let spawn_chance = base_chance + clump_boost;

                                    if rng_grass3.gen::<f32>() > spawn_chance {
                                        continue;
                                    }

                                    let rotation = rng_grass3.gen::<f32>() * std::f32::consts::TAU;
                                    // Very tall wispy beach grass with high variation
                                    let scale = 4.0 + rng_grass3.gen::<f32>() * 4.0; // 4.0-8.0 scale

                                    // DEBUG: +1.0 offset to test - remove once model issue found
                                    let transform = Mat4::from_scale_rotation_translation(
                                        Vec3::splat(scale),
                                        glam::Quat::from_rotation_y(rotation),
                                        Vec3::new(world_x, height + 1.0, world_z),
                                    );
                                    grass3_transforms.push(transform);
                                }
                            }

                            // Create LOD pipelines for grass3
                            if !grass3_transforms.is_empty() {
                                let shadow_map_g3 = shadow_map_mutex.safe_lock();

                                if let Some(lod0_mesh) = state.mesh_registry.get("grass3_lod0") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_g3);
                                    p.set_mesh(lod0_mesh.clone());
                                    p.upload_instances(ctx.device(), &grass3_transforms);
                                    grass3_lod0_pipelines.push(p);
                                }

                                if let Some(lod1_mesh) = state.mesh_registry.get("grass3_lod1") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_g3);
                                    p.set_mesh(lod1_mesh.clone());
                                    p.upload_instances(ctx.device(), &grass3_transforms);
                                    grass3_lod1_pipelines.push(p);
                                }

                                if let Some(lod2_mesh) = state.mesh_registry.get("grass3_lod2") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_g3);
                                    p.set_mesh(lod2_mesh.clone());
                                    p.upload_instances(ctx.device(), &grass3_transforms);
                                    grass3_lod2_pipelines.push(p);
                                }

                                drop(shadow_map_g3);
                                println!("[GRASS3] Created {} instances for chunk", grass3_transforms.len());
                            }

                            // ================================================================
                            // FLOWERS: Chamomile & Clover patches with surrounding groundcover
                            // ================================================================
                            let mut chamomile_lod0_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut chamomile_lod1_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut clover_patch_lod0_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut clover_patch_lod1_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut groundcover_lod0_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut groundcover_lod1_pipelines: Vec<TreePipeline> = Vec::new();

                            let mut chamomile_transforms: Vec<Mat4> = Vec::new();
                            let mut clover_patch_transforms: Vec<Mat4> = Vec::new();
                            let mut daisy_transforms: Vec<Mat4> = Vec::new();

                            // Seed for flower placement
                            let flower_seed = state.seed.wrapping_add(9999) ^ (offset_x as u32) ^ ((offset_z as u32) << 16);
                            let mut rng_flowers = rand::rngs::StdRng::seed_from_u64(flower_seed as u64);

                            // Flower patch spacing (patches every ~25m for larger clearings)
                            let patch_spacing = 25.0;
                            let patch_steps = (chunk_size / patch_spacing) as i32;
                            let mut flower_debug_checks = 0;
                            let mut flower_debug_passed = 0;

                            // CHAMOMILE PATCHES
                            for pz in 0..patch_steps {
                                for px in 0..patch_steps {
                                    use rand::Rng;
                                    flower_debug_checks += 1;

                                    let jitter_x = (rng_flowers.gen::<f32>() - 0.5) * patch_spacing * 0.8;
                                    let jitter_z = (rng_flowers.gen::<f32>() - 0.5) * patch_spacing * 0.8;

                                    let patch_x = offset_x as f32 + (px as f32 * patch_spacing) + jitter_x;
                                    let patch_z = offset_z as f32 + (pz as f32 * patch_spacing) + jitter_z;

                                    let (height, _color) = croatoan_wfc::get_height_at(patch_x, patch_z, state.seed);
                                    let biome_t = croatoan_wfc::get_biome_t(patch_x, patch_z, state.seed);

                                    // Forest clearings: biome_t 0.65-0.80 (wider range for more clearings)
                                    // Height > 2m (above water, on forest floor)
                                    if height < 2.0 || biome_t < 0.65 || biome_t > 0.80 {
                                        continue;
                                    }
                                    flower_debug_passed += 1;

                                    // 70% spawn chance (more patches in clearings)
                                    if rng_flowers.gen::<f32>() > 0.7 {
                                        continue;
                                    }

                                    // Spawn 2-3 chamomile plants in a small cluster
                                    let chamomile_count = 2 + (rng_flowers.gen::<f32>() * 1.5) as i32;
                                    for _c in 0..chamomile_count {
                                        let cx = patch_x + (rng_flowers.gen::<f32>() - 0.5) * 2.0;
                                        let cz = patch_z + (rng_flowers.gen::<f32>() - 0.5) * 2.0;
                                        let (ch, _) = croatoan_wfc::get_height_at(cx, cz, state.seed);

                                        let rotation = rng_flowers.gen::<f32>() * std::f32::consts::TAU;
                                        let scale = 0.8 + rng_flowers.gen::<f32>() * 0.4; // 0.8-1.2

                                        let transform = Mat4::from_scale_rotation_translation(
                                            Vec3::splat(scale),
                                            glam::Quat::from_rotation_y(rotation),
                                            Vec3::new(cx, ch, cz),
                                        );
                                        chamomile_transforms.push(transform);
                                    }

                                    // Spawn 5-6 surrounding daisy groundcover
                                    let groundcover_count = 5 + (rng_flowers.gen::<f32>() * 2.0) as i32;
                                    for _g in 0..groundcover_count {
                                        let angle = rng_flowers.gen::<f32>() * std::f32::consts::TAU;
                                        let dist = 1.5 + rng_flowers.gen::<f32>() * 2.5;
                                        let gx = patch_x + angle.cos() * dist;
                                        let gz = patch_z + angle.sin() * dist;
                                        let (gh, _) = croatoan_wfc::get_height_at(gx, gz, state.seed);

                                        let rotation = rng_flowers.gen::<f32>() * std::f32::consts::TAU;
                                        let scale = 0.6 + rng_flowers.gen::<f32>() * 0.4;

                                        let transform = Mat4::from_scale_rotation_translation(
                                            Vec3::splat(scale),
                                            glam::Quat::from_rotation_y(rotation),
                                            Vec3::new(gx, gh, gz),
                                        );
                                        daisy_transforms.push(transform);
                                    }
                                }
                            }

                            // CLOVER PATCHES (offset from chamomile by half spacing)
                            let clover_seed = state.seed.wrapping_add(8888) ^ (offset_x as u32) ^ ((offset_z as u32) << 16);
                            let mut rng_clover = rand::rngs::StdRng::seed_from_u64(clover_seed as u64);

                            for pz in 0..patch_steps {
                                for px in 0..patch_steps {
                                    use rand::Rng;

                                    // Offset clover patches by half spacing so they don't overlap chamomile
                                    let jitter_x = (rng_clover.gen::<f32>() - 0.5) * patch_spacing * 0.8;
                                    let jitter_z = (rng_clover.gen::<f32>() - 0.5) * patch_spacing * 0.8;

                                    let patch_x = offset_x as f32 + (px as f32 * patch_spacing) + (patch_spacing * 0.5) + jitter_x;
                                    let patch_z = offset_z as f32 + (pz as f32 * patch_spacing) + (patch_spacing * 0.5) + jitter_z;

                                    let (height, _color) = croatoan_wfc::get_height_at(patch_x, patch_z, state.seed);
                                    let biome_t = croatoan_wfc::get_biome_t(patch_x, patch_z, state.seed);

                                    // Forest clearings: biome_t 0.65-0.80 (wider range for more clearings)
                                    // Height > 2m (above water, on forest floor)
                                    if height < 2.0 || biome_t < 0.65 || biome_t > 0.80 {
                                        continue;
                                    }

                                    // 70% spawn chance (more patches in clearings)
                                    if rng_clover.gen::<f32>() > 0.7 {
                                        continue;
                                    }

                                    // Spawn 2-3 clover plants in a small cluster
                                    let clover_count = 2 + (rng_clover.gen::<f32>() * 1.5) as i32;
                                    for _c in 0..clover_count {
                                        let cx = patch_x + (rng_clover.gen::<f32>() - 0.5) * 2.0;
                                        let cz = patch_z + (rng_clover.gen::<f32>() - 0.5) * 2.0;
                                        let (ch, _) = croatoan_wfc::get_height_at(cx, cz, state.seed);

                                        let rotation = rng_clover.gen::<f32>() * std::f32::consts::TAU;
                                        let scale = 0.7 + rng_clover.gen::<f32>() * 0.5; // 0.7-1.2

                                        let transform = Mat4::from_scale_rotation_translation(
                                            Vec3::splat(scale),
                                            glam::Quat::from_rotation_y(rotation),
                                            Vec3::new(cx, ch, cz),
                                        );
                                        clover_patch_transforms.push(transform);
                                    }

                                    // Spawn 5-6 surrounding daisy groundcover around clover patches too
                                    let groundcover_count = 5 + (rng_clover.gen::<f32>() * 2.0) as i32;
                                    for _g in 0..groundcover_count {
                                        let angle = rng_clover.gen::<f32>() * std::f32::consts::TAU;
                                        let dist = 1.5 + rng_clover.gen::<f32>() * 2.5;
                                        let gx = patch_x + angle.cos() * dist;
                                        let gz = patch_z + angle.sin() * dist;
                                        let (gh, _) = croatoan_wfc::get_height_at(gx, gz, state.seed);

                                        let rotation = rng_clover.gen::<f32>() * std::f32::consts::TAU;
                                        let scale = 0.6 + rng_clover.gen::<f32>() * 0.4;

                                        let transform = Mat4::from_scale_rotation_translation(
                                            Vec3::splat(scale),
                                            glam::Quat::from_rotation_y(rotation),
                                            Vec3::new(gx, gh, gz),
                                        );
                                        daisy_transforms.push(transform);
                                    }
                                }
                            }

                            // Create chamomile LOD pipelines
                            if !chamomile_transforms.is_empty() {
                                let shadow_map_ch = shadow_map_mutex.safe_lock();

                                if let Some(lod0_mesh) = state.mesh_registry.get("chamomile_lod0") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_ch);
                                    p.set_mesh(lod0_mesh.clone());
                                    p.upload_instances(ctx.device(), &chamomile_transforms);
                                    chamomile_lod0_pipelines.push(p);
                                } else {
                                    println!("[FLOWERS ERROR] chamomile_lod0 mesh NOT FOUND in registry!");
                                }

                                if let Some(lod1_mesh) = state.mesh_registry.get("chamomile_lod1") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_ch);
                                    p.set_mesh(lod1_mesh.clone());
                                    p.upload_instances(ctx.device(), &chamomile_transforms);
                                    chamomile_lod1_pipelines.push(p);
                                } else {
                                    println!("[FLOWERS ERROR] chamomile_lod1 mesh NOT FOUND in registry!");
                                }

                                drop(shadow_map_ch);
                                println!("[FLOWERS] Chamomile: {} instances", chamomile_transforms.len());
                            }

                            // Create clover patch LOD pipelines
                            if !clover_patch_transforms.is_empty() {
                                let shadow_map_cl = shadow_map_mutex.safe_lock();

                                if let Some(lod0_mesh) = state.mesh_registry.get("clover_lod0") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_cl);
                                    p.set_mesh(lod0_mesh.clone());
                                    p.upload_instances(ctx.device(), &clover_patch_transforms);
                                    clover_patch_lod0_pipelines.push(p);
                                } else {
                                    println!("[FLOWERS ERROR] clover_lod0 mesh NOT FOUND in registry!");
                                }

                                if let Some(lod1_mesh) = state.mesh_registry.get("clover_lod1") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_cl);
                                    p.set_mesh(lod1_mesh.clone());
                                    p.upload_instances(ctx.device(), &clover_patch_transforms);
                                    clover_patch_lod1_pipelines.push(p);
                                } else {
                                    println!("[FLOWERS ERROR] clover_lod1 mesh NOT FOUND in registry!");
                                }

                                drop(shadow_map_cl);
                                println!("[FLOWERS] Clover patches: {} instances", clover_patch_transforms.len());
                            }

                            // Create groundcover LOD pipelines (daisy surrounding patches)
                            if !daisy_transforms.is_empty() {
                                let shadow_map_gc = shadow_map_mutex.safe_lock();

                                if let Some(lod0_mesh) = state.mesh_registry.get("daisy_lod0") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_gc);
                                    p.set_mesh(lod0_mesh.clone());
                                    p.upload_instances(ctx.device(), &daisy_transforms);
                                    groundcover_lod0_pipelines.push(p);
                                } else {
                                    println!("[FLOWERS ERROR] daisy_lod0 mesh NOT FOUND in registry!");
                                }
                                if let Some(lod1_mesh) = state.mesh_registry.get("daisy_lod1") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_gc);
                                    p.set_mesh(lod1_mesh.clone());
                                    p.upload_instances(ctx.device(), &daisy_transforms);
                                    groundcover_lod1_pipelines.push(p);
                                } else {
                                    println!("[FLOWERS ERROR] daisy_lod1 mesh NOT FOUND in registry!");
                                }

                                drop(shadow_map_gc);
                                println!("[FLOWERS] Daisy groundcover: {} instances", daisy_transforms.len());
                            }

                            // ================================================================
                            // SPIKEGRASS: Sparse on beach, dense and tall in salt marsh
                            // ================================================================
                            let mut spikegrass_lod0_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut spikegrass_lod1_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut spikegrass_transforms: Vec<Mat4> = Vec::new();

                            let spikegrass_seed = state.seed.wrapping_add(5555) ^ (offset_x as u32) ^ ((offset_z as u32) << 16);
                            let mut rng_spike = rand::rngs::StdRng::seed_from_u64(spikegrass_seed as u64);

                            // Beach: very sparse (spacing ~30m), tall scale 1.5-2.5
                            // Salt marsh: dense (spacing ~4m), very tall scale 2.0-3.5
                            let beach_spacing = 30.0;
                            let marsh_spacing = 4.0;

                            // Beach pass (sparse, biome_t 0.48-0.58, height 1-5m)
                            let beach_steps = (chunk_size / beach_spacing) as i32;
                            for pz in 0..beach_steps {
                                for px in 0..beach_steps {
                                    use rand::Rng;

                                    let jitter_x = (rng_spike.gen::<f32>() - 0.5) * beach_spacing * 0.6;
                                    let jitter_z = (rng_spike.gen::<f32>() - 0.5) * beach_spacing * 0.6;

                                    let world_x = offset_x as f32 + (px as f32 * beach_spacing) + jitter_x;
                                    let world_z = offset_z as f32 + (pz as f32 * beach_spacing) + jitter_z;

                                    let (height, _) = croatoan_wfc::get_height_at(world_x, world_z, state.seed);
                                    let biome_t = croatoan_wfc::get_biome_t(world_x, world_z, state.seed);

                                    // Beach zone: biome_t 0.48-0.58, height 1-5m
                                    if height < 1.0 || height > 5.0 || biome_t < 0.48 || biome_t > 0.58 {
                                        continue;
                                    }

                                    // 10% spawn chance on beach (very sparse)
                                    if rng_spike.gen::<f32>() > 0.10 {
                                        continue;
                                    }

                                    let rotation = rng_spike.gen::<f32>() * std::f32::consts::TAU;
                                    let scale = 1.5 + rng_spike.gen::<f32>() * 1.0; // 1.5-2.5 (tall)

                                    let transform = Mat4::from_scale_rotation_translation(
                                        Vec3::splat(scale),
                                        glam::Quat::from_rotation_y(rotation),
                                        Vec3::new(world_x, height, world_z),
                                    );
                                    spikegrass_transforms.push(transform);
                                }
                            }

                            // Salt marsh pass (dense + tall, biome_t 0.55-0.65, height 0.5-3m)
                            let marsh_steps = (chunk_size / marsh_spacing) as i32;
                            for pz in 0..marsh_steps {
                                for px in 0..marsh_steps {
                                    use rand::Rng;

                                    let jitter_x = (rng_spike.gen::<f32>() - 0.5) * marsh_spacing * 0.6;
                                    let jitter_z = (rng_spike.gen::<f32>() - 0.5) * marsh_spacing * 0.6;

                                    let world_x = offset_x as f32 + (px as f32 * marsh_spacing) + jitter_x;
                                    let world_z = offset_z as f32 + (pz as f32 * marsh_spacing) + jitter_z;

                                    let (height, _) = croatoan_wfc::get_height_at(world_x, world_z, state.seed);
                                    let biome_t = croatoan_wfc::get_biome_t(world_x, world_z, state.seed);

                                    // Salt marsh zone: biome_t 0.55-0.65, height 0.5-3m (low wet areas)
                                    if height < 0.5 || height > 3.0 || biome_t < 0.55 || biome_t > 0.65 {
                                        continue;
                                    }

                                    // 70% spawn chance in marsh (dense)
                                    if rng_spike.gen::<f32>() > 0.7 {
                                        continue;
                                    }

                                    let rotation = rng_spike.gen::<f32>() * std::f32::consts::TAU;
                                    // Very tall scale for marsh: 2.0-3.5
                                    let scale = 2.0 + rng_spike.gen::<f32>() * 1.5;

                                    let transform = Mat4::from_scale_rotation_translation(
                                        Vec3::splat(scale),
                                        glam::Quat::from_rotation_y(rotation),
                                        Vec3::new(world_x, height, world_z),
                                    );
                                    spikegrass_transforms.push(transform);
                                }
                            }

                            // Create spikegrass LOD pipelines
                            if !spikegrass_transforms.is_empty() {
                                let shadow_map_sg = shadow_map_mutex.safe_lock();

                                if let Some(lod0_mesh) = state.mesh_registry.get("spikegrass_lod0") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_sg);
                                    p.set_mesh(lod0_mesh.clone());
                                    p.upload_instances(ctx.device(), &spikegrass_transforms);
                                    spikegrass_lod0_pipelines.push(p);
                                }

                                if let Some(lod1_mesh) = state.mesh_registry.get("spikegrass_lod1") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_sg);
                                    p.set_mesh(lod1_mesh.clone());
                                    p.upload_instances(ctx.device(), &spikegrass_transforms);
                                    spikegrass_lod1_pipelines.push(p);
                                }

                                drop(shadow_map_sg);
                                println!("[SPIKEGRASS] {} instances (beach + salt marsh)", spikegrass_transforms.len());
                            }

                            // ================================================================
                            // HEDGE: Forest edges and beach transition zones
                            // ================================================================
                            let mut hedge_lod0_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut hedge_lod1_pipelines: Vec<TreePipeline> = Vec::new();
                            let mut hedge_transforms: Vec<Mat4> = Vec::new();

                            let hedge_seed = state.seed.wrapping_add(4444) ^ (offset_x as u32) ^ ((offset_z as u32) << 16);
                            let mut rng_hedge = rand::rngs::StdRng::seed_from_u64(hedge_seed as u64);

                            // FOREST EDGE pass: Dense thickets (spacing ~5m, 85% spawn, large scale 1.5-2.5)
                            let forest_hedge_spacing = 5.0;
                            let forest_hedge_steps = (chunk_size / forest_hedge_spacing) as i32;

                            for pz in 0..forest_hedge_steps {
                                for px in 0..forest_hedge_steps {
                                    use rand::Rng;

                                    let jitter_x = (rng_hedge.gen::<f32>() - 0.5) * forest_hedge_spacing * 0.7;
                                    let jitter_z = (rng_hedge.gen::<f32>() - 0.5) * forest_hedge_spacing * 0.7;

                                    let world_x = offset_x as f32 + (px as f32 * forest_hedge_spacing) + jitter_x;
                                    let world_z = offset_z as f32 + (pz as f32 * forest_hedge_spacing) + jitter_z;

                                    let (height, _) = croatoan_wfc::get_height_at(world_x, world_z, state.seed);
                                    let biome_t = croatoan_wfc::get_biome_t(world_x, world_z, state.seed);

                                    // Forest edge only (0.68-0.72), height > 3m
                                    if height < 3.0 || biome_t < 0.68 || biome_t > 0.72 {
                                        continue;
                                    }

                                    // 85% spawn chance (dense thicket along forest edge)
                                    if rng_hedge.gen::<f32>() > 0.85 {
                                        continue;
                                    }

                                    let rotation = rng_hedge.gen::<f32>() * std::f32::consts::TAU;
                                    let scale = 1.5 + rng_hedge.gen::<f32>() * 1.0; // 1.5-2.5 (large bushes)

                                    let transform = Mat4::from_scale_rotation_translation(
                                        Vec3::splat(scale),
                                        glam::Quat::from_rotation_y(rotation),
                                        Vec3::new(world_x, height, world_z),
                                    );
                                    hedge_transforms.push(transform);
                                }
                            }

                            // BEACH EDGE pass: Very sparse (spacing ~25m, 15% spawn, medium-large scale 1.3-2.0)
                            let beach_hedge_spacing = 25.0;
                            let beach_hedge_steps = (chunk_size / beach_hedge_spacing) as i32;

                            for pz in 0..beach_hedge_steps {
                                for px in 0..beach_hedge_steps {
                                    use rand::Rng;

                                    let jitter_x = (rng_hedge.gen::<f32>() - 0.5) * beach_hedge_spacing * 0.7;
                                    let jitter_z = (rng_hedge.gen::<f32>() - 0.5) * beach_hedge_spacing * 0.7;

                                    let world_x = offset_x as f32 + (px as f32 * beach_hedge_spacing) + jitter_x;
                                    let world_z = offset_z as f32 + (pz as f32 * beach_hedge_spacing) + jitter_z;

                                    let (height, _) = croatoan_wfc::get_height_at(world_x, world_z, state.seed);
                                    let biome_t = croatoan_wfc::get_biome_t(world_x, world_z, state.seed);

                                    // Beach edge/transition only (0.58-0.65), height > 3m
                                    if height < 3.0 || biome_t < 0.58 || biome_t > 0.65 {
                                        continue;
                                    }

                                    // 15% spawn chance (very sparse on beach)
                                    if rng_hedge.gen::<f32>() > 0.15 {
                                        continue;
                                    }

                                    let rotation = rng_hedge.gen::<f32>() * std::f32::consts::TAU;
                                    let scale = 1.3 + rng_hedge.gen::<f32>() * 0.7; // 1.3-2.0 (medium-large)

                                    let transform = Mat4::from_scale_rotation_translation(
                                        Vec3::splat(scale),
                                        glam::Quat::from_rotation_y(rotation),
                                        Vec3::new(world_x, height, world_z),
                                    );
                                    hedge_transforms.push(transform);
                                }
                            }

                            // Create hedge LOD pipelines
                            if !hedge_transforms.is_empty() {
                                let shadow_map_hg = shadow_map_mutex.safe_lock();

                                if let Some(lod0_mesh) = state.mesh_registry.get("hedge0_lod0") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_hg);
                                    p.set_mesh(lod0_mesh.clone());
                                    p.upload_instances(ctx.device(), &hedge_transforms);
                                    hedge_lod0_pipelines.push(p);
                                }

                                if let Some(lod1_mesh) = state.mesh_registry.get("hedge0_lod1") {
                                    let mut p = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_hg);
                                    p.set_mesh(lod1_mesh.clone());
                                    p.upload_instances(ctx.device(), &hedge_transforms);
                                    hedge_lod1_pipelines.push(p);
                                }

                                drop(shadow_map_hg);
                                println!("[HEDGE] {} instances (forest/beach edges)", hedge_transforms.len());
                            }

                            // ================================================================
                            // CAVE SYSTEM: Generate Perlin worm caves for this chunk
                            // ================================================================
                            let chunk_min = Vec3::new(offset_x as f32, -100.0, offset_z as f32);
                            let chunk_max = Vec3::new(offset_x as f32 + chunk_size, 200.0, offset_z as f32 + chunk_size);

                            // Check if this chunk should have a cave entrance
                            // Use noise to deterministically place cave entrances
                            let cave_entrance_noise = croatoan_wfc::fbm_3d(
                                Vec3::new(offset_x as f32 * 0.003, 0.0, offset_z as f32 * 0.003),
                                2, 2.0, 0.5, state.seed.wrapping_add(99999)
                            );

                            let mut cave_mesh_pipeline: Option<TreePipeline> = None;
                            let mut chunk_bio_orbs = Vec::new();
                            let mut chunk_bones = Vec::new();
                            let mut chunk_artifacts = Vec::new();

                            // ~30% of chunks get cave entrances for testing (was 0.7 = 5%)
                            if cave_entrance_noise > 0.2 {
                                // Find a suitable entrance point (hillside or mountain)
                                let entrance_x = offset_x as f32 + chunk_size * 0.5;
                                let entrance_z = offset_z as f32 + chunk_size * 0.5;
                                let (entrance_height, _) = croatoan_wfc::get_height_at(entrance_x, entrance_z, state.seed);

                                // Create caves in any terrain above sea level (was 15.0)
                                if entrance_height > 2.0 {
                                    let entrance_pos = Vec3::new(entrance_x, entrance_height - 2.0, entrance_z);

                                    // Calculate initial direction (into the hillside)
                                    let biome_t = croatoan_wfc::get_biome_t(entrance_x, entrance_z, state.seed);
                                    let inward_angle = std::f32::consts::PI * (0.5 + biome_t); // Varies based on terrain
                                    let initial_dir = Vec3::new(inward_angle.cos(), -0.2, inward_angle.sin()).normalize();

                                    // Configure worm tunnel
                                    let worm_config = WormConfig {
                                        seed: state.seed.wrapping_add(offset_x as u32).wrapping_add((offset_z as u32) << 16),
                                        step_size: 2.0,
                                        min_radius: 2.5,
                                        max_radius: 8.0,
                                        radius_frequency: 0.02,
                                        direction_frequency: 0.015,
                                        branch_probability: 0.15, // Increased for testing (was 0.015)
                                        min_tunnel_length: 60.0,
                                        max_tunnel_length: 200.0,
                                        descent_bias: 0.12,
                                        humidity_frequency: 0.05,
                                    };

                                    // Terrain height function for the worm to follow
                                    let seed_for_terrain = state.seed;
                                    let terrain_height_fn = |x: f32, z: f32| -> f32 {
                                        let (h, _) = croatoan_wfc::get_height_at(x, z, seed_for_terrain);
                                        h
                                    };

                                    // Generate the Perlin worm tunnel
                                    let worm_tunnel = generate_perlin_worm(entrance_pos, initial_dir, &worm_config, &terrain_height_fn);

                                    // Store worm tunnel for collision detection
                                    state.worm_tunnels.push(worm_tunnel.clone());

                                    // Generate cave mesh for this chunk
                                    let mesh_config = CaveMeshConfig::default();
                                    if let Some(cave_mesh_data) = generate_cave_mesh_for_chunk(&worm_tunnel, chunk_min, chunk_max, &mesh_config) {
                                        if !cave_mesh_data.positions.is_empty() {
                                            // Generate dummy UVs (caves don't need textures)
                                            let dummy_uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; cave_mesh_data.positions.len()];

                                            // Create TreeMesh from cave mesh data
                                            let cave_tree_mesh = TreePipeline::create_mesh(
                                                ctx.device(),
                                                &cave_mesh_data.positions,
                                                &cave_mesh_data.normals,
                                                &dummy_uvs,
                                                &cave_mesh_data.indices,
                                                None, // No texture
                                            );

                                            // Create pipeline with identity transform (mesh is already in world space)
                                            let shadow_map_cave = shadow_map_mutex.safe_lock();
                                            let mut cave_pipeline = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), &shadow_map_cave);
                                            cave_pipeline.set_mesh(cave_tree_mesh);
                                            cave_pipeline.upload_instances(ctx.device(), &[Mat4::IDENTITY]);
                                            drop(shadow_map_cave);

                                            cave_mesh_pipeline = Some(cave_pipeline);
                                        }
                                    }

                                    // Generate bioluminescent orbs (density 1.0 = default)
                                    let orbs = generate_bio_orbs(&worm_tunnel, 1.0);

                                    // Filter orbs to this chunk
                                    for orb in orbs {
                                        if orb.position.x >= chunk_min.x && orb.position.x < chunk_max.x &&
                                           orb.position.z >= chunk_min.z && orb.position.z < chunk_max.z {
                                            chunk_bio_orbs.push(orb);
                                        }
                                    }

                                    // Generate bones and artifacts
                                    let cave_gen_config = CaveGenConfig::default();
                                    let (bones, artifacts) = generate_worm_cave_items(&worm_tunnel, &cave_gen_config);

                                    // Filter items to this chunk
                                    for bone in bones {
                                        if bone.position.x >= chunk_min.x && bone.position.x < chunk_max.x &&
                                           bone.position.z >= chunk_min.z && bone.position.z < chunk_max.z {
                                            chunk_bones.push(bone);
                                        }
                                    }

                                    for artifact in artifacts {
                                        if artifact.position.x >= chunk_min.x && artifact.position.x < chunk_max.x &&
                                           artifact.position.z >= chunk_min.z && artifact.position.z < chunk_max.z {
                                            chunk_artifacts.push(artifact);
                                        }
                                    }

                                }
                            }

                            // Add to Manager
                            let loaded_chunk = LoadedChunk {
                                terrain: terrain_pipeline,
                                grass: grass_pipeline,
                                trees: foliage_pipelines,
                                trees_lod1: foliage_pipelines_lod1,
                                trees_lod2: foliage_pipelines_lod2,
                                ferns: fern_pipelines,
                                grass2_lod0: grass2_lod0_pipelines,
                                grass2_lod1: grass2_lod1_pipelines,
                                grass2_lod2: grass2_lod2_pipelines,
                                grass3_lod0: grass3_lod0_pipelines,
                                grass3_lod1: grass3_lod1_pipelines,
                                grass3_lod2: grass3_lod2_pipelines,
                                detritus: detritus_pipeline,
                                rocks: rock_pipelines,
                                boulders_lod0,
                                boulders_lod1,
                                boulders_lod2,
                                dead_logs_lod0,
                                dead_logs_lod1,
                                dead_logs_lod2,
                                conifer_shrubs_lod0,
                                conifer_shrubs_lod1,
                                conifer_shrubs_lod2,
                                chamomile_lod0: chamomile_lod0_pipelines,
                                chamomile_lod1: chamomile_lod1_pipelines,
                                clover_patch_lod0: clover_patch_lod0_pipelines,
                                clover_patch_lod1: clover_patch_lod1_pipelines,
                                groundcover_lod0: groundcover_lod0_pipelines,
                                groundcover_lod1: groundcover_lod1_pipelines,
                                spikegrass_lod0: spikegrass_lod0_pipelines,
                                spikegrass_lod1: spikegrass_lod1_pipelines,
                                hedge_lod0: hedge_lod0_pipelines,
                                hedge_lod1: hedge_lod1_pipelines,
                                buildings: building_pipelines,
                                river_water: Vec::new(), // TODO: implement river water
                                cave_mesh: cave_mesh_pipeline,
                                bio_orbs: chunk_bio_orbs,
                                cave_bones: chunk_bones,
                                cave_artifacts: chunk_artifacts,
                                bounds,
                            };
                            
                            let coord = ChunkCoord::from_world_pos(Vec3::new(offset_x as f32, 0.0, offset_z as f32), chunk_size);
                            manager.add_chunk(coord, loaded_chunk);

                            // Spawn animals for this chunk
                            // Note: We need to destructure to avoid borrow checker issues
                            let player_pos = state.player.position;
                            let seed = state.seed;

                            // Update ecology modifier from SystemsManager before spawning
                            let ecology_modifier = state.systems_manager.get_global_ecology_modifier();
                            state.animal_spawner.set_ecology_modifier(ecology_modifier);

                            let SharedState { animal_spawner, animal_manager, .. } = &mut *state;
                            animal_spawner.on_chunk_loaded(
                                coord.x,
                                coord.z,
                                chunk_size,
                                animal_manager,
                                player_pos,
                                seed,
                            );

                            // Update uploaded count
                            state.loading_progress.chunks_uploaded += 1;

                            // Check if loading is complete
                            // Wait for ALL chunks to load before allowing play
                            if state.game_state == GameState::Loading {
                                let uploaded = state.loading_progress.chunks_uploaded;
                                let total = state.loading_progress.total_chunks;
                                // Wait for all chunks to be uploaded before transitioning
                                if total > 0 && uploaded >= total {
                                    println!("[LOAD] All {} chunks loaded! Transitioning to Playing...", uploaded);
                                    state.loading_progress.current_status = "Ready!".to_string();
                                    state.game_state = GameState::Playing;
                                }
                            }
                        },
                        Err(_) => break,
                    }
                }
            }
        } // Release manager lock

        // Render frame (re-acquire locks as needed)
        let manager = chunk_manager.safe_lock();
        if state.game_state == GameState::Playing && manager.chunk_count() > 0 {
            let elapsed = start_time.elapsed().as_secs_f32();

            // Get the current frame
            let output = match ctx.surface.get_current_texture() {
                Ok(output) => output,
                Err(wgpu::SurfaceError::Outdated) => return,
                Err(e) => {
                    eprintln!("Render error: {}", e);
                    return;
                }
            };
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Create command encoder
            let mut encoder = ctx.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            // Get offscreen target view for post-process rendering
            let offscreen_guard = offscreen_target_mutex.safe_lock();
            let offscreen_view = offscreen_guard.as_ref().map(|t| &t.view);

            // Calculate sun direction
            let hour_angle = (state.time_of_day - 6.0) * (std::f32::consts::PI / 12.0);
            let sun_pos_x = hour_angle.cos();
            let sun_pos_y = hour_angle.sin(); // Removed max(0.1) to allow setting
            let sun_pos_z = 0.3;
            let sun_dir = Vec3::new(-sun_pos_x, -sun_pos_y, -sun_pos_z).normalize();

            // Calculate moon direction (opposite to sun)
            let moon_dir = -sun_dir;

            // Determine main light source (Sun or Moon)
            let is_day = sun_pos_y > -0.1; // Sun is visible or just setting
            let light_dir = if is_day { sun_dir } else { moon_dir };

            // Stable shadow projection
            let shadow_map_size = 1024.0_f32; // Reduced from 2048 for FPS
            let ortho_size = 600.0_f32;
            let shadow_center = Vec3::new(
                (state.player.position.x / 64.0).round() * 64.0,
                0.0,
                (state.player.position.z / 64.0).round() * 64.0,
            );
            let light_pos = shadow_center - light_dir * 500.0;
            let light_view = Mat4::look_at_rh(light_pos, shadow_center, Vec3::Y);
            let light_proj = Mat4::orthographic_rh(-ortho_size, ortho_size, -ortho_size, ortho_size, 1.0, 1500.0);
            let mut light_view_proj = light_proj * light_view;

            // Snap to shadow map texel grid
            let texel_size = (ortho_size * 2.0) / shadow_map_size;
            let shadow_origin = light_view_proj.transform_point3(Vec3::ZERO);
            let snapped_x = (shadow_origin.x / texel_size).round() * texel_size;
            let snapped_y = (shadow_origin.y / texel_size).round() * texel_size;
            let snap_offset = Vec3::new(snapped_x - shadow_origin.x, snapped_y - shadow_origin.y, 0.0);
            light_view_proj = Mat4::from_translation(snap_offset) * light_view_proj;

            // Update grass and tree cameras
            let view_proj = state.camera.view_projection_matrix();
            let frustum = Frustum::from_view_proj(&view_proj);

            // Compute fog parameters for tree shader (same as used in render pass)
            let fog_params = state.atmosphere.fog_params();
            let fog_color = state.atmosphere.fog_color();
            let fog_density = fog_params[0];
            let fog_start = fog_params[1];
            let fog_end = fog_params[2];

            {
                // Pre-render camera updates: only for pipeline types NOT updated in the render loop.
                // Trees, ferns, rocks are updated with LOD-specific settings inside the render loop.
            }

            // Update Water & Dispatch Compute
            {
                let mut water = water_system_mutex.safe_lock();
                // Connect weather wind to ocean waves
                let wind_strength = state.weather.wind_strength();
                // Wind direction from weather offset drift (slow rotation over time)
                let wind_angle = state.weather.wind_offset[0] * std::f32::consts::PI;
                water.set_wind(wind_angle, wind_strength);
                water.update(ctx.queue(), elapsed, delta, (-sun_dir).to_array());
                water.update_camera(ctx.queue(), view_proj.to_cols_array_2d(), state.camera.position.to_array());
                water.dispatch(&mut encoder);
            }

            // Update Pond Water (inland bodies)
            {
                let mut pond_water = pond_water_system_mutex.safe_lock();
                pond_water.update(ctx.queue(), view_proj.to_cols_array_2d(), state.camera.position.to_array(), delta);
            }

            // 0. Shadow Pass (skip at night — no sun = no shadows)
            {
                let shadow_map = shadow_map_mutex.safe_lock();
                if sun_pos_y > 0.02 {
                    let shadow_pipeline = shadow_pipeline_mutex.safe_lock();
                    let instanced_shadow_pipeline = instanced_shadow_pipeline_mutex.safe_lock();

                    shadow_pipeline.update_uniforms(ctx.queue(), &light_view_proj);
                    instanced_shadow_pipeline.update_uniforms(ctx.queue(), &light_view_proj);

                    let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Shadow Pass"),
                        color_attachments: &[],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &shadow_map.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    let shadow_max_dist = 250.0;
                    for (_coord, chunk) in manager.iter_chunks() {
                        let shadow_dist = (chunk.bounds.center - state.camera.position).length();
                        if shadow_dist > shadow_max_dist + chunk.bounds.radius { continue; }

                        // Terrain shadows
                        shadow_pipeline.render(
                            &mut shadow_pass,
                            &chunk.terrain.vertex_buffer,
                            &chunk.terrain.index_buffer,
                            chunk.terrain.index_count,
                        );

                        // Tree shadows (instanced)
                        for tree_pipeline in &chunk.trees {
                            if let Some((vb, ib, inst_buf, idx_count, inst_count)) = tree_pipeline.get_shadow_buffers() {
                                instanced_shadow_pipeline.render(
                                    &mut shadow_pass, vb, ib, inst_buf, idx_count, inst_count,
                                );
                            }
                        }

                        // Rock shadows (instanced)
                        for rock_pipeline in &chunk.rocks {
                            if let Some((vb, ib, inst_buf, idx_count, inst_count)) = rock_pipeline.get_shadow_buffers() {
                                instanced_shadow_pipeline.render(
                                    &mut shadow_pass, vb, ib, inst_buf, idx_count, inst_count,
                                );
                            }
                        }

                        // Boulder shadows - LOD0 only
                        for boulder in &chunk.boulders_lod0 {
                            if let Some((vb, ib, inst_buf, idx_count, inst_count)) = boulder.get_shadow_buffers() {
                                instanced_shadow_pipeline.render(
                                    &mut shadow_pass, vb, ib, inst_buf, idx_count, inst_count,
                                );
                            }
                        }
                    }
                } else {
                    // Night: clear shadow map to max depth (no shadows)
                    let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Shadow Clear"),
                        color_attachments: &[],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &shadow_map.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }
            }

            // Dynamic sky color
            let sky_color = {
                let sun_elevation = sun_pos_y;
                let t = sun_elevation.clamp(0.0, 1.0);
                
                let night_sky = (0.01_f32, 0.01, 0.03); // Deeper dark blue/black
                let sunrise_sky = (0.95_f32, 0.55, 0.35); // Slightly more vibrant sunrise
                let midday_sky = (0.2_f32, 0.4, 0.8);    // Deeper, richer blue sky

                if sun_elevation > 0.0 {
                    // Day: Sunrise -> Midday
                    wgpu::Color {
                        r: (sunrise_sky.0 * (1.0 - t) + midday_sky.0 * t) as f64,
                        g: (sunrise_sky.1 * (1.0 - t) + midday_sky.1 * t) as f64,
                        b: (sunrise_sky.2 * (1.0 - t) + midday_sky.2 * t) as f64,
                        a: 1.0,
                    }
                } else {
                    // Night: Sunset -> Night
                    let t_night = (-sun_elevation * 5.0).clamp(0.0, 1.0); // Transition quickly to night
                    wgpu::Color {
                        r: (sunrise_sky.0 * (1.0 - t_night) + night_sky.0 * t_night) as f64,
                        g: (sunrise_sky.1 * (1.0 - t_night) + night_sky.1 * t_night) as f64,
                        b: (sunrise_sky.2 * (1.0 - t_night) + night_sky.2 * t_night) as f64,
                        a: 1.0,
                    }
                }
            };

            // Determine which view to render scene to (offscreen for post-process, or direct)
            let scene_view = offscreen_view.unwrap_or(&view);

            // 0.5 Sky Pass (Draw Skybox/Clouds first)
            {
                let sky_pipeline = sky_pipeline_mutex.safe_lock();
                // Use rotation-only view matrix for sky - removes translation so sky
                // appears at infinity and doesn't shift with player movement
                let sky_view_proj = state.camera.sky_view_projection_matrix();
                sky_pipeline.update_uniforms(
                    ctx.queue(),
                    sky_view_proj,
                    sun_dir,
                    moon_dir,
                    Vec3::new(1.0, 1.0, 1.0), // Sun Color (White for now)
                    elapsed,
                    state.weather.cloud_coverage,
                    state.weather.cloud_color_base,
                    state.weather.cloud_density,
                    state.weather.cloud_color_shade,
                    state.weather.cloud_scale,
                    state.weather.wind_offset,
                    state.weather.rain_intensity(),
                    state.weather.ambient_dimming(),
                );

                let mut sky_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sky Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: scene_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(sky_color), // Clear with gradient base, then draw clouds over
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None, // Sky draws at max depth or ignores depth
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                sky_pipeline.render(&mut sky_pass);
            }

            // 1. Sun/Moon Pass
            {
                // Acquire locks before starting render pass to ensure they outlive the pass
                let sun_pipeline = sun_pipeline_mutex.safe_lock();
                let moon_pipeline = moon_pipeline_mutex.safe_lock();

                let mut sun_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sun/Moon Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: scene_view,
                        resolve_target: None,

                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load, // Load sky from previous pass
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // Render Sun
                if sun_pos_y > -0.2 { // Visible until slightly below horizon
                    sun_pipeline.update(ctx.queue(), &view_proj, sun_dir, state.camera.position, state.camera.right(), state.camera.up, state.time_of_day, elapsed);
                    sun_pipeline.render(&mut sun_pass);
                }

                // Render Moon
                if sun_pos_y < 0.2 { // Visible when sun is low or set
                    // Moon phase cycles roughly every 29.5 days, map to 0-1 range
                    let moon_phase = 0.5; // Full moon for now (TODO: calculate from game days)
                    moon_pipeline.update(ctx.queue(), &view_proj, moon_dir, state.camera.position, state.camera.right(), state.camera.up, moon_phase, elapsed);
                    moon_pipeline.render(&mut sun_pass);
                }
            }

            // 2. Main Render Pass
            {
                let water_system_guard = water_system_mutex.safe_lock();
                let pond_water_guard = pond_water_system_mutex.safe_lock();
                let orb_pipeline = animal_orb_pipeline_mutex.safe_lock();
                // Lock model_pipeline early so it outlives render_pass
                let model_pipeline = animal_model_pipeline_mutex.safe_lock();
                // Lock rain_pipeline early so it outlives render_pass
                let mut rain_pipeline = rain_pipeline_mutex.safe_lock();
                // Lock ember_pipeline early so it outlives render_pass
                let mut ember_pipeline = ember_pipeline_mutex.safe_lock();
                // Lock bio_orb_pipeline early so it outlives render_pass
                let mut bio_orb_pipeline = bio_orb_pipeline_mutex.safe_lock();
                // Reusable pipeline pool — avoids per-frame TreePipeline::new() GPU overhead
                // Pipelines are popped from the free list, used, then returned after render_pass
                static PIPELINE_FREE_LIST: OnceLock<Mutex<Vec<TreePipeline>>> = OnceLock::new();
                // Cache campfire meshes — they're deterministic and never change after creation
                static CAMPFIRE_MESH_CACHE: OnceLock<Mutex<std::collections::HashMap<u64, TreeMesh>>> = OnceLock::new();
                let pipeline_free_list_mutex = PIPELINE_FREE_LIST.get_or_init(|| Mutex::new(Vec::new()));
                let mut pipeline_free_list = pipeline_free_list_mutex.safe_lock();
                let mut pipeline_pool: Vec<TreePipeline> = pipeline_free_list.drain(..).collect();

                let mut container_pipelines: Vec<TreePipeline> = Vec::new();
                let mut campfire_pipelines: Vec<TreePipeline> = Vec::new();
                let mut weapon_pipelines: Vec<TreePipeline> = Vec::new();
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: scene_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load, // Keep sky + sun from previous pass
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: ctx.depth_view(),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // Atmospheric fog from time-of-day system (use validated params)
                let fog_params = state.atmosphere.fog_params();
                let fog_color = state.atmosphere.fog_color();
                let fog_density = fog_params[0];  // Base fog intensity
                let fog_start = fog_params[1];
                let fog_end = fog_params[2];

                // Render chunks with frustum culling and LOD
                let mut terrain_rendered = 0;
                let mut terrain_culled = 0;
                let grass_rendered = 0;
                let mut trees_rendered = 0;
                let mut trees_lod1_rendered = 0;
                let mut trees_lod2_rendered = 0;
                let mut rocks_rendered: usize = 0;
                let mut boulders_rendered: usize = 0;
                let mut shrubs_rendered: usize = 0;
                let mut ferns_rendered: usize = 0;
                let dead_logs_rendered: usize = 0;
                let mut buildings_rendered = 0;

                // Use render distance setting from pause menu
                // Distance is to chunk CENTER (not edge), so with 256-unit chunks,
                // player can be up to 181 units from center (corner to center diagonal)
                let grass_max_distance = 250.0;  // Grass visible within 250 units
                let detritus_max_distance = 0.0; // DISABLED - detritus is FPS killer
                let building_max_distance = state.render_distance * 1.0; // Buildings visible at render dist

                // LOD distance configuration for trees (3-tier with extended back distance)
                // Distances scale with dither_distance_ratio for player-controlled performance
                // Lower ratio = closer dither = better FPS, shorter sight
                let dither_base = state.render_distance * state.dither_distance_ratio;
                let _fade_width = state.dither_fade_width; // Reserved for future fine-tuning

                // 3-tier LOD with tightened distances for performance:
                // LOD0 solid: 0-170 (full detail only where visible)
                // LOD0->LOD1 transition: 170-270 (100 unit crossfade)
                // LOD1 solid: 270-510 (mid-range)
                // LOD1->LOD2 transition: 510-680 (170 unit crossfade)
                // LOD2 solid: 680-1020 (distant silhouettes fade into fog)
                let lod0_fade_start = dither_base * 0.5;    // ~170 @ default
                let lod0_fade_end = dither_base * 0.8;      // ~270 @ default (LOD0 fully faded)
                let lod1_fade_start = dither_base * 1.5;    // ~510 @ default (LOD1 starts fading)
                let lod1_fade_end = dither_base * 2.0;      // ~680 @ default (LOD1 fully faded)
                let lod2_max = dither_base * 3.0;           // ~1020 @ default (fog fade)
                let lod_config = TreeLODConfig {
                    lod0_fade_start,                        // ~170: LOD0 starts fading
                    lod0_fade_end,                          // ~270: LOD0 fully faded, LOD1 solid
                    lod1_fade_start,                        // ~510: LOD1 starts fading
                    lod1_fade_end,                          // ~680: LOD1 fully faded, LOD2 solid
                    lod2_max_distance: lod2_max,            // ~1020: LOD2 fades into fog
                };

                // Collect bio-orbs during main chunk loop (avoids separate chunk iteration)
                let mut all_bio_orbs: Vec<BioOrbInstance> = Vec::new();

                // Get campfire lights once (used by terrain shader + ember particles)
                let campfire_light_data: Vec<[f32; 4]> = state.campfire_manager
                    .get_light_data(state.camera.position, 50.0, 8)
                    .iter()
                    .map(|l| [l.position.x, l.position.y, l.position.z, l.intensity])
                    .collect();

                for (_coord, chunk) in manager.iter_chunks() {
                    // Frustum cull - skip chunks outside view
                    if !frustum.contains_sphere(chunk.bounds.center, chunk.bounds.radius) {
                        terrain_culled += 1;
                        continue;
                    }
                    terrain_rendered += 1;

                    // Calculate muzzle flash world position (in front of player)
                    let flash_offset = state.camera.forward() * 1.5 + Vec3::new(0.3, -0.3, 0.0);
                    let flash_world_pos = state.camera.position + flash_offset;

                    // Terrain
                    chunk.terrain.update_uniforms(
                        ctx.queue(),
                        &view_proj,
                        &light_view_proj,
                        elapsed,
                        fog_color,
                        fog_start,
                        fog_end,
                        fog_density,
                        sun_dir.to_array(),
                        state.camera.position.to_array(),
                        state.camera.position.to_array(),
                        flash_world_pos.to_array(),
                        state.swing_animation.muzzle_flash,
                        &campfire_light_data,
                    );
                    chunk.terrain.render(&mut render_pass);

                    let dist = (chunk.bounds.center - state.camera.position).length();

                    // Procedural grass render DISABLED - using grass2/grass3 models instead

                    // Trees with 3-tier LOD system - tightened for performance
                    // LOD0 (full detail): 0-270, dither out at 170-270
                    // LOD1 (simplified): 170-680, dither in at 170-270, solid 270-510, dither out 510-680
                    // LOD2 (low-poly): 510-1020, dither in at 510-680, solid into fog

                    // Determine LOD ranges and transition zones
                    let in_lod0_range = dist <= lod_config.lod0_fade_end;
                    let in_lod1_range = dist >= lod_config.lod0_fade_start && dist <= lod_config.lod1_fade_end;
                    let in_lod2_range = dist >= lod_config.lod1_fade_start && dist <= lod_config.lod2_max_distance;
                    let in_near_transition = dist >= lod_config.lod0_fade_start && dist <= lod_config.lod0_fade_end;
                    let in_mid_transition = dist >= lod_config.lod1_fade_start && dist <= lod_config.lod1_fade_end;

                    // Render LOD0 (full detail) when in range
                    if in_lod0_range {
                        for trees in &chunk.trees {
                            if !trees.is_visible(&frustum) { continue; }
                            if in_near_transition {
                                trees.update_camera_with_lod(
                                    ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density,
                                    0.5, 1.0,
                                    LODFadeMode::LOD0FadeOut,
                                    lod_config.lod0_fade_start,
                                    lod_config.lod0_fade_end,
                                );
                            } else {
                                trees.update_camera_full(
                                    ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density,
                                    0.5, 1.0,
                                );
                            }
                            trees_rendered += 1;
                            trees.render(&mut render_pass);
                        }
                    }

                    // Render LOD1 (simplified) when in range
                    // Mid transition (to LOD2) takes priority over near transition
                    if in_lod1_range {
                        for trees_lod1 in &chunk.trees_lod1 {
                            if !trees_lod1.is_visible(&frustum) { continue; }
                            if in_mid_transition {
                                // Fading OUT to LOD2 (far end of LOD1)
                                trees_lod1.update_camera_with_lod(
                                    ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density,
                                    0.5, 1.0,
                                    LODFadeMode::LOD1FadeOut,
                                    lod_config.lod1_fade_start,
                                    lod_config.lod1_fade_end,
                                );
                            } else if in_near_transition {
                                // Fading IN from LOD0 (near end)
                                trees_lod1.update_camera_with_lod(
                                    ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density,
                                    0.5, 1.0,
                                    LODFadeMode::LOD1FadeIn,
                                    lod_config.lod0_fade_start,
                                    lod_config.lod0_fade_end,
                                );
                            } else {
                                trees_lod1.update_camera_full(
                                    ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density,
                                    0.5, 1.0,
                                );
                            }
                            trees_lod1_rendered += 1;
                            trees_lod1.render(&mut render_pass);
                        }
                    }

                    // Render LOD2 (distant billboards) when in range
                    // Fades in from LOD1 at mid transition, solid beyond into fog
                    if in_lod2_range {
                        for trees_lod2 in &chunk.trees_lod2 {
                            if !trees_lod2.is_visible(&frustum) { continue; }
                            if in_mid_transition {
                                // Fading IN from LOD1
                                trees_lod2.update_camera_with_lod(
                                    ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density,
                                    0.5, 1.0,
                                    LODFadeMode::LOD2FadeIn,
                                    lod_config.lod1_fade_start,
                                    lod_config.lod1_fade_end,
                                );
                            } else {
                                // Solid LOD2 (650+), let fog handle the fade-out
                                trees_lod2.update_camera_full(
                                    ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density,
                                    0.5, 1.0,
                                );
                            }
                            trees_lod2_rendered += 1;
                            trees_lod2.render(&mut render_pass);
                        }
                    }

                    // Detritus (update_camera + render in one pass — no separate pre-render loop)
                    if let Some(detritus) = &chunk.detritus {
                        if dist <= detritus_max_distance {
                            detritus.update_camera(
                                ctx.queue(), &view_proj, sun_dir.to_array(),
                                state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                            );
                            detritus.render(&mut render_pass);
                        }
                    }

                    // Rocks (non-boulder, moderate draw distance)
                    let rock_max_distance = 500.0;
                    for rock in &chunk.rocks {
                        if dist <= rock_max_distance && rock.is_visible(&frustum) {
                            rock.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.0, 0.0, // no alpha cutoff, procedural coloring
                            );
                            rocks_rendered += rock.instance_count() as usize;
                            rock.render(&mut render_pass);
                        }
                    }

                    // Boulders with LOD system - tightened distances for performance
                    // LOD0: 0-200 units (high detail), fade out 150-200
                    // LOD1: 150-450 units (medium), fade 150-200 in, 400-450 out
                    // LOD2: 400-700 units (low detail), fade in 400-450
                    let boulder_lod0_end = 200.0;
                    let boulder_lod0_fade_start = 150.0;
                    let boulder_lod1_end = 450.0;
                    let boulder_lod1_fade_start = 400.0;

                    let boulder_max_distance = 700.0;
                    let in_boulder_lod0 = dist <= boulder_lod0_end;
                    let in_boulder_lod1 = dist >= boulder_lod0_fade_start && dist <= boulder_lod1_end;
                    let in_boulder_lod2 = dist >= boulder_lod1_fade_start && dist <= boulder_max_distance;
                    let boulder_transition_0_1 = dist >= boulder_lod0_fade_start && dist <= boulder_lod0_end;
                    let boulder_transition_1_2 = dist >= boulder_lod1_fade_start && dist <= boulder_lod1_end;

                    // Boulder LOD0 (high detail) - NO WIND (boulders are static)
                    if in_boulder_lod0 {
                        for boulder in &chunk.boulders_lod0 {
                            if !boulder.is_visible(&frustum) { continue; }
                            boulder.update_camera_no_wind(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.0, 1.0,
                                if boulder_transition_0_1 { LODFadeMode::LOD0FadeOut } else { LODFadeMode::Disabled },
                                boulder_lod0_fade_start, boulder_lod0_end,
                            );
                            boulders_rendered += boulder.instance_count() as usize;
                            boulder.render(&mut render_pass);
                        }
                    }

                    // Boulder LOD1 (medium detail) - NO WIND
                    if in_boulder_lod1 {
                        for boulder in &chunk.boulders_lod1 {
                            if !boulder.is_visible(&frustum) { continue; }
                            let (lod_mode, fade_start, fade_end) = if boulder_transition_0_1 {
                                (LODFadeMode::LOD1FadeIn, boulder_lod0_fade_start, boulder_lod0_end)
                            } else if boulder_transition_1_2 {
                                (LODFadeMode::LOD0FadeOut, boulder_lod1_fade_start, boulder_lod1_end)
                            } else {
                                (LODFadeMode::Disabled, 0.0, 0.0)
                            };
                            boulder.update_camera_no_wind(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.0, 1.0, lod_mode, fade_start, fade_end,
                            );
                            boulders_rendered += boulder.instance_count() as usize;
                            boulder.render(&mut render_pass);
                        }
                    }

                    // Boulder LOD2 (low detail) - NO WIND
                    if in_boulder_lod2 {
                        for boulder in &chunk.boulders_lod2 {
                            if !boulder.is_visible(&frustum) { continue; }
                            boulder.update_camera_no_wind(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.0, 1.0,
                                if boulder_transition_1_2 { LODFadeMode::LOD1FadeIn } else { LODFadeMode::Disabled },
                                boulder_lod1_fade_start, boulder_lod1_end,
                            );
                            boulders_rendered += boulder.instance_count() as usize;
                            boulder.render(&mut render_pass);
                        }
                    }

                    // Dead logs (fallen trees / driftwood) - LOD based on distance
                    // LOD distances: 0-100 (LOD0), 80-250 (LOD1), 200-500 (LOD2)
                    let dead_log_lod0_end = 100.0;
                    let dead_log_lod0_fade_start = 80.0;
                    let dead_log_lod1_end = 250.0;
                    let dead_log_lod1_fade_start = 200.0;

                    let in_dead_log_lod0 = dist <= dead_log_lod0_end;
                    let in_dead_log_lod1 = dist >= dead_log_lod0_fade_start && dist <= dead_log_lod1_end;
                    let in_dead_log_lod2 = dist >= dead_log_lod1_fade_start && dist <= 500.0;

                    if in_dead_log_lod0 {
                        for dead_log in &chunk.dead_logs_lod0 {
                            if !dead_log.is_visible(&frustum) { continue; }
                            dead_log.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.0, 1.0,
                            );
                            dead_log.render(&mut render_pass);
                        }
                    }

                    if in_dead_log_lod1 && !in_dead_log_lod0 {
                        for dead_log in &chunk.dead_logs_lod1 {
                            if !dead_log.is_visible(&frustum) { continue; }
                            dead_log.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.0, 1.0,
                            );
                            dead_log.render(&mut render_pass);
                        }
                    }

                    if in_dead_log_lod2 && !in_dead_log_lod1 {
                        for dead_log in &chunk.dead_logs_lod2 {
                            if !dead_log.is_visible(&frustum) { continue; }
                            dead_log.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.0, 1.0,
                            );
                            dead_log.render(&mut render_pass);
                        }
                    }

                    // Conifer shrubs - LOD based on distance
                    // LOD distances: 0-100 (LOD0), 80-250 (LOD1), 200-500 (LOD2)
                    let shrub_lod0_end = 100.0;
                    let shrub_lod1_end = 250.0;
                    let shrub_lod2_end = 500.0;

                    let in_shrub_lod0 = dist <= shrub_lod0_end;
                    let in_shrub_lod1 = dist > shrub_lod0_end && dist <= shrub_lod1_end;
                    let in_shrub_lod2 = dist > shrub_lod1_end && dist <= shrub_lod2_end;

                    if in_shrub_lod0 {
                        for shrub in &chunk.conifer_shrubs_lod0 {
                            if !shrub.is_visible(&frustum) { continue; }
                            shrub.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0, // alpha_cutoff 0.5 for clean foliage edges
                            );
                            shrubs_rendered += shrub.instance_count() as usize;
                            shrub.render(&mut render_pass);
                        }
                    }

                    if in_shrub_lod1 {
                        for shrub in &chunk.conifer_shrubs_lod1 {
                            if !shrub.is_visible(&frustum) { continue; }
                            shrub.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0, // alpha_cutoff 0.5 for clean foliage edges
                            );
                            shrubs_rendered += shrub.instance_count() as usize;
                            shrub.render(&mut render_pass);
                        }
                    }

                    if in_shrub_lod2 {
                        for shrub in &chunk.conifer_shrubs_lod2 {
                            if !shrub.is_visible(&frustum) { continue; }
                            shrub.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0, // alpha_cutoff 0.5 for clean foliage edges
                            );
                            shrubs_rendered += shrub.instance_count() as usize;
                            shrub.render(&mut render_pass);
                        }
                    }

                    // ================================================================
                    // FLOWERS: Chamomile, Clover patches, and groundcover (forest floor)
                    // ================================================================
                    // Chamomile LOD: 0-60 (LOD0), 50-150 (LOD1)
                    // Clover patch LOD: 0-50 (LOD0), 40-120 (LOD1)
                    // Groundcover LOD: 0-40 (LOD0), 30-100 (LOD1)
                    let chamomile_lod0_end = 60.0;
                    let chamomile_lod1_end = 150.0;
                    let clover_patch_lod0_end = 50.0;
                    let clover_patch_lod1_end = 120.0;
                    let groundcover_lod0_end = 40.0;
                    let groundcover_lod1_end = 100.0;

                    // Chamomile rendering
                    if dist <= chamomile_lod0_end {
                        for flower in &chunk.chamomile_lod0 {
                            if !flower.is_visible(&frustum) { continue; }
                            flower.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            flower.render(&mut render_pass);
                        }
                    } else if dist <= chamomile_lod1_end {
                        for flower in &chunk.chamomile_lod1 {
                            if !flower.is_visible(&frustum) { continue; }
                            flower.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            flower.render(&mut render_pass);
                        }
                    }

                    // Clover patch rendering
                    if dist <= clover_patch_lod0_end {
                        for flower in &chunk.clover_patch_lod0 {
                            if !flower.is_visible(&frustum) { continue; }
                            flower.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            flower.render(&mut render_pass);
                        }
                    } else if dist <= clover_patch_lod1_end {
                        for flower in &chunk.clover_patch_lod1 {
                            if !flower.is_visible(&frustum) { continue; }
                            flower.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            flower.render(&mut render_pass);
                        }
                    }

                    // Groundcover (daisy) rendering
                    if dist <= groundcover_lod0_end {
                        for flower in &chunk.groundcover_lod0 {
                            if !flower.is_visible(&frustum) { continue; }
                            flower.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            flower.render(&mut render_pass);
                        }
                    } else if dist <= groundcover_lod1_end {
                        for flower in &chunk.groundcover_lod1 {
                            if !flower.is_visible(&frustum) { continue; }
                            flower.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            flower.render(&mut render_pass);
                        }
                    }

                    // Spikegrass (beach sparse, salt marsh dense)
                    let spikegrass_lod0_end = 50.0;
                    let spikegrass_lod1_end = 120.0;

                    if dist <= spikegrass_lod0_end {
                        for sg in &chunk.spikegrass_lod0 {
                            if !sg.is_visible(&frustum) { continue; }
                            sg.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            sg.render(&mut render_pass);
                        }
                    } else if dist <= spikegrass_lod1_end {
                        for sg in &chunk.spikegrass_lod1 {
                            if !sg.is_visible(&frustum) { continue; }
                            sg.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            sg.render(&mut render_pass);
                        }
                    }

                    // Hedge (forest/beach edges)
                    let hedge_lod0_end = 80.0;
                    let hedge_lod1_end = 200.0;

                    if dist <= hedge_lod0_end {
                        for hg in &chunk.hedge_lod0 {
                            if !hg.is_visible(&frustum) { continue; }
                            hg.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            hg.render(&mut render_pass);
                        }
                    } else if dist <= hedge_lod1_end {
                        for hg in &chunk.hedge_lod1 {
                            if !hg.is_visible(&frustum) { continue; }
                            hg.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            hg.render(&mut render_pass);
                        }
                    }

                    // Ferns (forest understory - small plants, limited draw distance)
                    let fern_max_distance = 150.0;
                    for fern in &chunk.ferns {
                        if dist <= fern_max_distance && fern.is_visible(&frustum) {
                            fern.update_camera_full(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                            );
                            ferns_rendered += fern.instance_count() as usize;
                            fern.render(&mut render_pass);
                        }
                    }

                    // ================================================================
                    // GRASS2: Inland/meadow/forest ground cover with 3 LOD levels + crossfade
                    // ================================================================
                    // LOD0: 0-55, LOD1: 45-125, LOD2: 115-250 (with 10m crossfade zones)
                    let grass2_lod0_fade_start = 45.0;  // LOD0 starts fading
                    let grass2_lod0_end = 55.0;         // LOD0 fully gone
                    let grass2_lod1_fade_start = 115.0; // LOD1 starts fading
                    let grass2_lod1_end = 125.0;        // LOD1 fully gone
                    let grass2_lod2_end = 250.0;

                    // Determine LOD visibility with crossfade zones
                    let in_grass2_lod0 = dist <= grass2_lod0_end;
                    let in_grass2_lod1 = dist >= grass2_lod0_fade_start && dist <= grass2_lod1_end;
                    let in_grass2_lod2 = dist >= grass2_lod1_fade_start && dist <= grass2_lod2_end;

                    // Transition zones (both LODs render during crossfade)
                    let grass2_transition_0_1 = dist >= grass2_lod0_fade_start && dist <= grass2_lod0_end;
                    let grass2_transition_1_2 = dist >= grass2_lod1_fade_start && dist <= grass2_lod1_end;

                    if in_grass2_lod0 {
                        for g2 in &chunk.grass2_lod0 {
                            if !g2.is_visible(&frustum) { continue; }
                            g2.update_camera_with_lod(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                                if grass2_transition_0_1 { LODFadeMode::LOD0FadeOut } else { LODFadeMode::Disabled },
                                grass2_lod0_fade_start, grass2_lod0_end,
                            );
                            g2.render(&mut render_pass);
                        }
                    }

                    if in_grass2_lod1 {
                        for g2 in &chunk.grass2_lod1 {
                            if !g2.is_visible(&frustum) { continue; }
                            let (lod_mode, fade_start, fade_end) = if grass2_transition_0_1 {
                                (LODFadeMode::LOD1FadeIn, grass2_lod0_fade_start, grass2_lod0_end)
                            } else if grass2_transition_1_2 {
                                (LODFadeMode::LOD0FadeOut, grass2_lod1_fade_start, grass2_lod1_end)
                            } else {
                                (LODFadeMode::Disabled, 0.0, 0.0)
                            };
                            g2.update_camera_with_lod(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0, lod_mode, fade_start, fade_end,
                            );
                            g2.render(&mut render_pass);
                        }
                    }

                    if in_grass2_lod2 {
                        for g2 in &chunk.grass2_lod2 {
                            if !g2.is_visible(&frustum) { continue; }
                            g2.update_camera_with_lod(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                                if grass2_transition_1_2 { LODFadeMode::LOD2FadeIn } else { LODFadeMode::Disabled },
                                grass2_lod1_fade_start, grass2_lod1_end,
                            );
                            g2.render(&mut render_pass);
                        }
                    }

                    // ================================================================
                    // GRASS3: Beach grass with 3 LOD levels + crossfade transitions
                    // ================================================================
                    // LOD0: 0-50, LOD1: 40-100, LOD2: 90-200 (with 10m crossfade zones)
                    // Extended LOD0 range for more detail at close range
                    let grass3_lod0_fade_start = 40.0;  // LOD0 starts fading
                    let grass3_lod0_end = 50.0;         // LOD0 fully gone
                    let grass3_lod1_fade_start = 90.0;  // LOD1 starts fading
                    let grass3_lod1_end = 100.0;        // LOD1 fully gone
                    let grass3_lod2_end = 200.0;

                    // Determine LOD visibility with crossfade zones
                    let in_grass3_lod0 = dist <= grass3_lod0_end;
                    let in_grass3_lod1 = dist >= grass3_lod0_fade_start && dist <= grass3_lod1_end;
                    let in_grass3_lod2 = dist >= grass3_lod1_fade_start && dist <= grass3_lod2_end;

                    // Transition zones (both LODs render during crossfade)
                    let grass3_transition_0_1 = dist >= grass3_lod0_fade_start && dist <= grass3_lod0_end;
                    let grass3_transition_1_2 = dist >= grass3_lod1_fade_start && dist <= grass3_lod1_end;

                    // Render grass3 LOD0 (highest detail, close range)
                    if in_grass3_lod0 {
                        for g3 in &chunk.grass3_lod0 {
                            if !g3.is_visible(&frustum) { continue; }
                            g3.update_camera_with_lod(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                                if grass3_transition_0_1 { LODFadeMode::LOD0FadeOut } else { LODFadeMode::Disabled },
                                grass3_lod0_fade_start, grass3_lod0_end,
                            );
                            g3.render(&mut render_pass);
                        }
                    }

                    // Render grass3 LOD1 (mid-range)
                    if in_grass3_lod1 {
                        for g3 in &chunk.grass3_lod1 {
                            if !g3.is_visible(&frustum) { continue; }
                            let (lod_mode, fade_start, fade_end) = if grass3_transition_0_1 {
                                (LODFadeMode::LOD1FadeIn, grass3_lod0_fade_start, grass3_lod0_end)
                            } else if grass3_transition_1_2 {
                                (LODFadeMode::LOD0FadeOut, grass3_lod1_fade_start, grass3_lod1_end)
                            } else {
                                (LODFadeMode::Disabled, 0.0, 0.0)
                            };
                            g3.update_camera_with_lod(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0, lod_mode, fade_start, fade_end,
                            );
                            g3.render(&mut render_pass);
                        }
                    }

                    // Render grass3 LOD2 (far range, lowest detail)
                    if in_grass3_lod2 {
                        for g3 in &chunk.grass3_lod2 {
                            if !g3.is_visible(&frustum) { continue; }
                            g3.update_camera_with_lod(
                                ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density,
                                0.5, 1.0,
                                if grass3_transition_1_2 { LODFadeMode::LOD2FadeIn } else { LODFadeMode::Disabled },
                                grass3_lod1_fade_start, grass3_lod1_end,
                            );
                            g3.render(&mut render_pass);
                        }
                    }

                    // Buildings
                    for building in &chunk.buildings {
                        if dist <= building_max_distance {
                            buildings_rendered += 1;
                            building.update_uniforms(
                                ctx.queue(),
                                &view_proj,
                                &light_view_proj,
                                sun_dir,
                                state.camera.position,
                                fog_color,
                                fog_start,
                                fog_end,
                                state.weather.ambient_dimming(),
                                0.8, // shadow_strength - strong shadows for contrast
                                state.weather.rain_intensity(),
                            );
                            building.render(&mut render_pass);
                        }
                    }

                    // Cave mesh (Perlin worm caves)
                    if let Some(ref cave_mesh) = chunk.cave_mesh {
                        cave_mesh.update_camera_full(
                            ctx.queue(), &view_proj, &light_view_proj,
                            sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                            fog_color, fog_start, fog_end, fog_density,
                            0.4, // ambient (darker in caves)
                            0.6, // shadow strength
                        );
                        cave_mesh.render(&mut render_pass);
                    }

                    // Collect bio-orbs from this chunk (merged into main loop to avoid extra iteration)
                    for orb in &chunk.bio_orbs {
                        all_bio_orbs.push(BioOrbInstance {
                            position: orb.position.to_array(),
                            radius: orb.cluster_size * 0.3,
                            color: orb.color,
                            intensity: orb.intensity,
                            pulse_phase: orb.pulse_phase,
                            pulse_speed: orb.pulse_speed,
                            _padding: [0.0, 0.0],
                        });
                    }
                }

                // Render Storage Containers (chests, crates, etc.)
                // Uses container_pipelines vec declared at outer scope (outlives render_pass)
                let containers_rendered = {
                    let mut rendered_count = 0usize;
                    let player_pos = state.camera.position;
                    let container_max_distance = 200.0;
                    let lod0_threshold = 20.0;  // Close: use high-poly LOD0
                    let lod1_threshold = 50.0;  // Medium: use LOD1

                    // Debug: log container count once
                    static CONTAINER_DEBUG_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                    let total_containers = state.storage_manager.all_containers().count();
                    if total_containers > 0 && !CONTAINER_DEBUG_LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        println!("[CONTAINER] {} containers in world, player at ({:.1}, {:.1}, {:.1})",
                            total_containers, player_pos.x, player_pos.y, player_pos.z);
                        for c in state.storage_manager.all_containers() {
                            println!("[CONTAINER]   - {:?} at ({:.1}, {:.1}, {:.1}) dist={:.1}",
                                c.container_type, c.position.x, c.position.y, c.position.z,
                                c.position.distance(player_pos));
                        }
                    }

                    // Collect transforms for each mesh type and LOD level
                    let mut closed_lod0_transforms: Vec<Mat4> = Vec::new();
                    let mut closed_lod1_transforms: Vec<Mat4> = Vec::new();
                    let mut closed_lod2_transforms: Vec<Mat4> = Vec::new();
                    let mut open_lod0_transforms: Vec<Mat4> = Vec::new();
                    let mut open_lod1_transforms: Vec<Mat4> = Vec::new();
                    let mut open_lod2_transforms: Vec<Mat4> = Vec::new();

                    for container in state.storage_manager.all_containers() {
                        let dist = container.position.distance(player_pos);
                        if dist > container_max_distance {
                            continue;
                        }

                        // Chest model origin is at center - lift up by half scaled height
                        // Scale 4.0 with base model height ~0.2 = scaled height ~0.8, half = 0.4
                        let chest_pos = container.position + Vec3::new(0.0, 0.4, 0.0);
                        let transform = Mat4::from_scale_rotation_translation(
                            Vec3::splat(4.0), // 4x scale for visible chest
                            Quat::from_rotation_y(container.rotation),
                            chest_pos,
                        );

                        // Select LOD based on distance: LOD0 (close), LOD1 (medium), LOD2 (far)
                        if container.is_open {
                            if dist < lod0_threshold { open_lod0_transforms.push(transform); }
                            else if dist < lod1_threshold { open_lod1_transforms.push(transform); }
                            else { open_lod2_transforms.push(transform); }
                        } else {
                            if dist < lod0_threshold { closed_lod0_transforms.push(transform); }
                            else if dist < lod1_threshold { closed_lod1_transforms.push(transform); }
                            else { closed_lod2_transforms.push(transform); }
                        }
                    }

                    // Create pipelines for each variant with transforms
                    let shadow_map = shadow_map_mutex.safe_lock();

                    // Helper: get a reusable pipeline from pool (avoids per-frame GPU resource creation)
                    macro_rules! get_pipeline {
                        ($pool:expr, $shadow_map:expr) => {
                            $pool.pop().unwrap_or_else(|| TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format(), $shadow_map))
                        };
                    }

                    // LOD0 - High poly for close range
                    if !closed_lod0_transforms.is_empty() {
                        if let Some(mesh) = state.mesh_registry.get("chest_closed_lod0") {
                            let mut p = get_pipeline!(pipeline_pool, &shadow_map);
                            p.set_mesh(mesh.clone());
                            p.upload_instances(ctx.device(), &closed_lod0_transforms);
                            p.update_camera_full(ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density, 0.0, 1.0);
                            container_pipelines.push(p);
                            rendered_count += closed_lod0_transforms.len();
                        }
                    }
                    if !open_lod0_transforms.is_empty() {
                        if let Some(mesh) = state.mesh_registry.get("chest_open_lod0") {
                            let mut p = get_pipeline!(pipeline_pool, &shadow_map);
                            p.set_mesh(mesh.clone());
                            p.upload_instances(ctx.device(), &open_lod0_transforms);
                            p.update_camera_full(ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density, 0.0, 1.0);
                            container_pipelines.push(p);
                            rendered_count += open_lod0_transforms.len();
                        }
                    }

                    // LOD1 - Medium poly for medium range
                    if !closed_lod1_transforms.is_empty() {
                        if let Some(mesh) = state.mesh_registry.get("chest_closed_lod1") {
                            let mut p = get_pipeline!(pipeline_pool, &shadow_map);
                            p.set_mesh(mesh.clone());
                            p.upload_instances(ctx.device(), &closed_lod1_transforms);
                            p.update_camera_full(ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density, 0.0, 1.0);
                            container_pipelines.push(p);
                            rendered_count += closed_lod1_transforms.len();
                        }
                    }
                    if !closed_lod2_transforms.is_empty() {
                        if let Some(mesh) = state.mesh_registry.get("chest_closed_lod2") {
                            let mut p = get_pipeline!(pipeline_pool, &shadow_map);
                            p.set_mesh(mesh.clone());
                            p.upload_instances(ctx.device(), &closed_lod2_transforms);
                            p.update_camera_full(ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density, 0.0, 1.0);
                            container_pipelines.push(p);
                            rendered_count += closed_lod2_transforms.len();
                        }
                    }
                    if !open_lod1_transforms.is_empty() {
                        if let Some(mesh) = state.mesh_registry.get("chest_open_lod1") {
                            let mut p = get_pipeline!(pipeline_pool, &shadow_map);
                            p.set_mesh(mesh.clone());
                            p.upload_instances(ctx.device(), &open_lod1_transforms);
                            p.update_camera_full(ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density, 0.0, 1.0);
                            container_pipelines.push(p);
                            rendered_count += open_lod1_transforms.len();
                        }
                    }
                    if !open_lod2_transforms.is_empty() {
                        if let Some(mesh) = state.mesh_registry.get("chest_open_lod2") {
                            let mut p = get_pipeline!(pipeline_pool, &shadow_map);
                            p.set_mesh(mesh.clone());
                            p.upload_instances(ctx.device(), &open_lod2_transforms);
                            p.update_camera_full(ctx.queue(), &view_proj, &light_view_proj,
                                sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                fog_color, fog_start, fog_end, fog_density, 0.0, 1.0);
                            container_pipelines.push(p);
                            rendered_count += open_lod2_transforms.len();
                        }
                    }
                    // Generate campfire meshes before dropping shadow_map
                    let nearby_campfires: Vec<_> = state.campfire_manager
                        .campfires_near(state.camera.position, 100.0)
                        .into_iter()
                        .cloned()
                        .collect();

                    {
                        let campfire_cache_mutex = CAMPFIRE_MESH_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
                        let mut campfire_cache = campfire_cache_mutex.safe_lock();

                        for campfire in &nearby_campfires {
                            // Get cached mesh or generate + cache on first encounter
                            let tree_mesh = campfire_cache.entry(campfire.id.0).or_insert_with(|| {
                                let mesh_data = campfire::CampfireMesh::generate(campfire);
                                let (positions, normals, uvs, indices) = mesh_data.to_tree_mesh_data();
                                TreePipeline::create_mesh(ctx.device(), &positions, &normals, &uvs, &indices, None)
                            });

                            if tree_mesh.index_count > 0 {
                                let mut pipeline = get_pipeline!(pipeline_pool, &shadow_map);
                                pipeline.set_mesh(tree_mesh.clone());
                                pipeline.upload_instances(ctx.device(), &[Mat4::IDENTITY]);
                                pipeline.update_camera_no_wind(
                                    ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density, 0.0, 1.0,
                                    LODFadeMode::Disabled, 0.0, 0.0
                                );
                                campfire_pipelines.push(pipeline);
                            }
                        }
                    }

                    drop(shadow_map);

                    // Render all container pipelines
                    for pipeline in &container_pipelines {
                        pipeline.render(&mut render_pass);
                    }

                    // Render all campfire pipelines
                    for pipeline in &campfire_pipelines {
                        pipeline.render(&mut render_pass);
                    }

                    // Render Dropped Weapons (actual 3D models, not orbs)
                    {
                        let shadow_map_weapons = shadow_map_mutex.safe_lock();

                        // Group dropped items by weapon type
                        let mut dagger_transforms: Vec<Mat4> = Vec::new();
                        let mut flintlock_transforms: Vec<Mat4> = Vec::new();
                        let mut hatchet_transforms: Vec<Mat4> = Vec::new();

                        let game_time = state.time_of_day;
                        for drop in state.dropped_items.all_drops() {
                            // Only render weapon items as models
                            if drop.item.item_type != economy::ItemType::Weapon {
                                continue;
                            }

                            let bounce = drop.bounce_offset(game_time);
                            // Position on ground with gentle bob animation
                            let pos = drop.position + Vec3::new(0.0, bounce * 0.05, 0.0);
                            let rotation = Quat::from_rotation_y(drop.rotation + game_time * 0.5); // Slow spin

                            // Scale per weapon type (models have different base sizes)
                            // Models are exported at ~10m scale, need ~0.01-0.03 for realistic world size
                            let scale = match drop.item.template_id.as_str() {
                                "dagger" => 0.015,        // Small blade (~15cm)
                                "flintlock_pistol" => 0.03, // Medium pistol (~30cm)
                                "hatchet" => 0.02,        // Small axe (~20cm)
                                _ => 0.02,
                            };

                            let transform = Mat4::from_scale_rotation_translation(
                                Vec3::splat(scale),
                                rotation,
                                pos,
                            );

                            match drop.item.template_id.as_str() {
                                "dagger" => dagger_transforms.push(transform),
                                "flintlock_pistol" => flintlock_transforms.push(transform),
                                "hatchet" => hatchet_transforms.push(transform),
                                _ => {}
                            }
                        }

                        // Render daggers
                        if !dagger_transforms.is_empty() {
                            if let Some(mesh) = state.mesh_registry.get("dagger_lod0") {
                                let mut p = get_pipeline!(pipeline_pool, &shadow_map_weapons);
                                p.set_mesh(mesh.clone());
                                p.upload_instances(ctx.device(), &dagger_transforms);
                                p.update_camera_no_wind(ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density, 0.0, 1.0,
                                    LODFadeMode::Disabled, 0.0, 0.0);
                                weapon_pipelines.push(p);
                            }
                        }

                        // Render flintlocks
                        if !flintlock_transforms.is_empty() {
                            if let Some(mesh) = state.mesh_registry.get("flintlock_lod0") {
                                let mut p = get_pipeline!(pipeline_pool, &shadow_map_weapons);
                                p.set_mesh(mesh.clone());
                                p.upload_instances(ctx.device(), &flintlock_transforms);
                                p.update_camera_no_wind(ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density, 0.0, 1.0,
                                    LODFadeMode::Disabled, 0.0, 0.0);
                                weapon_pipelines.push(p);
                            }
                        }

                        // Render hatchets
                        if !hatchet_transforms.is_empty() {
                            if let Some(mesh) = state.mesh_registry.get("hatchet_lod0") {
                                let mut p = get_pipeline!(pipeline_pool, &shadow_map_weapons);
                                p.set_mesh(mesh.clone());
                                p.upload_instances(ctx.device(), &hatchet_transforms);
                                p.update_camera_no_wind(ctx.queue(), &view_proj, &light_view_proj,
                                    sun_dir.to_array(), elapsed, state.camera.position.to_array(),
                                    fog_color, fog_start, fog_end, fog_density, 0.0, 1.0,
                                    LODFadeMode::Disabled, 0.0, 0.0);
                                weapon_pipelines.push(p);
                            }
                        }
                    }

                    for pipeline in &weapon_pipelines {
                        pipeline.render(&mut render_pass);
                    }

                    rendered_count
                };

                let campfires_rendered = campfire_pipelines.len();

                // Render Animal Models (3D models for species that have them)
                model_pipeline.update_camera(
                    ctx.queue(),
                    &view_proj,
                    &light_view_proj,
                    state.camera.position,
                    state.time_of_day,
                    sun_dir,
                    Vec3::from_array(fog_color),
                    fog_start,
                    fog_end,
                    1.5, // fog_density
                    state.weather.ambient_dimming(),
                    0.8, // shadow_strength - strong shadows for contrast
                    state.weather.rain_intensity(),
                );
                model_pipeline.render(&mut render_pass);

                // Render Animal Orbs (fallback for species without 3D models)
                orb_pipeline.update_camera(ctx.queue(), &view_proj, state.camera.position);
                orb_pipeline.render(&mut render_pass);

                // Render Water
                water_system_guard.draw(&mut render_pass);

                // Render Pond/Lake Water (inland bodies)
                pond_water_guard.draw(&mut render_pass);

                // Render Rain Particles (skip when not raining)
                if state.weather.rain_intensity() > 0.01 {
                    rain_pipeline.update(
                        ctx.queue(),
                        &view_proj,
                        state.camera.position,
                        state.camera.right(),
                        state.camera.up,
                        elapsed,
                        state.weather.rain_intensity(),
                        state.weather.wind_strength(),
                        Vec3::from_array(fog_color),
                        fog_start,
                        fog_end,
                    );
                    rain_pipeline.render(&mut render_pass);
                }

                // Render Ember Particles (skip when no campfires nearby)
                // Reuses campfire_light_data queried earlier (same spatial query)
                if !campfire_light_data.is_empty() {
                    ember_pipeline.update(
                        ctx.queue(),
                        &view_proj,
                        state.camera.position,
                        state.camera.right(),
                        state.camera.up,
                        elapsed,
                        &campfire_light_data,
                    );
                    ember_pipeline.render(&mut render_pass);
                }

                // Render Bioluminescent Orbs (collected during main chunk loop above)
                if !all_bio_orbs.is_empty() {
                    bio_orb_pipeline.upload_instances(ctx.device(), &all_bio_orbs);
                    bio_orb_pipeline.update_camera(ctx.queue(), &view_proj, state.camera.position, elapsed);
                    bio_orb_pipeline.render(&mut render_pass);
                }

                // Log culling stats occasionally (every ~60 frames)
                static FRAME_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let frame = FRAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if frame % 300 == 1 {
                    println!("[RENDER STATS] terrain={}, grass={}, trees={}/lod1:{}, shrubs={}, ferns={}, rocks={}, boulders={}, buildings={}, containers={}",
                        terrain_rendered, grass_rendered, trees_rendered, trees_lod1_rendered,
                        shrubs_rendered, ferns_rendered, rocks_rendered, boulders_rendered, buildings_rendered, containers_rendered);
                }
                let _ = (terrain_rendered, terrain_culled, grass_rendered, trees_rendered, trees_lod1_rendered,
                         rocks_rendered, boulders_rendered, shrubs_rendered, ferns_rendered, dead_logs_rendered, buildings_rendered, containers_rendered);

                drop(render_pass);

                // Return pipelines to free list for reuse next frame
                pipeline_free_list.extend(container_pipelines.drain(..));
                pipeline_free_list.extend(campfire_pipelines.drain(..));
                pipeline_free_list.extend(weapon_pipelines.drain(..));
                pipeline_free_list.extend(pipeline_pool.drain(..));
            } // End Main Pass

            // 2.5 Light Shaft Post-Process Pass
            if offscreen_view.is_some() {
                let mut light_shaft_pipeline = light_shaft_pipeline_mutex.safe_lock();

                // Apply light shafts when sun is visible and intense enough
                let sun_screen_pos = LightShaftPipeline::calculate_sun_screen_pos(sun_dir, view_proj);
                let atmo = &state.atmosphere.state;
                if let Some(pos) = sun_screen_pos {
                    if atmo.light_shaft_intensity > 0.01 && sun_pos_y > 0.0 {
                        light_shaft_pipeline.update_uniforms(
                            ctx.queue(), pos,
                            atmo.light_shaft_intensity, atmo.light_shaft_decay, atmo.light_shaft_density,
                        );
                    } else {
                        light_shaft_pipeline.update_uniforms(ctx.queue(), [0.5, 0.5], 0.0, 0.96, 0.5);
                    }
                } else {
                    light_shaft_pipeline.update_uniforms(ctx.queue(), [0.5, 0.5], 0.0, 0.96, 0.5);
                }

                // Cached bind group — only created on first frame or after resize invalidation
                light_shaft_pipeline.ensure_bind_group(
                    ctx.device(), offscreen_view.unwrap(),
                );

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Light Shaft Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                light_shaft_pipeline.render_cached(&mut pass);
            }

            // 3. Weapon Viewmodel Pass (First-person weapon)
            if state.game_state == GameState::Playing {
                let weapon_pipeline = weapon_viewmodel_mutex.safe_lock();
                if weapon_pipeline.has_weapon() {
                    weapon_pipeline.update_uniforms(
                        ctx.queue(),
                        state.camera.aspect_ratio,
                        state.swing_animation.swing_progress,
                        elapsed,
                        state.player.velocity,
                        state.swing_animation.muzzle_flash,
                    );

                    let mut viewmodel_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Weapon Viewmodel Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: ctx.depth_view(),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    weapon_pipeline.render(&mut viewmodel_pass);
                }
            }

            // 4. Egui Pass
            {
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [ctx.config().width, ctx.config().height],
                    pixels_per_point: ctx.window.scale_factor() as f32,
                };

                let tris = state.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                let mut renderer = egui_renderer_mutex.safe_lock();
                for (id, image_delta) in &full_output.textures_delta.set {
                    renderer.update_texture(ctx.device(), ctx.queue(), *id, image_delta);
                }

                renderer.update_buffers(
                    ctx.device(),
                    ctx.queue(),
                    &mut encoder,
                    &tris,
                    &screen_descriptor,
                );

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Egui Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    renderer.render(&mut render_pass, &tris, &screen_descriptor);
                }

                for id in &full_output.textures_delta.free {
                    renderer.free_texture(id);
                }
            }

            ctx.queue().submit(std::iter::once(encoder.finish()));
            output.present();
        } else {
            // Menu or Loading rendering (just egui)
            let output = ctx.surface.get_current_texture().unwrap();
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = ctx.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Menu Render Encoder"),
            });

            // Clear screen
            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }

            // Egui Pass
            {
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [ctx.config().width, ctx.config().height],
                    pixels_per_point: ctx.window.scale_factor() as f32,
                };

                let tris = state.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                let mut renderer = egui_renderer_mutex.safe_lock();
                for (id, image_delta) in &full_output.textures_delta.set {
                    renderer.update_texture(ctx.device(), ctx.queue(), *id, image_delta);
                }

                renderer.update_buffers(
                    ctx.device(),
                    ctx.queue(),
                    &mut encoder,
                    &tris,
                    &screen_descriptor,
                );

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Egui Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    renderer.render(&mut render_pass, &tris, &screen_descriptor);
                }

                for id in &full_output.textures_delta.free {
                    renderer.free_texture(id);
                }
            }

            ctx.queue().submit(std::iter::once(encoder.finish()));
            output.present();
        }
    });

    // Run
    if let Err(e) = app.run() {
        eprintln!("Engine error: {}", e);
    }
}
