//! Grass Species System
//!
//! Provides biome-specific grass configurations based on monocotyledon morphology.
//! Each species has characteristic blade geometry, coloration, density, and wind response.

use crate::grass::GrassBladeRecipe;

/// Distinct grass species adapted to different biomes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrassSpecies {
    /// Sea Oats (Uniola paniculata) - Beach dune colonizer
    /// Tall wispy stalks, strong graceful droop, bleached coloration
    SeaOats,

    /// Smooth Cordgrass (Spartina alterniflora) - Salt marsh dominant
    /// Dense upright clumps, robust blades, dark green
    Cordgrass,

    /// Sawgrass (Cladium jamaicense) - Meadow/grassland
    /// Medium height, flowing motion, classic meadow appearance
    Sawgrass,

    /// Forest floor grass - Shade-adapted understory
    /// Taller blades reaching for light, darker coloration
    ForestFloor,

    /// Alpine grass - Mountain meadow hardy grass
    /// Short wind-resistant tufts, compact growth
    AlpineGrass,
}

/// Complete configuration for a grass species including rendering parameters
#[derive(Debug, Clone)]
pub struct GrassSpeciesConfig {
    /// The blade generation recipe
    pub recipe: GrassBladeRecipe,

    /// Density range (min, max) in blades per square meter
    pub density_range: (f32, f32),

    /// Wind animation amplitude multiplier (0.0 = still, 1.0 = maximum sway)
    pub wind_amplitude: f32,

    /// Wind animation frequency multiplier (affects oscillation speed)
    pub wind_frequency: f32,

    /// Clumping factor - how much grass clusters together (0.0 = uniform, 1.0 = highly clumped)
    pub clumping_factor: f32,

    /// Whether this species can grow in partially submerged conditions
    pub tolerates_water: bool,
}

impl GrassSpecies {
    /// Get all available grass species
    pub fn all() -> &'static [GrassSpecies] {
        &[
            GrassSpecies::SeaOats,
            GrassSpecies::Cordgrass,
            GrassSpecies::Sawgrass,
            GrassSpecies::ForestFloor,
            GrassSpecies::AlpineGrass,
        ]
    }

    /// Get the common name for display purposes
    pub fn common_name(&self) -> &'static str {
        match self {
            GrassSpecies::SeaOats => "Sea Oats",
            GrassSpecies::Cordgrass => "Smooth Cordgrass",
            GrassSpecies::Sawgrass => "Sawgrass",
            GrassSpecies::ForestFloor => "Forest Floor Grass",
            GrassSpecies::AlpineGrass => "Alpine Grass",
        }
    }

    /// Get the scientific name
    pub fn scientific_name(&self) -> &'static str {
        match self {
            GrassSpecies::SeaOats => "Uniola paniculata",
            GrassSpecies::Cordgrass => "Spartina alterniflora",
            GrassSpecies::Sawgrass => "Cladium jamaicense",
            GrassSpecies::ForestFloor => "Carex spp.",
            GrassSpecies::AlpineGrass => "Deschampsia cespitosa",
        }
    }
}

/// Get the complete configuration for a grass species
pub fn get_species_config(species: GrassSpecies) -> GrassSpeciesConfig {
    match species {
        GrassSpecies::SeaOats => sea_oats_config(),
        GrassSpecies::Cordgrass => cordgrass_config(),
        GrassSpecies::Sawgrass => sawgrass_config(),
        GrassSpecies::ForestFloor => forest_floor_config(),
        GrassSpecies::AlpineGrass => alpine_grass_config(),
    }
}

/// Select the appropriate grass species for a given biome and conditions
///
/// # Arguments
/// * `biome` - The primary biome type (uses string matching for cross-crate compatibility)
/// * `height` - Terrain height in meters
/// * `moisture` - Moisture level 0.0 (arid) to 1.0 (saturated)
///
/// # Returns
/// The most appropriate grass species for the conditions
pub fn get_species_for_conditions(biome: &str, height: f32, moisture: f32) -> GrassSpecies {
    match biome {
        "Beach" => GrassSpecies::SeaOats,
        "SaltMarsh" => GrassSpecies::Cordgrass,
        "Wetland" => {
            if moisture > 0.7 {
                GrassSpecies::Cordgrass
            } else {
                GrassSpecies::Sawgrass
            }
        }
        "Grassland" | "CoastalScrub" => GrassSpecies::Sawgrass,
        "DeciduousForest" => GrassSpecies::ForestFloor,
        "AlpineMeadow" | "Foothills" => GrassSpecies::AlpineGrass,
        "RollingMountains" => {
            if height > 80.0 {
                GrassSpecies::AlpineGrass
            } else {
                GrassSpecies::Sawgrass
            }
        }
        // Ocean, River, MountainPeak, Cave - minimal grass
        _ => GrassSpecies::Sawgrass, // Default fallback
    }
}

