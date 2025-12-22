//! Weapon viewmodel pipeline for first-person weapon rendering
//!
//! Renders textured GLB weapon models in first-person view.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Quat};
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPipeline, Sampler, Texture, TextureView};

/// Vertex data for weapon models
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct WeaponVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl WeaponVertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self { position, normal, uv }
    }
}

/// Uniform data for weapon viewmodel - must match weapon_viewmodel.wgsl Uniforms struct
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct WeaponUniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    muzzle_flash: [f32; 4], // x = flash intensity (0-1), yzw = barrel tip position in model space
}

/// GPU mesh data for a weapon
pub struct WeaponMeshGpu {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub texture: Option<Texture>,
    pub texture_view: Option<TextureView>,
    pub texture_bind_group: Option<BindGroup>,
}

/// Pipeline for rendering first-person weapon viewmodels
pub struct WeaponViewModelPipeline {
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,
    sampler: Sampler,
    default_texture: Texture,
    default_texture_view: TextureView,
    default_texture_bind_group: BindGroup,
    /// Currently loaded weapon mesh
    weapon_mesh: Option<WeaponMeshGpu>,
    /// Position offset in view space
    pub position_offset: Vec3,
    /// Rotation offset (euler angles in radians)
    pub rotation_offset: Vec3,
    /// Scale factor
    pub scale: f32,
    /// Sway amount (horizontal, vertical)
    pub sway_amount: Vec3,
    /// Sway speed multiplier
    pub sway_speed: f32,
}

impl WeaponViewModelPipeline {
    /// Create a new weapon viewmodel pipeline
    pub fn new(device: &Device, queue: &Queue, surface_format: wgpu::TextureFormat) -> Self {
        // Create uniform bind group layout
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Weapon Uniform Layout"),
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

        // Create texture bind group layout
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Weapon Texture Layout"),
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Weapon Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create default white texture
        let default_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Weapon Default Texture"),
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

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &default_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255u8, 255, 255, 255],
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

