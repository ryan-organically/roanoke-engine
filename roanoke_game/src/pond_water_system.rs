//! Pond and Lake Water Rendering System
//!
//! Renders calm water surfaces for inland water bodies (ponds, lakes, wetlands, marsh pools).
//! Uses a simplified version of the ocean water shader with reduced wave motion.

use wgpu;
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec2};
use bytemuck::{Pod, Zeroable};

// Import from croatoan_wfc
use croatoan_wfc::{generate_all_water_meshes, get_water_params, WaterBodyType, WaterMeshData};

// ============================================================================
// UNIFORMS
// ============================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PondCameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_position: [f32; 3],
    pub time: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PondMaterialUniform {
    pub deep_color: [f32; 4],
    pub shallow_color: [f32; 4],
    pub foam_color: [f32; 4],
    pub wave_amplitude: f32,
    pub wave_frequency: f32,
    pub turbidity: f32,
    pub transparency_depth: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PondInstanceUniform {
    pub center: [f32; 2],
    pub radius: f32,
    pub water_level: f32,
}

// ============================================================================
// SINGLE POND MESH
// ============================================================================

struct PondMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    center: Vec2,
    water_type: WaterBodyType,
    material_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

// ============================================================================
// POND WATER SYSTEM
// ============================================================================

pub struct PondWaterSystem {
    render_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    ponds: Vec<PondMesh>,
    time: f32,
}

impl PondWaterSystem {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, seed: u32) -> Self {
        // Generate all water body meshes
        let water_meshes = generate_all_water_meshes(seed);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pond Water Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/shaders/pond_water.wgsl").into()),
        });

        // Camera bind group layout (group 0)
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Pond Camera Bind Group Layout"),
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

        // Material bind group layout (group 1)
        let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Pond Material Bind Group Layout"),
            entries: &[
                // Material uniform
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
                // Instance uniform (center, radius)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pond Water Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &material_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pond Water Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    // Position + UV vertex layout
                    wgpu::VertexBufferLayout {
                        array_stride: 20, // 3 floats pos + 2 floats uv = 20 bytes
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
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Render both sides
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Transparent water must NOT write depth
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

        // Camera buffer
        let camera_uniform = PondCameraUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_position: [0.0, 0.0, 0.0],
            time: 0.0,
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pond Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pond Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });

        // Create pond meshes
        let mut ponds = Vec::with_capacity(water_meshes.len());

        for mesh_data in water_meshes {
            let pond_mesh = Self::create_pond_mesh(
                device,
                &material_bind_group_layout,
                &mesh_data,
            );
            ponds.push(pond_mesh);
        }

        Self {
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            ponds,
            time: 0.0,
        }
    }

    fn create_pond_mesh(
        device: &wgpu::Device,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_data: &WaterMeshData,
    ) -> PondMesh {
        // Interleave position and UV data
        let mut vertex_data: Vec<f32> = Vec::with_capacity(mesh_data.positions.len() * 5);
        for (pos, uv) in mesh_data.positions.iter().zip(mesh_data.uvs.iter()) {
            vertex_data.push(pos.x);
            vertex_data.push(pos.y);
            vertex_data.push(pos.z);
            vertex_data.push(uv.x);
            vertex_data.push(uv.y);
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pond Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pond Index Buffer"),
            contents: bytemuck::cast_slice(&mesh_data.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Get water parameters for this type
        let params = get_water_params(mesh_data.water_type);

        let material_uniform = PondMaterialUniform {
            deep_color: [params.deep_color[0], params.deep_color[1], params.deep_color[2], 1.0],
            shallow_color: [params.shallow_color[0], params.shallow_color[1], params.shallow_color[2], 1.0],
            foam_color: [params.foam_color[0], params.foam_color[1], params.foam_color[2], 1.0],
            wave_amplitude: params.wave_amplitude,
            wave_frequency: params.wave_frequency,
            turbidity: params.turbidity,
            transparency_depth: params.transparency_depth,
        };

        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pond Material Buffer"),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Calculate water level based on type
        let water_level = match mesh_data.water_type {
            WaterBodyType::Pond => -0.3,
            WaterBodyType::Lake => -0.5,
            WaterBodyType::Wetland => 0.2,
            WaterBodyType::MarshPool => 0.1,
        };

        // Calculate radius from mesh bounds
        let mut max_dist = 0.0f32;
        for pos in &mesh_data.positions {
            let dist = ((pos.x - mesh_data.center.x).powi(2) + (pos.z - mesh_data.center.y).powi(2)).sqrt();
            max_dist = max_dist.max(dist);
        }

        let instance_uniform = PondInstanceUniform {
            center: [mesh_data.center.x, mesh_data.center.y],
            radius: max_dist,
            water_level,
        };

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pond Instance Buffer"),
            contents: bytemuck::cast_slice(&[instance_uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pond Material Bind Group"),
            layout: material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_buffer.as_entire_binding(),
                },
            ],
        });

        PondMesh {
            vertex_buffer,
            index_buffer,
            num_indices: mesh_data.indices.len() as u32,
            center: mesh_data.center,
            water_type: mesh_data.water_type,
            material_buffer,
            instance_buffer,
            bind_group,
        }
    }

    /// Update time and camera
    pub fn update(&mut self, queue: &wgpu::Queue, view_proj: [[f32; 4]; 4], camera_pos: [f32; 3], delta_time: f32) {
        self.time += delta_time;

        let camera_uniform = PondCameraUniform {
            view_proj,
            camera_position: camera_pos,
            time: self.time,
        };

        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    /// Render all pond water surfaces
    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.camera_bind_group, &[]);

        for pond in &self.ponds {
            rpass.set_bind_group(1, &pond.bind_group, &[]);
            rpass.set_vertex_buffer(0, pond.vertex_buffer.slice(..));
            rpass.set_index_buffer(pond.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..pond.num_indices, 0, 0..1);
        }
    }

    /// Get number of water bodies being rendered
    pub fn pond_count(&self) -> usize {
        self.ponds.len()
    }

    /// Check if a position is near a pond (for gameplay)
    pub fn is_near_pond(&self, x: f32, z: f32, threshold: f32) -> bool {
        for pond in &self.ponds {
            let dist = ((x - pond.center.x).powi(2) + (z - pond.center.y).powi(2)).sqrt();
            if dist < threshold {
                return true;
            }
        }
        false
    }
}
