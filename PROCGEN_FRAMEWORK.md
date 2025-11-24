# Procedural Generation Framework
## Trillion-Dollar Techniques for Git-Friendly Asset Generation

This document outlines the algorithmic frameworks we'll use to generate photorealistic natural and architectural objects without storing massive asset files.

---

## 🌲 Trees - L-Systems (Lindenmayer Systems)

### Algorithm Overview
L-Systems use recursive string rewriting to generate branching structures. A simple grammar produces complex organic trees.

### Grammar Example
```
Axiom: F
Rules:
  F → F[+F]F[-F]F   (Branch with left/right splits)
  + → Rotate right
  - → Rotate left
  [ → Push state (save position/angle)
  ] → Pop state (return to saved point)
```

### Recipe Structure
```rust
struct TreeRecipe {
    axiom: String,              // "F"
    rules: HashMap<char, String>, // F → "F[+F]F[-F]F"
    iterations: u32,            // 4-6 for detailed trees
    angle: f32,                 // 25.5 degrees
    length_decay: f32,          // 0.7 (branches get shorter)
    thickness_decay: f32,       // 0.6 (branches get thinner)
    leaf_probability: f32,      // 0.3 at terminals
    species: TreeSpecies,       // Oak, Pine, Birch, etc.
}
```

### Species Variation
- **Oak**: Wide angle (30°), thick trunk, bushy
- **Pine**: Narrow angle (15°), tall, conical
- **Willow**: Downward gravity modifier, drooping branches
- **Palm**: Single trunk, terminal frond cluster

### Implementation Plan
1. String parser for L-System grammar
2. Turtle graphics interpreter (position, heading, stack)
3. Mesh generation from path (cylinders for branches)
4. Leaf placement at terminals
5. Bark texture coordinate generation
6. Wind animation (vertex shader, hierarchical sway)

### File Size
- **Recipe**: ~500 bytes
- **Generated tree**: 50K-200K triangles
- **Equivalent OBJ**: 20-100 MB
- **GitHub storage**: Recipe only (~500 bytes)

---

## 🪨 Rocks - Voronoi Fracturing + Noise Displacement

### Algorithm Overview
1. Start with base primitive (sphere, cube, or random convex hull)
2. Apply Voronoi fracturing for angular facets
3. Displace vertices with multi-octave noise
4. Erosion simulation (optional, for weathering)

### Recipe Structure
```rust
struct RockRecipe {
    base_shape: RockBase,       // Sphere, Cube, Boulder
    voronoi_points: u32,        // 20-100 (more = more facets)
    noise_octaves: u32,         // 4-6
    noise_scale: f32,           // 0.5-2.0
    noise_strength: f32,        // 0.3 (displacement amount)
    erosion_factor: f32,        // 0.0-1.0 (smooths edges)
    size_range: (f32, f32),     // (0.5, 3.0) meters
    color_base: [f32; 3],       // Gray, brown, etc.
    moss_coverage: f32,         // 0.0-1.0 (for forest rocks)
}
```

### Voronoi Fracturing Steps
1. Scatter N random points in 3D space
2. For each vertex, find closest Voronoi point
3. Calculate distance to Voronoi boundaries
4. Create angular facets (not smooth)
5. Optionally subdivide large faces

### Noise Displacement
- **Perlin noise**: Smooth, organic (boulders)
- **Ridged noise**: Sharp edges (cliff faces)
- **Cellular noise**: Crystal-like (fantasy crystals)

### Weathering Simulation
- Erosion: Smooth vertices near edges (high curvature)
- Cracks: Add thin grooves using noise
- Moss: Green color in crevices (high ambient occlusion)

### File Size
- **Recipe**: ~300 bytes
- **Generated rock**: 5K-20K triangles
- **Equivalent OBJ**: 2-10 MB
- **GitHub storage**: Recipe only (~300 bytes)

---

## 🦌 Animals - Parametric Skeletons + Metaballs

### Algorithm Overview
1. Define skeletal hierarchy (bone chains)
2. Place metaballs at joints and along bones
3. Use marching cubes to generate surface mesh
4. Rig mesh to skeleton automatically
5. Generate animation cycles procedurally

### Recipe Structure
```rust
struct AnimalRecipe {
    species: AnimalType,        // Deer, Bird, Fish, etc.
    skeleton: SkeletonDef,      // Bone hierarchy
    body_proportions: BodyRatios, // Leg length, torso size, etc.
    metaball_radii: Vec<f32>,   // Size at each joint
    skin_smoothness: f32,       // Metaball threshold
    fur_density: f32,           // Hair particles (optional)
    color_pattern: ColorScheme, // Spots, stripes, solid
    animation_style: AnimStyle, // Walk cycle parameters
}

struct SkeletonDef {
    bones: Vec<Bone>,
    // Example deer skeleton:
    // Spine (5 segments), Neck (3), Head (1)
    // Legs (4x): Thigh, Shin, Foot (3 bones each)
    // Tail (optional, 4 segments)
}
```

