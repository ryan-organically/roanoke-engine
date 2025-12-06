# NPC & Village System Specification

**Date**: 2024-12-05
**Status**: Phase 1 Implemented (Generation)
**Priority**: HIGH - Core gameplay feature
**Target Version**: v0.0.3+

---

## Executive Summary

This document specifies the NPC (Non-Player Character) and village settlement systems for the Roanoke engine. The initial implementation focuses on **Native American longhouse villages** with procedurally generated structures, characters, and agricultural elements.

### Implementation Status

| Component | Status | Location |
|-----------|--------|----------|
| Longhouse Generation | **COMPLETE** | `croatoan_procgen/src/longhouse.rs` |
| NPC Generation | **COMPLETE** | `croatoan_procgen/src/npc.rs` |
| Village Layout | **COMPLETE** | `croatoan_procgen/src/village.rs` |
| World Integration | **COMPLETE** | `croatoan_wfc/src/villages.rs` |
| Behavior System | PLANNED | - |
| Animation System | PLANNED | - |

---

## Quick Start

### Generate a Village

```rust
use croatoan_wfc::{find_village_sites, generate_world_village, VillageId};
use glam::Vec3;

// Find suitable locations in a region
let sites = find_village_sites(
    world_seed,
    Vec3::new(-500.0, 0.0, -500.0),  // Region min
    Vec3::new(500.0, 0.0, 500.0),    // Region max
    5,                                // Max villages
);

// Generate a village at a site
let village = generate_world_village(
    sites[0],           // Center position
    world_seed,         // World seed
    1,                  // Village ID
);

println!("Village '{}' has {} longhouses and {} NPCs",
    village.layout.name,
    village.layout.longhouses.len(),
    village.layout.npcs.len()
);
```

### Get Structures for a Chunk

