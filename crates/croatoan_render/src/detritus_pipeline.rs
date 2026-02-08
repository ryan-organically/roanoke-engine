use wgpu::{Device, Queue, RenderPipeline, Buffer, BindGroup, BindGroupLayout, util::DeviceExt};
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use std::sync::{Arc, OnceLock, Mutex};

use crate::pipeline_validation::{
    MeshValidator, PipelineResult,
    log_pipeline_error, sanitize_vec3, sanitize_float,
};

/// Maximum vertices per detritus mesh
const MAX_DETRITUS_VERTICES: usize = 1_000_000;
/// Maximum indices per detritus mesh
const MAX_DETRITUS_INDICES: usize = 3_000_000;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct DetritusVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],       // 64 bytes (0-64)
    sun_dir: [f32; 3],              // 12 bytes (64-76)
    fog_density: f32,               // 4 bytes (76-80)
    view_pos: [f32; 3],             // 12 bytes (80-92)
    fog_start: f32,                 // 4 bytes (92-96)
    fog_color: [f32; 3],            // 12 bytes (96-108)
    fog_end: f32,                   // 4 bytes (108-112) -> Total 112 bytes (aligned to 16)
}

/// GPU resources shared across ALL DetritusPipeline instances (created once, never changes)
struct SharedDetritusState {
    pipeline: RenderPipeline,
    camera_bind_group_layout: BindGroupLayout,
}

static SHARED_DETRITUS: OnceLock<Mutex<Option<Arc<SharedDetritusState>>>> = OnceLock::new();

pub struct DetritusPipeline {
    shared: Arc<SharedDetritusState>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    index_count: u32,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl DetritusPipeline {
    /// Get or create the shared pipeline state (shader, render pipeline, layout).
    /// Created once on first call; all subsequent DetritusPipelines share the same GPU objects.
    fn get_shared(device: &Device, surface_format: wgpu::TextureFormat) -> Arc<SharedDetritusState> {
        let mutex = SHARED_DETRITUS.get_or_init(|| Mutex::new(None));
        let mut guard = mutex.lock().unwrap();
        if let Some(shared) = guard.as_ref() {
            return Arc::clone(shared);
        }

        // First call — create all shared GPU resources
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Detritus Camera Bind Group Layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Detritus Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Detritus Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/detritus.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Detritus Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<DetritusVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2,
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

        let shared = Arc::new(SharedDetritusState {
            pipeline,
            camera_bind_group_layout,
        });
        *guard = Some(Arc::clone(&shared));
        shared
    }

    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        let shared = Self::get_shared(device, surface_format);

        // Per-pipeline: camera uniform buffer + bind group
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Detritus Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Detritus Camera Bind Group"),
            layout: &shared.camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            shared,
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
            camera_buffer,
            camera_bind_group,
        }
    }

    /// Upload detritus mesh data to GPU with validation
    ///
    /// # Errors
    /// Returns `PipelineError` if mesh data is invalid
    pub fn try_upload_mesh(
        &mut self,
        device: &Device,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
    ) -> PipelineResult<()> {
        let validator = MeshValidator::new(MAX_DETRITUS_VERTICES, MAX_DETRITUS_INDICES);
        validator.validate_model(positions, normals, uvs, indices)?;

        self.upload_mesh_unchecked(device, positions, normals, uvs, indices);
        Ok(())
    }

    /// Upload detritus mesh data to GPU (skips rendering on invalid data)
    pub fn upload_mesh(
        &mut self,
        device: &Device,
        _queue: &Queue,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
    ) {
        match self.try_upload_mesh(device, positions, normals, uvs, indices) {
            Ok(()) => {}
            Err(e) => {
                log_pipeline_error("DetritusPipeline", &e);
                // Don't panic - just skip rendering detritus
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
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
    ) {
        let vertex_count = positions.len().min(MAX_DETRITUS_VERTICES);
        let index_count = indices.len().min(MAX_DETRITUS_INDICES);

        // Interleave vertex data with NaN/Inf sanitization
        let vertices: Vec<DetritusVertex> = (0..vertex_count)
            .map(|i| DetritusVertex {
                position: sanitize_vec3(positions[i]),
                normal: sanitize_vec3(normals[i]),
                uv: [sanitize_float(uvs[i][0]), sanitize_float(uvs[i][1])],
            })
            .collect();

        // Create vertex buffer
        self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Detritus Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        // Create index buffer
        self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Detritus Index Buffer"),
            contents: bytemuck::cast_slice(&indices[..index_count]),
            usage: wgpu::BufferUsages::INDEX,
        }));

        self.index_count = index_count as u32;

        log::debug!("Uploaded detritus mesh: {} vertices, {} triangles", vertices.len(), index_count / 3);
    }

    /// Update camera uniform with fog parameters
    pub fn update_camera(
        &self,
        queue: &Queue,
        view_proj: &Mat4,
        sun_dir: [f32; 3],
        view_pos: [f32; 3],
        fog_color: [f32; 3],
        fog_start: f32,
        fog_end: f32,
        fog_density: f32,
    ) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            sun_dir,
            fog_density,
            view_pos,
            fog_start,
            fog_color,
            fog_end,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render the detritus
    ///
    /// # Safety
    /// This method uses defensive checks to avoid panics even with invalid state.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Early exit if no triangles to render
        if self.index_count == 0 {
            return;
        }

        // Defensive: require both buffers to be present
        let (vertex_buffer, index_buffer) = match (&self.vertex_buffer, &self.index_buffer) {
            (Some(vb), Some(ib)) => (vb, ib),
            _ => {
                log::trace!("Detritus render skipped: missing vertex or index buffer");
                return;
            }
        };

        render_pass.set_pipeline(&self.shared.pipeline);
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
