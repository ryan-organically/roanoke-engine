# Procedural Generation Framework
## Trillion-Dollar Techniques for Git-Friendly Asset Generation

This document outlines the algorithmic frameworks we'll use to generate photorealistic natural and architectural objects without storing massive asset files.

---

## 🌲 Trees - L-Systems (Lindenmayer Systems)

### Algorithm Overview
L-Systems use recursive string rewriting to generate branching structures. A simple grammar produces complex organic trees.

... (Previous sections remain) ...

---

## 🪨 Rocks - Voronoi Fracturing + Noise Displacement

**(Implemented in `crates/croatoan_procgen/src/rock.rs`)**

### Algorithm Used
1.  **Base Shape**: Icosphere (subdivided icosahedron) or Cube.
2.  **Displacement**: Perlin noise displacement along normals.
3.  **Deformation**: Scaling and flattening for specific types (boulders, river stones).
4.  **Material**: Currently vertex normals, planned PBR texture support.

### Recipe Structure
```rust
struct RockRecipe {
    rock_type: RockType,        // Boulder, RiverStone, SharpRock
    base_size: Vec3,            // Scaling factor
    subdivision_levels: u32,    // Detail level
    roughness: f32,             // Noise amplitude
    deformation: f32,           // Shape distortion
}
```

---

## 🏛️ Buildings - Shape Grammars

**(Implemented in `crates/croatoan_procgen/src/building.rs`)**

### Algorithm Used
A simplified Shape Grammar approach:
1.  **Foundation**: Base block.
2.  **Floors**: Stacked blocks with parameterized height and material.
3.  **Facade**: Boolean operations (simulated) to add windows and doors.
4.  **Roof**: Procedural prism (pitched) or flat block based on style.

### Recipe Structure
```rust
struct BuildingRecipe {
    style: ArchStyle,          // Colonial, Modern, Rustic
    floors: u32,
    width: f32,
    depth: f32,
    floor_height: f32,
    roof_height: f32,
}
```

---

## 🦌 Animals - Parametric Skeletons + Metaballs

**(Next Priority)**

... (Rest of document) ...