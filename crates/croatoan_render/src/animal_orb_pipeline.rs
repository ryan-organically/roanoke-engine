//! Animal orb rendering pipeline
//!
//! Renders colored spheres to represent animals in the world.
//! Each species has a distinct base color, modified by behavior state.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline};

use crate::pipeline_validation::{
    PipelineError, PipelineResult, sanitize_float, sanitize_vec3,
};

/// Maximum animal orb instances per frame (safety limit)
const MAX_ORB_INSTANCES: usize = 5_000;

/// Instance data for a single animal orb
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct OrbInstance {
    /// Model matrix (position, rotation, scale)
    pub model_matrix: [[f32; 4]; 4],
    /// RGB color
    pub color: [f32; 3],
    /// Emissive intensity (for glow effect)
    pub emissive: f32,
}

/// Camera uniform data
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _padding: f32,
}

/// Pipeline for rendering animal representation orbs
pub struct AnimalOrbPipeline {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    instance_buffer: Option<Buffer>,
    instance_count: u32,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl AnimalOrbPipeline {
    /// Create a new animal orb pipeline
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create bind group layout for camera uniforms
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Animal Orb Camera Layout"),
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
            label: Some("Animal Orb Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Animal Orb Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/animal_orb.wgsl").into(),
            ),
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Animal Orb Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex buffer layout (sphere mesh)
                    wgpu::VertexBufferLayout {
                        array_stride: 24, // position (12) + normal (12)
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
                        ],
                    },
                    // Instance buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<OrbInstance>() as u64,
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            label: Some("Animal Orb Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animal Orb Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Generate sphere mesh
        let (vertices, indices) = generate_sphere(12, 8); // segments, rings

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animal Orb Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animal Orb Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            instance_buffer: None,
            instance_count: 0,
            camera_buffer,
            camera_bind_group,
        }
    }

    /// Upload instance data with validation
    ///
    /// # Errors
    /// Returns `PipelineError` if instance count exceeds limit
    pub fn try_upload_instances(&mut self, device: &Device, instances: &[OrbInstance]) -> PipelineResult<()> {
        if instances.len() > MAX_ORB_INSTANCES {
            return Err(PipelineError::TooManyInstances {
                count: instances.len(),
                max: MAX_ORB_INSTANCES,
            });
        }
        self.upload_instances_unchecked(device, instances);
        Ok(())
    }

    /// Upload instance data for all visible animal orbs (clamps if too many)
    pub fn upload_instances(&mut self, device: &Device, instances: &[OrbInstance]) {
        if instances.is_empty() {
            self.instance_count = 0;
            self.instance_buffer = None;
            return;
        }

        // Safety check: clamp instance count
        if instances.len() > MAX_ORB_INSTANCES {
            log::warn!("Too many animal orb instances ({} requested), clamping to {}", instances.len(), MAX_ORB_INSTANCES);
        }

        self.upload_instances_unchecked(device, &instances[..instances.len().min(MAX_ORB_INSTANCES)]);
    }

    /// Upload instances without validation (internal use)
    fn upload_instances_unchecked(&mut self, device: &Device, instances: &[OrbInstance]) {
        if instances.is_empty() {
            self.instance_count = 0;
            self.instance_buffer = None;
            return;
        }

        self.instance_count = instances.len() as u32;

        // Sanitize instance data to prevent GPU undefined behavior
        let sanitized: Vec<OrbInstance> = instances.iter()
            .map(|inst| {
                let m = inst.model_matrix;
                OrbInstance {
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

        self.instance_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Animal Orb Instance Buffer"),
                contents: bytemuck::cast_slice(&sanitized),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
    }

    /// Update camera uniforms
    pub fn update_camera(&self, queue: &Queue, view_proj: &Mat4, camera_pos: Vec3) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            _padding: 0.0,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render all animal orbs
    ///
    /// # Safety
    /// This method uses defensive checks to avoid panics even with invalid state.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Early exit if no instances to render
        if self.instance_count == 0 {
            return;
        }

        // Defensive: require instance buffer
        let instance_buffer = match &self.instance_buffer {
            Some(ib) => ib,
            None => {
                log::trace!("Animal orb render skipped: no instance buffer");
                return;
            }
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
    }

    /// Check if the pipeline has instances ready for rendering
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.instance_buffer.is_some() && self.instance_count > 0
    }

    /// Get the number of instances currently uploaded
    pub fn instance_count(&self) -> u32 {
        self.instance_count
    }
}

/// Sphere vertex with position and normal
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct SphereVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

/// Generate a UV sphere mesh
fn generate_sphere(segments: u32, rings: u32) -> (Vec<SphereVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for ring in 0..=rings {
        let phi = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = phi.cos();
        let ring_radius = phi.sin();

        for segment in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = [x, y, z];
            let normal = [x, y, z]; // For a unit sphere, position = normal

            vertices.push(SphereVertex { position, normal });
        }
    }

    // Generate indices
    for ring in 0..rings {
        for segment in 0..segments {
            let current = ring * (segments + 1) + segment;
            let next = current + segments + 1;

            // First triangle
            indices.push(current);
            indices.push(next);
            indices.push(current + 1);

            // Second triangle
            indices.push(current + 1);
            indices.push(next);
            indices.push(next + 1);
        }
    }

    (vertices, indices)
}
