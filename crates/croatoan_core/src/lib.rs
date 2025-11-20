use croatoan_render::GraphicsContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, CursorGrabMode},
};
use std::sync::Arc;

// Re-export winit event types for use in game code
pub use winit::event::{DeviceEvent, ElementState, KeyEvent};
pub use winit::keyboard::{KeyCode, PhysicalKey};
pub use winit::event::Event as WinitEvent;
pub use winit::event::WindowEvent as WinitWindowEvent;

/// Main application structure that manages the engine loop
pub struct App {
    title: String,
    width: u32,
    height: u32,
    render_callback: Option<Box<dyn FnMut(&mut GraphicsContext) + 'static>>,
    input_callback: Option<Box<dyn FnMut(&Event<()>) + 'static>>,
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
        }
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
        F: FnMut(&Event<()>) + 'static,
    {
        self.input_callback = Some(Box::new(callback));
    }

    /// Run the application event loop
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize logging
        env_logger::init();

        // Create event loop
        let event_loop = EventLoop::new()?;

        // Create window
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(&self.title)
                .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height))
                .build(&event_loop)?
        );

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
                callback(&event);
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
