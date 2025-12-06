# Tree System Audit & Asset Pipeline Specification

**Date**: 2024-12-05 (Updated)
**Status**: Treeline & Bunch System Implemented
**Priority**: HIGH - Visual quality blocker (assets still needed)

---

## Recent Changes (2024-12-05)

### Treeline System Overhaul
- **40-yard treeline**: Trees now only spawn 40+ yards (~36.6m) from the shoreline
- **Distance-based logic**: Uses `distance_to_shoreline()` function instead of height-only checks
- **Clean visual boundary**: Creates a natural-looking treeline that follows coastline contours

### LowlandBunch System (NEW)
Clustered vegetation units for natural distribution:
- **1 anchor rock** (large boulder at center)
- **8-15 pebbles** scattered within radius
- **2 bushes** near the anchor
- **1 large tree** (if beyond treeline)

Bunches are spread on a jittered 18m grid across lowland/scrub zones.

### Density Increases (10x)
| Element | Old Density | New Density |
|---------|-------------|-------------|
| Large rocks | 0.02/m² | 0.2/m² |
| Pebbles | 0.12/m² | 1.2/m² |
| Deadwood | 0.008/m² | 0.08/m² |

### Key Files Modified
- `crates/croatoan_wfc/src/trees.rs` - Complete rewrite with bunch system
- `crates/croatoan_wfc/src/rocks.rs` - 10x density, bunch integration
- `crates/croatoan_wfc/src/vegetation.rs` - 10x detritus density
- `crates/croatoan_wfc/src/mesh_gen.rs` - Added `distance_to_shoreline()`, `get_biome_t()`
- `crates/croatoan_wfc/src/lib.rs` - New exports for bunch system

---

## Executive Summary

The current tree system renders "giant cardboard leaves" due to a broken asset pipeline. A 966K-line Blender OBJ export with 247K faces is being instanced 100+ times, destroying GPU performance and visual quality.

**Root Cause**: `trees/trees9.obj` contains unfiltered leaf geometry that bypasses the material name filter.

**Solution**: Establish a clean Blender-to-engine asset pipeline with separated trunk meshes, bark textures, and billboard leaf clusters.

---

## Current System Analysis

### Two Code Paths for Trees

| Path | Location | Status |
|------|----------|--------|
| OBJ Import | `trees/trees9.obj` | BROKEN - cardboard leaves |
| L-System Procgen | `croatoan_procgen/src/tree.rs` | Leaves disabled (commented out) |

### The Broken OBJ File

**File**: `trees/trees9.obj`
- **Lines**: 966,215
- **Faces**: 246,884
- **Objects**: 8 (3 are leaf meshes)

**Objects in file**:
```
Walnut_L   → Walnut leaves (LEAF)
Mossy_Tr   → Mossy trunk
Bark___S   → Bark
Bark___0   → Bark
Bottom_T   → Bottom texture
Sonnerat   → Sonneratia/mangrove leaves (LEAF)
Bark___1   → Bark
Oak_Leav   → Oak leaves (LEAF)
```

### Why Leaf Filter Fails

The filter at `roanoke_game/src/asset_loader.rs:31-35`:
```rust
if mat_name.contains("leaf") || mat_name.contains("leaves") || mat_name.contains("frond")
   || mat_name.contains("oak_leav") || mat_name.contains("sonnerat") || mat_name.contains("walnut_l") {
    println!("[ASSET] Skipping leaf mesh {}: {}", i, mat_name);
    continue;
}
```

**Problem**: Filter checks material names, but meshes without assigned material IDs bypass the filter entirely. The massive leaf geometry renders as flat, untextured quads.

---

## L-System Procgen Status

**File**: `crates/croatoan_procgen/src/tree.rs`

### What Works
- L-System string generation (7 species)
- Branch segment generation with proper tapering
- Turtle graphics interpretation
- Mesh generation for cylindrical branches

### What's Disabled

