//! Extended Weather and Storm System
//!
//! Expands the base weather system with:
//! - Realistic storm progression and hurricane simulation
//! - Beaufort wind scale
//! - Seasonal weather patterns
//! - Weather-based gameplay effects
//!
//! Based on historical weather patterns of the Carolina/Virginia coast.

pub mod storms;
pub mod effects;

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Extracted storm data to avoid borrow checker issues
struct StormWeatherData {
    intensity: f32,
    wind_speed: f32,
    wind_direction: f32,
    visibility: f32,
    category: storms::HurricaneCategory,
}

/// Extended weather types beyond basic cloud states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtendedWeatherType {
    // Fair weather
    Clear,
    PartlyCloudy,
    Hazy,

    // Precipitation
    Overcast,
    LightRain,
    HeavyRain,
    Thunderstorm,

    // Severe
    TropicalStorm,
    Hurricane,

    // Visibility
    Fog,
    DenseFog,
    Mist,

    // Winter (rare in colonial Carolina but possible)
    Sleet,
    LightSnow,
}

impl ExtendedWeatherType {
    pub fn base_visibility(&self) -> f32 {
        match self {
            Self::Clear => 1.0,
            Self::PartlyCloudy => 0.95,
            Self::Hazy => 0.7,
            Self::Overcast => 0.8,
            Self::LightRain => 0.6,
            Self::HeavyRain => 0.35,
            Self::Thunderstorm => 0.25,
            Self::TropicalStorm => 0.15,
            Self::Hurricane => 0.05,
            Self::Fog => 0.2,
            Self::DenseFog => 0.05,
            Self::Mist => 0.4,
            Self::Sleet => 0.3,
            Self::LightSnow => 0.5,
        }
    }

    pub fn movement_modifier(&self) -> f32 {
        match self {
            Self::Clear | Self::PartlyCloudy | Self::Hazy => 1.0,
            Self::Overcast | Self::Mist => 0.95,
            Self::LightRain | Self::Fog => 0.85,
            Self::HeavyRain | Self::DenseFog => 0.7,
            Self::Thunderstorm | Self::Sleet => 0.5,
            Self::TropicalStorm | Self::LightSnow => 0.3,
            Self::Hurricane => 0.1,
        }
    }

    pub fn hunting_modifier(&self) -> f32 {
        match self {
            Self::Clear => 1.0,
            Self::PartlyCloudy => 1.1,  // Animals more active
            Self::Overcast | Self::Hazy | Self::Mist => 1.05,
            Self::LightRain => 0.7,     // Animals shelter
            Self::HeavyRain | Self::Fog => 0.4,
            Self::Thunderstorm | Self::DenseFog => 0.2,
            Self::TropicalStorm | Self::Hurricane => 0.0,
            Self::Sleet | Self::LightSnow => 0.3,
        }
    }

    pub fn sailing_modifier(&self) -> f32 {
        match self {
            Self::Clear => 0.8,         // Light winds
            Self::PartlyCloudy => 1.0,
            Self::Hazy => 0.9,
            Self::Overcast => 1.1,
            Self::LightRain => 1.0,
            Self::HeavyRain => 0.8,
            Self::Thunderstorm => 0.5,  // Dangerous
            Self::TropicalStorm => 0.3,
            Self::Hurricane => 0.0,     // Cannot sail
            Self::Fog | Self::DenseFog | Self::Mist => 0.6,
            Self::Sleet | Self::LightSnow => 0.7,
        }
    }
}

/// Beaufort wind scale (0-12)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BeaufortScale {
    Calm = 0,          // < 1 knot
    LightAir = 1,      // 1-3 knots
    LightBreeze = 2,   // 4-6 knots
    GentleBreeze = 3,  // 7-10 knots
    ModerateBreeze = 4, // 11-16 knots
    FreshBreeze = 5,   // 17-21 knots
    StrongBreeze = 6,  // 22-27 knots
    NearGale = 7,      // 28-33 knots
    Gale = 8,          // 34-40 knots
    StrongGale = 9,    // 41-47 knots
    Storm = 10,        // 48-55 knots
    ViolentStorm = 11, // 56-63 knots
    Hurricane = 12,    // 64+ knots
}

