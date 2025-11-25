use wgpu::util::DeviceExt;
use glam::{Vec3, Mat4};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SunUniforms {
    view_proj: [[f32; 4]; 4],
    sun_world_pos: [f32; 3],
    sun_size: f32,
    sun_color: [f32; 3],
    _padding: f32,
    camera_right: [f32; 3],
    _padding2: f32,
    camera_up: [f32; 3],
    _padding3: f32,
}

pub struct SunPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl SunPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sun Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/sun.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sun Uniform Buffer"),
            size: std::mem::size_of::<SunUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sun Bind Group Layout"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sun Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sun Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sun Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // No vertex buffer - generate in shader
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
                ..Default::default()
            },
            // No depth test - sun is always in background (rendered first)
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
        }
    }

    /// Update sun position and appearance
    /// sun_dir: direction FROM sun TO scene (normalized)
    /// camera_pos: viewer position
    /// time_of_day: 0-24 hours (affects color)
    pub fn update(&self, queue: &wgpu::Queue, view_proj: &Mat4, sun_dir: Vec3, camera_pos: Vec3, camera_right: Vec3, camera_up: Vec3, time_of_day: f32) {
        // Position sun far away in opposite direction of sun_dir
        // sun_dir points toward scene, so -sun_dir points toward sun
        let sun_distance = 800.0; // Far enough to be behind everything
        let sun_world_pos = camera_pos - sun_dir * sun_distance;

        // Sun size in world units (appears as ~30 degree disk)
        let sun_size = 40.0;

        // Sun color based on time of day
        let hour = time_of_day;
        let sun_color = if hour < 7.0 || hour > 18.0 {
            // Sunrise/sunset - orange-red
            [1.0, 0.6, 0.2]
        } else if hour < 9.0 || hour > 16.0 {
            // Morning/evening - warm yellow
            [1.0, 0.9, 0.6]
        } else {
            // Midday - bright white-yellow
            [1.0, 1.0, 0.9]
        };

        let uniforms = SunUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            sun_world_pos: sun_world_pos.to_array(),
            sun_size,
            sun_color,
            _padding: 0.0,
            camera_right: camera_right.to_array(),
            _padding2: 0.0,
            camera_up: camera_up.to_array(),
            _padding3: 0.0,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Render the sun billboard
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1); // 6 vertices for quad (2 triangles)
    }
}
