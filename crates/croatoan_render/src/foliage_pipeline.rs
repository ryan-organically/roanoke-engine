//! Foliage Pipeline - Multi-model tree and shrub rendering
//!
//! Renders trees and shrubs loaded from GLTF files with instanced rendering.
//! Supports multiple models for variety (e.g., 5 different tree types, 5 shrub types).

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPipeline};

use crate::pipeline_validation::{sanitize_float, sanitize_vec3};

/// Maximum instances per model type per frame
const MAX_INSTANCES_PER_MODEL: usize = 2000;

/// Vertex data for foliage models
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FoliageVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Instance data for a single foliage item
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FoliageInstance {
    /// Model matrix (position, rotation, scale)
    pub model_matrix: [[f32; 4]; 4],
}

impl FoliageInstance {
    pub fn from_mat4(m: &Mat4) -> Self {
        Self {
            model_matrix: m.to_cols_array_2d(),
        }
    }
}

/// Camera/lighting uniform data
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],  // 64 bytes
    sun_dir: [f32; 3],          // 12 bytes
    time: f32,                  // 4 bytes
    view_pos: [f32; 3],         // 12 bytes
    fog_density: f32,           // 4 bytes
    fog_color: [f32; 3],        // 12 bytes
    fog_start: f32,             // 4 bytes
    fog_end: f32,               // 4 bytes
    alpha_cutoff: f32,          // 4 bytes
    use_texture: f32,           // 4 bytes
    wind_strength: f32,         // 4 bytes - for tree sway
}

/// GPU mesh data for a single foliage model
pub struct FoliageMeshGpu {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub texture_bind_group: Option<Arc<BindGroup>>,
}

/// Instance data for rendering (model index + transform)
pub struct FoliageInstanceData {
    pub model_index: usize,
    pub transform: Mat4,
}

/// Pipeline for rendering foliage (trees and shrubs)
pub struct FoliagePipeline {
    pipeline: RenderPipeline,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,
    default_bind_group: BindGroup,

    /// Loaded models by name (e.g., "tree_oak", "shrub_fern")
    models: HashMap<String, FoliageMeshGpu>,
    /// Model names in order for index-based lookup
    model_names: Vec<String>,
    /// Instance buffers per model
    instance_buffers: HashMap<String, (Buffer, u32)>,
}

impl FoliagePipeline {
    /// Create a new foliage pipeline
    pub fn new(device: &Device, queue: &Queue, surface_format: wgpu::TextureFormat) -> Self {
        // Group 0: Camera uniforms
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Foliage Camera Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Group 1: Texture
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Foliage Texture Layout"),
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

        // Create default white texture for untextured models
        let default_texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let default_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Foliage Default Texture"),
            size: default_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &default_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            default_texture_size,
        );