**Leaf generation** (lines 331-340):
```rust
// DISABLED for performance/style
/*
if random() < recipe.leaf_probability && turtle.thickness < 0.05 {
    leaves.push(LeafInstance {
        position: end,
        normal: turtle.direction,
        size: 0.2 + random() * 0.3,
    });
}
*/
```

**Leaf billboard mesh** (lines 483-525):
```rust
// Generate leaf billboards
// DISABLED for performance/style
/*
for leaf in &tree.leaves {
    // Creates fixed-orientation billboards (NOT camera-facing)
    let right = Vec3::X;  // PROBLEM: should face camera
    let up = Vec3::Z;
    ...
}
*/
```

### Issues with L-System Leaves (if re-enabled)
1. **Wrong orientation**: Billboards face up (`Vec3::Z`), not camera
2. **No texture**: Uses untextured white quads
3. **No wind**: Static while grass animates
4. **Performance**: Individual quads, not clustered

---

## Proposed Asset Pipeline

### Directory Structure

```
assets/
├── trees/
│   ├── trunks/
│   │   ├── oak_trunk.obj           # <2000 faces
│   │   ├── pine_trunk.obj
│   │   ├── birch_trunk.obj
│   │   ├── willow_trunk.obj
│   │   ├── mangrove_trunk.obj
│   │   └── palm_trunk.obj
│   │
│   └── textures/
│       ├── bark_oak.png            # 512x512, tileable
│       ├── bark_pine.png
│       ├── bark_birch.png
│       ├── bark_willow.png
│       ├── bark_mangrove.png
│       └── bark_palm.png
│
├── foliage/
│   ├── leaf_cluster_oak.png        # 256x256, alpha cutout
│   ├── leaf_cluster_maple.png
│   ├── leaf_cluster_birch.png
│   ├── leaf_cluster_willow.png
│   ├── leaf_cluster_pine.png       # Needle clusters
│   ├── leaf_cluster_palm.png       # Frond texture
│   └── leaf_cluster_mangrove.png
│
├── bushes/
│   ├── bush_fern.png               # Alpha cutout
│   ├── bush_shrub.png
│   ├── bush_berry.png
│   └── bush_grass_clump.png
│
└── terrain/
    ├── grass_diffuse.png           # 1024x1024, tileable
    ├── grass_normal.png
    ├── dirt_diffuse.png
    ├── dirt_normal.png
    ├── rock_diffuse.png
    ├── rock_normal.png
    ├── sand_diffuse.png
    └── sand_normal.png
```

### Blender Export Settings

**For trunk meshes**:
- Triangulate Faces: ON
- Apply Modifiers: ON
- Objects as OBJ Objects: ON
- Material Groups: ON
- Write Normals: ON
- Include UVs: ON
- **Face count limit**: 2000 faces per trunk

**Naming convention**:
- Trunk objects: `{species}_trunk`
- NO leaf geometry in OBJ files
- Materials: `bark_{species}`

### Leaf Cluster Texture Specification

| Property | Requirement |
|----------|-------------|
| Format | PNG with alpha channel |
| Size | 256x256 or 512x512 |
| Content | 5-7 leaves arranged in cluster |
| Background | Fully transparent (alpha = 0) |
| Edge quality | Clean alpha, no fringing |
| Orientation | Roughly facing camera |

**Creation workflow in Blender**:
1. Model single leaf mesh (10-20 faces)
2. Duplicate 5-7 times with rotation variation
3. Arrange in natural-looking cluster
4. Set up orthographic camera
5. Render to PNG with transparent background
6. Clean alpha edges in image editor (GIMP/Photoshop)

---

## Biome-Specific Tree System

### Data Structure

```rust
pub struct TreeBiomeSpec {
    pub trunk_mesh: &'static str,      // Path to OBJ
    pub bark_texture: &'static str,    // Path to PNG
    pub leaf_texture: &'static str,    // Path to alpha PNG
    pub leaf_density: f32,             // 0.0-1.0
    pub leaf_cluster_count: u32,       // Clusters per tree
    pub height_range: (f32, f32),      // Min/max meters
    pub trunk_lean_max: f32,           // Radians
    pub root_spread: f32,              // 0.0-1.0
    pub biome_elevation: (f32, f32),   // Terrain height range
    pub wind_sensitivity: f32,         // Leaf sway amount
}
```

