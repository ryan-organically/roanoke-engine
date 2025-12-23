use crate::noise_util;
use glam::{Vec2, Vec3};

// ============================================================================
// SPAWN AREA TERRAIN FEATURES
// Large-scale river valley system with natural elevation gradients
// ============================================================================

/// Calculate marsh zone influence south of spawn
/// Returns 0.0 outside marsh, up to 1.0 at marsh center
fn calculate_marsh_zone(x: f32, z: f32) -> f32 {
    // Marsh centered at (0, -150) with ~250m radius
    let marsh_center = Vec2::new(0.0, -150.0);
    let dist = Vec2::new(x, z).distance(marsh_center);
    let marsh_radius = 250.0;

    if dist < marsh_radius {
        // Smooth quadratic falloff at edges
        let factor = 1.0 - (dist / marsh_radius);
        factor * factor
    } else {
        0.0
    }
}

/// River valley terrain system
/// Creates a natural river flowing from inland highlands to the ocean
/// with gradual elevation changes over thousands of units
struct RiverValleyResult {
    /// Height modification from the valley/ridge system
    height_mod: f32,
    /// River proximity (0.0 = not in river, 1.0 = river center)
    river_factor: f32,
    /// Current terrain slope (for waterfall detection)
    slope: f32,
    /// Whether this is a waterfall zone
    is_waterfall: bool,
    /// Whether this is rocky rapids
    is_rocky: bool,
}

/// Calculate the main river valley system
/// This creates an E-W oriented valley with the river flowing toward the ocean
fn calculate_river_valley(x: f32, z: f32, seed: u32) -> RiverValleyResult {
    // River flows from west (inland, high) to east (ocean, low)
    // Origin around x = -800 (headwaters), mouth around x = 300 (beach)

    let river_start_x = -800.0;
    let river_end_x = 350.0;
    let river_length = river_end_x - river_start_x;

    // Progress along river (0.0 = headwaters, 1.0 = mouth)
    let river_progress = ((x - river_start_x) / river_length).clamp(0.0, 1.0);

    // River path - meanders using low-frequency noise
    // More meandering in the middle section, straighter at ends
    let meander_strength = (river_progress * (1.0 - river_progress) * 4.0).sqrt() * 80.0;
    let meander = noise_util::fbm(
        Vec2::new(x * 0.002, 0.0),
        3, 2.0, 0.5, seed.wrapping_add(800)
    ) * meander_strength;

    let river_center_z = meander;

    // River width: narrow creek at start, wider toward mouth
    // Logarithmic growth for natural feel
    let base_width = 4.0 + (1.0 + river_progress * 10.0).ln() * 12.0;
    let width_noise = noise_util::fbm(
        Vec2::new(x * 0.01, z * 0.01),
        2, 2.0, 0.5, seed.wrapping_add(801)
    );
    let river_width = base_width * (0.8 + width_noise.abs() * 0.4);

    let dist_from_river = (z - river_center_z).abs();

    // =========================================================================
    // ELEVATION PROFILE (logarithmic descent with local variations)
    // =========================================================================

    // Base elevation: high inland, low at coast
    // Using logarithmic curve for natural river gradient
    let elevation_at_start = 45.0;  // Headwaters elevation
    let elevation_at_end = 2.0;     // Near sea level at mouth

    // Logarithmic interpolation for gentle gradient that steepens at drops
    let log_progress = if river_progress > 0.01 {
        river_progress.ln().abs() / 5.0_f32.ln()  // Normalized log curve
    } else {
        0.0
    };
    let base_elevation = lerp(elevation_at_start, elevation_at_end, river_progress.sqrt());

    // Add terrain undulations - gentle rolling hills along the valley
    let terrain_undulation = noise_util::fbm(
        Vec2::new(x * 0.0008, z * 0.0008),
        4, 2.0, 0.5, seed.wrapping_add(802)
    ) * 8.0;

    // Valley walls - terrain rises away from river
    let valley_width = 200.0 + river_progress * 150.0;  // Valley widens downstream
    let valley_factor = (dist_from_river / valley_width).clamp(0.0, 1.0);
    let valley_wall_height = valley_factor * valley_factor * 15.0;

    // =========================================================================
    // SLOPE AND WATERFALL DETECTION
    // =========================================================================

    // Calculate local slope by sampling nearby
    let slope_sample_dist = 20.0;
    let height_here = base_elevation;
    let height_downstream = {
        let downstream_progress = ((x + slope_sample_dist - river_start_x) / river_length).clamp(0.0, 1.0);
        lerp(elevation_at_start, elevation_at_end, downstream_progress.sqrt())
    };
    let slope = (height_here - height_downstream) / slope_sample_dist;

    // Add occasional steeper sections (potential waterfalls/rapids)
    let steep_noise = noise_util::fbm(
        Vec2::new(x * 0.003, 0.0),
        2, 2.0, 0.5, seed.wrapping_add(803)
    );

    // Waterfalls occur where slope is steep AND we're in the right progress zone
    let waterfall_zones = [0.15, 0.35, 0.55];  // Three potential waterfall locations
    let mut is_waterfall = false;
    let mut waterfall_drop = 0.0;

    for &wf_progress in &waterfall_zones {
        let wf_dist = (river_progress - wf_progress).abs();
        if wf_dist < 0.03 && steep_noise > 0.3 {
            is_waterfall = dist_from_river < river_width * 1.5;
            if is_waterfall {
                // Sharp drop at waterfall
                let wf_factor = 1.0 - (wf_dist / 0.03);
                waterfall_drop = wf_factor * wf_factor * 8.0;
            }
        }
    }

    // Rocky sections - mid-river where gradient is moderate
    let rocky_noise = noise_util::turbulence(
        Vec2::new(x * 0.008, z * 0.008),
        3, 2.0, 0.5, seed.wrapping_add(804)
    );
    let is_rocky = rocky_noise > 0.6
        && river_progress > 0.1
        && river_progress < 0.7
        && dist_from_river < river_width * 0.8;

    // =========================================================================
    // FINAL HEIGHT CALCULATION
    // =========================================================================

    let mut height_mod = base_elevation + terrain_undulation + valley_wall_height - waterfall_drop;

    // River bed depression
    let river_factor = if dist_from_river < river_width && x > river_start_x && x < river_end_x {
        let raw_factor = 1.0 - (dist_from_river / river_width);
        raw_factor * raw_factor
    } else {
        0.0
    };

    // Add rocky bed texture
    if is_rocky && river_factor > 0.3 {
        let rock_bumps = noise_util::fbm(
            Vec2::new(x * 0.05, z * 0.05),
            3, 2.5, 0.6, seed.wrapping_add(805)
        );
        height_mod += rock_bumps.abs() * 0.8;
    }

    RiverValleyResult {
        height_mod,
        river_factor,
        slope,
        is_waterfall,
        is_rocky,
    }
}