```rust
use croatoan_wfc::get_village_structures_for_chunk;

let structures = get_village_structures_for_chunk(
    &village,
    chunk_x,        // Chunk min X
    chunk_z,        // Chunk min Z
    chunk_size,     // Chunk dimensions
    world_seed,
);

for structure in structures {
    // structure.mesh_vertices - flat f32 array (pos, normal, uv, color)
    // structure.mesh_indices - triangle indices
    // structure.transform - world transform matrix
    // structure.structure_type - Longhouse, FirePit, CornPlant, etc.
}
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        croatoan_procgen                          │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   longhouse.rs  │     npc.rs      │         village.rs          │
│                 │                 │                             │
│ LonghouseRecipe │ NpcRecipe       │ VillageRecipe               │
│ LonghouseMesh   │ NpcData         │ VillageLayout               │
│ LonghouseVertex │ NpcAppearance   │ LonghousePlacement          │
│                 │ NpcMesh         │ FirePit, CornField          │
│ generate_       │ generate_npc()  │ generate_village()          │
│   longhouse()   │ generate_       │ generate_fire_pit()         │
│                 │   npc_mesh()    │ generate_corn_plant()       │
└─────────────────┴─────────────────┴─────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         croatoan_wfc                             │
├─────────────────────────────────────────────────────────────────┤
│                        villages.rs                               │
│                                                                 │
│  find_village_sites()      - Terrain-aware site selection       │
│  generate_world_village()  - Creates full village at location   │
│  get_village_structures_   - Returns renderable meshes per      │
│    for_chunk()               chunk for streaming                │
│                                                                 │
│  WorldVillage              - Village instance in world          │
│  VillageStructure          - Single renderable structure        │
│  VillageStructureType      - Longhouse, FirePit, CornPlant...   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Longhouse Generation

### File: `crates/croatoan_procgen/src/longhouse.rs`

The Iroquoian longhouse was a multi-family dwelling constructed from bark over a bent-pole frame. Our procedural generator creates historically-inspired structures.

### Architectural Styles

```rust
pub enum LonghouseStyle {
    Iroquoian,      // Bark-covered, rounded ends (DEFAULT)
    Algonquian,     // Bark/mat-covered, dome ends
    Coastal,        // Plank-covered, flat ends (Pacific NW)
}
```

### Recipe Configuration

```rust
pub struct LonghouseRecipe {
    pub style: LonghouseStyle,
    pub family_units: u32,      // 2-10, determines length
    pub width: f32,             // 6.0-7.0 meters typical
    pub height: f32,            // 5.0-6.0 meters at ridge
    pub seed: u32,              // Randomization seed
}
```

**Derived Properties:**
- `length()` → `family_units * 6.0` meters
- `door_count()` → 2 (ends) + 1 if `family_units > 5`
- `smoke_hole_count()` → `max(1, family_units / 2)`
- `hearth_count()` → `family_units / 2`

### Presets

| Preset | Family Units | Dimensions (WxHxL) | Use Case |
|--------|--------------|-------------------|----------|
| `small_clan_house()` | 3 | 6.0 x 5.0 x 18m | Small families |
| `iroquoian_medium()` | 5 | 6.5 x 5.5 x 30m | Average dwelling |
| `large_council_house()` | 8 | 7.0 x 6.0 x 48m | Council meetings |

### Generated Mesh Components

1. **Frame Poles** - Bent saplings forming arches every 1.5m
2. **Ridge Pole** - Horizontal beam along roof peak
3. **Horizontal Stringers** - 3 per side at varying heights
4. **Bark Shell** - Curved panels with color variation
5. **End Walls** - Rounded (Iroquoian), Dome (Algonquian), or Flat (Coastal)
6. **Doorways** - Frame + opening at ends and optionally center
7. **Smoke Holes** - Dark-stained rectangular openings in roof
8. **Interior Hearths** - Stone ring fire pits inside

### Color Palette

| Component | RGB | Description |
|-----------|-----|-------------|
| Frame poles | (0.55, 0.40, 0.25) | Stripped sapling wood |
| Elm bark | (0.45, 0.35, 0.28) | Exterior covering |
| Smoke stain | (0.30, 0.27, 0.25) | Around smoke holes |
| Door frame | (0.40, 0.30, 0.22) | Darker wood trim |
| Hearth stones | (0.40, 0.40, 0.42) | Gray fieldstones |

### Vertex Format

```rust
#[repr(C)]
pub struct LonghouseVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
}
// Total: 44 bytes per vertex (11 floats)
```

---

## NPC Generation

### File: `crates/croatoan_procgen/src/npc.rs`

NPCs are procedurally generated with culturally-appropriate appearances, roles, and names.

### NPC Roles

```rust
pub enum NpcRole {
    Chief,          // Village leader, elder male
    Shaman,         // Spiritual leader, elder, any gender
    Warrior,        // Combat-ready adult male
    Hunter,         // Provides food
    Farmer,         // Tends corn fields
    Craftsperson,   // Makes goods
    Elder,          // Respected elder
    Child,          // Young villager
    Villager,       // General population
}
```

### Role Distribution in Villages

| Role | Percentage | Notes |
|------|------------|-------|
| Chief | 1 per village | Always elder male |
| Shaman | 1 per village | Elder, any gender |
| Warriors | 10% | Adult males |
| Farmers | 30% | Any adult |
| Villagers | Remainder | Mixed |

### Appearance Generation

```rust
pub struct NpcAppearance {
    pub gender: Gender,           // Male, Female
    pub age_category: AgeCategory, // Child, Youth, Adult, Elder
    pub height: f32,              // 1.0-1.9 meters
    pub build: BodyBuild,         // Slim, Average, Stocky
    pub skin_tone: [f32; 3],      // Warm copper/bronze tones
    pub hair_style: HairStyle,    // Long, Braided, Mohawk, etc.
    pub hair_color: [f32; 3],     // Black/brown, gray for elders
    pub clothing: Vec<ClothingType>,
    pub adornments: Vec<Adornment>,
}
```

### Height by Age/Gender

| Age | Male Height | Female Height |
|-----|-------------|---------------|
| Child | 1.0-1.3m | 1.0-1.3m |
| Youth | 1.4-1.6m | 1.4-1.6m |
| Adult | 1.65-1.85m | 1.55-1.70m |
| Elder | 1.60-1.75m | 1.50-1.62m |

### Hair Styles

| Style | Gender | Roles |
|-------|--------|-------|
| `Mohawk` | Male | Warriors |
| `Topknot` | Male | Chiefs, some males |
| `Braided` | Female | Most females |
| `Long` | Both | Common |
| `Shaved` | Male | Some males |

### Clothing by Gender

**Male:**
- Breechcloth (always)
- Leggings (60% chance)
- Robe (elders/chiefs)
- Moccasins (always)

**Female:**
- Dress (always)
- Leggings (40% chance)
- Moccasins (always)

### Adornments by Role

| Role | Adornments |
|------|------------|
| Chief | Feather, Beads, Necklace |
| Shaman | Feather, Tattoo, Necklace |
| Warrior | WarPaint, Feather (50%) |
| Others | Beads (30%), Earring (20%) |

### Name Generation

Names are generated using authentic syllable patterns:

**Male syllables:** Ta, Wa, Ki, Mo, Ha, Ne, So, Ke, O, A, hon, kan, wen, ta, da, ko, na, wa, he, yo

**Female syllables:** A, O, Ka, Te, Wa, Ya, Ne, Hi, Sa, Mi, wen, da, na, ko, ya, wa, ni, ta, he, la

Names are 2-3 syllables, first capitalized: "Tawenho", "Awakoya", "Moheda"

---

## Village Layout

### File: `crates/croatoan_procgen/src/village.rs`

Villages are organic arrangements of longhouses, fire pits, agricultural fields, and sacred sites.

### Village Sizes

```rust
pub struct VillageRecipe {
    pub population: u32,
    pub seed: u32,
    pub style: LonghouseStyle,
}

