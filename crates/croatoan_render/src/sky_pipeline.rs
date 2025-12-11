use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyUniforms {
    view_proj: [f32; 16],
    inv_view_proj: [f32; 16],
    sun_dir: [f32; 3],
    time: f32,
    sun_color: [f32; 3],
    cloud_coverage: f32,
    cloud_color_base: [f32; 3],
    cloud_density: f32,
    cloud_color_shade: [f32; 3],
    cloud_scale: f32,
    moon_dir: [f32; 3],
    rain_intensity: f32,  // 0-1 rain amount for stormy sky
    wind_offset: [f32; 2],
    ambient_dimming: f32, // 0-1 overall atmospheric dimming for moody look
    _pad1: f32,
}

pub struct SkyPipeline {
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl SkyPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../../assets/shaders/sky.wgsl"));

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sky Uniform Buffer"),
            contents: bytemuck::cast_slice(&[SkyUniforms {
                view_proj: Mat4::IDENTITY.to_cols_array(),
                inv_view_proj: Mat4::IDENTITY.to_cols_array(),
                sun_dir: [0.0, 1.0, 0.0],
                time: 0.0,
                sun_color: [1.0, 1.0, 1.0],
                cloud_coverage: 0.6, // Default to slightly overcast for moody feel
                cloud_color_base: [0.65, 0.68, 0.72], // Grey-blue clouds
                cloud_density: 0.7,
                cloud_color_shade: [0.45, 0.48, 0.52], // Darker grey undersides
                cloud_scale: 1.0,
                moon_dir: [0.0, -1.0, 0.0],
                rain_intensity: 0.0,
                wind_offset: [0.0, 0.0],
                ambient_dimming: 0.15, // Slight ambient dim for moody default
                _pad1: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sky Bind Group Layout"),
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
            label: Some("Sky Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sky Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // No vertex buffers, we generate full screen quad in shader
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            render_pipeline,
            uniform_buffer,
            bind_group,
        }
    }

    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        sun_dir: Vec3,
        moon_dir: Vec3,
        sun_color: Vec3,
        time: f32,
        cloud_coverage: f32,
        cloud_color_base: Vec3,
        cloud_density: f32,
        cloud_color_shade: Vec3,
        cloud_scale: f32,
        wind_offset: [f32; 2],
        rain_intensity: f32,
        ambient_dimming: f32,
    ) {
        let inv_view_proj = view_proj.inverse();
        let uniforms = SkyUniforms {
            view_proj: view_proj.to_cols_array(),
            inv_view_proj: inv_view_proj.to_cols_array(),
            sun_dir: sun_dir.to_array(),
            time,
            sun_color: sun_color.to_array(),
            cloud_coverage,
            cloud_color_base: cloud_color_base.to_array(),
            cloud_density,
            cloud_color_shade: cloud_color_shade.to_array(),
            cloud_scale,
            moon_dir: moon_dir.to_array(),
            rain_intensity,
            wind_offset,
            ambient_dimming,
            _pad1: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1); // Draw 3 vertices (full screen triangle)
    }
}
