use wgpu::{Device, Queue, RenderPipeline, Buffer, BindGroup, util::DeviceExt};
use bytemuck::{Pod, Zeroable};
use glam::Mat4;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct DetritusVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

pub struct DetritusPipeline {
    pipeline: RenderPipeline,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    index_count: u32,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl DetritusPipeline {
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        // Camera bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Detritus Camera Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Detritus Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Detritus Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/detritus.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Detritus Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<DetritusVertex>() as wgpu::BufferAddress,
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
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        // UV
                        wgpu::VertexAttribute {
                            offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Detritus Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Detritus Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
            camera_buffer,
            camera_bind_group,
        }
    }

    /// Upload detritus mesh data to GPU
    pub fn upload_mesh(
        &mut self,
        device: &Device,
        _queue: &Queue,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
    ) {
        // Safety check: GPU has 256 MB max buffer size
        const MAX_VERTICES: usize = 1_000_000; // ~80 MB vertex buffer
        const MAX_INDICES: usize = 3_000_000;  // ~12 MB index buffer

        if positions.len() > MAX_VERTICES {
            log::warn!("Detritus mesh too large ({} vertices), skipping. Max: {}", positions.len(), MAX_VERTICES);
            return;
        }

        if indices.len() > MAX_INDICES {
            log::warn!("Detritus mesh too large ({} indices), skipping. Max: {}", indices.len(), MAX_INDICES);
            return;
        }

        // Interleave vertex data
        let vertices: Vec<DetritusVertex> = (0..positions.len())
            .map(|i| DetritusVertex {
                position: positions[i],
                normal: normals[i],
                uv: uvs[i],
            })
            .collect();

        // Create vertex buffer
        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Detritus Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        // Create index buffer
        self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Detritus Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        }));

        self.index_count = indices.len() as u32;

        log::info!("Uploaded detritus mesh: {} vertices, {} triangles", vertices.len(), indices.len() / 3);
    }

    /// Update camera uniform
    pub fn update_camera(&self, queue: &Queue, view_proj: &Mat4) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer) {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
    }
}