// Presets
VillageRecipe::small_camp(seed)      // 15 population
VillageRecipe::medium_village(seed)  // 35 population
VillageRecipe::large_village(seed)   // 60 population
```

### Layout Structure

```rust
pub struct VillageLayout {
    pub id: VillageId,
    pub center: Vec3,
    pub name: String,                    // "Kanata", "Onondaga", etc.
    pub longhouses: Vec<LonghousePlacement>,
    pub fire_pits: Vec<FirePit>,
    pub corn_fields: Vec<CornField>,
    pub prayer_sites: Vec<PrayerSite>,
    pub npcs: Vec<NpcData>,
    pub bounds_radius: f32,
}
```

### Spatial Arrangement

```
                    N (Prayer - Sacred Tree)
                           │
              ┌────────────┼────────────┐
              │     ═══════════════     │  ← Corn Fields
              │    ╱               ╲    │
    W (Shrine)│   ║   Longhouse    ║   │E (Sunrise Prayer)
              │   ║                ║    │
              │    ╲    ● Fire    ╱     │  ← Central Ceremonial Fire
              │     ║           ║       │
              │     ║ Longhouse ║       │
              │      ╲         ╱        │
              │       ═══════════       │  ← Corn Fields
              └────────────┼────────────┘
                           │
                    S (Shrine)
```

**Radii:**
- Longhouse ring: 25m + 4m per longhouse
- Corn fields: Longhouse radius + 35m
- Prayer sites: Longhouse radius + 15m

### Fire Pit Types

```rust
pub struct FirePit {
    pub position: Vec3,
    pub radius: f32,
    pub is_ceremonial: bool,         // Central fire = true
    pub dance_circle_radius: f32,    // For dancing (ceremonial only)
}
```

| Type | Radius | Features |
|------|--------|----------|
| Ceremonial (central) | 1.5m | 16 stones, log pile, dance circle |
| Domestic (per longhouse) | 0.8m | 10 stones, simple |

### Corn Fields

```rust
pub struct CornField {
    pub position: Vec3,
    pub size: Vec2,              // 18-30m x 12-20m
    pub rows: u32,               // 6-10 rows
    pub mounds: Vec<Vec3>,       // Three Sisters mound positions
}
```

**Three Sisters Agriculture:** Each mound grows corn, beans, and squash together. Mounds are spaced ~3m apart in rows.

### Corn Growth Stages

```rust
pub enum CornGrowthStage {
    Sprout,      // 0.15m height, 2 leaves
    Young,       // 0.5m height, 4 leaves
    Growing,     // 1.2m height, 6 leaves
    Tasseling,   // 1.8m height, 8 leaves + golden tassel
    Mature,      // 2.2m height, 10 leaves + tassel + ear
}
```

### Prayer Sites

```rust
pub enum PrayerSiteType {
    SunriseKnoll,       // East - morning prayers
    AncestorShrine,     // West/South - remembrance
    SacredTree,         // North - old growth
    WaterEdge,          // Near water (future)
}
```

### Village Names

Generated from historical Iroquoian names:
- Kanata, Ossernenon, Onondaga, Cayuga, Seneca, Mohawk
- Stadacona, Hochelaga, Ganondagan, Caughnawaga, Kahnawake

### Clan Names

Longhouses are assigned clan names:
- Bear, Wolf, Turtle, Deer, Hawk
- Beaver, Eel, Heron, Snipe, Eagle

---

## World Integration

### File: `crates/croatoan_wfc/src/villages.rs`

This module handles village placement in the world and provides chunk-based structure streaming.

### Site Selection

```rust
pub fn find_village_sites(
    world_seed: u32,
    region_min: Vec3,
    region_max: Vec3,
    max_villages: u32,
) -> Vec<Vec3>
```

**Terrain Requirements:**
- Elevation: 3m - 60m (above water, below mountains)
- Slope: < 8m height difference across 80m
- Spacing: 400m minimum between villages
- Grid sampling: Every 200m

**Scoring Factors:**
- Flatness (flatter = better)
- Mid-elevation bonus
- Noise-based randomization

### World Village Structure

```rust
pub struct WorldVillage {
    pub id: VillageId,
    pub center: Vec3,
    pub layout: VillageLayout,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}
