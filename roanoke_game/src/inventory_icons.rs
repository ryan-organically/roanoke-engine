//! Inventory Icon System
//!
//! Renders 3D models as rotating icons for inventory display.
//! Uses the IconRenderPipeline from croatoan_render.

use croatoan_render::{IconRenderPipeline, IconVertex, RenderedIcon, ICON_SIZE};
use std::collections::HashMap;

/// Convert RenderedIcon to egui ColorImage
fn to_egui_image(icon: &RenderedIcon) -> egui::ColorImage {
    egui::ColorImage::from_rgba_unmultiplied(
        [icon.width as usize, icon.height as usize],
        &icon.pixels,
    )
}

/// Model configuration for an inventory item icon
pub struct IconModelConfig {
    pub template_id: &'static str,
    pub model_path: &'static str,
    pub scale: f32,
}

/// Get all registered icon model configurations
pub fn get_icon_configs() -> Vec<IconModelConfig> {
    vec![
        // Weapons - scaled large to fill icon frame
        IconModelConfig {
            template_id: "flintlock_pistol",
            model_path: "assets/models/weapons/flintlock_lod2.glb",
            scale: 5.0,  // Large to fill icon frame
        },
        IconModelConfig {
            template_id: "dagger",
            model_path: "assets/models/weapons/dagger_lod2.glb",
            scale: 1.5,  // Dagger model is large, scale down to fit view
        },
        IconModelConfig {
            template_id: "hatchet",
            model_path: "assets/models/weapons/hatchet_lod2.glb",
            scale: 5.0,  // Large to fill icon frame
        },
        // Animal carcasses
        IconModelConfig {
            template_id: "pheasant_carcass",
            model_path: "assets/models/animals/ring_necked_pheasant.glb",
            scale: 0.15,
        },
    ]
}

/// Manages inventory icon rendering and caching
pub struct InventoryIconCache {
    /// The render pipeline for icons
    pub pipeline: Option<IconRenderPipeline>,
    /// Cached egui textures by template_id
    pub textures: HashMap<String, egui::TextureHandle>,
    /// Current rotation angle for animation
    pub rotation: f32,
    /// Whether the system has been initialized
    pub initialized: bool,
    /// List of models that need to be loaded
    pending_loads: Vec<IconModelConfig>,
}

impl Default for InventoryIconCache {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryIconCache {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            textures: HashMap::new(),
            rotation: 0.0,
            initialized: false,
            pending_loads: get_icon_configs(),
        }
    }

    /// Initialize the icon render pipeline (call once GPU is ready)
    pub fn init_pipeline(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.pipeline.is_none() {
            self.pipeline = Some(IconRenderPipeline::new(device, queue));
            log::info!("[InventoryIcons] Initialized render pipeline");
        }
    }

    /// Load pending models into the pipeline
    pub fn load_models(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let pipeline = match &mut self.pipeline {
            Some(p) => p,
            None => return,
        };

        // Take pending loads to avoid borrow issues
        let pending: Vec<IconModelConfig> = self.pending_loads.drain(..).collect();

        for config in pending {
            if pipeline.has_model(config.template_id) {
                continue;
            }

            // Load the GLB model
            match crate::gltf_loader::load_gltf(config.model_path) {
                Ok(model) => {
                    let mut vertices: Vec<IconVertex> = Vec::new();
                    let mut indices: Vec<u32> = Vec::new();
                    let mut texture_data: Option<(Vec<u8>, u32, u32)> = None;

                    for mesh in &model.meshes {
                        let base_index = vertices.len() as u32;

                        for i in 0..mesh.positions.len() {
                            vertices.push(IconVertex {
                                position: mesh.positions[i],
                                normal: mesh.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                                uv: mesh.uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                            });
                        }

                        for idx in &mesh.indices {
                            indices.push(*idx + base_index);
                        }

                        // Get texture from material
                        if texture_data.is_none() {
                            if let Some(tex) = mesh.material.get_or_create_texture() {
                                texture_data = Some((tex.data.clone(), tex.width, tex.height));
                            }
                        }
                    }

                    if !vertices.is_empty() && !indices.is_empty() {
                        let tex_ref = texture_data.as_ref()
                            .map(|(data, w, h)| (data.as_slice(), *w, *h));

                        pipeline.upload_model(
                            device,
                            queue,
                            config.template_id,
                            &vertices,
                            &indices,
                            tex_ref,
                            config.scale,
                        );

                        log::info!("[InventoryIcons] Loaded model: {} ({} verts)",
                            config.template_id, vertices.len());
                    }
                }
                Err(e) => {
                    log::warn!("[InventoryIcons] Failed to load {}: {}", config.model_path, e);
                }
            }
        }
    }

    /// Update rotation for animated icons
    pub fn update(&mut self, delta: f32) {
        self.rotation += delta * 0.8; // Rotation speed
        if self.rotation > std::f32::consts::TAU {
            self.rotation -= std::f32::consts::TAU;
        }
    }

    /// Render an icon and cache as egui texture
    pub fn render_and_cache(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        template_id: &str,
        egui_ctx: &egui::Context,
    ) -> bool {
        let pipeline = match &self.pipeline {
            Some(p) => p,
            None => return false,
        };

        if !pipeline.has_model(template_id) {
            return false;
        }

        // Render the icon
        if let Some(rendered) = pipeline.render_icon(device, queue, template_id, self.rotation) {
            let image = to_egui_image(&rendered);
            let texture = egui_ctx.load_texture(
                format!("icon_{}", template_id),
                image,
                egui::TextureOptions::LINEAR,
            );
            self.textures.insert(template_id.to_string(), texture);
            return true;
        }

        false
    }

    /// Get or render an icon texture
    pub fn get_icon(&self, template_id: &str) -> Option<&egui::TextureHandle> {
        self.textures.get(template_id)
    }

    /// Check if a model is available for an item
    pub fn has_model(&self, template_id: &str) -> bool {
        self.pipeline.as_ref().map_or(false, |p| p.has_model(template_id))
    }
}

/// Initialize the icon system (call during game initialization with GPU access)
pub fn initialize_icons(
    cache: &mut InventoryIconCache,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    egui_ctx: &egui::Context,
) {
    // Initialize pipeline if needed
    cache.init_pipeline(device, queue);

    // Load models
    cache.load_models(device, queue);

    // Pre-render all icons
    let pipeline = match &cache.pipeline {
        Some(p) => p,
        None => return,
    };

    let model_ids: Vec<String> = pipeline.loaded_models().iter().map(|s| s.to_string()).collect();

    for template_id in model_ids {
        cache.render_and_cache(device, queue, &template_id, egui_ctx);
    }

    cache.initialized = true;
    log::info!("[InventoryIcons] Initialized {} icon textures", cache.textures.len());
}

/// Update icons each frame (re-renders with new rotation)
pub fn update_icons(
    cache: &mut InventoryIconCache,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    egui_ctx: &egui::Context,
    delta: f32,
) {
    cache.update(delta);

    // Re-render all loaded icons with updated rotation
    let pipeline = match &cache.pipeline {
        Some(p) => p,
        None => return,
    };

    let model_ids: Vec<String> = pipeline.loaded_models().iter().map(|s| s.to_string()).collect();

    for template_id in model_ids {
        cache.render_and_cache(device, queue, &template_id, egui_ctx);
    }
}
