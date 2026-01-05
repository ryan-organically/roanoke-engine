//! Audio System for Roanoke Engine
//!
//! Provides menu music, weather ambience, and procedural soundtrack generation
//! inspired by Jeremy Soule's atmospheric exploration music.

use kira::{
    manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    tween::Tween,
    Volume,
};
use std::time::Duration;
use rand::Rng;

use crate::weather_system::WeatherType;

/// Musical scales used for procedural generation (semitone intervals from root)
pub mod scales {
    /// Dorian mode - melancholic but hopeful, Soule's signature
    pub const DORIAN: [i32; 7] = [0, 2, 3, 5, 7, 9, 10];
    /// Aeolian (natural minor) - darker, mysterious
    pub const AEOLIAN: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];
    /// Pentatonic minor - safe, ancient feeling
    pub const PENTATONIC_MINOR: [i32; 5] = [0, 3, 5, 7, 10];
    /// Lydian - bright, ethereal, otherworldly
    pub const LYDIAN: [i32; 7] = [0, 2, 4, 6, 7, 9, 11];
}

/// Game states that affect the soundtrack
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicState {
    MainMenu,
    Exploration,
    Discovery,     // Finding something interesting
    Tension,       // Danger nearby
    Combat,
    Peaceful,      // Safe area, resting
}

/// Time-of-day periods affecting musical mood
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDayMood {
    Dawn,      // Hopeful, rising
    Day,       // Active, bright
    Dusk,      // Melancholic, winding down
    Night,     // Mysterious, contemplative
}

/// A single procedural layer in the soundtrack
#[derive(Debug, Clone)]
pub struct ProceduralLayer {
    pub name: String,
    pub base_volume: f32,
    pub current_volume: f32,
    pub target_volume: f32,
    pub frequency_hz: f32,      // Base frequency for synthesis
    pub note_index: usize,      // Current position in scale
    pub octave: i32,
    pub attack: f32,
    pub release: f32,
    pub is_active: bool,
}

impl ProceduralLayer {
    pub fn new(name: &str, base_freq: f32, base_vol: f32, octave: i32) -> Self {
        Self {
            name: name.to_string(),
            base_volume: base_vol,
            current_volume: 0.0,
            target_volume: 0.0,
            frequency_hz: base_freq,
            note_index: 0,
            octave,
            attack: 2.0,
            release: 3.0,
            is_active: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Smooth volume interpolation
        let rate = if self.target_volume > self.current_volume {
            self.attack
        } else {
            self.release
        };
        self.current_volume += (self.target_volume - self.current_volume) * dt * rate;
    }
}

/// Configuration for the procedural soundtrack engine
#[derive(Debug, Clone)]
pub struct SoundtrackConfig {
    /// Root note in Hz (A4 = 440)
    pub root_frequency: f32,
    /// Current musical scale
    pub scale: Vec<i32>,
    /// Tempo in BPM
    pub tempo_bpm: f32,
    /// Overall intensity (0.0 = ambient, 1.0 = dramatic)
    pub intensity: f32,
    /// Target intensity for smooth transitions
    pub target_intensity: f32,
    /// Reverb amount (0.0 - 1.0)
    pub reverb: f32,
}

impl Default for SoundtrackConfig {
    fn default() -> Self {
        Self {
            root_frequency: 220.0, // A3 - a warm, grounded root
            scale: scales::DORIAN.to_vec(),
            tempo_bpm: 60.0,
            intensity: 0.3,
            target_intensity: 0.3,
            reverb: 0.6,
        }
    }
}

/// Weather-specific ambient sound configuration
#[derive(Debug, Clone)]
pub struct WeatherAmbience {
    pub wind_volume: f32,
    pub rain_volume: f32,
    pub thunder_volume: f32,
    pub birds_volume: f32,
    pub crickets_volume: f32,

    // Target values for smooth transitions
    pub target_wind: f32,
    pub target_rain: f32,
    pub target_thunder: f32,
    pub target_birds: f32,
    pub target_crickets: f32,
}

impl WeatherAmbience {
    pub fn new() -> Self {
        Self {
            wind_volume: 0.0,
            rain_volume: 0.0,
            thunder_volume: 0.0,
            birds_volume: 0.3,
            crickets_volume: 0.0,
            target_wind: 0.0,
            target_rain: 0.0,
            target_thunder: 0.0,
            target_birds: 0.3,
            target_crickets: 0.0,
        }
    }

