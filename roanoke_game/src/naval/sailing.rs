//! Sailing mechanics and wind system
//!
//! Realistic Age of Sail wind mechanics including:
//! - Points of sail
//! - Tacking and wearing
//! - Weather effects on sailing
//! - Navigation

use serde::{Deserialize, Serialize};
use super::Ship;
use std::f32::consts::PI;

/// Wind conditions affecting sailing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindConditions {
    /// Wind direction in radians (0 = from North)
    pub direction: f32,
    /// Wind speed in knots
    pub speed: f32,
    /// Variability (gusting)
    pub variability: f32,
    /// Sea state (wave height factor)
    pub sea_state: SeaState,
}

impl Default for WindConditions {
    fn default() -> Self {
        Self {
            direction: PI / 4.0, // NE wind
            speed: 12.0,         // Moderate
            variability: 0.1,
            sea_state: SeaState::Moderate,
        }
    }
}

impl WindConditions {
    /// Get wind direction as compass bearing
    pub fn bearing(&self) -> &'static str {
        let deg = (self.direction.to_degrees() + 360.0) % 360.0;
        match deg as u32 {
            338..=360 | 0..=22 => "N",
            23..=67 => "NE",
            68..=112 => "E",
            113..=157 => "SE",
            158..=202 => "S",
            203..=247 => "SW",
            248..=292 => "W",
            293..=337 => "NW",
            _ => "N",
        }
    }

    /// Get wind strength description
    pub fn strength_description(&self) -> &'static str {
        match self.speed as u32 {
            0..=3 => "Calm",
            4..=7 => "Light",
            8..=12 => "Gentle",
            13..=18 => "Moderate",
            19..=24 => "Fresh",
            25..=31 => "Strong",
            32..=38 => "Near Gale",
            39..=46 => "Gale",
            47..=55 => "Strong Gale",
            56..=63 => "Storm",
            64..=72 => "Violent Storm",
            _ => "Hurricane",
        }
    }

    /// Update wind conditions over time
    pub fn update(&mut self, delta_time: f32) {
        // Wind shifts gradually
        let shift = (rand_float() - 0.5) * self.variability * delta_time;
        self.direction = (self.direction + shift) % (2.0 * PI);

        // Speed varies
        let speed_change = (rand_float() - 0.5) * self.variability * 5.0 * delta_time;
        self.speed = (self.speed + speed_change).clamp(0.0, 80.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeaState {
    Calm,      // Glassy
    Slight,    // Small waves
    Moderate,  // Some whitecaps
    Rough,     // Many whitecaps, spray
    VeryRough, // Large waves
    High,      // Very large waves
    Phenomenal,// Extreme
}

impl SeaState {
    pub fn speed_modifier(&self) -> f32 {
        match self {
            Self::Calm => 0.9,      // Need some wind to fill sails
            Self::Slight => 1.0,
            Self::Moderate => 1.0,
            Self::Rough => 0.9,
            Self::VeryRough => 0.7,
            Self::High => 0.5,
            Self::Phenomenal => 0.2,
        }
    }

    pub fn damage_risk(&self) -> f32 {
        match self {
            Self::Calm => 0.0,
            Self::Slight => 0.0,
            Self::Moderate => 0.0,
            Self::Rough => 0.01,
            Self::VeryRough => 0.05,
            Self::High => 0.15,
            Self::Phenomenal => 0.4,
        }
    }
}

/// Point of sail relative to wind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointOfSail {
    InIrons,       // Directly into wind, no movement
    CloseHauled,   // ~45 degrees to wind
    CloseReach,    // ~60 degrees
    BeamReach,     // 90 degrees, usually fastest
    BroadReach,    // ~120 degrees
    Running,       // Wind directly behind
}

impl PointOfSail {
    /// Calculate point of sail from ship heading and wind direction
    pub fn from_headings(ship_heading: f32, wind_from: f32) -> Self {
        // Angle between ship direction and wind origin
        let mut relative_wind = (wind_from - ship_heading).abs();
        if relative_wind > PI {
            relative_wind = 2.0 * PI - relative_wind;
        }

        let degrees = relative_wind.to_degrees();

        match degrees as u32 {
            0..=30 => Self::InIrons,
            31..=50 => Self::CloseHauled,
            51..=70 => Self::CloseReach,
            71..=110 => Self::BeamReach,
            111..=150 => Self::BroadReach,
            _ => Self::Running,
        }
    }

    /// Speed multiplier for this point of sail
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            Self::InIrons => 0.0,
            Self::CloseHauled => 0.4,
            Self::CloseReach => 0.6,
            Self::BeamReach => 1.0,
            Self::BroadReach => 0.9,
            Self::Running => 0.7,
        }
    }

    /// Maneuverability at this point
    pub fn maneuver_modifier(&self) -> f32 {
        match self {
            Self::InIrons => 0.2,     // Hard to maneuver
            Self::CloseHauled => 0.7,
            Self::CloseReach => 0.9,
            Self::BeamReach => 1.0,
            Self::BroadReach => 0.9,
            Self::Running => 0.6,     // Risk of accidental jibe
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::InIrons => "In Irons (stalled)",
            Self::CloseHauled => "Close Hauled",
            Self::CloseReach => "Close Reach",
            Self::BeamReach => "Beam Reach",
            Self::BroadReach => "Broad Reach",
            Self::Running => "Running",
        }
    }
}

