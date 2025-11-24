use croatoan_procgen::{generate_grass_blade, GrassBladeRecipe};
use glam::Vec3;

fn main() {
    println!("=== Procedural Grass Generation Demo ===\n");

    // Create a grass blade recipe
    let recipe = GrassBladeRecipe {
        height_range: (0.3, 0.8),
        blade_segments: 4,
        curve_factor: 0.3,
        width_base: 0.04,
        width_tip: 0.01,
        color_base: [0.2, 0.5, 0.1],
        color_tip: [0.3, 0.7, 0.2],
    };

    println!("Recipe:");
    println!("  Height Range: {:?}", recipe.height_range);
    println!("  Segments: {}", recipe.blade_segments);
    println!("  Curve Factor: {}", recipe.curve_factor);
    println!("  Width: {} -> {}", recipe.width_base, recipe.width_tip);
    println!();

    // Generate a single grass blade
    let blade = generate_grass_blade(&recipe, 42, Vec3::new(0.0, 0.0, 0.0));

    println!("Generated Grass Blade:");
    println!("  Vertices: {}", blade.positions.len());
    println!("  Triangles: {}", blade.indices.len() / 3);
    println!("  Memory (approx): {} bytes",
        blade.positions.len() * 24 + blade.indices.len() * 4);
    println!();

    // Compare to OBJ file size
    let obj_estimate = blade.positions.len() * 100; // Rough estimate for text format
    println!("Comparison:");
    println!("  Recipe size: ~200 bytes (can generate infinite variations)");
    println!("  Single blade mesh: ~{} bytes in memory",
        blade.positions.len() * 24 + blade.indices.len() * 4);
    println!("  Equivalent OBJ file: ~{} bytes", obj_estimate);
    println!();

    // Simulate a patch
    let blades_per_chunk = 1000;
    let total_verts = blade.positions.len() * blades_per_chunk;
    let total_tris = (blade.indices.len() / 3) * blades_per_chunk;
    let memory_kb = (total_verts * 24 + blade.indices.len() * 4 * blades_per_chunk) / 1024;

    println!("Grass Patch (1000 blades):");
    println!("  Total Vertices: {}", total_verts);
    println!("  Total Triangles: {}", total_tris);
    println!("  Memory: ~{} KB", memory_kb);
    println!("  Recipe + Generation Code: < 1 KB");
    println!();

    println!("✅ This tiny recipe file can generate photorealistic grass!");
    println!("✅ No need for Git LFS!");
    println!("✅ Infinite variation with different seeds!");
}
