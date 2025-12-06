use wgpu::{Device, Queue, RenderPipeline, Buffer, BindGroup};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use crate::pipeline_validation::{
    MeshValidator, PipelineResult,
    log_pipeline_error, sanitize_vec3,
};

/// Maximum vertices per grass mesh (safety limit - grass is highest vertex count)
const MAX_GRASS_VERTICES: usize = 500_000;
/// Maximum indices per grass mesh (safety limit)
const MAX_GRASS_INDICES: usize = 1_500_000;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GrassVertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],       // 64 bytes (0-64)
    light_view_proj: [[f32; 4]; 4], // 64 bytes (64-128)
    time: f32,                      // 4 bytes (128-132)
    _gap1: [f32; 3],                // 12 bytes (132-144) - align _padding1 to 16-byte
    _padding1: [f32; 3],            // 12 bytes (144-156)
    _gap2: f32,                     // 4 bytes (156-160) - align sun_dir to 16-byte
    sun_dir: [f32; 3],              // 12 bytes (160-172)
    fog_density: f32,               // 4 bytes (172-176)
    view_pos: [f32; 3],             // 12 bytes (176-188)
    fog_start: f32,                 // 4 bytes (188-192)
    fog_color: [f32; 3],            // 12 bytes (192-204)
    fog_end: f32,                   // 4 bytes (204-208)
}

pub struct GrassPipeline {
    pipeline: RenderPipeline,
    pub vertex_buffer: Option<Buffer>,
    pub index_buffer: Option<Buffer>,
    pub index_count: u32,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl GrassPipeline {
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat, shadow_map: &crate::shadows::ShadowMap) -> Self {
        // Camera bind group layout with shadow map
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grass Camera Bind Group Layout"),
            entries: &[
                // Uniforms
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
                // Shadow Map Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                // Shadow Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grass Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grass Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/grass.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grass Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GrassVertex>() as wgpu::BufferAddress,
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
                cull_mode: None, // No culling for grass (visible from both sides)
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
            label: Some("Grass Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group with shadow map
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grass Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_map.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_map.sampler),
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

    /// Upload grass mesh data to GPU with validation
    ///
    /// # Errors
    /// Returns `PipelineError` if mesh data is invalid
    pub fn try_upload_mesh(
        &mut self,
        device: &Device,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        indices: &[u32],
    ) -> PipelineResult<()> {
        // Validate mesh data before GPU allocation
        let validator = MeshValidator::new(MAX_GRASS_VERTICES, MAX_GRASS_INDICES);
        validator.validate_grass(positions, colors, indices)?;

        self.upload_mesh_unchecked(device, positions, colors, indices);
        Ok(())
    }

    /// Upload grass mesh data to GPU (panics on invalid data)
    pub fn upload_mesh(
        &mut self,
        device: &Device,
        _queue: &Queue,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        indices: &[u32],
    ) {
        match self.try_upload_mesh(device, positions, colors, indices) {
            Ok(()) => {}
            Err(e) => {
                log_pipeline_error("GrassPipeline", &e);
                // Don't panic for grass - just skip rendering
                log::warn!("Grass mesh upload failed, skipping grass rendering");
                self.vertex_buffer = None;
                self.index_buffer = None;
                self.index_count = 0;
            }
        }
    }

    /// Upload mesh without validation (internal use)
    fn upload_mesh_unchecked(
        &mut self,
        device: &Device,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        indices: &[u32],
    ) {
        let vertex_count = positions.len().min(MAX_GRASS_VERTICES);
        let index_count = indices.len().min(MAX_GRASS_INDICES);

        // Interleave positions and colors with NaN/Inf sanitization
        let vertices: Vec<GrassVertex> = positions[..vertex_count]
            .iter()
            .zip(colors[..vertex_count].iter())
            .map(|(pos, col)| GrassVertex {
                position: sanitize_vec3(*pos),
                color: sanitize_vec3(*col),
            })
            .collect();

        // Create vertex buffer
        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grass Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        // Create index buffer
        self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grass Index Buffer"),
            contents: bytemuck::cast_slice(&indices[..index_count]),
            usage: wgpu::BufferUsages::INDEX,
        }));

        self.index_count = index_count as u32;

        log::debug!("Uploaded grass mesh: {} vertices, {} triangles", vertices.len(), index_count / 3);
    }

    /// Update camera uniform with time for wind animation, shadow data, and fog
    pub fn update_camera(
        &self,
        queue: &Queue,
        view_proj: &Mat4,
        light_view_proj: &Mat4,
        sun_dir: [f32; 3],
        time: f32,
        view_pos: [f32; 3],
        fog_color: [f32; 3],
        fog_start: f32,
        fog_end: f32,
        fog_density: f32,
    ) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            light_view_proj: light_view_proj.to_cols_array_2d(),
            time,
            _gap1: [0.0; 3],
            _padding1: [0.0; 3],
            _gap2: 0.0,
            sun_dir,
            fog_density,
            view_pos,
            fog_start,
            fog_color,
            fog_end,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render the grass
    ///
    /// # Safety
    /// This method uses defensive checks to avoid panics even with invalid state.
    pub fn render<'rpass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        // Early exit if no triangles to render
        if self.index_count == 0 {
            return;
        }

        // Defensive: require both buffers to be present
        let (vertex_buffer, index_buffer) = match (&self.vertex_buffer, &self.index_buffer) {
            (Some(vb), Some(ib)) => (vb, ib),
            _ => {
                log::trace!("Grass render skipped: missing vertex or index buffer");
                return;
            }
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    /// Check if the pipeline has valid mesh data ready for rendering
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.vertex_buffer.is_some() && self.index_buffer.is_some() && self.index_count > 0
    }

    /// Get the current triangle count
    #[inline]
    pub fn triangle_count(&self) -> u32 {
        self.index_count / 3
    }
}