### Biome Definitions

#### Oak Forest (Temperate Lowland)
```rust
pub const OAK_FOREST: TreeBiomeSpec = TreeBiomeSpec {
    trunk_mesh: "trees/trunks/oak_trunk.obj",
    bark_texture: "trees/textures/bark_oak.png",
    leaf_texture: "foliage/leaf_cluster_oak.png",
    leaf_density: 0.7,
    leaf_cluster_count: 12,
    height_range: (8.0, 18.0),
    trunk_lean_max: 0.05,
    root_spread: 0.2,
    biome_elevation: (15.0, 45.0),
    wind_sensitivity: 0.8,
};
```

#### Mangrove Swamp (Coastal/Tidal)
```rust
pub const MANGROVE_SWAMP: TreeBiomeSpec = TreeBiomeSpec {
    trunk_mesh: "trees/trunks/mangrove_trunk.obj",
    bark_texture: "trees/textures/bark_mangrove.png",
    leaf_texture: "foliage/leaf_cluster_mangrove.png",
    leaf_density: 0.5,
    leaf_cluster_count: 8,
    height_range: (4.0, 10.0),
    trunk_lean_max: 0.15,
    root_spread: 0.8,              // Wide stilt roots
    biome_elevation: (0.5, 4.0),   // Tidal zone
    wind_sensitivity: 0.6,
};
```

#### Dryland Pine (Arid Highland)
```rust
pub const DRYLAND_PINE: TreeBiomeSpec = TreeBiomeSpec {
    trunk_mesh: "trees/trunks/pine_trunk.obj",
    bark_texture: "trees/textures/bark_pine.png",
    leaf_texture: "foliage/leaf_cluster_pine.png",
    leaf_density: 0.4,
    leaf_cluster_count: 15,
    height_range: (6.0, 12.0),
    trunk_lean_max: 0.02,
    root_spread: 0.1,
    biome_elevation: (35.0, 55.0),
    wind_sensitivity: 0.3,
};
```

#### Weeping Willow (Riparian)
```rust
pub const WILLOW_RIVERSIDE: TreeBiomeSpec = TreeBiomeSpec {
    trunk_mesh: "trees/trunks/willow_trunk.obj",
    bark_texture: "trees/textures/bark_willow.png",
    leaf_texture: "foliage/leaf_cluster_willow.png",
    leaf_density: 0.9,
    leaf_cluster_count: 20,
    height_range: (6.0, 14.0),
    trunk_lean_max: 0.08,
    root_spread: 0.4,
    biome_elevation: (2.0, 8.0),   // Near water
    wind_sensitivity: 1.2,          // Extra droopy sway
};
```

---

## Implementation Phases

### Phase 1: Immediate Fixes
- [ ] Delete or archive `trees/trees9.obj` (966K lines)
- [ ] Fall back to L-system procgen temporarily
- [ ] Create ONE working trunk mesh (oak, <2K faces)
- [ ] Create ONE bark texture (512x512)
- [ ] Create ONE leaf cluster texture (256x256, alpha)

### Phase 2: Core Foliage System
- [ ] Implement camera-facing leaf billboards in `tree.wgsl`
- [ ] Add leaf cluster instancing
- [ ] Add wind animation to leaf shader
- [ ] Implement tree shadow casting
- [ ] Test with single tree type

### Phase 3: Biome Variety
- [ ] Create remaining trunk meshes (5 types)
- [ ] Create remaining bark textures
- [ ] Create remaining leaf cluster textures
- [ ] Implement `TreeBiomeSpec` system
- [ ] Wire biome selection to terrain elevation

### Phase 4: Polish
- [ ] Add bush system
- [ ] Add ground detail textures
- [ ] Implement tree LOD (distance-based simplification)
- [ ] Add seasonal color variation (optional)

