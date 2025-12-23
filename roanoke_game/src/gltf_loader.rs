//! GLTF model loader for animal and foliage models
//!
//! Loads GLTF files and extracts mesh data, skeleton, and animations for rendering.

use std::collections::HashMap;
use std::path::Path;

/// A single joint/bone in the skeleton
#[derive(Debug, Clone)]
pub struct Joint {
    /// Index of this joint
    pub index: usize,
    /// Name of the joint (from GLTF node name)
    pub name: String,
    /// Parent joint index (None for root)
    pub parent: Option<usize>,
    /// Children joint indices
    pub children: Vec<usize>,
    /// Local transform (translation, rotation, scale)
    pub local_translation: [f32; 3],
    pub local_rotation: [f32; 4], // quaternion [x, y, z, w]
    pub local_scale: [f32; 3],
}

/// Skeleton data for skinned meshes
#[derive(Debug, Clone)]
pub struct Skeleton {
    /// All joints in the skeleton
    pub joints: Vec<Joint>,
    /// Inverse bind matrices (one per joint) - transforms from mesh space to bone space
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
    /// Root joint indices
    pub roots: Vec<usize>,
}

impl Skeleton {
    /// Get the number of joints
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }
}

/// Interpolation method for animation keyframes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interpolation {
    Linear,
    Step,
    CubicSpline,
}

/// A single animation channel targeting a specific joint property
#[derive(Debug, Clone)]
pub struct AnimationChannel {
    /// Target joint index
    pub joint_index: usize,
    /// Property being animated
    pub property: AnimationProperty,
    /// Keyframe times (in seconds)
    pub times: Vec<f32>,
    /// Keyframe values (interpretation depends on property)
    pub values: Vec<[f32; 4]>, // Max 4 components (quaternion)
    /// Interpolation method
    pub interpolation: Interpolation,
}

/// Property being animated
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationProperty {
    Translation,
    Rotation,
    Scale,
}

/// A complete animation clip
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// Name of the animation (e.g., "Idle", "Walk", "Gallop")
    pub name: String,
    /// Duration in seconds
    pub duration: f32,
    /// Animation channels (one per joint/property combination)
    pub channels: Vec<AnimationChannel>,
}

impl AnimationClip {
    /// Sample the animation at a given time, returning joint local transforms
    /// Returns Vec of (translation, rotation, scale) for each joint that has animation
    pub fn sample(&self, time: f32, joint_count: usize) -> Vec<Option<([f32; 3], [f32; 4], [f32; 3])>> {
        let mut result = vec![None; joint_count];
        let t = time % self.duration.max(0.001); // Loop animation

        for channel in &self.channels {
            if channel.joint_index >= joint_count {
                continue;
            }

            let value = self.sample_channel(channel, t);

            let entry = result[channel.joint_index].get_or_insert((
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
            ));

            match channel.property {
                AnimationProperty::Translation => {
                    entry.0 = [value[0], value[1], value[2]];
                }
                AnimationProperty::Rotation => {
                    entry.1 = value;
                }
                AnimationProperty::Scale => {
                    entry.2 = [value[0], value[1], value[2]];
                }
            }
        }

        result
    }

    fn sample_channel(&self, channel: &AnimationChannel, t: f32) -> [f32; 4] {
        if channel.times.is_empty() || channel.values.is_empty() {
            return match channel.property {
                AnimationProperty::Translation | AnimationProperty::Scale => [0.0, 0.0, 0.0, 0.0],
                AnimationProperty::Rotation => [0.0, 0.0, 0.0, 1.0],
            };
        }

        // Find keyframe indices
        let mut i = 0;
        while i < channel.times.len() - 1 && channel.times[i + 1] < t {
            i += 1;
        }

        if i >= channel.times.len() - 1 {
            return channel.values[channel.values.len() - 1];
        }

        let t0 = channel.times[i];
        let t1 = channel.times[i + 1];
        let factor = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };

