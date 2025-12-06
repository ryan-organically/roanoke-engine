use tobj;
use croatoan_wfc::TreeTemplate;

/// Security limits for external asset loading
mod asset_limits {
    /// Maximum file size for OBJ files (50 MB)
    pub const MAX_OBJ_FILE_SIZE: usize = 50 * 1024 * 1024;
    /// Maximum vertices per mesh
    pub const MAX_VERTICES: usize = 500_000;
    /// Maximum indices per mesh
    pub const MAX_INDICES: usize = 3_000_000;
    /// Maximum materials per file
    pub const MAX_MATERIALS: usize = 100;
    /// Maximum models per file
    pub const MAX_MODELS: usize = 1000;
}

/// Validate file path for security (prevent path traversal)
fn validate_asset_path(path: &str) -> bool {
    // Reject empty paths
    if path.is_empty() {
        log::warn!("[SECURITY] Rejected empty asset path");
        return false;
    }

    // Reject path traversal attempts
    if path.contains("..") {
        log::warn!("[SECURITY] Rejected path traversal attempt: {}", path);
        return false;
    }

    // Reject absolute paths outside expected directories
    if path.starts_with('/') || path.starts_with('\\') {
        // Allow only if it contains expected asset directories
        if !path.contains("assets") && !path.contains("models") {
            log::warn!("[SECURITY] Rejected suspicious absolute path: {}", path);
            return false;
        }
    }

    // Reject null bytes
    if path.contains('\0') {
        log::warn!("[SECURITY] Rejected path with null byte");
        return false;
    }

    true
}

/// Sanitize a float value from external source
#[inline]
fn sanitize_float(v: f32) -> f32 {
    if v.is_finite() && v.abs() <= 1e10 { v } else { 0.0 }
}

/// Sanitize a vertex position from external source
#[inline]
fn sanitize_position(v: [f32; 3]) -> [f32; 3] {
    [sanitize_float(v[0]), sanitize_float(v[1]), sanitize_float(v[2])]
}