```

### Chunk Integration

```rust
pub fn get_village_structures_for_chunk(
    village: &WorldVillage,
    chunk_min_x: f32,
    chunk_min_z: f32,
    chunk_size: f32,
    world_seed: u32,
) -> Vec<VillageStructure>
```

Returns structures whose centers fall within the chunk bounds (with margin for large structures like longhouses).

### Structure Output Format

```rust
pub struct VillageStructure {
    pub structure_type: VillageStructureType,
    pub transform: Mat4,           // World transform
    pub mesh_vertices: Vec<f32>,   // Flattened: [px,py,pz, nx,ny,nz, u,v, r,g,b] × N
    pub mesh_indices: Vec<u32>,    // Triangle indices
}

pub enum VillageStructureType {
    Longhouse,
    FirePit,
    CornPlant,
    PrayerSite,
}
```

**Vertex stride:** 11 floats (44 bytes)
- Position: 3 floats
- Normal: 3 floats
- UV: 2 floats
- Color: 3 floats

---

## Rendering Integration (TODO)

To render villages, integrate with the existing building pipeline:

```rust
// In chunk generation:
for village in &world_villages {
    let structures = get_village_structures_for_chunk(
        village,
        chunk_offset_x,
        chunk_offset_z,
        chunk_size,
        world_seed,
    );

    for structure in structures {
        // Create GPU buffers from mesh_vertices and mesh_indices
        // Apply structure.transform
        // Render with building shader (same vertex format)
    }
}
```

---

## Future Work

### Phase 2: Behavior System
- [ ] Behavior tree executor
- [ ] Needs system (hunger, rest, harmony)
- [ ] Daily schedules
- [ ] Activity state machines

### Phase 3: Activities
- [ ] Corn tending animations
- [ ] Fire dancing with drum sync
- [ ] Prayer poses and transitions
- [ ] Conversation system

### Phase 4: Animation
- [ ] Skeletal animation system
- [ ] Walk/run cycles
- [ ] Work animations
- [ ] Dance animations

### Phase 5: Integration
- [ ] Player interaction
- [ ] Trading system
- [ ] Quest hooks
- [ ] Save/load NPC state

---

## Test Coverage

All modules have unit tests:

```bash
cargo test --package croatoan_procgen
# 17 tests: longhouse, npc, village, corn plants

cargo test --package croatoan_wfc villages
# 3 tests: site finding, village generation, chunk structures
```

**Example test output:**
```
Village 'Seneca' with 4 longhouses, 35 NPCs
Found 5 village sites
Found 9 structures in chunk: 4 Longhouses, 5 FirePits
```

---

## File Reference

| File | Lines | Purpose |
|------|-------|---------|
| `croatoan_procgen/src/longhouse.rs` | 475 | Longhouse mesh generation |
| `croatoan_procgen/src/npc.rs` | 545 | NPC data and mesh generation |
| `croatoan_procgen/src/village.rs` | 530 | Village layout, fire pits, corn |
| `croatoan_wfc/src/villages.rs` | 310 | World integration, chunk streaming |

---

*Document updated 2024-12-05 to reflect Phase 1 implementation.*
