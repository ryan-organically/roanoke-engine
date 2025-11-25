use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BuildingVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

pub struct BuildingMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct BuildingPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    mesh: Option<Arc<BuildingMesh>>,
    instance_buffer: Option<wgpu::Buffer>,
    instance_count: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    light_dir: [f32; 3],
    _padding: f32,
    view_pos: [f32; 3],
    _padding2: f32,
    fog_color: [f32; 3],
    _padding3: f32,
    fog_start: f32,
    fog_end: f32,
    _padding4: [f32; 2],
}

impl BuildingPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../../assets/shaders/building.wgsl"));

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                light_dir: [0.5, 1.0, 0.3],
                _padding: 0.0,
                view_pos: [0.0; 3],
                _padding2: 0.0,
                fog_color: [0.5, 0.6, 0.7],
                _padding3: 0.0,
                fog_start: 100.0,
                fog_end: 500.0,
                _padding4: [0.0; 2],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Building Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Building Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Building Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Building Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex Buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<BuildingVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 }, // Pos
                            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 }, // Normal
                            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2 }, // UV
                            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 32, shader_location: 3 }, // Color
                        ],
                    },
                    // Instance Buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 5 },
                            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 6 },
                            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 7 },
                            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 48, shader_location: 8 },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            mesh: None,
            instance_buffer: None,
            instance_count: 0,
        }
    }

    pub fn create_mesh(
        device: &wgpu::Device,
        vertices: &[BuildingVertex],
        indices: &[u32],
    ) -> Arc<BuildingMesh> {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Arc::new(BuildingMesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        })
    }

    pub fn set_mesh(&mut self, mesh: Arc<BuildingMesh>) {
        self.mesh = Some(mesh);
    }

    pub fn upload_instances(&mut self, device: &wgpu::Device, instances: &[Mat4]) {
        let raw_data: Vec<InstanceRaw> = instances.iter().map(|m| InstanceRaw {
            model: m.to_cols_array_2d(),
        }).collect();

        self.instance_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Instance Buffer"),
            contents: bytemuck::cast_slice(&raw_data),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        self.instance_count = instances.len() as u32;
    }

    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        view_proj: &Mat4,
        light_dir: Vec3,
        view_pos: Vec3,
        fog_color: [f32; 3],
        fog_start: f32,
        fog_end: f32,
    ) {
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            light_dir: light_dir.to_array(),
            _padding: 0.0,
            view_pos: view_pos.to_array(),
            _padding2: 0.0,
            fog_color,
            _padding3: 0.0,
            fog_start,
            fog_end,
            _padding4: [0.0; 2],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        if let Some(mesh) = &self.mesh {
            if self.instance_count > 0 {
                if let Some(instance_buffer) = &self.instance_buffer {
                    rpass.set_pipeline(&self.pipeline);
                    rpass.set_bind_group(0, &self.bind_group, &[]);
                    rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    rpass.set_vertex_buffer(1, instance_buffer.slice(..));
                    rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..mesh.index_count, 0, 0..self.instance_count);
                }
            }
        }
    }
}
