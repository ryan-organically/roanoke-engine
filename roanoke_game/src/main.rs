use croatoan_core::{App, CursorGrabMode, DeviceEvent, ElementState, KeyCode, PhysicalKey, WinitEvent as Event, WinitWindowEvent as WindowEvent};
use croatoan_wfc::{generate_terrain_chunk, generate_vegetation_for_chunk, generate_trees_for_chunk, generate_detritus_for_chunk, generate_rocks_for_chunk, generate_buildings_for_chunk, TreeTemplate};
use croatoan_render::{Camera, TerrainPipeline, ShadowMap, ShadowPipeline, GrassPipeline, TreePipeline, TreeMesh, DetritusPipeline, BuildingPipeline, BuildingMesh, BuildingVertex, Frustum, ChunkBounds, SunPipeline, SkyPipeline};
use croatoan_procgen::{TreeRecipe, generate_tree, generate_tree_mesh, RockRecipe, generate_rock, BuildingRecipe, generate_building};
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
mod chunk_manager;
mod asset_loader;
use player::Player;
use chunk_manager::{ChunkManager, ChunkCoord, ChunkRequest, LoadedChunk};

// Extend LoadedChunk to include buildings (we can't modify the struct definition in chunk_manager.rs from here easily without replacing the file, 
// but wait, LoadedChunk is defined in chunk_manager.rs. I need to modify chunk_manager.rs FIRST or define a wrapper.
// Actually, I should modify chunk_manager.rs to add buildings field.
// But for now, I will modify main.rs to import the struct and I will modify chunk_manager.rs in a separate step.
// Wait, I can't modify main.rs to use a field that doesn't exist yet.
// I will assume I will modify chunk_manager.rs in the next step.


mod water_system;

use water_system::WaterSystem;
mod weather_system;
use weather_system::{WeatherSystem, WeatherType};

