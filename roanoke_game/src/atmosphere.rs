// Atmosphere Engine - Fog, Light Shafts, Soft Weather
//
// AGENT: This module handles atmospheric effects that vary with time of day and weather.
// Key integration points:
// - Updates in main game loop with time_of_day and weather state
// - Provides uniforms for terrain, sky, and post-process shaders

use glam::Vec3;

/// Atmospheric conditions computed from time and weather
#[derive(Debug, Clone, Copy)]
pub struct AtmosphereState {
    // Fog
    pub fog_density: f32,      // 0.0 = clear, 1.0 = thick fog
    pub fog_start: f32,        // Distance where fog begins
    pub fog_end: f32,          // Distance where fog is fully opaque
    pub fog_color: Vec3,       // Fog color (affected by sun)
    pub fog_height_falloff: f32, // Vertical fog density falloff

    // Light Shafts
    pub light_shaft_intensity: f32,  // 0.0 = none, 1.0 = strong
    pub light_shaft_decay: f32,      // How quickly shafts fade
    pub light_shaft_density: f32,    // Scattering density

    // Ambient
    pub ambient_intensity: f32,      // Overall ambient light
    pub ambient_color: Vec3,         // Ambient tint

    // Sun
    pub sun_intensity: f32,
    pub sun_color: Vec3,
    pub sun_direction: Vec3,
}

impl Default for AtmosphereState {
    fn default() -> Self {
        // Moody, overcast defaults
        Self {
            fog_density: 0.15,
            fog_start: 30.0,
            fog_end: 350.0,
            fog_color: Vec3::new(0.5, 0.52, 0.58), // Desaturated grey-blue
            fog_height_falloff: 0.025,

            light_shaft_intensity: 0.0,  // No shafts in overcast
            light_shaft_decay: 0.96,
            light_shaft_density: 0.5,

            ambient_intensity: 0.22,     // Dimmer ambient
            ambient_color: Vec3::new(0.55, 0.58, 0.65), // Cool grey

            sun_intensity: 0.7,          // Dimmer sun through clouds
            sun_color: Vec3::new(0.9, 0.88, 0.85), // Slightly desaturated
            sun_direction: Vec3::new(0.5, 0.6, 0.3).normalize(),
        }
    }
}

/// Time periods for atmospheric variation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimePeriod {
    Night,       // 0:00 - 5:00
    Dawn,        // 5:00 - 7:00  (foggy, light shafts)
    Morning,     // 7:00 - 10:00 (clearing fog, warm light)
    Midday,      // 10:00 - 16:00 (clear, bright)
    Afternoon,   // 16:00 - 18:00 (golden hour)
    Dusk,        // 18:00 - 20:00 (orange/pink, haze)
    Evening,     // 20:00 - 22:00 (blue hour)
}

impl TimePeriod {
    pub fn from_hour(hour: f32) -> Self {
        match hour {
            h if h < 5.0 => TimePeriod::Night,
            h if h < 7.0 => TimePeriod::Dawn,
            h if h < 10.0 => TimePeriod::Morning,
            h if h < 16.0 => TimePeriod::Midday,
            h if h < 18.0 => TimePeriod::Afternoon,
            h if h < 20.0 => TimePeriod::Dusk,
            h if h < 22.0 => TimePeriod::Evening,
            _ => TimePeriod::Night,
        }
    }
}

/// Main atmosphere engine
pub struct AtmosphereEngine {
    pub state: AtmosphereState,
    time_of_day: f32,
    weather_fog_modifier: f32,   // From weather system
    weather_cloud_coverage: f32,
}

impl AtmosphereEngine {
    pub fn new() -> Self {
        Self {
            state: AtmosphereState::default(),
            time_of_day: 8.0,
            weather_fog_modifier: 0.0,
            weather_cloud_coverage: 0.5,
        }
    }