/// Generate a procedural terrain chunk mesh
/// Returns (positions, colors, normals, indices)
pub fn generate_terrain_chunk(
    seed: u32,
    size: u32,
    offset_x: i32,
    offset_z: i32,
    scale: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let grid_size = size + 1; // Number of vertices per dimension
    let vertex_count = (grid_size * grid_size) as usize;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);

    // Generate vertices with biome-based height and coloring
    for z in 0..grid_size {
        for x in 0..grid_size {
            // Global coordinates
            // Scale determines the distance between vertices
            let global_x = (x as f32 * scale) + offset_x as f32;
            let global_z = (z as f32 * scale) + offset_z as f32;

            let (height, base_color) = get_height_at(global_x, global_z, seed);

            // Global position for the mesh
            // We use global coordinates so the chunks align perfectly without needing model matrices
            positions.push([global_x, height, global_z]);
            colors.push(base_color);
        }
    }

    // Generate indices for triangles
    let triangle_count = (size * size * 2) as usize;
    let mut indices = Vec::with_capacity(triangle_count * 3);

    for z in 0..size {
        for x in 0..size {
            let top_left = z * grid_size + x;
            let top_right = top_left + 1;
            let bottom_left = (z + 1) * grid_size + x;
            let bottom_right = bottom_left + 1;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    // Calculate smooth normals
    let normals = calculate_smooth_normals(&positions, &indices, grid_size);

    // VERIFICATION OUTPUT
    if offset_x == 0 && offset_z == 0 {
        println!("[VERIFY] Generated Terrain Chunk: {}x{} (Scale {}) at ({}, {})", size, size, scale, offset_x, offset_z);
        println!("[VERIFY] Vertex Count: {}", positions.len());
        println!("[VERIFY] Triangle Count: {}", indices.len() / 3);
    }

    (positions, colors, normals, indices)
}

/// Calculate smooth vertex normals by averaging face normals
fn calculate_smooth_normals(positions: &[[f32; 3]], indices: &[u32], _grid_size: u32) -> Vec<[f32; 3]> {
    let vertex_count = positions.len();
    let mut normals = vec![[0.0f32; 3]; vertex_count];

    // Accumulate face normals for each vertex
    for triangle in indices.chunks(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        let p0 = Vec3::from_array(positions[i0]);
        let p1 = Vec3::from_array(positions[i1]);
        let p2 = Vec3::from_array(positions[i2]);

        // Calculate face normal
        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let face_normal = edge1.cross(edge2);

        // Add to each vertex
        normals[i0][0] += face_normal.x;
        normals[i0][1] += face_normal.y;
        normals[i0][2] += face_normal.z;

        normals[i1][0] += face_normal.x;
        normals[i1][1] += face_normal.y;
        normals[i1][2] += face_normal.z;

        normals[i2][0] += face_normal.x;
        normals[i2][1] += face_normal.y;
        normals[i2][2] += face_normal.z;
    }

    // Normalize all normals
    for normal in &mut normals {
        let n = Vec3::from_array(*normal);
        let normalized = n.normalize();
        *normal = normalized.to_array();
    }

    normals
}

/// Calculate height and color at a specific global position
pub fn get_height_at(x: f32, z: f32, seed: u32) -> (f32, [f32; 3]) {
    // 1. Biome Noise (Low Frequency)
    let biome_scale = 0.002;
    let biome_noise = noise_util::fbm(
        Vec2::new(x * biome_scale, z * biome_scale),
        3, 2.0, 0.5, seed + 100
    );
    let noise_norm = (biome_noise + 1.0) * 0.5;

    // 2. Eastern Sea Gradient
    let gradient = -x * 0.001;
    let t = (noise_norm * 0.3 + gradient + 0.5).clamp(0.0, 1.0);

    // 3. Detail Noise
    let detail_noise = noise_util::fbm(
        Vec2::new(x * 0.05, z * 0.05),
        4, 2.0, 0.5, seed
    );

    // 4. River system - carves channels through terrain
    let river_depth = calculate_river_depth(x, z, seed);

    // 5. Pond system - small water bodies
    let pond_depth = calculate_pond_depth(x, z, seed);

    // 6. Rolling hills for inland areas (increases with -X distance)
    let inland_dist = (-x - 200.0).max(0.0); // Hills start 200 units inland
    let hill_strength = (inland_dist / 500.0).min(1.0); // Full strength at 700 units
    let hill_noise = noise_util::fbm(
        Vec2::new(x * 0.008, z * 0.008),
        3, 2.0, 0.5, seed + 500
    );
    let rolling_hills = hill_noise * 25.0 * hill_strength; // Up to 25m hills

    // 7. Biome Definitions
    // Beach: 0.45-0.65 - steeper incline with dunes
    // Forest-edge zone: 0.65-0.72 - drift plateaus and treeline
    // Forest starts higher: 8.0m+ with sandy understory near coast

    // Dune noise - creates rolling sand dunes on beach
    let dune_noise = noise_util::fbm(
        Vec2::new(x * 0.015, z * 0.02),
        4, 2.0, 0.5, seed + 777
    );

    // Drift plateau noise - creates flat sandy terraces near treeline
    let plateau_noise = noise_util::fbm(
        Vec2::new(x * 0.008, z * 0.008),
        2, 2.0, 0.5, seed + 888
    );

    let (mut base_height, height_mult, mut base_color) = if t < 0.45 {
        // Ocean / Shallow Water
        let sandbar = if detail_noise > 0.5 { 0.5 } else { 0.0 };
        let water_depth = lerp(-5.0, -0.5, t / 0.45);
        let h = water_depth + sandbar;
        let depth_factor = (t / 0.45).clamp(0.0, 1.0);
        let c = lerp_color([0.05, 0.3, 0.4], [0.2, 0.8, 0.8], depth_factor);
        (h, 0.1, c)
    } else if t < 0.65 {
        // Beach / Dunes - steeper incline (0.0 to 6.0m) with dune formations
        let blend = (t - 0.45) / 0.20;
        let base_h = lerp(0.0, 6.0, blend); // Steeper: was 3.0, now 6.0
        // Add dune ridges that increase toward treeline
        let dune_strength = blend * 2.5; // Dunes get bigger inland
        let dunes = dune_noise.abs() * dune_strength;
        let h = base_h + dunes;
        let c = lerp_color([0.60, 0.50, 0.40], [0.52, 0.46, 0.36], blend);
        (h, 0.4, c)
    } else if t < 0.72 {
        // Forest-Edge Zone (Treeline) - drift plateaus with sandy patches
        let blend = (t - 0.65) / 0.07;
        let base_h = lerp(6.0, 10.0, blend); // Higher: was 3.0-5.5, now 6.0-10.0
        // Drift plateaus - flat sandy terraces held by vegetation
        let plateau_strength = (1.0 - blend) * 3.0; // Stronger at start of treeline
        let plateaus = if plateau_noise > 0.2 {
            (plateau_noise - 0.2) * plateau_strength
        } else {
            0.0
        };
        let h = base_h + plateaus;
        // Sandy soil color transitioning to forest floor
        let c = lerp_color([0.50, 0.45, 0.35], [0.30, 0.40, 0.20], blend);
        (h, 0.6, c)
    } else {
        // Coastal Deciduous Forest - starts higher, sandy understory near treeline
        let blend = (t - 0.72) / 0.28;
        // Hills only start at t > 0.82 (far inland), not immediately
        let far_inland = ((t - 0.82) / 0.18).clamp(0.0, 1.0);
        let adjusted_hills = rolling_hills * far_inland;
        let h = lerp(10.0, 15.0, blend) + adjusted_hills; // Higher: was 5.5-10.0, now 10.0-15.0
        let m = 0.7 * (1.0 - far_inland * 0.3);
        // Sandy understory near coast, darker forest floor inland
        let sandy_factor = (1.0 - blend).powi(2) * 0.3;
        let base_forest = lerp_color([0.22, 0.38, 0.12], [0.08, 0.22, 0.06], blend);
        let c = lerp_color(base_forest, [0.45, 0.42, 0.32], sandy_factor);
        (h, m, c)
    };

    // Apply detail noise
    let mut height = base_height + detail_noise * height_mult;

    // Apply river carving (cuts into terrain)
    if river_depth > 0.0 && height > -1.0 {
        let river_bottom = -1.5; // Rivers are shallow
        height = lerp(height, river_bottom, river_depth);
        // River bed color
        if river_depth > 0.3 {
            base_color = lerp_color(base_color, [0.15, 0.25, 0.30], river_depth);
        }
    }

    // Apply pond carving
    if pond_depth > 0.0 && height > -0.5 {
        let pond_bottom = -1.0;
        height = lerp(height, pond_bottom, pond_depth);
        if pond_depth > 0.3 {
            base_color = lerp_color(base_color, [0.12, 0.28, 0.35], pond_depth);
        }
    }

    // ========================================================================
    // WETLAND TRANSITION ZONE - flat marshy area between spawn and salt marsh
    // Located immediately south of spawn (z = -60 to z = -140)
    // ========================================================================
    let (wetland_factor, is_wetland_flooded) = calculate_wetland_zone(x, z, seed);
    if wetland_factor > 0.0 {
        // Flatten terrain to near sea level (0.5 - 2.0m)
        let wetland_height = 1.0 + noise_util::fbm(
            Vec2::new(x * 0.01, z * 0.01),
            2, 2.0, 0.5, seed + 450
        ) * 0.5;

        // Blend toward flat wetland
        height = lerp(height, wetland_height, wetland_factor * 0.8);

        // Wetland color: dark muddy green-brown
        let wetland_color = if is_wetland_flooded {
            [0.18, 0.28, 0.22] // Darker, wetter
        } else {
            [0.30, 0.38, 0.25] // Marshy grass
        };
        base_color = lerp_color(base_color, wetland_color, wetland_factor * 0.7);

        // Add subtle hummocks (raised grass clumps)
        let hummock_noise = noise_util::fbm(
            Vec2::new(x * 0.15, z * 0.15),
            2, 2.0, 0.5, seed + 451
        );
        if hummock_noise > 0.5 && !is_wetland_flooded {
            height += (hummock_noise - 0.5) * 0.4 * wetland_factor;
        }
    }

    // Cave entrance near spawn (around -50, 0 to -100, 50)
    let cave_entrance = calculate_cave_entrance(x, z);
    if cave_entrance > 0.0 {
        height = lerp(height, -3.0, cave_entrance);
        base_color = lerp_color(base_color, [0.15, 0.12, 0.10], cave_entrance);
    }

    // Mine entrance on beach (around 150, -30)
    let mine_entrance = calculate_mine_entrance(x, z);
    if mine_entrance > 0.0 {
        height = lerp(height, -4.0, mine_entrance);
        base_color = lerp_color(base_color, [0.25, 0.20, 0.15], mine_entrance);
    }

    // Boulder depressions on beach - waves scour sand around rocks
    // Only apply in beach zone (t = 0.45-0.65) and above water
    if t >= 0.45 && t < 0.65 && height > 0.3 {
        let boulder_depression = calculate_boulder_depression(x, z, seed);
        if boulder_depression > 0.0 {
            height -= boulder_depression;
            // Slightly darker/wetter color in depressions
            let wet_factor = (boulder_depression * 2.0).min(1.0);
            base_color = lerp_color(base_color, [0.45, 0.38, 0.28], wet_factor * 0.3);
        }
    }

    // ========================================================================
    // SPAWN AREA TERRAIN FEATURES
    // Large-scale river valley with natural gradients
    // ========================================================================

    // 1. Marsh zone south of spawn - forces low wet terrain
    let marsh_factor = calculate_marsh_zone(x, z);
    if marsh_factor > 0.0 {
        // Force height to marsh level (0.5-2.0m)
        let marsh_height = lerp(2.0, 0.5, marsh_factor);
        height = lerp(height, marsh_height, marsh_factor);
        // Override color to marsh green-brown
        base_color = lerp_color(base_color, [0.35, 0.42, 0.28], marsh_factor * 0.8);
    }

    // 2. River valley system (large-scale, gradual terrain)
    // ONLY apply to land areas (t > 0.45 means not ocean)
    let valley = calculate_river_valley(x, z, seed);

    // Valley influence: only on land, fades near coast
    let land_factor = ((t - 0.45) / 0.20).clamp(0.0, 1.0);  // 0 at ocean, 1 well inland

    if land_factor > 0.0 {
        // Blend valley elevation with existing terrain
        // Valley influence is strongest near the river, fades with distance
        let valley_influence = if valley.river_factor > 0.0 {
            0.7 * land_factor  // Strong influence in river, but respect coastline
        } else {
            // Gradual influence based on distance from river center
            let dist_factor = 1.0 - (valley.height_mod / 60.0).clamp(0.0, 1.0);
            dist_factor * 0.3 * land_factor
        };

        // Apply valley terrain modification
        height = lerp(height, valley.height_mod, valley_influence);
    }

    // River bed carving (only on land)
    if valley.river_factor > 0.0 && land_factor > 0.5 {
        // River depth: deeper in middle, shallow at edges
        let river_depth = valley.river_factor * 2.5;
        height -= river_depth;

        // River coloring based on type
        if valley.is_waterfall {
            // White churning water
            base_color = lerp_color(base_color, [0.75, 0.85, 0.90], valley.river_factor * 0.7);
        } else if valley.is_rocky {
            // Rocky bed visible through shallow water
            let rock_color = [0.35, 0.32, 0.28];
            let water_color = [0.15, 0.30, 0.38];
            let blended = lerp_color(rock_color, water_color, valley.river_factor);
            base_color = lerp_color(base_color, blended, valley.river_factor * 0.8);
        } else {
            // Normal river water - deeper = darker
            let shallow_color = [0.20, 0.38, 0.42];
            let deep_color = [0.10, 0.25, 0.35];
            let water_color = lerp_color(shallow_color, deep_color, valley.river_factor);
            base_color = lerp_color(base_color, water_color, valley.river_factor * 0.9);
        }
    }

    // Valley walls get slightly different coloring (exposed earth/rock) - only on land
    if land_factor > 0.5 && valley.height_mod > 20.0 && valley.river_factor < 0.1 {
        let elevation_factor = ((valley.height_mod - 20.0) / 30.0).clamp(0.0, 1.0);
        // Rocky outcrops on higher elevations
        base_color = lerp_color(base_color, [0.42, 0.40, 0.35], elevation_factor * 0.3);
    }

    (height, base_color)
}

/// Calculate river depth at a position (0.0 = no river, 1.0 = center of river)
pub fn calculate_river_depth(x: f32, z: f32, seed: u32) -> f32 {
    // Multiple river channels using sine waves with noise perturbation
    let mut max_depth = 0.0f32;

    // River 1: Flows roughly north-south near spawn, curves with noise
    let river1_center_x = -80.0 + noise_util::fbm(Vec2::new(z * 0.003, 0.0), 2, 2.0, 0.5, seed + 200) * 60.0;
    let river1_width = 8.0;
    let dist1 = (x - river1_center_x).abs();
    if dist1 < river1_width {
        let depth1 = 1.0 - (dist1 / river1_width);
        max_depth = max_depth.max(depth1 * depth1); // Quadratic falloff
    }

    // River 2: Flows diagonally, joins river 1
    let river2_base_x = -150.0 + z * 0.4;
    let river2_center_x = river2_base_x + noise_util::fbm(Vec2::new(z * 0.005, 1.0), 2, 2.0, 0.5, seed + 201) * 40.0;
    let river2_width = 6.0;
    let dist2 = (x - river2_center_x).abs();
    // Only active for z < 100
    if dist2 < river2_width && z < 100.0 && z > -200.0 {
        let depth2 = 1.0 - (dist2 / river2_width);
        max_depth = max_depth.max(depth2 * depth2);
    }

    // River 3: Smaller stream near spawn
    let river3_center_z = 40.0 + noise_util::fbm(Vec2::new(x * 0.004, 2.0), 2, 2.0, 0.5, seed + 202) * 30.0;
    let river3_width = 4.0;
    let dist3 = (z - river3_center_z).abs();
    if dist3 < river3_width && x > -150.0 && x < 50.0 {
        let depth3 = 1.0 - (dist3 / river3_width);
        max_depth = max_depth.max(depth3 * depth3);
    }

    max_depth
}

/// Calculate pond depth at a position
fn calculate_pond_depth(x: f32, z: f32, seed: u32) -> f32 {
    let mut max_depth = 0.0f32;

    // ========================================================================
    // FIXED WATER BODIES - strategically placed for gameplay
    // ========================================================================

    // Near spawn area ponds (tutorial/early game)
    let spawn_ponds = [
        (Vec2::new(-40.0, 80.0), 12.0, 1.5),    // Small pond near spawn
        (Vec2::new(-120.0, -50.0), 15.0, 2.0),  // Forest pond
        (Vec2::new(30.0, 120.0), 10.0, 1.2),    // Coastal pond
        (Vec2::new(-200.0, 30.0), 18.0, 2.5),   // Large inland pond
    ];

    // Inland lakes (larger water bodies for exploration)
    let inland_lakes = [
        (Vec2::new(-350.0, 150.0), 35.0, 4.0),   // Large inland lake
        (Vec2::new(-280.0, -100.0), 25.0, 3.0),  // Forest lake
        (Vec2::new(-450.0, -50.0), 28.0, 3.5),   // Western lake
        (Vec2::new(-180.0, 200.0), 20.0, 2.5),   // Northern pond
        (Vec2::new(-400.0, 250.0), 32.0, 4.0),   // Large northern lake
    ];

    // Wetland pools in the transition zone (between spawn and salt marsh)
    let wetland_pools = [
        (Vec2::new(-50.0, -80.0), 8.0, 0.8),     // Shallow wetland pool
        (Vec2::new(-80.0, -100.0), 10.0, 1.0),   //
        (Vec2::new(-30.0, -120.0), 12.0, 1.2),   // Larger wetland pool
        (Vec2::new(-100.0, -130.0), 9.0, 0.9),   //
        (Vec2::new(-60.0, -150.0), 14.0, 1.0),   // Near salt marsh
        (Vec2::new(-20.0, -90.0), 7.0, 0.7),     // Small pool
        (Vec2::new(-130.0, -110.0), 11.0, 1.1),  //
        (Vec2::new(20.0, -100.0), 8.0, 0.8),     // Eastern wetland
    ];

    // Salt marsh tidal pools (shallow, irregular)
    let marsh_pools = [
        (Vec2::new(-40.0, -180.0), 15.0, 0.6),   // Tidal pool 1
        (Vec2::new(30.0, -200.0), 12.0, 0.5),    // Tidal pool 2
        (Vec2::new(-80.0, -220.0), 18.0, 0.7),   // Larger tidal area
        (Vec2::new(10.0, -160.0), 10.0, 0.5),    // Small tidal
        (Vec2::new(-120.0, -190.0), 14.0, 0.6),  //
    ];

    // Process all pond types
    let all_ponds: Vec<(Vec2, f32, f32)> = spawn_ponds.iter()
        .chain(inland_lakes.iter())
        .chain(wetland_pools.iter())
        .chain(marsh_pools.iter())
        .copied()
        .collect();

    for (center, radius, max_pond_depth) in all_ponds {
        let dist = Vec2::new(x, z).distance(center);
        if dist < radius * 1.3 {
            // Smooth edges with noise for natural shoreline
            let edge_noise = noise_util::fbm(
                Vec2::new(x * 0.1 + center.x * 0.01, z * 0.1 + center.y * 0.01),
                2, 2.0, 0.5, seed + 300
            ) * (radius * 0.2);
            let effective_radius = radius + edge_noise;

            if dist < effective_radius {
                let normalized = dist / effective_radius;
                // Smooth bowl shape: steep sides, flat bottom
                let depth_curve = 1.0 - (normalized * normalized);
                let depth = depth_curve * (max_pond_depth / 4.0); // Normalize to 0-1 range
                max_depth = max_depth.max(depth);
            }
        }
    }

    // Additional procedural ponds based on noise (smaller random pools)
    let pond_noise = noise_util::fbm(Vec2::new(x * 0.02, z * 0.02), 3, 2.0, 0.5, seed + 301);
    if pond_noise > 0.65 {
        let pond_strength = (pond_noise - 0.65) / 0.35;
        max_depth = max_depth.max(pond_strength * 0.5);
    }

    max_depth
}

/// Calculate wetland transition zone - flat marshy area between spawn and salt marsh
/// Returns (wetland_factor, is_flooded)
pub fn calculate_wetland_zone(x: f32, z: f32, seed: u32) -> (f32, bool) {
    // Wetland transition zone: roughly z = -60 to z = -140
    // Centered at x = -50, extends from x = -200 to x = 100
    let wetland_center = Vec2::new(-50.0, -100.0);
    let wetland_width = 300.0;  // X extent
    let wetland_depth = 80.0;   // Z extent

    // Distance from wetland center (elliptical)
    let dx = (x - wetland_center.x) / wetland_width;
    let dz = (z - wetland_center.y) / wetland_depth;
    let dist = (dx * dx + dz * dz).sqrt();

    if dist > 1.0 {
        return (0.0, false);
    }

    // Smooth falloff at edges
    let base_factor = 1.0 - (dist * dist);

    // Add noise for irregular edges
    let edge_noise = noise_util::fbm(
        Vec2::new(x * 0.02, z * 0.02),
        3, 2.0, 0.5, seed + 400
    ) * 0.3;

    let wetland_factor = (base_factor + edge_noise).clamp(0.0, 1.0);

    // Flooded areas within wetland (standing water)
    let flood_noise = noise_util::fbm(
        Vec2::new(x * 0.05, z * 0.05),
        2, 2.0, 0.5, seed + 401
    );
    let is_flooded = wetland_factor > 0.3 && flood_noise > 0.2;

    (wetland_factor, is_flooded)
}

/// Get all static water body definitions for rendering
/// Returns Vec of (center, radius, depth, water_type)
/// water_type: 0 = pond, 1 = lake, 2 = wetland, 3 = marsh
pub fn get_water_bodies() -> Vec<(Vec2, f32, f32, u32)> {
    let mut bodies = Vec::new();

    // Spawn ponds (type 0)
    bodies.push((Vec2::new(-40.0, 80.0), 12.0, 1.5, 0));
    bodies.push((Vec2::new(-120.0, -50.0), 15.0, 2.0, 0));
    bodies.push((Vec2::new(30.0, 120.0), 10.0, 1.2, 0));
    bodies.push((Vec2::new(-200.0, 30.0), 18.0, 2.5, 0));

    // Inland lakes (type 1)
    bodies.push((Vec2::new(-350.0, 150.0), 35.0, 4.0, 1));
    bodies.push((Vec2::new(-280.0, -100.0), 25.0, 3.0, 1));
    bodies.push((Vec2::new(-450.0, -50.0), 28.0, 3.5, 1));
    bodies.push((Vec2::new(-180.0, 200.0), 20.0, 2.5, 1));
    bodies.push((Vec2::new(-400.0, 250.0), 32.0, 4.0, 1));

    // Wetland pools (type 2)
    bodies.push((Vec2::new(-50.0, -80.0), 8.0, 0.8, 2));
    bodies.push((Vec2::new(-80.0, -100.0), 10.0, 1.0, 2));
    bodies.push((Vec2::new(-30.0, -120.0), 12.0, 1.2, 2));
    bodies.push((Vec2::new(-100.0, -130.0), 9.0, 0.9, 2));
    bodies.push((Vec2::new(-60.0, -150.0), 14.0, 1.0, 2));
    bodies.push((Vec2::new(-20.0, -90.0), 7.0, 0.7, 2));
    bodies.push((Vec2::new(-130.0, -110.0), 11.0, 1.1, 2));
    bodies.push((Vec2::new(20.0, -100.0), 8.0, 0.8, 2));

    // Salt marsh tidal pools (type 3)
    bodies.push((Vec2::new(-40.0, -180.0), 15.0, 0.6, 3));
    bodies.push((Vec2::new(30.0, -200.0), 12.0, 0.5, 3));
    bodies.push((Vec2::new(-80.0, -220.0), 18.0, 0.7, 3));
    bodies.push((Vec2::new(10.0, -160.0), 10.0, 0.5, 3));
    bodies.push((Vec2::new(-120.0, -190.0), 14.0, 0.6, 3));

    bodies
}

/// Calculate cave entrance depression
fn calculate_cave_entrance(x: f32, z: f32) -> f32 {
    // Cave entrance at approximately (-70, 25)
    let cave_center = Vec2::new(-70.0, 25.0);
    let cave_radius = 8.0;
    let dist = Vec2::new(x, z).distance(cave_center);

    if dist < cave_radius {
        let depth = 1.0 - (dist / cave_radius);
        depth * depth // Quadratic for bowl shape
    } else {
        0.0
    }
}

/// Calculate mine entrance on beach
fn calculate_mine_entrance(x: f32, z: f32) -> f32 {
    // Mine entrance at approximately (120, -20) on the beach
    let mine_center = Vec2::new(120.0, -20.0);
    let mine_radius = 5.0;
    let dist = Vec2::new(x, z).distance(mine_center);

    if dist < mine_radius {
        let depth = 1.0 - (dist / mine_radius);
        depth * depth
    } else {
        0.0
    }
}

/// Calculate sand depression around beach boulders
/// Creates bowl-shaped depressions where waves have scoured sand around rocks
/// Uses same noise seed as boulder placement for consistency
fn calculate_boulder_depression(x: f32, z: f32, seed: u32) -> f32 {
    // Same noise as boulder placement in rocks.rs Phase 5 (seed + 888)
    let boulder_noise = noise_util::fbm(
        Vec2::new(x * 0.06, z * 0.06),
        2, 2.0, 0.5, seed + 888
    );

    // Check if a boulder would spawn near here
    // Boulder spawn threshold from rocks.rs: size_roll > 0.85 for large
    // We create depressions at all potential boulder spots (above noise threshold)
    if boulder_noise > 0.4 {
        // Depression strength increases with boulder likelihood
        let depression_strength = (boulder_noise - 0.4) / 0.6; // 0.0 to 1.0

        // Create radial bowl shape - depression is strongest at center
        // Secondary noise for radial variation
        let radial_noise = noise_util::fbm(
            Vec2::new(x * 0.3, z * 0.3),
            2, 2.0, 0.5, seed + 889
        );

        // Depression depth: up to 0.4m deep for large boulders
        let depth = depression_strength * 0.4 * (1.0 + radial_noise * 0.3);

        // Ring effect - slight raised rim at edge (wave-deposited sand)
        let rim_noise = noise_util::fbm(
            Vec2::new(x * 0.15, z * 0.15),
            1, 2.0, 0.5, seed + 890
        );
        let rim_height = if rim_noise > 0.3 && depression_strength < 0.5 {
            (rim_noise - 0.3) * 0.1
        } else {
            0.0
        };

        return depth - rim_height;
    }

    0.0
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

/// Calculate the approximate distance from a point to the nearest shoreline.
///
/// The shoreline is defined as where terrain height transitions from water (< 0.5m) to land.
/// This function samples in the ocean direction (+X gradient) to find the shoreline.
///
/// # Returns
/// - Positive value: distance to shoreline (point is inland)
/// - Zero or negative: point is at or in the water
///
/// # Algorithm
/// 1. If current point is underwater, return 0
/// 2. March toward ocean (+X direction with noise influence) until water is found
/// 3. Binary search to refine shoreline position
/// 4. Return Euclidean distance to that point
pub fn distance_to_shoreline(x: f32, z: f32, seed: u32) -> f32 {
    let (height, _) = get_height_at(x, z, seed);

    // Water threshold - below this is considered water
    const WATER_LEVEL: f32 = 0.5;

    // If we're already in water, distance is 0
    if height < WATER_LEVEL {
        return 0.0;
    }

    // March toward ocean (+X direction) to find shoreline
    // The gradient is -x * 0.001, so +X = more ocean
    let max_search_dist = 500.0; // Don't search beyond 500m
    let step_size = 10.0; // Initial coarse step

    let mut search_x = x;
    let mut prev_x = x;
    let mut found_water = false;

    // Coarse search: find where water starts
    while search_x < x + max_search_dist {
        let (h, _) = get_height_at(search_x, z, seed);
        if h < WATER_LEVEL {
            found_water = true;
            break;
        }
        prev_x = search_x;
        search_x += step_size;
    }

    // If no water found within search distance, return max distance
    if !found_water {
        return max_search_dist;
    }

    // Binary search to refine shoreline position
    let mut low = prev_x;
    let mut high = search_x;
    for _ in 0..8 {
        let mid = (low + high) * 0.5;
        let (h, _) = get_height_at(mid, z, seed);
        if h < WATER_LEVEL {
            high = mid;
        } else {
            low = mid;
        }
    }

    // Shoreline is approximately at 'high'
    let shoreline_x = high;

    // Return distance from original point to shoreline
    // Since we only searched in X, this is just the X difference
    // For more accuracy, we could search in multiple directions
    (shoreline_x - x).abs()
}

/// Get the biome "t" value at a position (0.0-1.0 scale from ocean to deep forest)
/// Useful for determining spawn zones without full height calculation.
///
/// # Biome Zones (Updated v2)
/// - t < 0.45: Ocean
/// - t 0.45-0.65: Beach/Sand (expanded 5x)
/// - t 0.65-0.72: Forest-Edge/Treeline (very dense)
/// - t 0.72-0.82: Coastal Forest (flat)
/// - t > 0.82: Inland Forest (rolling hills, birch)
pub fn get_biome_t(x: f32, z: f32, seed: u32) -> f32 {
    let biome_scale = 0.002;
    let biome_noise = noise_util::fbm(
        Vec2::new(x * biome_scale, z * biome_scale),
        3, 2.0, 0.5, seed + 100
    );
    let noise_norm = (biome_noise + 1.0) * 0.5;
    let gradient = -x * 0.001;
    (noise_norm * 0.3 + gradient + 0.5).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_generation() {
        let (positions, colors, normals, indices) = generate_terrain_chunk(1587, 64, 0, 0, 1.0);

        // Verify dimensions
        assert_eq!(positions.len(), 65 * 65);
        assert_eq!(colors.len(), 65 * 65);
        assert_eq!(normals.len(), 65 * 65);
        assert_eq!(indices.len(), 64 * 64 * 2 * 3);
    }

    #[test]
    fn test_small_mesh() {
        let (positions, colors, normals, indices) = generate_terrain_chunk(42, 4, 0, 0, 1.0);

        // 5x5 grid = 25 vertices
        assert_eq!(positions.len(), 25);
        assert_eq!(colors.len(), 25);
        assert_eq!(normals.len(), 25);

        // 4x4 quads = 32 triangles = 96 indices
        assert_eq!(indices.len(), 96);
    }

    #[test]
    fn test_eastern_sea_gradient() {
        // Generate West Chunk (Spawn)
        let (west_pos, _, _, _) = generate_terrain_chunk(12345, 64, 0, 0, 1.0);

        // Generate East Chunk (Far East)
        let (east_pos, _, _, _) = generate_terrain_chunk(12345, 64, 1000, 0, 1.0);
        
        // Calculate average height
        let west_avg: f32 = west_pos.iter().map(|p| p[1]).sum::<f32>() / west_pos.len() as f32;
        let east_avg: f32 = east_pos.iter().map(|p| p[1]).sum::<f32>() / east_pos.len() as f32;

        println!("West Avg Height: {}, East Avg Height: {}", west_avg, east_avg);

        // The East side should be lower (Ocean)
        assert!(east_avg < west_avg, "East side should be lower than West side due to gradient");
    }
}

/// Generate detritus (logs, driftwood, dead trees) for a chunk
/// Returns (positions, normals, uvs, indices)
pub fn generate_detritus_for_chunk(
    seed: u32,
    size: f32,
    offset_x: f32,
    offset_z: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut index_offset = 0;

    // Use a fixed grid for potential spawn points
    let grid_step = 4.0; // Check every 4 meters
    let steps = (size / grid_step) as i32;

    for z in 0..steps {
        for x in 0..steps {
            let global_x = offset_x + (x as f32 * grid_step);
            let global_z = offset_z + (z as f32 * grid_step);

            // Add some jitter to position
            let jitter_x = noise_util::hash(seed + (global_x as u32) * 73856093) * 3.0;
            let jitter_z = noise_util::hash(seed + (global_z as u32) * 19349663) * 3.0;
            let px = global_x + jitter_x;
            let pz = global_z + jitter_z;

            // Get biome info
            // Replicating get_height_at logic partially to get 't'
            let biome_scale = 0.002;
            let biome_noise = noise_util::fbm(
                Vec2::new(px * biome_scale, pz * biome_scale),
                3, 2.0, 0.5, seed + 100
            );
            let noise_norm = (biome_noise + 1.0) * 0.5;
            let gradient = -px * 0.001; 
            let t = (noise_norm * 0.3 + gradient + 0.5).clamp(0.0, 1.0);

            let (terrain_height, _) = get_height_at(px, pz, seed);

            // Spawn Logic based on Biome
            let spawn_chance = noise_util::hash(seed + (px as u32) ^ (pz as u32));
            
            if t < 0.45 {
                // Ocean / Shallow Water (Inlets)
                // Spawn small sticks poking up from shallow water
                if terrain_height > -2.0 && terrain_height < 0.5 && spawn_chance > 0.95 {
                    // Small stick (Vertical) - reduced from 0.3 radius logs
                    add_cylinder(
                        &mut positions, &mut normals, &mut uvs, &mut indices, &mut index_offset,
                        Vec3::new(px, terrain_height, pz),
                        0.02, // Radius (was 0.3 - now thin stick)
                        0.4 + spawn_chance * 0.4, // Height 0.4-0.8m (was 4-7m)
                        Vec3::Y, // Up
                        4 // Segments (was 8)
                    );
                }
            } else if t < 0.55 {
                // Beach
                // Spawn tiny driftwood sticks
                if spawn_chance > 0.92 {
                    // Small driftwood stick (random orientation)
                    let rot_x = (spawn_chance * 10.0).sin();
                    let rot_z = (spawn_chance * 10.0).cos();
                    let axis = Vec3::new(rot_x, 0.1, rot_z).normalize();

                    add_cylinder(
                        &mut positions, &mut normals, &mut uvs, &mut indices, &mut index_offset,
                        Vec3::new(px, terrain_height + 0.02, pz),
                        0.015, // Radius (was 0.1 - now tiny stick)
                        0.25 + spawn_chance * 0.2, // Length 0.25-0.45m (was 1.5m)
                        axis,
                        4 // Segments (was 6)
                    );
                }
            } else if t > 0.75 {
                // Forest
                // Spawn small fallen twigs/sticks (high-fidelity logs use dead_log_0 model)
                if spawn_chance > 0.97 {
                    // Small twig (Horizontal)
                    let angle = spawn_chance * std::f32::consts::PI * 2.0;
                    let axis = Vec3::new(angle.cos(), 0.0, angle.sin());

                    add_cylinder(
                        &mut positions, &mut normals, &mut uvs, &mut indices, &mut index_offset,
                        Vec3::new(px, terrain_height + 0.02, pz),
                        0.02, // Radius (was 0.4 - now small twig)
                        0.3 + spawn_chance * 0.3, // Length 0.3-0.6m (was 3-5m)
                        axis,
                        4 // Segments (was 8)
                    );
                }
            }
        }
    }

    (positions, normals, uvs, indices)
}

/// Helper to add a cylinder mesh
fn add_cylinder(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    index_offset: &mut u32,
    center: Vec3,
    radius: f32,
    length: f32,
    axis: Vec3,
    segments: u32,
) {
    // Basis vectors for the cylinder cap
    let up = axis.normalize();
    let arbitrary = if up.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
    let right = up.cross(arbitrary).normalize();
    let forward = up.cross(right).normalize();

    let half_len = length * 0.5;
    let start = center - up * half_len;
    let end = center + up * half_len;

    // Generate vertices for the side
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
        let x = angle.cos();
        let z = angle.sin();

        let normal = (right * x + forward * z).normalize();
        let offset = normal * radius;

        // Bottom vertex
        positions.push((start + offset).to_array());
        normals.push(normal.to_array());
        uvs.push([i as f32 / segments as f32, 0.0]);

        // Top vertex
        positions.push((end + offset).to_array());
        normals.push(normal.to_array());
        uvs.push([i as f32 / segments as f32, 1.0]);
    }

    // Generate indices
    for i in 0..segments {
        let base = *index_offset + i * 2;
        
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);

        indices.push(base + 1);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    *index_offset += (segments + 1) * 2;
}
