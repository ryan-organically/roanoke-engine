# Grass System Design: Monocot-First Flora Strategy

## Strategic Intuition

Grass represents the optimal entry point for establishing high-fidelity flora in the Roanoke Engine. This document captures the architectural reasoning and implementation strategy for a differentiated grass-encephalon system.

### Why Grass First?

**Botanical Simplicity (Monocotyledon Order)**
- Linear blade geometry: parallel venation, no branching complexity
- Simple vertex topology: ribbon meshes with predictable tapering
- Uniform growth pattern: base-to-tip gradients map directly to UV/color interpolation
- Joint structure: blade segments naturally represent node/internode morphology

**Infrastructure Readiness**
- `GrassBladeRecipe` already parameterizes: height, segments, curve, width, color
- Wind shader operational: dual sine waves with height-based amplitude
- Shadow mapping integrated: grass receives terrain shadows
- Biome hooks exist: `vegetation.rs` already queries height/biome for density

**Visual Dominance**
- Grass covers more screen real estate than any other flora type
- Establishes the "floral tone" of each biome immediately
- Low polygon cost per blade enables high density where needed

### Low Hanging Fruit Analysis

| System | Complexity | Visual Impact | Dependencies | Priority |
|--------|------------|---------------|--------------|----------|
| Grass Species Differentiation | Low | High | None | **1** |
| Wind Response Tuning | Low | High | Species system | 2 |
| Cattails (Typha) | Low | Medium | Grass system | 3 |
| Ferns (Pteridophyta) | Low-Medium | Medium | Blade generation | 4 |
| Ground Detritus | Low | Medium | Instancing | 5 |
| Wildflowers | Low | High | Color system | 6 |
| Shrub Silhouettes | Medium | High | Billboard/mesh | 7 |

### Compound Benefits

Once grass species are differentiated, adjacent systems unlock:

1. **Cattails** share blade geometry + add cylindrical spike primitive
2. **Rushes/Sedges** are near-identical monocots with minor parameter changes
3. **Ferns** extend the curved-blade pattern with fractal repetition
4. **Wildflowers** add color quads atop grass-like stems
5. **Detritus** (leaves, needles) reuses the instancing patterns

---

## V1 Grass Species Specifications

### 1. Sea Oats (Uniola paniculata) - Beach Dune Grass

**Habitat**: Beach biome, height 0.8-3.0m, dune crests and transitions
**Morphology**: Tall wispy stalks, strong graceful droop, bleached coloration
**Density**: 0.05-0.15 blades/m² (very sparse, wind-scattered clumps)

```
Height Range:    0.8 - 1.4m (tall emergent stalks)
Blade Segments:  5 (more joints for elegant curvature)
Curve Factor:    0.6 (strong bend, tips point downwind)
Width Base:      0.025m (thin wiry stalks)
Width Tip:       0.005m (very fine, almost hair-like)
Color Base:      RGB(0.55, 0.50, 0.35) - sandy tan
Color Tip:       RGB(0.70, 0.65, 0.45) - sun-bleached gold
Wind Response:   High amplitude, fast frequency (exposed to coastal wind)
```

**Visual Reference**: Barrier island dunes, Outer Banks aesthetic

### 2. Smooth Cordgrass (Spartina alterniflora) - Salt Marsh

**Habitat**: SaltMarsh biome, tidal zones, avoids channels and salt pans
**Morphology**: Dense upright clumps, robust blades, dark green
**Density**: 0.4-0.6 blades/m² (dense monoculture stands)

```
Height Range:    0.6 - 2.0m (varies by tidal elevation)
Blade Segments:  4 (stiffer, fewer articulation points)
Curve Factor:    0.25 (upright growth habit)
Width Base:      0.06m (thick robust blades)
Width Tip:       0.015m (tapers to sturdy point)
Color Base:      RGB(0.20, 0.40, 0.15) - dark marsh green
Color Tip:       RGB(0.35, 0.55, 0.25) - lighter photosynthetic green
Wind Response:   Low amplitude, slow frequency (stiff resistance)
```

**Visual Reference**: Chesapeake Bay marshes, Carolina lowcountry

### 3. Sawgrass (Cladium jamaicense) - Meadow/Grassland

**Habitat**: Grassland, CoastalScrub, open areas 3-15m elevation
**Morphology**: Medium height, flowing motion, classic meadow appearance
**Density**: 0.25-0.4 blades/m² (moderate coverage)

```
Height Range:    0.4 - 0.9m (medium meadow grass)
Blade Segments:  4 (balanced flexibility)
Curve Factor:    0.35 (gentle natural bend)
Width Base:      0.04m (standard blade width)
Width Tip:       0.01m (gradual taper)
Color Base:      RGB(0.25, 0.50, 0.18) - healthy green
Color Tip:       RGB(0.40, 0.65, 0.25) - sun-touched tips
Wind Response:   Medium amplitude, medium frequency (responsive)
```

**Visual Reference**: Virginia piedmont meadows, colonial-era pastures

### 4. Forest Floor Grass - Understory

**Habitat**: DeciduousForest, height > 6.0m, shaded understory
**Morphology**: Taller shade-adapted blades, darker coloration, drooping
**Density**: 0.3-0.5 blades/m² (patchy, follows light gaps)

```
Height Range:    0.6 - 1.2m (reaching for light)
Blade Segments:  4 (shade-drooping articulation)
Curve Factor:    0.45 (pronounced droop in low light)
Width Base:      0.05m (broader for light capture)
Width Tip:       0.012m (moderate taper)
Color Base:      RGB(0.15, 0.42, 0.12) - deep forest green
Color Tip:       RGB(0.28, 0.58, 0.22) - slightly lighter
Wind Response:   Low amplitude (sheltered by canopy)
```

