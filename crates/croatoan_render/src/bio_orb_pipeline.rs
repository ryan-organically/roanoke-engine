//! Bioluminescent Orb Rendering Pipeline
//!
//! Renders glowing fungi, moss, and crystal orbs in caves.
//! Uses additive blending for natural glow accumulation
//! and time-based pulsing for organic feel.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline};

/// Maximum bioluminescent orb instances per frame
const MAX_BIO_ORB_INSTANCES: usize = 10_000;

/// Instance data for a single bioluminescent orb
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BioOrbInstance {
    /// World position
    pub position: [f32; 3],
    /// Radius/size of the orb
    pub radius: f32,
    /// RGB glow color
    pub color: [f32; 3],
    /// Base intensity (0.5-2.0)
    pub intensity: f32,
    /// Pulse phase offset (0-2*PI)
    pub pulse_phase: f32,
    /// Pulse speed (0.3-0.8)
    pub pulse_speed: f32,
    /// Padding for alignment
    pub _padding: [f32; 2],
}

/// Camera and time uniform data
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraTimeUniform {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    time: f32,
}

/// Pipeline for rendering bioluminescent cave orbs
pub struct BioOrbPipeline {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    instance_buffer: Option<Buffer>,
    instance_count: u32,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl BioOrbPipeline {
    /// Create a new bioluminescent orb pipeline
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create bind group layout for camera/time uniforms
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bio Orb Camera Layout"),
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
            label: Some("Bio Orb Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bio Orb Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/bio_orb.wgsl").into(),
            ),
        });

        // Create render pipeline with additive blending
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bio Orb Pipeline"),
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
                        array_stride: std::mem::size_of::<BioOrbInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Position
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // Radius
                            wgpu::VertexAttribute {
                                offset: 12,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32,
                            },
                            // Color
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 7,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // Intensity
                            wgpu::VertexAttribute {
                                offset: 28,
                                shader_location: 8,
                                format: wgpu::VertexFormat::Float32,
                            },
                            // Pulse phase
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 9,
                                format: wgpu::VertexFormat::Float32,
                            },
                            // Pulse speed
                            wgpu::VertexAttribute {
                                offset: 36,
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
                    // Additive blending for glow accumulation
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Max,
                        },
                    }),
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
            // Read depth but don't write (for proper layering of glows)
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Don't write depth for additive objects
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bio Orb Camera Buffer"),
            size: std::mem::size_of::<CameraTimeUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bio Orb Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Generate small sphere mesh (for orbs)
        let (vertices, indices) = generate_sphere(8, 6); // Lower poly for many instances

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bio Orb Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bio Orb Index Buffer"),
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

    /// Upload instance data for all visible bioluminescent orbs
    pub fn upload_instances(&mut self, device: &Device, instances: &[BioOrbInstance]) {
        if instances.is_empty() {
            self.instance_count = 0;
            self.instance_buffer = None;
            return;
        }

        // Safety clamp
        let count = instances.len().min(MAX_BIO_ORB_INSTANCES);
        if instances.len() > MAX_BIO_ORB_INSTANCES {
            log::warn!(
                "Too many bio orb instances ({} requested), clamping to {}",
                instances.len(),
                MAX_BIO_ORB_INSTANCES
            );
        }

        self.instance_count = count as u32;
        self.instance_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Bio Orb Instance Buffer"),
                contents: bytemuck::cast_slice(&instances[..count]),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
    }

    /// Update camera uniforms with time for pulsing animation
    pub fn update_camera(&self, queue: &Queue, view_proj: &Mat4, camera_pos: Vec3, time: f32) {
        let uniform = CameraTimeUniform {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            time,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render all bioluminescent orbs
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count == 0 {
            return;
        }

        let instance_buffer = match &self.instance_buffer {
            Some(ib) => ib,
            None => return,
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
    }

    /// Check if the pipeline has instances ready
    pub fn is_ready(&self) -> bool {
        self.instance_buffer.is_some() && self.instance_count > 0
    }

    /// Get instance count
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

    for ring in 0..=rings {
        let phi = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = phi.cos();
        let ring_radius = phi.sin();

        for segment in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let position = [x, y, z];
            let normal = [x, y, z];

            vertices.push(SphereVertex { position, normal });
        }
    }

    for ring in 0..rings {
        for segment in 0..segments {
            let current = ring * (segments + 1) + segment;
            let next = current + segments + 1;

            indices.push(current);
            indices.push(next);
            indices.push(current + 1);

            indices.push(current + 1);
            indices.push(next);
            indices.push(next + 1);
        }
    }

    (vertices, indices)
}

/// Convert BioOrb data from croatoan_wfc to render instances
pub fn bio_orbs_to_instances(
    orbs: &[(glam::Vec3, glam::Vec3, [f32; 3], f32, f32, f32, f32)],
) -> Vec<BioOrbInstance> {
    orbs.iter()
        .map(|(pos, _normal, color, intensity, pulse_phase, pulse_speed, size)| {
            BioOrbInstance {
                position: pos.to_array(),
                radius: *size * 0.3, // Scale down for rendering
                color: *color,
                intensity: *intensity,
                pulse_phase: *pulse_phase,
                pulse_speed: *pulse_speed,
                _padding: [0.0, 0.0],
            }
        })
        .collect()
}