        match channel.interpolation {
            Interpolation::Step => channel.values[i],
            Interpolation::Linear => {
                if channel.property == AnimationProperty::Rotation {
                    // Spherical linear interpolation for quaternions
                    slerp(channel.values[i], channel.values[i + 1], factor)
                } else {
                    lerp4(channel.values[i], channel.values[i + 1], factor)
                }
            }
            Interpolation::CubicSpline => {
                // For now, fall back to linear
                if channel.property == AnimationProperty::Rotation {
                    slerp(channel.values[i], channel.values[i + 1], factor)
                } else {
                    lerp4(channel.values[i], channel.values[i + 1], factor)
                }
            }
        }
    }
}

/// Linear interpolation for vec4
fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Spherical linear interpolation for quaternions
fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];

    // If dot is negative, negate one quaternion to take shorter path
    let b = if dot < 0.0 {
        dot = -dot;
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };

    // If very close, use linear interpolation
    if dot > 0.9995 {
        let result = lerp4(a, b, t);
        // Normalize
        let len = (result[0] * result[0] + result[1] * result[1] +
                   result[2] * result[2] + result[3] * result[3]).sqrt();
        if len > 0.0001 {
            return [result[0] / len, result[1] / len, result[2] / len, result[3] / len];
        }
        return result;
    }

    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let sin_theta = theta.sin();
    let sin_theta_0 = theta_0.sin();

    let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;

    [
        a[0] * s0 + b[0] * s1,
        a[1] * s0 + b[1] * s1,
        a[2] * s0 + b[2] * s1,
        a[3] * s0 + b[3] * s1,
    ]
}

/// Material information extracted from GLTF
#[derive(Debug, Clone, Default)]
pub struct LoadedMaterial {
    /// Path to base color texture (relative to GLTF file) - for external textures
    pub base_color_texture: Option<String>,
    /// Embedded base color texture data (for GLB files)
    pub base_color_texture_data: Option<LoadedTexture>,
    /// Path to normal map texture
    pub normal_texture: Option<String>,
    /// Base color factor (RGBA) if no texture
    pub base_color_factor: [f32; 4],
    /// Alpha mode: "OPAQUE", "MASK", or "BLEND"
    pub alpha_mode: String,
    /// Alpha cutoff for MASK mode
    pub alpha_cutoff: f32,
    /// Whether material is double-sided
    pub double_sided: bool,
}

impl LoadedMaterial {
    /// Check if this material has a texture (either embedded or external path)
    pub fn has_texture(&self) -> bool {
        self.base_color_texture_data.is_some() || self.base_color_texture.is_some()
    }

    /// Get or create texture data for this material
    /// If no texture exists but baseColorFactor is set, creates a 1x1 texture from it
    pub fn get_or_create_texture(&self) -> Option<LoadedTexture> {
        // Return existing texture if available
        if let Some(ref tex) = self.base_color_texture_data {
            return Some(tex.clone());
        }

        // Create synthetic 1x1 texture from baseColorFactor
        // Only if the color is not default white (1,1,1,1)
        let [r, g, b, a] = self.base_color_factor;
        if (r - 1.0).abs() < 0.01 && (g - 1.0).abs() < 0.01 && (b - 1.0).abs() < 0.01 {
            // Default white factor, don't create texture
            return None;
        }

        // Convert linear color to sRGB for proper display
        let to_srgb = |linear: f32| -> u8 {
            let srgb = if linear <= 0.0031308 {
                linear * 12.92
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            };
            (srgb.clamp(0.0, 1.0) * 255.0) as u8
        };

        let data = vec![
            to_srgb(r),
            to_srgb(g),
            to_srgb(b),
            (a.clamp(0.0, 1.0) * 255.0) as u8,
        ];

        Some(LoadedTexture {
            width: 1,
            height: 1,
            data,
        })
    }
}

/// Loaded mesh data ready for GPU upload
#[derive(Debug, Clone)]
pub struct LoadedMesh {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Vertex colors (RGBA) - from COLOR_0 attribute or material baseColorFactor
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    /// Material for this mesh
    pub material: LoadedMaterial,
    /// Joint indices per vertex (up to 4 joints) for skeletal animation
    pub joint_indices: Vec<[u16; 4]>,
    /// Joint weights per vertex (up to 4 weights, should sum to 1.0)
    pub joint_weights: Vec<[f32; 4]>,
}