**Visual Reference**: Appalachian forest floor, oak-hickory understory

### 5. Alpine Grass - Mountain Meadow

**Habitat**: AlpineMeadow, Foothills, elevation 50-100m
**Morphology**: Short, hardy, wind-resistant tufts
**Density**: 0.2-0.35 blades/m² (sparse, rocky soil)

```
Height Range:    0.2 - 0.5m (low wind-resistant profile)
Blade Segments:  3 (compact, sturdy)
Curve Factor:    0.2 (wind-trained, mostly upright)
Width Base:      0.035m (compact blades)
Width Tip:       0.008m (fine tips)
Color Base:      RGB(0.30, 0.48, 0.20) - mountain green
Color Tip:       RGB(0.45, 0.60, 0.30) - alpine gold-green
Wind Response:   Medium amplitude (constant exposure)
```

**Visual Reference**: Blue Ridge balds, high meadows

---

## Implementation Architecture

### File Structure

```
crates/croatoan_procgen/src/
├── grass.rs              # Existing blade generation primitives
├── grass_species.rs      # NEW: Species-specific recipes and selection
└── lib.rs                # Export grass_species module

crates/croatoan_wfc/src/
├── vegetation.rs         # MODIFY: Use species selection
└── biome.rs              # Reference for BiomeType matching
```

### Core Interface

```rust
// grass_species.rs

pub enum GrassSpecies {
    SeaOats,           // Beach dunes
    Cordgrass,         // Salt marsh
    Sawgrass,          // Meadow/grassland
    ForestFloor,       // Understory
    AlpineGrass,       // Mountain meadow
}

pub struct GrassSpeciesConfig {
    pub recipe: GrassBladeRecipe,
    pub density_range: (f32, f32),
    pub wind_amplitude: f32,
    pub wind_frequency: f32,
    pub clumping_factor: f32,
}

pub fn get_species_for_biome(
    biome: BiomeType,
    height: f32,
    moisture: f32,
) -> GrassSpecies;

pub fn get_species_config(species: GrassSpecies) -> GrassSpeciesConfig;
```

### Biome Selection Logic

```rust
fn get_species_for_biome(biome: BiomeType, height: f32, moisture: f32) -> GrassSpecies {
    match biome {
        BiomeType::Beach => GrassSpecies::SeaOats,
        BiomeType::SaltMarsh => GrassSpecies::Cordgrass,
        BiomeType::Grassland | BiomeType::CoastalScrub => GrassSpecies::Sawgrass,
        BiomeType::DeciduousForest => GrassSpecies::ForestFloor,
        BiomeType::AlpineMeadow | BiomeType::Foothills => GrassSpecies::AlpineGrass,
        BiomeType::Wetland => {
            if moisture > 0.7 { GrassSpecies::Cordgrass }
            else { GrassSpecies::Sawgrass }
        }
        _ => GrassSpecies::Sawgrass, // Default fallback
    }
}
```

---

## Wind Response System

Each species has characteristic wind behavior encoded in the shader via per-species uniforms or vertex attributes:

| Species | Amplitude | Frequency | Character |
|---------|-----------|-----------|-----------|
| Sea Oats | 0.25 | 2.5 | Dramatic coastal sway |
| Cordgrass | 0.08 | 1.2 | Stiff resistance |
| Sawgrass | 0.15 | 2.0 | Flowing meadow waves |
| Forest Floor | 0.06 | 1.5 | Gentle sheltered motion |
| Alpine Grass | 0.12 | 2.2 | Constant buffeting |

Future enhancement: Pass wind parameters per-chunk or per-instance for species-specific behavior without shader branching.

---

## Integration with Existing Systems

### Biome System (biome.rs)
- BiomeType enum provides primary habitat classification
- BiomeData.moisture/temperature enable environmental modifiers
- TerrainFeature flags (TidalChannel, SaltPan) exclude grass from water

### Vegetation System (vegetation.rs)
- Currently uses height as biome proxy
- Will be updated to query actual BiomeType
- Density modifiers preserved, now species-aware

### Flora System (flora/mod.rs)
- 85 existing species definitions (trees, shrubs, herbs)
- Grass species complement rather than replace
- Shared habitat classification vocabulary

### Rendering Pipeline (grass_pipeline.rs)
- No changes required for V1
- Future: per-species wind uniforms
- Future: instanced rendering for higher density

---

## Performance Considerations

Current grass system is heavily optimized:
- Density reduced from 8.0 to 0.3 blades/m² (26x reduction)
- Segments reduced from 5 to 3-4 per blade
- Validation prevents GPU crashes from bad geometry

Species differentiation maintains these constraints:
- Sea Oats: Very low density compensates for extra segments
- Cordgrass: High density but fewer segments
- All species: Within 500K vertex / 1.5M index limits per chunk

---

## Future Extensions

### Phase 2: Related Monocots
- **Cattails (Typha)**: Blade + cylindrical spike primitive
- **Bulrush (Scirpus)**: Triangular stem cross-section
- **Sedges (Carex)**: "Sedges have edges" - triangular blades

### Phase 3: Ground Cover
- **Ferns**: Fractal frond generation from blade primitives
- **Moss**: Low carpet with normal-map detail
- **Lichen**: Rock surface coverage

### Phase 4: Flowering
- **Wildflowers**: Color quads on grass-like stems
- **Seasonal variation**: Bloom timing per species

---

## References

- Existing `GrassBladeRecipe` in `crates/croatoan_procgen/src/grass.rs`
- Biome definitions in `crates/croatoan_wfc/src/biome.rs`
- Vegetation generation in `crates/croatoan_wfc/src/vegetation.rs`
- Wind shader in `assets/shaders/grass.wgsl`
