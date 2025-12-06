//! Storm and Hurricane System
//!
//! Realistic hurricane simulation based on historical Atlantic storm patterns.
//! Includes storm tracking, intensity changes, and landfall effects.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Saffir-Simpson Hurricane Wind Scale
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HurricaneCategory {
    TropicalDepression,  // < 39 mph
    TropicalStorm,       // 39-73 mph
    Category1,           // 74-95 mph
    Category2,           // 96-110 mph
    Category3,           // 111-129 mph (Major)
    Category4,           // 130-156 mph
    Category5,           // > 157 mph
}

impl HurricaneCategory {
    pub fn from_wind_speed(mph: f32) -> Self {
        match mph as u32 {
            0..=38 => Self::TropicalDepression,
            39..=73 => Self::TropicalStorm,
            74..=95 => Self::Category1,
            96..=110 => Self::Category2,
            111..=129 => Self::Category3,
            130..=156 => Self::Category4,
            _ => Self::Category5,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::TropicalDepression => "Tropical Depression - Minimal damage",
            Self::TropicalStorm => "Tropical Storm - Some damage to foliage",
            Self::Category1 => "Category 1 Hurricane - Some damage",
            Self::Category2 => "Category 2 Hurricane - Extensive damage",
            Self::Category3 => "Category 3 Hurricane - Devastating damage",
            Self::Category4 => "Category 4 Hurricane - Catastrophic damage",
            Self::Category5 => "Category 5 Hurricane - Complete destruction",
        }
    }

    pub fn damage_multiplier(&self) -> f32 {
        match self {
            Self::TropicalDepression => 0.1,
            Self::TropicalStorm => 0.3,
            Self::Category1 => 0.5,
            Self::Category2 => 0.7,
            Self::Category3 => 0.85,
            Self::Category4 => 0.95,
            Self::Category5 => 1.0,
        }
    }

    pub fn storm_surge_feet(&self) -> (f32, f32) {
        match self {
            Self::TropicalDepression => (0.0, 1.0),
            Self::TropicalStorm => (1.0, 3.0),
            Self::Category1 => (4.0, 5.0),
            Self::Category2 => (6.0, 8.0),
            Self::Category3 => (9.0, 12.0),
            Self::Category4 => (13.0, 18.0),
            Self::Category5 => (18.0, 25.0),
        }
    }
}

/// Storm lifecycle phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StormPhase {
    Forming,       // Initial development
    Intensifying,  // Gaining strength
    Mature,        // Peak intensity
    Weakening,     // After landfall or over cool water
    Dissipating,   // Breaking apart
    Extratropical, // Transitioning to mid-latitude system
}

/// Active storm system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormSystem {
    pub name: String,
    pub category: HurricaneCategory,
    pub phase: StormPhase,

    // Position and movement
    pub position: [f32; 2],        // World coordinates
    pub heading: f32,              // Direction of movement (radians)
    pub forward_speed: f32,        // mph

    // Storm characteristics
    pub current_wind_speed: f32,   // Sustained winds in mph
    pub gust_factor: f32,          // Gusts as multiplier of sustained
    pub wind_direction: f32,       // Rotation direction
    pub eye_radius: f32,           // Radius of calm eye (miles)
    pub max_wind_radius: f32,      // Radius of max winds from center
    pub outer_radius: f32,         // Total storm radius

    // Intensity tracking
    pub min_pressure: f32,         // Central pressure in mb
    pub time_active: f32,          // Seconds since formation
    pub time_since_peak: f32,      // Seconds since peak intensity
    pub peak_intensity: f32,       // Highest wind speed reached

    // Landfall
    pub has_made_landfall: bool,
    pub time_since_landfall: f32,
    pub landfall_position: Option<[f32; 2]>,
}

impl StormSystem {
    pub fn new(name: &str, position: [f32; 2]) -> Self {
        Self {
            name: name.to_string(),
            category: HurricaneCategory::TropicalDepression,
            phase: StormPhase::Forming,
            position,
            heading: -PI / 4.0, // Northwest movement typical for Atlantic
            forward_speed: 12.0,
            current_wind_speed: 35.0,
            gust_factor: 1.25,
            wind_direction: 0.0,
            eye_radius: 5.0,
            max_wind_radius: 25.0,
            outer_radius: 150.0,
            min_pressure: 1005.0,
            time_active: 0.0,
            time_since_peak: 0.0,
            peak_intensity: 35.0,
            has_made_landfall: false,
            time_since_landfall: 0.0,
            landfall_position: None,
        }
    }

