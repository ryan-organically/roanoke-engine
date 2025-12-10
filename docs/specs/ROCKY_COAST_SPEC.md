# Rocky Coast Generation Specification

## Overview

The Rocky Coast system generates dramatic boulder-strewn shorelines as an alternative to sandy beaches. Rocky coasts feature clustered boulders, tide pools, sea stacks, and wave-carved formations that create visually striking and gameplay-rich coastal environments.

## Design Philosophy

Rocky coasts occur where harder geological substrates meet the sea - areas resistant to erosion that form dramatic boulder fields rather than sandy beaches. These coastlines are characterized by:

- **Wave-rounded boulders** clustered at the waterline
- **Tidal zones** with distinct rock formations at different elevations
- **Sea stacks** - isolated tall rock formations offshore
- **Tide pools** - depressions between rocks that hold water
- **Drift lines** of smaller cobbles marking high tide

## Biome Integration

Rocky coasts occupy the same `biome_t` range as beaches (0.45-0.58) but are distinguished by a secondary **coastal_type** noise layer:

```
biome_t 0.45-0.48: Water's edge
biome_t 0.48-0.52: Intertidal zone (primary boulder field)
biome_t 0.52-0.58: Supratidal zone (scattered boulders, coastal scrub)

coastal_type > 0.6: Rocky coast
coastal_type < 0.4: Sandy beach
coastal_type 0.4-0.6: Mixed (cobble beach)
```

## Boulder Classification

### Primary Types

| Type | Base Scale | Sink | Description |
|------|------------|------|-------------|
| `SeaBoulder` | 2.5-4.0m | 0.4 | Massive wave-rounded boulders |
| `TidalRock` | 1.0-2.0m | 0.3 | Mid-sized rocks in intertidal zone |
| `Cobble` | 0.3-0.6m | 0.15 | Rounded stones above tide line |
| `SeaStack` | 3.0-6.0m | 0.6 | Tall isolated offshore formations |
| `ShelfRock` | 1.5-3.0m | 0.5 | Flat-topped rocks (tide pool bases) |
| `WaveCarved` | 1.2-2.5m | 0.35 | Rocks with erosion patterns |

### Visual Properties

```rust
pub struct CoastalBoulderProperties {
    /// Base color influenced by:
    /// - Barnacle coverage (white-grey below high tide)
    /// - Seaweed staining (green-brown in tidal zone)
    /// - Lichen growth (orange-grey above spray zone)
    pub zone_tint: ZoneTint,

    /// Wetness factor (0.0-1.0) based on tide simulation
    pub wetness: f32,

    /// Roughness variation for wave-worn vs protected faces
    pub erosion_factor: f32,
}

pub enum ZoneTint {
    Submerged,      // Dark, clean rock
    Barnacle,       // White-grey encrustation
    Seaweed,        // Brown-green staining
    SprayZone,      // Salt-bleached, lichen spots
    Supratidal,     // Natural rock color
}
```

## Generation Zones

### Zone 1: Subtidal (height < 0.3)

Permanently submerged boulders visible through water:

- **Sea Stacks**: 1-3 per 256m chunk, offshore (biome_t 0.42-0.46)
- **Submerged Boulders**: Density 0.03/m², SeaBoulder type
- Spacing: minimum 8m between large formations

### Zone 2: Intertidal (height 0.3-1.2)

Primary boulder field - the visual heart of rocky coasts:

- **Boulder Clusters**: 3-6 per chunk
  - Each cluster: 1 anchor boulder (SeaBoulder) + 4-8 TidalRocks + 12-20 Cobbles
  - Cluster radius: 8-15m
  - Arranged in wave-shadow patterns (smaller rocks behind larger)

- **Shelf Formations**: 1-2 per chunk
  - Flat-topped ShelfRock groups forming tide pool basins
  - 3-5 ShelfRocks per formation

- **Scatter**: TidalRock density 0.08/m² between clusters

### Zone 3: High Tide Line (height 1.2-2.0)

Drift line and spray zone:

- **Cobble Banks**: Linear accumulations parallel to shore
  - Density: 0.4/m² in 2-3m wide bands
  - Sorted by size (larger toward water)

- **Isolated Boulders**: Density 0.02/m², WaveCarved type
  - Often with lichen tinting

### Zone 4: Supratidal (height 2.0-4.0)

Transition to coastal scrub:

- **Scattered Boulders**: Density 0.01/m², mix of types
- **Boulder-Vegetation Clusters**: Integration with existing LowlandBunch
  - Replace bunch anchor rock with SeaBoulder when coastal_type > 0.6

## Cluster Generation Algorithm

