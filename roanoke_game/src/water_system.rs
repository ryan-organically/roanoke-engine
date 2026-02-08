use wgpu;
use wgpu::util::DeviceExt;
use glam::Mat4;
use bytemuck::{Pod, Zeroable};

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
    // World offset of the water mesh
    pub world_offset_x: f32,
    pub world_offset_z: f32,
    pub shoreline_x: f32,  // X coordinate where shoreline is
    pub _padding: f32,
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
    pub turbidity: f32,            // 0 = crystal clear, 1 = murky
    pub max_transparency_depth: f32, // depth at which water becomes fully opaque
}

/// Water biome types for color variation
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub enum WaterBiomeType {
    #[default]
    Ocean = 0,
    Tropical = 1,
    SaltMarsh = 2,
    River = 3,
    Lake = 4,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct WaterBiomeData {
    pub biome_type: u32,
    pub _pad1: [u32; 3],      // 12 bytes to align _padding to 16
    pub _padding: [u32; 3],   // vec3<u32> in WGSL (12 bytes)
    pub _pad2: u32,           // 4 bytes to round struct to 32
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
    biome_buffer: wgpu::Buffer,
    time_buffer: wgpu::Buffer,

    // Textures / Buffers
    h0_texture: wgpu::Texture,
    hkt_buffer: wgpu::Buffer, // Storage buffer for H(k,t)

    displacement_texture: wgpu::Texture,
    normal_texture: wgpu::Texture,
    shore_distance_texture: wgpu::Texture, // R = shore distance, G = water depth

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    uniforms: WaterUniforms,
    biome_data: WaterBiomeData,
    grid_size: u32,
}

impl WaterSystem {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let grid_size = 256;
        let patch_size = 512.0; // Meters - large enough to cover visible ocean

        // 1. Create Buffers & Textures

        // Ocean mesh positioning - starts AT shoreline and extends out to sea
        // Shoreline is around x=200-210, ocean is positive X direction
        let shoreline_x = 200.0;    // Where land meets water
        let ocean_center_x = shoreline_x + (patch_size / 2.0); // Center mesh so edge is at shoreline
        let ocean_center_z = 0.0;

        // Uniforms - visible waves that blend into base layer
        let uniforms = WaterUniforms {
            time: 0.0,
            delta_time: 0.0,
            wind_direction: [-1.0, 0.0], // West (towards shore)
            wind_speed: 6.0,
            amplitude: 0.8,   // Lower waves that blend smoothly
            choppiness: 0.6,  // Gentle horizontal displacement
            size: patch_size,
            world_offset_x: ocean_center_x,
            world_offset_z: ocean_center_z,
            shoreline_x,
            _padding: 0.0,
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
            deep_color: [0.0, 0.0, 0.0, 0.0], // Use 0 alpha to let shader use built-in colors
            shallow_color: [0.0, 0.0, 0.0, 0.0],
            foam_color: [0.95, 0.97, 1.0, 1.0],  // Slightly blue-white foam
            smoothness: 0.98,           // Very shiny
            metallic: 0.0,
            turbidity: 0.02,            // Crystal clear
            max_transparency_depth: 15.0, // Deep visibility
        };
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Material Buffer"),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let biome_data = WaterBiomeData {
            biome_type: WaterBiomeType::Ocean as u32,
            _pad1: [0; 3],
            _padding: [0; 3],
            _pad2: 0,
        };
        let biome_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Biome Buffer"),
            contents: bytemuck::cast_slice(&[biome_data]),
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

        // Shore distance texture (R = distance to shore, G = water depth)
        // Using Rgba16Float because Rg32Float is not filterable on most GPUs
        let shore_distance_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shore Distance Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float, // R = shore dist, G = depth (filterable)
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Initialize shore distance with gradient (placeholder - real data from terrain)
        // Creates a simple gradient: shore at x=0, deep water at x=1
        // Using half-precision floats (f16) packed as u16 for Rgba16Float

        // Simple f32 to f16 conversion (IEEE 754 half-precision)
        fn f32_to_f16(value: f32) -> u16 {
            let bits = value.to_bits();
            let sign = (bits >> 16) & 0x8000;
            let exp = ((bits >> 23) & 0xFF) as i32;
            let mant = bits & 0x7FFFFF;

            if exp == 255 {
                // Inf or NaN
                return (sign | 0x7C00 | ((mant >> 13) as u32)) as u16;
            }

            let new_exp = exp - 127 + 15;
            if new_exp >= 31 {
                // Overflow to infinity
                return (sign | 0x7C00) as u16;
            }
            if new_exp <= 0 {
                // Underflow to zero (or denormal, but we'll just use zero)
                return sign as u16;
            }

            (sign | ((new_exp as u32) << 10) | (mant >> 13)) as u16
        }

        let mut shore_data_f16: Vec<u16> = Vec::with_capacity((grid_size * grid_size * 4) as usize);
        for y in 0..grid_size {
            for x in 0..grid_size {
                let u = x as f32 / grid_size as f32;
                let v = y as f32 / grid_size as f32;
                // Shore distance: 0 at edge, increases toward center
                let shore_dist = (u.min(1.0 - u).min(v).min(1.0 - v)) * 2.0;
                // Water depth: shallow at edges, deeper toward center
                let depth = shore_dist * 0.5;

                shore_data_f16.push(f32_to_f16(shore_dist));
                shore_data_f16.push(f32_to_f16(depth));
                shore_data_f16.push(f32_to_f16(0.0)); // B unused
                shore_data_f16.push(f32_to_f16(0.0)); // A unused
            }
        }

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
        
        // Ocean mesh covers ocean area (centered at ocean_center_x, ocean_center_z)
        // Water level slightly submerged so shallow water "cuts into" beach sand
        let water_base_y = -0.4;  // Submerged below beach level

        for y in 0..grid_size {
            for x in 0..grid_size {
                let u = x as f32 / grid_size as f32;
                let v = y as f32 / grid_size as f32;
                // Position is flat plane at water_base_y, waves add height on top
                let px = (u - 0.5) * patch_size + ocean_center_x;
                let pz = (v - 0.5) * patch_size + ocean_center_z;

                vertices.push(px);
                vertices.push(water_base_y); // Below beach so waves cut into sand
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
                // Shore Distance Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Shore Distance Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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

        // Create filtering sampler for shore distance
        let shore_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shore Distance Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
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
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&shore_distance_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&shore_sampler),
                },
            ],
        });

        // 4. Render Pipeline
        let render_shader = device.create_shader_module(wgpu::include_wgsl!("../../assets/shaders/water.wgsl"));

        let render_bind_group_layout_0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Water Render Bind Group Layout 0 (Camera + Time)"),
            entries: &[
                // Camera
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
                // Time (for foam animation in fragment shader)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                // Material Uniform (binding 0)
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
                // Biome Data Uniform (binding 1)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Displacement Texture (binding 2) - used in both vertex and fragment
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Displacement Sampler (binding 3) - used in both vertex and fragment
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Normal Texture (binding 4)
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Normal Sampler (binding 5)
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Shore Distance Texture (binding 6)
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Shore Distance Sampler (binding 7)
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
                depth_write_enabled: false, // Transparent water must NOT write depth to allow see-through
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

        // Time buffer for fragment shader foam animation
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct TimeUniform {
            time: f32,
            _pad1: [f32; 3],      // 12 bytes to align _padding to 16
            _padding: [f32; 3],   // vec3<f32> in WGSL (12 bytes)
            _pad2: f32,           // 4 bytes to round struct to 32
        }
        let time_uniform = TimeUniform { time: 0.0, _pad1: [0.0; 3], _padding: [0.0; 3], _pad2: 0.0 };
        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Time Buffer"),
            contents: bytemuck::cast_slice(&[time_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let render_bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Water Render Bind Group 0"),
            layout: &render_bind_group_layout_0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: time_buffer.as_entire_binding(),
                },
            ],
        });

        // Create filtering sampler for shore texture in render pass
        let shore_render_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shore Render Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
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
                    resource: biome_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&displacement_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&shore_distance_texture.create_view(&Default::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&shore_render_sampler),
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
            biome_buffer,
            time_buffer,
            h0_texture,
            hkt_buffer,
            displacement_texture,
            normal_texture,
            shore_distance_texture,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            uniforms,
            biome_data,
            grid_size,
        }
    }

    /// Update the water biome type (changes color palette)
    pub fn set_biome(&mut self, queue: &wgpu::Queue, biome: WaterBiomeType) {
        self.biome_data.biome_type = biome as u32;
        queue.write_buffer(&self.biome_buffer, 0, bytemuck::cast_slice(&[self.biome_data]));
    }

    /// Update shore distance data from terrain system
    /// Data should be grid_size x grid_size pixels, each pixel is [shore_dist, depth] as f32
    /// This will convert to f16 internally for the Rgba16Float texture
    pub fn update_shore_data(&self, queue: &wgpu::Queue, data: &[f32]) {
        // Convert f32 pairs to Rgba16Float (4 x f16 per pixel)
        fn f32_to_f16(value: f32) -> u16 {
            let bits = value.to_bits();
            let sign = (bits >> 16) & 0x8000;
            let exp = ((bits >> 23) & 0xFF) as i32;
            let mant = bits & 0x7FFFFF;
            if exp == 255 { return (sign | 0x7C00 | ((mant >> 13) as u32)) as u16; }
            let new_exp = exp - 127 + 15;
            if new_exp >= 31 { return (sign | 0x7C00) as u16; }
            if new_exp <= 0 { return sign as u16; }
            (sign | ((new_exp as u32) << 10) | (mant >> 13)) as u16
        }

        let pixel_count = (self.grid_size * self.grid_size) as usize;
        let mut f16_data: Vec<u16> = Vec::with_capacity(pixel_count * 4);

        for i in 0..pixel_count {
            let shore_dist = data.get(i * 2).copied().unwrap_or(0.0);
            let depth = data.get(i * 2 + 1).copied().unwrap_or(0.0);
            f16_data.push(f32_to_f16(shore_dist));
            f16_data.push(f32_to_f16(depth));
            f16_data.push(0); // B
            f16_data.push(0); // A
        }

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.shore_distance_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&f16_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.grid_size * 8), // 4 x f16 = 8 bytes per pixel
                rows_per_image: Some(self.grid_size),
            },
            wgpu::Extent3d {
                width: self.grid_size,
                height: self.grid_size,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Update wind parameters from weather system
    pub fn set_wind(&mut self, wind_direction_rad: f32, wind_strength: f32) {
        // Convert wind direction (radians, 0=from north) to 2D vector
        let dx = wind_direction_rad.sin();
        let dz = -wind_direction_rad.cos();
        self.uniforms.wind_direction = [dx, dz];
        // Map weather wind_strength (0.3-2.5) to water wind_speed (3-12 m/s)
        self.uniforms.wind_speed = 3.0 + wind_strength * 3.6;
        // Also scale amplitude with wind (calm=0.5, storm=1.5)
        self.uniforms.amplitude = 0.4 + wind_strength * 0.4;
        self.uniforms.choppiness = 0.3 + wind_strength * 0.25;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, time: f32, delta_time: f32, sun_dir: [f32; 3]) {
        self.uniforms.time = time;
        self.uniforms.delta_time = delta_time;
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));

        // Update time + light buffer for fragment shader (32 bytes: time + padding + sun_dir + padding)
        let time_data: [f32; 8] = [time, 0.0, 0.0, 0.0, sun_dir[0], sun_dir[1], sun_dir[2], 0.0];
        queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&time_data));
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
