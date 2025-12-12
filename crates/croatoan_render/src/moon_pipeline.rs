use glam::{Vec3, Mat4};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MoonUniforms {
    view_proj: [[f32; 4]; 4],
    moon_world_pos: [f32; 3],
    moon_size: f32,
    moon_color: [f32; 3],
    phase: f32,
    camera_right: [f32; 3],
    moon_elevation: f32,  // For horizon shimmer effect
    camera_up: [f32; 3],
    time: f32,
}

pub struct MoonPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl MoonPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Moon Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/moon.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Moon Uniform Buffer"),
            size: std::mem::size_of::<MoonUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Moon Bind Group Layout"),
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
            label: Some("Moon Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Moon Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Moon Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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

    /// Update moon position and appearance
    /// moon_dir: direction FROM moon TO scene (normalized)
    /// camera_pos: viewer position
    /// phase: moon phase 0.0-1.0 (0/1 = new moon, 0.5 = full moon)
    /// elapsed_time: total elapsed time for animations
    pub fn update(
        &self,
        queue: &wgpu::Queue,
        view_proj: &Mat4,
        moon_dir: Vec3,
        camera_pos: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
        phase: f32,
        elapsed_time: f32,
    ) {
        // Position moon far away in opposite direction of moon_dir
        let moon_distance = 800.0;
        let moon_world_pos = camera_pos - moon_dir * moon_distance;

        // Moon size - smaller than sun, more subtle presence
        let moon_size = 25.0;

        // Moon elevation (y component of moon direction, negative = below horizon)
        let moon_elevation = -moon_dir.y;

        // Silvery moon color
        let moon_color = [0.9, 0.92, 0.98];

        let uniforms = MoonUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            moon_world_pos: moon_world_pos.to_array(),
            moon_size,
            moon_color,
            phase,
            camera_right: camera_right.to_array(),
            moon_elevation,
            camera_up: camera_up.to_array(),
            time: elapsed_time,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Render the moon billboard
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
// test
