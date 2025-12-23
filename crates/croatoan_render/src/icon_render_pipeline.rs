//! Icon Render Pipeline - Renders 3D models to textures for inventory icons
//!
//! Creates offscreen render targets and renders GLB models with rotation animation.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Quat};
use wgpu::util::DeviceExt;
use std::collections::HashMap;

/// Size of rendered icons (square)
pub const ICON_SIZE: u32 = 64;

/// Vertex format for icon models
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct IconVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Uniforms for icon rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct IconUniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    light_dir: [f32; 4],
}

/// GPU data for an icon model
pub struct IconModelGpu {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub texture_bind_group: wgpu::BindGroup,
    /// Model center offset for rotation
    pub center_offset: Vec3,
    /// Scale to fit in view
    pub scale: f32,
}

/// Rendered icon ready for display (raw RGBA pixels)
pub struct RenderedIcon {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Pipeline for rendering 3D icons
pub struct IconRenderPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    default_texture: wgpu::Texture,
    default_texture_view: wgpu::TextureView,
    default_texture_bind_group: wgpu::BindGroup,
    /// Offscreen render target
    render_texture: wgpu::Texture,
    render_texture_view: wgpu::TextureView,
    /// Depth buffer for offscreen rendering
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    /// Buffer for reading back pixels
    readback_buffer: wgpu::Buffer,
    /// Loaded icon models
    models: HashMap<String, IconModelGpu>,
}

impl IconRenderPipeline {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        // Shader for icon rendering
        let shader_source = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    light_dir: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_pos;
    out.uv = in.uv;
    out.normal = (uniforms.model * vec4<f32>(in.normal, 0.0)).xyz;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);

    // Simple diffuse lighting
    let normal = normalize(in.normal);
    let light_dir = normalize(uniforms.light_dir.xyz);
    let ndotl = max(dot(normal, light_dir), 0.0);
    let ambient = 0.4;
    let diffuse = ndotl * 0.6;

    let lit_color = tex_color.rgb * (ambient + diffuse);

    // Discard transparent pixels
    if tex_color.a < 0.1 {
        discard;
    }

    return vec4<f32>(lit_color, tex_color.a);
}
"#;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Icon Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Uniform bind group layout
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Icon Uniform Layout"),
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

        // Texture bind group layout
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Icon Texture Layout"),
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

        // Uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Icon Uniform Buffer"),
            contents: bytemuck::cast_slice(&[IconUniforms {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                model: Mat4::IDENTITY.to_cols_array_2d(),
                light_dir: [0.5, 0.8, 0.3, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Icon Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Icon Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Default white texture
        let default_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Icon Default Texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
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
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let default_texture_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let default_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Icon Default Texture Bind Group"),
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

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Icon Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline - renders to RGBA8 for readback
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Icon Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<IconVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
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
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Offscreen render texture
        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Icon Render Texture"),
            size: wgpu::Extent3d {
                width: ICON_SIZE,
                height: ICON_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let render_texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Depth texture for offscreen rendering
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Icon Depth Texture"),
            size: wgpu::Extent3d {
                width: ICON_SIZE,
                height: ICON_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Readback buffer - must be aligned to 256 bytes per row
        let bytes_per_row = (ICON_SIZE * 4 + 255) & !255; // Align to 256
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Icon Readback Buffer"),
            size: (bytes_per_row * ICON_SIZE) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
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
            render_texture,
            render_texture_view,
            depth_texture,
            depth_texture_view,
            readback_buffer,
            models: HashMap::new(),
        }
    }

    /// Upload a model for icon rendering
    pub fn upload_model(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        template_id: &str,
        vertices: &[IconVertex],
        indices: &[u32],
        texture_data: Option<(&[u8], u32, u32)>,
        scale: f32,
    ) {
        if vertices.is_empty() || indices.is_empty() {
            log::warn!("[IconPipeline] Skipping empty model: {}", template_id);
            return;
        }

        // Calculate bounding box for centering
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in vertices {
            min = min.min(Vec3::from(v.position));
            max = max.max(Vec3::from(v.position));
        }
        let center_offset = (min + max) * 0.5;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Icon Vertex Buffer: {}", template_id)),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Icon Index Buffer: {}", template_id)),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create texture bind group
        let texture_bind_group = if let Some((data, width, height)) = texture_data {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Icon Texture: {}", template_id)),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            );
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Icon Texture Bind Group: {}", template_id)),
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
            })
        } else {
            // Create a new bind group using the default texture view
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Icon Default Texture Bind Group: {}", template_id)),
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.default_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            })
        };

        self.models.insert(template_id.to_string(), IconModelGpu {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            texture_bind_group,
            center_offset,
            scale,
        });

        log::info!("[IconPipeline] Uploaded model: {} ({} verts, {} indices)",
            template_id, vertices.len(), indices.len());
    }

    /// Check if a model is loaded
    pub fn has_model(&self, template_id: &str) -> bool {
        self.models.contains_key(template_id)
    }

    /// Render an icon and return the pixels
    pub fn render_icon(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        template_id: &str,
        rotation: f32,
    ) -> Option<RenderedIcon> {
        let model = self.models.get(template_id)?;

        // Setup view-projection for icon (orthographic, looking at model)
        let aspect = 1.0; // Square
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 2.0),  // Eye position
            Vec3::ZERO,                  // Look at center
            Vec3::Y,                     // Up
        );
        let proj = Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0);
        let view_proj = proj * view;

        // Model matrix with rotation
        let rotation_quat = Quat::from_rotation_y(rotation);
        let model_mat = Mat4::from_scale_rotation_translation(
            Vec3::splat(model.scale),
            rotation_quat,
            -model.center_offset * model.scale,
        );

        // Update uniforms
        let uniforms = IconUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: model_mat.to_cols_array_2d(),
            light_dir: [0.5, 0.8, 0.3, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Icon Render Encoder"),
        });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Icon Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0, // Transparent background
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &model.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
            render_pass.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..model.index_count, 0, 0..1);
        }

        // Copy to readback buffer
        let bytes_per_row = (ICON_SIZE * 4 + 255) & !255;
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.readback_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(ICON_SIZE),
                },
            },
            wgpu::Extent3d {
                width: ICON_SIZE,
                height: ICON_SIZE,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Map and read buffer
        let buffer_slice = self.readback_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait);

        let data = buffer_slice.get_mapped_range();

        // Copy data, removing row padding
        let mut pixels = Vec::with_capacity((ICON_SIZE * ICON_SIZE * 4) as usize);
        for y in 0..ICON_SIZE {
            let row_start = (y * bytes_per_row) as usize;
            let row_end = row_start + (ICON_SIZE * 4) as usize;
            pixels.extend_from_slice(&data[row_start..row_end]);
        }

        drop(data);
        self.readback_buffer.unmap();

        Some(RenderedIcon {
            pixels,
            width: ICON_SIZE,
            height: ICON_SIZE,
        })
    }

    /// Get list of loaded model IDs
    pub fn loaded_models(&self) -> Vec<&str> {
        self.models.keys().map(|s| s.as_str()).collect()
    }
}