```rust
pub struct BoulderCluster {
    pub center: Vec3,
    pub anchor: SeaBoulder,
    pub satellites: Vec<TidalRock>,
    pub cobbles: Vec<Cobble>,
    pub wave_direction: Vec2,  // Dominant wave approach
}

impl BoulderCluster {
    pub fn generate(center: Vec3, seed: u32) -> Self {
        let wave_dir = get_wave_direction(center, seed);

        // Anchor boulder faces waves
        let anchor = SeaBoulder::new(center, seed)
            .with_rotation_facing(-wave_dir);

        // Satellites cluster in wave shadow
        let shadow_cone = Cone::new(center, wave_dir, 60.0.to_radians());
        let satellites = (0..rng.gen_range(4..9))
            .map(|i| {
                let pos = shadow_cone.random_point(seed + i, 3.0..12.0);
                TidalRock::new(pos, seed + i)
            })
            .collect();

        // Cobbles fill gaps, denser in protected areas
        let cobbles = generate_cobble_fill(center, &satellites, seed);

        Self { center, anchor, satellites, cobbles, wave_direction: wave_dir }
    }
}
```

## Sea Stack Generation

Sea stacks are tall isolated rock formations that create dramatic silhouettes:

```rust
pub struct SeaStack {
    pub base_position: Vec3,
    pub height: f32,           // 3.0-8.0m above water
    pub base_radius: f32,      // 2.0-4.0m
    pub taper: f32,            // 0.3-0.7 (top radius / base radius)
    pub lean_angle: f32,       // Slight tilt for character
    pub erosion_notch: bool,   // Wave-carved notch at waterline
}

pub fn generate_sea_stacks(chunk: ChunkCoord, seed: u32) -> Vec<SeaStack> {
    let mut stacks = Vec::new();

    // Poisson disk sampling for natural spacing
    let candidates = poisson_disk_2d(chunk.bounds(), 40.0, seed);

    for pos in candidates {
        let biome_t = get_biome_t(pos.x, pos.z, seed);
        let coastal = get_coastal_type(pos.x, pos.z, seed);
        let height = get_height_at(pos.x, pos.z, seed).0;

        // Sea stacks only in rocky coast, shallow water
        if coastal > 0.55 && biome_t < 0.46 && height > -2.0 && height < 0.5 {
            let stack_height = lerp(3.0, 8.0, noise(pos, seed + 100));
            stacks.push(SeaStack {
                base_position: Vec3::new(pos.x, height, pos.z),
                height: stack_height,
                base_radius: lerp(2.0, 4.0, noise(pos, seed + 101)),
                taper: lerp(0.3, 0.7, noise(pos, seed + 102)),
                lean_angle: lerp(-5.0, 5.0, noise(pos, seed + 103)).to_radians(),
                erosion_notch: noise(pos, seed + 104) > 0.4,
            });
        }
    }

    stacks
}
```

## Tide Pool System

Tide pools form in depressions between ShelfRock formations:

```rust
pub struct TidePool {
    pub center: Vec3,
    pub radius: f32,           // 0.5-3.0m
    pub depth: f32,            // 0.1-0.4m
    pub boundary_rocks: Vec<ShelfRock>,
}

pub fn generate_tide_pools(
    shelf_formations: &[ShelfFormation],
    seed: u32
) -> Vec<TidePool> {
    shelf_formations.iter()
        .filter_map(|formation| {
            // Find natural depressions in rock arrangement
            let depression = find_central_depression(formation);

            if depression.depth > 0.1 {
                Some(TidePool {
                    center: depression.center,
                    radius: depression.radius.min(3.0),
                    depth: depression.depth,
                    boundary_rocks: formation.rocks.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}
```

## Data Structures

```rust
/// Coastal type classification
#[derive(Clone, Copy, Debug)]
pub enum CoastalType {
    Sandy,          // coastal_type < 0.4
    Cobble,         // coastal_type 0.4-0.6
    Rocky,          // coastal_type > 0.6
}

/// Boulder instance for rendering
#[derive(Clone, Debug)]
pub struct CoastalBoulderInstance {
    pub transform: Mat4,
    pub boulder_type: BoulderType,
    pub zone_tint: ZoneTint,
    pub wetness: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum BoulderType {
    SeaBoulder,
    TidalRock,
    Cobble,
    SeaStack,
    ShelfRock,
    WaveCarved,
}

impl BoulderType {
    pub fn base_scale(&self) -> (f32, f32) {
        match self {
            Self::SeaBoulder => (2.5, 4.0),
            Self::TidalRock => (1.0, 2.0),
            Self::Cobble => (0.3, 0.6),
            Self::SeaStack => (3.0, 6.0),
            Self::ShelfRock => (1.5, 3.0),
            Self::WaveCarved => (1.2, 2.5),
        }
    }

    pub fn sink_amount(&self) -> f32 {
        match self {
            Self::SeaBoulder => 0.4,
            Self::TidalRock => 0.3,
            Self::Cobble => 0.15,
            Self::SeaStack => 0.6,
            Self::ShelfRock => 0.5,
            Self::WaveCarved => 0.35,
        }
    }
}

/// Main generation output
pub struct RockyCoastChunk {
    pub boulder_instances: Vec<CoastalBoulderInstance>,
    pub sea_stacks: Vec<SeaStack>,
    pub tide_pools: Vec<TidePool>,
    pub clusters: Vec<BoulderCluster>,
}
```

## Integration with Existing Systems

### Mesh Generation (mesh_gen.rs)

Add `get_coastal_type()` function:

