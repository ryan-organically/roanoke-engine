//! Animal model rendering pipeline
//!
//! Renders actual 3D animal models loaded from GLTF files.
//! Supports instanced rendering for multiple animals of the same species.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline};

use crate::pipeline_validation::{sanitize_float, sanitize_vec3};

/// Maximum animal instances per species per frame
const MAX_INSTANCES_PER_SPECIES: usize = 500;

/// Vertex data for animal models
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct AnimalVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Instance data for a single animal
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct AnimalInstance {
    /// Model matrix (position, rotation, scale)
    pub model_matrix: [[f32; 4]; 4],
    /// RGB tint color
    pub color: [f32; 3],
    /// Emissive intensity (for damage flash, etc.)
    pub emissive: f32,
}

impl AnimalInstance {
    pub fn new(transform: Mat4, color: [f32; 3], emissive: f32) -> Self {
        Self {
            model_matrix: transform.to_cols_array_2d(),
            color,
            emissive,
        }
    }
}

/// Camera uniform data
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    time: f32,
    fog_color: [f32; 3],
    fog_start: f32,
    fog_end: f32,
    fog_density: f32,
    _padding: [f32; 2],
}

/// GPU mesh data for a single animal species
pub struct AnimalMeshGpu {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub vertex_count: u32,
}

/// Pipeline for rendering animal models
pub struct AnimalModelPipeline {
    pipeline: RenderPipeline,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    /// Mesh data per species (keyed by species name)
    species_meshes: HashMap<String, AnimalMeshGpu>,
    /// Instance buffer per species
    instance_buffers: HashMap<String, (Buffer, u32)>,
}

impl AnimalModelPipeline {
    /// Create a new animal model pipeline
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create bind group layout for camera uniforms
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Animal Model Camera Layout"),
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

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Animal Model Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Animal Model Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/animal_model.wgsl").into(),
            ),
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Animal Model Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<AnimalVertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // Position
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // Normal
                            wgpu::VertexAttribute {
                                offset: 12,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // UV
                            wgpu::VertexAttribute {
                                offset: 24,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    // Instance buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<AnimalInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Model matrix row 0
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Model matrix row 1
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Model matrix row 2
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 7,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Model matrix row 3
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 8,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Color
                            wgpu::VertexAttribute {
                                offset: 64,
                                shader_location: 9,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // Emissive
                            wgpu::VertexAttribute {
                                offset: 76,
                                shader_location: 10,
                                format: wgpu::VertexFormat::Float32,
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
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
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

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Animal Model Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animal Model Camera Bind Group"),
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
            species_meshes: HashMap::new(),
            instance_buffers: HashMap::new(),
        }
    }

    /// Upload mesh data for a species
    pub fn upload_species_mesh(
        &mut self,
        device: &Device,
        species_name: &str,
        vertices: &[AnimalVertex],
        indices: &[u32],
    ) {
        if vertices.is_empty() || indices.is_empty() {
            log::warn!("[AnimalModel] Skipping empty mesh for {}", species_name);
            return;
        }

        // Sanitize vertex data
        let sanitized_vertices: Vec<AnimalVertex> = vertices
            .iter()
            .map(|v| AnimalVertex {
                position: sanitize_vec3(v.position),
                normal: sanitize_vec3(v.normal),
                uv: [sanitize_float(v.uv[0]), sanitize_float(v.uv[1])],
            })
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Animal Vertex Buffer: {}", species_name)),
            contents: bytemuck::cast_slice(&sanitized_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Animal Index Buffer: {}", species_name)),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        log::info!(
            "[AnimalModel] Uploaded mesh for '{}': {} vertices, {} indices",
            species_name,
            vertices.len(),
            indices.len()
        );

        self.species_meshes.insert(
            species_name.to_string(),
            AnimalMeshGpu {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
                vertex_count: vertices.len() as u32,
            },
        );
    }

    /// Upload instance data for a species
    pub fn upload_instances(
        &mut self,
        device: &Device,
        species_name: &str,
        instances: &[AnimalInstance],
    ) {
        if instances.is_empty() {
            self.instance_buffers.remove(species_name);
            return;
        }

        // Clamp instance count
        let count = instances.len().min(MAX_INSTANCES_PER_SPECIES);
        if instances.len() > MAX_INSTANCES_PER_SPECIES {
            log::warn!(
                "[AnimalModel] Clamping {} instances to {} for {}",
                instances.len(),
                MAX_INSTANCES_PER_SPECIES,
                species_name
            );
        }

        // Sanitize instance data
        let sanitized: Vec<AnimalInstance> = instances[..count]
            .iter()
            .map(|inst| {
                let m = inst.model_matrix;
                AnimalInstance {
                    model_matrix: [
                        [sanitize_float(m[0][0]), sanitize_float(m[0][1]), sanitize_float(m[0][2]), sanitize_float(m[0][3])],
                        [sanitize_float(m[1][0]), sanitize_float(m[1][1]), sanitize_float(m[1][2]), sanitize_float(m[1][3])],
                        [sanitize_float(m[2][0]), sanitize_float(m[2][1]), sanitize_float(m[2][2]), sanitize_float(m[2][3])],
                        [sanitize_float(m[3][0]), sanitize_float(m[3][1]), sanitize_float(m[3][2]), sanitize_float(m[3][3])],
                    ],
                    color: sanitize_vec3(inst.color),
                    emissive: sanitize_float(inst.emissive),
                }
            })
            .collect();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Animal Instance Buffer: {}", species_name)),
            contents: bytemuck::cast_slice(&sanitized),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.instance_buffers
            .insert(species_name.to_string(), (buffer, count as u32));
    }

    /// Update camera uniforms
    pub fn update_camera(
        &self,
        queue: &Queue,
        view_proj: &Mat4,
        camera_pos: Vec3,
        time: f32,
        fog_color: Vec3,
        fog_start: f32,
        fog_end: f32,
        fog_density: f32,
    ) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            time,
            fog_color: fog_color.to_array(),
            fog_start,
            fog_end,
            fog_density,
            _padding: [0.0; 2],
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render all animal models
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        // Render each species that has both mesh and instances
        for (species_name, mesh) in &self.species_meshes {
            if let Some((instance_buffer, instance_count)) = self.instance_buffers.get(species_name)
            {
                if *instance_count == 0 {
                    continue;
                }

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..*instance_count);
            }
        }
    }

    /// Check if a species has mesh data uploaded
    pub fn has_mesh(&self, species_name: &str) -> bool {
        self.species_meshes.contains_key(species_name)
    }

    /// Get the list of loaded species
    pub fn loaded_species(&self) -> Vec<&str> {
        self.species_meshes.keys().map(|s| s.as_str()).collect()
    }

    /// Get total instance count across all species
    pub fn total_instance_count(&self) -> u32 {
        self.instance_buffers.values().map(|(_, count)| count).sum()
    }
}
