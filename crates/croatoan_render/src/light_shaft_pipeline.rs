// Light Shaft (God Ray) Post-Process Pipeline
//
// AGENT: This pipeline renders volumetric light shafts as a post-process effect.
// It requires the scene to be rendered to a texture first, then applies radial blur
// from the sun's screen position.

use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightShaftUniforms {
    pub sun_screen_pos: [f32; 2],  // Sun position in screen space (0-1)
    pub intensity: f32,             // Overall intensity (0-1)
    pub decay: f32,                 // Ray decay (0.9-0.99)
    pub density: f32,               // Scattering density
    pub weight: f32,                // Sample weight
    pub exposure: f32,              // Exposure multiplier
    pub num_samples: i32,           // Sample count (32-128)
}

impl Default for LightShaftUniforms {
    fn default() -> Self {
        Self {
            sun_screen_pos: [0.5, 0.3],
            intensity: 0.5,
            decay: 0.96,
            density: 0.5,
            weight: 0.1,
            exposure: 1.0,
            num_samples: 24, // Reduced from 64 for FPS
        }
    }
}

pub struct LightShaftPipeline {
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl LightShaftPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Light Shaft Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/light_shafts.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Shaft Uniforms"),
            contents: bytemuck::cast_slice(&[LightShaftUniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Light Shaft Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Shaft Bind Group Layout"),
            entries: &[
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Scene texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Shaft Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Light Shaft Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_occlusion", // Use occlusion-based version
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
            bind_group_layout,
            sampler,
        }
    }

    /// Calculate sun's screen position from world direction and view-projection matrix
    pub fn calculate_sun_screen_pos(sun_dir: Vec3, view_proj: Mat4) -> Option<[f32; 2]> {
        // Sun direction points FROM sun, so negate for position far away
        let sun_world_pos = -sun_dir.normalize() * 1000.0;
        let sun_clip = view_proj * Vec4::new(sun_world_pos.x, sun_world_pos.y, sun_world_pos.z, 1.0);

        // Check if sun is in front of camera
        if sun_clip.w <= 0.0 {
            return None;
        }

        // Perspective divide
        let sun_ndc = sun_clip.truncate() / sun_clip.w;

        // Convert NDC (-1 to 1) to screen space (0 to 1)
        let screen_x = sun_ndc.x * 0.5 + 0.5;
        let screen_y = 1.0 - (sun_ndc.y * 0.5 + 0.5); // Flip Y

        // Check if on screen (with margin for off-screen rays)
        if screen_x < -0.5 || screen_x > 1.5 || screen_y < -0.5 || screen_y > 1.5 {
            return None;
        }

        Some([screen_x, screen_y])
    }

    /// Update uniforms for light shaft rendering
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        sun_screen_pos: [f32; 2],
        intensity: f32,
        decay: f32,
        density: f32,
    ) {
        let uniforms = LightShaftUniforms {
            sun_screen_pos,
            intensity,
            decay,
            density,
            weight: 0.15,
            exposure: 1.2,
            num_samples: 24, // Reduced from 64 for FPS
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Create a bind group for the scene texture
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        scene_texture: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Shaft Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// Render light shafts to the output
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, bind_group: &'a wgpu::BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
