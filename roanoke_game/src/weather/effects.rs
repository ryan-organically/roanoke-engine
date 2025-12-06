//! Weather effects on gameplay
//!
//! Translates weather conditions into concrete gameplay modifiers.

use serde::{Deserialize, Serialize};
use super::{WeatherState, ExtendedWeatherType};

/// All weather-based gameplay modifiers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeatherEffects {
    // Movement
    pub movement_speed: f32,       // Multiplier
    pub stamina_drain: f32,        // Multiplier

    // Combat
    pub ranged_accuracy: f32,      // Multiplier
    pub melee_effectiveness: f32,  // Multiplier
    pub gunpowder_reliability: f32, // Chance of misfires

    // Survival
    pub exposure_risk: f32,        // Damage per hour from elements
    pub fire_difficulty: f32,      // Multiplier for starting fires
    pub tracking_difficulty: f32,  // Multiplier for tracking

    // Resource gathering
    pub foraging_yield: f32,       // Multiplier
    pub fishing_success: f32,      // Multiplier
    pub hunting_success: f32,      // Multiplier

    // Sailing
    pub sailing_speed: f32,        // Multiplier
    pub navigation_difficulty: f32, // Multiplier
    pub shipwreck_risk: f32,       // Base chance per hour

    // Health
    pub disease_risk: f32,         // Modifier to disease chance
    pub healing_rate: f32,         // Multiplier

    // Visibility
    pub detection_range: f32,      // Multiplier for how far you can see
    pub stealth_bonus: f32,        // Bonus to stealth
}

impl WeatherEffects {
    /// Calculate effects from current weather state
    pub fn from_weather(weather: &WeatherState) -> Self {
        let mut effects = Self::default_values();

        // Apply weather type effects
        effects.apply_weather_type(weather.weather_type);

        // Apply wind effects
        effects.apply_wind(weather.wind_speed);

        // Apply temperature effects
        effects.apply_temperature(weather.temperature);

        // Apply visibility effects
        effects.apply_visibility(weather.visibility);

        // Apply precipitation effects
        effects.apply_precipitation(weather.precipitation_rate);

        effects
    }

    fn default_values() -> Self {
        Self {
            movement_speed: 1.0,
            stamina_drain: 1.0,
            ranged_accuracy: 1.0,
            melee_effectiveness: 1.0,
            gunpowder_reliability: 1.0,
            exposure_risk: 0.0,
            fire_difficulty: 1.0,
            tracking_difficulty: 1.0,
            foraging_yield: 1.0,
            fishing_success: 1.0,
            hunting_success: 1.0,
            sailing_speed: 1.0,
            navigation_difficulty: 1.0,
            shipwreck_risk: 0.0,
            disease_risk: 0.0,
            healing_rate: 1.0,
            detection_range: 1.0,
            stealth_bonus: 0.0,
        }
    }

    fn apply_weather_type(&mut self, weather: ExtendedWeatherType) {
        match weather {
            ExtendedWeatherType::Clear => {
                self.detection_range = 1.2;
                self.ranged_accuracy = 1.1;
            }
            ExtendedWeatherType::PartlyCloudy => {
                self.hunting_success = 1.1; // Animals more active
            }
            ExtendedWeatherType::Hazy => {
                self.detection_range = 0.8;
                self.stealth_bonus = 0.1;
            }
            ExtendedWeatherType::Overcast => {
                self.hunting_success = 1.05;
                self.stealth_bonus = 0.05;
            }
            ExtendedWeatherType::LightRain => {
                self.fire_difficulty = 1.5;
                self.gunpowder_reliability = 0.9;
                self.tracking_difficulty = 1.3;
                self.fishing_success = 1.2;
                self.stealth_bonus = 0.15;
            }
            ExtendedWeatherType::HeavyRain => {
                self.movement_speed = 0.8;
                self.fire_difficulty = 3.0;
                self.gunpowder_reliability = 0.6;
                self.ranged_accuracy = 0.7;
                self.tracking_difficulty = 2.0;
                self.hunting_success = 0.5;
                self.stealth_bonus = 0.3;
            }
            ExtendedWeatherType::Thunderstorm => {
                self.movement_speed = 0.6;
                self.fire_difficulty = 5.0;
                self.gunpowder_reliability = 0.3;
                self.ranged_accuracy = 0.4;
                self.exposure_risk = 0.5;
                self.hunting_success = 0.2;
                self.shipwreck_risk = 0.05;
            }
            ExtendedWeatherType::TropicalStorm => {
                self.movement_speed = 0.4;
                self.fire_difficulty = 10.0;
                self.gunpowder_reliability = 0.1;
                self.ranged_accuracy = 0.2;
                self.exposure_risk = 2.0;
                self.hunting_success = 0.0;
                self.shipwreck_risk = 0.15;
                self.sailing_speed = 0.3;
            }
            ExtendedWeatherType::Hurricane => {
                self.movement_speed = 0.1;
                self.fire_difficulty = 100.0;
                self.gunpowder_reliability = 0.0;
                self.ranged_accuracy = 0.0;
                self.exposure_risk = 10.0;
                self.hunting_success = 0.0;
                self.shipwreck_risk = 0.5;
                self.sailing_speed = 0.0;
            }
            ExtendedWeatherType::Fog | ExtendedWeatherType::DenseFog => {
                self.detection_range = 0.2;
                self.ranged_accuracy = 0.3;
                self.navigation_difficulty = 2.5;
                self.stealth_bonus = 0.5;
                self.shipwreck_risk = 0.02;
            }
            ExtendedWeatherType::Mist => {
                self.detection_range = 0.5;
                self.stealth_bonus = 0.2;
            }
            ExtendedWeatherType::Sleet => {
                self.movement_speed = 0.7;
                self.fire_difficulty = 2.0;
                self.exposure_risk = 3.0;
                self.ranged_accuracy = 0.6;
            }
            ExtendedWeatherType::LightSnow => {
                self.movement_speed = 0.8;
                self.tracking_difficulty = 0.5; // Tracks visible in snow!
                self.fire_difficulty = 1.5;
                self.exposure_risk = 2.0;
            }
        }
    }