        let default_texture_view =
            default_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let default_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&default_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&default_sampler),
                },
            ],
            label: Some("Foliage Default Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Foliage Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Use tree shader (has wind animation, procedural coloring)
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Foliage Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/tree.wgsl").into(),
            ),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Foliage Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<FoliageVertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            wgpu::VertexAttribute {
                                offset: 12,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            wgpu::VertexAttribute {
                                offset: 24,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    // Instance buffer (4x4 matrix)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<FoliageInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 7,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 8,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Two-sided for leaves
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Camera buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Foliage Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Foliage Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
            texture_bind_group_layout,
            default_bind_group,
            models: HashMap::new(),
            model_names: Vec::new(),
            instance_buffers: HashMap::new(),
        }
    }

    /// Register a model from vertex data
    pub fn register_model(
        &mut self,
        device: &Device,
        name: &str,
        vertices: &[FoliageVertex],
        indices: &[u32],
        texture_bind_group: Option<Arc<BindGroup>>,
    ) {
        if vertices.is_empty() || indices.is_empty() {
            log::warn!("[Foliage] Skipping empty model '{}'", name);
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Foliage Vertex Buffer: {}", name)),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Foliage Index Buffer: {}", name)),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        log::info!(
            "[Foliage] Registered model '{}': {} vertices, {} indices",
            name,
            vertices.len(),
            indices.len()
        );

        if !self.model_names.contains(&name.to_string()) {
            self.model_names.push(name.to_string());
        }

        self.models.insert(
            name.to_string(),
            FoliageMeshGpu {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
                texture_bind_group,
            },
        );
    }

    /// Register a model from raw arrays (convenience wrapper)
    pub fn register_model_from_arrays(
        &mut self,
        device: &Device,
        name: &str,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
        texture_bind_group: Option<Arc<BindGroup>>,
    ) {
        let vertices: Vec<FoliageVertex> = positions
            .iter()
            .zip(normals.iter())
            .zip(uvs.iter())
            .map(|((p, n), uv)| FoliageVertex {
                position: sanitize_vec3(*p),
                normal: sanitize_vec3(*n),
                uv: [sanitize_float(uv[0]), sanitize_float(uv[1])],
            })
            .collect();

        self.register_model(device, name, &vertices, indices, texture_bind_group);
    }

    /// Create a texture bind group for a model
    pub fn create_texture_bind_group(
        &self,
        device: &Device,
        texture_view: &wgpu::TextureView,
        label: Option<&str>,
    ) -> BindGroup {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label,
        })
    }

    /// Get list of registered model names
    pub fn model_names(&self) -> &[String] {
        &self.model_names
    }

    /// Get number of registered models
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Check if a model is registered
    pub fn has_model(&self, name: &str) -> bool {
        self.models.contains_key(name)
    }

    /// Upload instances for a specific model
    pub fn upload_instances(&mut self, device: &Device, model_name: &str, instances: &[Mat4]) {
        if instances.is_empty() {
            self.instance_buffers.remove(model_name);
            return;
        }

        let count = instances.len().min(MAX_INSTANCES_PER_MODEL);
        if instances.len() > MAX_INSTANCES_PER_MODEL {
            log::warn!(
                "[Foliage] Too many instances for '{}': {} (max {})",
                model_name,
                instances.len(),
                MAX_INSTANCES_PER_MODEL
            );
        }

        let instance_data: Vec<FoliageInstance> = instances[..count]
            .iter()
            .map(FoliageInstance::from_mat4)
            .collect();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Foliage Instance Buffer: {}", model_name)),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.instance_buffers
            .insert(model_name.to_string(), (buffer, count as u32));
    }

    /// Upload instances for all models from a map
    pub fn upload_all_instances(
        &mut self,
        device: &Device,
        instances_by_model: &HashMap<String, Vec<Mat4>>,
    ) {
        // Clear old buffers for models not in the new set
        let new_models: std::collections::HashSet<_> = instances_by_model.keys().collect();
        self.instance_buffers
            .retain(|k, _| new_models.contains(k));

        // Upload new instances
        for (model_name, instances) in instances_by_model {
            self.upload_instances(device, model_name, instances);
        }
    }

    /// Update camera uniforms
    pub fn update_camera(
        &self,
        queue: &Queue,
        view_proj: &Mat4,
        sun_dir: [f32; 3],
        time: f32,
        view_pos: [f32; 3],
        fog_color: [f32; 3],
        fog_start: f32,
        fog_end: f32,
        fog_density: f32,
    ) {
        self.update_camera_full(
            queue,
            view_proj,
            sun_dir,
            time,
            view_pos,
            fog_color,
            fog_start,
            fog_end,
            fog_density,
            0.5, // alpha_cutoff
            0.0, // use_texture (0 = procedural)
            1.0, // wind_strength
        );
    }

    /// Update camera uniforms with all parameters
    pub fn update_camera_full(
        &self,
        queue: &Queue,
        view_proj: &Mat4,
        sun_dir: [f32; 3],
        time: f32,
        view_pos: [f32; 3],
        fog_color: [f32; 3],
        fog_start: f32,
        fog_end: f32,
        fog_density: f32,
        alpha_cutoff: f32,
        use_texture: f32,
        wind_strength: f32,
    ) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            sun_dir,
            time,
            view_pos,
            fog_density,
            fog_color,
            fog_start,
            fog_end,
            alpha_cutoff,
            use_texture,
            wind_strength,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render all foliage models with their instances
    pub fn render<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        // Render each model with its instances
        for (model_name, mesh) in &self.models {
            let (instance_buffer, instance_count) = match self.instance_buffers.get(model_name) {
                Some((buf, count)) if *count > 0 => (buf, *count),
                _ => continue,
            };

            // Set texture bind group
            let texture_bg = mesh
                .texture_bind_group
                .as_ref()
                .map_or(&self.default_bind_group, |arc| arc.as_ref());
            render_pass.set_bind_group(1, texture_bg, &[]);

            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..instance_count);
        }
    }

    /// Get total instance count across all models
    pub fn total_instance_count(&self) -> u32 {
        self.instance_buffers.values().map(|(_, c)| *c).sum()
    }

    /// Check if pipeline has any models loaded
    pub fn is_ready(&self) -> bool {
        !self.models.is_empty()
    }
}