impl LoadedMesh {
    /// Check if this mesh has skinning data
    pub fn is_skinned(&self) -> bool {
        !self.joint_indices.is_empty() && !self.joint_weights.is_empty()
    }
}

/// A complete loaded model with potentially multiple meshes
#[derive(Debug, Clone)]
pub struct LoadedModel {
    pub name: String,
    pub meshes: Vec<LoadedMesh>,
    /// Bounding box: (min, max)
    pub bounds: ([f32; 3], [f32; 3]),
    /// Skeleton for skinned animation (if present)
    pub skeleton: Option<Skeleton>,
    /// Animation clips available for this model
    pub animations: Vec<AnimationClip>,
}

impl LoadedModel {
    /// Check if this model has skeletal animation
    pub fn is_animated(&self) -> bool {
        self.skeleton.is_some() && !self.animations.is_empty()
    }

    /// Find an animation by name
    pub fn find_animation(&self, name: &str) -> Option<&AnimationClip> {
        self.animations.iter().find(|a| a.name.eq_ignore_ascii_case(name))
    }

    /// Get list of animation names
    pub fn animation_names(&self) -> Vec<&str> {
        self.animations.iter().map(|a| a.name.as_str()).collect()
    }
}

/// Security limits for GLTF loading
mod limits {
    pub const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50 MB
    pub const MAX_VERTICES: usize = 100_000;
    pub const MAX_INDICES: usize = 500_000;
    pub const MAX_MESHES: usize = 50;
    pub const MAX_TEXTURE_SIZE: u32 = 4096;
}

/// Loaded texture data ready for GPU upload
#[derive(Debug, Clone)]
pub struct LoadedTexture {
    pub width: u32,
    pub height: u32,
    /// RGBA8 pixel data
    pub data: Vec<u8>,
}

/// Load a texture from disk (PNG, JPEG, etc.)
pub fn load_texture(path: &str) -> Result<LoadedTexture, String> {
    use image::GenericImageView;

    // Security: validate path
    if path.contains("..") {
        return Err("Path traversal not allowed".to_string());
    }

    let img = image::open(path)
        .map_err(|e| format!("Failed to load texture '{}': {}", path, e))?;

    let (width, height) = img.dimensions();

    // Security: limit texture size
    if width > limits::MAX_TEXTURE_SIZE || height > limits::MAX_TEXTURE_SIZE {
        return Err(format!(
            "Texture too large: {}x{} (max {})",
            width, height, limits::MAX_TEXTURE_SIZE
        ));
    }

    // Convert to RGBA8
    let rgba = img.to_rgba8();
    let data = rgba.into_raw();

    log::debug!("[Texture] Loaded '{}': {}x{}", path, width, height);

    Ok(LoadedTexture { width, height, data })
}

/// Create a wgpu texture from loaded texture data
pub fn create_gpu_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &LoadedTexture,
    label: Option<&str>,
) -> (wgpu::Texture, wgpu::TextureView) {
    let size = wgpu::Extent3d {
        width: texture.width,
        height: texture.height,
        depth_or_array_layers: 1,
    };

    let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &gpu_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texture.data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * texture.width),
            rows_per_image: Some(texture.height),
        },
        size,
    );

    let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
    (gpu_texture, view)
}

/// Sanitize a float value from external source
#[inline]
fn sanitize_float(v: f32) -> f32 {
    if v.is_finite() && v.abs() <= 1e6 {
        v
    } else {
        0.0
    }
}

/// Load a GLTF model from disk
pub fn load_gltf(path: &str) -> Result<LoadedModel, String> {
    load_gltf_with_options(path, 1.0, [0.0, 0.0, 0.0])
}

