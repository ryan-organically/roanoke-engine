use glam::Vec3;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherType {
    Clear,
    PartlyCloudy,
    Overcast,
    Stormy,
    Foggy,
}

pub struct WeatherSystem {
    pub current_weather: WeatherType,
    pub target_weather: WeatherType,
    pub transition_progress: f32,  // 0.0 = start, 1.0 = complete
    pub transition_duration: f32,
    pub time_since_last_change: f32,

    // Cloud Parameters (Current interpolated values)
    pub cloud_coverage: f32,
    pub cloud_density: f32,
    pub cloud_scale: f32,
    pub cloud_color_base: Vec3,
    pub cloud_color_shade: Vec3,
    pub wind_offset: [f32; 2],

    // Starting Parameters (for smooth interpolation)
    start_coverage: f32,
    start_density: f32,
    start_scale: f32,
    start_color_base: Vec3,
    start_color_shade: Vec3,

    // Target Parameters
    target_coverage: f32,
    target_density: f32,
    target_scale: f32,
    target_color_base: Vec3,
    target_color_shade: Vec3,

    // Dev control: disable auto weather changes
    pub auto_weather_enabled: bool,
}

impl WeatherSystem {
    pub fn new() -> Self {
        // Default to moody, overcast weather
        let mut system = Self {
            current_weather: WeatherType::Overcast,
            target_weather: WeatherType::Overcast,
            transition_progress: 1.0, // Start fully transitioned
            transition_duration: 60.0,
            time_since_last_change: 0.0,

            cloud_coverage: 0.75,
            cloud_density: 0.7,
            cloud_scale: 0.9,
            cloud_color_base: Vec3::new(0.6, 0.62, 0.65), // Cool grey
            cloud_color_shade: Vec3::new(0.45, 0.47, 0.52), // Darker grey
            wind_offset: [0.0, 0.0],

            // Start values (same as current initially)
            start_coverage: 0.75,
            start_density: 0.7,
            start_scale: 0.9,
            start_color_base: Vec3::new(0.6, 0.62, 0.65),
            start_color_shade: Vec3::new(0.45, 0.47, 0.52),

            target_coverage: 0.75,
            target_density: 0.7,
            target_scale: 0.9,
            target_color_base: Vec3::new(0.6, 0.62, 0.65),
            target_color_shade: Vec3::new(0.45, 0.47, 0.52),

            auto_weather_enabled: true, // Enable auto weather by default
        };
        system.set_weather(WeatherType::Overcast, true);
        system
    }

    pub fn update(&mut self, dt: f32) {
        self.time_since_last_change += dt;
        self.wind_offset[0] += dt * 0.002; // Very slow wind drift

        // Random weather change every 60-120 seconds (only if auto weather enabled)
        if self.auto_weather_enabled && self.time_since_last_change > 60.0 {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.005) { // Small chance per frame after 60s
                let next_weather = match rng.gen_range(0..5) {
                    0 => WeatherType::Clear,
                    1 => WeatherType::PartlyCloudy,
                    2 => WeatherType::Overcast,
                    3 => WeatherType::Stormy,
                    _ => WeatherType::Foggy,
                };
                println!("[WEATHER] Auto-changing to {:?}", next_weather);
                self.set_weather(next_weather, false);
                self.time_since_last_change = 0.0;
            }
        }

