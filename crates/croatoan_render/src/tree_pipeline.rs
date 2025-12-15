use wgpu::{Device, Queue, RenderPipeline, Buffer, BindGroupLayout, BindGroup};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use std::sync::Arc;

use crate::pipeline_validation::{
    MeshValidator, PipelineError, PipelineResult,
    log_pipeline_error, sanitize_vec3, sanitize_float,
};

/// Maximum vertices per tree mesh (safety limit)
const MAX_TREE_VERTICES: usize = 100_000;
/// Maximum indices per tree mesh (safety limit)
const MAX_TREE_INDICES: usize = 300_000;
/// Maximum tree instances per frame (safety limit)
const MAX_TREE_INSTANCES: usize = 10_000;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TreeVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],       // 64 bytes (0-64)
    light_view_proj: [[f32; 4]; 4], // 64 bytes (64-128) - shadow map transform
    sun_dir: [f32; 3],              // 12 bytes (128-140)
    time: f32,                      // 4 bytes (140-144)
    view_pos: [f32; 3],             // 12 bytes (144-156)
    fog_density: f32,               // 4 bytes (156-160)
    fog_color: [f32; 3],            // 12 bytes (160-172)
    fog_start: f32,                 // 4 bytes (172-176)
    fog_end: f32,                   // 4 bytes (176-180)
    alpha_cutoff: f32,              // 4 bytes (180-184)
    use_texture: f32,               // 4 bytes (184-188)
    // LOD dither fade parameters
    lod_fade_start: f32,            // 4 bytes (188-192) - distance where fade begins
    lod_fade_end: f32,              // 4 bytes (192-196) - distance where fade ends
    lod_fade_mode: f32,             // 4 bytes (196-200) - 0=disabled, 1=LOD0 fade out, 2=LOD1 fade in
    _padding: f32,                  // 4 bytes (200-204)
    _padding2: f32,                 // 4 bytes (204-208) - 16-byte struct alignment
}

/// Configuration for tree LOD distance bands
#[derive(Clone, Copy, Debug)]
pub struct TreeLODConfig {
    /// Distance at which LOD0 starts fading out (full detail)
    pub lod0_fade_start: f32,
    /// Distance at which LOD0 is fully faded (switch to LOD1)
    pub lod0_fade_end: f32,
    /// Maximum distance for LOD1 rendering
    pub lod1_max_distance: f32,
}

impl Default for TreeLODConfig {
    fn default() -> Self {
        Self {
            lod0_fade_start: 400.0,
            lod0_fade_end: 500.0,
            lod1_max_distance: 1200.0,
        }
    }
}