    fn apply_wind(&mut self, wind_speed: f32) {
        // High winds affect many activities
        if wind_speed > 20.0 {
            let factor = 1.0 - (wind_speed - 20.0) / 60.0;
            self.ranged_accuracy *= factor.max(0.2);
            self.fire_difficulty *= 1.0 / factor.max(0.3);
        }

        // Sailing benefits from moderate wind
        if wind_speed < 5.0 {
            self.sailing_speed *= 0.5; // Becalmed
        } else if wind_speed < 20.0 {
            self.sailing_speed *= 1.0 + (wind_speed - 5.0) / 30.0;
        } else if wind_speed > 30.0 {
            self.sailing_speed *= 1.0 - (wind_speed - 30.0) / 50.0;
            self.shipwreck_risk += (wind_speed - 30.0) / 200.0;
        }

        // High winds make ranged combat harder
        if wind_speed > 15.0 {
            self.ranged_accuracy *= 1.0 - (wind_speed - 15.0) / 100.0;
        }

        // Wind affects fire
        if wind_speed > 10.0 {
            self.fire_difficulty *= 1.0 + wind_speed / 30.0;
        }
    }

    fn apply_temperature(&mut self, temp: f32) {
        // Extreme cold
        if temp < 40.0 {
            let cold_factor = (40.0 - temp) / 40.0;
            self.stamina_drain *= 1.0 + cold_factor * 0.5;
            self.exposure_risk += cold_factor * 2.0;
            self.healing_rate *= 1.0 - cold_factor * 0.3;
        }

        // Extreme heat
        if temp > 90.0 {
            let heat_factor = (temp - 90.0) / 20.0;
            self.stamina_drain *= 1.0 + heat_factor * 0.7;
            self.exposure_risk += heat_factor * 1.0;
            self.disease_risk += heat_factor * 0.1;
        }

        // Optimal temperature range
        if temp > 60.0 && temp < 80.0 {
            self.stamina_drain *= 0.9;
            self.healing_rate *= 1.1;
        }
    }

    fn apply_visibility(&mut self, visibility: f32) {
        self.detection_range *= visibility;
        self.ranged_accuracy *= (visibility * 2.0).min(1.0);

        if visibility < 0.5 {
            self.stealth_bonus += 0.5 - visibility;
            self.navigation_difficulty *= 1.0 / visibility.max(0.1);
        }

        if visibility < 0.2 {
            self.shipwreck_risk += 0.01;
        }
    }

    fn apply_precipitation(&mut self, rate: f32) {
        if rate > 0.0 {
            // Rain washes away tracks
            self.tracking_difficulty *= 1.0 + rate * 2.0;

            // Wet ground
            self.movement_speed *= 1.0 - rate * 0.2;

            // Gunpowder gets wet
            self.gunpowder_reliability *= 1.0 - rate * 0.5;

            // Harder to stay healthy
            if rate > 0.5 {
                self.disease_risk += 0.02;
            }
        }

        // Light rain can be good for fishing
        if rate > 0.0 && rate < 0.3 {
            self.fishing_success *= 1.2;
        }
    }
}