        // Interpolate parameters using proper lerp from start to target
        if self.transition_progress < 1.0 {
            // Advance transition progress
            if self.transition_duration > 0.0 {
                self.transition_progress += dt / self.transition_duration;
            } else {
                self.transition_progress = 1.0;
            }
            self.transition_progress = self.transition_progress.clamp(0.0, 1.0);

            // Smoothstep for smooth easing
            let t = self.transition_progress;
            let smooth_t = t * t * (3.0 - 2.0 * t);

            // Interpolate between start and target
            self.cloud_coverage = lerp(self.start_coverage, self.target_coverage, smooth_t);
            self.cloud_density = lerp(self.start_density, self.target_density, smooth_t);
            self.cloud_scale = lerp(self.start_scale, self.target_scale, smooth_t);
            self.cloud_color_base = self.start_color_base.lerp(self.target_color_base, smooth_t);
            self.cloud_color_shade = self.start_color_shade.lerp(self.target_color_shade, smooth_t);

            // If transition finished
            if self.transition_progress >= 1.0 {
                self.current_weather = self.target_weather;
            }
        }
    }

    pub fn set_weather(&mut self, weather: WeatherType, instant: bool) {
        self.target_weather = weather;
        // Store current values as start values for smooth interpolation
        self.start_coverage = self.cloud_coverage;
        self.start_density = self.cloud_density;
        self.start_scale = self.cloud_scale;
        self.start_color_base = self.cloud_color_base;
        self.start_color_shade = self.cloud_color_shade;

        // 60-90s transition for gradual, realistic weather changes
        self.transition_duration = if instant { 0.0 } else { 60.0 + rand::thread_rng().gen_range(0.0..30.0) };
        self.transition_progress = 0.0; // Reset progress

        match weather {
            WeatherType::Clear => {
                self.target_coverage = 0.15;
                self.target_density = 0.3;
                self.target_scale = 1.5;
                self.target_color_base = Vec3::new(0.95, 0.95, 0.97); // Soft white
                self.target_color_shade = Vec3::new(0.85, 0.87, 0.92); // Light blue-gray
            }
            WeatherType::PartlyCloudy => {
                self.target_coverage = 0.45;
                self.target_density = 0.55;
                self.target_scale = 1.2;
                // Soft cream and lavender
                self.target_color_base = Vec3::new(0.92, 0.90, 0.88); // Warm cream
                self.target_color_shade = Vec3::new(0.78, 0.75, 0.82); // Soft lavender gray
            }
            WeatherType::Overcast => {
                self.target_coverage = 0.85;
                self.target_density = 0.7;
                self.target_scale = 0.9;
                self.target_color_base = Vec3::new(0.7, 0.7, 0.72); // Cool gray
                self.target_color_shade = Vec3::new(0.55, 0.55, 0.58); // Darker gray
            }
            WeatherType::Stormy => {
                self.target_coverage = 0.95;
                self.target_density = 0.9;
                self.target_scale = 0.7;
                self.target_color_base = Vec3::new(0.35, 0.35, 0.4); // Dark blue-gray
                self.target_color_shade = Vec3::new(0.2, 0.2, 0.25); // Deep slate
            }
            WeatherType::Foggy => {
                self.target_coverage = 0.4;
                self.target_density = 0.25;
                self.target_scale = 2.5;
                self.target_color_base = Vec3::new(0.88, 0.88, 0.9); // Pale gray
                self.target_color_shade = Vec3::new(0.82, 0.82, 0.86); // Misty
            }
        }

        if instant {
            self.cloud_coverage = self.target_coverage;
            self.cloud_density = self.target_density;
            self.cloud_scale = self.target_scale;
            self.cloud_color_base = self.target_color_base;
            self.cloud_color_shade = self.target_color_shade;
            self.current_weather = weather;
            self.transition_progress = 1.0;
        }

        println!("[WEATHER] Transitioning to {:?} (coverage: {:.2} -> {:.2})",
                 weather, self.start_coverage, self.target_coverage);
    }

    /// Get rain intensity (0-1) for rendering systems
    pub fn rain_intensity(&self) -> f32 {
        match self.current_weather {
            WeatherType::Stormy => 0.8,
            WeatherType::Overcast => 0.0, // Overcast but not raining
            _ => 0.0,
        }
    }

    /// Get ambient dimming factor (0-1) for moody atmosphere
    pub fn ambient_dimming(&self) -> f32 {
        // Use interpolated cloud coverage to determine dimming
        let base_dimming = match self.current_weather {
            WeatherType::Clear => 0.0,
            WeatherType::PartlyCloudy => 0.05,
            WeatherType::Overcast => 0.2,
            WeatherType::Stormy => 0.4,
            WeatherType::Foggy => 0.15,
        };
        // Additional dimming from cloud coverage
        base_dimming + self.cloud_coverage * 0.1
    }

    /// Get wind strength for rain angle
    pub fn wind_strength(&self) -> f32 {
        match self.current_weather {
            WeatherType::Stormy => 2.5,
            WeatherType::Overcast => 0.8,
            WeatherType::PartlyCloudy => 0.5,
            _ => 0.3,
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
