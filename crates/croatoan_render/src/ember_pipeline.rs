//! Ember particle rendering pipeline
//!
//! Renders rising ember particles from campfires as camera-facing billboards.
//! Uses procedural positioning based on instance ID for efficient particle distribution.
//! Follows the pattern established by rain_pipeline.rs.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Maximum number of campfires to render embers for
pub const MAX_EMBER_CAMPFIRES: usize = 8;

/// Number of ember particles per campfire
const EMBERS_PER_CAMPFIRE: u32 = 200;

/// Total ember particle count
const TOTAL_EMBER_PARTICLES: u32 = EMBERS_PER_CAMPFIRE * MAX_EMBER_CAMPFIRES as u32;

/// Ember uniform data
/// NOTE: Must match WGSL std140 alignment rules exactly
/// WGSL vec3 needs 16-byte alignment, so _pad3 is aligned to 256, making total 272 bytes
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct EmberUniforms {
    view_proj: [[f32; 4]; 4],       // 64 bytes (offset 0)
    camera_pos: [f32; 3],           // 12 bytes (offset 64)
    time: f32,                      // 4 bytes (offset 76) -> 80
    camera_right: [f32; 3],         // 12 bytes (offset 80)
    _pad1: f32,                     // 4 bytes (offset 92) -> 96
    camera_up: [f32; 3],            // 12 bytes (offset 96)
    _pad2: f32,                     // 4 bytes (offset 108) -> 112
    // Campfire positions (xyz = position, w = intensity)
    campfire_data: [[f32; 4]; MAX_EMBER_CAMPFIRES],  // 128 bytes (offset 112) -> 240
    campfire_count: u32,            // 4 bytes (offset 240)
    _pad3a: f32,                    // 4 bytes (offset 244) - alignment padding
    _pad3b: f32,                    // 4 bytes (offset 248) - alignment padding
    _pad3c: f32,                    // 4 bytes (offset 252) - alignment padding -> 256 (16-byte aligned)
    _pad3: [f32; 4],                // 16 bytes (offset 256) -> Total 272 bytes
}

/// Pipeline for rendering ember particles
pub struct EmberPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    enabled: bool,
}

impl EmberPipeline {
    /// Create a new ember pipeline
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ember Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/ember.wgsl").into(),
            ),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ember Uniform Buffer"),
            contents: bytemuck::cast_slice(&[EmberUniforms {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                camera_pos: [0.0; 3],
                time: 0.0,
                camera_right: [1.0, 0.0, 0.0],
                _pad1: 0.0,
                camera_up: [0.0, 1.0, 0.0],
                _pad2: 0.0,
                campfire_data: [[0.0; 4]; MAX_EMBER_CAMPFIRES],
                campfire_count: 0,
                _pad3a: 0.0,
                _pad3b: 0.0,
                _pad3c: 0.0,
                _pad3: [0.0; 4],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ember Bind Group Layout"),
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
            label: Some("Ember Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ember Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline with additive blending for glowing embers
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Ember Pipeline"),
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
                    // Additive blending for glowing embers
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,  // Additive
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
                cull_mode: None, // Render both sides of particles
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Particles don't write to depth
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

    /// Update ember uniforms with campfire data
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        view_proj: &Mat4,
        camera_pos: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
        time: f32,
        campfire_lights: &[[f32; 4]],  // Up to 8 campfires, each is [x, y, z, intensity]
    ) {
        // Enable/disable based on campfire count
        self.enabled = !campfire_lights.is_empty();

        if !self.enabled {
            return;
        }

        // Pack campfire data into fixed array
        let mut campfire_data = [[0.0f32; 4]; MAX_EMBER_CAMPFIRES];
        let count = campfire_lights.len().min(MAX_EMBER_CAMPFIRES);
        for (i, light) in campfire_lights.iter().take(MAX_EMBER_CAMPFIRES).enumerate() {
            campfire_data[i] = *light;
        }

        let uniforms = EmberUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            time,
            camera_right: camera_right.to_array(),
            _pad1: 0.0,
            camera_up: camera_up.to_array(),
            _pad2: 0.0,
            campfire_data,
            campfire_count: count as u32,
            _pad3a: 0.0,
            _pad3b: 0.0,
            _pad3c: 0.0,
            _pad3: [0.0; 4],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Render ember particles
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if !self.enabled {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        // Draw 6 vertices per particle (2 triangles), instanced
        render_pass.draw(0..6, 0..TOTAL_EMBER_PARTICLES);
    }

    /// Check if embers are currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