    /// Update atmosphere based on time and weather
    /// render_distance: player's current render distance setting, used to ensure fog masks pop-in
    pub fn update(&mut self, time_of_day: f32, weather_fog: f32, cloud_coverage: f32, render_distance: f32) {
        self.time_of_day = time_of_day;
        self.weather_fog_modifier = weather_fog;
        self.weather_cloud_coverage = cloud_coverage;

        let period = TimePeriod::from_hour(time_of_day);

        // Minimum fog_end based on render distance to hide object pop-in
        // Add 10% buffer so fog is fully opaque before objects appear
        let min_fog_end = render_distance * 1.1;

        // Calculate sun position
        // Calculate sun position - Allow negative height for night detection
        let sun_angle = (time_of_day - 6.0) / 12.0 * std::f32::consts::PI;
        let sun_height = sun_angle.sin(); // Removed .max(0.0)
        let sun_horizontal = sun_angle.cos();
        self.state.sun_direction = Vec3::new(sun_horizontal * 0.5, sun_height, 0.3).normalize();

        // Time-based atmosphere
        match period {
            TimePeriod::Night => {
                // ... (Night settings remain same)
                self.state.fog_density = 0.1 + weather_fog * 0.3;
                self.state.fog_color = Vec3::new(0.005, 0.005, 0.01);
                self.state.fog_start = 30.0;
                self.state.fog_end = 200.0;
                self.state.light_shaft_intensity = 0.0;
                self.state.ambient_intensity = 0.08;
                self.state.ambient_color = Vec3::new(0.2, 0.25, 0.4);
                self.state.sun_intensity = 0.0;
                self.state.sun_color = Vec3::new(0.3, 0.35, 0.5);
            }
            TimePeriod::Dawn => {
                // ... (Dawn settings)
                self.state.fog_density = 0.4 + weather_fog * 0.4;
                self.state.fog_color = Vec3::new(0.6, 0.5, 0.45);
                self.state.fog_start = 10.0;
                self.state.fog_end = 150.0;
                self.state.fog_height_falloff = 0.05; 
                self.state.light_shaft_intensity = 0.7 * (1.0 - cloud_coverage * 0.8);
                self.state.light_shaft_decay = 0.94;
                self.state.ambient_intensity = 0.25;
                self.state.ambient_color = Vec3::new(0.7, 0.6, 0.5);
                self.state.sun_intensity = 0.6;
                self.state.sun_color = Vec3::new(1.0, 0.7, 0.4);
            }
            TimePeriod::Morning => {
                // Morning - fog lingers, moody atmosphere
                let clear_progress = (time_of_day - 7.0) / 3.0;
                self.state.fog_density = (0.3 - clear_progress * 0.15).max(0.1) + weather_fog * 0.35;
                self.state.fog_color = Vec3::new(0.65, 0.68, 0.72);
                self.state.fog_start = 25.0 + clear_progress * 40.0;
                self.state.fog_end = 200.0 + clear_progress * 100.0;
                self.state.fog_height_falloff = 0.035;
                self.state.light_shaft_intensity = (0.35 - clear_progress * 0.2) * (1.0 - cloud_coverage * 0.85);
                self.state.ambient_intensity = 0.22 + clear_progress * 0.08;
                self.state.ambient_color = Vec3::new(0.65, 0.63, 0.6);
                self.state.sun_intensity = 0.65 + clear_progress * 0.15;
                self.state.sun_color = Vec3::new(0.95, 0.85, 0.75);
            }
            TimePeriod::Midday => {
                // Midday - still moody with cloud coverage reducing brightness
                self.state.fog_density = 0.08 + weather_fog * 0.25;
                self.state.fog_color = Vec3::new(0.6, 0.63, 0.7);
                self.state.fog_start = 60.0;
                self.state.fog_end = 450.0;
                self.state.fog_height_falloff = 0.015;
                // Reduce light shafts significantly when overcast
                self.state.light_shaft_intensity = 0.1 * (1.0 - cloud_coverage * 0.95);
                self.state.ambient_intensity = 0.32 * (1.0 - cloud_coverage * 0.3);
                self.state.ambient_color = Vec3::new(0.7, 0.72, 0.78);
                self.state.sun_intensity = 0.85 * (1.0 - cloud_coverage * 0.4);
                self.state.sun_color = Vec3::new(0.95, 0.92, 0.88);
            }
            TimePeriod::Afternoon => {
                // Afternoon - moody golden hour, muted by clouds
                let golden_progress = (time_of_day - 16.0) / 2.0;
                self.state.fog_density = 0.1 + golden_progress * 0.12 + weather_fog * 0.3;
                let clear_fog = Vec3::new(0.75, 0.68, 0.55);
                let overcast_fog = Vec3::new(0.55, 0.55, 0.58);
                self.state.fog_color = clear_fog.lerp(overcast_fog, cloud_coverage);
                self.state.fog_start = 50.0;
                self.state.fog_end = 350.0;
                self.state.light_shaft_intensity = (0.25 + golden_progress * 0.3) * (1.0 - cloud_coverage * 0.8);
                self.state.ambient_intensity = 0.28 * (1.0 - cloud_coverage * 0.2);
                self.state.ambient_color = Vec3::new(0.7, 0.65, 0.55);
                self.state.sun_intensity = 0.75 * (1.0 - cloud_coverage * 0.35);
                self.state.sun_color = Vec3::new(0.95, 0.8, 0.6);
            }
            TimePeriod::Dusk => {
                self.state.fog_density = 0.15 + weather_fog * 0.3;
                self.state.fog_color = Vec3::new(0.6, 0.4, 0.35);
                self.state.fog_start = 50.0;
                self.state.fog_end = 300.0;
                self.state.light_shaft_intensity = 0.6 * (1.0 - cloud_coverage * 0.5);
                self.state.light_shaft_decay = 0.92;
                self.state.ambient_intensity = 0.15; // Reduced from 0.25 for darker dusk
                self.state.ambient_color = Vec3::new(0.6, 0.4, 0.4); // Darker reddish
                self.state.sun_intensity = 0.5;
                self.state.sun_color = Vec3::new(1.0, 0.5, 0.3);
            }
            TimePeriod::Evening => {
                let night_progress = (time_of_day - 20.0) / 2.0;
                self.state.fog_density = 0.1 + night_progress * 0.05 + weather_fog * 0.25;
                self.state.fog_color = Vec3::new(0.2, 0.2, 0.3).lerp(Vec3::new(0.005, 0.005, 0.01), night_progress); // Transition to black
                self.state.fog_start = 40.0;
                self.state.fog_end = 250.0;
                self.state.light_shaft_intensity = 0.0;
                self.state.ambient_intensity = (0.10 - night_progress * 0.05).max(0.05); // Rapidly drop to near-black
                self.state.ambient_color = Vec3::new(0.2, 0.2, 0.3);
                self.state.sun_intensity = 0.1 * (1.0 - night_progress);
                self.state.sun_color = Vec3::new(0.6, 0.4, 0.5);
            }
        }

        // Fog distances - fog should be very visible
        // fog_end at render distance to hide chunk pop-in
        let target_fog_end = render_distance * 0.9;
        self.state.fog_end = target_fog_end;
        // fog_start varies by time of day - closer at dawn/dusk for atmosphere
        let base_fog_start = match period {
            TimePeriod::Dawn | TimePeriod::Dusk => target_fog_end * 0.03,
            TimePeriod::Morning | TimePeriod::Evening => target_fog_end * 0.08,
            TimePeriod::Night => target_fog_end * 0.05,
            _ => target_fog_end * 0.12,
        };
        self.state.fog_start = base_fog_start.max(5.0);

        // Ensure minimum fog density for atmospheric feel
        // Higher baseline + weather contribution
        let min_fog = match period {
            TimePeriod::Dawn => 0.35,
            TimePeriod::Dusk => 0.30,
            TimePeriod::Morning => 0.20,
            TimePeriod::Evening => 0.25,
            TimePeriod::Night => 0.15,
            TimePeriod::Midday | TimePeriod::Afternoon => 0.12,
        };
        self.state.fog_density = self.state.fog_density.max(min_fog);
    }

    /// Get fog uniforms for shaders [density, start, end, height_falloff]
    /// Values are clamped to safe ranges to prevent shader issues
    pub fn fog_params(&self) -> [f32; 4] {
        [
            self.state.fog_density.clamp(0.0, 1.0),
            self.state.fog_start.clamp(1.0, 1000.0),
            self.state.fog_end.clamp(self.state.fog_start + 10.0, 2000.0), // End must be > start
            self.state.fog_height_falloff.clamp(0.001, 0.5),
        ]
    }

    /// Get fog color
    pub fn fog_color(&self) -> [f32; 3] {
        self.state.fog_color.to_array()
    }

    /// Get light shaft params [intensity, decay, density, 0]
    pub fn light_shaft_params(&self) -> [f32; 4] {
        [
            self.state.light_shaft_intensity,
            self.state.light_shaft_decay,
            self.state.light_shaft_density,
            0.0,
        ]
    }

    pub fn current_period(&self) -> TimePeriod {
        TimePeriod::from_hour(self.time_of_day)
    }
}