    pub fn set_for_weather(&mut self, weather: WeatherType, time_of_day: f32) {
        let is_night = time_of_day < 0.25 || time_of_day > 0.85;
        let is_dawn_dusk = (time_of_day > 0.2 && time_of_day < 0.35)
                        || (time_of_day > 0.75 && time_of_day < 0.9);

        match weather {
            WeatherType::Clear => {
                self.target_wind = 0.1;
                self.target_rain = 0.0;
                self.target_thunder = 0.0;
                self.target_birds = if is_night { 0.0 } else { 0.4 };
                self.target_crickets = if is_night { 0.5 } else if is_dawn_dusk { 0.3 } else { 0.0 };
            }
            WeatherType::PartlyCloudy => {
                self.target_wind = 0.2;
                self.target_rain = 0.0;
                self.target_thunder = 0.0;
                self.target_birds = if is_night { 0.0 } else { 0.3 };
                self.target_crickets = if is_night { 0.4 } else { 0.0 };
            }
            WeatherType::Overcast => {
                self.target_wind = 0.35;
                self.target_rain = 0.05; // Light drizzle possible
                self.target_thunder = 0.0;
                self.target_birds = 0.1;
                self.target_crickets = 0.0;
            }
            WeatherType::Stormy => {
                self.target_wind = 0.7;
                self.target_rain = 0.8;
                self.target_thunder = 0.6;
                self.target_birds = 0.0;
                self.target_crickets = 0.0;
            }
            WeatherType::Foggy => {
                self.target_wind = 0.05;
                self.target_rain = 0.0;
                self.target_thunder = 0.0;
                self.target_birds = if is_night { 0.0 } else { 0.15 };
                self.target_crickets = if is_night { 0.2 } else { 0.0 };
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        let transition_speed = 0.5; // Smooth 2-second transitions
        self.wind_volume += (self.target_wind - self.wind_volume) * dt * transition_speed;
        self.rain_volume += (self.target_rain - self.rain_volume) * dt * transition_speed;
        self.thunder_volume += (self.target_thunder - self.thunder_volume) * dt * transition_speed;
        self.birds_volume += (self.target_birds - self.birds_volume) * dt * transition_speed;
        self.crickets_volume += (self.target_crickets - self.crickets_volume) * dt * transition_speed;
    }
}

/// Coastal/environmental ambience based on player location
#[derive(Debug, Clone)]
pub struct CoastalAmbience {
    // Current volumes
    pub waves_near_volume: f32,
    pub beach_wind_volume: f32,
    pub forest_wind_volume: f32,      // Constant loop in forest
    pub forest_breeze_volume: f32,    // Occasional gusts
    pub mockingjay_volume: f32,       // Occasional bird calls

    // Target volumes (for smooth transitions)
    pub target_waves_near: f32,
    pub target_beach_wind: f32,
    pub target_forest_wind: f32,
    pub target_forest_breeze: f32,
    pub target_mockingjay: f32,

    // Distance thresholds
    pub waves_max_distance: f32,      // Full fade at this distance from surf
    pub beach_wind_max_distance: f32,
    pub forest_breeze_chance: f32,    // Random trigger chance per second
    pub mockingjay_chance: f32,       // Random trigger chance per second
}

impl CoastalAmbience {
    pub fn new() -> Self {
        Self {
            waves_near_volume: 0.0,
            beach_wind_volume: 0.0,
            forest_wind_volume: 0.0,
            forest_breeze_volume: 0.0,
            mockingjay_volume: 0.0,
            target_waves_near: 0.0,
            target_beach_wind: 0.0,
            target_forest_wind: 0.0,
            target_forest_breeze: 0.0,
            target_mockingjay: 0.0,
            waves_max_distance: 150.0,      // Waves audible deep inland toward forest
            beach_wind_max_distance: 100.0, // Beach wind carries far
            forest_breeze_chance: 0.02,     // 2% chance per second when in forest
            mockingjay_chance: 0.008,       // ~1 call every 2 minutes on average
        }
    }

    /// Update target volumes based on player distance to coastline/forest
    /// biome_t: 0.0 = deep ocean, 0.5 = beach, 0.72+ = forest
    pub fn set_for_location(&mut self, distance_to_surf: f32, biome_t: f32) {
        // Waves: loud at surf, taper toward forest
        // Full volume at surf, still faintly audible at forest edge
        if distance_to_surf < self.waves_max_distance {
            let t = distance_to_surf / self.waves_max_distance;
            // Cubic falloff - loud near surf, gentle fade inland
            self.target_waves_near = (1.0 - t * t * t).max(0.0);
        } else {
            self.target_waves_near = 0.0;
        }

        // Beach wind: similar but extends further
        if distance_to_surf < self.beach_wind_max_distance {
            let t = distance_to_surf / self.beach_wind_max_distance;
            self.target_beach_wind = (1.0 - t).max(0.0) * 0.5;
        } else {
            self.target_beach_wind = 0.0;
        }

        // Forest wind: gradient fade-in from coastal forest to deep woods
        // biome_t 0.65 = coastal forest edge, 0.72 = treeline, 0.85+ = deep forest
        let forest_start = 0.65;
        let forest_full = 0.80;
        if biome_t > forest_start {
            let forest_t = ((biome_t - forest_start) / (forest_full - forest_start)).clamp(0.0, 1.0);
            self.target_forest_wind = forest_t * 0.7; // Gradual fade-in
        } else {
            self.target_forest_wind = 0.0;
        }

        // Mockingjay: continuous loop in forest, layered with forest wind
        let in_forest = biome_t >= 0.72;
        if in_forest {
            self.target_mockingjay = 0.6; // Steady presence with birds
        } else {
            self.target_mockingjay = 0.0;
        }

        // Forest breeze gusts: handled externally via trigger
        if !in_forest {
            self.target_forest_breeze = 0.0;
        }
    }

    /// Check if player is in forest based on biome_t
    pub fn is_forest(biome_t: f32) -> bool {
        biome_t >= 0.72
    }

    /// Trigger a forest breeze gust
    pub fn trigger_forest_breeze(&mut self) {
        self.target_forest_breeze = 0.7;
    }

    pub fn update(&mut self, dt: f32) {
        let transition_speed = 0.8; // Slightly faster than weather for responsiveness
        let forest_transition = 0.4; // Slower fade for forest ambience
        let breeze_fade_speed = 0.3; // Slower fade for breeze gusts

        self.waves_near_volume += (self.target_waves_near - self.waves_near_volume) * dt * transition_speed;
        self.beach_wind_volume += (self.target_beach_wind - self.beach_wind_volume) * dt * transition_speed;
        self.forest_wind_volume += (self.target_forest_wind - self.forest_wind_volume) * dt * forest_transition;
        self.mockingjay_volume += (self.target_mockingjay - self.mockingjay_volume) * dt * forest_transition;

        // Forest breeze fades out after trigger
        self.forest_breeze_volume += (self.target_forest_breeze - self.forest_breeze_volume) * dt * breeze_fade_speed;
        if self.target_forest_breeze > 0.0 {
            self.target_forest_breeze -= dt * 0.15;
            if self.target_forest_breeze < 0.01 {
                self.target_forest_breeze = 0.0;
            }
        }
    }
}

impl Default for CoastalAmbience {
    fn default() -> Self {
        Self::new()
    }
}

/// The main audio system managing all game audio
pub struct AudioSystem {
    manager: Option<AudioManager>,

    // Music state
    pub music_state: MusicState,
    pub previous_music_state: MusicState,
    state_transition_timer: f32,

    // Menu music
    menu_music_handle: Option<StaticSoundHandle>,
    menu_music_volume: f32,

    // Ambient sound handles
    wind_handle: Option<StaticSoundHandle>,
    rain_handle: Option<StaticSoundHandle>,
    thunder_handle: Option<StaticSoundHandle>,
    birds_handle: Option<StaticSoundHandle>,
    crickets_handle: Option<StaticSoundHandle>,

    // Coastal/environmental sound handles
    waves_near_handle: Option<StaticSoundHandle>,
    beach_wind_handle: Option<StaticSoundHandle>,
    forest_wind_handle: Option<StaticSoundHandle>,
    forest_breeze_handle: Option<StaticSoundHandle>,
    mockingjay_handle: Option<StaticSoundHandle>,

    // Footsteps - elastic loop that adjusts to player speed
    footsteps_gravel_handle: Option<StaticSoundHandle>,
    footsteps_gravel_data: Option<StaticSoundData>,
    footsteps_water_handle: Option<StaticSoundHandle>,
    footsteps_water_data: Option<StaticSoundData>,
    footsteps_playing: bool,
    footsteps_in_water: bool, // Track which type is currently playing

    // Weather ambience system
    pub weather_ambience: WeatherAmbience,

    // Coastal ambience system
    pub coastal_ambience: CoastalAmbience,

    // Procedural soundtrack
    pub soundtrack_config: SoundtrackConfig,
    pub layers: Vec<ProceduralLayer>,

    // Procedural generation state
    beat_timer: f32,
    phrase_position: usize,
    phrase_length: usize,
    melody_cooldown: f32,

    // Master controls
    pub master_volume: f32,
    pub music_volume: f32,
    pub ambience_volume: f32,
    pub sfx_volume: f32,

    // Audio enabled state
    pub enabled: bool,
    initialized: bool,
}

impl AudioSystem {
    pub fn new() -> Self {
        // Note: ALSA errors in WSL2 are expected - audio requires PulseAudio/PipeWire bridge
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default());

        let initialized = manager.is_ok();
        if let Err(ref e) = manager {
            // Don't spam logs - just note that audio is disabled
            println!("[AUDIO] Audio unavailable (no audio device found). Game will run silently.");
            log::info!("Audio init failed: {:?}", e);
        } else {
            println!("[AUDIO] Audio system initialized successfully");
        }

        // Create procedural layers inspired by Soule's orchestration:
        // - Deep drone/pad layer (strings, choir)
        // - Mid-range harmonic layer (brass, woodwinds)
        // - High melodic fragments (solo instruments)
        // - Texture/shimmer layer (bells, harp arpeggios)
        let layers = vec![
            ProceduralLayer::new("drone", 110.0, 0.4, -1),      // Deep foundation
            ProceduralLayer::new("pad", 220.0, 0.35, 0),        // Harmonic bed
            ProceduralLayer::new("melody", 440.0, 0.25, 1),     // Melodic voice
            ProceduralLayer::new("shimmer", 880.0, 0.15, 2),    // High texture
            ProceduralLayer::new("bass", 55.0, 0.3, -2),        // Sub bass pulse
        ];

        Self {
            manager: manager.ok(),
            music_state: MusicState::MainMenu,
            previous_music_state: MusicState::MainMenu,
            state_transition_timer: 0.0,
            menu_music_handle: None,
            menu_music_volume: 0.0,
            wind_handle: None,
            rain_handle: None,
            thunder_handle: None,
            birds_handle: None,
            crickets_handle: None,
            waves_near_handle: None,
            beach_wind_handle: None,
            forest_wind_handle: None,
            forest_breeze_handle: None,
            mockingjay_handle: None,
            footsteps_gravel_handle: None,
            footsteps_gravel_data: None,
            footsteps_water_handle: None,
            footsteps_water_data: None,
            footsteps_playing: false,
            footsteps_in_water: false,
            weather_ambience: WeatherAmbience::new(),
            coastal_ambience: CoastalAmbience::new(),
            soundtrack_config: SoundtrackConfig::default(),
            layers,
            beat_timer: 0.0,
            phrase_position: 0,
            phrase_length: 8,
            melody_cooldown: 0.0,
            master_volume: 0.8,
            music_volume: 0.7,
            ambience_volume: 0.6,
            sfx_volume: 0.8,
            enabled: true,
            initialized,
        }
    }

    /// Initialize the audio system and load sounds
    pub fn initialize(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Audio manager failed to initialize".to_string());
        }

        log::info!("[AUDIO] Audio system initialized successfully");
        Ok(())
    }