impl BeaufortScale {
    pub fn from_knots(knots: f32) -> Self {
        match knots as u32 {
            0 => Self::Calm,
            1..=3 => Self::LightAir,
            4..=6 => Self::LightBreeze,
            7..=10 => Self::GentleBreeze,
            11..=16 => Self::ModerateBreeze,
            17..=21 => Self::FreshBreeze,
            22..=27 => Self::StrongBreeze,
            28..=33 => Self::NearGale,
            34..=40 => Self::Gale,
            41..=47 => Self::StrongGale,
            48..=55 => Self::Storm,
            56..=63 => Self::ViolentStorm,
            _ => Self::Hurricane,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Calm => "Calm - smoke rises vertically",
            Self::LightAir => "Light air - smoke drifts",
            Self::LightBreeze => "Light breeze - leaves rustle",
            Self::GentleBreeze => "Gentle breeze - leaves and twigs move",
            Self::ModerateBreeze => "Moderate breeze - small branches move",
            Self::FreshBreeze => "Fresh breeze - small trees sway",
            Self::StrongBreeze => "Strong breeze - large branches move",
            Self::NearGale => "Near gale - whole trees in motion",
            Self::Gale => "Gale - twigs break off trees",
            Self::StrongGale => "Strong gale - branches break",
            Self::Storm => "Storm - trees uprooted",
            Self::ViolentStorm => "Violent storm - widespread damage",
            Self::Hurricane => "Hurricane - devastating damage",
        }
    }

    pub fn sea_state(&self) -> &'static str {
        match self {
            Self::Calm => "Mirror-like",
            Self::LightAir => "Ripples",
            Self::LightBreeze => "Small wavelets",
            Self::GentleBreeze => "Large wavelets, crests begin to break",
            Self::ModerateBreeze => "Small waves, frequent whitecaps",
            Self::FreshBreeze => "Moderate waves, many whitecaps",
            Self::StrongBreeze => "Large waves, extensive whitecaps",
            Self::NearGale => "Sea heaps up, foam streaks",
            Self::Gale => "Moderately high waves, spray",
            Self::StrongGale => "High waves, dense foam",
            Self::Storm => "Very high waves, visibility reduced",
            Self::ViolentStorm => "Exceptionally high waves",
            Self::Hurricane => "Air filled with foam and spray",
        }
    }

    pub fn min_knots(&self) -> f32 {
        match self {
            Self::Calm => 0.0,
            Self::LightAir => 1.0,
            Self::LightBreeze => 4.0,
            Self::GentleBreeze => 7.0,
            Self::ModerateBreeze => 11.0,
            Self::FreshBreeze => 17.0,
            Self::StrongBreeze => 22.0,
            Self::NearGale => 28.0,
            Self::Gale => 34.0,
            Self::StrongGale => 41.0,
            Self::Storm => 48.0,
            Self::ViolentStorm => 56.0,
            Self::Hurricane => 64.0,
        }
    }
}

/// Seasonal weather patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,  // March-May: Variable, thunderstorms
    Summer,  // June-August: Hot, humid, hurricane season
    Fall,    // September-November: Hurricane peak, cooling
    Winter,  // December-February: Cold fronts, occasional snow
}

impl Season {
    pub fn from_day_of_year(day: u32) -> Self {
        match day {
            60..=151 => Self::Spring,   // ~March 1 - May 31
            152..=243 => Self::Summer,  // ~June 1 - August 31
            244..=334 => Self::Fall,    // ~September 1 - November 30
            _ => Self::Winter,          // December 1 - February 28
        }
    }

    /// Hurricane probability modifier
    pub fn hurricane_chance(&self) -> f32 {
        match self {
            Self::Summer => 0.02,   // Low but possible
            Self::Fall => 0.05,     // Peak season
            Self::Spring => 0.005,  // Very rare
            Self::Winter => 0.0,    // None
        }
    }

    /// Base temperature range (Fahrenheit)
    pub fn temperature_range(&self) -> (f32, f32) {
        match self {
            Self::Spring => (50.0, 75.0),
            Self::Summer => (70.0, 95.0),
            Self::Fall => (45.0, 75.0),
            Self::Winter => (30.0, 55.0),
        }
    }

