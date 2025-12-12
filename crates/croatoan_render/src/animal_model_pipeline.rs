//! Animal model rendering pipeline
//!
//! Renders actual 3D animal models loaded from GLTF files.
//! Supports instanced rendering for multiple animals of the same species.
//! Now with texture support for proper animal appearance.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPipeline, Sampler, Texture, TextureView};

use crate::pipeline_validation::{sanitize_float, sanitize_vec3};

/// Maximum animal instances per species per frame
const MAX_INSTANCES_PER_SPECIES: usize = 500;

/// Maximum joints supported for skeletal animation
pub const MAX_JOINTS: usize = 64;

/// Vertex data for animal models with skeletal skinning support
/// All animals use this format - non-skinned models have default joint/weight values
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct AnimalVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// Joint indices (up to 4 influencing joints)
    pub joints: [u32; 4],
    /// Joint weights (should sum to 1.0)
    pub weights: [f32; 4],
}

impl AnimalVertex {
    /// Create a non-skinned vertex (default joint weights)
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
            joints: [0, 0, 0, 0],
            weights: [0.0, 0.0, 0.0, 0.0], // No skinning - shader will skip
        }
    }

    /// Create a skinned vertex with joint weights
    pub fn skinned(
        position: [f32; 3],
        normal: [f32; 3],
        uv: [f32; 2],
        joints: [u32; 4],
        weights: [f32; 4],
    ) -> Self {
        Self {
            position,
            normal,
            uv,
            joints,
            weights,
        }
    }
}

/// Instance data for a single animal
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct AnimalInstance {
    /// Model matrix (position, rotation, scale)
    pub model_matrix: [[f32; 4]; 4],
    /// RGB tint color
    pub color: [f32; 3],
    /// Emissive intensity (for damage flash, etc.)
    pub emissive: f32,
}

impl AnimalInstance {
    pub fn new(transform: Mat4, color: [f32; 3], emissive: f32) -> Self {
        Self {
            model_matrix: transform.to_cols_array_2d(),
            color,
            emissive,
        }
    }
}

/// Camera uniform data
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    light_view_proj: [[f32; 4]; 4],  // Shadow mapping
    camera_pos: [f32; 3],
    time: f32,
    light_dir: [f32; 3],
    ambient_dimming: f32,
    fog_color: [f32; 3],
    fog_start: f32,
    fog_end: f32,
    fog_density: f32,
    shadow_strength: f32,
    rain_wetness: f32,
}

/// Skeleton data stored for GPU animation
#[derive(Clone)]
pub struct SkeletonGpu {
    /// Joint inverse bind matrices (pre-multiplied transforms)
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
    /// Joint parent indices (None for root joints)
    pub parents: Vec<Option<usize>>,
    /// Joint local transforms (rest pose)
    pub local_transforms: Vec<([[f32; 4]; 4], [f32; 4], [f32; 3])>, // (matrix, rotation quat, scale)
    /// Root joint indices
    pub roots: Vec<usize>,
}

/// Animation clip stored for GPU animation
#[derive(Clone)]
pub struct AnimationGpu {
    pub name: String,
    pub duration: f32,
    /// Per-joint keyframes: (times, translations, rotations, scales)
    pub joint_keyframes: Vec<JointKeyframes>,
}

/// Keyframes for a single joint
#[derive(Clone, Default)]
pub struct JointKeyframes {
    pub translation_times: Vec<f32>,
    pub translations: Vec<[f32; 3]>,
    pub rotation_times: Vec<f32>,
    pub rotations: Vec<[f32; 4]>,
    pub scale_times: Vec<f32>,
    pub scales: Vec<[f32; 3]>,
}

/// GPU mesh data for a single animal species
pub struct AnimalMeshGpu {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub vertex_count: u32,
    /// Optional texture for this species
    pub texture: Option<Texture>,
    pub texture_view: Option<TextureView>,
    pub texture_bind_group: Option<BindGroup>,
    /// Whether this species has a texture
    pub has_texture: bool,
    /// Whether this mesh has skinning data
    pub is_skinned: bool,
    /// Skeleton for this species (if animated)
    pub skeleton: Option<SkeletonGpu>,
    /// Animation clips for this species
    pub animations: Vec<AnimationGpu>,
    /// Joint matrix buffer for GPU skinning
    pub joint_buffer: Option<Buffer>,
    /// Joint bind group
    pub joint_bind_group: Option<BindGroup>,
}