    /// Load a sound file from the assets directory
    pub fn load_sound(&mut self, path: &str) -> Result<StaticSoundData, String> {
        StaticSoundData::from_file(path)
            .map_err(|e| format!("Failed to load sound '{}': {:?}", path, e))
    }

    /// Load all coastal/environmental ambient sounds
    pub fn load_coastal_sounds(&mut self) {
        // Waves near - loops when player is close to surf, audible toward forest
        if let Ok(sound) = self.load_sound("assets/audio/ambience/waves near.mp3") {
            self.load_ambient_loop("waves_near", sound);
            log::info!("[AUDIO] Loaded coastal sound: waves_near");
        } else {
            log::warn!("[AUDIO] Could not load waves near.mp3");
        }

        // Forest wind - constant ambient loop in the woods
        if let Ok(sound) = self.load_sound("assets/audio/ambience/forest wind.mp3") {
            self.load_ambient_loop("forest_wind", sound);
            log::info!("[AUDIO] Loaded coastal sound: forest_wind");
        } else {
            log::warn!("[AUDIO] Could not load forest wind.mp3");
        }

        // Mockingjay - occasional bird calls in the forest
        if let Ok(sound) = self.load_sound("assets/audio/ambience/mockingjay.mp3") {
            self.load_ambient_loop("mockingjay", sound);
            log::info!("[AUDIO] Loaded coastal sound: mockingjay");
        } else {
            log::warn!("[AUDIO] Could not load mockingjay.mp3");
        }

        // Beach wind - future: uncomment when file is added
        // if let Ok(sound) = self.load_sound("assets/audio/ambience/beach_wind.ogg") {
        //     self.load_ambient_loop("beach_wind", sound);
        // }

        // Forest breeze gusts - future: uncomment when file is added
        // if let Ok(sound) = self.load_sound("assets/audio/ambience/forest_breeze.ogg") {
        //     self.load_ambient_loop("forest_breeze", sound);
        // }
    }