/// Load a GLTF model with scale and offset normalization
pub fn load_gltf_with_options(
    path: &str,
    scale: f32,
    offset: [f32; 3],
) -> Result<LoadedModel, String> {
    let path_obj = Path::new(path);
    let base_dir = path_obj.parent().unwrap_or(Path::new("."));

    // Security: validate path
    if path.contains("..") {
        return Err("Path traversal not allowed".to_string());
    }

    // Security: check file exists and size
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Cannot read file metadata: {}", e))?;

    if metadata.len() > limits::MAX_FILE_SIZE {
        return Err(format!(
            "File too large: {} bytes (max {})",
            metadata.len(),
            limits::MAX_FILE_SIZE
        ));
    }

    // Load GLTF - images contains decoded embedded textures from GLB files
    let (document, buffers, images) = gltf::import(path)
        .map_err(|e| format!("Failed to load GLTF: {}", e))?;

    // Convert embedded images to LoadedTexture format
    let embedded_textures: Vec<Option<LoadedTexture>> = images
        .iter()
        .map(|img| {
            let width = img.width;
            let height = img.height;

            // Security: limit texture size
            if width > limits::MAX_TEXTURE_SIZE || height > limits::MAX_TEXTURE_SIZE {
                log::warn!(
                    "[GLTF] Embedded texture too large: {}x{} (max {}), skipping",
                    width, height, limits::MAX_TEXTURE_SIZE
                );
                return None;
            }

            // Convert to RGBA8 format
            let data = match img.format {
                gltf::image::Format::R8G8B8A8 => img.pixels.clone(),
                gltf::image::Format::R8G8B8 => {
                    // Convert RGB to RGBA
                    img.pixels
                        .chunks(3)
                        .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                        .collect()
                }
                gltf::image::Format::R8 => {
                    // Grayscale to RGBA
                    img.pixels
                        .iter()
                        .flat_map(|&g| [g, g, g, 255])
                        .collect()
                }
                gltf::image::Format::R8G8 => {
                    // RG to RGBA (treat as grayscale + alpha)
                    img.pixels
                        .chunks(2)
                        .flat_map(|rg| [rg[0], rg[0], rg[0], rg[1]])
                        .collect()
                }
                _ => {
                    log::warn!("[GLTF] Unsupported image format: {:?}", img.format);
                    return None;
                }
            };

            println!("[GLTF] Extracted embedded texture {}: {}x{}", images.iter().position(|i| std::ptr::eq(i, img)).unwrap_or(999), width, height);
            Some(LoadedTexture { width, height, data })
        })
        .collect();

    println!("[GLTF] Total embedded textures: {}", embedded_textures.iter().filter(|t| t.is_some()).count());

    // Build image URI lookup for external textures
    let image_uris: Vec<Option<String>> = document
        .images()
        .map(|img| {
            match img.source() {
                gltf::image::Source::Uri { uri, .. } => {
                    Some(base_dir.join(uri).to_string_lossy().to_string())
                }
                gltf::image::Source::View { .. } => None, // Embedded - handled above
            }
        })
        .collect();

    // Build texture to image mapping
    let texture_to_image: Vec<Option<usize>> = document
        .textures()
        .map(|tex| Some(tex.source().index()))
        .collect();

    let model_name = path_obj
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut meshes = Vec::new();
    let mut global_min = [f32::MAX; 3];
    let mut global_max = [f32::MIN; 3];

    // Process all meshes in the document
    for mesh in document.meshes() {
        if meshes.len() >= limits::MAX_MESHES {
            log::warn!("[GLTF] Too many meshes, stopping at {}", limits::MAX_MESHES);
            break;
        }

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // Extract material information
            let material = if let Some(mat) = primitive.material().index() {
                let gltf_mat = document.materials().nth(mat).unwrap();
                let pbr = gltf_mat.pbr_metallic_roughness();

                // Get base color texture - try embedded first, then external path
                let texture_idx = pbr.base_color_texture().map(|info| info.texture().index());
                let texture_image_idx = texture_idx
                    .and_then(|idx| texture_to_image.get(idx).cloned().flatten());

                // Debug: log which texture index this mesh uses
                if let Some(idx) = texture_idx {
                    println!("[GLTF]     -> uses texture {} -> image {:?}", idx, texture_image_idx);
                }

                // Try to get embedded texture data
                let base_color_texture_data = texture_image_idx
                    .and_then(|img_idx| embedded_textures.get(img_idx).cloned().flatten());

                // Fall back to external texture path
                let base_color_texture = if base_color_texture_data.is_none() {
                    texture_image_idx
                        .and_then(|img_idx| image_uris.get(img_idx).cloned().flatten())
                } else {
                    None
                };

                // Get normal texture path
                let normal_texture = gltf_mat
                    .normal_texture()
                    .and_then(|info| texture_to_image.get(info.texture().index()).cloned().flatten())
                    .and_then(|img_idx| image_uris.get(img_idx).cloned().flatten());

                let base_color_factor = pbr.base_color_factor();

                LoadedMaterial {
                    base_color_texture,
                    base_color_texture_data,
                    normal_texture,
                    base_color_factor,
                    alpha_mode: match gltf_mat.alpha_mode() {
                        gltf::material::AlphaMode::Opaque => "OPAQUE".to_string(),
                        gltf::material::AlphaMode::Mask => "MASK".to_string(),
                        gltf::material::AlphaMode::Blend => "BLEND".to_string(),
                    },
                    alpha_cutoff: gltf_mat.alpha_cutoff().unwrap_or(0.5),
                    double_sided: gltf_mat.double_sided(),
                }
            } else {
                LoadedMaterial::default()
            };

            // Read positions (required) with scale and offset
            let positions: Vec<[f32; 3]> = match reader.read_positions() {
                Some(iter) => iter
                    .map(|p| [
                        (sanitize_float(p[0]) + offset[0]) * scale,
                        (sanitize_float(p[1]) + offset[1]) * scale,
                        (sanitize_float(p[2]) + offset[2]) * scale,
                    ])
                    .collect(),
                None => continue, // Skip primitives without positions
            };

            if positions.is_empty() {
                continue;
            }

            if positions.len() > limits::MAX_VERTICES {
                log::warn!(
                    "[GLTF] Mesh {} has too many vertices ({}), skipping",
                    mesh.name().unwrap_or("unnamed"),
                    positions.len()
                );
                continue;
            }

            // Update bounds
            for pos in &positions {
                for i in 0..3 {
                    global_min[i] = global_min[i].min(pos[i]);
                    global_max[i] = global_max[i].max(pos[i]);
                }
            }

            // Read normals (generate if missing)
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| {
                    iter.map(|n| {
                        let sanitized = [sanitize_float(n[0]), sanitize_float(n[1]), sanitize_float(n[2])];
                        // Normalize
                        let len_sq = sanitized[0] * sanitized[0]
                            + sanitized[1] * sanitized[1]
                            + sanitized[2] * sanitized[2];
                        if len_sq > 0.0001 {
                            let len = len_sq.sqrt();
                            [sanitized[0] / len, sanitized[1] / len, sanitized[2] / len]
                        } else {
                            [0.0, 1.0, 0.0]
                        }
                    })
                    .collect()
                })
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            // Read UVs (generate if missing)
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|iter| {
                    iter.into_f32()
                        .map(|uv| [sanitize_float(uv[0]), sanitize_float(uv[1])])
                        .collect()
                })
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            // Read vertex colors (COLOR_0) - fallback to material baseColorFactor
            let colors: Vec<[f32; 4]> = reader
                .read_colors(0)
                .map(|iter| {
                    iter.into_rgba_f32()
                        .map(|c| [
                            sanitize_float(c[0]),
                            sanitize_float(c[1]),
                            sanitize_float(c[2]),
                            sanitize_float(c[3]),
                        ])
                        .collect()
                })
                .unwrap_or_else(|| {
                    // No vertex colors - use material's baseColorFactor for all vertices
                    vec![material.base_color_factor; positions.len()]
                });

            // Read indices
            let indices: Vec<u32> = match reader.read_indices() {
                Some(iter) => iter.into_u32().collect(),
                None => {
                    // Generate indices for non-indexed geometry
                    (0..positions.len() as u32).collect()
                }
            };

            if indices.len() > limits::MAX_INDICES {
                log::warn!(
                    "[GLTF] Mesh {} has too many indices ({}), skipping",
                    mesh.name().unwrap_or("unnamed"),
                    indices.len()
                );
                continue;
            }

            // Validate indices
            let max_index = positions.len() as u32;
            let valid_indices: Vec<u32> = indices
                .into_iter()
                .filter(|&i| i < max_index)
                .collect();

            if valid_indices.is_empty() {
                continue;
            }

            // Read joint indices (JOINTS_0) for skeletal animation
            let joint_indices: Vec<[u16; 4]> = reader
                .read_joints(0)
                .map(|iter| {
                    iter.into_u16()
                        .map(|j| j)
                        .collect()
                })
                .unwrap_or_else(Vec::new);

            // Read joint weights (WEIGHTS_0) for skeletal animation
            let joint_weights: Vec<[f32; 4]> = reader
                .read_weights(0)
                .map(|iter| {
                    iter.into_f32()
                        .map(|w| [
                            sanitize_float(w[0]),
                            sanitize_float(w[1]),
                            sanitize_float(w[2]),
                            sanitize_float(w[3]),
                        ])
                        .collect()
                })
                .unwrap_or_else(Vec::new);

            let mesh_name = mesh.name().unwrap_or("unnamed").to_string();
            let is_skinned = !joint_indices.is_empty() && !joint_weights.is_empty();
            println!(
                "[GLTF]   Mesh '{}': {} verts, {} tris, has_texture={}, alpha_mode={}, skinned={}",
                mesh_name,
                positions.len(),
                valid_indices.len() / 3,
                material.has_texture(),
                material.alpha_mode,
                is_skinned
            );

            meshes.push(LoadedMesh {
                name: mesh_name,
                positions,
                normals,
                uvs,
                colors,
                indices: valid_indices,
                material,
                joint_indices,
                joint_weights,
            });
        }
    }

    if meshes.is_empty() {
        return Err("No valid meshes found in GLTF".to_string());
    }

    // Ensure bounds are valid
    if global_min[0] > global_max[0] {
        global_min = [-1.0, -1.0, -1.0];
        global_max = [1.0, 1.0, 1.0];
    }

    // Load skeleton from first skin (if present)
    let skeleton = load_skeleton(&document, &buffers);
    if let Some(ref skel) = skeleton {
        println!("[GLTF] Loaded skeleton with {} joints", skel.joints.len());
    }

    // Load animations
    let animations = load_animations(&document, &buffers, skeleton.as_ref());
    if !animations.is_empty() {
        println!("[GLTF] Loaded {} animations: {:?}",
            animations.len(),
            animations.iter().map(|a| &a.name).collect::<Vec<_>>()
        );
    }

    log::info!(
        "[GLTF] Loaded '{}': {} meshes, {} animations, bounds: {:?} to {:?}",
        model_name,
        meshes.len(),
        animations.len(),
        global_min,
        global_max
    );

    Ok(LoadedModel {
        name: model_name,
        meshes,
        bounds: (global_min, global_max),
        skeleton,
        animations,
    })
}