/// Pipeline for rendering animal models
pub struct AnimalModelPipeline {
    pipeline: RenderPipeline,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    /// Texture bind group layout (for per-species textures)
    texture_bind_group_layout: BindGroupLayout,
    /// Shadow bind group layout
    shadow_bind_group_layout: BindGroupLayout,
    /// Shadow bind group (created when shadow map is bound)
    shadow_bind_group: Option<BindGroup>,
    /// Joint matrices bind group layout (for skeletal animation)
    joint_bind_group_layout: BindGroupLayout,
    /// Default joint matrices buffer (identity matrices for non-animated models)
    default_joint_buffer: Buffer,
    /// Default joint bind group
    default_joint_bind_group: BindGroup,
    /// Shared texture sampler
    sampler: Sampler,
    /// Default white texture for untextured models
    default_texture: Texture,
    default_texture_view: TextureView,
    default_texture_bind_group: BindGroup,
    /// Mesh data per species (keyed by species name)
    species_meshes: HashMap<String, AnimalMeshGpu>,
    /// Instance buffer per species
    instance_buffers: HashMap<String, (Buffer, u32)>,
}

impl AnimalModelPipeline {
    /// Create a new animal model pipeline
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        Self::new_with_queue(device, None, surface_format)
    }

    /// Create a new animal model pipeline with queue for texture initialization
    pub fn new_with_queue(device: &Device, queue: Option<&Queue>, surface_format: wgpu::TextureFormat) -> Self {
        // Create bind group layout for camera uniforms
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Animal Model Camera Layout"),
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

        // Create bind group layout for textures (per-species)
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Animal Model Texture Layout"),
                entries: &[
                    // Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
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
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create bind group layout for shadow map
        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Animal Model Shadow Layout"),
                entries: &[
                    // Shadow texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Shadow sampler (comparison)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

        // Create bind group layout for joint matrices (skeletal animation)
        let joint_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Animal Model Joint Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create default joint buffer with identity matrices
        let identity_matrices: Vec<[[f32; 4]; 4]> = (0..MAX_JOINTS)
            .map(|_| Mat4::IDENTITY.to_cols_array_2d())
            .collect();
        let default_joint_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animal Default Joint Buffer"),
            contents: bytemuck::cast_slice(&identity_matrices),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create shared sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Animal Model Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create default white texture (1x1 white pixel)
        let default_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Animal Default Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Initialize white texture if queue is available
        if let Some(q) = queue {
            q.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &default_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &[255u8, 255, 255, 255], // RGBA white pixel
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }

        let default_texture_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create default texture bind group
        let default_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animal Default Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&default_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create default joint bind group
        let default_joint_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animal Default Joint Bind Group"),
            layout: &joint_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: default_joint_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout (camera, texture, shadow, joints)
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Animal Model Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
                &shadow_bind_group_layout,
                &joint_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Animal Model Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/animal_model.wgsl").into(),
            ),
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Animal Model Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex buffer layout (with skeletal skinning attributes)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<AnimalVertex>() as u64,
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
                                offset: 12,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // UV
                            wgpu::VertexAttribute {
                                offset: 24,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            // Joint indices (vec4<u32>)
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Uint32x4,
                            },
                            // Joint weights (vec4<f32>)
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                    // Instance buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<AnimalInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Model matrix row 0
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Model matrix row 1
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Model matrix row 2
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 7,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Model matrix row 3
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 8,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Color
                            wgpu::VertexAttribute {
                                offset: 64,
                                shader_location: 9,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // Emissive
                            wgpu::VertexAttribute {
                                offset: 76,
                                shader_location: 10,
                                format: wgpu::VertexFormat::Float32,
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
                    blend: Some(wgpu::BlendState::REPLACE),
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
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Animal Model Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animal Model Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
            texture_bind_group_layout,
            shadow_bind_group_layout,
            shadow_bind_group: None,  // Created when shadow map is bound
            joint_bind_group_layout,
            default_joint_buffer,
            default_joint_bind_group,
            sampler,
            default_texture,
            default_texture_view,
            default_texture_bind_group,
            species_meshes: HashMap::new(),
            instance_buffers: HashMap::new(),
        }
    }

    /// Bind the shadow map resources - must be called before rendering
    pub fn bind_shadow_map(
        &mut self,
        device: &Device,
        shadow_view: &TextureView,
        shadow_sampler: &Sampler,
    ) {
        self.shadow_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animal Model Shadow Bind Group"),
            layout: &self.shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(shadow_sampler),
                },
            ],
        }));
    }

    /// Upload mesh data for a species (without texture)
    pub fn upload_species_mesh(
        &mut self,
        device: &Device,
        species_name: &str,
        vertices: &[AnimalVertex],
        indices: &[u32],
    ) {
        self.upload_species_mesh_with_texture(device, species_name, vertices, indices, None);
    }

    /// Upload mesh data for a species with optional texture
    pub fn upload_species_mesh_with_texture(
        &mut self,
        device: &Device,
        species_name: &str,
        vertices: &[AnimalVertex],
        indices: &[u32],
        texture_data: Option<&[u8]>,
    ) {
        self.upload_species_mesh_with_texture_dims(device, species_name, vertices, indices, texture_data, 1, 1);
    }

    /// Upload mesh data for a species with optional texture and dimensions
    pub fn upload_species_mesh_with_texture_dims(
        &mut self,
        device: &Device,
        species_name: &str,
        vertices: &[AnimalVertex],
        indices: &[u32],
        texture_data: Option<&[u8]>,
        texture_width: u32,
        texture_height: u32,
    ) {
        if vertices.is_empty() || indices.is_empty() {
            log::warn!("[AnimalModel] Skipping empty mesh for {}", species_name);
            return;
        }

        // Sanitize vertex data (preserve joints/weights for skinning)
        let sanitized_vertices: Vec<AnimalVertex> = vertices
            .iter()
            .map(|v| AnimalVertex {
                position: sanitize_vec3(v.position),
                normal: sanitize_vec3(v.normal),
                uv: [sanitize_float(v.uv[0]), sanitize_float(v.uv[1])],
                joints: v.joints,
                weights: [
                    sanitize_float(v.weights[0]),
                    sanitize_float(v.weights[1]),
                    sanitize_float(v.weights[2]),
                    sanitize_float(v.weights[3]),
                ],
            })
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Animal Vertex Buffer: {}", species_name)),
            contents: bytemuck::cast_slice(&sanitized_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Animal Index Buffer: {}", species_name)),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create texture if provided
        let (texture, texture_view, texture_bind_group, has_texture) = if let Some(data) = texture_data {
            if data.len() == (texture_width * texture_height * 4) as usize {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("Animal Texture: {}", species_name)),
                    size: wgpu::Extent3d {
                        width: texture_width,
                        height: texture_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });

                // Write texture data using queue (need to get queue somehow)
                // For now, create with data
                let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Animal Texture Bind Group: {}", species_name)),
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&tex_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                });

                log::info!(
                    "[AnimalModel] Uploaded mesh for '{}': {} vertices, {} indices, {}x{} texture",
                    species_name,
                    vertices.len(),
                    indices.len(),
                    texture_width,
                    texture_height
                );

                (Some(tex), Some(tex_view), Some(bind_group), true)
            } else {
                log::warn!(
                    "[AnimalModel] Texture data size mismatch for '{}': expected {}, got {}",
                    species_name,
                    texture_width * texture_height * 4,
                    data.len()
                );
                (None, None, None, false)
            }
        } else {
            log::info!(
                "[AnimalModel] Uploaded mesh for '{}': {} vertices, {} indices (no texture)",
                species_name,
                vertices.len(),
                indices.len()
            );
            (None, None, None, false)
        };

        self.species_meshes.insert(
            species_name.to_string(),
            AnimalMeshGpu {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
                vertex_count: vertices.len() as u32,
                texture,
                texture_view,
                texture_bind_group,
                has_texture,
                is_skinned: false,
                skeleton: None,
                animations: Vec::new(),
                joint_buffer: None,
                joint_bind_group: None,
            },
        );
    }

    /// Upload texture data for an existing species mesh (using queue)
    pub fn upload_species_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        species_name: &str,
        texture_data: &[u8],
        width: u32,
        height: u32,
    ) {
        if texture_data.len() != (width * height * 4) as usize {
            log::warn!(
                "[AnimalModel] Texture size mismatch for '{}': expected {}, got {}",
                species_name,
                width * height * 4,
                texture_data.len()
            );
            return;
        }

        // Create texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Animal Texture: {}", species_name)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload texture data
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Animal Texture Bind Group: {}", species_name)),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        // Update existing mesh if present
        if let Some(mesh) = self.species_meshes.get_mut(species_name) {
            mesh.texture = Some(texture);
            mesh.texture_view = Some(texture_view);
            mesh.texture_bind_group = Some(texture_bind_group);
            mesh.has_texture = true;
            log::info!(
                "[AnimalModel] Uploaded {}x{} texture for '{}'",
                width,
                height,
                species_name
            );
        } else {
            log::warn!(
                "[AnimalModel] Cannot upload texture for '{}': mesh not found",
                species_name
            );
        }
    }

    /// Upload skeleton and animation data for a species
    /// This stores the animation data for later sampling and creates GPU resources
    pub fn upload_species_animation(
        &mut self,
        device: &Device,
        species_name: &str,
        skeleton: SkeletonGpu,
        animations: Vec<AnimationGpu>,
    ) {
        if let Some(mesh) = self.species_meshes.get_mut(species_name) {
            let anim_names: Vec<&str> = animations.iter().map(|a| a.name.as_str()).collect();
            let joint_count = skeleton.inverse_bind_matrices.len();

            // Create joint buffer for this species
            let identity_matrices: Vec<[[f32; 4]; 4]> = (0..MAX_JOINTS)
                .map(|_| Mat4::IDENTITY.to_cols_array_2d())
                .collect();
            let joint_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Animal Joint Buffer: {}", species_name)),
                contents: bytemuck::cast_slice(&identity_matrices),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            // Create joint bind group for this species
            let joint_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Animal Joint Bind Group: {}", species_name)),
                layout: &self.joint_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: joint_buffer.as_entire_binding(),
                }],
            });

            log::info!(
                "[AnimalModel] Uploaded skeleton ({} joints) and {} animations for '{}': {:?}",
                joint_count,
                animations.len(),
                species_name,
                anim_names
            );

            mesh.skeleton = Some(skeleton);
            mesh.animations = animations;
            mesh.is_skinned = true;
            mesh.joint_buffer = Some(joint_buffer);
            mesh.joint_bind_group = Some(joint_bind_group);
        } else {
            log::warn!(
                "[AnimalModel] Cannot upload animation for '{}': mesh not found",
                species_name
            );
        }
    }

    /// Update joint matrices for a species (call every frame for animated species)
    pub fn update_joint_matrices(
        &self,
        queue: &Queue,
        species_name: &str,
        joint_matrices: &[[[f32; 4]; 4]],
    ) {
        if let Some(mesh) = self.species_meshes.get(species_name) {
            if let Some(joint_buffer) = &mesh.joint_buffer {
                // Pad to MAX_JOINTS if necessary
                let mut padded = vec![Mat4::IDENTITY.to_cols_array_2d(); MAX_JOINTS];
                for (i, mat) in joint_matrices.iter().enumerate().take(MAX_JOINTS) {
                    padded[i] = *mat;
                }
                queue.write_buffer(joint_buffer, 0, bytemuck::cast_slice(&padded));
            }
        }
    }

    /// Sample animation and compute joint matrices for a species
    /// Returns the computed joint matrices ready for GPU upload
    pub fn compute_animation_matrices(
        &self,
        species_name: &str,
        animation_name: &str,
        time: f32,
    ) -> Option<Vec<[[f32; 4]; 4]>> {
        let mesh = self.species_meshes.get(species_name)?;
        let skeleton = mesh.skeleton.as_ref()?;
        let animation = mesh.animations.iter().find(|a| a.name.eq_ignore_ascii_case(animation_name))?;

        let t = if animation.duration > 0.001 {
            time % animation.duration
        } else {
            0.0
        };

        let joint_count = skeleton.inverse_bind_matrices.len();
        let mut local_transforms: Vec<Mat4> = Vec::with_capacity(joint_count);
        let mut world_transforms: Vec<Mat4> = Vec::with_capacity(joint_count);

        // Sample animation for each joint
        for joint_idx in 0..joint_count {
            let keyframes = animation.joint_keyframes.get(joint_idx).cloned().unwrap_or_default();

            // Sample translation, rotation, scale
            let translation = sample_vec3_keyframes(&keyframes.translation_times, &keyframes.translations, t);
            let rotation = sample_quat_keyframes(&keyframes.rotation_times, &keyframes.rotations, t);
            let scale = if keyframes.scale_times.is_empty() {
                Vec3::ONE
            } else {
                sample_vec3_keyframes(&keyframes.scale_times, &keyframes.scales, t)
            };

            // Build local transform matrix
            let local = Mat4::from_scale_rotation_translation(scale, rotation, translation);
            local_transforms.push(local);
        }

        // Compute world transforms (parent chain)
        for joint_idx in 0..joint_count {
            let local = local_transforms[joint_idx];
            let world = if let Some(parent_idx) = skeleton.parents[joint_idx] {
                if parent_idx < world_transforms.len() {
                    world_transforms[parent_idx] * local
                } else {
                    local
                }
            } else {
                local
            };
            world_transforms.push(world);
        }

        // Compute final joint matrices (world * inverse_bind)
        let joint_matrices: Vec<[[f32; 4]; 4]> = world_transforms
            .iter()
            .enumerate()
            .map(|(i, world)| {
                let inv_bind = Mat4::from_cols_array_2d(&skeleton.inverse_bind_matrices[i]);
                (*world * inv_bind).to_cols_array_2d()
            })
            .collect();

        Some(joint_matrices)
    }

    /// Get animation names for a species
    pub fn get_animation_names(&self, species_name: &str) -> Vec<String> {
        self.species_meshes
            .get(species_name)
            .map(|mesh| mesh.animations.iter().map(|a| a.name.clone()).collect())
            .unwrap_or_default()
    }

    /// Check if a species has animations
    pub fn has_animations(&self, species_name: &str) -> bool {
        self.species_meshes
            .get(species_name)
            .map(|mesh| !mesh.animations.is_empty())
            .unwrap_or(false)
    }

    /// Sample an animation for a species and return root joint transform offset
    /// This is a simplified animation that returns a transform modifier based on the root bone
    /// For full skeletal animation, GPU skinning would be needed
    pub fn sample_animation_root_transform(
        &self,
        species_name: &str,
        animation_name: &str,
        time: f32,
    ) -> Option<(Vec3, glam::Quat)> {
        let mesh = self.species_meshes.get(species_name)?;
        let animation = mesh.animations.iter().find(|a| a.name.eq_ignore_ascii_case(animation_name))?;

        // Get the first root joint's keyframes (usually the main body bone)
        if animation.joint_keyframes.is_empty() {
            return None;
        }

        let t = if animation.duration > 0.001 {
            time % animation.duration
        } else {
            0.0
        };

        // Sample the root joint (index 0 is typically root/pelvis)
        let keyframes = &animation.joint_keyframes[0];

        // Sample translation
        let translation = sample_vec3_keyframes(&keyframes.translation_times, &keyframes.translations, t);

        // Sample rotation
        let rotation = sample_quat_keyframes(&keyframes.rotation_times, &keyframes.rotations, t);

        Some((translation, rotation))
    }

    /// Upload instance data for a species
    pub fn upload_instances(
        &mut self,
        device: &Device,
        species_name: &str,
        instances: &[AnimalInstance],
    ) {
        if instances.is_empty() {
            self.instance_buffers.remove(species_name);
            return;
        }

        // Clamp instance count
        let count = instances.len().min(MAX_INSTANCES_PER_SPECIES);
        if instances.len() > MAX_INSTANCES_PER_SPECIES {
            log::warn!(
                "[AnimalModel] Clamping {} instances to {} for {}",
                instances.len(),
                MAX_INSTANCES_PER_SPECIES,
                species_name
            );
        }

        // Sanitize instance data
        let sanitized: Vec<AnimalInstance> = instances[..count]
            .iter()
            .map(|inst| {
                let m = inst.model_matrix;
                AnimalInstance {
                    model_matrix: [
                        [sanitize_float(m[0][0]), sanitize_float(m[0][1]), sanitize_float(m[0][2]), sanitize_float(m[0][3])],
                        [sanitize_float(m[1][0]), sanitize_float(m[1][1]), sanitize_float(m[1][2]), sanitize_float(m[1][3])],
                        [sanitize_float(m[2][0]), sanitize_float(m[2][1]), sanitize_float(m[2][2]), sanitize_float(m[2][3])],
                        [sanitize_float(m[3][0]), sanitize_float(m[3][1]), sanitize_float(m[3][2]), sanitize_float(m[3][3])],
                    ],
                    color: sanitize_vec3(inst.color),
                    emissive: sanitize_float(inst.emissive),
                }
            })
            .collect();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Animal Instance Buffer: {}", species_name)),
            contents: bytemuck::cast_slice(&sanitized),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.instance_buffers
            .insert(species_name.to_string(), (buffer, count as u32));
    }

    /// Update camera uniforms
    pub fn update_camera(
        &self,
        queue: &Queue,
        view_proj: &Mat4,
        light_view_proj: &Mat4,
        camera_pos: Vec3,
        time: f32,
        light_dir: Vec3,
        fog_color: Vec3,
        fog_start: f32,
        fog_end: f32,
        fog_density: f32,
        ambient_dimming: f32,
        shadow_strength: f32,
        rain_wetness: f32,
    ) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            light_view_proj: light_view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            time,
            light_dir: light_dir.to_array(),
            ambient_dimming,
            fog_color: fog_color.to_array(),
            fog_start,
            fog_end,
            fog_density,
            shadow_strength,
            rain_wetness,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Render all animal models
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Skip rendering if shadow map not bound
        let shadow_bind_group = match &self.shadow_bind_group {
            Some(bg) => bg,
            None => {
                log::trace!("Animal model render skipped: shadow map not bound");
                return;
            }
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(2, shadow_bind_group, &[]);

        // Render each species that has both mesh and instances
        for (species_name, mesh) in &self.species_meshes {
            if let Some((instance_buffer, instance_count)) = self.instance_buffers.get(species_name)
            {
                if *instance_count == 0 {
                    continue;
                }

                // Bind texture (use species texture or default white)
                let texture_bind_group = mesh.texture_bind_group
                    .as_ref()
                    .unwrap_or(&self.default_texture_bind_group);
                render_pass.set_bind_group(1, texture_bind_group, &[]);

                // Bind joint matrices (use species joint buffer or default identity matrices)
                let joint_bind_group = mesh.joint_bind_group
                    .as_ref()
                    .unwrap_or(&self.default_joint_bind_group);
                render_pass.set_bind_group(3, joint_bind_group, &[]);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..*instance_count);
            }
        }
    }

    /// Check if a species has mesh data uploaded
    pub fn has_mesh(&self, species_name: &str) -> bool {
        self.species_meshes.contains_key(species_name)
    }

    /// Get the list of loaded species
    pub fn loaded_species(&self) -> Vec<&str> {
        self.species_meshes.keys().map(|s| s.as_str()).collect()
    }

    /// Get total instance count across all species
    pub fn total_instance_count(&self) -> u32 {
        self.instance_buffers.values().map(|(_, count)| count).sum()
    }
}

