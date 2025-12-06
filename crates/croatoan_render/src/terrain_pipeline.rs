use wgpu::util::DeviceExt;
use glam::Mat4;

use crate::pipeline_validation::{
    MeshValidator, PipelineResult,
    log_pipeline_error, sanitize_vec3,
};

/// Maximum vertices per terrain chunk (safety limit)
const MAX_TERRAIN_VERTICES: usize = 100_000;
/// Maximum indices per terrain chunk (safety limit)
const MAX_TERRAIN_INDICES: usize = 600_000;

/// Uniform data structure matching WGSL layout
/// Must match the shader struct exactly!
#[repr(C)]
#[derive(Copy, Clone)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],       // 64 bytes (0-64)
    light_view_proj: [[f32; 4]; 4], // 64 bytes (64-128)
    fog_color: [f32; 3],            // 12 bytes (128-140)
    time: f32,                      // 4 bytes (140-144)
    fog_start: f32,                 // 4 bytes (144-148)
    fog_end: f32,                   // 4 bytes (148-152)
    fog_density: f32,               // 4 bytes (152-156)
    _padding1: f32,                 // 4 bytes (156-160)
    sun_dir: [f32; 3],              // 12 bytes (160-172)
    _padding2: f32,                 // 4 bytes (172-176)
    view_pos: [f32; 3],             // 12 bytes (176-188)
    _padding3: f32,                 // 4 bytes (188-192) -> Total 192 bytes
}

// SAFETY: Uniforms is repr(C) and contains only f32, which is Pod
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

/// Terrain rendering pipeline with vertex buffers
pub struct TerrainPipeline {
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pub index_count: u32,
    pub vertex_buffer: wgpu::Buffer, // Made public for shadow pass
    pub index_buffer: wgpu::Buffer,  // Made public for shadow pass
}

impl TerrainPipeline {
    /// Create a new terrain pipeline with validation
    ///
    /// # Errors
    /// Returns `PipelineError` if mesh data is invalid (mismatched arrays, out-of-bounds indices, etc.)
    pub fn try_new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        normals: &[[f32; 3]],
        indices: &[u32],
        shadow_map: &crate::shadows::ShadowMap,
    ) -> PipelineResult<Self> {
        // Validate mesh data before GPU allocation
        let validator = MeshValidator::new(MAX_TERRAIN_VERTICES, MAX_TERRAIN_INDICES);
        validator.validate_terrain(positions, colors, normals, indices)?;

        Ok(Self::new_unchecked(device, surface_format, positions, colors, normals, indices, shadow_map))
    }

    /// Create a new terrain pipeline (panics on invalid data)
    ///
    /// For production code, prefer `try_new()` which returns a Result.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        normals: &[[f32; 3]],
        indices: &[u32],
        shadow_map: &crate::shadows::ShadowMap,
    ) -> Self {
        match Self::try_new(device, surface_format, positions, colors, normals, indices, shadow_map) {
            Ok(pipeline) => pipeline,
            Err(e) => {
                log_pipeline_error("TerrainPipeline", &e);
                panic!("Failed to create terrain pipeline: {}", e);
            }
        }
    }

    /// Create pipeline without validation (internal use)
    fn new_unchecked(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        normals: &[[f32; 3]],
        indices: &[u32],
        shadow_map: &crate::shadows::ShadowMap,
    ) -> Self {
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/terrain.wgsl").into()),
        });

        // Create uniform buffer for view-projection matrix and time
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
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

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Terrain Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
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

        // Create vertex buffers
        let (vertex_buffer, index_buffer) = Self::create_buffers(device, positions, colors, normals, indices);
        let index_count = indices.len() as u32;

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terrain Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Define vertex buffer layout
        // Stride: 36 bytes (3 floats position + 3 floats color + 3 floats normal)
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: 36,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position (location 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color (location 1)
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal (location 2)
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        };

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling to debug visibility
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
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            index_count,
        }
    }

    /// Create vertex and index buffers
    ///
    /// Assumes data has been pre-validated by `try_new()`.
    /// Sanitizes NaN/Inf values to prevent GPU undefined behavior.
    fn create_buffers(
        device: &wgpu::Device,
        positions: &[[f32; 3]],
        colors: &[[f32; 3]],
        normals: &[[f32; 3]],
        indices: &[u32],
    ) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_count = positions.len().min(MAX_TERRAIN_VERTICES);
        let index_count = indices.len().min(MAX_TERRAIN_INDICES);

        // Interleave position, color, and normal data with NaN/Inf sanitization
        let mut vertex_data = Vec::with_capacity(vertex_count * 9);
        for i in 0..vertex_count {
            // Sanitize each component to prevent GPU undefined behavior
            vertex_data.extend_from_slice(&sanitize_vec3(positions[i]));
            vertex_data.extend_from_slice(&sanitize_vec3(colors[i]));
            vertex_data.extend_from_slice(&sanitize_vec3(normals[i]));
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Index Buffer"),
            contents: bytemuck::cast_slice(&indices[..index_count]),
            usage: wgpu::BufferUsages::INDEX,
        });

        log::debug!("Created terrain buffers: {} vertices, {} indices", vertex_count, index_count / 3);

        (vertex_buffer, index_buffer)
    }

    /// Update uniform buffer with camera, time, fog, and light matrix
    pub fn update_uniforms(&self, queue: &wgpu::Queue, view_proj: &Mat4, light_view_proj: &Mat4, time: f32, fog_color: [f32; 3], fog_start: f32, fog_end: f32, fog_density: f32, sun_dir: [f32; 3], view_pos: [f32; 3], _camera_pos: [f32; 3]) {
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            light_view_proj: light_view_proj.to_cols_array_2d(),
            fog_color,
            time,
            fog_start,
            fog_end,
            fog_density,
            _padding1: 0.0,
            sun_dir,
            _padding2: 0.0,
            view_pos,
            _padding3: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Render the terrain
    ///
    /// # Safety
    /// This method uses defensive checks to avoid panics even with invalid state.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Early exit if no triangles to render
        if self.index_count == 0 {
            log::trace!("Terrain render skipped: no indices");
            return;
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    /// Check if the pipeline has valid mesh data ready for rendering
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.index_count > 0
    }

    /// Get the current triangle count
    #[inline]
    pub fn triangle_count(&self) -> u32 {
        self.index_count / 3
    }
}
