use tobj;
use croatoan_wfc::TreeTemplate;

pub fn load_obj(path: &str) -> Option<TreeTemplate> {
    println!("[ASSET] Loading model: {}", path);
    
    let load_options = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ignore_points: true,
        ignore_lines: true,
    };

    match tobj::load_obj(path, &load_options) {
        Ok((models, materials)) => {
            let materials = materials.unwrap_or_default();
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

                // Positions
                for i in 0..mesh.positions.len() / 3 {
                    positions.push([
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                    ]);
                }

                // Normals
                if !mesh.normals.is_empty() {
                    for i in 0..mesh.normals.len() / 3 {
                        normals.push([
                            mesh.normals[i * 3],
                            mesh.normals[i * 3 + 1],
                            mesh.normals[i * 3 + 2],
                        ]);
                    }
                } else {
                    // Generate dummy normals if missing (up)
                    for _ in 0..mesh.positions.len() / 3 {
                        normals.push([0.0, 1.0, 0.0]);
                    }
                }

                // UVs
                if !mesh.texcoords.is_empty() {
                    for i in 0..mesh.texcoords.len() / 2 {
                        uvs.push([
                            mesh.texcoords[i * 2],
                            1.0 - mesh.texcoords[i * 2 + 1], // Flip Y
                        ]);
                    }
                } else {
                    // Generate dummy UVs
                    for _ in 0..mesh.positions.len() / 3 {
                        uvs.push([0.0, 0.0]);
                    }
                }

                // Indices
                for idx in &mesh.indices {
                    indices.push(*idx + vertex_offset);
                }

                vertex_offset += (mesh.positions.len() / 3) as u32;
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