/// Sample Vec3 keyframes at a given time
fn sample_vec3_keyframes(times: &[f32], values: &[[f32; 3]], t: f32) -> Vec3 {
    if times.is_empty() || values.is_empty() {
        return Vec3::ZERO;
    }

    if times.len() == 1 || t <= times[0] {
        return Vec3::from_array(values[0]);
    }

    if t >= *times.last().unwrap() {
        return Vec3::from_array(*values.last().unwrap());
    }

    // Find the two keyframes to interpolate between
    let mut i = 0;
    while i < times.len() - 1 && times[i + 1] < t {
        i += 1;
    }

    let t0 = times[i];
    let t1 = times[i + 1];
    let factor = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };

    let v0 = Vec3::from_array(values[i]);
    let v1 = Vec3::from_array(values[i + 1]);

    v0.lerp(v1, factor)
}

/// Sample quaternion keyframes at a given time (spherical interpolation)
fn sample_quat_keyframes(times: &[f32], values: &[[f32; 4]], t: f32) -> glam::Quat {
    if times.is_empty() || values.is_empty() {
        return glam::Quat::IDENTITY;
    }

    if times.len() == 1 || t <= times[0] {
        let v = values[0];
        return glam::Quat::from_xyzw(v[0], v[1], v[2], v[3]);
    }

    if t >= *times.last().unwrap() {
        let v = values.last().unwrap();
        return glam::Quat::from_xyzw(v[0], v[1], v[2], v[3]);
    }

    // Find the two keyframes to interpolate between
    let mut i = 0;
    while i < times.len() - 1 && times[i + 1] < t {
        i += 1;
    }

    let t0 = times[i];
    let t1 = times[i + 1];
    let factor = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };

    let v0 = values[i];
    let v1 = values[i + 1];
    let q0 = glam::Quat::from_xyzw(v0[0], v0[1], v0[2], v0[3]);
    let q1 = glam::Quat::from_xyzw(v1[0], v1[1], v1[2], v1[3]);

    q0.slerp(q1, factor)
}