---

## Asset Checklist

### Priority 1: Unblock Development

| Asset | Format | Size | Status |
|-------|--------|------|--------|
| `oak_trunk.obj` | OBJ | <2K faces | NEEDED |
| `bark_oak.png` | PNG | 512x512 | NEEDED |
| `leaf_cluster_oak.png` | PNG+Alpha | 256x256 | NEEDED |

### Priority 2: Biome Variety

| Asset | Format | Size | Status |
|-------|--------|------|--------|
| `pine_trunk.obj` | OBJ | <2K faces | NEEDED |
| `birch_trunk.obj` | OBJ | <2K faces | NEEDED |
| `willow_trunk.obj` | OBJ | <2K faces | NEEDED |
| `mangrove_trunk.obj` | OBJ | <2K faces | NEEDED |
| `palm_trunk.obj` | OBJ | <1K faces | NEEDED |
| All bark textures | PNG | 512x512 | NEEDED |
| All leaf cluster textures | PNG+Alpha | 256x256 | NEEDED |

### Priority 3: Ground Cover

| Asset | Format | Size | Status |
|-------|--------|------|--------|
| `bush_fern.png` | PNG+Alpha | 256x256 | NEEDED |
| `bush_shrub.png` | PNG+Alpha | 256x256 | NEEDED |
| `grass_diffuse.png` | PNG | 1024x1024 | NEEDED |
| `grass_normal.png` | PNG | 1024x1024 | NEEDED |
| `dirt_diffuse.png` | PNG | 1024x1024 | NEEDED |
| `rock_diffuse.png` | PNG | 1024x1024 | NEEDED |

---

## Missing Systems (Future Work)

### Animal Pathing
- No navigation mesh system exists
- No A* or pathfinding implementation
- No steering behaviors
- **Required**: NavMesh generation, pathfinding algorithm, agent steering

### AI Agentic Humans
- No NPC system exists yet
- No behavior trees or utility AI
- No needs/drives simulation
- No social graph
- **Required**: Full agent architecture, animations, scheduling system
- **SPEC CREATED**: See `NPC_VILLAGE_SPECIFICATION.md` for full design including:
  - Native American longhouse villages
  - NPC behavior trees (farming, dancing, praying)
  - Needs-driven scheduling system
  - Village layout and placement algorithms

---

## Files to Modify

| File | Changes Needed |
|------|----------------|
| `roanoke_game/src/main.rs:407-414` | Remove trees9.obj loading |
| `roanoke_game/src/asset_loader.rs` | Rewrite for new asset structure |
| `crates/croatoan_procgen/src/tree.rs` | Re-enable leaves OR replace with cluster system |
| `assets/shaders/tree.wgsl` | Add camera-facing billboards, wind |
| `crates/croatoan_render/src/tree_pipeline.rs` | Add leaf buffer, cluster instancing |
| `crates/croatoan_wfc/src/trees.rs` | Add biome-based tree selection |

---

## References

- Current tree shader: `assets/shaders/tree.wgsl`
- Tree pipeline: `crates/croatoan_render/src/tree_pipeline.rs`
- Tree procgen: `crates/croatoan_procgen/src/tree.rs`
- Tree placement: `crates/croatoan_wfc/src/trees.rs`
- Asset loader: `roanoke_game/src/asset_loader.rs`
- Main integration: `roanoke_game/src/main.rs:400-550`

---

*Document generated from Claude Code audit session.*

---

## Appendix A: Treeline & Bunch System API Reference

### Distance to Shoreline Function

```rust
/// Calculate distance from a point to the nearest shoreline
/// Returns 0 if point is in water, positive distance if on land
pub fn distance_to_shoreline(x: f32, z: f32, seed: u32) -> f32
```

**Algorithm**:
1. Check if current point is underwater (height < 0.5m)
2. March toward ocean (+X direction) in 10m steps
3. Binary search to refine shoreline position
4. Return Euclidean distance