### Metaball Surface Generation
Metaballs create organic blobs that blend smoothly. Formula:
```
f(x,y,z) = Σ (r_i^2 / distance_to_ball_i^2)
```
Use marching cubes to extract isosurface where `f(x,y,z) = threshold`.

### Parametric Body Proportions
```rust
struct BodyRatios {
    leg_length: f32,      // 0.8-1.2 (relative to torso)
    neck_length: f32,     // 0.5-1.5 (giraffe vs bear)
    head_size: f32,       // 0.8-1.2
    torso_width: f32,     // 0.9-1.1
    tail_length: f32,     // 0.0-2.0
}
```
**Same skeleton, different proportions = different species!**

### Animation Generation
Procedural walk cycles using inverse kinematics (IK):
1. Define foot placement pattern
2. Solve IK for leg bones (foot reaches ground)
3. Spine follows center of mass
4. Add secondary motion (head bob, tail sway)

### File Size
- **Recipe**: ~1-2 KB
- **Generated animal**: 30K-100K triangles
- **Equivalent OBJ**: 15-50 MB
- **GitHub storage**: Recipe only (~1-2 KB)

---

## 🏛️ Buildings - Modular Grammar Systems

### Algorithm Overview
Similar to L-Systems but in 3D space with architectural rules.

### Shape Grammar Example
```
Building → Foundation + Walls + Roof
Walls → Wall | Wall + Wall (recursive subdivision)
Wall → Window | Door | Solid | Balcony
Roof → Flat | Pitched | Dome
```

### Recipe Structure
```rust
struct BuildingRecipe {
    style: ArchStyle,          // Colonial, Modern, Fantasy, etc.
    floors: u32,               // 1-5
    footprint: (f32, f32),     // Width x Depth
    wall_grammar: GrammarRules,
    window_spacing: f32,       // 2.0-4.0 meters
    door_width: f32,           // 1.0-2.0 meters
    roof_style: RoofType,
    material_palette: Materials,
    decoration_density: f32,   // Trim, columns, etc.
}

enum ArchStyle {
    Colonial,     // Symmetrical, brick, shutters
    Modern,       // Glass, flat roof, minimal
    Fantasy,      // Irregular, towers, arches
    Industrial,   // Metal, exposed structure
    Rustic,       // Wood, stone, asymmetrical
}
```

### Procedural Details
1. **Windows**: Subdivide walls, place at intervals
2. **Doors**: Ground floor, centered or offset
3. **Trim**: Extrude edges for molding
4. **Columns**: Cylindrical supports (Doric, Ionic, Corinthian)
5. **Roofs**: Pitched (slope), flat, mansard, dome
6. **Textures**: Brick, siding, stucco (procedural)

### Architectural Rules (Grammar Constraints)
- Windows must be above ground
- Doors require adjacent ground
- Roofs must cover all walls
- Floors align vertically
- Structural support (columns/walls every N meters)

### Roanoke-Specific: Colonial Architecture
```rust
impl RoanokeColonialRecipe {
    floors: 1-2,
    wall_material: Wood("Clapboard"),
    roof_type: Pitched(35°),  // degrees
    windows: Sash(6, 6),      // 6x6 panes
    door: Panel(4),           // 4-panel door
    chimney: Brick,
    foundation: Stone,
    porch: Optional(true),
}
```

### File Size
- **Recipe**: ~1 KB
- **Generated building**: 50K-200K triangles
- **Equivalent OBJ**: 25-100 MB
- **GitHub storage**: Recipe only (~1 KB)

---

## 🌾 Grass (Already Implemented!) ✅

### Algorithm: Curved Ribbon
- Quadratic bezier curve
- Tapering width
- Multi-segment for flexibility
- Wind via vertex shader

**Recipe**: 200 bytes → Infinite blades

---

## 🪨 Insects - Segmented Exoskeletons

### Algorithm Overview
Similar to animals but using rigid segments instead of smooth metaballs.

### Recipe Structure
```rust
struct InsectRecipe {
    species: InsectType,       // Beetle, Ant, Butterfly, etc.
    segment_count: u32,        // 3 (head, thorax, abdomen)
    leg_count: u32,            // 6 (insects), 8 (spiders)
    leg_segments: u32,         // 3-4 per leg
    antenna_length: f32,
    wing_type: WingStyle,      // None, Shell, Transparent, etc.
    exoskeleton_sheen: f32,    // Metallic shine (beetles)
    size: f32,                 // 0.01-0.05 meters
}
```