    /// Play menu music with fade in
    pub fn play_menu_music(&mut self, sound_data: StaticSoundData) {
        // Stop any existing menu music first
        if let Some(ref mut handle) = self.menu_music_handle {
            let _ = handle.stop(Tween {
                duration: Duration::from_secs(2),
                ..Default::default()
            });
        }
        self.menu_music_handle = None;

        // Now play new music
        if let Some(ref mut manager) = self.manager {
            let settings = StaticSoundSettings::new()
                .volume(Volume::Amplitude(0.0)); // Start silent for fade in

            match manager.play(sound_data.with_settings(settings)) {
                Ok(handle) => {
                    self.menu_music_handle = Some(handle);
                    self.menu_music_volume = 0.0;
                    log::info!("[AUDIO] Menu music started");
                }
                Err(e) => log::error!("[AUDIO] Failed to play menu music: {:?}", e),
            }
        }
    }

    /// Stop menu music with fade out
    pub fn stop_menu_music(&mut self) {
        if let Some(ref mut handle) = self.menu_music_handle {
            let _ = handle.stop(Tween {
                duration: Duration::from_secs(2),
                ..Default::default()
            });
        }
        self.menu_music_handle = None;
    }

    /// Load and start ambient sound loop
    pub fn load_ambient_loop(&mut self, ambient_type: &str, sound_data: StaticSoundData) {
        if let Some(ref mut manager) = self.manager {
            let settings = StaticSoundSettings::new()
                .volume(Volume::Amplitude(0.0))
                .loop_region(..);

            match manager.play(sound_data.with_settings(settings)) {
                Ok(handle) => {
                    match ambient_type {
                        "wind" => self.wind_handle = Some(handle),
                        "rain" => self.rain_handle = Some(handle),
                        "thunder" => self.thunder_handle = Some(handle),
                        "birds" => self.birds_handle = Some(handle),
                        "crickets" => self.crickets_handle = Some(handle),
                        "waves_near" => self.waves_near_handle = Some(handle),
                        "beach_wind" => self.beach_wind_handle = Some(handle),
                        "forest_wind" => self.forest_wind_handle = Some(handle),
                        "forest_breeze" => self.forest_breeze_handle = Some(handle),
                        "mockingjay" => self.mockingjay_handle = Some(handle),
                        _ => log::warn!("[AUDIO] Unknown ambient type: {}", ambient_type),
                    }
                    log::info!("[AUDIO] Loaded ambient loop: {}", ambient_type);
                }
                Err(e) => log::error!("[AUDIO] Failed to load ambient '{}': {:?}", ambient_type, e),
            }
        }
    }

