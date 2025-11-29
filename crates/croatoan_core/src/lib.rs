use croatoan_render::GraphicsContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::sync::Arc;

// Re-export winit event types for use in game code
pub use winit::event::{DeviceEvent, ElementState, KeyEvent};
pub use winit::keyboard::{KeyCode, PhysicalKey};
pub use winit::event::Event as WinitEvent;
pub use winit::event::WindowEvent as WinitWindowEvent;
pub use winit::window::CursorGrabMode;

/// Main application structure that manages the engine loop
pub struct App {
    title: String,
    width: u32,
    height: u32,
    render_callback: Option<Box<dyn FnMut(&mut GraphicsContext) + 'static>>,
    input_callback: Option<Box<dyn FnMut(&Event<()>, &winit::window::Window) + 'static>>,
    key_states: std::collections::HashMap<KeyCode, ElementState>,
}

impl App {
    /// Create a new App with the specified title and dimensions
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            render_callback: None,
            input_callback: None,
            key_states: std::collections::HashMap::new(),
        }
    }

    /// Get the current state of a key
    pub fn get_key_state(&self, key: KeyCode) -> ElementState {
        *self.key_states.get(&key).unwrap_or(&ElementState::Released)
    }

    /// Set the render callback that will be called each frame
    pub fn set_render_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&mut GraphicsContext) + 'static,
    {
        self.render_callback = Some(Box::new(callback));
    }

    /// Set the input callback that will be called for input events
    pub fn set_input_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&Event<()>, &winit::window::Window) + 'static,
    {
        self.input_callback = Some(Box::new(callback));
    }

    /// Run the application event loop
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize logging
        env_logger::init();

        // Create event loop
        let event_loop = EventLoop::new()?;

        // Create window builder
        let mut window_builder = WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height));

        // Load Icon
        let icon_path = "assets/taskbar icon.jpg";
        if let Ok(image) = image::open(icon_path) {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            let icon_data = rgba.into_raw();
            
            if let Ok(icon) = winit::window::Icon::from_rgba(icon_data, width, height) {
                window_builder = window_builder.with_window_icon(Some(icon));
                log::info!("Loaded window icon from {}", icon_path);
            } else {
                log::warn!("Failed to create window icon from {}", icon_path);
            }
        } else {
            log::warn!("Failed to load window icon from {}", icon_path);
        }

        let window = Arc::new(window_builder.build(&event_loop)?);

        log::info!("Window created: {} ({}x{})", self.title, self.width, self.height);

        // Set cursor grab mode to confine cursor to window
        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
            log::warn!("Failed to confine cursor: {}", e);
            // Try locked mode as fallback
            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Locked) {
                log::warn!("Failed to lock cursor: {}", e);
            }
        }
        window.set_cursor_visible(false);

        // Initialize graphics context
        let mut graphics_context = GraphicsContext::new(window.clone());

        // Run the event loop
        let result = event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            // Call input callback for all events
            if let Some(callback) = &mut self.input_callback {
                callback(&event, &window);
            }

            // Update key states
            if let Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } = &event {
                if let PhysicalKey::Code(keycode) = key_event.physical_key {
                    self.key_states.insert(keycode, key_event.state);
                }
            }

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Close requested, exiting...");
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        graphics_context.resize(physical_size);
                        log::info!("Window resized to: {:?}", physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        // Call user-provided render callback if set
                        if let Some(callback) = &mut self.render_callback {
                            callback(&mut graphics_context);
                        } else {
                            // Default: clear to black
                            let _ = graphics_context.render(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            });
                        }
                        window.request_redraw();
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        });

        result.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