        let default_texture_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let default_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Weapon Default Texture Bind Group"),
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

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Weapon Uniform Buffer"),
            size: std::mem::size_of::<WeaponUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Weapon Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Weapon Viewmodel Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/weapon_viewmodel.wgsl").into(),
            ),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Weapon Viewmodel Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Weapon Viewmodel Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<WeaponVertex>() as u64,
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
                    ],
                }],
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

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            sampler,
            default_texture,
            default_texture_view,
            default_texture_bind_group,
            weapon_mesh: None,
            // Default position: right side, lower, in front
            position_offset: Vec3::new(0.25, -0.2, -0.4),
            // Default rotation: slightly tilted
            rotation_offset: Vec3::new(0.0, -0.3, 0.0),
            scale: 0.15,
            // Sway settings
            sway_amount: Vec3::new(0.015, 0.01, 0.005),
            sway_speed: 1.5,
        }
    }

    /// Upload weapon mesh data
    pub fn upload_weapon_mesh(
        &mut self,
        device: &Device,
        queue: &Queue,
        vertices: &[WeaponVertex],
        indices: &[u32],
        texture_data: Option<(&[u8], u32, u32)>,
    ) {
        if vertices.is_empty() || indices.is_empty() {
            log::warn!("[WeaponViewmodel] Skipping empty weapon mesh");
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Weapon Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Weapon Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create texture if provided
        let (texture, texture_view, texture_bind_group) = if let Some((data, width, height)) = texture_data {
            if data.len() == (width * height * 4) as usize {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Weapon Texture"),
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

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
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

                let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Weapon Texture Bind Group"),
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                });

                log::info!("[WeaponViewmodel] Uploaded weapon mesh: {} vertices, {} indices, {}x{} texture",
                    vertices.len(), indices.len(), width, height);

                (Some(tex), Some(view), Some(bind_group))
            } else {
                log::warn!("[WeaponViewmodel] Texture size mismatch");
                (None, None, None)
            }
        } else {
            log::info!("[WeaponViewmodel] Uploaded weapon mesh: {} vertices, {} indices (no texture)",
                vertices.len(), indices.len());
            (None, None, None)
        };

        self.weapon_mesh = Some(WeaponMeshGpu {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            texture,
            texture_view,
            texture_bind_group,
        });
    }

    /// Check if a weapon is loaded
    pub fn has_weapon(&self) -> bool {
        self.weapon_mesh.is_some()
    }

    /// Update uniforms for rendering
    pub fn update_uniforms(&self, queue: &Queue, aspect_ratio: f32, recoil_progress: f32, time: f32, velocity: Vec3, muzzle_flash: f32) {
        // Viewmodel uses a separate projection with closer near plane
        let fov = 70.0_f32.to_radians();
        let proj = Mat4::perspective_rh(fov, aspect_ratio, 0.01, 10.0);

        // View is identity - viewmodel is positioned in camera space
        let view = Mat4::IDENTITY;
        let view_proj = proj * view;

        // Build model matrix from position, rotation, scale
        let mut position = self.position_offset;
        let mut rotation = Quat::from_euler(
            glam::EulerRot::YXZ,
            self.rotation_offset.y,
            self.rotation_offset.x,
            self.rotation_offset.z,
        );

        // Apply weapon sway based on movement
        let speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
        let sway_time = time * self.sway_speed;

        // Idle sway (subtle breathing motion)
        let idle_sway_x = (sway_time * 0.8).sin() * self.sway_amount.x * 0.3;
        let idle_sway_y = (sway_time * 1.2).cos() * self.sway_amount.y * 0.3;

        // Movement sway (bob when walking/running)
        let move_intensity = (speed * 0.15).min(1.0);
        let bob_x = (sway_time * 2.5).sin() * self.sway_amount.x * move_intensity;
        let bob_y = (sway_time * 5.0).sin().abs() * self.sway_amount.y * move_intensity * 1.5;
        let bob_z = (sway_time * 2.5).cos() * self.sway_amount.z * move_intensity;

        position.x += idle_sway_x + bob_x;
        position.y += idle_sway_y - bob_y; // Subtract for downward bob
        position.z += bob_z;

        // Slight rotation sway
        let rot_sway = (sway_time * 2.0).sin() * 0.02 * move_intensity;
        rotation = rotation * Quat::from_rotation_z(rot_sway);

        // Apply recoil animation (backward kick + barrel points to sky)
        if recoil_progress > 0.0 {
            let t = recoil_progress;

            // Quick snap back, slow recovery
            // First 15% is the snap, remaining 85% is recovery
            let recoil_amount = if t < 0.15 {
                // Snap phase - very quick upward kick
                let snap_t = t / 0.15;
                snap_t // Linear for snappy feel
            } else {
                // Recovery phase - smooth return
                let recover_t = (t - 0.15) / 0.85;
                1.0 - recover_t * recover_t // Ease out from max
            };

            // Backward recoil (push toward player)
            position.z += recoil_amount * 0.1;

            // Dramatic upward kick - barrel points toward sky (55 degrees)
            let kick_angle = recoil_amount * 55.0_f32.to_radians();
            rotation = rotation * Quat::from_rotation_x(-kick_angle);

            // Upward position shift as gun kicks back
            position.y += recoil_amount * 0.06;
        }

        let model = Mat4::from_scale_rotation_translation(
            Vec3::splat(self.scale),
            rotation,
            position,
        );

        // Barrel tip position in model space (for muzzle flash positioning)
        // This is approximate - adjust based on actual flintlock model
        let barrel_tip = Vec3::new(0.0, 0.05, -0.3);

        let uniforms = WeaponUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: model.to_cols_array_2d(),
            muzzle_flash: [muzzle_flash, barrel_tip.x, barrel_tip.y, barrel_tip.z],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Render the weapon viewmodel
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let mesh = match &self.weapon_mesh {
            Some(m) => m,
            None => return,
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        // Use weapon texture or default
        let texture_bind_group = mesh.texture_bind_group
            .as_ref()
            .unwrap_or(&self.default_texture_bind_group);
        render_pass.set_bind_group(1, texture_bind_group, &[]);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }
}