/// Load skeleton from GLTF skins
fn load_skeleton(document: &gltf::Document, buffers: &[gltf::buffer::Data]) -> Option<Skeleton> {
    let skin = document.skins().next()?;

    // Build node index to joint index mapping
    let joint_nodes: Vec<usize> = skin.joints().map(|j| j.index()).collect();
    let node_to_joint: HashMap<usize, usize> = joint_nodes.iter()
        .enumerate()
        .map(|(joint_idx, &node_idx)| (node_idx, joint_idx))
        .collect();

    // Read inverse bind matrices
    let inverse_bind_matrices: Vec<[[f32; 4]; 4]> = skin.inverse_bind_matrices()
        .map(|accessor| {
            let reader = accessor.view().map(|view| {
                let buffer = &buffers[view.buffer().index()];
                &buffer[view.offset()..view.offset() + view.length()]
            });

            if let Some(data) = reader {
                let stride = accessor.size();
                let count = accessor.count();
                let mut matrices = Vec::with_capacity(count);

                for i in 0..count {
                    let offset = i * stride;
                    if offset + 64 <= data.len() {
                        let mut mat = [[0.0f32; 4]; 4];
                        for row in 0..4 {
                            for col in 0..4 {
                                let idx = offset + (row * 4 + col) * 4;
                                mat[row][col] = f32::from_le_bytes([
                                    data[idx], data[idx + 1], data[idx + 2], data[idx + 3]
                                ]);
                            }
                        }
                        matrices.push(mat);
                    }
                }
                matrices
            } else {
                vec![[[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0],
                      [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]; joint_nodes.len()]
            }
        })
        .unwrap_or_else(|| {
            vec![[[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0],
                  [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]; joint_nodes.len()]
        });

    // Build joint data
    let mut joints = Vec::with_capacity(joint_nodes.len());
    let mut roots = Vec::new();

    for (joint_idx, &node_idx) in joint_nodes.iter().enumerate() {
        let node = document.nodes().nth(node_idx)?;

        let (translation, rotation, node_scale) = node.transform().decomposed();

        let parent = node.index();
        // Find parent in joint list by traversing the node tree
        let parent_joint = document.nodes()
            .find(|n| n.children().any(|c| c.index() == parent))
            .and_then(|p| node_to_joint.get(&p.index()).copied());

        if parent_joint.is_none() {
            roots.push(joint_idx);
        }

        let children: Vec<usize> = node.children()
            .filter_map(|c| node_to_joint.get(&c.index()).copied())
            .collect();

        joints.push(Joint {
            index: joint_idx,
            name: node.name().unwrap_or("unnamed").to_string(),
            parent: parent_joint,
            children,
            local_translation: translation,
            local_rotation: rotation,
            local_scale: node_scale,
        });
    }

    Some(Skeleton {
        joints,
        inverse_bind_matrices,
        roots,
    })
}

/// Load animations from GLTF
fn load_animations(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    skeleton: Option<&Skeleton>,
) -> Vec<AnimationClip> {
    let skeleton = match skeleton {
        Some(s) => s,
        None => return Vec::new(),
    };

    // Build node index to joint index mapping
    let node_to_joint: HashMap<usize, usize> = skeleton.joints.iter()
        .enumerate()
        .map(|(joint_idx, joint)| {
            // Find the node index for this joint by matching name
            document.nodes()
                .find(|n| n.name() == Some(&joint.name))
                .map(|n| (n.index(), joint_idx))
        })
        .flatten()
        .collect();

    let mut animations = Vec::new();

    for anim in document.animations() {
        let name = anim.name().unwrap_or("unnamed").to_string();
        let mut channels = Vec::new();
        let mut max_time = 0.0f32;

        for channel in anim.channels() {
            let target = channel.target();
            let node_idx = target.node().index();

            // Skip if this node isn't part of the skeleton
            let joint_index = match node_to_joint.get(&node_idx) {
                Some(&idx) => idx,
                None => continue,
            };

            let property = match target.property() {
                gltf::animation::Property::Translation => AnimationProperty::Translation,
                gltf::animation::Property::Rotation => AnimationProperty::Rotation,
                gltf::animation::Property::Scale => AnimationProperty::Scale,
                _ => continue, // Skip morph targets
            };

            let sampler = channel.sampler();
            let interpolation = match sampler.interpolation() {
                gltf::animation::Interpolation::Linear => Interpolation::Linear,
                gltf::animation::Interpolation::Step => Interpolation::Step,
                gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
            };

            // Read keyframe times
            let times = read_accessor_f32(&sampler.input(), buffers);
            if let Some(last) = times.last() {
                max_time = max_time.max(*last);
            }

            // Read keyframe values
            let values = read_accessor_vec4(&sampler.output(), buffers, property);

            channels.push(AnimationChannel {
                joint_index,
                property,
                times,
                values,
                interpolation,
            });
        }

        if !channels.is_empty() {
            animations.push(AnimationClip {
                name,
                duration: max_time,
                channels,
            });
        }
    }

    animations
}

/// Read f32 values from a GLTF accessor
fn read_accessor_f32(accessor: &gltf::Accessor, buffers: &[gltf::buffer::Data]) -> Vec<f32> {
    let view = match accessor.view() {
        Some(v) => v,
        None => return Vec::new(),
    };

    let buffer = &buffers[view.buffer().index()];
    let data = &buffer[view.offset()..view.offset() + view.length()];

    let count = accessor.count();
    let mut result = Vec::with_capacity(count);

    for i in 0..count {
        let offset = i * 4;
        if offset + 4 <= data.len() {
            result.push(f32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
            ]));
        }
    }

    result
}