/// Sanitize a normal vector from external source
#[inline]
fn sanitize_normal(v: [f32; 3]) -> [f32; 3] {
    let sanitized = [sanitize_float(v[0]), sanitize_float(v[1]), sanitize_float(v[2])];
    // Normalize if possible, otherwise return up vector
    let len_sq = sanitized[0] * sanitized[0] + sanitized[1] * sanitized[1] + sanitized[2] * sanitized[2];
    if len_sq > 0.0001 {
        let len = len_sq.sqrt();
        [sanitized[0] / len, sanitized[1] / len, sanitized[2] / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}

/// Sanitize UV coordinates from external source
#[inline]
fn sanitize_uv(v: [f32; 2]) -> [f32; 2] {
    [sanitize_float(v[0]), sanitize_float(v[1])]
}

pub fn load_obj(path: &str) -> Option<TreeTemplate> {
    println!("[ASSET] Loading model: {}", path);

    // Security: validate path
    if !validate_asset_path(path) {
        eprintln!("[ASSET] Security: Invalid path rejected: {}", path);
        return None;
    }

    // Security: check file size before loading
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.len() as usize > asset_limits::MAX_OBJ_FILE_SIZE {
            eprintln!("[ASSET] Security: File too large ({} bytes, max {})",
                     metadata.len(), asset_limits::MAX_OBJ_FILE_SIZE);
            return None;
        }
    }

    let load_options = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ignore_points: true,
        ignore_lines: true,
    };

    match tobj::load_obj(path, &load_options) {
        Ok((models, materials)) => {
            // Security: check model count
            if models.len() > asset_limits::MAX_MODELS {
                eprintln!("[ASSET] Security: Too many models ({}, max {})",
                         models.len(), asset_limits::MAX_MODELS);
                return None;
            }

            let materials = materials.unwrap_or_default();

            // Security: check material count
            if materials.len() > asset_limits::MAX_MATERIALS {
                eprintln!("[ASSET] Security: Too many materials ({}, max {})",
                         materials.len(), asset_limits::MAX_MATERIALS);
                return None;
            }

            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut indices = Vec::new();
            let mut vertex_offset = 0;

            let mut skipped = Vec::new();
            let mut loaded = Vec::new();

            for (_idx, m) in models.iter().enumerate() {
                let mesh = &m.mesh;
                let obj_name = m.name.to_lowercase();

                // Only load Bark___0 - the other bark meshes have oversized cardboard leaves
                // Bark___0 has acceptable leaf geometry that looks good at all scales
                let is_good_bark = obj_name == "bark___0";

                if !is_good_bark {
                    skipped.push(format!("{} ({} faces)", m.name, mesh.indices.len() / 3));
                    continue;
                }

                // Double-check material name matches bark
                if let Some(mat_id) = mesh.material_id {
                    if mat_id < materials.len() {
                        let mat_name = &materials[mat_id].name.to_lowercase();
                        // Skip if material suggests leaves/foliage
                        if mat_name.contains("leaf") || mat_name.contains("leaves") || mat_name.contains("frond") {
                            skipped.push(format!("{} [mat:{}] ({} faces)", m.name, mat_name, mesh.indices.len() / 3));
                            continue;
                        }
                    }
                }

                loaded.push(format!("{} ({} faces)", m.name, mesh.indices.len() / 3));

                let mesh_vertex_count = mesh.positions.len() / 3;

                // Security: check vertex count before processing
                if positions.len() + mesh_vertex_count > asset_limits::MAX_VERTICES {
                    eprintln!("[ASSET] Security: Too many vertices, skipping mesh {}", m.name);
                    continue;
                }

                // Security: check index count
                if indices.len() + mesh.indices.len() > asset_limits::MAX_INDICES {
                    eprintln!("[ASSET] Security: Too many indices, skipping mesh {}", m.name);
                    continue;
                }

                // Positions (sanitized)
                for i in 0..mesh_vertex_count {
                    positions.push(sanitize_position([
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                    ]));
                }

                // Normals (sanitized)
                if !mesh.normals.is_empty() {
                    for i in 0..mesh.normals.len() / 3 {
                        normals.push(sanitize_normal([
                            mesh.normals[i * 3],
                            mesh.normals[i * 3 + 1],
                            mesh.normals[i * 3 + 2],
                        ]));
                    }
                } else {
                    // Generate dummy normals if missing (up)
                    for _ in 0..mesh_vertex_count {
                        normals.push([0.0, 1.0, 0.0]);
                    }
                }

                // UVs (sanitized)
                if !mesh.texcoords.is_empty() {
                    for i in 0..mesh.texcoords.len() / 2 {
                        uvs.push(sanitize_uv([
                            mesh.texcoords[i * 2],
                            1.0 - mesh.texcoords[i * 2 + 1], // Flip Y
                        ]));
                    }
                } else {
                    // Generate dummy UVs
                    for _ in 0..mesh_vertex_count {
                        uvs.push([0.0, 0.0]);
                    }
                }

                // Indices (validated)
                for idx in &mesh.indices {
                    let adjusted_idx = *idx + vertex_offset;
                    // Security: validate index is in bounds
                    if adjusted_idx as usize >= positions.len() + mesh_vertex_count {
                        eprintln!("[ASSET] Security: Invalid index {} in mesh {}", adjusted_idx, m.name);
                        continue;
                    }
                    indices.push(adjusted_idx);
                }

                vertex_offset += mesh_vertex_count as u32;
            }

            // Print summary
            println!("[ASSET] === TREE MESH SUMMARY ===");
            println!("[ASSET] SKIPPED (leaves): {:?}", skipped);
            println!("[ASSET] LOADED (bark): {:?}", loaded);
            println!("[ASSET] Total: {} verts, {} tris", positions.len(), indices.len() / 3);

            if positions.is_empty() {
                println!("[ASSET] WARNING: No mesh data loaded!");
                return None;
            }

            Some(TreeTemplate {
                positions,
                normals,
                uvs,
                indices,
            })
        }
        Err(e) => {
            eprintln!("[ASSET] Failed to load model '{}': {}", path, e);
            None
        }
    }
}