/// LOD fade mode for shader
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LODFadeMode {
    /// No dither fade (fully visible)
    Disabled,
    /// LOD0: fade OUT as distance increases
    LOD0FadeOut,
    /// LOD1: fade IN as distance increases
    LOD1FadeIn,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TreeInstance {
    model_matrix: [[f32; 4]; 4],
}

#[derive(Clone)]
pub struct TreeMesh {
    pub vertex_buffer: Arc<Buffer>,
    pub index_buffer: Arc<Buffer>,
    pub index_count: u32,
    pub texture_bind_group: Option<Arc<BindGroup>>, // Added for textures
}

pub struct TreePipeline {
    pipeline: RenderPipeline,
    mesh: Option<TreeMesh>,
    instance_buffer: Option<Buffer>,
    instance_count: u32,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    camera_bind_group_layout: BindGroupLayout,
    // We store the texture layout here so we can create bind groups later if needed
    pub texture_bind_group_layout: BindGroupLayout,
    default_bind_group: BindGroup,
}



impl TreePipeline {
    pub fn new(device: &Device, queue: &Queue, surface_format: wgpu::TextureFormat, shadow_map: &crate::shadows::ShadowMap) -> Self {
        // Group 0: Camera + Shadow Map
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tree Camera Bind Group Layout"),
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

        // Group 1: Texture
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tree Texture Bind Group Layout"),
            entries: &[
                // Diffuse Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create default white texture
        let default_texture_size = wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 };
        let default_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default White Texture"),
            size: default_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        // Write white pixel
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &default_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            default_texture_size,
        );

        let default_texture_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let default_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&default_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&default_sampler),
                },
            ],
            label: Some("Default Texture Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Tree Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tree Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/tree.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tree Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex Buffer Layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<TreeVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // Position
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // Normal
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // UV
                            wgpu::VertexAttribute {
                                offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    // Instance Buffer Layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<TreeInstance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Model Matrix (4x vec4)
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                                shader_location: 7,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                                shader_location: 8,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                ],
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
                cull_mode: None,
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
            label: Some("Tree Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group with shadow map
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tree Camera Bind Group"),
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
            mesh: None,
            instance_buffer: None,
            instance_count: 0,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            texture_bind_group_layout,
            default_bind_group,
        }
    }

    /// Create the shared tree mesh buffers with validation
    ///
    /// # Errors
    /// Returns `PipelineError` if mesh data is invalid
    pub fn try_create_mesh(
        device: &Device,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
        texture_bind_group: Option<Arc<BindGroup>>,
    ) -> PipelineResult<TreeMesh> {
        // Validate mesh data before GPU allocation
        let validator = MeshValidator::new(MAX_TREE_VERTICES, MAX_TREE_INDICES);
        validator.validate_model(positions, normals, uvs, indices)?;

        Ok(Self::create_mesh_unchecked(device, positions, normals, uvs, indices, texture_bind_group))
    }

    /// Create the shared tree mesh buffers (panics on invalid data)
    pub fn create_mesh(
        device: &Device,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
        texture_bind_group: Option<Arc<BindGroup>>,
    ) -> TreeMesh {
        match Self::try_create_mesh(device, positions, normals, uvs, indices, texture_bind_group) {
            Ok(mesh) => mesh,
            Err(e) => {
                log_pipeline_error("TreePipeline", &e);
                panic!("Failed to create tree mesh: {}", e);
            }
        }
    }

    /// Create mesh without validation (internal use)
    fn create_mesh_unchecked(
        device: &Device,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u32],
        texture_bind_group: Option<Arc<BindGroup>>,
    ) -> TreeMesh {
        let vertex_count = positions.len().min(MAX_TREE_VERTICES);
        let index_count = indices.len().min(MAX_TREE_INDICES);

        // Interleave vertex data with NaN/Inf sanitization
        let vertices: Vec<TreeVertex> = (0..vertex_count)
            .map(|i| TreeVertex {
                position: sanitize_vec3(positions[i]),
                normal: sanitize_vec3(normals[i]),
                uv: [sanitize_float(uvs[i][0]), sanitize_float(uvs[i][1])],
            })
            .collect();

        // Create vertex buffer
        let vertex_buffer = Arc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        // Create index buffer
        let index_buffer = Arc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Index Buffer"),
            contents: bytemuck::cast_slice(&indices[..index_count]),
            usage: wgpu::BufferUsages::INDEX,
        }));

        log::debug!("Created tree mesh: {} vertices, {} triangles", vertices.len(), index_count / 3);

        TreeMesh {
            vertex_buffer,
            index_buffer,
            index_count: index_count as u32,
            texture_bind_group,
        }
    }

    /// Set the shared mesh for this pipeline
    pub fn set_mesh(&mut self, mesh: TreeMesh) {
        self.mesh = Some(mesh);
    }

    /// Check if this pipeline's mesh has a custom texture (vs default/procedural)
    pub fn has_texture(&self) -> bool {
        self.mesh.as_ref()
            .and_then(|m| m.texture_bind_group.as_ref())
            .is_some()
    }

    /// Upload instances for a chunk with validation
    ///
    /// # Errors
    /// Returns `PipelineError` if instance count exceeds limit
    pub fn try_upload_instances(
        &mut self,
        device: &Device,
        instances: &[Mat4],
    ) -> PipelineResult<()> {
        if instances.len() > MAX_TREE_INSTANCES {
            return Err(PipelineError::TooManyInstances {
                count: instances.len(),
                max: MAX_TREE_INSTANCES,
            });
        }
        self.upload_instances_unchecked(device, instances);
        Ok(())
    }

    /// Upload instances for a chunk (clamps if too many)
    pub fn upload_instances(
        &mut self,
        device: &Device,
        instances: &[Mat4],
    ) {
        if instances.is_empty() {
            self.instance_count = 0;
            self.instance_buffer = None;
            return;
        }

        // Safety check: clamp instance count
        if instances.len() > MAX_TREE_INSTANCES {
            log::warn!("Too many tree instances ({} requested), clamping to {}", instances.len(), MAX_TREE_INSTANCES);
        }

        self.upload_instances_unchecked(device, &instances[..instances.len().min(MAX_TREE_INSTANCES)]);
    }

    /// Upload instances without validation (internal use)
    fn upload_instances_unchecked(
        &mut self,
        device: &Device,
        instances: &[Mat4],
    ) {
        if instances.is_empty() {
            self.instance_count = 0;
            self.instance_buffer = None;
            return;
        }

        self.instance_count = instances.len() as u32;

        // Convert to instance data with matrix sanitization
        let instance_data: Vec<TreeInstance> = instances.iter()
            .map(|m| {
                let cols = m.to_cols_array_2d();
                // Sanitize each matrix component
                TreeInstance {
                    model_matrix: [
                        [sanitize_float(cols[0][0]), sanitize_float(cols[0][1]), sanitize_float(cols[0][2]), sanitize_float(cols[0][3])],
                        [sanitize_float(cols[1][0]), sanitize_float(cols[1][1]), sanitize_float(cols[1][2]), sanitize_float(cols[1][3])],
                        [sanitize_float(cols[2][0]), sanitize_float(cols[2][1]), sanitize_float(cols[2][2]), sanitize_float(cols[2][3])],
                        [sanitize_float(cols[3][0]), sanitize_float(cols[3][1]), sanitize_float(cols[3][2]), sanitize_float(cols[3][3])],
                    ],
                }
            })
            .collect();

        self.instance_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        }));
    }

    /// Update camera uniform with fog parameters
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
        // Default: procedural rendering, no alpha cutoff, no LOD fade
        self.update_camera_full(queue, view_proj, light_view_proj, sun_dir, time, view_pos, fog_color, fog_start, fog_end, fog_density, 0.0, 0.0);
    }

    /// Update camera uniform with all parameters including texture/alpha settings
    pub fn update_camera_full(
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
        alpha_cutoff: f32,
        use_texture: f32,
    ) {
        // No LOD fade by default
        self.update_camera_with_lod(queue, view_proj, light_view_proj, sun_dir, time, view_pos, fog_color, fog_start, fog_end, fog_density, alpha_cutoff, use_texture, LODFadeMode::Disabled, 0.0, 0.0);
    }

    /// Update camera uniform with LOD fade parameters for distance-based LOD transitions
    ///
    /// # Arguments
    /// * `lod_mode` - LOD fade mode (Disabled, LOD0FadeOut, or LOD1FadeIn)
    /// * `fade_start` - Distance where fade begins
    /// * `fade_end` - Distance where fade completes
    pub fn update_camera_with_lod(
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
        alpha_cutoff: f32,
        use_texture: f32,
        lod_mode: LODFadeMode,
        fade_start: f32,
        fade_end: f32,
    ) {
        let lod_fade_mode = match lod_mode {
            LODFadeMode::Disabled => 0.0,
            LODFadeMode::LOD0FadeOut => 1.0,
            LODFadeMode::LOD1FadeIn => 2.0,
        };

        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            light_view_proj: light_view_proj.to_cols_array_2d(),
            sun_dir,
            time,
            view_pos,
            fog_density,
            fog_color,
            fog_start,
            fog_end,
            alpha_cutoff,
            use_texture,
            lod_fade_start: fade_start,
            lod_fade_end: fade_end,
            lod_fade_mode,
            _padding: 0.0,
            _padding2: 0.0,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render the trees
    ///
    /// # Safety
    /// This method uses defensive checks to avoid panics even with invalid state.
    pub fn render<'rpass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        // Early exit if no instances to render
        if self.instance_count == 0 {
            return;
        }

        // Defensive: require mesh and instance buffer
        let (mesh, instance_buffer) = match (&self.mesh, &self.instance_buffer) {
            (Some(m), Some(ib)) => (m, ib),
            _ => {
                log::trace!("Tree render skipped: missing mesh or instance buffer");
                return;
            }
        };

        // Validate mesh has triangles
        if mesh.index_count == 0 {
            log::trace!("Tree render skipped: mesh has no indices");
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        // Use texture bind group if available, otherwise use default
        let texture_bind_group: &BindGroup = mesh.texture_bind_group
            .as_ref()
            .map_or(&self.default_bind_group, |arc| arc.as_ref());
        render_pass.set_bind_group(1, texture_bind_group, &[]);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.index_count, 0, 0..self.instance_count);
    }

    /// Check if the pipeline has valid data ready for rendering
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.mesh.is_some()
            && self.instance_buffer.is_some()
            && self.instance_count > 0
            && self.mesh.as_ref().map(|m| m.index_count > 0).unwrap_or(false)
    }

    /// Create a texture bind group from a texture view
    /// Use this for loading external textures (e.g., from GLTF models)
    pub fn create_texture_bind_group(
        &self,
        device: &Device,
        texture_view: &wgpu::TextureView,
        label: Option<&str>,
    ) -> BindGroup {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label,
        })
    }

    /// Get the current instance count
    #[inline]
    pub fn instance_count(&self) -> u32 {
        self.instance_count
    }

    /// Get buffers for shadow rendering (vertex, index, instance, counts)
    /// Returns None if pipeline is not ready
    pub fn get_shadow_buffers(&self) -> Option<(&Buffer, &Buffer, &Buffer, u32, u32)> {
        let mesh = self.mesh.as_ref()?;
        let instance_buffer = self.instance_buffer.as_ref()?;
        if self.instance_count == 0 || mesh.index_count == 0 {
            return None;
        }
        Some((
            mesh.vertex_buffer.as_ref(),
            mesh.index_buffer.as_ref(),
            instance_buffer,
            mesh.index_count,
            self.instance_count,
        ))
    }
}