// ... (Existing structs remain same) ...



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
    // Asset Registry
    mesh_registry: std::collections::HashMap<String, TreeMesh>, // For Trees/Rocks
    building_registry: std::collections::HashMap<String, Arc<BuildingMesh>>, // For Buildings
    background_texture: Option<egui::TextureHandle>, // For Home Screen
    loading_texture: Option<egui::TextureHandle>, // For Loading Screen
    weather: WeatherSystem,
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
        background_texture: None,
        loading_texture: None,
        weather: WeatherSystem::new(),
    }));

    // ... (Channel setup) ...
    // Response Data: (Terrain, Grass, Trees, Detritus, Rocks, Coord X, Coord Z)
    type ChunkData = (
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>, // Terrain
        Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>, // Grass
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
            let chunk_world_size = 256.0;
            let chunk_resolution = 64;
            let scale = 4.0;
            let (offset_x, offset_z) = req.coord.world_offset(chunk_world_size);
            let offset_x = offset_x as i32;
            let offset_z = offset_z as i32;

            // Generate terrain
            let (terrain_pos, terrain_col, terrain_nrm, terrain_idx) =
                generate_terrain_chunk(req.seed, chunk_resolution, offset_x, offset_z, scale);

            // Generate grass
            let (grass_pos, grass_col, grass_idx) = generate_vegetation_for_chunk(
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
            if chunk_tx.send((
                terrain_pos, terrain_col, terrain_nrm, terrain_idx,
                grass_pos, grass_col, grass_idx,
                tree_instances,
                det_pos, det_nrm, det_uv, det_idx,
                rock_instances,
                building_instances,
                offset_x, offset_z
            )).is_err() {
                println!("[GEN] Receiver dropped, stopping thread.");
                break;
            }
        }
    });

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
                                KeyCode::KeyU => {
                                    state.weather.set_weather(WeatherType::Clear, false);
                                    println!("[WEATHER] Set to Clear");
                                }
                                KeyCode::KeyI => {
                                    state.weather.set_weather(WeatherType::PartlyCloudy, false);
                                    println!("[WEATHER] Set to PartlyCloudy");
                                }
                                KeyCode::KeyO => {
                                    state.weather.set_weather(WeatherType::Stormy, false);
                                    println!("[WEATHER] Set to Stormy");
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
        // Initialize Asset Registry if empty
        {
            let mut state = render_state.lock().unwrap();
            if state.mesh_registry.is_empty() {
                println!("[GPU] Initializing Mesh Registry...");

                // 1. Oak Tree (Loaded from OBJ)
                {
                    println!("[ASSET] Loading tree model...");
                    // Try multiple paths for robustness
                    let obj_paths = ["assets/trees/trees9.obj", "trees/trees9.obj"];
                    let mut template = None;
                    for path in obj_paths {
                        if let Some(t) = asset_loader::load_obj(path) {
                            template = Some(t);
                            break;
                        }
                    }

                    if let Some(template) = template {
                        // Load Texture
                        let texture_paths = ["assets/trees/Texture/Bark___0.jpg", "trees/Texture/Bark___0.jpg"];
                        let mut texture_bytes = Vec::new();
                        let mut loaded = false;
                        
                        for path in texture_paths {
                            if let Ok(bytes) = std::fs::read(path) {
                                texture_bytes = bytes;
                                loaded = true;
                                println!("[ASSET] Loaded tree texture from {}", path);
                                break;
                            }
                        }
                        
                        if !loaded {
                            println!("[WARN] Failed to load tree texture from any path, using fallback pink");
                            texture_bytes = vec![255, 0, 255, 255];
                        }

                        let texture_image = image::load_from_memory(&texture_bytes).unwrap_or_else(|_| {
                             image::DynamicImage::new_rgba8(1, 1)
                        });
                        let rgba = texture_image.to_rgba8();
                        let dimensions = rgba.dimensions();

                        let texture_size = wgpu::Extent3d {
                            width: dimensions.0,
                            height: dimensions.1,
                            depth_or_array_layers: 1,
                        };

                        let texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
                            size: texture_size,
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                            label: Some("Tree Diffuse Texture"),
                            view_formats: &[],
                        });

                        ctx.queue().write_texture(
                            wgpu::ImageCopyTexture {
                                texture: &texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            &rgba,
                            wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(4 * dimensions.0),
                                rows_per_image: Some(dimensions.1),
                            },
                            texture_size,
                        );

                        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let sampler = ctx.device().create_sampler(&wgpu::SamplerDescriptor {
                            address_mode_u: wgpu::AddressMode::Repeat,
                            address_mode_v: wgpu::AddressMode::Repeat,
                            mag_filter: wgpu::FilterMode::Linear,
                            min_filter: wgpu::FilterMode::Linear,
                            mipmap_filter: wgpu::FilterMode::Nearest,
                            ..Default::default()
                        });

                        // We need to create a dummy pipeline to get the layout... 
                        // Or better, expose a static function or create the layout here.
                        // TreePipeline::new creates the layout internally.
                        // We can just create a temporary pipeline to grab the layout or duplicate the layout creation.
                        // Since we need the bind group to CREATE the mesh, we have a chicken-and-egg if the layout is inside pipeline.
                        // Solution: Instantiate a dummy pipeline first to get the layout? No, expensive.
                        // Better: Create the BindGroup here using a locally created layout that MATCHES the pipeline's layout.
                        
                        let texture_bind_group_layout = ctx.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some("Tree Texture Bind Group Layout"),
                            entries: &[
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                    ty: wgpu::BindingType::Texture {
                                        multisampled: false,
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                    },
                                    count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: 1,
                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                    count: None,
                                },
                            ],
                        });

                        let bind_group = ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&texture_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&sampler),
                                },
                            ],
                            label: Some("Tree Texture Bind Group"),
                        });

                        let gpu_mesh = TreePipeline::create_mesh(
                            ctx.device(),
                            &template.positions,
                            &template.normals,
                            &template.uvs,
                            &template.indices,
                            Some(Arc::new(bind_group)),
                        );
                        state.mesh_registry.insert("tree_oak".to_string(), gpu_mesh);
                    } else {
                        println!("[WARN] Failed to load OBJ, falling back to procedural");
                        let recipe = TreeRecipe::oak();
                        let tree = generate_tree(&recipe, 12345);
                        let mesh = generate_tree_mesh(&tree);
                        // ... fallback code ...
                    }
                }

                // 2. Rock (Boulder)
                {
                    let recipe = RockRecipe::boulder();
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
                    state.mesh_registry.insert("rock_boulder".to_string(), gpu_mesh);
                }

                println!("[GPU] Assets registered: {:?}", state.mesh_registry.keys());
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
        static CHUNK_MANAGER: OnceLock<Mutex<ChunkManager>> = OnceLock::new();
        let chunk_manager = CHUNK_MANAGER.get_or_init(|| {
            // Load radius 2 = 5x5 grid (visible ~500 units), Unload radius 4 = buffer zone
            // Reduced from 4 (9x9) for performance
            Mutex::new(ChunkManager::new(256.0, 2, 4))
        });

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

        // Water System
        static WATER_SYSTEM: OnceLock<Mutex<WaterSystem>> = OnceLock::new();
        // let water_system_mutex = WATER_SYSTEM.get_or_init(|| {
        //     Mutex::new(WaterSystem::new(ctx.device(), ctx.surface_format()))
        // });

        let mut state = render_state.lock().unwrap();

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
            ui_ctx.set_style(style);

            // Sync Cursor State with Game State
            match state.game_state {
                GameState::Menu | GameState::Loading => {
                    ctx.window.set_cursor_visible(true);
                    let _ = ctx.window.set_cursor_grab(CursorGrabMode::None);
                }
                GameState::Playing => {
                    ctx.window.set_cursor_visible(true);
                    let _ = ctx.window.set_cursor_grab(CursorGrabMode::None);
                }
            }

            match state.game_state {
                GameState::Loading => {
                    egui::CentralPanel::default().show(ui_ctx, |ui| {
                        // Load loading texture if not loaded
                        if state.loading_texture.is_none() {
                            let path = "assets/ui/loading/loading.png";
                            if let Ok(bytes) = std::fs::read(path) {
                                if let Ok(image) = image::load_from_memory(&bytes) {
                                    let size = [image.width() as usize, image.height() as usize];
                                    let image_buffer = image.to_rgba8();
                                    let pixels = image_buffer.as_flat_samples();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        pixels.as_slice(),
                                    );
                                    state.loading_texture = Some(ui.ctx().load_texture(
                                        "loading_background",
                                        color_image,
                                        egui::TextureOptions::LINEAR,
                                    ));
                                    println!("[UI] Loaded loading image from {}", path);
                                }
                            }
                        }

                        // Draw Loading Background
                        if let Some(texture) = &state.loading_texture {
                            let screen_rect = ui.ctx().screen_rect();
                            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                            ui.painter().image(
                                texture.id(),
                                screen_rect,
                                uv,
                                egui::Color32::WHITE,
                            );
                        }

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
                            } else {
                                // println!("[UI] Background image not found at {}", path);
                            }
                        }

                        // Draw Background
                        if let Some(texture) = &state.background_texture {
                            // Draw image covering the whole screen
                            let screen_rect = ui.ctx().screen_rect();
                            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                            ui.painter().image(
                                texture.id(),
                                screen_rect,
                                uv,
                                egui::Color32::WHITE,
                            );
                        }

                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);
                            ui.heading(egui::RichText::new("Roanoke Engine").size(40.0).color(egui::Color32::BLACK));
                            ui.add_space(50.0);

                            ui.label(egui::RichText::new("Enter Seed:").color(egui::Color32::BLACK));
                            ui.text_edit_singleline(&mut state.seed_input);
                            
                            if ui.button(egui::RichText::new("New Game").size(20.0)).clicked() {
                                // TODO: Play Menu Select Sound
                                // audio.play("ui_select.wav");
                                
                                if let Ok(seed) = state.seed_input.parse::<u32>() {
                                    state.seed = seed;
                                    state.game_state = GameState::Loading;
                                    state.save_name_input = format!("seed_{}", seed); // Default save name
                                    state.player = Player::new(Vec3::new(0.0, 50.0, 0.0)); // Reset player position
                                    println!("[GAME] Starting new game with seed: {}", seed);

                                    // Initialize loading progress
                                    // Range 3 = 7x7 = 49 chunks
                                    let range = 3;
                                    let total = ((range * 2 + 1) * (range * 2 + 1)) as usize;
                                    state.loading_progress = LoadingProgress {
                                        total_chunks: total,
                                        chunks_generated: 0,
                                        chunks_uploaded: 0,
                                        current_status: "Initializing world generation...".to_string(),
                                    };

                                    // Force regeneration by clearing chunks
                                    if let Some(manager) = CHUNK_MANAGER.get() {
                                        let mut mgr = manager.lock().unwrap();
                                        mgr.loaded_chunks.clear();
                                        mgr.loading_chunks.clear();
                                    }
                                    
                                    // We don't spawn a thread here anymore. 
                                    // The render loop will detect we are in Loading state and the ChunkManager will request chunks.
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
                                            // TODO: Play Menu Select Sound
                                            // audio.play("ui_select.wav");

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
                                                let range = 3;
                                                let total = ((range * 2 + 1) * (range * 2 + 1)) as usize;
                                                state.loading_progress = LoadingProgress {
                                                    total_chunks: total,
                                                    chunks_generated: 0,
                                                    chunks_uploaded: 0,
                                                    current_status: "Loading saved world...".to_string(),
                                                };

                                                // Force regeneration by clearing chunks
                                                if let Some(manager) = CHUNK_MANAGER.get() {
                                                    let mut mgr = manager.lock().unwrap();
                                                    mgr.loaded_chunks.clear();
                                                    mgr.loading_chunks.clear();
                                                }
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
                        let hours = state.time_of_day as u32;
                        let minutes = ((state.time_of_day - hours as f32) * 60.0) as u32;
                        ui.label(format!("Time: {:02}:{:02}", hours, minutes));
                        ui.label("T/Y keys: Change time");
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
            let mut manager = chunk_manager.lock().unwrap();

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
                            grass_pos, grass_col, grass_idx,
                            tree_instances,
                            det_pos, det_nrm, det_uv, det_idx,
                            rock_instances,
                            building_instances,
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
                                let shadow_map = shadow_map_mutex.lock().unwrap();
                                TerrainPipeline::new(
                                    ctx.device(),
                                    ctx.surface_format(),
                                    &terrain_pos, &terrain_col, &terrain_nrm, &terrain_idx,
                                    &shadow_map
                                )
                            };

                            let mut grass_pipeline = None;
                            if !grass_pos.is_empty() {
                                let shadow_map = shadow_map_mutex.lock().unwrap();
                                let mut gp = GrassPipeline::new(ctx.device(), ctx.surface_format(), &shadow_map);
                                drop(shadow_map);
                                gp.upload_mesh(ctx.device(), ctx.queue(), &grass_pos, &grass_col, &grass_idx);
                                grass_pipeline = Some(gp);
                            }

                            let mut tree_pipeline = None;
                            if !tree_instances.is_empty() {
                                if let Some(mesh) = state.mesh_registry.get("tree_oak") {
                                    let mut tp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());
                                    tp.set_mesh(mesh.clone());
                                    tp.upload_instances(ctx.device(), &tree_instances);
                                    tree_pipeline = Some(tp);
                                }
                            }

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

                            let mut rock_pipelines = Vec::new();
                            for (name, transforms) in rock_groups {
                                if let Some(mesh) = state.mesh_registry.get(&name) {
                                    let mut rp = TreePipeline::new(ctx.device(), ctx.queue(), ctx.surface_format());
                                    rp.set_mesh(mesh.clone());
                                    rp.upload_instances(ctx.device(), &transforms);
                                    rock_pipelines.push(rp);
                                } else {
                                    println!("[WARN] Unknown rock type '{}' requested by generator", name);
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

                            // Add to Manager
                            let loaded_chunk = LoadedChunk {
                                terrain: terrain_pipeline,
                                grass: grass_pipeline,
                                trees: tree_pipeline,
                                detritus: detritus_pipeline,
                                rocks: rock_pipelines,
                                buildings: building_pipelines,
                                bounds,
                            };
                            
                            let coord = ChunkCoord::from_world_pos(Vec3::new(offset_x as f32, 0.0, offset_z as f32), chunk_size);
                            manager.add_chunk(coord, loaded_chunk);

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
        let manager = chunk_manager.lock().unwrap();
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
            let shadow_map_size = 2048.0_f32;
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

            {
                for (_coord, chunk) in manager.iter_chunks() {
                    if let Some(grass) = &chunk.grass {
                        grass.update_camera(ctx.queue(), &view_proj, &light_view_proj, light_dir.to_array(), elapsed);
                    }
                    if let Some(trees) = &chunk.trees {
                        trees.update_camera(ctx.queue(), &view_proj);
                    }
                    if let Some(detritus) = &chunk.detritus {
                        detritus.update_camera(ctx.queue(), &view_proj);
                    }
                    for rock in &chunk.rocks {
                        rock.update_camera(ctx.queue(), &view_proj);
                    }
                    // for building in &chunk.buildings {
                    //     building.update_camera(ctx.queue(), &view_proj);
                    // }
                }
            }

            // Update Water & Dispatch Compute
            // {
            //     let mut water = water_system_mutex.lock().unwrap();
            //     water.update(ctx.queue(), elapsed, delta);
            //     water.update_camera(ctx.queue(), view_proj.to_cols_array_2d(), state.camera.position.to_array());
            //     water.dispatch(&mut encoder);
            // }

            // 0. Shadow Pass
            {
                let shadow_map = shadow_map_mutex.lock().unwrap();
                let shadow_pipeline = shadow_pipeline_mutex.lock().unwrap();
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

            // 0.5 Sky Pass (Draw Skybox/Clouds first)
            {
                let sky_pipeline = sky_pipeline_mutex.lock().unwrap();
                sky_pipeline.update_uniforms(
                    ctx.queue(),
                    view_proj,
                    sun_dir,
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
                        view: &view,
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
                let sun_pipeline = sun_pipeline_mutex.lock().unwrap();
                let moon_pipeline = moon_pipeline_mutex.lock().unwrap();

                let mut sun_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sun/Moon Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
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
                // let water_system_guard = water_system_mutex.lock().unwrap();
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
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

                // Dynamic fog color matching sky
                let fog_color = [
                    sky_color.r as f32 * 0.9,
                    sky_color.g as f32 * 0.9,
                    sky_color.b as f32 * 0.9,
                ];
                let fog_start = 200.0;
                let fog_end = 600.0;

                // Render chunks with frustum culling and LOD
                let mut terrain_rendered = 0;
                let mut terrain_culled = 0;
                let mut grass_rendered = 0;
                let mut trees_rendered = 0;
                let mut buildings_rendered = 0;

                let grass_max_distance = 350.0;
                let tree_max_distance = 600.0;
                let detritus_max_distance = 500.0;
                let building_max_distance = 1000.0; // Buildings visible further

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

                    // Trees
                    if let Some(trees) = &chunk.trees {
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

                    // Rocks (Same LOD as trees for now)
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

                // Render Water
                // water_system_guard.draw(&mut render_pass);

                // Log culling stats occasionally (every ~60 frames)
                let _ = (terrain_rendered, terrain_culled, grass_rendered, trees_rendered, buildings_rendered);
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
