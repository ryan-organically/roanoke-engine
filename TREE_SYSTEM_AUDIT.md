# Tree System Audit & Asset Pipeline Specification

**Date**: 2024-11-29
**Status**: Action Required
**Priority**: HIGH - Visual quality blocker

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
- No NPC system exists
- No behavior trees or utility AI
- No needs/drives simulation
- No social graph
- **Required**: Full agent architecture, animations, scheduling system

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