/// Calculate effective density based on species and environmental factors
pub fn calculate_density(
    species: GrassSpecies,
    moisture: f32,
    temperature: f32,
    slope: f32,
) -> f32 {
    let config = get_species_config(species);
    let base_density = (config.density_range.0 + config.density_range.1) * 0.5;

    // Moisture modifier
    let moisture_mod = match species {
        GrassSpecies::Cordgrass => 0.5 + moisture * 0.5, // Loves water
        GrassSpecies::SeaOats => 1.0 - moisture * 0.3,   // Prefers drier dunes
        GrassSpecies::ForestFloor => 0.7 + moisture * 0.3,
        _ => 0.8 + moisture * 0.2,
    };

    // Temperature modifier (colonial Virginia climate assumed)
    let temp_mod = 0.8 + temperature * 0.2;

    // Slope modifier - less grass on steep slopes
    let slope_mod = (1.0 - slope * 0.5).max(0.2);

    (base_density * moisture_mod * temp_mod * slope_mod).clamp(
        config.density_range.0,
        config.density_range.1,
    )
}

// ============================================================================
// Species Configuration Functions
// ============================================================================

/// Sea Oats - Beach dune grass
/// Tall wispy stalks that colonize and stabilize dunes
fn sea_oats_config() -> GrassSpeciesConfig {
    GrassSpeciesConfig {
        recipe: GrassBladeRecipe {
            height_range: (0.6, 2.2),   // Extended height variance for visual interest
            blade_segments: 5,
            curve_factor: 0.6,          // Graceful droop
            width_base: 0.04,
            width_tip: 0.01,
            color_base: [0.45, 0.40, 0.28],  // Darker sandy tan base
            color_tip: [0.55, 0.50, 0.35],   // Darker straw tip
        },
        density_range: (0.3, 0.6),    // Sparse but visible dune colonizer
        wind_amplitude: 1.2,
        wind_frequency: 1.3,
        clumping_factor: 0.6,         // Grows in tufts
        tolerates_water: false,
    }
}

/// Smooth Cordgrass - Salt marsh dominant
/// Dense monoculture in tidal zones
fn cordgrass_config() -> GrassSpeciesConfig {
    GrassSpeciesConfig {
        recipe: GrassBladeRecipe {
            height_range: (0.6, 2.0), // Highly variable by tidal zone
            blade_segments: 4,        // Stiffer, fewer bends
            curve_factor: 0.25,       // More upright growth
            width_base: 0.06,         // Thick robust blades
            width_tip: 0.015,         // Tapers to sturdy point
            color_base: [0.10, 0.28, 0.08], // Darker marsh green
            color_tip: [0.18, 0.38, 0.14],  // Darker tip
        },
        density_range: (3.0, 5.0), // Very dense monoculture stands
        wind_amplitude: 0.4,       // Stiff resistance
        wind_frequency: 0.8,       // Slow heavy sway
        clumping_factor: 0.3,      // Relatively uniform coverage
        tolerates_water: true,     // Adapted to tidal flooding
    }
}

/// Sawgrass - Meadow and grassland
/// The quintessential flowing meadow grass
fn sawgrass_config() -> GrassSpeciesConfig {
    GrassSpeciesConfig {
        recipe: GrassBladeRecipe {
            height_range: (0.4, 0.8),   // Medium height meadow grass
            blade_segments: 4,
            curve_factor: 0.35,         // Gentle flowing curve
            width_base: 0.04,
            width_tip: 0.01,
            color_base: [0.12, 0.30, 0.08],  // Darker meadow green
            color_tip: [0.22, 0.42, 0.15],   // Darker green tip
        },
        density_range: (2.0, 3.5),    // Dense meadow coverage
        wind_amplitude: 0.8,
        wind_frequency: 1.0,
        clumping_factor: 0.4,
        tolerates_water: false,
    }
}

/// Forest Floor - Shade-adapted understory grass
/// Reaches upward for light gaps - very dense at treeline
fn forest_floor_config() -> GrassSpeciesConfig {
    GrassSpeciesConfig {
        recipe: GrassBladeRecipe {
            height_range: (0.6, 1.2), // Taller, reaching for light
            blade_segments: 4,
            curve_factor: 0.45, // Pronounced droop in low light
            width_base: 0.05,   // Broader for light capture
            width_tip: 0.012,
            color_base: [0.08, 0.28, 0.06], // Darker deep forest green
            color_tip: [0.15, 0.40, 0.12],  // Darker tip
        },
        density_range: (4.0, 6.0), // Very dense - lush forest floor
        wind_amplitude: 0.3,       // Sheltered by canopy
        wind_frequency: 0.7,       // Gentle movement
        clumping_factor: 0.6,      // Grows in patches
        tolerates_water: false,
    }
}

