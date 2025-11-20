use croatoan_core::{App, DeviceEvent, ElementState, KeyCode, PhysicalKey, WinitEvent as Event, WinitWindowEvent as WindowEvent};
use croatoan_wfc::generate_terrain_chunk;
use croatoan_render::{Camera, TerrainPipeline};
use glam::Vec3;
use wgpu;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() {
    println!("=== ROANOKE ENGINE: VERTEX FACTORY (STRICT MODE) ===\n");

    // TASK 4: Generate 64x64 chunk
    let chunk_size = 64;
    let seed = 1587;

    println!("[GENERATOR] Generating terrain chunk...");
    let (positions, colors, indices) = generate_terrain_chunk(seed, chunk_size);

    // STRICT VERIFICATION: Assert vertex count
    let expected_vertices = (chunk_size + 1) * (chunk_size + 1);
    println!("\n[STRICT CHECK] Expected vertices: {}", expected_vertices);
    println!("[STRICT CHECK] Actual vertices: {}", positions.len());

    if positions.len() != expected_vertices as usize {
        panic!(
            "Math Failure: Expected {} vertices, got {}",
            expected_vertices,
            positions.len()
        );
    }

    println!("[STRICT CHECK] ✓ Vertex count validated\n");

    // Create application
    let mut app = App::new("Roanoke Engine - Vertex Factory", 1280, 720);

    // Create shared camera state (SYNCHRONIZED between input and render)
    let shared_camera = Arc::new(Mutex::new(Camera::new(
        Vec3::new(32.0, 50.0, -30.0),
        Vec3::new(32.0, 0.0, 32.0),
        1280.0 / 720.0,
    )));

    println!("[CAMERA] Shared Camera Instance Created");
    println!("[CAMERA] Position: ({}, {}, {})", 32.0, 50.0, -30.0);
    println!("[CAMERA] Target: ({}, {}, {})", 32.0, 0.0, 32.0);

    // Create time tracker
    let start_time = Instant::now();
    println!("[CLOCK] Time system initialized\n");

    // Clone Arc for render callback
    let render_camera = Arc::clone(&shared_camera);

    // Set up rendering with terrain pipeline
    app.set_render_callback(move |graphics_context| {
        // Initialize terrain pipeline (happens once due to move)
        use std::sync::OnceLock;
        static PIPELINE: OnceLock<Mutex<TerrainPipeline>> = OnceLock::new();
        static INIT: OnceLock<()> = OnceLock::new();

        if INIT.get().is_none() {
            println!("\n[PIPELINE] Initializing terrain rendering pipeline...");

            // Create terrain pipeline
            let pipeline = TerrainPipeline::new(
                graphics_context.device(),
                graphics_context.surface_format(),
                &positions,
                &colors,
                &indices,
            );

            println!("[PIPELINE] ✓ Terrain pipeline created\n");
            println!("=== RENDERING ACTIVE ===\n");

            PIPELINE.set(Mutex::new(pipeline)).ok();
            INIT.set(()).ok();
        }

        // Render frame
        if let Some(pipeline_mutex) = PIPELINE.get() {
            let pipeline = pipeline_mutex.lock().unwrap();
            let camera = render_camera.lock().unwrap();

            // Calculate elapsed time
            let elapsed_time = start_time.elapsed().as_secs_f32();

            // Update uniforms with camera and time
            let view_proj = camera.view_projection_matrix();
            pipeline.update_uniforms(graphics_context.queue(), &view_proj, elapsed_time);

                // Get current frame
                let output = match graphics_context.surface.get_current_texture() {
                    Ok(output) => output,
                    Err(_) => return,
                };

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Create command encoder
                let mut encoder =
                    graphics_context.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                // Render pass
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Terrain Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.1,
                                    b: 0.15,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: graphics_context.depth_view(),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    pipeline.render(&mut render_pass);
                }

            // Submit and present
            graphics_context.queue().submit(std::iter::once(encoder.finish()));
            output.present();
        }
    });

    // Clone Arc for input callback
    let input_camera = Arc::clone(&shared_camera);

    // Set up input handling for camera control
    app.set_input_callback(move |event| {
        let mut camera = input_camera.lock().unwrap();

        match event {
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                println!("[DEBUG] Mouse Delta: ({:.2}, {:.2})", delta.0, delta.1);
                camera.process_mouse(delta.0 as f32, delta.1 as f32, 0.002);
            }
            Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } => {
                if key_event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(keycode) = key_event.physical_key {
                        let speed = 5.0; // 10x increase for visibility
                        match keycode {
                            KeyCode::KeyW => {
                                println!("[DEBUG] Key W Pressed - Moving Forward (speed: {})", speed);
                                camera.move_forward(speed);
                                println!("[DEBUG] Camera Position: {:?}", camera.position);
                            }
                            KeyCode::KeyS => {
                                println!("[DEBUG] Key S Pressed - Moving Backward");
                                camera.move_forward(-speed);
                                println!("[DEBUG] Camera Position: {:?}", camera.position);
                            }
                            KeyCode::KeyA => {
                                println!("[DEBUG] Key A Pressed - Strafing Left");
                                camera.move_right(-speed);
                                println!("[DEBUG] Camera Position: {:?}", camera.position);
                            }
                            KeyCode::KeyD => {
                                println!("[DEBUG] Key D Pressed - Strafing Right");
                                camera.move_right(speed);
                                println!("[DEBUG] Camera Position: {:?}", camera.position);
                            }
                            KeyCode::Space => {
                                println!("[DEBUG] Space Pressed - Moving Up");
                                camera.move_up(speed);
                                println!("[DEBUG] Camera Position: {:?}", camera.position);
                            }
                            KeyCode::ShiftLeft => {
                                println!("[DEBUG] Shift Pressed - Moving Down");
                                camera.move_up(-speed);
                                println!("[DEBUG] Camera Position: {:?}", camera.position);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    });

    println!("[ENGINE] Launching application...\n");

    // Run the application
    if let Err(e) = app.run() {
        eprintln!("Engine error: {}", e);
    }
}
