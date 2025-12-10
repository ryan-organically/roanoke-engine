// Allow dead code for planned features not yet integrated
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use croatoan_core::{App, CursorGrabMode, DeviceEvent, ElementState, KeyCode, MouseButton, PhysicalKey, WinitEvent as Event, WinitWindowEvent as WindowEvent};
use croatoan_wfc::{generate_terrain_chunk, generate_vegetation_for_chunk, generate_trees_for_chunk, generate_detritus_for_chunk, generate_rocks_for_chunk, generate_buildings_for_chunk, generate_foliage_for_chunk, FoliageInstances};
use croatoan_render::{Camera, TerrainPipeline, TerrainTextures, ShadowMap, ShadowPipeline, GrassPipeline, TreePipeline, TreeMesh, DetritusPipeline, BuildingPipeline, BuildingMesh, BuildingVertex, Frustum, ChunkBounds, SunPipeline, SkyPipeline, ViewModelPipeline, LightShaftPipeline, AnimalOrbPipeline, OrbInstance, AnimalModelPipeline, AnimalVertex, AnimalInstance, FoliagePipeline, FoliageVertex};
use croatoan_procgen::{generate_simple_tree_mesh, RockRecipe, generate_rock, BuildingRecipe, generate_building};
use glam::{Vec3, Mat4};
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
mod ecology;
mod naval;
mod weather;
mod systems_manager;
mod character_agent;

use water_system::WaterSystem;
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

// Swing animation state for viewmodel
struct SwingAnimation {
    is_swinging: bool,
    swing_progress: f32,  // 0.0 to 1.0
    swing_duration: f32,  // Total animation duration in seconds
    hit_processed: bool,  // Whether hit was processed this swing
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
    master_volume: f32,
    // Swing Animation
    swing_animation: SwingAnimation,
    atmosphere: AtmosphereEngine,
    show_load_menu: bool, // For Load Game submenu
    // Animal System
    animal_manager: AnimalManager,
    animal_spawner: AnimalSpawner,
    // Village System
    village_manager: VillageManager,
    // Progression System
    game_progression: GameProgression,
    // Economy System
    economy_manager: economy::EconomyManager,
    player_economy: economy::PlayerEconomy,
    dropped_items: economy::DroppedItemManager,
    // Hotbar (quick-slot for inventory access)
    active_hotbar_slot: usize, // 0-9, maps to first 10 inventory slots
    // Combat state
    combat_kill_time: f32, // Tracks time spent fighting current target
    // Debug
    debug_timer: f32,
    fog_level: u8, // 0=Off, 1=Light, 2=Medium, 3=Heavy, 4=Dense
    // Audio state tracking
    was_in_village: bool,
    // Data Pipeline
    data_pipeline: DataPipeline,
    npc_audio: NpcAudioIntegration,
    progression_audio: ProgressionAudioBridge,
    faction_audio: FactionAudioBridge,
    // Systems Manager (encyclopedia, flora, ecology, weather coordination)
    systems_manager: systems_manager::SystemsManager,
    // Dialogue UI state
    current_dialogue: Option<npc::interaction::DialogueUIData>,
    // Character Sheet (Tab menu)
    character_sheet_tab: CharacterSheetTab,
    character_preview_rotation: f32,  // Y-axis rotation for 3D model preview
    character_preview_dragging: bool, // Is user dragging to rotate?
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