**Usage**:
```rust
use croatoan_wfc::distance_to_shoreline;

let dist = distance_to_shoreline(world_x, world_z, seed);
if dist > 36.6 {
    // Beyond treeline, can spawn trees
}
```

### LowlandBunch Structure

```rust
/// A vegetation cluster containing rocks, pebbles, bushes, and optionally a tree
pub struct LowlandBunch {
    pub center: Vec3,       // World position
    pub radius: f32,        // 8-12m typically
    pub seed: u32,          // Deterministic generation seed
    pub has_tree: bool,     // True if beyond treeline
    pub biome_factor: f32,  // 0.0 = scrub, 1.0 = deep forest
}

impl LowlandBunch {
    /// Generate all instances within this bunch
    pub fn generate(&self, world_seed: u32) -> BunchInstances
}

/// Result containing all instance transforms
pub struct BunchInstances {
    pub trees: Vec<Mat4>,
    pub bushes: Vec<Mat4>,
    pub large_rocks: Vec<Mat4>,
    pub pebbles: Vec<Mat4>,
}
```

### Generation Functions

```rust
/// Generate trees and bushes for a chunk (includes bunches)
pub fn generate_trees_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<Mat4>

/// Get raw bunch data for external rock integration
pub fn generate_bunches_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<LowlandBunch>

/// Generate rocks with bunch integration (10x density)
pub fn generate_rocks_for_chunk(
    seed: u32,
    chunk_size: f32,
    offset_x: f32,
    offset_z: f32,
) -> Vec<(String, Mat4)>
```

### Biome Zones

| Zone | t Value | Height | Vegetation |
|------|---------|--------|------------|
| Ocean | < 0.45 | < 0m | None |
| Beach | 0.45-0.55 | 0-2m | Beach pebbles only |
| Scrub/Lowland | 0.55-0.65 | 2-6m | Bunches (no trees if < 40yd from shore) |
| Forest | > 0.65 | 6-15m+ | Dense trees + bunches |

### Constants

```rust
/// Minimum distance from shoreline for trees (40 yards)
const TREELINE_DISTANCE: f32 = 36.6;

/// Upper elevation where trees fade out
const UPPER_TREELINE_START: f32 = 40.0;
const UPPER_TREELINE_END: f32 = 55.0;

/// Bunch grid spacing
const BUNCH_GRID_SIZE: f32 = 18.0;
```

---

## Appendix B: Rock Type Reference

| Type | Scale | Sink | Description |
|------|-------|------|-------------|
| Pebble | 0.12 | 0.03 | Tiny stones, everywhere above water |
| SmallRock | 0.35 | 0.08 | Common ground detail |
| MediumRock | 0.75 | 0.18 | Slopes and rocky areas |
| LargeBoulder | 1.5 | 0.35 | Sparse landmarks, bunch anchors |
| FlatRock | 0.55 | 0.12 | Near water, paths |
| MossyRock | 0.65 | 0.22 | Damp/shaded areas |

### Rock Generation Phases

1. **Bunch-Integrated**: Anchor rocks + pebbles from LowlandBunch system
2. **Scattered Large**: Independent boulders on slopes (0.2/m²)
3. **Dense Pebbles**: Ground coverage everywhere (1.2/m²)
4. **Beach Strips**: Tide line pebbles (0.8/m² in beach zone)

---

## Appendix C: Performance Considerations

### Expected Instance Counts (256x256 chunk)

| Element | Count | Notes |
|---------|-------|-------|
| Trees | ~200-500 | Depends on biome |
| Bushes | ~100-300 | From bunches |
| Large rocks | ~500-2000 | Slopes + bunches |
| Pebbles | ~30,000-50,000 | Simplified transforms |
| Detritus | ~1,000-3,000 | Logs + branches |

### Optimization Notes

- Pebbles use Y-rotation only (no tilt) for faster transforms
- Bunches provide coordinated placement, reducing overlap checks
- Clustering noise reduces visual randomness while maintaining density
- Rock instances are grouped by type for batch rendering
