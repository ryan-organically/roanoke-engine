//! Rain particle rendering pipeline
//!
//! Renders rain drops as instanced camera-facing stretched billboards.
//! Uses procedural positioning based on instance ID for efficient particle distribution.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Number of rain particles to render
const RAIN_PARTICLE_COUNT: u32 = 4000;

/// Rain uniform data
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct RainUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    time: f32,
    camera_right: [f32; 3],
    rain_intensity: f32,
    camera_up: [f32; 3],
    wind_strength: f32,
    fog_color: [f32; 3],
    fog_start: f32,
    fog_end: f32,
    _padding: [f32; 3],
}

/// Pipeline for rendering rain particles
pub struct RainPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    enabled: bool,
}

impl RainPipeline {
    /// Create a new rain pipeline
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rain Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/rain.wgsl").into(),
            ),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rain Uniform Buffer"),
            contents: bytemuck::cast_slice(&[RainUniforms {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                camera_pos: [0.0; 3],
                time: 0.0,
                camera_right: [1.0, 0.0, 0.0],
                rain_intensity: 0.0,
                camera_up: [0.0, 1.0, 0.0],
                wind_strength: 0.0,
                fog_color: [0.5, 0.52, 0.58],
                fog_start: 30.0,
                fog_end: 200.0,
                _padding: [0.0; 3],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rain Bind Group Layout"),
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

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rain Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rain Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline with alpha blending
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rain Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // No vertex buffers - procedural geometry
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Render both sides of rain drops
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Rain doesn't write to depth
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
            enabled: false,
        }
    }

    /// Update rain uniforms
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        view_proj: &Mat4,
        camera_pos: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
        time: f32,
        rain_intensity: f32,
        wind_strength: f32,
        fog_color: Vec3,
        fog_start: f32,
        fog_end: f32,
    ) {
        // Enable/disable based on intensity
        self.enabled = rain_intensity > 0.01;

        if !self.enabled {
            return;
        }

        let uniforms = RainUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            time,
            camera_right: camera_right.to_array(),
            rain_intensity,
            camera_up: camera_up.to_array(),
            wind_strength,
            fog_color: fog_color.to_array(),
            fog_start,
            fog_end,
            _padding: [0.0; 3],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Render rain particles
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if !self.enabled {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        // Draw 6 vertices per particle (2 triangles), instanced
        render_pass.draw(0..6, 0..RAIN_PARTICLE_COUNT);
    }

    /// Check if rain is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set rain intensity directly (0-1)
    pub fn set_intensity(&mut self, intensity: f32) {
        self.enabled = intensity > 0.01;
    }
}