### Segmented Mesh Generation
1. Create ellipsoids for body segments
2. Generate legs as tapered cylinders
3. Wings as thin quads with venation texture
4. Antennae as curved splines
5. Eyes as spherical facets

### Animation
- Walk cycle: Tripod gait (3 legs alternate)
- Flight: Wing oscillation (sine wave)
- Idle: Antenna twitch, mandible movement

### File Size
- **Recipe**: ~400 bytes
- **Generated insect**: 2K-10K triangles
- **GitHub storage**: Recipe only (~400 bytes)

---

## 🧬 Biodiversity Through Parameters

The beauty of procedural generation: **Same algorithm, different parameters = different species!**

### Example: Trees
```rust
// Oak
TreeRecipe { angle: 30.0, iterations: 5, thickness: 0.6, ... }

// Pine
TreeRecipe { angle: 15.0, iterations: 6, thickness: 0.7, ... }

// Willow
TreeRecipe { angle: 25.0, gravity: -0.5, iterations: 5, ... }
```

**3 recipes (~1.5 KB) → Infinite forest variety**

### Example: Animals
```rust
// Deer
AnimalRecipe { leg_length: 1.1, neck_length: 1.2, ... }

// Bear
AnimalRecipe { leg_length: 0.9, neck_length: 0.6, torso_width: 1.3, ... }

// Giraffe
AnimalRecipe { leg_length: 1.5, neck_length: 2.5, ... }
```

**Same skeleton code, different proportions!**

---

## 📊 Total Storage Comparison

| Asset Type | Traditional OBJ | Procgen Recipe | Savings |
|------------|----------------|----------------|---------|
| **Grass (1000)** | 50 MB | < 1 KB | 50,000x |
| **Tree** | 50 MB | ~500 B | 100,000x |
| **Rock** | 5 MB | ~300 B | 16,666x |
| **Animal** | 30 MB | ~1 KB | 30,000x |
| **Building** | 75 MB | ~1 KB | 75,000x |
| **Insect** | 2 MB | ~400 B | 5,000x |

### Entire Game World
- **Traditional**: 10GB+ (needs Git LFS, slow clone)
- **Procedural**: < 10 MB (pure text files, instant clone)

---

## 🔬 Academic References

1. **L-Systems**: Prusinkiewicz & Lindenmayer, "The Algorithmic Beauty of Plants" (1990)
2. **Voronoi Fracturing**: Fortune's Algorithm (1986)
3. **Metaballs**: Blinn, "A Generalization of Algebraic Surface Drawing" (1982)
4. **Marching Cubes**: Lorensen & Cline (1987)
5. **Shape Grammars**: Stiny & Gips (1971)
6. **Procedural Cities**: Parish & Müller, "Procedural Modeling of Cities" (2001)

---

## 🛠️ Implementation Priority

1. ✅ **Grass** (DONE!)
2. 🌲 **Trees** (Next - L-Systems, ~2 days)
3. 🪨 **Rocks** (Voronoi + Noise, ~1 day)
4. 🏛️ **Buildings** (Shape Grammar, ~3 days)
5. 🦌 **Animals** (Metaballs + Skeleton, ~4 days)
6. 🪰 **Insects** (Simplified animals, ~1 day)

**Total development time: ~2 weeks**
**Total GitHub storage: < 50 KB for ALL asset recipes**

---

## 🎯 Benefits Summary

✅ **No Git LFS needed** - All recipes < 1KB
✅ **Instant cloning** - No large binary downloads
✅ **Infinite variation** - Seed-based randomization
✅ **Memory efficient** - Generate on-demand, cull aggressively
✅ **LOD-friendly** - Adjust complexity based on distance
✅ **Moddable** - Users edit text files, not 3D software
✅ **Version control friendly** - Text diffs show parameter changes
✅ **Collaborative** - No merge conflicts on binary files

---

## 💡 Future Enhancements

### GPU Acceleration
Move generation to compute shaders for real-time creation:
- Generate grass blades in compute shader
- Cull invisible instances
- LOD transitions on GPU
- Potential 100x speedup

### Neural Network Assistance
Train models to generate optimal parameters:
- "Generate realistic oak tree" → Recipe parameters
- Style transfer (photo → tree recipe)
- User-guided editing ("make it bushier")

### Hybrid Approach
For hero assets (unique important objects):
- Artist creates base mesh
- Procgen adds details (leaves, cracks, weathering)
- Best of both worlds

---

## 📝 Notes

This framework allows the Roanoke Engine to have:
- **Photorealistic forests** without storing 10GB of trees
- **Diverse wildlife** without animator-created rigs
- **Entire colonial town** in < 100 KB of recipes
- **Living ecosystems** (grass, insects, animals) for free

**All stored in pure text files, Git-friendly, no LFS required.**

---

*"The best compression algorithm is not storing the data at all - generate it instead."*
