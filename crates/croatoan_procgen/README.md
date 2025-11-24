# Croatoan Procgen

Procedural mesh generation system for the Roanoke Engine. This crate enables generation of natural objects (grass, trees, rocks, etc.) without requiring large asset files, keeping your repository Git-friendly.

## Philosophy

Instead of storing millions of vertices in OBJ files, we store **generation recipes** - tiny parameter files that can create infinite variations of natural objects procedurally.

### Benefits

- **Tiny file sizes**: Recipes are < 1KB vs 50MB+ for OBJ files
- **No Git LFS needed**: All assets stay well under GitHub's 100MB limit
- **Infinite variation**: Different seeds produce unique objects
- **Memory efficient**: Generate on-demand, discard when not visible
- **LOD-friendly**: Generate simpler meshes when far away

## Current Features

### Grass Generation

Procedural grass blade generation with:
- Curved ribbon geometry (quadratic bending)
- Segment-based construction (configurable detail)
- Width tapering from base to tip
- Color gradients
- Wind animation (via shader)
- Biome-aware spawning

#### Example Usage

```rust
use croatoan_procgen::{GrassBladeRecipe, generate_grass_blade};
use glam::Vec3;

// Define grass properties
let recipe = GrassBladeRecipe {
    height_range: (0.3, 0.8),
    blade_segments: 4,
    curve_factor: 0.3,
    width_base: 0.04,
    width_tip: 0.01,
    color_base: [0.2, 0.5, 0.1],
    color_tip: [0.3, 0.7, 0.2],
};

// Generate a single blade
let blade = generate_grass_blade(&recipe, 42, Vec3::ZERO);
// Returns: positions, colors, indices
```

#### Grass Patch Generation

```rust
use croatoan_procgen::generate_grass_patch;

let (positions, colors, indices) = generate_grass_patch(
    &recipe,
    seed,
    (chunk_x, chunk_z),
    chunk_size,
    density,  // blades per square unit
    |x, z| get_terrain_height(x, z),  // terrain height function
    |x, z| is_grassland_biome(x, z),  // biome filter
);
```

## File Size Comparison

| Method | File Size | Variation | Git-Friendly |
|--------|-----------|-----------|--------------|
| **OBJ File (1000 blades)** | ~50 MB | None (fixed) | ❌ Needs LFS |
| **Procedural Recipe** | < 1 KB | Infinite | ✅ Yes |

### Example: Grass Patch

- **Recipe**: 200 bytes
- **Generated mesh (1000 blades)**: ~328 KB in memory
- **Equivalent OBJ file**: ~50 MB
- **GitHub storage**: Recipe only (< 1 KB)

## Roadmap

### Planned Features

1. **Trees** (L-System based)
   - Branching algorithms
   - Procedural bark
   - Leaf placement
   - Species variation

2. **Rocks** (Voronoi/Noise based)
   - Size variation
   - Weathering parameters
   - Cliff generation

3. **Characters/Animals**
   - Parametric skeletons
   - Procedural rigging
   - Body part variation

4. **Buildings**
   - Modular components
   - Architectural styles
   - Procedural details

## Integration with Roanoke Engine

The `croatoan_wfc` crate provides biome-aware vegetation generation:

```rust
use croatoan_wfc::generate_vegetation_for_chunk;

let (grass_positions, grass_colors, grass_indices) =
    generate_vegetation_for_chunk(seed, chunk_size, offset_x, offset_z);
```

This automatically:
- Queries terrain height
- Filters by biome (no grass underwater, sparse in forests, etc.)
- Generates appropriate density

## Rendering

Grass is rendered via `croatoan_render::GrassPipeline`:

- Vertex shader applies wind animation
- Fragment shader applies simple lighting
- No backface culling (grass visible from both sides)
- Alpha blending supported

Shader location: `assets/shaders/grass.wgsl`

## Performance

### Grass Rendering

- **1000 blades**: ~10,000 vertices, 8,000 triangles
- **Generation time**: < 1ms
- **Memory**: ~328 KB per 1000 blades

Future optimizations:
- Instanced rendering (single blade mesh, thousands of instances)
- GPU-based generation (compute shaders)
- LOD system (fewer segments when distant)

## Examples

Run the demo:

```bash
cargo run --package croatoan_procgen --example grass_demo
```

## Technical Details

### Grass Blade Geometry

Each blade is a curved ribbon with:
- `(segments + 1) * 2` vertices (left and right edges)
- `segments * 2` triangles
- Quadratic curve based on random direction
- Perpendicular width calculation for ribbon shape

### Wind Animation

Implemented in shader using sine waves:
- Multiple frequencies for organic motion
- Height-based falloff (base stays fixed)
- World position used as pseudo-time (static animation)

Future: Add time uniform for true dynamic wind

## Contributing

When adding new procedural generators:

1. Create a new module in `src/`
2. Define a `Recipe` struct for parameters
3. Implement generation function
4. Add tests
5. Create an example
6. Document in this README

## License

Part of the Roanoke Engine project.