    /// Set the current music state (triggers transitions)
    pub fn set_music_state(&mut self, state: MusicState) {
        if state != self.music_state {
            self.previous_music_state = self.music_state;
            self.music_state = state;
            self.state_transition_timer = 5.0; // 5 second transition

            // Adjust soundtrack parameters based on state
            match state {
                MusicState::MainMenu => {
                    self.soundtrack_config.target_intensity = 0.2;
                    self.soundtrack_config.tempo_bpm = 50.0;
                    self.soundtrack_config.scale = scales::DORIAN.to_vec();
                }
                MusicState::Exploration => {
                    self.soundtrack_config.target_intensity = 0.35;
                    self.soundtrack_config.tempo_bpm = 65.0;
                    self.soundtrack_config.scale = scales::DORIAN.to_vec();
                }
                MusicState::Discovery => {
                    self.soundtrack_config.target_intensity = 0.5;
                    self.soundtrack_config.tempo_bpm = 75.0;
                    self.soundtrack_config.scale = scales::LYDIAN.to_vec();
                }
                MusicState::Tension => {
                    self.soundtrack_config.target_intensity = 0.65;
                    self.soundtrack_config.tempo_bpm = 85.0;
                    self.soundtrack_config.scale = scales::AEOLIAN.to_vec();
                }
                MusicState::Combat => {
                    self.soundtrack_config.target_intensity = 0.9;
                    self.soundtrack_config.tempo_bpm = 120.0;
                    self.soundtrack_config.scale = scales::AEOLIAN.to_vec();
                }
                MusicState::Peaceful => {
                    self.soundtrack_config.target_intensity = 0.15;
                    self.soundtrack_config.tempo_bpm = 45.0;
                    self.soundtrack_config.scale = scales::PENTATONIC_MINOR.to_vec();
                }
            }

            log::info!("[AUDIO] Music state changed to {:?}", state);
        }
    }