                // Item name (truncated)
                let name: String = item.name.chars().take(6).collect();
                ui.painter().text(
                    slot_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    name,
                    egui::FontId::proportional(10.0),
                    ink_color,
                );

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
        render_distance: 150.0, // Reduced from 250 for better FPS
        master_volume: 80.0, // 0-100 scale, 80 = default
        swing_animation: SwingAnimation {
            is_swinging: false,
            swing_progress: 0.0,
            swing_duration: 0.35, // 350ms swing - fast like Minecraft
            hit_processed: false,
        },
        atmosphere: AtmosphereEngine::new(),
        show_load_menu: false,
        // Animal System
        animal_manager: AnimalManager::new(Difficulty::Normal),
        animal_spawner: AnimalSpawner::new(12345), // Will be re-seeded when game starts
        // Village System
        village_manager: VillageManager::new(12345), // Will be re-seeded when game starts
        // Progression System
        game_progression: GameProgression::new(),
        // Economy System
        economy_manager: economy::EconomyManager::new(),
        player_economy: economy::PlayerEconomy::new(),
        dropped_items: economy::DroppedItemManager::new(),
        // Hotbar
        active_hotbar_slot: 0,
        // Combat state
        combat_kill_time: 0.0,
        // Debug
        debug_timer: 0.0,
        fog_level: 0, // Start with fog off
        // Audio state tracking
        was_in_village: false,
        // Data Pipeline
        data_pipeline: DataPipeline::new(),
        npc_audio: NpcAudioIntegration::new(),
        progression_audio: ProgressionAudioBridge::new(),
        faction_audio: FactionAudioBridge::new(),
        // Systems Manager (will be re-seeded when game starts)
        systems_manager: systems_manager::SystemsManager::new(12345),
        // Dialogue state
        current_dialogue: None,
        // Character Sheet (Tab menu)
        character_sheet_tab: CharacterSheetTab::Inventory,
        character_preview_rotation: 0.0,
        character_preview_dragging: false,
    }));

    // ... (Channel setup) ...
    // Response Data: (Terrain, Grass, Trees, Detritus, Rocks, Coord X, Coord Z)
    type ChunkData = (
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>, // Terrain
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<f32>, Vec<u32>, // Grass (pos, col, local_height, idx)
        Vec<Mat4>, // Trees (Instanced)
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>, // Detritus
        Vec<(String, Mat4)>, // Rocks (Named Instances)
        Vec<(String, Mat4)>, // Buildings (Named Instances)
        i32, i32 // Offsets (World Space)
    );
    
    // Channel for requesting chunks
    let (request_tx, request_rx): (Sender<ChunkRequest>, Receiver<ChunkRequest>) = channel();
    // Channel for receiving generated chunks
    let (chunk_tx, chunk_rx): (Sender<ChunkData>, Receiver<ChunkData>) = channel();
    
    let chunk_rx = Arc::new(Mutex::new(chunk_rx));

    // Spawn Persistent Generation Thread
    thread::spawn(move || {
        println!("[GEN] Generation thread started.");
        while let Ok(req) = request_rx.recv() {
            println!("[GEN] Received request for chunk ({}, {})", req.coord.x, req.coord.z);
            let chunk_world_size = 256.0;
            let chunk_resolution = 64;
            let scale = 4.0;
            let (offset_x, offset_z) = req.coord.world_offset(chunk_world_size);
            let offset_x = offset_x as i32;
            let offset_z = offset_z as i32;

            // Generate terrain
            println!("[GEN] Generating terrain...");
            let (terrain_pos, terrain_col, terrain_nrm, terrain_idx) =
                generate_terrain_chunk(req.seed, chunk_resolution, offset_x, offset_z, scale);

            // Generate grass
            let (grass_pos, grass_col, grass_heights, grass_idx) = generate_vegetation_for_chunk(
                req.seed,
                chunk_world_size,
                offset_x as f32,
                offset_z as f32,
            );

            // Generate trees
            let tree_instances = generate_trees_for_chunk(
                req.seed,
                chunk_world_size,
                offset_x as f32,
                offset_z as f32,
            );

            // Generate detritus
            let (det_pos, det_nrm, det_uv, det_idx) = generate_detritus_for_chunk(
                req.seed,
                chunk_world_size,
                offset_x as f32,
                offset_z as f32,
            );

            // Generate rocks
            let rock_instances = generate_rocks_for_chunk(
                req.seed,
                chunk_world_size,
                offset_x as f32,
                offset_z as f32,
            );

            // Generate buildings
            let building_instances = generate_buildings_for_chunk(
                req.seed,
                chunk_world_size,
                offset_x as f32,
                offset_z as f32,
            );

            // Send result
            println!("[GEN] Chunk ({}, {}) generated, sending to main thread...", req.coord.x, req.coord.z);
            if chunk_tx.send((
                terrain_pos, terrain_col, terrain_nrm, terrain_idx,
                grass_pos, grass_col, grass_heights, grass_idx,
                tree_instances,
                det_pos, det_nrm, det_uv, det_idx,
                rock_instances,
                building_instances,
                offset_x, offset_z
            )).is_err() {
                println!("[GEN] Receiver dropped, stopping thread.");
                break;
            }
            println!("[GEN] Chunk ({}, {}) sent successfully!", req.coord.x, req.coord.z);
        }
    });

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
        if let Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } = event {
            if let PhysicalKey::Code(KeyCode::Tab) = key_event.physical_key {
                if key_event.state == ElementState::Pressed {
                    if state.game_state == GameState::Playing {
                        state.game_state = GameState::Paused;
                        state.pause_menu_page = PauseMenuPage::CharacterSheet;
                        state.character_sheet_tab = CharacterSheetTab::Inventory;
                        println!("[MENU] Character sheet opened");
                        return; // Don't pass Tab to egui or other handlers
                    } else if state.game_state == GameState::Paused && state.pause_menu_page == PauseMenuPage::CharacterSheet {
                        state.game_state = GameState::Playing;
                        println!("[MENU] Character sheet closed");
                        return; // Don't pass Tab to egui or other handlers
                    }
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

        // Handle Game Input (only if Playing, not during Loading)
        if state.game_state == GameState::Playing {
            match event {
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                    // Mouse Look - convert 0-100 scale to actual sensitivity (50 = 0.002)
                    let sensitivity = state.mouse_sensitivity / 25000.0;
                    state.player.yaw += delta.0 as f32 * sensitivity;
                    state.player.pitch -= delta.1 as f32 * sensitivity;
                    state.player.pitch = state.player.pitch.clamp(-1.5, 1.5);
                }
                Event::WindowEvent { event: WindowEvent::MouseInput { state: button_state, button, .. }, .. } => {
                    // Left mouse click triggers swing animation
                    if *button == MouseButton::Left && *button_state == ElementState::Pressed {
                        if !state.swing_animation.is_swinging {
                            state.swing_animation.is_swinging = true;
                            state.swing_animation.swing_progress = 0.0;
                            state.swing_animation.hit_processed = false;
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
                                    state.weather.set_weather(prev, false);
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
                                    state.weather.set_weather(next, false);
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
                                        if let Some(removed) = state.player_economy.inventory.remove_item(item_id) {
                                            // Drop in front of player
                                            let drop_pos = state.player.position + state.camera.forward() * 1.5 + Vec3::new(0.0, 1.0, 0.0);
                                            state.dropped_items.spawn_drop(removed, drop_pos);
                                            log::info!("[DROP] {} from hotbar slot {}", item.name, slot + 1);
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
                // - tree_instances from generate_trees_for_chunk() for placement
                // ========================================================================
                // Create a temporary TreePipeline to get access to texture_bind_group_layout
                let texture_helper = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());

                // Load tree models from assets/models/trees/
                // Each tree GLB contains multiple meshes (bark + leaves) with different textures.
                // We create TWO meshes per tree: one for bark (OPAQUE), one for leaves (BLEND).
                let tree_models = ["tree_0", "tree_1"];
                let mut tree_cache = gltf_loader::ModelCache::new("assets/models/trees");
                for name in &tree_models {
                    if let Some(model) = tree_cache.load(name) {
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
                        // Leaves below this threshold (50% up from bottom) will be culled - halves leaf density
                        let leaf_cull_height = min_y + height_range * 0.50;

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
                            println!("[FOLIAGE] Registered {}_bark: {} verts, {} tris",
                                name, bark_positions.len(), bark_indices.len() / 3);
                        }

                        // Create leaves mesh
                        if !leaf_positions.is_empty() {
                            let texture_bind_group = leaf_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_leaf_texture", name)),
                                );
                                println!("[FOLIAGE] Created leaf texture for {}: {}x{}", name, tex_data.width, tex_data.height);
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_leaf_bind", name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &leaf_positions, &leaf_normals, &leaf_uvs, &leaf_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_leaves", name), gpu_mesh);
                            println!("[FOLIAGE] Registered {}_leaves: {} verts, {} tris",
                                name, leaf_positions.len(), leaf_indices.len() / 3);
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
                                    Some(&format!("{}_combined_texture", name)),
                                );
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_combined_bind", name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &all_positions, &all_normals, &all_uvs, &all_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(name.to_string(), gpu_mesh);
                            println!("[FOLIAGE] Registered {} (combined): {} verts", name, all_positions.len());
                        }
                    } else {
                        println!("[FOLIAGE] WARNING: Tree model '{}' not found", name);
                    }
                }

                // Load shrub/bush models from assets/models/shrubs/
                // Same pattern as trees: separate bark and leaves meshes
                let shrub_models = ["shrub_0", "bush_0", "grass_0"];
                let mut shrub_cache = gltf_loader::ModelCache::new("assets/models/shrubs");
                for name in &shrub_models {
                    if let Some(model) = shrub_cache.load(name) {
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
                            println!("[FOLIAGE] Registered {}_bark: {} verts", name, bark_positions.len());
                        }

                        // Create leaves mesh
                        if !leaf_positions.is_empty() {
                            let texture_bind_group = leaf_texture.map(|tex_data| {
                                let (_gpu_tex, tex_view) = gltf_loader::create_gpu_texture(
                                    ctx.device(), ctx.queue(), tex_data,
                                    Some(&format!("{}_leaf_texture", name)),
                                );
                                println!("[FOLIAGE] Created leaf texture for {}: {}x{}", name, tex_data.width, tex_data.height);
                                std::sync::Arc::new(texture_helper.create_texture_bind_group(
                                    ctx.device(), &tex_view, Some(&format!("{}_leaf_bind", name)),
                                ))
                            });
                            let gpu_mesh = TreePipeline::create_mesh(
                                ctx.device(), &leaf_positions, &leaf_normals, &leaf_uvs, &leaf_indices,
                                texture_bind_group,
                            );
                            state.mesh_registry.insert(format!("{}_leaves", name), gpu_mesh);
                            println!("[FOLIAGE] Registered {}_leaves: {} verts", name, leaf_positions.len());
                        }
                    } else {
                        println!("[FOLIAGE] WARNING: Shrub model '{}' not found", name);
                    }
                }

                // 2. Rocks - All Types (boulder, pebble, small, medium, flat, mossy)
                let rock_types: Vec<(RockRecipe, &str)> = vec![
                    (RockRecipe::boulder(), "rock_boulder"),
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
            // Initialize radius based on default render_distance (150)
            manager.update_radius_for_render_distance(150.0);
            Mutex::new(manager)
        });

        // Shadow System
        static SHADOW_SYSTEM: OnceLock<(Mutex<ShadowMap>, Mutex<ShadowPipeline>)> = OnceLock::new();
        let (shadow_map_mutex, shadow_pipeline_mutex) = SHADOW_SYSTEM.get_or_init(|| {
            let shadow_map = ShadowMap::new(ctx.device(), 1024); // Reduced from 2048 for FPS
            let shadow_pipeline = ShadowPipeline::new(ctx.device());
            (Mutex::new(shadow_map), Mutex::new(shadow_pipeline))
        });

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
            let tree_pipeline = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());
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

        // Water System
        static WATER_SYSTEM: OnceLock<Mutex<WaterSystem>> = OnceLock::new();
        // let water_system_mutex = WATER_SYSTEM.get_or_init(|| {
        //     Mutex::new(WaterSystem::new(ctx.device(), ctx.surface_format()))
        // });

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
        let animal_model_pipeline_mutex = ANIMAL_MODEL_PIPELINE.get_or_init(|| {
            Mutex::new(AnimalModelPipeline::new_with_queue(ctx.device(), Some(ctx.queue()), ctx.surface_format()))
        });

        // Animal Model Cache (loads GLTF models)
        static ANIMAL_MODEL_CACHE: OnceLock<Mutex<gltf_loader::ModelCache>> = OnceLock::new();
        let animal_model_cache_mutex = ANIMAL_MODEL_CACHE.get_or_init(|| {
            let mut cache = gltf_loader::ModelCache::new("assets/models/animals");
            // Preload available models
            cache.preload(&["Wolf", "Deer", "Stag", "Horse", "Donkey", "Fox", "Husky"]);
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
            }
        }

        let mut state = render_state.safe_lock();

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

            // Update dropped items (despawn timer)
            state.dropped_items.update(delta);

            // Update Systems Manager (encyclopedia, flora, ecology pipelines)
            // This coordinates data flow between weather, ecology, observations
            state.update_systems(delta);

            // Update Audio System (responds to weather, time, game state)
            let time_normalized = state.time_of_day / 24.0; // Normalize to 0.0-1.0
            let current_weather = state.weather.current_weather;
            state.audio_system.update(delta, current_weather, time_normalized);

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

                    // Find closest animal in attack range
                    let attack_range = 3.0; // Melee range in world units
                    if let Some((animal_id, dist)) = animals::combat::find_closest_animal(
                        &state.animal_manager,
                        state.player.position,
                        attack_range,
                    ) {
                        // Base damage from weapon (could be modified by equipped weapon)
                        let base_damage = 25.0;
                        let combat_ctx = animals::combat::CombatContext::default();

                        // Process the attack
                        if let Some(result) = animals::combat::player_attack_animal(
                            &mut state.animal_manager,
                            animal_id,
                            base_damage,
                            Some("hunter_knife"), // TODO: Get from equipped weapon
                            &combat_ctx,
                        ) {
                            // Process loot if killed
                            if result.killed {
                                let loot_result = state.process_combat_loot(&result, "hunter_knife");

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

            // Update Atmosphere (fog, light shafts based on time/weather)
            // Pass render_distance so fog_end is scaled to hide object pop-in
            let weather_fog = match state.weather.current_weather {
                WeatherType::Foggy => 0.8,
                WeatherType::Overcast => 0.3,
                WeatherType::Stormy => 0.5,
                _ => 0.0,
            };
            let time_of_day = state.time_of_day;
            let cloud_coverage = state.weather.cloud_coverage;
            let render_dist = state.render_distance;
            state.atmosphere.update(time_of_day, weather_fog, cloud_coverage, render_dist);

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

                    // Combine base rotation with IK pelvis tilt
                    let final_rotation = animal.rotation * ik_tilt;

                    // Model position - animal.position.y is already ground height
                    let model_position = animal.position + Vec3::new(0.0, y_offset, 0.0);

                    let model_matrix = Mat4::from_scale_rotation_translation(
                        Vec3::splat(model_scale),
                        final_rotation,
                        model_position,
                    );
                    let instance = AnimalInstance::new(model_matrix, color, emissive);
                    model_instances.entry(model_name).or_insert_with(Vec::new).push(instance);
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

                            // Find the first mesh with a texture
                            let mut texture_data: Option<&gltf_loader::LoadedTexture> = None;

                            for mesh in &loaded_model.meshes {
                                let vertex_offset = all_vertices.len() as u32;
                                for i in 0..mesh.positions.len() {
                                    all_vertices.push(AnimalVertex {
                                        position: mesh.positions[i],
                                        normal: mesh.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                                        uv: mesh.uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                                    });
                                }
                                for idx in &mesh.indices {
                                    all_indices.push(*idx + vertex_offset);
                                }

                                // Get texture from first mesh that has one
                                if texture_data.is_none() {
                                    if let Some(ref tex) = mesh.material.base_color_texture_data {
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

                            // Upload texture if available
                            if let Some(tex) = texture_data {
                                model_pipeline.upload_species_texture(
                                    ctx.device(),
                                    ctx.queue(),
                                    model_name,
                                    &tex.data,
                                    tex.width,
                                    tex.height,
                                );
                            }
                        }
                    }

                    // Upload instances for this species
                    model_pipeline.upload_instances(ctx.device(), model_name, instances);
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

            // Add dropped item orbs
            let game_time = state.time_of_day; // For bounce animation
            for drop in state.dropped_items.all_drops() {
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
            state.player.update(delta, input_dir, seed);

            // Restore original speed
            state.player.speed = original_speed;

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


        // Moon Billboard (Reusing SunPipeline)
        static MOON_PIPELINE: OnceLock<Mutex<SunPipeline>> = OnceLock::new();
        let moon_pipeline_mutex = MOON_PIPELINE.get_or_init(|| {
            Mutex::new(SunPipeline::new(ctx.device(), ctx.surface_format()))
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

            // Sync Cursor State with Game State
            match state.game_state {
                GameState::Menu | GameState::Loading | GameState::Paused => {
                    ctx.window.set_cursor_visible(true);
                    let _ = ctx.window.set_cursor_grab(CursorGrabMode::None);
                }
                GameState::Playing => {
                    // Lock cursor to window and hide it during gameplay
                    ctx.window.set_cursor_visible(false);
                    let _ = ctx.window.set_cursor_grab(CursorGrabMode::Confined);
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
                            "v0.0.1",
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
                                                // Initialize village system
                                                let player_pos = state.player.position;
                                                state.village_manager = VillageManager::new(data.seed);
                                                state.village_manager.discover_villages(
                                                    player_pos,
                                                    2000.0, // 2km radius
                                                    10,     // max 10 villages
                                                );
                                                // Spawn tame animals (horses, donkeys) in villages
                                                {
                                                    let village_data = state.village_manager.get_village_spawn_data();
                                                    let seed = state.village_manager.get_seed();
                                                    spawn_village_animals(&mut state.animal_manager, &village_data, seed);
                                                }
                                                // Spawn wild horse herds on beaches
                                                spawn_beach_horses(&mut state.animal_manager, data.seed);
                                                // Register village factions
                                                register_village_factions(&mut *state);
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
                                        state.player = Player::new(Vec3::new(0.0, 50.0, 0.0));
                                        println!("[GAME] Starting new game with seed: {}", seed);
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
                                        // Initialize village system
                                        let player_pos = state.player.position;
                                        state.village_manager = VillageManager::new(seed);
                                        state.village_manager.discover_villages(
                                            player_pos,
                                            2000.0, // 2km radius
                                            10,     // max 10 villages
                                        );
                                        // Spawn tame animals (horses, donkeys) in villages
                                        {
                                            let village_data = state.village_manager.get_village_spawn_data();
                                            let seed = state.village_manager.get_seed();
                                            spawn_village_animals(&mut state.animal_manager, &village_data, seed);
                                        }
                                        // Spawn wild horse herds on beaches
                                        spawn_beach_horses(&mut state.animal_manager, seed);
                                        // Register village factions
                                        register_village_factions(&mut *state);
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
                                        // Initialize village system
                                        let player_pos = state.player.position;
                                        state.village_manager = VillageManager::new(data.seed);
                                        state.village_manager.discover_villages(
                                            player_pos,
                                            2000.0, // 2km radius
                                            10,     // max 10 villages
                                        );
                                        // Register village factions
                                        register_village_factions(&mut *state);
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
                                                let color = match item.rarity {
                                                    economy::Rarity::Crude => egui::Color32::GRAY,
                                                    economy::Rarity::Common => egui::Color32::WHITE,
                                                    economy::Rarity::Uncommon => egui::Color32::GREEN,
                                                    economy::Rarity::Rare => egui::Color32::from_rgb(0, 112, 221),
                                                    economy::Rarity::Epic => egui::Color32::from_rgb(163, 53, 238),
                                                    economy::Rarity::Legendary => egui::Color32::from_rgb(255, 128, 0),
                                                    economy::Rarity::Mythic => egui::Color32::from_rgb(230, 30, 30),
                                                    economy::Rarity::Primordial => egui::Color32::from_rgb(255, 215, 0),
                                                };
                                                // Item icon (first char of name)
                                                let icon = item.name.chars().next().unwrap_or('?');
                                                ui.label(egui::RichText::new(icon.to_string()).color(color).size(18.0));
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
                    if state.current_dialogue.is_none() {
                        if let Some((name, role, distance)) = state.village_manager.get_focused_npc_info() {
                            egui::Area::new(egui::Id::new("npc_interact_prompt"))
                                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 100.0))
                                .show(ui_ctx, |ui| {
                                    let bg = egui::Frame::none()
                                        .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180))
                                        .rounding(egui::Rounding::same(8.0))
                                        .inner_margin(egui::Margin::same(12.0));
                                    bg.show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(egui::RichText::new(format!("{} - {}", name, role))
                                                .color(egui::Color32::WHITE)
                                                .size(18.0));
                                            ui.label(egui::RichText::new(format!("{:.1}m away", distance))
                                                .color(egui::Color32::GRAY)
                                                .size(12.0));
                                            ui.add_space(4.0);
                                            ui.label(egui::RichText::new("[E] Talk")
                                                .color(egui::Color32::from_rgb(100, 200, 255))
                                                .size(14.0));
                                        });
                                    });
                                });
                        }
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

                    // === Debug window (existing) ===
                    egui::Window::new("Game Menu").show(ui_ctx, |ui| {
                        ui.label(format!("FPS: {:.1}", state.fps));
                        let hours = state.time_of_day as u32;
                        let minutes = ((state.time_of_day - hours as f32) * 60.0) as u32;
                        ui.label(format!("Time: {:02}:{:02}", hours, minutes));
                        ui.label("T/Y keys: Change time");
                        ui.separator();

                        // Dev Stats - Weather/Fog/Render
                        ui.collapsing("Dev Stats", |ui| {
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
    };
                            save_game(&state.save_name_input, &data);
                        }
                        if ui.button("Back to Menu").clicked() {
                            state.game_state = GameState::Menu;
                        }
                        ui.label(format!("Camera: {:.1?}", state.camera.position));
                    });
                }
                GameState::Paused => {
                    // Character Sheet uses different layout than normal pause menu
                    if state.pause_menu_page == PauseMenuPage::CharacterSheet {
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
                                    if ui.add_sized([200.0, 40.0], egui::Button::new("Settings")).clicked() {
                                        state.pause_menu_page = PauseMenuPage::Settings;
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
                                        ui.add(egui::Slider::new(&mut state.movement_speed, 1.0..=30.0)
                                            .text("Speed")
                                            .custom_formatter(|n, _| format!("{:.0}", n)));
                                    });
                                    ui.add_space(15.0);

                                    // Render Distance (minimum 200 to ensure chunks load)
                                    ui.label(egui::RichText::new("Render Distance:").color(egui::Color32::BLACK));
                                    let old_render_dist = state.render_distance;
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 300.0) / 2.0);
                                        ui.add(egui::Slider::new(&mut state.render_distance, 75.0..=200.0)
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
                                    ui.label("Tab - Character Sheet (Inventory/Skills/Commendations)");
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
                let requests = manager.update(state.player.position, state.seed);
                for req in requests {
                    let _ = request_tx.send(req);
                }
                
                // Update Loading Progress stats
                state.loading_progress.chunks_generated = manager.chunk_count(); // Approximation
            }

            // Check for new chunks from background thread
            if let Ok(rx) = render_rx.try_lock() {
                // During Loading: Process 1 chunk per frame
                // During Playing: Process up to 2 chunks per frame to avoid stutter
                let chunks_per_frame = if state.game_state == GameState::Loading { 1 } else { 2 };
                for _ in 0..chunks_per_frame {
                    match rx.try_recv() {
                        Ok((terrain_pos, terrain_col, terrain_nrm, terrain_idx,
                            grass_pos, grass_col, grass_heights, grass_idx,
                            tree_instances,
                            det_pos, det_nrm, det_uv, det_idx,
                            rock_instances,
                            building_instances,
                            offset_x, offset_z)) => {

                            // Debug: Show generation counts
                            println!("[CHUNK] Received chunk ({}, {}): trees={}, rocks={}, detritus_verts={}",
                                offset_x, offset_z, tree_instances.len(), rock_instances.len(), det_pos.len());

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

                            let mut grass_pipeline = None;
                            if !grass_pos.is_empty() {
                                let shadow_map = shadow_map_mutex.safe_lock();
                                let mut gp = GrassPipeline::new(ctx.device(), ctx.surface_format(), &shadow_map);
                                drop(shadow_map);
                                gp.upload_mesh(ctx.device(), ctx.queue(), &grass_pos, &grass_col, &grass_heights, &grass_idx);
                                grass_pipeline = Some(gp);
                            }

                            // FOLIAGE: Create pipelines for trees and shrubs
                            let mut foliage_pipelines: Vec<TreePipeline> = Vec::new();
                            let tree_model_names = ["tree_0", "tree_1"];
                            let shrub_model_names = ["shrub_0", "bush_0"];

                            // Group trees by model
                            let mut tree_groups: std::collections::HashMap<String, Vec<Mat4>> = std::collections::HashMap::new();
                            for (i, transform) in tree_instances.iter().enumerate() {
                                let name = tree_model_names[i % tree_model_names.len()].to_string();
                                tree_groups.entry(name).or_default().push(*transform);
                            }
                            for (name, transforms) in &tree_groups {
                                // Render bark mesh (OPAQUE)
                                let bark_name = format!("{}_bark", name);
                                if let Some(mesh) = state.mesh_registry.get(&bark_name) {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), transforms);
                                    foliage_pipelines.push(tp);
                                    println!("[FOLIAGE] Tree '{}' bark: {} instances", name, transforms.len());
                                }
                                // Render leaves mesh (BLEND) with same transforms
                                let leaves_name = format!("{}_leaves", name);
                                if let Some(mesh) = state.mesh_registry.get(&leaves_name) {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), transforms);
                                    foliage_pipelines.push(tp);
                                    println!("[FOLIAGE] Tree '{}' leaves: {} instances", name, transforms.len());
                                }
                            }

                            // Generate shrubs around trees with randomization
                            let mut shrub_groups: std::collections::HashMap<String, Vec<Mat4>> = std::collections::HashMap::new();
                            let all_shrub_models = ["shrub_0", "bush_0", "grass_0"];

                            for (_i, t) in tree_instances.iter().enumerate() {
                                let pos = t.w_axis;

                                // Create deterministic RNG seeded from tree position
                                use rand::SeedableRng;
                                use rand::Rng;
                                let pos_seed = ((pos.x * 1000.0) as u64)
                                    .wrapping_add((pos.z * 1000.0) as u64)
                                    .wrapping_mul(2654435761); // Knuth's multiplicative hash
                                let mut rng = rand::rngs::StdRng::seed_from_u64(pos_seed);

                                // Random number of shrubs per tree (1-4)
                                let shrub_count = rng.gen_range(1..=4);

                                for _ in 0..shrub_count {
                                    // Random angle (full circle)
                                    let ang = rng.gen_range(0.0..std::f32::consts::TAU);
                                    // Random distance from tree (10.0 to 100.0 units - MASSIVE spread, 10x scatter)
                                    let dist = rng.gen_range(10.0..100.0);
                                    // Random scale variation (0.1 to 6.0 - huge variety from tiny to large)
                                    let scale = rng.gen_range(0.1..6.0);
                                    // Random rotation
                                    let rot_y = rng.gen_range(0.0..std::f32::consts::TAU);
                                    // Random height offset (-2.0 to 2.0 for terrain variation)
                                    let height_offset = rng.gen_range(-2.0..2.0);
                                    // Random model selection
                                    let model_idx = rng.gen_range(0..all_shrub_models.len());
                                    let name = all_shrub_models[model_idx].to_string();

                                    let shrub_t = Mat4::from_scale_rotation_translation(
                                        Vec3::splat(scale),
                                        glam::Quat::from_rotation_y(rot_y),
                                        Vec3::new(pos.x + ang.cos() * dist, pos.y + height_offset, pos.z + ang.sin() * dist),
                                    );
                                    shrub_groups.entry(name).or_default().push(shrub_t);
                                }
                            }
                            for (name, transforms) in &shrub_groups {
                                // Render bark mesh
                                let bark_name = format!("{}_bark", name);
                                if let Some(mesh) = state.mesh_registry.get(&bark_name) {
                                    let mut sp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());
                                    sp.set_mesh(mesh.clone());
                                    sp.upload_instances(ctx.device(), transforms);
                                    foliage_pipelines.push(sp);
                                    println!("[FOLIAGE] Shrub '{}' bark: {} instances", name, transforms.len());
                                }
                                // Render leaves mesh
                                let leaves_name = format!("{}_leaves", name);
                                if let Some(mesh) = state.mesh_registry.get(&leaves_name) {
                                    let mut sp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());
                                    sp.set_mesh(mesh.clone());
                                    sp.upload_instances(ctx.device(), transforms);
                                    foliage_pipelines.push(sp);
                                    println!("[FOLIAGE] Shrub '{}' leaves: {} instances", name, transforms.len());
                                }
                            }
                            println!("[CHUNK] Foliage: {} pipelines", foliage_pipelines.len());

                            let mut detritus_pipeline = None;
                            if !det_pos.is_empty() {
                                let mut dp = DetritusPipeline::new(ctx.device(), ctx.surface_format());
                                dp.upload_mesh(ctx.device(), ctx.queue(), &det_pos, &det_nrm, &det_uv, &det_idx);
                                detritus_pipeline = Some(dp);
                            }

                            // Group rocks by type
                            let mut rock_groups: std::collections::HashMap<String, Vec<Mat4>> = std::collections::HashMap::new();
                            for (name, transform) in rock_instances {
                                rock_groups.entry(name).or_default().push(transform);
                            }

                            // Debug: Show rock type breakdown
                            println!("[CHUNK] Rock types: {:?}", rock_groups.keys().collect::<Vec<_>>());
                            for (name, transforms) in &rock_groups {
                                println!("[CHUNK]   {}: {} instances", name, transforms.len());
                            }

                            let mut rock_pipelines = Vec::new();
                            for (name, transforms) in rock_groups {
                                if let Some(mesh) = state.mesh_registry.get(&name) {
                                    let mut rp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());
                                    rp.set_mesh(mesh.clone());
                                    rp.upload_instances(ctx.device(), &transforms);
                                    rock_pipelines.push(rp);
                                    println!("[CHUNK] Created rock pipeline '{}' with {} instances", name, transforms.len());
                                } else {
                                    println!("[WARN] Unknown rock type '{}' requested - mesh not in registry!", name);
                                    println!("[WARN] Available meshes: {:?}", state.mesh_registry.keys().collect::<Vec<_>>());
                                }
                            }

                            // Process Buildings
                            let mut building_pipelines = Vec::new();
                            let mut buildings_by_type: std::collections::HashMap<String, Vec<Mat4>> = std::collections::HashMap::new();
                            for (name, transform) in building_instances {
                                buildings_by_type.entry(name).or_default().push(transform);
                            }

                            for (name, transforms) in buildings_by_type {
                                if let Some(mesh) = state.building_registry.get(&name) {
                                    let mut pipeline = BuildingPipeline::new(ctx.device(), ctx.surface_format());
                                    pipeline.set_mesh(mesh.clone());
                                    pipeline.upload_instances(ctx.device(), &transforms);
                                    building_pipelines.push(pipeline);
                                } else {
                                    println!("[WARN] Building mesh '{}' not found in registry", name);
                                }
                            }

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
                                building_pipelines.push(pipeline);
                            }

                            // Add to Manager
                            let loaded_chunk = LoadedChunk {
                                terrain: terrain_pipeline,
                                grass: grass_pipeline,
                                trees: foliage_pipelines,
                                detritus: detritus_pipeline,
                                rocks: rock_pipelines,
                                buildings: building_pipelines,
                                bounds,
                            };
                            
                            let coord = ChunkCoord::from_world_pos(Vec3::new(offset_x as f32, 0.0, offset_z as f32), chunk_size);
                            manager.add_chunk(coord, loaded_chunk);

                            // Spawn animals for this chunk
                            // Note: We need to destructure to avoid borrow checker issues
                            let player_pos = state.player.position;
                            let seed = state.seed;
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
                            // For streaming, "complete" just means "initial batch done"
                            if state.game_state == GameState::Loading {
                                let (loaded, loading) = manager.get_stats();
                                // If we have loaded enough and no more pending, switch to playing
                                if loading == 0 && loaded > 0 {
                                    println!("[LOAD] Initial chunks loaded! Transitioning to Playing...");
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
                for (_coord, chunk) in manager.iter_chunks() {
                    if let Some(grass) = &chunk.grass {
                        grass.update_camera(
                            ctx.queue(),
                            &view_proj,
                            &light_view_proj,
                            light_dir.to_array(),
                            elapsed,
                            state.camera.position.to_array(),
                            fog_color,
                            fog_start,
                            fog_end,
                            fog_density,
                        );
                    }
                    for trees in &chunk.trees {
                        // Textured foliage from GLTF - enable texture sampling + alpha discard
                        trees.update_camera_full(
                            ctx.queue(),
                            &view_proj,
                            sun_dir.to_array(),
                            elapsed,
                            state.camera.position.to_array(),
                            fog_color,
                            fog_start,
                            fog_end,
                            fog_density,
                            0.5,   // alpha_cutoff - use 0.5 for clean leaf edges
                            1.0,   // use_texture = sample from texture
                        );
                    }
                    if let Some(detritus) = &chunk.detritus {
                        detritus.update_camera(
                            ctx.queue(),
                            &view_proj,
                            sun_dir.to_array(),
                            state.camera.position.to_array(),
                            fog_color,
                            fog_start,
                            fog_end,
                            fog_density,
                        );
                    }
                    for rock in &chunk.rocks {
                        rock.update_camera(
                            ctx.queue(),
                            &view_proj,
                            sun_dir.to_array(),
                            elapsed,
                            state.camera.position.to_array(),
                            fog_color,
                            fog_start,
                            fog_end,
                            fog_density,
                        );
                    }
                    // for building in &chunk.buildings {
                    //     building.update_camera(ctx.queue(), &view_proj);
                    // }
                }
            }

            // Update Water & Dispatch Compute
            // {
            //     let mut water = water_system_mutex.safe_lock();
            //     water.update(ctx.queue(), elapsed, delta);
            //     water.update_camera(ctx.queue(), view_proj.to_cols_array_2d(), state.camera.position.to_array());
            //     water.dispatch(&mut encoder);
            // }

            // 0. Shadow Pass
            {
                let shadow_map = shadow_map_mutex.safe_lock();
                let shadow_pipeline = shadow_pipeline_mutex.safe_lock();
                shadow_pipeline.update_uniforms(ctx.queue(), &light_view_proj);

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

                for (_coord, chunk) in manager.iter_chunks() {
                    shadow_pipeline.render(
                        &mut shadow_pass,
                        &chunk.terrain.vertex_buffer,
                        &chunk.terrain.index_buffer,
                        chunk.terrain.index_count,
                    );
                    // for building in &chunk.buildings {
                    //     building.render_shadow(&mut shadow_pass, &shadow_pipeline);
                    // }
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
                    sun_pipeline.update(ctx.queue(), &view_proj, sun_dir, state.camera.position, state.camera.right(), state.camera.up, state.time_of_day);
                    sun_pipeline.render(&mut sun_pass);
                }

                // Render Moon
                if sun_pos_y < 0.2 { // Visible when sun is low or set
                    // Hack: Pass a fixed "midday" time (12.0) to get white color from sun logic, 
                    // or we could modify sun pipeline to take explicit color.
                    // For now, let's rely on the fact that 12.0 gives white.
                    moon_pipeline.update(ctx.queue(), &view_proj, moon_dir, state.camera.position, state.camera.right(), state.camera.up, 12.0);
                    moon_pipeline.render(&mut sun_pass);
                }
            }

            // 2. Main Render Pass
            {
                // let water_system_guard = water_system_mutex.safe_lock();
                let orb_pipeline = animal_orb_pipeline_mutex.safe_lock();
                // Lock model_pipeline early so it outlives render_pass
                let model_pipeline = animal_model_pipeline_mutex.safe_lock();
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
                let mut grass_rendered = 0;
                let mut trees_rendered = 0;
                let mut buildings_rendered = 0;

                // Use render distance setting from pause menu
                // Distance is to chunk CENTER (not edge), so with 256-unit chunks,
                // player can be up to 181 units from center (corner to center diagonal)
                let grass_max_distance = 0.0;  // DISABLED - using GLTF foliage instead
                let tree_max_distance = (state.render_distance * 1.5).max(300.0);   // Foliage visible far
                let detritus_max_distance = 0.0; // DISABLED - detritus is FPS killer
                let building_max_distance = state.render_distance * 1.0; // Buildings visible at render dist

                for (_coord, chunk) in manager.iter_chunks() {
                    // Frustum cull - skip chunks outside view
                    if !frustum.contains_sphere(chunk.bounds.center, chunk.bounds.radius) {
                        terrain_culled += 1;
                        continue;
                    }
                    terrain_rendered += 1;

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
                        state.camera.position.to_array()
                    );
                    chunk.terrain.render(&mut render_pass);

                    let dist = (chunk.bounds.center - state.camera.position).length();

                    // Grass
                    if let Some(grass) = &chunk.grass {
                        if dist <= grass_max_distance {
                            grass_rendered += 1;
                            grass.render(&mut render_pass);
                        }
                    }

                    // Trees - RE-ENABLED with simple low-poly mesh (~36 tris per tree)
                    // Previously: 94K tris per instance (247K face OBJ)
                    // Now: ~36 tris per instance (cylinder trunk + icosphere canopy)
                    for trees in &chunk.trees {
                        if dist <= tree_max_distance {
                            trees_rendered += 1;
                            trees.render(&mut render_pass);
                        }
                    }

                    // Detritus
                    if let Some(detritus) = &chunk.detritus {
                        if dist <= detritus_max_distance {
                            detritus.render(&mut render_pass);
                        }
                    }

                    // Rocks
                    for rock in &chunk.rocks {
                        if dist <= tree_max_distance {
                            rock.render(&mut render_pass);
                        }
                    }

                    // Buildings
                    for building in &chunk.buildings {
                        if dist <= building_max_distance {
                            buildings_rendered += 1;
                            building.update_uniforms(
                                ctx.queue(),
                                &view_proj,
                                sun_dir,
                                state.camera.position,
                                fog_color,
                                fog_start,
                                fog_end,
                            );
                            building.render(&mut render_pass);
                        }
                    }
                }

                // Render Animal Models (3D models for species that have them)
                model_pipeline.update_camera(
                    ctx.queue(),
                    &view_proj,
                    state.camera.position,
                    state.time_of_day,
                    Vec3::from_array(fog_color),
                    fog_start,
                    fog_end,
                    1.5, // fog_density
                );
                model_pipeline.render(&mut render_pass);

                // Render Animal Orbs (fallback for species without 3D models)
                orb_pipeline.update_camera(ctx.queue(), &view_proj, state.camera.position);
                orb_pipeline.render(&mut render_pass);

                // Render Water
                // water_system_guard.draw(&mut render_pass);

                // Log culling stats occasionally (every ~60 frames)
                let _ = (terrain_rendered, terrain_culled, grass_rendered, trees_rendered, buildings_rendered);
            } // End Main Pass

            // 2.5 Light Shaft Post-Process Pass
            if offscreen_view.is_some() {
                let light_shaft_pipeline = light_shaft_pipeline_mutex.safe_lock();
                let atmo = &state.atmosphere.state;

                // Calculate sun screen position
                if let Some(sun_screen_pos) = LightShaftPipeline::calculate_sun_screen_pos(sun_dir, view_proj) {
                    // Only render light shafts during daytime with sufficient intensity
                    if atmo.light_shaft_intensity > 0.01 && sun_pos_y > 0.0 {
                        light_shaft_pipeline.update_uniforms(
                            ctx.queue(),
                            sun_screen_pos,
                            atmo.light_shaft_intensity,
                            atmo.light_shaft_decay,
                            atmo.light_shaft_density,
                        );

                        // Create bind group with offscreen texture
                        let bind_group = light_shaft_pipeline.create_bind_group(
                            ctx.device(),
                            offscreen_view.unwrap(),
                        );

                        let mut light_shaft_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

                        light_shaft_pipeline.render(&mut light_shaft_pass, &bind_group);
                    } else {
                        // No light shafts - just copy offscreen to view
                        let bind_group = light_shaft_pipeline.create_bind_group(
                            ctx.device(),
                            offscreen_view.unwrap(),
                        );
                        light_shaft_pipeline.update_uniforms(ctx.queue(), [0.5, 0.5], 0.0, 0.96, 0.5);

                        let mut copy_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Copy Pass"),
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

                        light_shaft_pipeline.render(&mut copy_pass, &bind_group);
                    }
                } else {
                    // Sun off-screen - just copy offscreen to view
                    let bind_group = light_shaft_pipeline.create_bind_group(
                        ctx.device(),
                        offscreen_view.unwrap(),
                    );
                    light_shaft_pipeline.update_uniforms(ctx.queue(), [0.5, 0.5], 0.0, 0.96, 0.5);

                    let mut copy_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Copy Pass"),
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

                    light_shaft_pipeline.render(&mut copy_pass, &bind_group);
                }
            }

            // 3. Viewmodel Pass (First-person arms and weapon) - only in playing mode
            if state.game_state == GameState::Playing {
                let viewmodel_pipeline = viewmodel_pipeline_mutex.safe_lock();

                // Update viewmodel uniforms with camera rotation and swing animation
                viewmodel_pipeline.update_uniforms(
                    ctx.queue(),
                    state.camera.yaw,
                    state.camera.pitch,
                    state.camera.aspect_ratio,
                    state.swing_animation.swing_progress,
                );

                // Render viewmodel on top of world but below UI
                let mut viewmodel_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Viewmodel Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load, // Load existing pixels (world scene)
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: ctx.depth_view(),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0), // Clear to far plane (1.0) so viewmodel is always closest
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                viewmodel_pipeline.render(&mut viewmodel_pass);
            } // End Viewmodel Pass

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