```rust
pub fn get_coastal_type(x: f32, z: f32, seed: u32) -> f32 {
    let coastal_noise = Perlin::new(seed + 7777);

    // Low frequency for large coastal regions
    let base = coastal_noise.get([x as f64 * 0.0008, z as f64 * 0.0008]) as f32;

    // Medium frequency for coastline variation
    let detail = coastal_noise.get([x as f64 * 0.004, z as f64 * 0.004]) as f32;

    ((base * 0.7 + detail * 0.3) * 0.5 + 0.5).clamp(0.0, 1.0)
}
```

### Rock Generation (rocks.rs)

Modify `generate_rocks_for_chunk()` to delegate to rocky coast system:

```rust
pub fn generate_rocks_for_chunk(...) -> Vec<Mat4> {
    let coastal_type = get_coastal_type(offset_x + chunk_size/2.0, offset_z + chunk_size/2.0, seed);

    if coastal_type > 0.6 {
        // Use rocky coast generation
        let rocky_chunk = generate_rocky_coast_chunk(seed, chunk_size, offset_x, offset_z);
        return rocky_chunk.to_transforms();
    }

    // Existing generation for sandy/mixed coasts
    // ...
}
```

### LowlandBunch Integration

When in rocky coastal zone, modify bunch generation:

```rust
impl LowlandBunch {
    pub fn generate(&self, seed: u32) -> BunchInstances {
        let coastal = get_coastal_type(self.center.x, self.center.z, seed);

        if coastal > 0.6 && self.biome_factor < 0.6 {
            // Rocky coast variant: larger anchor rock, more pebbles
            return self.generate_rocky_variant(seed);
        }

        // Standard generation
        // ...
    }
}
```

## Shader Considerations

### Zone Tinting

```wgsl
fn get_zone_tint(world_pos: vec3<f32>, base_color: vec3<f32>) -> vec3<f32> {
    let height = world_pos.y;

    if height < 0.3 {
        // Submerged: darken
        return base_color * 0.7;
    } else if height < 0.8 {
        // Barnacle zone: white-grey spots
        let barnacle = noise(world_pos * 4.0);
        return mix(base_color, vec3(0.85, 0.82, 0.78), barnacle * 0.4);
    } else if height < 1.2 {
        // Seaweed zone: brown-green tint
        return mix(base_color, vec3(0.3, 0.35, 0.25), 0.3);
    } else if height < 2.0 {
        // Spray zone: salt bleaching + lichen
        let lichen = noise(world_pos * 8.0);
        let bleached = mix(base_color, vec3(0.9), 0.15);
        return mix(bleached, vec3(0.8, 0.6, 0.3), lichen * 0.2);
    }

    return base_color;
}
```

### Wetness

```wgsl
fn apply_wetness(color: vec3<f32>, wetness: f32) -> vec3<f32> {
    // Wet rocks are darker and more saturated
    let darkened = color * (1.0 - wetness * 0.3);
    let saturated = mix(vec3(dot(darkened, vec3(0.299, 0.587, 0.114))), darkened, 1.0 + wetness * 0.2);
    return saturated;
}
```

## Implementation Phases

### Phase 1: Core Infrastructure
- [ ] Add `get_coastal_type()` to mesh_gen.rs
- [ ] Create `rocky_coast.rs` module in croatoan_wfc
- [ ] Define BoulderType enum and properties
- [ ] Basic boulder instance generation

### Phase 2: Cluster Generation
- [ ] Implement BoulderCluster struct and generation
- [ ] Wave direction calculation
- [ ] Shadow-cone satellite placement
- [ ] Cobble fill algorithm

### Phase 3: Sea Stacks & Tide Pools
- [ ] Sea stack generation with Poisson disk sampling
- [ ] Shelf formation placement
- [ ] Tide pool detection algorithm

### Phase 4: Visual Polish
- [ ] Zone tinting in shader
- [ ] Wetness system (static initially)
- [ ] Erosion patterns on WaveCarved type

### Phase 5: Integration & Tuning
- [ ] Hook into existing rock generation
- [ ] LowlandBunch rocky variant
- [ ] Density and spacing tuning
- [ ] Performance optimization (LOD, culling)

## Performance Considerations

- **Instance Batching**: Group boulders by type for efficient rendering
- **LOD System**: Reduce cobble density at distance
- **Chunk Caching**: Cache coastal_type values per chunk
- **Culling**: Skip boulder generation for chunks without rocky coast

## Density Summary

| Zone | Primary Type | Density | Notes |
|------|--------------|---------|-------|
| Subtidal | SeaBoulder | 0.03/m² | Plus 1-3 sea stacks |
| Intertidal | Clusters | 3-6/chunk | ~50-80 rocks per cluster |
| High Tide | Cobble | 0.4/m² | In 2-3m bands |
| Supratidal | Mixed | 0.01/m² | Blends with scrub |

## Future Enhancements

- **Dynamic Tides**: Wetness varies with time-of-day tide cycle
- **Seaweed Props**: Kelp and seaweed attached to intertidal rocks
- **Tide Pool Life**: Visual details in tide pools (anemones, small fish)
- **Wave Spray Particles**: Particles when waves hit sea stacks
- **Climbable Boulders**: Gameplay integration for traversal