/// Read vec4 values from a GLTF accessor (for animation keyframes)
fn read_accessor_vec4(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
    property: AnimationProperty,
) -> Vec<[f32; 4]> {
    let view = match accessor.view() {
        Some(v) => v,
        None => return Vec::new(),
    };

    let buffer = &buffers[view.buffer().index()];
    let data = &buffer[view.offset()..view.offset() + view.length()];

    let count = accessor.count();
    let components = match property {
        AnimationProperty::Rotation => 4, // quaternion
        _ => 3, // vec3
    };
    let stride = components * 4; // 4 bytes per f32

    let mut result = Vec::with_capacity(count);

    for i in 0..count {
        let offset = i * stride;
        if offset + stride <= data.len() {
            let mut v = [0.0f32; 4];
            for c in 0..components {
                let idx = offset + c * 4;
                v[c] = f32::from_le_bytes([
                    data[idx], data[idx + 1], data[idx + 2], data[idx + 3]
                ]);
            }
            // For vec3 properties, set w to 1.0 (identity for scale) or 0.0 (translation)
            if components == 3 {
                v[3] = if property == AnimationProperty::Scale { 1.0 } else { 0.0 };
            }
            result.push(v);
        }
    }

    result
}

/// Cache for loaded models
pub struct ModelCache {
    models: HashMap<String, LoadedModel>,
    base_path: String,
}

