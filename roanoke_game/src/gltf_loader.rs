//! GLTF model loader for animal and foliage models
//!
//! Loads GLTF files and extracts mesh data for rendering.

use std::collections::HashMap;
use std::path::Path;

/// Material information extracted from GLTF
#[derive(Debug, Clone, Default)]
pub struct LoadedMaterial {
    /// Path to base color texture (relative to GLTF file)
    pub base_color_texture: Option<String>,
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

/// Loaded mesh data ready for GPU upload
#[derive(Debug, Clone)]
pub struct LoadedMesh {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    /// Material for this mesh
    pub material: LoadedMaterial,
}

/// A complete loaded model with potentially multiple meshes
#[derive(Debug, Clone)]
pub struct LoadedModel {
    pub name: String,
    pub meshes: Vec<LoadedMesh>,
    /// Bounding box: (min, max)
    pub bounds: ([f32; 3], [f32; 3]),
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

    // Load GLTF
    let (document, buffers, _images) = gltf::import(path)
        .map_err(|e| format!("Failed to load GLTF: {}", e))?;

    // Build image URI lookup
    let image_uris: Vec<Option<String>> = document
        .images()
        .map(|img| {
            match img.source() {
                gltf::image::Source::Uri { uri, .. } => {
                    Some(base_dir.join(uri).to_string_lossy().to_string())
                }
                gltf::image::Source::View { .. } => None, // Embedded image, not external file
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

                // Get base color texture path
                let base_color_texture = pbr
                    .base_color_texture()
                    .and_then(|info| texture_to_image.get(info.texture().index()).cloned().flatten())
                    .and_then(|img_idx| image_uris.get(img_idx).cloned().flatten());

                // Get normal texture path
                let normal_texture = gltf_mat
                    .normal_texture()
                    .and_then(|info| texture_to_image.get(info.texture().index()).cloned().flatten())
                    .and_then(|img_idx| image_uris.get(img_idx).cloned().flatten());

                let base_color_factor = pbr.base_color_factor();

                LoadedMaterial {
                    base_color_texture,
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

            meshes.push(LoadedMesh {
                name: mesh.name().unwrap_or("unnamed").to_string(),
                positions,
                normals,
                uvs,
                indices: valid_indices,
                material,
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

    log::info!(
        "[GLTF] Loaded '{}': {} meshes, bounds: {:?} to {:?}",
        model_name,
        meshes.len(),
        global_min,
        global_max
    );

    Ok(LoadedModel {
        name: model_name,
        meshes,
        bounds: (global_min, global_max),
    })
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
    pub fn load(&mut self, name: &str) -> Option<&LoadedModel> {
        if self.models.contains_key(name) {
            return self.models.get(name);
        }

        let path = format!("{}/{}.gltf", self.base_path, name);

        match load_gltf(&path) {
            Ok(model) => {
                self.models.insert(name.to_string(), model);
                self.models.get(name)
            }
            Err(e) => {
                log::warn!("[ModelCache] Failed to load '{}': {}", name, e);
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