    /// Probability of rain on any given day
    pub fn rain_probability(&self) -> f32 {
        match self {
            Self::Spring => 0.35,
            Self::Summer => 0.45,  // Afternoon thunderstorms
            Self::Fall => 0.25,
            Self::Winter => 0.30,
        }
    }
}

/// Complete weather state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub weather_type: ExtendedWeatherType,
    pub wind_speed: f32,           // Knots
    pub wind_direction: f32,       // Radians (0 = from north)
    pub temperature: f32,          // Fahrenheit
    pub humidity: f32,             // 0-1
    pub visibility: f32,           // 0-1
    pub precipitation_rate: f32,   // inches per hour
    pub barometric_pressure: f32,  // inches of mercury
    pub cloud_cover: f32,          // 0-1
    pub time_in_state: f32,        // Seconds in current weather
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            weather_type: ExtendedWeatherType::PartlyCloudy,
            wind_speed: 8.0,
            wind_direction: PI / 4.0,
            temperature: 72.0,
            humidity: 0.6,
            visibility: 0.9,
            precipitation_rate: 0.0,
            barometric_pressure: 30.0,
            cloud_cover: 0.4,
            time_in_state: 0.0,
        }
    }
}

impl WeatherState {
    pub fn beaufort(&self) -> BeaufortScale {
        BeaufortScale::from_knots(self.wind_speed)
    }

    pub fn wind_direction_compass(&self) -> &'static str {
        let deg = (self.wind_direction.to_degrees() + 360.0) % 360.0;
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

    pub fn is_dangerous(&self) -> bool {
        matches!(
            self.weather_type,
            ExtendedWeatherType::Thunderstorm
            | ExtendedWeatherType::TropicalStorm
            | ExtendedWeatherType::Hurricane
        ) || self.wind_speed >= 34.0
    }

    pub fn is_sailing_dangerous(&self) -> bool {
        self.wind_speed >= 28.0 || self.visibility < 0.2
    }

    /// Apply weather effects for gameplay
    pub fn apply_effects(&self, base_value: f32, effect_type: WeatherEffectType) -> f32 {
        let modifier = match effect_type {
            WeatherEffectType::Movement => self.weather_type.movement_modifier(),
            WeatherEffectType::Hunting => self.weather_type.hunting_modifier(),
            WeatherEffectType::Sailing => self.weather_type.sailing_modifier(),
            WeatherEffectType::Visibility => self.visibility,
            WeatherEffectType::Foraging => {
                if self.precipitation_rate > 0.5 {
                    0.5
                } else {
                    1.0
                }
            }
        };
        base_value * modifier
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WeatherEffectType {
    Movement,
    Hunting,
    Sailing,
    Visibility,
    Foraging,
}

/// Weather forecast entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherForecast {
    pub time_ahead: f32,           // Hours from now
    pub predicted_weather: ExtendedWeatherType,
    pub predicted_wind: (f32, f32), // Speed, direction
    pub confidence: f32,            // 0-1, decreases for further predictions
    pub storm_warning: bool,
}

/// Extended weather manager
#[derive(Debug)]
pub struct ExtendedWeatherManager {
    pub current_state: WeatherState,
    pub current_season: Season,
    pub day_of_year: u32,
    pub hour_of_day: f32,

    // Storm tracking
    pub active_storm: Option<storms::StormSystem>,
    pub storm_history: Vec<storms::StormRecord>,

    // Transition
    target_state: WeatherState,
    transition_progress: f32,
    transition_duration: f32,

    // Forecasting
    pub forecasts: Vec<WeatherForecast>,
    forecast_update_timer: f32,