impl ModelCache {
    pub fn new(base_path: &str) -> Self {
        Self {
            models: HashMap::new(),
            base_path: base_path.to_string(),
        }
    }

    /// Load a model by name (without extension)
    /// Tries .glb first, then .gltf
    pub fn load(&mut self, name: &str) -> Option<&LoadedModel> {
        if self.models.contains_key(name) {
            return self.models.get(name);
        }

        // Try .glb first (more common), then .gltf
        let glb_path = format!("{}/{}.glb", self.base_path, name);
        let gltf_path = format!("{}/{}.gltf", self.base_path, name);

        // Try loading .glb first - don't rely on exists() which can fail on Windows
        match load_gltf(&glb_path) {
            Ok(model) => {
                self.models.insert(name.to_string(), model);
                return self.models.get(name);
            }
            Err(_) => {
                // GLB failed, try GLTF
            }
        }

        // Fall back to .gltf
        match load_gltf(&gltf_path) {
            Ok(model) => {
                self.models.insert(name.to_string(), model);
                self.models.get(name)
            }
            Err(e) => {
                // Show current working directory for debugging path issues
                let cwd = std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".to_string());
                println!("[ModelCache] Failed to load '{}' from '{}': {} (cwd: {})", name, gltf_path, e, cwd);
                None
            }
        }
    }

    /// Get a previously loaded model
    pub fn get(&self, name: &str) -> Option<&LoadedModel> {
        self.models.get(name)
    }

    /// Check if a model is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.models.contains_key(name)
    }

    /// Preload multiple models
    pub fn preload(&mut self, names: &[&str]) {
        for name in names {
            let _ = self.load(name);
        }
    }
}