/// Sailing maneuvers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SailingManeuver {
    None,
    Tacking,      // Turning through the wind (bow through wind)
    Wearing,      // Turning away from wind (stern through wind)
    HeaveTO,      // Stop the ship
    FullSail,     // All canvas set
    ReducedSail,  // Reefed sails for safety
    Anchoring,
    WeighAnchor,
}

impl SailingManeuver {
    pub fn duration(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Tacking => 15.0,    // Seconds
            Self::Wearing => 30.0,    // Longer, safer turn
            Self::HeaveTO => 20.0,
            Self::FullSail => 45.0,
            Self::ReducedSail => 30.0,
            Self::Anchoring => 60.0,
            Self::WeighAnchor => 120.0,
        }
    }

    pub fn crew_required(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Tacking => 10,
            Self::Wearing => 8,
            Self::HeaveTO => 5,
            Self::FullSail => 15,
            Self::ReducedSail => 10,
            Self::Anchoring => 12,
            Self::WeighAnchor => 15,
        }
    }
}

/// Ship navigation and movement system
#[derive(Debug, Default)]
pub struct SailingSystem {
    pub wind: WindConditions,
    pub current_direction: f32,
    pub current_speed: f32,
}

impl SailingSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate effective speed for a ship
    pub fn calculate_speed(&self, ship: &Ship) -> f32 {
        let point = PointOfSail::from_headings(ship.heading, self.wind.direction);
        let wind_factor = point.speed_multiplier();
        let sea_factor = self.wind.sea_state.speed_modifier();
        let wind_speed_factor = (self.wind.speed / 15.0).min(1.5); // More wind = faster, up to a point

        ship.effective_speed(wind_factor * sea_factor * wind_speed_factor)
    }

    /// Update ship position based on sailing conditions
    pub fn update_ship(&self, ship: &mut Ship, delta_time: f32) {
        if ship.anchor_down {
            ship.current_speed = 0.0;
            return;
        }

        // Calculate speed
        let target_speed = self.calculate_speed(ship);

        // Speed changes gradually
        let speed_diff = target_speed - ship.current_speed;
        ship.current_speed += speed_diff * delta_time * 0.1;

        // Apply current
        let current_x = self.current_direction.sin() * self.current_speed * delta_time;
        let current_z = self.current_direction.cos() * self.current_speed * delta_time;

        // Apply ship movement
        let speed_per_frame = ship.current_speed * delta_time / 3600.0; // Convert knots to units/sec
        let dx = ship.heading.sin() * speed_per_frame + current_x;
        let dz = ship.heading.cos() * speed_per_frame + current_z;

        ship.position[0] += dx;
        ship.position[1] += dz;
    }

    /// Start a turn maneuver
    pub fn turn_ship(&self, ship: &mut Ship, turn_amount: f32, delta_time: f32) {
        let point = PointOfSail::from_headings(ship.heading, self.wind.direction);
        let turn_rate = ship.effective_maneuverability() * point.maneuver_modifier();

        // Apply turn
        let actual_turn = turn_amount * turn_rate * delta_time;
        ship.heading = (ship.heading + actual_turn) % (2.0 * PI);
    }

    /// Check if a maneuver is possible
    pub fn can_perform_maneuver(&self, ship: &Ship, maneuver: SailingManeuver) -> bool {
        let required_crew = maneuver.crew_required();
        let available_crew = ship.crew_count;

        if available_crew < required_crew {
            return false;
        }

        match maneuver {
            SailingManeuver::Tacking => {
                // Can't tack if in irons or running
                let point = PointOfSail::from_headings(ship.heading, self.wind.direction);
                point != PointOfSail::InIrons && point != PointOfSail::Running
            }
            SailingManeuver::WeighAnchor => ship.anchor_down,
            SailingManeuver::Anchoring => !ship.anchor_down,
            _ => true,
        }
    }

    /// Get current point of sail for a ship
    pub fn get_point_of_sail(&self, ship: &Ship) -> PointOfSail {
        PointOfSail::from_headings(ship.heading, self.wind.direction)
    }

    /// Calculate time to reach a destination
    pub fn estimate_travel_time(&self, ship: &Ship, destination: [f32; 2]) -> f32 {
        let dx = destination[0] - ship.position[0];
        let dz = destination[1] - ship.position[1];
        let distance = (dx * dx + dz * dz).sqrt();

        let avg_speed = self.calculate_speed(ship) * 0.7; // Account for tacking
        if avg_speed < 0.1 {
            return f32::INFINITY;
        }

        distance / (avg_speed / 3600.0) // Convert to seconds
    }
}

