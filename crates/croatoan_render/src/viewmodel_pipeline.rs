use wgpu::{Device, Queue, RenderPipeline, Buffer, BindGroup, BindGroupLayout};
use wgpu::util::DeviceExt;
use glam::{Vec3, Mat4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewModelUniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

pub struct ViewModelPipeline {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    #[allow(dead_code)]
    bind_group_layout: BindGroupLayout,
}

impl ViewModelPipeline {
    pub fn new(device: &Device, format: wgpu::TextureFormat) -> Self {
        // Generate arm and staff geometry
        let (vertices, indices) = Self::generate_viewmodel_geometry();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Viewmodel Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Viewmodel Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Viewmodel Uniform Buffer"),
            size: std::mem::size_of::<ViewModelUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Viewmodel Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Viewmodel Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Viewmodel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/viewmodel.wgsl").into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Viewmodel Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Viewmodel Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            uniform_buffer,
            bind_group,
            bind_group_layout,
        }
    }

    /// Generate geometry for viewmodel (arms + staff)
    fn generate_viewmodel_geometry() -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Right arm (simple rectangular prism) - MADE BIGGER AND MORE VISIBLE
        let arm_color = [0.9, 0.7, 0.5]; // Brighter skin tone
        Self::add_box(
            &mut vertices,
            &mut indices,
            Vec3::new(0.3, -0.3, -0.5),    // Position (right side, lower, closer)
            Vec3::new(0.12, 0.35, 0.12),   // Size (BIGGER)
            arm_color,
        );

        // Left arm (simple rectangular prism) - MADE BIGGER AND MORE VISIBLE
        Self::add_box(
            &mut vertices,
            &mut indices,
            Vec3::new(-0.3, -0.3, -0.5),   // Position (left side, lower, closer)
            Vec3::new(0.12, 0.35, 0.12),   // Size (BIGGER)
            arm_color,
        );

        // Staff/stick held in right hand - MADE MUCH BIGGER
        let staff_color = [0.6, 0.4, 0.2]; // Brighter brown wood color
        Self::add_box(
            &mut vertices,
            &mut indices,
            Vec3::new(0.25, -0.2, -0.45),  // Position (in right hand, more centered)
            Vec3::new(0.05, 0.8, 0.05),    // Size (MUCH BIGGER - thicker and taller stick)
            staff_color,
        );

        (vertices, indices)
    }

    /// Helper to add a box (rectangular prism) to the mesh
    fn add_box(
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u16>,
        pos: Vec3,
        size: Vec3,
        color: [f32; 3],
    ) {
        let base_idx = vertices.len() as u16;

        // Half extents
        let hx = size.x / 2.0;
        let hy = size.y / 2.0;
        let hz = size.z / 2.0;

        // 8 corners of the box
        let corners = [
            [pos.x - hx, pos.y - hy, pos.z - hz], // 0
            [pos.x + hx, pos.y - hy, pos.z - hz], // 1
            [pos.x + hx, pos.y + hy, pos.z - hz], // 2
            [pos.x - hx, pos.y + hy, pos.z - hz], // 3
            [pos.x - hx, pos.y - hy, pos.z + hz], // 4
            [pos.x + hx, pos.y - hy, pos.z + hz], // 5
            [pos.x + hx, pos.y + hy, pos.z + hz], // 6
            [pos.x - hx, pos.y + hy, pos.z + hz], // 7
        ];

        // Add vertices for each corner
        for corner in corners {
            vertices.push(Vertex {
                position: corner,
                color,
            });
        }

        // Add indices for each face (2 triangles per face, 6 faces)
        let box_indices: [u16; 36] = [
            // Front face
            0, 1, 2, 2, 3, 0,
            // Back face
            5, 4, 7, 7, 6, 5,
            // Left face
            4, 0, 3, 3, 7, 4,
            // Right face
            1, 5, 6, 6, 2, 1,
            // Top face
            3, 2, 6, 6, 7, 3,
            // Bottom face
            4, 5, 1, 1, 0, 4,
        ];

        for idx in box_indices {
            indices.push(base_idx + idx);
        }
    }

    /// Update uniforms for rendering
    pub fn update_uniforms(&self, queue: &Queue, _camera_yaw: f32, _camera_pitch: f32, aspect_ratio: f32, swing_progress: f32) {
        // Create a special projection matrix for viewmodel with closer near plane
        let fov = 45.0_f32.to_radians();
        let proj = Mat4::perspective_rh(fov, aspect_ratio, 0.01, 10.0);

        // IMPORTANT: Viewmodel uses IDENTITY view matrix
        // The viewmodel is positioned in view space (camera space)
        // This locks it perfectly to the camera - no rotation needed!
        let view = Mat4::IDENTITY;

        let view_proj = proj * view;

        // Model matrix with swing animation
        // Minecraft-style swing: smooth arc motion forward and down
        let model = if swing_progress > 0.0 {
            // Use smooth easing function (ease-out quad) for natural feel
            let t = swing_progress;
            let eased = 1.0 - (1.0 - t) * (1.0 - t);

            // Swing arc: goes forward and rotates down, then returns
            let swing_angle = if t < 0.5 {
                // First half: swing forward and down (increased rotation)
                eased * -90.0_f32.to_radians()
            } else {
                // Second half: swing back up
                (1.0 - eased) * -90.0_f32.to_radians()
            };

            // Translation: pull forward more during swing (much more visible)
            let forward_offset = if t < 0.5 {
                eased * 0.4  // Increased from 0.15 to 0.4
            } else {
                (1.0 - eased) * 0.4
            };

            // Downward movement increased for more dramatic swing
            let down_offset = 0.15 * eased;  // Increased from 0.05 to 0.15

            // Apply rotation around X axis (pitch down) and translate forward (Z) and down (Y)
            Mat4::from_translation(Vec3::new(0.0, -down_offset, -forward_offset))
                * Mat4::from_rotation_x(swing_angle)
        } else {
            Mat4::IDENTITY
        };

        let uniforms = ViewModelUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: model.to_cols_array_2d(),
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Render the viewmodel
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
