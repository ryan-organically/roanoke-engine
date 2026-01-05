# Underbrush Vegetation Specification

## Roanoke Engine - Coastal North Carolina Flora (1595)

Lightweight ground-cover vegetation system for historically accurate 16th-century coastal NC wilderness.

---

## Table of Contents

1. [Overview](#overview)
2. [Current Pipeline State](#current-pipeline-state)
3. [Native Species Reference](#native-species-reference)
4. [Rendering Approaches](#rendering-approaches)
5. [Asset Categories](#asset-categories)
6. [LOD Specifications](#lod-specifications)
7. [Interaction System](#interaction-system)
8. [Free Model Sources](#free-model-sources)
9. [Pipeline Integration](#pipeline-integration)
10. [Biome Distribution](#biome-distribution)
11. [Performance Budget](#performance-budget)
12. [Implementation Phases](#implementation-phases)
13. [Animation Decisions](#animation-decisions)
14. [Handoff Checklist](#handoff-checklist)

---

## Overview

### Design Goals

- **Historical Accuracy**: Only species native to coastal NC pre-colonization
- **Lightweight Rendering**: Billboard + instanced mesh hybrid approach
- **Dense Coverage**: Fill forest floor without killing performance
- **Seasonal Variation**: Color shifts, flowering, dormancy

### Coverage Types

| Type | Height | Density | Method |
|------|--------|---------|--------|
| Ground Cover | 0-15cm | Very High | Texture atlas billboards |
| Low Herbs | 15-50cm | High | Single-quad billboards |
| Tall Grasses | 50-150cm | Medium | Simple mesh clusters |
| Ferns & Palmettos | 30-100cm | Medium | LOD meshes |
| Wildflowers | 20-80cm | Sparse-Medium | Billboard imposters |

---

## Current Pipeline State

### Existing Rendering Pipelines

| Pipeline | Assets | Wind/Sway | Interaction |
|----------|--------|-----------|-------------|
| `tree_lod` | pine, birch, deadwood | shader UV.y driven | none |
| `vegetation_clump` | grass species | vertex Y driven | none |
| `static_lod` | boulders, debris | none | none |
| `fauna_animated` | creatures | armature | none |

**Key Observation**: Nothing is currently interactable. Trees sway but can't be chopped. Grass waves but nothing can be picked.

### What's Missing for Believable Forests

| Asset Type | Examples (1580s Coastal Virginia) | Target Pipeline |
|------------|-----------------------------------|-----------------|
| Low shrubs | Wax myrtle, bayberry, yaupon holly | `vegetation_clump` or new `shrub_lod` |
| Ferns | Cinnamon fern, royal fern, bracken | `vegetation_clump` |
| Ground cover | Partridge berry, moss patches, leaf litter | Texture/decal + sparse geometry |
| Fallen debris | Logs, branches, stumps | `static_lod` |
| Vines | Virginia creeper, muscadine grape | TBD (draped geometry?) |

The existing `grass_spec.json` already has `forest_floor` species. This spec expands that into proper understory coverage.

---

## Native Species Reference

### Grasses (Pre-Colonial NC)

| Species | Common Name | Height | Habitat | Visual Notes |
|---------|-------------|--------|---------|--------------|
| *Panicum virgatum* | Switchgrass | 90-150cm | Meadows, edges | Tall, airy panicles |
| *Uniola paniculata* | Sea Oats | 100-180cm | Coastal dunes | Iconic drooping seed heads |
| *Schizachyrium scoparium* | Little Bluestem | 60-120cm | Dry uplands | Blue-green → bronze/copper fall |
| *Muhlenbergia capillaris* | Muhly Grass | 60-90cm | Wet meadows | Pink/purple plumes (fall) |
| *Andropogon virginicus* | Broomsedge | 60-100cm | Old fields | Golden in autumn |
| *Sorghastrum nutans* | Indian Grass | 120-200cm | Prairies | Bronze fall color |
| *Spartina alterniflora* | Cordgrass | 60-200cm | Salt marsh | Coastal wetland indicator |
| *Chasmanthium latifolium* | River Oats | 60-120cm | Woodland edges | Drooping flat seed heads |

### Native Palms

| Species | Common Name | Height | Habitat | Visual Notes |
|---------|-------------|--------|---------|--------------|
| *Sabal minor* | **Dwarf Palmetto** | 60-180cm | Coastal forest floor | Fan-shaped, no visible trunk |
| *Rhapidophyllum hystrix* | Needle Palm | 60-120cm | Swamp edges | Spiny, compact |
| *Sabal palmetto* | Cabbage Palm | 10-20m | Southern coastal | Tall trunk (more FL than NC) |

**Note**: Dwarf Palmetto is THE primary palm for NC forest understory.

### Ferns

| Species | Common Name | Height | Habitat | Visual Notes |
|---------|-------------|--------|---------|--------------|
| *Osmunda cinnamomea* | Cinnamon Fern | 60-150cm | Wet woods | Cinnamon-colored fertile fronds |
| *Osmunda regalis* | Royal Fern | 90-180cm | Swamps | Large, open fronds |
| *Pteridium aquilinum* | Bracken Fern | 60-120cm | Open woods | Triangular fronds |
| *Athyrium filix-femina* | Lady Fern | 30-90cm | Moist forest | Delicate, lacy |
| *Polystichum acrostichoides* | Christmas Fern | 30-60cm | Forest slopes | Evergreen, leathery |
| *Dryopteris* spp. | Wood Ferns | 30-90cm | Forest floor | Classic fern shape |

### Wildflowers

| Species | Common Name | Height | Bloom | Color | Habitat |
|---------|-------------|--------|-------|-------|---------|
| *Solidago* spp. | Goldenrod | 60-150cm | Fall | Yellow | Meadows, edges |
| *Rudbeckia hirta* | Black-eyed Susan | 30-90cm | Summer | Yellow/brown | Meadows |
| *Lobelia cardinalis* | Cardinal Flower | 60-120cm | Summer | Bright red | Stream banks |
| *Viola* spp. | Wild Violet | 10-20cm | Spring | Purple/white | Forest floor |
| *Trillium* spp. | Trillium | 20-40cm | Spring | White/pink | Rich forest |
| *Arisaema triphyllum* | Jack-in-the-Pulpit | 30-60cm | Spring | Green/purple | Wet forest |
| *Sanguinaria canadensis* | Bloodroot | 15-25cm | Early spring | White | Forest floor |
| *Iris virginica* | Virginia Iris | 60-90cm | Spring | Blue/purple | Wet areas |
| *Asclepias tuberosa* | Butterfly Weed | 30-60cm | Summer | Orange | Dry meadows |
| *Eupatorium* spp. | Joe-Pye Weed | 120-200cm | Late summer | Pink/purple | Wet meadows |

### Ground Cover

| Species | Common Name | Height | Habitat | Visual Notes |
|---------|-------------|--------|---------|--------------|
| *Mitchella repens* | Partridge Berry | 5-10cm | Forest floor | Red berries, evergreen |
| *Gaultheria procumbens* | Wintergreen | 10-15cm | Acidic forest | Red berries, aromatic |
| *Chimaphila maculata* | Spotted Wintergreen | 10-20cm | Dry forest | Striped leaves |
| *Galax urceolata* | Galax | 15-30cm | Mountain forest | Round leaves, bronze winter |
| Mosses | Various | 1-5cm | Moist areas | Lush green carpet |
| Lichens | Various | 1-3cm | Rocks, trees | Gray-green patches |

### Shrubby Underbrush

| Species | Common Name | Height | Habitat | Visual Notes |
|---------|-------------|--------|---------|--------------|
| *Callicarpa americana* | **Beautyberry** | 100-180cm | Forest edges | Bright purple berries |
| *Myrica cerifera* | Wax Myrtle | 150-300cm | Coastal | Aromatic, waxy berries |
| *Ilex vomitoria* | Yaupon Holly | 150-450cm | Coastal forest | Red berries, evergreen |
| *Vaccinium* spp. | Blueberry | 30-180cm | Acidic soils | Edible berries |
| *Kalmia latifolia* | Mountain Laurel | 150-300cm | Upland forest | Pink/white flowers |
| *Rhododendron* spp. | Rhododendron | 200-400cm | Ravines | Large flowers |

---

## Rendering Approaches

### Tier 1: Ground Texture Variation (0-15cm)

No geometry. Handled via terrain shader detail textures.

```rust
// In terrain shader
struct GroundCoverLayer {
    texture: TextureAtlas,
    scale: f32,
    blend_mode: BlendMode,
    seasonal_tint: Vec4,
}

// Layers blend based on biome, moisture, slope
let ground_layers = [
    GroundCoverLayer::moss(),      // Wet areas
    GroundCoverLayer::leaf_litter(), // Forest floor
    GroundCoverLayer::pine_needles(), // Pine forest
    GroundCoverLayer::sand(),      // Coastal
];
```

### Tier 2: Billboard Grass (15-50cm)

Single-quad billboards, heavily instanced.

```rust
struct GrassBillboard {
    position: Vec3,
    rotation: f32,        // Y-axis only
    scale: Vec2,          // Width, height
    atlas_index: u8,      // Which grass type
    color_variation: f32, // Per-instance tint
}

// Atlas contains multiple grass/herb varieties
// 8x8 atlas = 64 varieties
```

**Grass Billboard Atlas Layout:**
```
+---+---+---+---+---+---+---+---+
| S1| S2| S3| S4| L1| L2| L3| L4|  Row 0: Switchgrass variants
+---+---+---+---+---+---+---+---+
| B1| B2| B3| B4| M1| M2| M3| M4|  Row 1: Bluestem, Muhly
+---+---+---+---+---+---+---+---+
| G1| G2| G3| G4| H1| H2| H3| H4|  Row 2: Generic grass, herbs
+---+---+---+---+---+---+---+---+
| V1| V2| V3| V4| F1| F2| F3| F4|  Row 3: Violets, small flowers
+---+---+---+---+---+---+---+---+
```

### Tier 3: Mesh Clusters (50-150cm)

Low-poly mesh clumps, instanced.

```rust
// Pre-made clump meshes
struct GrassClump {
    mesh_id: &'static str,  // "switchgrass_clump", "muhly_clump"
    vertex_count: u32,      // Target: 50-200 verts
    bounds: AABB,
}

// Clumps placed sparsely, billboards fill gaps
```

### Tier 4: LOD Meshes (Ferns, Palmettos)

Full meshes with LOD for distinctive plants.

```rust
struct FernLOD {
    lod0: MeshHandle,  // 300-500 verts, 0-30m
    lod1: MeshHandle,  // 100-150 verts, 30-80m
    lod2: MeshHandle,  // Billboard, 80m+
}
```

---

## Asset Categories

### Category A: Grass Billboards (Texture Atlas)

**Search Terms for Free Textures:**
- "grass alpha texture"
- "grass billboard PNG transparent"
- "meadow grass cutout"
- "wild grass texture pack"
- "prairie grass alpha"

**Recommended Sources:**
| Source | Search | License |
|--------|--------|---------|
| Poly Haven | "grass" textures | CC0 |
| ambientCG | "grass", "plant" | CC0 |
| Quixel Megascans | Grass atlases | Free w/ Epic |
| FreePBR | Vegetation | CC0 |
| TextureCan | Grass | Free |

**Atlas Spec:**
```json
{
  "atlas_name": "underbrush_grass_atlas",
  "resolution": 2048,
  "grid": "8x8",
  "cell_size": 256,
  "format": "PNG",
  "channels": "RGBA",
  "contents": [
    {"row": 0, "species": "switchgrass", "variants": 8},
    {"row": 1, "species": "bluestem", "variants": 4},
    {"row": 1, "col_start": 4, "species": "muhly", "variants": 4},
    {"row": 2, "species": "broomsedge", "variants": 4},
    {"row": 2, "col_start": 4, "species": "generic_grass", "variants": 4},
    {"row": 3, "species": "small_herbs", "variants": 8},
    {"row": 4, "species": "violets", "variants": 4},
    {"row": 4, "col_start": 4, "species": "ground_flowers", "variants": 4}
  ]
}
```

### Category B: Fern/Palmetto Meshes

**Search Terms:**
- "fern 3D model free"
- "low poly fern"
- "palmetto palm free model"
- "bracken fern game asset"
- "forest fern GLB"

**Sources:**
| Source | Model Types | License |
|--------|-------------|---------|
| Sketchfab | Ferns, palms | CC/Free filter |
| Poly Haven | Coming soon | CC0 |
| Quixel | Fern scans | Free w/ Epic |
| TurboSquid | Filter "free" | Varies |
| CGTrader | Filter "free" | Varies |
| Kenney | Stylized plants | CC0 |

**Sketchfab Search Queries:**
```
fern downloadable price:0
palm plant downloadable price:0
bracken fern low poly
sword fern game
palmetto fan palm
```

### Category C: Wildflower Billboards

**Search Terms:**
- "wildflower alpha texture"
- "flower billboard transparent"
- "goldenrod texture"
- "black-eyed susan PNG"
- "meadow flowers cutout"

**Flower Billboard Spec:**
```json
{
  "atlas_name": "wildflower_atlas",
  "resolution": 2048,
  "grid": "4x4",
  "cell_size": 512,
  "format": "PNG",
  "channels": "RGBA",
  "contents": [
    {"cell": 0, "species": "goldenrod"},
    {"cell": 1, "species": "black_eyed_susan"},
    {"cell": 2, "species": "cardinal_flower"},
    {"cell": 3, "species": "wild_violet"},
    {"cell": 4, "species": "butterfly_weed"},
    {"cell": 5, "species": "joe_pye_weed"},
    {"cell": 6, "species": "trillium"},
    {"cell": 7, "species": "iris"},
    {"cell": 8, "species": "bloodroot"},
    {"cell": 9, "species": "jack_in_pulpit"},
    {"cell": 10, "species": "generic_white"},
    {"cell": 11, "species": "generic_yellow"},
    {"cell": 12, "species": "generic_purple"},
    {"cell": 13, "species": "generic_pink"},
    {"cell": 14, "species": "generic_mixed"},
    {"cell": 15, "species": "empty_stems"}
  ]
}
```

### Category D: Dwarf Palmetto

**Priority Asset** - Distinctive NC understory plant.

**Search Terms:**
- "sabal minor 3D"
- "dwarf palmetto model"
- "fan palm low poly"
- "palmetto frond"
- "saw palmetto 3D"

**Custom Build Option:**
Simple fan-palm geometry:
- 5-9 fan fronds from central point
- Each frond: ~20 triangles
- Total: 100-180 tris per plant
- Height: 0.6-1.5m

```rust
// Palmetto procedural generation
fn generate_palmetto(frond_count: u8, height: f32) -> Mesh {
    let mut verts = Vec::new();
    let mut indices = Vec::new();

    for i in 0..frond_count {
        let angle = (i as f32 / frond_count as f32) * TAU;
        let tilt = rand_range(0.3, 0.8); // Radians from vertical
        generate_fan_frond(&mut verts, &mut indices, angle, tilt, height);
    }

    Mesh::new(verts, indices)
}
```

---

## LOD Specifications

### Fern LOD Spec

```json
{
  "asset": "fern_generic",
  "description": "Generic deciduous forest fern (lady fern, wood fern)",

  "lods": {
    "lod0": {
      "triangles": "300-500",
      "distance": "0-30m",
      "notes": "Individual fronds visible"
    },
    "lod1": {
      "triangles": "80-150",
      "distance": "30-80m",
      "notes": "Simplified frond shapes"
    },
    "lod2": {
      "triangles": "2 (billboard)",
      "distance": "80m+",
      "notes": "Cross-billboard imposter"
    }
  },

  "export": {
    "format": "GLB",
    "path": "assets/models/shrubs/",
    "filenames": [
      "fern_generic_lod0.glb",
      "fern_generic_lod1.glb"
    ]
  },

  "billboard_atlas": {
    "path": "assets/textures/imposters/fern_imposter.png",
    "resolution": 256,
    "views": 1,
    "notes": "Single front view, cross-billboard"
  }
}
```

### Fern Interaction Spec (Harvestable)

```json
{
  "asset": "fern_cinnamon",
  "interaction": {
    "pickable": true,
    "tool_required": null,
    "resource_yield": { "fern_frond": [1, 2], "fiddlehead": [0, 1] },
    "pick_time_ms": 400,
    "respawn_hours": 48,
    "seasons_available": ["spring", "summer", "fall"]
  }
}
```

### Palmetto LOD Spec

```json
{
  "asset": "dwarf_palmetto",
  "description": "Sabal minor - primary NC forest understory palm",

  "lods": {
    "lod0": {
      "triangles": "150-250",
      "distance": "0-40m",
      "notes": "Full fan fronds with texture detail"
    },
    "lod1": {
      "triangles": "60-100",
      "distance": "40-100m",
      "notes": "Simplified fronds"
    },
    "lod2": {
      "triangles": "2 (billboard)",
      "distance": "100m+",
      "notes": "Cross-billboard"
    }
  },

  "export": {
    "format": "GLB",
    "path": "assets/models/shrubs/",
    "filenames": [
      "palmetto_dwarf_lod0.glb",
      "palmetto_dwarf_lod1.glb"
    ]
  },

  "variants": {
    "count": 3,
    "differentiation": "frond_count, height, rotation"
  }
}
```

### Grass Clump Spec

```json
{
  "asset": "grass_clump",
  "description": "Tall grass cluster for sparse placement",

  "variants": [
    {
      "name": "switchgrass_clump",
      "triangles": 80,
      "height": "1.0-1.5m",
      "width": "0.4-0.6m"
    },
    {
      "name": "muhly_clump",
      "triangles": 100,
      "height": "0.6-0.9m",
      "width": "0.5-0.8m",
      "notes": "Pink plumes in fall"
    },
    {
      "name": "bluestem_clump",
      "triangles": 60,
      "height": "0.8-1.2m",
      "width": "0.3-0.5m"
    }
  ],

  "export": {
    "format": "GLB",
    "path": "assets/models/shrubs/",
    "filenames": [
      "grass_switchgrass_clump.glb",
      "grass_muhly_clump.glb",
      "grass_bluestem_clump.glb"
    ]
  }
}
```

---

## Interaction System

### Overview

The current engine has **no interaction system**. Trees sway but can't be chopped; grass waves but nothing can be picked. This section defines the interaction properties needed for harvestable vegetation.

### Harvestable Trees (Axe Required)

New properties for tree assets:

```json
{
  "asset": "pine_loblolly",
  "interaction": {
    "harvestable": true,
    "tool_required": "axe",
    "hit_points": 100,
    "resource_yield": {
      "wood_log": [2, 4],
      "branch": [0, 2],
      "pine_resin": [0, 1]
    },
    "hit_vfx": "wood_chip_spray",
    "destroy_vfx": "tree_fall_dust",
    "stump_asset": "pine_stump_0"
  }
}
```

**Tree Harvesting Animations:**

| Animation | Description | Source |
|-----------|-------------|--------|
| `hit_reaction` | Brief violent shake on axe impact | Shader parameter burst OR Blender armature |
| `falling` | Tree topples after HP depleted | Engine physics OR baked Blender animation |
| `stump_spawn` | Static mesh swap on destroy | Separate asset (no animation) |

### Pickable Plants (No Tool Required)

New properties for flowers, herbs, and small plants:

```json
{
  "asset": "wildflower_goldenrod",
  "interaction": {
    "pickable": true,
    "tool_required": null,
    "resource_yield": { "goldenrod_flower": 1 },
    "pick_time_ms": 500,
    "respawn_hours": 24,
    "pick_vfx": "pollen_burst"
  }
}
```

**Flower Picking Animations:**

| Animation | Description | Source |
|-----------|-------------|--------|
| `idle_sway` | Already exists via vertex Y wind | Shader (existing) |
| `pick` | Bend toward player, despawn | Engine procedural (lerp + particle + remove) |

**Note**: Flowers likely don't need Blender animations—engine can handle pick interaction procedurally.

### Berry Bushes (Hand or Tool)

```json
{
  "asset": "shrub_blueberry",
  "interaction": {
    "harvestable": true,
    "tool_required": null,
    "resource_yield": {
      "blueberry": [3, 8]
    },
    "harvest_time_ms": 1000,
    "respawn_hours": 72,
    "seasons_available": ["summer"],
    "partial_harvest": true,
    "harvest_stages": 3
  }
}
```

### Engine-Side Requirements

For Roanoke to support vegetation interaction:

1. **Raycast Detection**: Camera ray → detect harvestable entity → show interaction prompt
2. **Tool Checking**: Is player holding correct tool? (axe for trees, bare hands for flowers)
3. **Resource/Inventory**: What items spawn when harvested?
4. **State Persistence**: Does a chopped tree stay chopped? Save to chunk data?
5. **Respawn Timer**: Track last harvest time, respawn after `respawn_hours`
6. **Animation Triggers**: Play `hit_reaction`, spawn particles, transition to destroyed/empty state
7. **Seasonal Gating**: Some plants only harvestable in certain seasons

### Interaction Component (Rust)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interactable {
    pub interaction_type: InteractionType,
    pub tool_required: Option<ToolType>,
    pub hit_points: Option<u32>,           // For multi-hit (trees)
    pub current_hp: u32,
    pub resource_yields: Vec<ResourceYield>,
    pub interaction_time_ms: u32,          // For single-action (flowers)
    pub respawn_hours: Option<f32>,
    pub last_harvested: Option<f64>,       // Game time
    pub seasons_available: Vec<Season>,
    pub destroy_on_harvest: bool,
    pub replacement_asset: Option<String>, // e.g., "pine_stump"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionType {
    Choppable,    // Trees - multi-hit with axe
    Pickable,     // Flowers - instant with hands
    Harvestable,  // Bushes - timed gather
    Mineable,     // Rocks - multi-hit with pickaxe
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceYield {
    pub item_id: String,
    pub quantity_min: u32,
    pub quantity_max: u32,
    pub chance: f32,  // 0.0-1.0, for rare drops
}
```

---

## Free Model Sources

### Recommended Search Strategy

**Step 1: Quixel Megascans (Best Quality)**
```
Free with Epic Games account
Search: fern, grass, forest floor, palm
Download: Nanite-ready but can export lower LODs
```

**Step 2: Sketchfab (Variety)**
```
URL: sketchfab.com/search?type=models&downloadable=true&price=0
Queries:
  - "fern" sort by relevance
  - "palm plant" exclude "tree"
  - "grass clump low poly"
  - "forest floor plants"
  - "wildflower"
```

**Step 3: Poly Haven**
```
URL: polyhaven.com/models
Limited plant models but excellent textures
Great for grass/leaf texture atlases
```

**Step 4: OpenGameArt**
```
URL: opengameart.org
Search: vegetation, plants, grass
Often stylized but game-ready
```

**Step 5: BlenderKit (Blender Users)**
```
Free tier has many plants
Direct import to Blender
Can reduce/export as needed
```

### Specific Model Recommendations

| Need | Source | Search Query | Notes |
|------|--------|--------------|-------|
| Ferns | Quixel | "fern" | Scan quality, needs LOD reduction |
| Ferns | Sketchfab | "fern low poly downloadable" | Game-ready options |
| Grass clumps | Quixel | "grass" | Atlas-ready scans |
| Palmetto | Sketchfab | "palm fan low poly" | May need modification |
| Wildflowers | ambientCG | "flower" textures | Build billboards from photos |
| Ground cover | Poly Haven | Moss, leaf textures | Terrain shader use |

### Creating Custom Billboards

For species without good free models:

1. **Source Photos**:
   - iNaturalist.org (CC licensed plant photos)
   - Wikimedia Commons
   - Search: "[species name] transparent background"

2. **Process**:
   ```
   1. Find high-res photo of plant
   2. Remove background (remove.bg, GIMP, Photoshop)
   3. Export as PNG with alpha
   4. Add to texture atlas
   5. Create simple cross-billboard geometry
   ```

3. **Photo Sources for NC Native Plants**:
   - North Carolina Native Plant Society photos
   - NC State Extension plant database
   - Lady Bird Johnson Wildflower Center

---

## Pipeline Integration

### Directory Structure

```
assets/
├── models/
│   └── shrubs/
│       ├── fern_cinnamon_lod0.glb
│       ├── fern_cinnamon_lod1.glb
│       ├── fern_bracken_lod0.glb
│       ├── fern_christmas_lod0.glb
│       ├── palmetto_dwarf_lod0.glb
│       ├── palmetto_dwarf_lod1.glb
│       ├── grass_switchgrass_clump.glb
│       ├── grass_muhly_clump.glb
│       └── grass_bluestem_clump.glb
│
└── textures/
    └── vegetation/
        ├── underbrush_grass_atlas.png
        ├── wildflower_atlas.png
        ├── fern_imposter_atlas.png
        └── ground_cover_detail.png
```

### Mesh Registry Addition

```rust
// main.rs mesh loading section

// Underbrush ferns
for species in ["cinnamon", "bracken", "christmas", "lady", "royal"] {
    for lod in 0..=1 {
        let path = format!("assets/models/shrubs/fern_{}_lod{}.glb", species, lod);
        if let Ok(mesh_data) = load_glb_model(&path, device, queue) {
            state.mesh_registry.insert(
                format!("fern_{}_lod{}", species, lod),
                create_mesh(device, mesh_data)
            );
        }
    }
}

// Palmetto variants
for variant in 0..3 {
    for lod in 0..=1 {
        let path = format!("assets/models/shrubs/palmetto_dwarf_{}_lod{}.glb", variant, lod);
        if let Ok(mesh_data) = load_glb_model(&path, device, queue) {
            state.mesh_registry.insert(
                format!("palmetto_{}_lod{}", variant, lod),
                create_mesh(device, mesh_data)
            );
        }
    }
}

// Grass clumps
for grass_type in ["switchgrass", "muhly", "bluestem", "broomsedge"] {
    let path = format!("assets/models/shrubs/grass_{}_clump.glb", grass_type);
    if let Ok(mesh_data) = load_glb_model(&path, device, queue) {
        state.mesh_registry.insert(
            format!("grass_{}", grass_type),
            create_mesh(device, mesh_data)
        );
    }
}
```

### Underbrush Pipeline (New)

```rust
// crates/croatoan_render/src/underbrush_pipeline.rs

pub struct UnderbrushPipeline {
    // Billboard grass (GPU instanced)
    grass_billboard_pipeline: wgpu::RenderPipeline,
    grass_instance_buffer: wgpu::Buffer,
    grass_atlas: wgpu::TextureView,

    // Mesh-based plants (LOD)
    mesh_pipeline: wgpu::RenderPipeline,
    fern_meshes: HashMap<String, MeshLOD>,
    palmetto_meshes: HashMap<String, MeshLOD>,

    // Wildflower billboards
    flower_billboard_pipeline: wgpu::RenderPipeline,
    flower_atlas: wgpu::TextureView,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GrassBillboardInstance {
    pub position: [f32; 3],
    pub rotation: f32,
    pub scale: [f32; 2],
    pub atlas_uv: [f32; 2],      // UV offset in atlas
    pub color_tint: [f32; 3],
    pub wind_phase: f32,
}

impl UnderbrushPipeline {
    pub fn generate_chunk_underbrush(
        &self,
        chunk: ChunkCoord,
        terrain: &TerrainData,
        biome_map: &BiomeMap,
        season: Season,
    ) -> ChunkUnderbrush {
        let mut grass_instances = Vec::new();
        let mut fern_instances = Vec::new();
        let mut palmetto_instances = Vec::new();
        let mut flower_instances = Vec::new();

        // Sample points across chunk
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_pos = chunk_to_world(chunk, x, z);
                let height = terrain.height_at(world_pos);
                let slope = terrain.slope_at(world_pos);
                let moisture = terrain.moisture_at(world_pos);
                let biome = biome_map.biome_at(world_pos);

                // Skip steep slopes
                if slope > 0.7 { continue; }

                // Biome-specific vegetation
                match biome {
                    Biome::DeciduousForest => {
                        // Dense ferns, scattered palmetto
                        if should_spawn_fern(world_pos, moisture) {
                            fern_instances.push(create_fern_instance(world_pos, height));
                        }
                        if should_spawn_palmetto(world_pos) {
                            palmetto_instances.push(create_palmetto_instance(world_pos, height));
                        }
                        // Ground herbs
                        add_ground_grass(&mut grass_instances, world_pos, height, "forest_floor");
                    }
                    Biome::Meadow => {
                        // Tall grasses, wildflowers
                        add_meadow_grass(&mut grass_instances, world_pos, height);
                        if should_spawn_flower(world_pos, season) {
                            flower_instances.push(create_flower_instance(world_pos, height, season));
                        }
                    }
                    Biome::Coastal => {
                        // Sea oats, palmetto dominant
                        if should_spawn_palmetto(world_pos) {
                            palmetto_instances.push(create_palmetto_instance(world_pos, height));
                        }
                        add_coastal_grass(&mut grass_instances, world_pos, height);
                    }
                    Biome::Swamp => {
                        // Ferns, no palmetto (too wet)
                        if should_spawn_fern(world_pos, moisture) {
                            fern_instances.push(create_swamp_fern_instance(world_pos, height));
                        }
                    }
                    _ => {}
                }
            }
        }

        ChunkUnderbrush {
            grass_instances,
            fern_instances,
            palmetto_instances,
            flower_instances,
        }
    }
}
```

---

## Biome Distribution

### Vegetation Density by Biome

| Biome | Grass Density | Fern Density | Palmetto | Flowers |
|-------|---------------|--------------|----------|---------|
| Deciduous Forest | Low | High | Medium | Low |
| Pine Forest | Medium | Low | High | Low |
| Mixed Forest | Medium | Medium | Medium | Low |
| Meadow | Very High | None | None | High |
| Forest Edge | High | Medium | Low | Medium |
| Coastal | Medium | Low | High | Low |
| Swamp | Low | Very High | None | Low |
| River Bank | Medium | High | Low | Medium |

### Species by Biome

```rust
pub fn get_underbrush_species(biome: Biome) -> UnderbrushPalette {
    match biome {
        Biome::DeciduousForest => UnderbrushPalette {
            grasses: vec![
                ("river_oats", 0.3),
                ("generic_forest_grass", 0.5),
            ],
            ferns: vec![
                ("christmas_fern", 0.3),  // Evergreen
                ("lady_fern", 0.25),
                ("wood_fern", 0.25),
                ("cinnamon_fern", 0.2),
            ],
            palms: vec![
                ("dwarf_palmetto", 0.15),
            ],
            flowers: vec![
                ("trillium", 0.1),
                ("violet", 0.15),
                ("bloodroot", 0.05),
                ("jack_in_pulpit", 0.05),
            ],
        },

        Biome::Meadow => UnderbrushPalette {
            grasses: vec![
                ("switchgrass", 0.3),
                ("little_bluestem", 0.25),
                ("indian_grass", 0.15),
                ("broomsedge", 0.2),
            ],
            ferns: vec![],
            palms: vec![],
            flowers: vec![
                ("goldenrod", 0.2),
                ("black_eyed_susan", 0.15),
                ("butterfly_weed", 0.1),
                ("joe_pye_weed", 0.1),
            ],
        },

        Biome::Coastal => UnderbrushPalette {
            grasses: vec![
                ("sea_oats", 0.4),
                ("cordgrass", 0.3),
            ],
            ferns: vec![],
            palms: vec![
                ("dwarf_palmetto", 0.4),
            ],
            flowers: vec![],
        },

        Biome::Swamp => UnderbrushPalette {
            grasses: vec![
                ("sedge", 0.3),
            ],
            ferns: vec![
                ("cinnamon_fern", 0.35),
                ("royal_fern", 0.3),
            ],
            palms: vec![],
            flowers: vec![
                ("cardinal_flower", 0.1),
                ("iris", 0.1),
            ],
        },

        _ => UnderbrushPalette::default(),
    }
}
```

---

## Performance Budget

### Target Metrics

| Metric | Budget | Notes |
|--------|--------|-------|
| Grass billboards/chunk | 2000-4000 | GPU instanced |
| Fern meshes/chunk | 50-100 | LOD managed |
| Palmetto meshes/chunk | 20-50 | LOD managed |
| Flower billboards/chunk | 100-300 | Seasonal |
| Draw calls for underbrush | 4-6 | Batched by type |
| VRAM for atlases | ~32MB | 2x 2048 + imposters |

### LOD Distances

```rust
const GRASS_BILLBOARD_FADE_START: f32 = 80.0;
const GRASS_BILLBOARD_FADE_END: f32 = 120.0;

const FERN_LOD0_DISTANCE: f32 = 30.0;
const FERN_LOD1_DISTANCE: f32 = 80.0;
const FERN_BILLBOARD_DISTANCE: f32 = 80.0;
const FERN_CULL_DISTANCE: f32 = 200.0;

const PALMETTO_LOD0_DISTANCE: f32 = 40.0;
const PALMETTO_LOD1_DISTANCE: f32 = 100.0;
const PALMETTO_CULL_DISTANCE: f32 = 250.0;
```

### Shader Optimization

```wgsl
// Grass billboard vertex shader with wind
@vertex
fn vs_grass(
    @builtin(instance_index) instance_idx: u32,
    @location(0) vertex_pos: vec2<f32>,
) -> VertexOutput {
    let instance = grass_instances[instance_idx];

    // Billboard facing camera
    let right = normalize(cross(vec3(0.0, 1.0, 0.0), camera.forward));
    let up = vec3(0.0, 1.0, 0.0);

    // Wind displacement (height-based)
    let wind_strength = vertex_pos.y * 0.15;
    let wind_offset = sin(time + instance.wind_phase) * wind_strength;

    // Final position
    var world_pos = instance.position;
    world_pos += right * vertex_pos.x * instance.scale.x;
    world_pos.y += vertex_pos.y * instance.scale.y;
    world_pos.x += wind_offset;

    // Distance fade
    let dist = distance(world_pos, camera.position);
    let fade = 1.0 - smoothstep(80.0, 120.0, dist);

    var out: VertexOutput;
    out.position = camera.view_proj * vec4(world_pos, 1.0);
    out.uv = vertex_pos * 0.5 + 0.5 + instance.atlas_uv;
    out.color = vec4(instance.color_tint, fade);
    return out;
}
```

---

## Implementation Phases

### Phase 1: Grass Billboard System
- [ ] Create grass texture atlas (8x8 varieties)
- [ ] Implement billboard instancing pipeline
- [ ] Add wind animation to shader
- [ ] Chunk-based grass generation
- [ ] Distance fade/culling

### Phase 2: Fern Meshes
- [ ] Acquire/create 3-4 fern LOD0 models
- [ ] Create LOD1 simplified versions
- [ ] Generate billboard imposters
- [ ] Implement LOD switching
- [ ] Biome-based fern placement

### Phase 3: Dwarf Palmetto
- [ ] Model or acquire palmetto mesh
- [ ] Create LOD variants
- [ ] Add to coastal/forest biomes
- [ ] Variant randomization (frond count, rotation)

### Phase 4: Wildflowers
- [ ] Create wildflower billboard atlas
- [ ] Seasonal bloom logic
- [ ] Meadow/edge placement
- [ ] Color variation per species

### Phase 5: Integration
- [ ] Connect to existing foliage pipeline
- [ ] Seasonal color tinting
- [ ] Performance profiling
- [ ] Quality settings (Low/Med/High density)

---

## Asset Acquisition Checklist

### Immediate Needs (Priority Order)

1. **Grass Billboard Atlas**
   - [ ] 8 switchgrass variants
   - [ ] 4 bluestem variants
   - [ ] 4 muhly variants (with pink plumes)
   - [ ] 8 generic grass/herb variants
   - [ ] Compile into 2048x2048 atlas

2. **Fern Models**
   - [ ] Christmas fern (evergreen) - LOD0 + LOD1
   - [ ] Lady fern (deciduous) - LOD0 + LOD1
   - [ ] Cinnamon fern (wet areas) - LOD0 + LOD1
   - [ ] Bracken fern (open woods) - LOD0 + LOD1

3. **Dwarf Palmetto**
   - [ ] Main model - LOD0 + LOD1
   - [ ] 2-3 variants (different frond arrangements)

4. **Wildflower Atlas**
   - [ ] Goldenrod
   - [ ] Black-eyed Susan
   - [ ] Cardinal flower
   - [ ] Wild violet
   - [ ] Generic colors (white, yellow, purple, pink)

---

## Animation Decisions

### Open Questions

| Decision | Option A | Option B | Recommendation |
|----------|----------|----------|----------------|
| Tree hit reaction | Shader parameter (wind burst) | Armature animation (NLA strip) | **Shader** - simpler, no rigging needed |
| Tree falling | Baked animation from Blender | Real-time physics in engine | **Baked** initially, physics later |
| Flower picking | Engine-only procedural | Blender baked bend animation | **Engine** - simpler, more flexible |
| Fern sway | Current vertex Y wind | Armature for more control | **Keep current** - works well |

### Shader-Based Hit Reaction (Recommended)

Add a uniform to tree shader for "impact strength":

```wgsl
struct TreeUniforms {
    // ... existing ...
    impact_strength: f32,  // 0.0 = normal, 1.0 = max shake
    impact_time: f32,      // Time since last hit
}

// In vertex shader
let impact_decay = exp(-impact_time * 5.0);
let shake = sin(impact_time * 30.0) * impact_strength * impact_decay;
let impact_offset = vec3(shake * 0.1, 0.0, shake * 0.05);
world_pos += impact_offset * vertex_height_factor;
```

### Baked Tree Fall Animation

If using baked animations:
- Export from Blender as separate GLB with animation
- Animation length: ~2-3 seconds
- Include dust particle trigger at impact frame
- Spawn stump asset at animation end

If using physics:
- Convert tree to rigid body on final hit
- Apply initial rotation impulse
- Detect ground collision for dust VFX
- More dynamic but harder to control

---

## Handoff Checklist

### New Specs to Create

| Spec File | Purpose | Status |
|-----------|---------|--------|
| `UNDERBRUSH_1595_SPEC.md` | This document | Created |
| `shrub_spec.json` | Understory bushes (wax myrtle, beautyberry) | TODO |
| `flower_spec.json` | Pickable wildflowers and herbs | TODO |
| `harvestable_tree_extension.json` | Interaction properties for existing trees | TODO |

### Asset Acquisition

| Asset Type | Count Needed | Source Strategy |
|------------|--------------|-----------------|
| Grass billboard atlas | 1 (2048x2048) | Quixel + custom assembly |
| Fern LOD meshes | 4-5 species x 2 LODs | Quixel/Sketchfab |
| Dwarf palmetto | 3 variants x 2 LODs | Sketchfab or custom |
| Wildflower atlas | 1 (2048x2048) | Photo-based billboards |
| Shrub meshes | 4-6 species | Quixel/Sketchfab |
| Tree stumps | 3-4 variants | Simple Blender models |

### Engine Implementation Tasks

| Task | Priority | Dependencies |
|------|----------|--------------|
| Interaction raycast system | High | None |
| Tool requirement checking | High | Inventory system |
| Resource drop spawning | High | Item system |
| Chunk-based harvest state | Medium | Chunk persistence |
| Respawn timer system | Medium | Game time |
| Tree hit shader parameter | Medium | None |
| Flower pick procedural anim | Low | Interaction system |
| Tree fall animation/physics | Low | Interaction system |

### Blender Export Needs

| Asset | Animations Needed | Notes |
|-------|-------------------|-------|
| Trees | `falling` (optional) | Can defer to physics |
| Shrubs | None | Static with wind shader |
| Ferns | None | Vertex wind only |
| Flowers | None | Engine procedural |
| Stumps | None | Static replacement mesh |

### Integration Points

```
main.rs
├── Mesh loading: Add underbrush meshes to registry
├── Chunk generation: Call underbrush spawning
└── Render loop: Add underbrush pipeline pass

interaction.rs (new)
├── Raycast detection
├── Tool validation
├── Harvest state tracking
└── Resource spawning

underbrush_pipeline.rs (new)
├── Billboard instancing
├── LOD management
└── Seasonal tinting
```

---

*The forest floor of 1595 Virginia teems with life - ferns unfurling in spring shade, palmetto fans rustling in coastal breeze, goldenrod blazing in autumn meadows.*