/// Weather warning for player notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherWarning {
    pub warning_type: WarningType,
    pub severity: WarningSeverity,
    pub message: String,
    pub hours_until: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningType {
    StormApproaching,
    HurricaneWarning,
    FloodRisk,
    Drought,
    ExtremeHeat,
    ExtremeCold,
    HighWinds,
    DenseFog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSeverity {
    Advisory,  // Worth knowing
    Watch,     // Conditions possible
    Warning,   // Conditions imminent
    Emergency, // Life-threatening
}

impl WeatherWarning {
    pub fn storm_approaching(hours: f32, category: super::storms::HurricaneCategory) -> Self {
        use super::storms::HurricaneCategory;

        let (severity, msg) = match category {
            HurricaneCategory::TropicalDepression |
            HurricaneCategory::TropicalStorm => (
                WarningSeverity::Watch,
                "Tropical system approaching - expect rough weather",
            ),
            HurricaneCategory::Category1 |
            HurricaneCategory::Category2 => (
                WarningSeverity::Warning,
                "Hurricane approaching! Seek shelter immediately",
            ),
            HurricaneCategory::Category3 |
            HurricaneCategory::Category4 |
            HurricaneCategory::Category5 => (
                WarningSeverity::Emergency,
                "MAJOR HURRICANE IMMINENT - LIFE-THREATENING CONDITIONS",
            ),
        };

        Self {
            warning_type: WarningType::HurricaneWarning,
            severity,
            message: msg.to_string(),
            hours_until: Some(hours),
        }
    }

    pub fn high_winds(speed: f32) -> Self {
        let severity = if speed > 50.0 {
            WarningSeverity::Warning
        } else if speed > 35.0 {
            WarningSeverity::Watch
        } else {
            WarningSeverity::Advisory
        };

        Self {
            warning_type: WarningType::HighWinds,
            severity,
            message: format!("High winds of {} mph expected", speed as u32),
            hours_until: None,
        }
    }

    pub fn fog_warning() -> Self {
        Self {
            warning_type: WarningType::DenseFog,
            severity: WarningSeverity::Advisory,
            message: "Dense fog reducing visibility".to_string(),
            hours_until: None,
        }
    }
}

/// Long-term weather impact on the world
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeatherImpact {
    pub flooding_level: f32,       // 0-1, affects low areas
    pub drought_severity: f32,     // 0-1, affects plant growth
    pub storm_damage: f32,         // 0-1, structural damage
    pub days_since_rain: u32,
    pub seasonal_rainfall: f32,    // Inches this season
    pub record_low_temp: f32,
    pub record_high_temp: f32,
}

impl WeatherImpact {
    pub fn update(&mut self, weather: &WeatherState, delta_hours: f32) {
        // Update flooding
        if weather.precipitation_rate > 1.0 {
            self.flooding_level += weather.precipitation_rate * 0.1 * delta_hours;
            self.flooding_level = self.flooding_level.min(1.0);
        } else {
            self.flooding_level -= 0.02 * delta_hours;
            self.flooding_level = self.flooding_level.max(0.0);
        }

        // Track rainfall
        self.seasonal_rainfall += weather.precipitation_rate * delta_hours;

        // Days since rain
        if weather.precipitation_rate > 0.0 {
            self.days_since_rain = 0;
        }

        // Drought builds slowly
        if self.days_since_rain > 7 {
            self.drought_severity += 0.01 * delta_hours / 24.0;
            self.drought_severity = self.drought_severity.min(1.0);
        } else {
            self.drought_severity -= 0.02 * delta_hours / 24.0;
            self.drought_severity = self.drought_severity.max(0.0);
        }

        // Storm damage decays slowly (repairs)
        self.storm_damage -= 0.001 * delta_hours;
        self.storm_damage = self.storm_damage.max(0.0);

        // Track temperature records
        if weather.temperature < self.record_low_temp || self.record_low_temp == 0.0 {
            self.record_low_temp = weather.temperature;
        }
        if weather.temperature > self.record_high_temp {
            self.record_high_temp = weather.temperature;
        }
    }

    pub fn apply_storm_damage(&mut self, intensity: f32) {
        self.storm_damage += intensity * 0.5;
        self.storm_damage = self.storm_damage.min(1.0);
    }

    /// Get modifier for crop/plant growth
    pub fn growth_modifier(&self) -> f32 {
        let mut modifier = 1.0;

        // Drought severely impacts growth
        modifier -= self.drought_severity * 0.5;

        // Flooding also bad
        modifier -= self.flooding_level * 0.3;

        // Storm damage affects established plants
        modifier -= self.storm_damage * 0.2;

        modifier.max(0.1)
    }
}