    /// Get the mood based on time of day (0.0 = midnight, 0.5 = noon)
    pub fn get_time_mood(&self, time_of_day: f32) -> TimeOfDayMood {
        if time_of_day >= 0.2 && time_of_day < 0.35 {
            TimeOfDayMood::Dawn
        } else if time_of_day >= 0.35 && time_of_day < 0.75 {
            TimeOfDayMood::Day
        } else if time_of_day >= 0.75 && time_of_day < 0.9 {
            TimeOfDayMood::Dusk
        } else {
            TimeOfDayMood::Night
        }
    }

    /// Update the procedural soundtrack engine
    fn update_procedural_soundtrack(&mut self, dt: f32, time_of_day: f32) {
        let mut rng = rand::thread_rng();

        // Smooth intensity transition
        let intensity_rate = 0.3;
        self.soundtrack_config.intensity +=
            (self.soundtrack_config.target_intensity - self.soundtrack_config.intensity) * dt * intensity_rate;

        // Beat timing
        let beat_duration = 60.0 / self.soundtrack_config.tempo_bpm;
        self.beat_timer += dt;

        if self.beat_timer >= beat_duration {
            self.beat_timer -= beat_duration;
            self.phrase_position = (self.phrase_position + 1) % self.phrase_length;

            // On phrase boundaries, potentially evolve the music
            if self.phrase_position == 0 {
                self.evolve_phrase(&mut rng);
            }
        }

        // Update layer target volumes based on intensity and time
        let time_mood = self.get_time_mood(time_of_day);
        let intensity = self.soundtrack_config.intensity;

        for layer in &mut self.layers {
            let base = layer.base_volume;
            layer.target_volume = match layer.name.as_str() {
                "drone" => {
                    // Drone is always present, stronger at night
                    let night_boost = match time_mood {
                        TimeOfDayMood::Night => 0.15,
                        TimeOfDayMood::Dusk | TimeOfDayMood::Dawn => 0.05,
                        TimeOfDayMood::Day => 0.0,
                    };
                    (base + night_boost) * (0.5 + intensity * 0.5)
                }
                "pad" => {
                    // Pad rises with intensity
                    base * (0.3 + intensity * 0.7)
                }
                "melody" => {
                    // Melody only at medium+ intensity, rare
                    if intensity > 0.3 && self.melody_cooldown <= 0.0 {
                        base * intensity
                    } else {
                        layer.current_volume * 0.95 // Fade out
                    }
                }
                "shimmer" => {
                    // Shimmer during peaceful/discovery moments
                    let shimmer_intensity = match self.music_state {
                        MusicState::Peaceful | MusicState::Discovery => 0.8,
                        MusicState::Exploration => 0.4,
                        _ => 0.1,
                    };
                    base * shimmer_intensity
                }
                "bass" => {
                    // Bass pulses during tension/combat
                    let bass_intensity = match self.music_state {
                        MusicState::Combat => 1.0,
                        MusicState::Tension => 0.7,
                        _ => 0.2,
                    };
                    base * bass_intensity * intensity
                }
                _ => base * intensity,
            };

            layer.update(dt);
        }

        // Melody cooldown management
        if self.melody_cooldown > 0.0 {
            self.melody_cooldown -= dt;
        }
    }

    /// Evolve the musical phrase (called at phrase boundaries)
    fn evolve_phrase(&mut self, rng: &mut impl Rng) {
        let scale = &self.soundtrack_config.scale;

        // Chance to trigger a melodic fragment
        if rng.gen_bool(0.3) && self.melody_cooldown <= 0.0 {
            if let Some(melody_layer) = self.layers.iter_mut().find(|l| l.name == "melody") {
                melody_layer.note_index = rng.gen_range(0..scale.len());
                melody_layer.is_active = true;
                self.melody_cooldown = rng.gen_range(8.0..20.0);
            }
        }

        // Occasionally shift the drone note for harmonic movement
        if rng.gen_bool(0.15) {
            if let Some(drone_layer) = self.layers.iter_mut().find(|l| l.name == "drone") {
                // Move to a consonant scale degree (root, fifth, fourth)
                let consonant_degrees = [0, 4, 3]; // Root, 5th, 4th in scale
                drone_layer.note_index = consonant_degrees[rng.gen_range(0..consonant_degrees.len())];
            }
        }
    }

