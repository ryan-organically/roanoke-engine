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
    pub transition_timer: f32,
    pub transition_duration: f32,
    pub time_since_last_change: f32,
    
    // Cloud Parameters (Current interpolated values)
    pub cloud_coverage: f32,
    pub cloud_density: f32,
    pub cloud_scale: f32,
    pub cloud_color_base: Vec3,
    pub cloud_color_shade: Vec3,
    pub wind_offset: [f32; 2],
    
    // Target Parameters
    target_coverage: f32,
    target_density: f32,
    target_scale: f32,
    target_color_base: Vec3,
    target_color_shade: Vec3,
}

impl WeatherSystem {
    pub fn new() -> Self {
        let mut system = Self {
            current_weather: WeatherType::PartlyCloudy,
            target_weather: WeatherType::PartlyCloudy,
            transition_timer: 0.0,
            transition_duration: 10.0,
            time_since_last_change: 0.0,

            cloud_coverage: 0.45,
            cloud_density: 0.55,
            cloud_scale: 1.2,
            cloud_color_base: Vec3::new(0.92, 0.90, 0.88), // Warm cream
            cloud_color_shade: Vec3::new(0.78, 0.75, 0.82), // Soft lavender gray
            wind_offset: [0.0, 0.0],

            target_coverage: 0.45,
            target_density: 0.55,
            target_scale: 1.2,
            target_color_base: Vec3::new(0.92, 0.90, 0.88),
            target_color_shade: Vec3::new(0.78, 0.75, 0.82),
        };
        system.set_weather(WeatherType::PartlyCloudy, true);
        system
    }

    pub fn update(&mut self, dt: f32) {
        self.time_since_last_change += dt;
        self.wind_offset[0] += dt * 0.002; // Very slow wind drift
        
        // Random weather change every 60-120 seconds
        if self.time_since_last_change > 60.0 {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.005) { // Small chance per frame after 60s
                let next_weather = match rng.gen_range(0..5) {
                    0 => WeatherType::Clear,
                    1 => WeatherType::PartlyCloudy,
                    2 => WeatherType::Overcast,
                    3 => WeatherType::Stormy,
                    _ => WeatherType::Foggy,
                };
                println!("[WEATHER] Changing to {:?}", next_weather);
                self.set_weather(next_weather, false);
                self.time_since_last_change = 0.0;
            }
        }

        // Interpolate parameters
        if self.transition_timer > 0.0 {
            self.transition_timer -= dt;
            let t = 1.0 - (self.transition_timer / self.transition_duration).clamp(0.0, 1.0);
            
            // Smoothstep interpolation
            let t = t * t * (3.0 - 2.0 * t);
            
            self.cloud_coverage = lerp(self.cloud_coverage, self.target_coverage, t * dt); // Simple lerp for now
            self.cloud_density = lerp(self.cloud_density, self.target_density, t * dt);
            self.cloud_scale = lerp(self.cloud_scale, self.target_scale, t * dt);
            self.cloud_color_base = self.cloud_color_base.lerp(self.target_color_base, t * dt);
            self.cloud_color_shade = self.cloud_color_shade.lerp(self.target_color_shade, t * dt);
            
            // If transition finished
            if self.transition_timer <= 0.0 {
                self.current_weather = self.target_weather;
            }
        } else {
             // Keep drifting towards target slowly to fix any lerp inaccuracies
            self.cloud_coverage = lerp(self.cloud_coverage, self.target_coverage, dt);
            self.cloud_density = lerp(self.cloud_density, self.target_density, dt);
            self.cloud_scale = lerp(self.cloud_scale, self.target_scale, dt);
            self.cloud_color_base = self.cloud_color_base.lerp(self.target_color_base, dt);
            self.cloud_color_shade = self.cloud_color_shade.lerp(self.target_color_shade, dt);
        }
    }

    pub fn set_weather(&mut self, weather: WeatherType, instant: bool) {
        self.target_weather = weather;
        self.transition_duration = if instant { 0.0 } else { 20.0 }; // 20s transition
        self.transition_timer = self.transition_duration;

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
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
