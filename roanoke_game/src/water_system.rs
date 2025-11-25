use wgpu;
use wgpu::util::DeviceExt;
use glam::{Vec2, Vec3, Mat4, Vec4};
use bytemuck::{Pod, Zeroable};
use std::mem;

// --- Uniforms ---

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct WaterUniforms {
    pub time: f32,
    pub delta_time: f32,
    pub wind_direction: [f32; 2],
    pub wind_speed: f32,
    pub amplitude: f32,
    pub choppiness: f32,
    pub size: f32,
    pub _padding: [f32; 1], // Align to 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub position: [f32; 3],
    pub _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct WaterMaterial {
    pub deep_color: [f32; 4],
    pub shallow_color: [f32; 4],
    pub foam_color: [f32; 4],
    pub smoothness: f32,
    pub metallic: f32,
    pub _padding: [f32; 2],
}

// --- Water System ---

pub struct WaterSystem {
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    
    compute_bind_group: wgpu::BindGroup,
    render_bind_group_0: wgpu::BindGroup, // Camera
    render_bind_group_1: wgpu::BindGroup, // Material + Textures
    
    uniform_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    material_buffer: wgpu::Buffer,
    
    // Textures / Buffers
    h0_texture: wgpu::Texture,
    hkt_buffer: wgpu::Buffer, // Storage buffer for H(k,t)
    
    displacement_texture: wgpu::Texture,
    normal_texture: wgpu::Texture,
    
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    
    uniforms: WaterUniforms,
    grid_size: u32,
}

impl WaterSystem {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let grid_size = 256;
        let patch_size = 256.0; // Meters
        
        // 1. Create Buffers & Textures
        
        // Uniforms
        let uniforms = WaterUniforms {
            time: 0.0,
            delta_time: 0.0,
            wind_direction: [-1.0, 0.0], // West (towards shore)
            wind_speed: 5.0,
            amplitude: 0.2, // Gentle waves
            choppiness: 1.0,
            size: patch_size,
            _padding: [0.0],
        };
        
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_uniform = CameraUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            position: [0.0; 3],
            _padding: 0.0,
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let material_uniform = WaterMaterial {
            deep_color: [0.0, 0.1, 0.4, 1.0],
            shallow_color: [0.0, 0.4, 0.6, 1.0],
            foam_color: [1.0, 1.0, 1.0, 1.0],
            smoothness: 0.9,
            metallic: 0.0,
            _padding: [0.0; 2],
        };
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Material Buffer"),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Textures
        let texture_size = wgpu::Extent3d {
            width: grid_size,
            height: grid_size,
            depth_or_array_layers: 1,
        };

        // H0 (Initial Spectrum) - For now just empty/noise
        let h0_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("H0 Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, // Read only in compute
            view_formats: &[],
        });

        // Hkt Buffer (Intermediate)
        let hkt_buffer_size = (grid_size * grid_size) as u64 * 8; // vec2<f32>
        let hkt_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hkt Buffer"),
            size: hkt_buffer_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // Output Textures (Storage + Sampled)
        let displacement_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Displacement Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let normal_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Normal Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Butterfly Texture (Placeholder)
        let butterfly_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Butterfly Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // 2. Create Grid Mesh
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for y in 0..grid_size {
            for x in 0..grid_size {
                let u = x as f32 / grid_size as f32;
                let v = y as f32 / grid_size as f32;
                // Position is just flat plane, displaced in shader
                // Centered around 0,0
                let px = (u - 0.5) * patch_size;
                let pz = (v - 0.5) * patch_size;
                
                vertices.push(px);
                vertices.push(0.0);
                vertices.push(pz);
                
                vertices.push(u);
                vertices.push(v);
            }
        }
        
        for y in 0..grid_size - 1 {
            for x in 0..grid_size - 1 {
                let tl = y * grid_size + x;
                let tr = tl + 1;
                let bl = (y + 1) * grid_size + x;
                let br = bl + 1;
                
                indices.push(tl);
                indices.push(bl);
                indices.push(tr);
                
                indices.push(tr);
                indices.push(bl);
                indices.push(br);
            }
        }
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // 3. Compute Pipeline
        let compute_shader = device.create_shader_module(wgpu::include_wgsl!("../../assets/shaders/water_compute.wgsl"));
        
        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Water Compute Bind Group Layout"),
            entries: &[
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // H0 Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Hkt Buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Butterfly Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Output Displacement (Storage Texture)
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Output Normal (Storage Texture)
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Water Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Water Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "compute_displacement", // Using the simplified kernel for now
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Water Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&h0_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: hkt_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&butterfly_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&displacement_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.create_view(&Default::default())),
                },
            ],
        });

        // 4. Render Pipeline
        let render_shader = device.create_shader_module(wgpu::include_wgsl!("../../assets/shaders/water.wgsl"));

        let render_bind_group_layout_0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Water Render Bind Group Layout 0 (Camera)"),
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

        let render_bind_group_layout_1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Water Render Bind Group Layout 1 (Material)"),
            entries: &[
                // Material Uniform
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
                // Displacement Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Displacement Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Normal Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Normal Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Water Render Pipeline Layout"),
            bind_group_layouts: &[&render_bind_group_layout_0, &render_bind_group_layout_1],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Water Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 20, // 3 pos + 2 uv * 4 bytes
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 12,
                                shader_location: 1,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Don't cull water
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Water Render Bind Group 0"),
            layout: &render_bind_group_layout_0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });

        let render_bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Water Render Bind Group 1"),
            layout: &render_bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&displacement_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            compute_pipeline,
            render_pipeline,
            compute_bind_group,
            render_bind_group_0,
            render_bind_group_1,
            uniform_buffer,
            camera_buffer,
            material_buffer,
            h0_texture,
            hkt_buffer,
            displacement_texture,
            normal_texture,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            uniforms,
            grid_size: grid_size,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, time: f32, delta_time: f32) {
        self.uniforms.time = time;
        self.uniforms.delta_time = delta_time;
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }

    pub fn dispatch(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Water Compute Pass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.compute_bind_group, &[]);
        // Dispatch 16x16 workgroups of 16x16 threads = 256x256 threads
        cpass.dispatch_workgroups(self.grid_size / 16, self.grid_size / 16, 1);
    }

    pub fn render(&self, _encoder: &mut wgpu::CommandEncoder, _view: &wgpu::TextureView, _depth_view: &wgpu::TextureView, _camera_view_proj: [[f32; 4]; 4], _camera_pos: [f32; 3]) {
        // Update Camera Buffer (needs to be done before render pass, but we can't write to buffer inside render pass)
        // Ideally this is done in update(), but we need camera info.
        // For now, let's assume the user calls a separate update_camera() or we use a staging buffer.
        // Actually, we can use queue.write_buffer here if we have reference to queue, but we only have encoder.
        // So we'll assume the camera buffer is updated elsewhere or we add a method.
    }
    
    pub fn update_camera(&self, queue: &wgpu::Queue, view_proj: [[f32; 4]; 4], position: [f32; 3]) {
        let camera_uniform = CameraUniform {
            view_proj,
            position,
            _padding: 0.0,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }
    
    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.render_bind_group_0, &[]);
        rpass.set_bind_group(1, &self.render_bind_group_1, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