    // Integration with ecology
    pub cumulative_rainfall: f32,  // Inches this season
    pub drought_index: f32,        // 0 = normal, 1 = severe drought
}

impl ExtendedWeatherManager {
    pub fn new() -> Self {
        // Default to moody, overcast weather
        let moody_default = WeatherState {
            weather_type: ExtendedWeatherType::Overcast,
            wind_speed: 12.0,
            wind_direction: PI / 3.0,
            temperature: 62.0,
            humidity: 0.75,
            visibility: 0.7,
            precipitation_rate: 0.0,
            barometric_pressure: 29.6,
            cloud_cover: 0.75,
            time_in_state: 0.0,
        };

        Self {
            current_state: moody_default.clone(),
            current_season: Season::Fall, // Fall for moody atmosphere
            day_of_year: 280,             // Late October
            hour_of_day: 14.0,            // Afternoon
            active_storm: None,
            storm_history: Vec::new(),
            target_state: moody_default,
            transition_progress: 1.0,
            transition_duration: 1800.0,  // 30 minutes - much slower transitions
            forecasts: Vec::new(),
            forecast_update_timer: 0.0,
            cumulative_rainfall: 0.0,
            drought_index: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update time
        self.hour_of_day += delta_time / 3600.0;
        if self.hour_of_day >= 24.0 {
            self.hour_of_day -= 24.0;
            self.day_of_year += 1;
            if self.day_of_year > 365 {
                self.day_of_year = 1;
            }
            self.current_season = Season::from_day_of_year(self.day_of_year);
        }

        // Update active storm
        let storm_data = if let Some(storm) = &mut self.active_storm {
            storm.update(delta_time);

            // Extract storm data we need
            let data = StormWeatherData {
                intensity: storm.intensity(),
                wind_speed: storm.current_wind_speed,
                wind_direction: storm.wind_direction,
                visibility: storm.visibility(),
                category: storm.category,
            };

            // Check if storm has dissipated
            let dissipated = storm.is_dissipated();
            if dissipated {
                self.storm_history.push(storms::StormRecord::from_storm(storm));
            }

            Some((data, dissipated))
        } else {
            None
        };

        // Apply storm effects after releasing the mutable borrow
        if let Some((data, dissipated)) = storm_data {
            self.apply_storm_weather_data(&data);
            if dissipated {
                self.active_storm = None;
            }
        }

        // Transition between weather states
        if self.transition_progress < 1.0 {
            self.transition_progress += delta_time / self.transition_duration;
            self.transition_progress = self.transition_progress.min(1.0);
            self.interpolate_weather();
        } else {
            // Random weather changes when not in storm
            if self.active_storm.is_none() {
                self.check_weather_change(delta_time);
            }
        }

        // Track rainfall
        self.cumulative_rainfall += self.current_state.precipitation_rate * delta_time / 3600.0;

        // Update drought index
        self.update_drought_index(delta_time);

        // Update forecasts periodically
        self.forecast_update_timer -= delta_time;
        if self.forecast_update_timer <= 0.0 {
            self.generate_forecasts();
            self.forecast_update_timer = 3600.0; // Update hourly
        }

        // Chance to spawn new storm
        if self.active_storm.is_none() {
            self.check_storm_spawn();
        }

        self.current_state.time_in_state += delta_time;
    }

    fn apply_storm_weather_data(&mut self, data: &StormWeatherData) {
        self.current_state.wind_speed = data.wind_speed;
        self.current_state.wind_direction = data.wind_direction;
        self.current_state.precipitation_rate = data.intensity * 2.0; // Up to 2 inches/hour
        self.current_state.visibility = data.visibility;
        self.current_state.barometric_pressure = 30.0 - data.intensity * 3.0; // Low pressure in storms

        self.current_state.weather_type = match data.category {
            storms::HurricaneCategory::TropicalDepression |
            storms::HurricaneCategory::TropicalStorm => ExtendedWeatherType::TropicalStorm,
            _ => ExtendedWeatherType::Hurricane,
        };
    }

    fn check_weather_change(&mut self, delta_time: f32) {
        // Base change probability
        let change_chance = 0.0001 * delta_time; // About once per 3 hours

        if rand_float() < change_chance {
            self.transition_to_random_weather();
        }
    }

    fn transition_to_random_weather(&mut self) {
        let roll = rand_float();
        let rain_prob = self.current_season.rain_probability();

        let new_weather = if roll < rain_prob {
            // Precipitation
            if rand_float() < 0.3 {
                ExtendedWeatherType::Thunderstorm
            } else if rand_float() < 0.5 {
                ExtendedWeatherType::HeavyRain
            } else {
                ExtendedWeatherType::LightRain
            }
        } else if rand_float() < 0.2 {
            ExtendedWeatherType::Clear
        } else if rand_float() < 0.3 {
            ExtendedWeatherType::Overcast
        } else {
            ExtendedWeatherType::PartlyCloudy
        };

        self.set_target_weather(new_weather);
    }

    pub fn set_target_weather(&mut self, weather: ExtendedWeatherType) {
        self.target_state = self.create_weather_state(weather);
        self.transition_progress = 0.0;
        // Weather transitions take 20-40 minutes for realistic, gradual change
        self.transition_duration = 1200.0 + rand_float() * 1200.0;
    }

    /// Force immediate weather change without transition (for testing/debugging)
    pub fn set_weather_immediate(&mut self, weather: ExtendedWeatherType) {
        self.current_state = self.create_weather_state(weather);
        self.target_state = self.current_state.clone();
        self.transition_progress = 1.0;
    }

    /// Get current rain intensity (0-1) for rendering systems
    pub fn rain_intensity(&self) -> f32 {
        match self.current_state.weather_type {
            ExtendedWeatherType::LightRain => 0.3,
            ExtendedWeatherType::HeavyRain => 0.7,
            ExtendedWeatherType::Thunderstorm => 0.9,
            ExtendedWeatherType::TropicalStorm => 0.95,
            ExtendedWeatherType::Hurricane => 1.0,
            ExtendedWeatherType::Sleet => 0.4,
            ExtendedWeatherType::LightSnow => 0.2,
            ExtendedWeatherType::Mist => 0.1,
            _ => self.current_state.precipitation_rate.min(1.0),
        }
    }

    /// Get ambient dimming factor (0-1) for moody atmosphere
    pub fn ambient_dimming(&self) -> f32 {
        let base_dimming = match self.current_state.weather_type {
            ExtendedWeatherType::Clear => 0.0,
            ExtendedWeatherType::PartlyCloudy => 0.05,
            ExtendedWeatherType::Hazy => 0.1,
            ExtendedWeatherType::Overcast => 0.2,
            ExtendedWeatherType::LightRain => 0.25,
            ExtendedWeatherType::HeavyRain => 0.35,
            ExtendedWeatherType::Thunderstorm => 0.45,
            ExtendedWeatherType::TropicalStorm => 0.5,
            ExtendedWeatherType::Hurricane => 0.6,
            ExtendedWeatherType::Fog | ExtendedWeatherType::DenseFog => 0.3,
            ExtendedWeatherType::Mist => 0.15,
            ExtendedWeatherType::Sleet | ExtendedWeatherType::LightSnow => 0.2,
        };
        // Additional dimming from cloud cover
        base_dimming + self.current_state.cloud_cover * 0.1
    }

    fn create_weather_state(&self, weather: ExtendedWeatherType) -> WeatherState {
        let (temp_min, temp_max) = self.current_season.temperature_range();
        let base_temp = temp_min + rand_float() * (temp_max - temp_min);

        WeatherState {
            weather_type: weather,
            wind_speed: self.random_wind_for_weather(weather),
            wind_direction: rand_float() * 2.0 * PI,
            temperature: base_temp + self.temperature_offset(weather),
            humidity: self.humidity_for_weather(weather),
            visibility: weather.base_visibility(),
            precipitation_rate: self.precipitation_for_weather(weather),
            barometric_pressure: self.pressure_for_weather(weather),
            cloud_cover: self.cloud_cover_for_weather(weather),
            time_in_state: 0.0,
        }
    }

    fn random_wind_for_weather(&self, weather: ExtendedWeatherType) -> f32 {
        match weather {
            ExtendedWeatherType::Clear => 2.0 + rand_float() * 8.0,
            ExtendedWeatherType::PartlyCloudy => 5.0 + rand_float() * 10.0,
            ExtendedWeatherType::Overcast => 8.0 + rand_float() * 12.0,
            ExtendedWeatherType::LightRain => 10.0 + rand_float() * 10.0,
            ExtendedWeatherType::HeavyRain => 15.0 + rand_float() * 15.0,
            ExtendedWeatherType::Thunderstorm => 20.0 + rand_float() * 25.0,
            ExtendedWeatherType::Fog | ExtendedWeatherType::DenseFog => 0.0 + rand_float() * 5.0,
            _ => 5.0 + rand_float() * 10.0,
        }
    }

    fn temperature_offset(&self, weather: ExtendedWeatherType) -> f32 {
        match weather {
            ExtendedWeatherType::Clear => 5.0,
            ExtendedWeatherType::Overcast | ExtendedWeatherType::LightRain => -5.0,
            ExtendedWeatherType::Thunderstorm | ExtendedWeatherType::HeavyRain => -8.0,
            ExtendedWeatherType::Fog | ExtendedWeatherType::Mist => -3.0,
            _ => 0.0,
        }
    }

    fn humidity_for_weather(&self, weather: ExtendedWeatherType) -> f32 {
        match weather {
            ExtendedWeatherType::Clear => 0.3 + rand_float() * 0.2,
            ExtendedWeatherType::PartlyCloudy => 0.4 + rand_float() * 0.2,
            ExtendedWeatherType::Overcast => 0.6 + rand_float() * 0.2,
            ExtendedWeatherType::LightRain | ExtendedWeatherType::HeavyRain => 0.85 + rand_float() * 0.15,
            ExtendedWeatherType::Thunderstorm => 0.9 + rand_float() * 0.1,
            ExtendedWeatherType::Fog | ExtendedWeatherType::DenseFog | ExtendedWeatherType::Mist => 0.95,
            _ => 0.5 + rand_float() * 0.3,
        }
    }

    fn precipitation_for_weather(&self, weather: ExtendedWeatherType) -> f32 {
        match weather {
            ExtendedWeatherType::Clear | ExtendedWeatherType::PartlyCloudy |
            ExtendedWeatherType::Hazy | ExtendedWeatherType::Overcast => 0.0,
            ExtendedWeatherType::Mist => 0.01,
            ExtendedWeatherType::LightRain => 0.1 + rand_float() * 0.2,
            ExtendedWeatherType::HeavyRain => 0.5 + rand_float() * 0.5,
            ExtendedWeatherType::Thunderstorm => 0.8 + rand_float() * 0.7,
            ExtendedWeatherType::Fog | ExtendedWeatherType::DenseFog => 0.0,
            ExtendedWeatherType::Sleet | ExtendedWeatherType::LightSnow => 0.2 + rand_float() * 0.3,
            _ => 0.0,
        }
    }

    fn pressure_for_weather(&self, weather: ExtendedWeatherType) -> f32 {
        match weather {
            ExtendedWeatherType::Clear => 30.2 + rand_float() * 0.3,
            ExtendedWeatherType::PartlyCloudy => 30.0 + rand_float() * 0.2,
            ExtendedWeatherType::Overcast => 29.8 + rand_float() * 0.2,
            ExtendedWeatherType::LightRain | ExtendedWeatherType::HeavyRain => 29.5 + rand_float() * 0.3,
            ExtendedWeatherType::Thunderstorm => 29.2 + rand_float() * 0.3,
            _ => 30.0,
        }
    }

    fn cloud_cover_for_weather(&self, weather: ExtendedWeatherType) -> f32 {
        match weather {
            ExtendedWeatherType::Clear => 0.1 + rand_float() * 0.1,
            ExtendedWeatherType::PartlyCloudy => 0.3 + rand_float() * 0.3,
            ExtendedWeatherType::Hazy => 0.2 + rand_float() * 0.2,
            ExtendedWeatherType::Overcast => 0.8 + rand_float() * 0.2,
            ExtendedWeatherType::LightRain | ExtendedWeatherType::HeavyRain => 0.9 + rand_float() * 0.1,
            ExtendedWeatherType::Thunderstorm => 0.95 + rand_float() * 0.05,
            ExtendedWeatherType::Fog | ExtendedWeatherType::DenseFog | ExtendedWeatherType::Mist => 0.5 + rand_float() * 0.3,
            _ => 0.5,
        }
    }

    fn interpolate_weather(&mut self) {
        let t = smooth_step(self.transition_progress);

        self.current_state.wind_speed = lerp(self.current_state.wind_speed, self.target_state.wind_speed, t);
        self.current_state.wind_direction = lerp_angle(self.current_state.wind_direction, self.target_state.wind_direction, t);
        self.current_state.temperature = lerp(self.current_state.temperature, self.target_state.temperature, t);
        self.current_state.humidity = lerp(self.current_state.humidity, self.target_state.humidity, t);
        self.current_state.visibility = lerp(self.current_state.visibility, self.target_state.visibility, t);
        self.current_state.precipitation_rate = lerp(self.current_state.precipitation_rate, self.target_state.precipitation_rate, t);
        self.current_state.barometric_pressure = lerp(self.current_state.barometric_pressure, self.target_state.barometric_pressure, t);
        self.current_state.cloud_cover = lerp(self.current_state.cloud_cover, self.target_state.cloud_cover, t);

        if self.transition_progress >= 0.5 {
            self.current_state.weather_type = self.target_state.weather_type;
        }
    }

    fn update_drought_index(&mut self, delta_time: f32) {
        let expected_rainfall = match self.current_season {
            Season::Spring => 4.0,  // inches per month
            Season::Summer => 5.0,
            Season::Fall => 3.0,
            Season::Winter => 3.0,
        };

        let monthly_rainfall = self.cumulative_rainfall / (self.day_of_year as f32 / 30.0).max(1.0);

        if monthly_rainfall < expected_rainfall * 0.5 {
            self.drought_index = (self.drought_index + delta_time / 86400.0 * 0.02).min(1.0);
        } else {
            self.drought_index = (self.drought_index - delta_time / 86400.0 * 0.05).max(0.0);
        }
    }

    fn check_storm_spawn(&mut self) {
        let base_chance = self.current_season.hurricane_chance();
        let hourly_chance = base_chance / 24.0;

        if rand_float() < hourly_chance / 3600.0 {
            // Spawn a storm
            let storm = storms::StormSystem::spawn_random(self.day_of_year);
            self.active_storm = Some(storm);
        }
    }

    fn generate_forecasts(&mut self) {
        self.forecasts.clear();

        // Generate 24-hour forecast in 6-hour increments
        for hours in [6.0, 12.0, 18.0, 24.0] {
            let confidence = 1.0 - (hours / 48.0);

            let predicted = if let Some(storm) = &self.active_storm {
                if storm.time_to_landfall() < hours * 3600.0 {
                    ExtendedWeatherType::Hurricane
                } else {
                    ExtendedWeatherType::TropicalStorm
                }
            } else {
                // Simple persistence forecast with random variation
                if rand_float() < 0.7 {
                    self.current_state.weather_type
                } else {
                    ExtendedWeatherType::PartlyCloudy
                }
            };

            self.forecasts.push(WeatherForecast {
                time_ahead: hours,
                predicted_weather: predicted,
                predicted_wind: (self.current_state.wind_speed, self.current_state.wind_direction),
                confidence,
                storm_warning: self.active_storm.is_some(),
            });
        }
    }
}

impl Default for ExtendedWeatherManager {
    fn default() -> Self {
        Self::new()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = b - a;
    while diff > PI { diff -= 2.0 * PI; }
    while diff < -PI { diff += 2.0 * PI; }
    a + diff * t
}

fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn rand_float() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}

// === PIPELINE HELPER METHODS ===

impl ExtendedWeatherManager {
    /// Get current weather type as numeric ID for save data
    pub fn current_weather_type_id(&self) -> u8 {
        match self.current_state.weather_type {
            ExtendedWeatherType::Clear => 0,
            ExtendedWeatherType::PartlyCloudy => 1,
            ExtendedWeatherType::Hazy => 2,
            ExtendedWeatherType::Overcast => 3,
            ExtendedWeatherType::LightRain => 4,
            ExtendedWeatherType::HeavyRain => 5,
            ExtendedWeatherType::Thunderstorm => 6,
            ExtendedWeatherType::TropicalStorm => 7,
            ExtendedWeatherType::Hurricane => 8,
            ExtendedWeatherType::Fog => 9,
            ExtendedWeatherType::DenseFog => 10,
            ExtendedWeatherType::Mist => 11,
            ExtendedWeatherType::Sleet => 12,
            ExtendedWeatherType::LightSnow => 13,
        }
    }

    /// Restore state from save data
    pub fn restore_state(&mut self, day_of_year: u32, cumulative_rainfall: f32) {
        self.day_of_year = day_of_year;
        self.cumulative_rainfall = cumulative_rainfall;
        self.current_season = Season::from_day_of_year(day_of_year);
    }
}

/// Type alias for pipeline compatibility
pub type WeatherManager = ExtendedWeatherManager;