    /// Spawn a random storm appropriate for the season
    pub fn spawn_random(day_of_year: u32) -> Self {
        let names = [
            "Ana", "Bill", "Claudette", "Danny", "Elsa", "Fred", "Grace",
            "Henri", "Ida", "Julian", "Kate", "Larry", "Mindy", "Nicholas",
        ];

        let name_idx = (rand_float() * names.len() as f32) as usize;

        // Storms form off the coast
        let spawn_x = 2000.0 + rand_float() * 1000.0; // Far east
        let spawn_z = rand_float() * 2000.0;

        let mut storm = Self::new(names[name_idx], [spawn_x, spawn_z]);

        // Seasonal intensity variation
        let is_peak_season = (200..=280).contains(&day_of_year); // Mid-July to early October
        if is_peak_season && rand_float() < 0.3 {
            // Higher chance of major storm in peak season
            storm.current_wind_speed = 80.0 + rand_float() * 50.0;
        }

        storm
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time_active += delta_time;
        self.wind_direction += delta_time * 0.01; // Slow rotation

        // Move storm
        let hours = delta_time / 3600.0;
        let distance = self.forward_speed * hours;
        self.position[0] += self.heading.sin() * distance;
        self.position[1] += self.heading.cos() * distance;

        // Curve northwest then northeast (typical Atlantic track)
        if self.position[1] > 1500.0 {
            self.heading += 0.0001 * delta_time; // Recurve
        }

        // Update intensity based on phase
        match self.phase {
            StormPhase::Forming => {
                if self.time_active > 43200.0 { // 12 hours
                    self.phase = StormPhase::Intensifying;
                }
            }
            StormPhase::Intensifying => {
                // Intensify over warm water
                if !self.has_made_landfall {
                    let intensification = rand_float() * 5.0 * (delta_time / 3600.0);
                    self.current_wind_speed += intensification;
                    self.min_pressure -= intensification * 0.3;
                }

                if self.current_wind_speed > 100.0 && rand_float() < 0.001 * delta_time {
                    self.phase = StormPhase::Mature;
                    self.peak_intensity = self.current_wind_speed;
                }
            }
            StormPhase::Mature => {
                self.time_since_peak += delta_time;
                // Maintain intensity briefly
                if self.time_since_peak > 21600.0 || self.has_made_landfall {
                    self.phase = StormPhase::Weakening;
                }
            }
            StormPhase::Weakening => {
                let weakening = (2.0 + rand_float() * 3.0) * (delta_time / 3600.0);
                self.current_wind_speed -= weakening;
                self.min_pressure += weakening * 0.4;

                if self.current_wind_speed < 40.0 {
                    self.phase = StormPhase::Dissipating;
                }
            }
            StormPhase::Dissipating => {
                self.current_wind_speed -= 5.0 * (delta_time / 3600.0);
                self.outer_radius *= 0.999; // Shrinking
            }
            StormPhase::Extratropical => {
                // Convert to different system
                self.outer_radius *= 1.001; // Spreading out
                self.current_wind_speed -= 2.0 * (delta_time / 3600.0);
            }
        }

        // Update category
        self.category = HurricaneCategory::from_wind_speed(self.current_wind_speed);

        // Update eye size based on intensity
        self.eye_radius = 5.0 + (self.current_wind_speed - 50.0).max(0.0) * 0.1;
        self.max_wind_radius = self.eye_radius + 10.0 + rand_float() * 20.0;

        // Landfall handling
        if self.has_made_landfall {
            self.time_since_landfall += delta_time;
            // Rapid weakening over land
            self.current_wind_speed -= 10.0 * (delta_time / 3600.0);
        }
    }

    /// Check if storm affects a given position
    pub fn affects_position(&self, pos: [f32; 2]) -> bool {
        let dx = pos[0] - self.position[0];
        let dz = pos[1] - self.position[1];
        let distance = (dx * dx + dz * dz).sqrt();
        distance < self.outer_radius
    }

    /// Get wind speed at a specific position
    pub fn wind_at_position(&self, pos: [f32; 2]) -> f32 {
        let dx = pos[0] - self.position[0];
        let dz = pos[1] - self.position[1];
        let distance = (dx * dx + dz * dz).sqrt();

        if distance < self.eye_radius {
            // In the eye - calm
            5.0 + rand_float() * 10.0
        } else if distance < self.max_wind_radius {
            // Eyewall - maximum winds
            self.current_wind_speed * (0.9 + rand_float() * 0.1)
        } else if distance < self.outer_radius {
            // Outer bands - decreasing winds
            let factor = 1.0 - (distance - self.max_wind_radius) / (self.outer_radius - self.max_wind_radius);
            self.current_wind_speed * 0.3 + self.current_wind_speed * 0.7 * factor
        } else {
            0.0
        }
    }