    /// Update ambient sound volumes based on weather
    fn update_ambient_volumes(&mut self) {
        let master = self.master_volume * self.ambience_volume;

        if let Some(ref mut handle) = self.wind_handle {
            let vol = self.weather_ambience.wind_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
        if let Some(ref mut handle) = self.rain_handle {
            let vol = self.weather_ambience.rain_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
        if let Some(ref mut handle) = self.thunder_handle {
            let vol = self.weather_ambience.thunder_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
        if let Some(ref mut handle) = self.birds_handle {
            let vol = self.weather_ambience.birds_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
        if let Some(ref mut handle) = self.crickets_handle {
            let vol = self.weather_ambience.crickets_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
    }

    /// Update coastal ambient sound volumes based on player location
    fn update_coastal_volumes(&mut self) {
        let master = self.master_volume * self.ambience_volume;

        if let Some(ref mut handle) = self.waves_near_handle {
            let vol = self.coastal_ambience.waves_near_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
        if let Some(ref mut handle) = self.beach_wind_handle {
            let vol = self.coastal_ambience.beach_wind_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
        if let Some(ref mut handle) = self.forest_wind_handle {
            let vol = self.coastal_ambience.forest_wind_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
        if let Some(ref mut handle) = self.forest_breeze_handle {
            let vol = self.coastal_ambience.forest_breeze_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
        if let Some(ref mut handle) = self.mockingjay_handle {
            let vol = self.coastal_ambience.mockingjay_volume * master;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
    }

    /// Update coastal ambience based on player position
    /// Call this from game code with distance to nearest coastline and biome_t value
    pub fn update_coastal(&mut self, dt: f32, distance_to_surf: f32, biome_t: f32) {
        self.coastal_ambience.set_for_location(distance_to_surf, biome_t);

        let in_forest = CoastalAmbience::is_forest(biome_t);

        // Random forest breeze gusts
        if in_forest && self.coastal_ambience.forest_breeze_volume < 0.1 {
            let chance = self.coastal_ambience.forest_breeze_chance * dt;
            if rand::random::<f32>() < chance {
                self.coastal_ambience.trigger_forest_breeze();
            }
        }

        self.coastal_ambience.update(dt);
        self.update_coastal_volumes();
    }

    /// Update menu music volume with fade
    fn update_menu_music(&mut self, dt: f32) {
        if self.music_state == MusicState::MainMenu {
            // Fade in
            self.menu_music_volume = (self.menu_music_volume + dt * 0.5).min(1.0);
        } else {
            // Fade out
            self.menu_music_volume = (self.menu_music_volume - dt * 0.3).max(0.0);
        }

        if let Some(ref mut handle) = self.menu_music_handle {
            let vol = self.menu_music_volume * self.master_volume * self.music_volume;
            let _ = handle.set_volume(Volume::Amplitude(vol as f64), Tween::default());
        }
    }

    /// Main update function - call every frame
    pub fn update(&mut self, dt: f32, weather: WeatherType, time_of_day: f32) {
        if !self.enabled || !self.initialized {
            return;
        }

        // Update state transition timer
        if self.state_transition_timer > 0.0 {
            self.state_transition_timer -= dt;
        }

        // Update weather ambience
        self.weather_ambience.set_for_weather(weather, time_of_day);
        self.weather_ambience.update(dt);
        self.update_ambient_volumes();

        // Update menu music
        self.update_menu_music(dt);

        // Update procedural soundtrack
        self.update_procedural_soundtrack(dt, time_of_day);
    }

    /// Get the current frequency for a layer based on scale position
    pub fn get_layer_frequency(&self, layer_index: usize) -> f32 {
        if layer_index >= self.layers.len() {
            return 0.0;
        }

        let layer = &self.layers[layer_index];
        let scale = &self.soundtrack_config.scale;
        let root = self.soundtrack_config.root_frequency;

        if layer.note_index >= scale.len() {
            return root;
        }

        let semitones = scale[layer.note_index] + (layer.octave * 12);
        root * 2.0_f32.powf(semitones as f32 / 12.0)
    }

    /// Play a one-shot sound effect
    pub fn play_sfx(&mut self, sound_data: StaticSoundData) {
        if let Some(ref mut manager) = self.manager {
            let vol = self.master_volume * self.sfx_volume;
            let settings = StaticSoundSettings::new()
                .volume(Volume::Amplitude(vol as f64));

            if let Err(e) = manager.play(sound_data.with_settings(settings)) {
                log::error!("[AUDIO] Failed to play SFX: {:?}", e);
            }
        }
    }

    /// Load footsteps sounds for elastic playback (gravel and water variants)
    pub fn load_footsteps(&mut self) {
        if let Ok(sound) = self.load_sound("assets/audio/sfx/gravel foosteps elastic.wav") {
            self.footsteps_gravel_data = Some(sound);
            log::info!("[AUDIO] Loaded gravel footsteps sound");
        } else {
            log::warn!("[AUDIO] Could not load gravel footsteps sound");
        }

        if let Ok(sound) = self.load_sound("assets/audio/sfx/shallow water foosteps elastic.wav") {
            self.footsteps_water_data = Some(sound);
            log::info!("[AUDIO] Loaded water footsteps sound");
        } else {
            log::warn!("[AUDIO] Could not load water footsteps sound");
        }
    }

    /// Update footsteps based on player movement speed and terrain
    /// speed: horizontal movement speed (0 = idle, ~5 = walking, ~10+ = sprinting)
    /// near_water: true if player is within ~5 yards of shoreline or in shallow water
    pub fn update_footsteps(&mut self, speed: f32, near_water: bool) {
        let is_moving = speed > 0.5;

        // Check if we need to switch footstep types
        let needs_type_switch = self.footsteps_playing && (near_water != self.footsteps_in_water);

        if needs_type_switch {
            // Stop current footsteps to switch types
            self.stop_footsteps();
        }

        if is_moving && !self.footsteps_playing {
            // Start appropriate footsteps loop
            let sound_data = if near_water {
                self.footsteps_water_data.clone()
            } else {
                self.footsteps_gravel_data.clone()
            };

            if let Some(sound) = sound_data {
                if let Some(ref mut manager) = self.manager {
                    let vol = self.master_volume * self.sfx_volume * 0.6;
                    let settings = StaticSoundSettings::new()
                        .volume(Volume::Amplitude(vol as f64))
                        .loop_region(..);

                    if let Ok(handle) = manager.play(sound.with_settings(settings)) {
                        if near_water {
                            self.footsteps_water_handle = Some(handle);
                        } else {
                            self.footsteps_gravel_handle = Some(handle);
                        }
                        self.footsteps_playing = true;
                        self.footsteps_in_water = near_water;
                        log::debug!("[AUDIO] Started {} footsteps loop", if near_water { "water" } else { "gravel" });
                    }
                }
            }
        } else if !is_moving && self.footsteps_playing {
            self.stop_footsteps();
        }

        // Adjust playback rate based on speed
        if self.footsteps_playing {
            let handle = if self.footsteps_in_water {
                self.footsteps_water_handle.as_mut()
            } else {
                self.footsteps_gravel_handle.as_mut()
            };

            if let Some(h) = handle {
                // Map speed to playback rate: 0.7x at slow walk, 1.0x at normal, 1.5x at sprint
                let rate = (speed / 7.0).clamp(0.7, 1.5);
                let _ = h.set_playback_rate(rate as f64, Tween {
                    duration: Duration::from_millis(50),
                    ..Default::default()
                });
            }
        }
    }

    /// Stop any playing footsteps
    fn stop_footsteps(&mut self) {
        if let Some(ref mut handle) = self.footsteps_gravel_handle {
            let _ = handle.stop(Tween {
                duration: Duration::from_millis(100),
                ..Default::default()
            });
        }
        if let Some(ref mut handle) = self.footsteps_water_handle {
            let _ = handle.stop(Tween {
                duration: Duration::from_millis(100),
                ..Default::default()
            });
        }
        self.footsteps_gravel_handle = None;
        self.footsteps_water_handle = None;
        self.footsteps_playing = false;
        log::debug!("[AUDIO] Stopped footsteps loop");
    }

    /// Get current layer volumes for external synthesis/visualization
    pub fn get_layer_volumes(&self) -> Vec<(String, f32, f32)> {
        self.layers
            .iter()
            .map(|l| (l.name.clone(), l.current_volume, self.get_layer_frequency(
                self.layers.iter().position(|x| x.name == l.name).unwrap_or(0)
            )))
            .collect()
    }

    /// Check if audio system is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set master volume (0.0 - 1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set music volume (0.0 - 1.0)
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    /// Set ambience volume (0.0 - 1.0)
    pub fn set_ambience_volume(&mut self, volume: f32) {
        self.ambience_volume = volume.clamp(0.0, 1.0);
    }

    /// Set SFX volume (0.0 - 1.0)
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Toggle audio on/off
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        log::info!("[AUDIO] Audio {}", if self.enabled { "enabled" } else { "disabled" });
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}