/// Alpine Grass - Mountain meadow
/// Hardy, wind-resistant tufts
fn alpine_grass_config() -> GrassSpeciesConfig {
    GrassSpeciesConfig {
        recipe: GrassBladeRecipe {
            height_range: (0.2, 0.5), // Low wind-resistant profile
            blade_segments: 3,        // Compact, sturdy
            curve_factor: 0.2,        // Wind-trained, mostly upright
            width_base: 0.035,
            width_tip: 0.008,
            color_base: [0.18, 0.32, 0.10], // Darker mountain green
            color_tip: [0.28, 0.42, 0.18],  // Darker alpine green
        },
        density_range: (1.5, 2.5),  // Moderate density in meadows
        wind_amplitude: 0.6,        // Constant exposure
        wind_frequency: 1.2,        // Persistent buffeting
        clumping_factor: 0.5,       // Grows in tussocks
        tolerates_water: false,
    }
}

// ============================================================================
// Biome Integration Helpers
// ============================================================================

/// Determines if grass should grow at a given location based on terrain features
pub fn should_spawn_grass(
    biome: &str,
    height: f32,
    _moisture: f32,
    is_water: bool,
    is_rock: bool,
) -> bool {
    // Never spawn in water or on bare rock
    if is_water || is_rock {
        return false;
    }

    match biome {
        "Ocean" => false,
        "River" => false,
        "MountainPeak" => height < 120.0, // Only below snowline
        "Cave" => false,
        "Beach" => height > 0.8 && height < 4.0, // Only on dunes, not wet sand
        "SaltMarsh" => !is_water, // Except in channels
        _ => true,
    }
}

/// Get wind parameters formatted for shader uniform
/// Returns (amplitude, frequency) tuple
pub fn get_wind_parameters(species: GrassSpecies) -> (f32, f32) {
    let config = get_species_config(species);
    (config.wind_amplitude, config.wind_frequency)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_configs_valid() {
        for species in GrassSpecies::all() {
            let config = get_species_config(*species);

            // Validate recipe ranges
            assert!(config.recipe.height_range.0 > 0.0);
            assert!(config.recipe.height_range.1 >= config.recipe.height_range.0);
            assert!(config.recipe.blade_segments >= 2);
            assert!(config.recipe.width_base > config.recipe.width_tip);

            // Validate density
            assert!(config.density_range.0 > 0.0);
            assert!(config.density_range.1 >= config.density_range.0);

            // Validate wind params
            assert!(config.wind_amplitude >= 0.0);
            assert!(config.wind_frequency > 0.0);
        }
    }

    #[test]
    fn test_biome_species_selection() {
        assert_eq!(
            get_species_for_conditions("Beach", 2.0, 0.3),
            GrassSpecies::SeaOats
        );
        assert_eq!(
            get_species_for_conditions("SaltMarsh", 1.0, 0.8),
            GrassSpecies::Cordgrass
        );
        assert_eq!(
            get_species_for_conditions("DeciduousForest", 10.0, 0.5),
            GrassSpecies::ForestFloor
        );
        assert_eq!(
            get_species_for_conditions("AlpineMeadow", 60.0, 0.4),
            GrassSpecies::AlpineGrass
        );
    }

    #[test]
    fn test_density_calculation() {
        let density = calculate_density(GrassSpecies::Cordgrass, 0.9, 0.7, 0.0);
        let config = get_species_config(GrassSpecies::Cordgrass);

        // Should be within range
        assert!(density >= config.density_range.0);
        assert!(density <= config.density_range.1);

        // High moisture should increase cordgrass density
        let low_moisture = calculate_density(GrassSpecies::Cordgrass, 0.2, 0.7, 0.0);
        let high_moisture = calculate_density(GrassSpecies::Cordgrass, 0.9, 0.7, 0.0);
        assert!(high_moisture > low_moisture);
    }

    #[test]
    fn test_spawn_conditions() {
        assert!(!should_spawn_grass("Ocean", 0.0, 1.0, true, false));
        assert!(!should_spawn_grass("Beach", 0.3, 0.5, false, false)); // Wet sand
        assert!(should_spawn_grass("Beach", 1.5, 0.3, false, false)); // Dune
        assert!(should_spawn_grass("Grassland", 5.0, 0.5, false, false));
        assert!(!should_spawn_grass("DeciduousForest", 10.0, 0.5, false, true)); // Rock
    }
}
