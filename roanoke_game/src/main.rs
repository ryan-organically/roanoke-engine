use croatoan_core::{App, CursorGrabMode, DeviceEvent, ElementState, KeyCode, PhysicalKey, WinitEvent as Event, WinitWindowEvent as WindowEvent};
use croatoan_wfc::{generate_terrain_chunk, generate_vegetation_for_chunk, generate_trees_for_chunk};
use croatoan_render::{Camera, TerrainPipeline, ShadowMap, ShadowPipeline, GrassPipeline, TreePipeline};
use glam::{Vec3, Mat4};
use wgpu;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};
use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod player;
use player::Player;

// --- Game State & Save System ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameState {
    Menu,
    Loading,
    Playing,
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

// --- Main Entry Point ---

fn main() {
    println!("=== ROANOKE ENGINE: HOME SCREEN & SAVE SYSTEM ===\n");

    // Initialize App
    let mut app = App::new("Roanoke Engine", 1280, 720);

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
    }));

    // Terrain Generation Channel
    // We send (terrain_pos, terrain_col, terrain_nrm, terrain_idx, grass_pos, grass_col, grass_idx, tree_pos, tree_nrm, tree_uv, tree_idx, offset_x, offset_z)
    type ChunkData = (
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>, // Terrain
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>, // Grass
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>, // Trees
        i32, i32 // Offsets
    );
    let (chunk_tx, chunk_rx): (Sender<ChunkData>, Receiver<ChunkData>) = channel();
    // We need to keep the receiver in a mutex or just move it into the render closure?
    // The render closure is called repeatedly, so we can't move the receiver into it if it's FnMut (which it is).
    // But we can put it in a Mutex/Arc if we need to share it, or just use a Mutex since it's single consumer.
    let chunk_rx = Arc::new(Mutex::new(chunk_rx));

    // Terrain Data (Protected by Mutex to allow regeneration)
    let _terrain_data = Arc::new(Mutex::new(None::<(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>)>));
    
    // Time tracking
    let start_time = Instant::now();

    // --- Input Callback ---
    let input_state = Arc::clone(&shared_state);
    app.set_input_callback(move |event, window| {
        let mut state = input_state.lock().unwrap();

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
                    // Mouse Look
                    state.player.yaw += delta.0 as f32 * 0.002;
                    state.player.pitch -= delta.1 as f32 * 0.002;
                    state.player.pitch = state.player.pitch.clamp(-1.5, 1.5);
                }
                Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } => {
                    if let PhysicalKey::Code(keycode) = key_event.physical_key {
                        state.keys.insert(keycode, key_event.state);
                        
                        // Handle Jump separately for single press logic if needed, but state check is fine for now
                        if keycode == KeyCode::Space && key_event.state == ElementState::Pressed && state.game_state == GameState::Playing {
                             state.player.jump();
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

        // Pipeline Store (Now stores a list of chunks)
        static PIPELINE_STORE: OnceLock<Mutex<Vec<TerrainPipeline>>> = OnceLock::new();
        let pipeline_store = PIPELINE_STORE.get_or_init(|| Mutex::new(Vec::new()));

        // Per-chunk vegetation pipelines
        static GRASS_PIPELINES: OnceLock<Mutex<Vec<GrassPipeline>>> = OnceLock::new();
        let grass_pipelines = GRASS_PIPELINES.get_or_init(|| Mutex::new(Vec::new()));

        static TREE_PIPELINES: OnceLock<Mutex<Vec<TreePipeline>>> = OnceLock::new();
        let tree_pipelines = TREE_PIPELINES.get_or_init(|| Mutex::new(Vec::new()));

        // Shadow System
        static SHADOW_SYSTEM: OnceLock<(Mutex<ShadowMap>, Mutex<ShadowPipeline>)> = OnceLock::new();
        let (shadow_map_mutex, shadow_pipeline_mutex) = SHADOW_SYSTEM.get_or_init(|| {
            let shadow_map = ShadowMap::new(ctx.device(), 2048);
            let shadow_pipeline = ShadowPipeline::new(ctx.device());
            (Mutex::new(shadow_map), Mutex::new(shadow_pipeline))
        });

        // Grass System (requires shadow map)
        static GRASS_PIPELINE: OnceLock<Mutex<GrassPipeline>> = OnceLock::new();
        let _grass_pipeline_mutex = GRASS_PIPELINE.get_or_init(|| {
            let shadow_map = shadow_map_mutex.lock().unwrap();
            let grass_pipeline = GrassPipeline::new(ctx.device(), ctx.surface_format(), &shadow_map);
            drop(shadow_map);  // Release lock
            Mutex::new(grass_pipeline)
        });

        // Tree System
        static TREE_PIPELINE: OnceLock<Mutex<TreePipeline>> = OnceLock::new();
        let tree_pipeline_mutex = TREE_PIPELINE.get_or_init(|| {
            let tree_pipeline = TreePipeline::new(ctx.device(), ctx.surface_format());
            Mutex::new(tree_pipeline)
        });

        let mut state = render_state.lock().unwrap();

        // Calculate FPS
        let now = Instant::now();
        let delta = now.duration_since(state.last_frame_time).as_secs_f32();
        state.last_frame_time = now;
        if delta > 0.0 {
            // Simple smoothing
            state.fps = state.fps * 0.9 + (1.0 / delta) * 0.1;
        }

        // Update Time of Day (FROZEN AT SUNRISE for shadow testing)
        if state.game_state == GameState::Playing {
            state.time_of_day = 6.0; // Frozen at 6 AM (sunrise) for long dramatic shadows
            // state.time_of_day += delta * 0.1; // 1 second = 0.1 hour (fast cycle for testing)
            // if state.time_of_day >= 24.0 {
            //     state.time_of_day -= 24.0;
            // }
        }

        // Handle Input (Player Controller)
        if state.game_state == GameState::Playing {
            let mut input_dir = Vec3::ZERO;
            if state.keys.get(&KeyCode::KeyW) == Some(&ElementState::Pressed) { input_dir.z += 1.0; }
            if state.keys.get(&KeyCode::KeyS) == Some(&ElementState::Pressed) { input_dir.z -= 1.0; }
            if state.keys.get(&KeyCode::KeyA) == Some(&ElementState::Pressed) { input_dir.x -= 1.0; }
            if state.keys.get(&KeyCode::KeyD) == Some(&ElementState::Pressed) { input_dir.x += 1.0; }
            // Jump is handled in input callback to avoid continuous jumping if holding space (optional, but better)

            let seed = state.seed; // Copy seed to avoid borrow error
            state.player.update(delta, input_dir, seed);

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
            ui_ctx.set_style(style);

            // Sync Cursor State with Game State
            match state.game_state {
                GameState::Menu | GameState::Loading => {
                    ctx.window.set_cursor_visible(true);
                    let _ = ctx.window.set_cursor_grab(CursorGrabMode::None);
                }
                GameState::Playing => {
                    // We keep the cursor visible and ungrabbed in Playing mode for now
                    // so that the user can interact with the "Game Menu" (Save/Back).
                    // In a future update, we should implement a proper "Pause" state
                    // that toggles the cursor, and keep it hidden/locked during gameplay.
                    ctx.window.set_cursor_visible(true);
                    let _ = ctx.window.set_cursor_grab(CursorGrabMode::None);
                }
            }

            match state.game_state {
                GameState::Loading => {
                    egui::CentralPanel::default().show(ui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(150.0);
                            ui.heading(egui::RichText::new("Loading World").size(40.0).color(egui::Color32::BLACK));
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
                                .color(egui::Color32::DARK_GRAY));

                            ui.add_space(10.0);

                            // Additional Progress Info
                            ui.label(egui::RichText::new(format!(
                                "Generated: {} | Uploaded: {}",
                                state.loading_progress.chunks_generated,
                                state.loading_progress.chunks_uploaded
                            )).color(egui::Color32::DARK_GRAY));
                        });
                    });
                }
                GameState::Menu => {
                    egui::CentralPanel::default().show(ui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);
                            ui.heading(egui::RichText::new("Roanoke Engine").size(40.0).color(egui::Color32::BLACK));
                            ui.add_space(50.0);

                            ui.label(egui::RichText::new("Enter Seed:").color(egui::Color32::BLACK));
                            ui.text_edit_singleline(&mut state.seed_input);
                            
                            if ui.button(egui::RichText::new("New Game").size(20.0)).clicked() {
                                if let Ok(seed) = state.seed_input.parse::<u32>() {
                                    state.seed = seed;
                                    state.game_state = GameState::Loading;
                                    state.save_name_input = format!("seed_{}", seed); // Default save name
                                    state.player = Player::new(Vec3::new(0.0, 50.0, 0.0)); // Reset player position
                                    println!("[GAME] Starting new game with seed: {}", seed);

                                    // Initialize loading progress
                                    let range = 1;
                                    let total = ((range * 2 + 1) * (range * 2 + 1)) as usize;
                                    state.loading_progress = LoadingProgress {
                                        total_chunks: total,
                                        chunks_generated: 0,
                                        chunks_uploaded: 0,
                                        current_status: "Initializing world generation...".to_string(),
                                    };

                                    // Force regeneration by clearing chunks
                                    pipeline_store.lock().unwrap().clear();
                                    grass_pipelines.lock().unwrap().clear();
                                    tree_pipelines.lock().unwrap().clear();

                                    // Spawn Generation Thread (terrain + vegetation per chunk)
                                    let tx = chunk_tx.clone();
                                    let progress_state = Arc::clone(&render_state);
                                    thread::spawn(move || {
                                        println!("[GEN] Starting background generation for seed {}", seed);
                                        let range = 1;  // 1 = 9 chunks (3x3), 2 = 25 chunks (5x5), 3 = 49 chunks (7x7)
                                        let chunk_world_size = 256.0;
                                        let chunk_resolution = 64;
                                        let scale = 4.0;

                                        for z in -range..=range {
                                            for x in -range..=range {
                                                let offset_x = (x as f32 * chunk_world_size) as i32;
                                                let offset_z = (z as f32 * chunk_world_size) as i32;

                                                // Update progress: Generating terrain
                                                if let Ok(mut state) = progress_state.try_lock() {
                                                    state.loading_progress.current_status =
                                                        format!("Generating terrain chunk ({}, {})...", x, z);
                                                }

                                                // Generate terrain
                                                let (terrain_pos, terrain_col, terrain_nrm, terrain_idx) =
                                                    generate_terrain_chunk(seed, chunk_resolution, offset_x, offset_z, scale);

                                                // Update progress: Generating grass
                                                if let Ok(mut state) = progress_state.try_lock() {
                                                    state.loading_progress.current_status =
                                                        format!("Generating grass for chunk ({}, {})...", x, z);
                                                }

                                                // Generate grass for this chunk
                                                let (grass_pos, grass_col, grass_idx) = generate_vegetation_for_chunk(
                                                    seed,
                                                    chunk_world_size,
                                                    offset_x as f32,
                                                    offset_z as f32,
                                                );

                                                // Update progress: Generating trees
                                                if let Ok(mut state) = progress_state.try_lock() {
                                                    state.loading_progress.current_status =
                                                        format!("Generating trees for chunk ({}, {})...", x, z);
                                                }

                                                // Generate trees for this chunk
                                                let (tree_pos, tree_nrm, tree_uv, tree_idx) = generate_trees_for_chunk(
                                                    seed,
                                                    chunk_world_size,
                                                    offset_x as f32,
                                                    offset_z as f32,
                                                );

                                                // Send all data together
                                                if tx.send((
                                                    terrain_pos, terrain_col, terrain_nrm, terrain_idx,
                                                    grass_pos, grass_col, grass_idx,
                                                    tree_pos, tree_nrm, tree_uv, tree_idx,
                                                    offset_x, offset_z
                                                )).is_err() {
                                                    break; // Receiver dropped
                                                }

                                                // Update chunks generated
                                                if let Ok(mut state) = progress_state.try_lock() {
                                                    state.loading_progress.chunks_generated += 1;
                                                }
                                            }
                                        }

                                        if let Ok(mut state) = progress_state.try_lock() {
                                            state.loading_progress.current_status = "All chunks generated! Uploading to GPU...".to_string();
                                        }
                                        println!("[GEN] Background generation complete.");
                                    });
                                }
                            }

                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("Saved Games:").strong());
                            ui.separator();
                            
                            // List Saves
                            let saves = list_saves();
                            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                for save_name in saves {
                                    ui.horizontal(|ui| {
                                        if ui.button(format!("Load {}", save_name)).clicked() {
                                            if let Some(data) = load_game(&save_name) {
                                                state.seed = data.seed;
                                                state.inventory = data.inventory;
                                                state.player.position = Vec3::from_array(data.player_pos);
                                                state.player.yaw = data.player_rot[0];
                                                state.player.pitch = data.player_rot[1];
                                                state.game_state = GameState::Loading;
                                                state.save_name_input = save_name.clone();

                                                println!("[GAME] Loaded game: {}", save_name);

                                                // Initialize loading progress
                                                let range = 1;
                                                let total = ((range * 2 + 1) * (range * 2 + 1)) as usize;
                                                state.loading_progress = LoadingProgress {
                                                    total_chunks: total,
                                                    chunks_generated: 0,
                                                    chunks_uploaded: 0,
                                                    current_status: "Loading saved world...".to_string(),
                                                };

                                                // Force regeneration by clearing chunks
                                                pipeline_store.lock().unwrap().clear();
                                                grass_pipelines.lock().unwrap().clear();
                                                tree_pipelines.lock().unwrap().clear();

                                                // Spawn Generation Thread (Same as New Game)
                                                let tx = chunk_tx.clone();
                                                let seed = state.seed;
                                                let progress_state = Arc::clone(&render_state);
                                                thread::spawn(move || {
                                                    println!("[GEN] Starting background generation for seed {}", seed);
                                                    let range = 1;  // 1 = 9 chunks (3x3), 2 = 25 chunks (5x5), 3 = 49 chunks (7x7)
                                                    let chunk_world_size = 256.0;
                                                    let chunk_resolution = 64;
                                                    let scale = 4.0;

                                                    for z in -range..=range {
                                                        for x in -range..=range {
                                                            let offset_x = (x as f32 * chunk_world_size) as i32;
                                                            let offset_z = (z as f32 * chunk_world_size) as i32;

                                                            // Update progress
                                                            if let Ok(mut state) = progress_state.try_lock() {
                                                                state.loading_progress.current_status =
                                                                    format!("Generating chunk ({}, {})...", x, z);
                                                            }

                                                            // Generate terrain
                                                            let (terrain_pos, terrain_col, terrain_nrm, terrain_idx) =
                                                                generate_terrain_chunk(seed, chunk_resolution, offset_x, offset_z, scale);

                                                            // Generate grass for this chunk
                                                            let (grass_pos, grass_col, grass_idx) = generate_vegetation_for_chunk(
                                                                seed,
                                                                chunk_world_size,
                                                                offset_x as f32,
                                                                offset_z as f32,
                                                            );

                                                            // Generate trees for this chunk
                                                            let (tree_pos, tree_nrm, tree_uv, tree_idx) = generate_trees_for_chunk(
                                                                seed,
                                                                chunk_world_size,
                                                                offset_x as f32,
                                                                offset_z as f32,
                                                            );

                                                            // Send all data together
                                                            if tx.send((
                                                                terrain_pos, terrain_col, terrain_nrm, terrain_idx,
                                                                grass_pos, grass_col, grass_idx,
                                                                tree_pos, tree_nrm, tree_uv, tree_idx,
                                                                offset_x, offset_z
                                                            )).is_err() {
                                                                break; // Receiver dropped
                                                            }

                                                            // Update chunks generated
                                                            if let Ok(mut state) = progress_state.try_lock() {
                                                                state.loading_progress.chunks_generated += 1;
                                                            }
                                                        }
                                                    }

                                                    if let Ok(mut state) = progress_state.try_lock() {
                                                        state.loading_progress.current_status = "Uploading to GPU...".to_string();
                                                    }
                                                    println!("[GEN] Background generation complete.");
                                                });
                                            }
                                        }
                                    });
                                }
                            });
                        });
                    });
                }
                GameState::Playing => {
                    egui::Window::new("Game Menu").show(ui_ctx, |ui| {
                        ui.label(format!("FPS: {:.1}", state.fps));
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
            }
        });

        // Handle Pipeline Updates (scoped to release locks early)
        {
            let mut pipeline_guard = pipeline_store.lock().unwrap();
            let mut grass_pipelines_guard = grass_pipelines.lock().unwrap();
            let mut tree_pipelines_guard = tree_pipelines.lock().unwrap();

            // Check for new chunks from background thread
        if let Ok(rx) = render_rx.try_lock() {
            // During Loading: Process 1 chunk per frame to keep UI responsive
            // During Playing: Process up to 5 chunks per frame for dynamic loading
            let chunks_per_frame = if state.game_state == GameState::Loading { 1 } else { 5 };
            for _ in 0..chunks_per_frame {
                match rx.try_recv() {
                    Ok((terrain_pos, terrain_col, terrain_nrm, terrain_idx,
                        grass_pos, grass_col, grass_idx,
                        tree_pos, tree_nrm, tree_uv, tree_idx,
                        _ox, _oz)) => {

                        // Update status: Uploading
                        state.loading_progress.current_status = format!(
                            "Uploading chunk {} to GPU...",
                            state.loading_progress.chunks_uploaded + 1
                        );

                        // Add terrain pipeline (acquire shadow_map only for creation)
                        let pipeline = {
                            let shadow_map = shadow_map_mutex.lock().unwrap();
                            TerrainPipeline::new(
                                ctx.device(),
                                ctx.surface_format(),
                                &terrain_pos, &terrain_col, &terrain_nrm, &terrain_idx,
                                &shadow_map
                            )
                        };
                        pipeline_guard.push(pipeline);

                        // Create per-chunk grass pipeline
                        if !grass_pos.is_empty() {
                            let shadow_map = shadow_map_mutex.lock().unwrap();
                            let mut grass_pipeline = GrassPipeline::new(ctx.device(), ctx.surface_format(), &shadow_map);
                            drop(shadow_map);
                            grass_pipeline.upload_mesh(ctx.device(), ctx.queue(), &grass_pos, &grass_col, &grass_idx);
                            grass_pipelines_guard.push(grass_pipeline);
                        }

                        // Create per-chunk tree pipeline
                        if !tree_pos.is_empty() {
                            let mut tree_pipeline = TreePipeline::new(ctx.device(), ctx.surface_format());
                            tree_pipeline.upload_mesh(ctx.device(), ctx.queue(), &tree_pos, &tree_nrm, &tree_uv, &tree_idx);
                            tree_pipelines_guard.push(tree_pipeline);
                        }

                        // Update uploaded count
                        state.loading_progress.chunks_uploaded += 1;

                        // Check if loading is complete
                        if state.loading_progress.chunks_uploaded >= state.loading_progress.total_chunks {
                            if state.game_state == GameState::Loading {
                                println!("[LOAD] Loading complete! Transitioning to Playing...");
                                state.loading_progress.current_status = "Loading complete! Starting game...".to_string();
                                state.game_state = GameState::Playing;
                            }
                        }
                    },
                    Err(_) => break, // Empty
                }
            }
        }
        } // Release pipeline locks before rendering

        // Render frame (re-acquire locks as needed)
        let pipeline_guard = pipeline_store.lock().unwrap();
        if state.game_state == GameState::Playing && !pipeline_guard.is_empty() {
            let elapsed = start_time.elapsed().as_secs_f32();

            // Get the current frame
            let output = ctx.surface.get_current_texture().unwrap();
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Create command encoder
            let mut encoder = ctx.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            // Calculate lighting - Sun rises from EAST (positive X) over the ocean
            // At 6 AM (sunrise): sun low on eastern horizon
            // At 12 PM (noon): sun high overhead
            // At 18 PM (sunset): sun low on western horizon
            let hour_angle = (state.time_of_day - 6.0) * 15.0f32.to_radians(); // Offset so 6 AM = 0°

            // Sun travels from East (+X) to West (-X)
            // X: cos(angle) - starts at 1.0 (east) at sunrise, goes to -1.0 (west) at sunset
            // Y: sin(angle) - starts at 0.0 (horizon) at sunrise, peaks at noon, back to 0.0 at sunset
            // Z: slight southward angle for more interesting shadows
            let sun_dir = Vec3::new(
                -hour_angle.cos(),  // Negative because light direction points TOWARD surface
                -hour_angle.sin(),
                -0.3  // Slight angle from south
            ).normalize();

            // Position light far away in direction of sun
            let light_pos = state.camera.target - sun_dir * 100.0;
            let light_view = Mat4::look_at_rh(light_pos, state.camera.target, Vec3::Y);

            // Larger orthographic projection to cover more terrain
            let light_proj = Mat4::orthographic_rh(-800.0, 800.0, -800.0, 800.0, 1.0, 2000.0);
            let light_view_proj = light_proj * light_view;


            // Update grass and tree cameras before render pass
            let view_proj = state.camera.view_projection_matrix();
            {
                let grass_guard = grass_pipelines.lock().unwrap();
                for grass_pipeline in grass_guard.iter() {
                    grass_pipeline.update_camera(ctx.queue(), &view_proj, &light_view_proj, sun_dir.to_array(), elapsed);
                }

                let tree_guard = tree_pipelines.lock().unwrap();
                for tree_pipeline in tree_guard.iter() {
                    tree_pipeline.update_camera(ctx.queue(), &view_proj);
                }
            }

            // 0. Shadow Pass - Render scene from light's perspective
            {
                let shadow_map = shadow_map_mutex.lock().unwrap();
                let shadow_pipeline = shadow_pipeline_mutex.lock().unwrap();

                // Update shadow uniforms with light view-projection
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

                // Render terrain into shadow map
                for pipeline in pipeline_guard.iter() {
                    shadow_pipeline.render(
                        &mut shadow_pass,
                        &pipeline.vertex_buffer,
                        &pipeline.index_buffer,
                        pipeline.index_count,
                    );
                }

                // TODO: Grass shadow casting requires different vertex stride (24 vs 36)
                // TODO: Add trees to shadow pass
            } // End Shadow Pass

            // 1. Main Render Pass
            {
                let grass_pipelines_guard = grass_pipelines.lock().unwrap();
                let tree_pipelines_guard = tree_pipelines.lock().unwrap();
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.95,  // Bright warm sunrise sky
                                g: 0.75,
                                b: 0.55,
                                a: 1.0,
                            }),
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

                // Render all terrain chunks
                for pipeline in pipeline_guard.iter() {
                    pipeline.update_uniforms(
                        ctx.queue(),
                        &view_proj,
                        &light_view_proj,
                        elapsed,
                        [0.9, 0.7, 0.5], // Fog Color - Warm sunrise/sunset haze
                        400.0,            // Fog Start - further away to see shadows
                        800.0,            // Fog End - much further for dramatic lighting
                        sun_dir.to_array(),
                        state.camera.position.to_array(),
                        state.camera.position.to_array()
                    );
                    pipeline.render(&mut render_pass);
                }

                // Render all grass chunks
                for grass_pipeline in grass_pipelines_guard.iter() {
                    grass_pipeline.render(&mut render_pass);
                }

                // Render all tree chunks
                for tree_pipeline in tree_pipelines_guard.iter() {
                    tree_pipeline.render(&mut render_pass);
                }
            } // End Main Pass

            // 2. Egui Pass
            {
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [ctx.config().width, ctx.config().height],
                    pixels_per_point: ctx.window.scale_factor() as f32,
                };

                let tris = state.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                let mut renderer = egui_renderer_mutex.lock().unwrap();
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

                let mut renderer = egui_renderer_mutex.lock().unwrap();
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