/// Navigation waypoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub position: [f32; 2],
    pub name: String,
    pub waypoint_type: WaypointType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaypointType {
    Port,
    Anchorage,
    Hazard,
    PointOfInterest,
    Custom,
}

/// Navigation tools and calculations
#[derive(Debug)]
pub struct Navigation {
    pub known_waypoints: Vec<Waypoint>,
    pub current_destination: Option<usize>,
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            known_waypoints: Vec::new(),
            current_destination: None,
        }
    }

    /// Add a discovered waypoint
    pub fn add_waypoint(&mut self, position: [f32; 2], name: &str, wtype: WaypointType) {
        self.known_waypoints.push(Waypoint {
            position,
            name: name.to_string(),
            waypoint_type: wtype,
        });
    }

    /// Calculate bearing to waypoint
    pub fn bearing_to(&self, from: [f32; 2], to: [f32; 2]) -> f32 {
        let dx = to[0] - from[0];
        let dz = to[1] - from[1];
        dx.atan2(dz)
    }

    /// Calculate distance to waypoint
    pub fn distance_to(&self, from: [f32; 2], to: [f32; 2]) -> f32 {
        let dx = to[0] - from[0];
        let dz = to[1] - from[1];
        (dx * dx + dz * dz).sqrt()
    }

    /// Get heading to sail to destination, accounting for wind
    pub fn calculate_course(&self, from: [f32; 2], to: [f32; 2], wind: &WindConditions) -> Vec<[f32; 2]> {
        let direct_bearing = self.bearing_to(from, to);

        // Check if we can sail directly
        let point = PointOfSail::from_headings(direct_bearing, wind.direction);

        if point != PointOfSail::InIrons {
            // Can sail directly (or close enough)
            return vec![to];
        }

        // Need to tack - calculate intermediate waypoints
        let distance = self.distance_to(from, to);
        let tack_angle = PI / 4.0; // 45 degrees off wind

        // First leg: sail at 45 degrees to wind
        let leg1_heading = wind.direction + tack_angle;
        let leg1_distance = distance / 2.0;
        let mid_point = [
            from[0] + leg1_heading.sin() * leg1_distance,
            from[1] + leg1_heading.cos() * leg1_distance,
        ];

        vec![mid_point, to]
    }
}

impl Default for Navigation {
    fn default() -> Self {
        Self::new()
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