    /// Get rain intensity at position (inches per hour)
    pub fn rain_at_position(&self, pos: [f32; 2]) -> f32 {
        let dx = pos[0] - self.position[0];
        let dz = pos[1] - self.position[1];
        let distance = (dx * dx + dz * dz).sqrt();

        if distance < self.eye_radius {
            0.0 // Dry eye
        } else if distance < self.max_wind_radius * 1.5 {
            // Eyewall - heaviest rain
            2.0 + rand_float() * 2.0
        } else if distance < self.outer_radius {
            // Outer bands
            let factor = 1.0 - distance / self.outer_radius;
            1.0 * factor + rand_float() * 0.5
        } else {
            0.0
        }
    }

    /// Check if storm has dissipated
    pub fn is_dissipated(&self) -> bool {
        self.current_wind_speed < 25.0 || self.outer_radius < 50.0
    }

    /// Get overall storm intensity (0-1)
    pub fn intensity(&self) -> f32 {
        (self.current_wind_speed / 157.0).min(1.0)
    }

    /// Get visibility in storm
    pub fn visibility(&self) -> f32 {
        match self.category {
            HurricaneCategory::TropicalDepression => 0.5,
            HurricaneCategory::TropicalStorm => 0.3,
            HurricaneCategory::Category1 => 0.2,
            HurricaneCategory::Category2 => 0.15,
            HurricaneCategory::Category3 => 0.1,
            HurricaneCategory::Category4 => 0.05,
            HurricaneCategory::Category5 => 0.02,
        }
    }

    /// Estimate time to reach a position (seconds)
    pub fn time_to_position(&self, target: [f32; 2]) -> f32 {
        let dx = target[0] - self.position[0];
        let dz = target[1] - self.position[1];
        let distance = (dx * dx + dz * dz).sqrt();

        // Rough estimate - doesn't account for track changes
        distance / (self.forward_speed / 3600.0)
    }

    /// Estimate time to landfall (if moving toward coast)
    pub fn time_to_landfall(&self) -> f32 {
        if self.has_made_landfall {
            return 0.0;
        }

        // Assume coast is at x = 0
        let distance_to_coast = self.position[0];
        let westward_speed = self.forward_speed * (-self.heading).sin();

        if westward_speed <= 0.0 {
            return f32::INFINITY;
        }

        distance_to_coast / (westward_speed / 3600.0)
    }

    /// Trigger landfall
    pub fn make_landfall(&mut self, position: [f32; 2]) {
        self.has_made_landfall = true;
        self.landfall_position = Some(position);
        self.time_since_landfall = 0.0;

        if self.phase == StormPhase::Mature || self.phase == StormPhase::Intensifying {
            self.phase = StormPhase::Weakening;
        }
    }
}

/// Historical record of a storm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormRecord {
    pub name: String,
    pub peak_category: HurricaneCategory,
    pub peak_wind_speed: f32,
    pub min_pressure: f32,
    pub duration_hours: f32,
    pub made_landfall: bool,
    pub landfall_position: Option<[f32; 2]>,
    pub damage_estimate: DamageLevel,
}

impl StormRecord {
    pub fn from_storm(storm: &StormSystem) -> Self {
        Self {
            name: storm.name.clone(),
            peak_category: HurricaneCategory::from_wind_speed(storm.peak_intensity),
            peak_wind_speed: storm.peak_intensity,
            min_pressure: storm.min_pressure,
            duration_hours: storm.time_active / 3600.0,
            made_landfall: storm.has_made_landfall,
            landfall_position: storm.landfall_position,
            damage_estimate: DamageLevel::from_category(
                HurricaneCategory::from_wind_speed(storm.peak_intensity),
                storm.has_made_landfall,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageLevel {
    None,
    Minor,
    Moderate,
    Significant,
    Severe,
    Catastrophic,
}

impl DamageLevel {
    pub fn from_category(cat: HurricaneCategory, made_landfall: bool) -> Self {
        if !made_landfall {
            return Self::None;
        }

        match cat {
            HurricaneCategory::TropicalDepression => Self::Minor,
            HurricaneCategory::TropicalStorm => Self::Moderate,
            HurricaneCategory::Category1 => Self::Moderate,
            HurricaneCategory::Category2 => Self::Significant,
            HurricaneCategory::Category3 => Self::Severe,
            HurricaneCategory::Category4 | HurricaneCategory::Category5 => Self::Catastrophic,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::None => "Storm passed without affecting land",
            Self::Minor => "Minor damage to vegetation and structures",
            Self::Moderate => "Some structural damage, downed trees",
            Self::Significant => "Significant structural damage, flooding",
            Self::Severe => "Major structural damage, dangerous flooding",
            Self::Catastrophic => "Catastrophic damage, area devastated",
        }
    }
}

fn rand_float() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}
