//! Audio Events System
//!
//! Central integration hub connecting game systems to the audio engine.
//! Provides biome-aware music, animal encounter triggers, NPC ambience,
//! faction themes, and progression audio cues.

use crate::audio_system::{AudioSystem, MusicState, scales};
use crate::weather_system::WeatherType;

/// Biome types for audio mapping (mirrors croatoan_wfc::BiomeType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioBiome {
    // Coastal
    Ocean,
    Beach,
    SaltMarsh,
    CoastalScrub,
    // Lowland
    Grassland,
    DeciduousForest,
    Wetland,
    River,
    // Highland
    Foothills,
    RollingMountains,
    MountainPeak,
    AlpineMeadow,
    // Special
    Cave,
    Waterfall,
    CanyonRiver,
    // Village/settlement
    Village,
}

/// Audio characteristics for each biome
#[derive(Debug, Clone)]
pub struct BiomeAudioProfile {
    pub scale: Vec<i32>,
    pub tempo_modifier: f32,      // Multiplier on base tempo
    pub reverb_amount: f32,       // 0.0 - 1.0
    pub intensity_base: f32,      // Base intensity level
    pub root_note_offset: i32,    // Semitones from A3
    // Ambient sound weights
    pub wind_weight: f32,
    pub water_weight: f32,
    pub birds_weight: f32,
    pub insects_weight: f32,
}

impl BiomeAudioProfile {
    pub fn for_biome(biome: AudioBiome) -> Self {
        match biome {
            AudioBiome::Ocean => Self {
                scale: scales::AEOLIAN.to_vec(),
                tempo_modifier: 0.7,
                reverb_amount: 0.8,
                intensity_base: 0.25,
                root_note_offset: -5, // E3 - deep, mysterious
                wind_weight: 0.6,
                water_weight: 1.0,
                birds_weight: 0.3, // Seabirds
                insects_weight: 0.0,
            },
            AudioBiome::Beach => Self {
                scale: scales::LYDIAN.to_vec(),
                tempo_modifier: 0.85,
                reverb_amount: 0.5,
                intensity_base: 0.3,
                root_note_offset: 0, // A3
                wind_weight: 0.4,
                water_weight: 0.7,
                birds_weight: 0.5,
                insects_weight: 0.1,
            },
            AudioBiome::SaltMarsh | AudioBiome::Wetland => Self {
                scale: scales::DORIAN.to_vec(),
                tempo_modifier: 0.75,
                reverb_amount: 0.6,
                intensity_base: 0.2,
                root_note_offset: -3, // F#3 - murky
                wind_weight: 0.2,
                water_weight: 0.5,
                birds_weight: 0.6,
                insects_weight: 0.8,
            },
            AudioBiome::CoastalScrub => Self {
                scale: scales::DORIAN.to_vec(),
                tempo_modifier: 0.9,
                reverb_amount: 0.4,
                intensity_base: 0.3,
                root_note_offset: 2, // B3
                wind_weight: 0.5,
                water_weight: 0.2,
                birds_weight: 0.6,
                insects_weight: 0.4,
            },
            AudioBiome::Grassland => Self {
                scale: scales::LYDIAN.to_vec(),
                tempo_modifier: 1.0,
                reverb_amount: 0.3,
                intensity_base: 0.35,
                root_note_offset: 5, // D4 - bright, open
                wind_weight: 0.7,
                water_weight: 0.0,
                birds_weight: 0.7,
                insects_weight: 0.5,
            },
            AudioBiome::DeciduousForest => Self {
                scale: scales::DORIAN.to_vec(),
                tempo_modifier: 0.85,
                reverb_amount: 0.55,
                intensity_base: 0.3,
                root_note_offset: 0, // A3 - Jeremy Soule's favorite
                wind_weight: 0.3,
                water_weight: 0.1,
                birds_weight: 0.8,
                insects_weight: 0.4,
            },
            AudioBiome::River | AudioBiome::CanyonRiver => Self {
                scale: scales::PENTATONIC_MINOR.to_vec(),
                tempo_modifier: 0.9,
                reverb_amount: 0.5,
                intensity_base: 0.35,
                root_note_offset: -2, // G3
                wind_weight: 0.2,
                water_weight: 0.9,
                birds_weight: 0.5,
                insects_weight: 0.3,
            },
            AudioBiome::Foothills => Self {
                scale: scales::DORIAN.to_vec(),
                tempo_modifier: 0.95,
                reverb_amount: 0.45,
                intensity_base: 0.35,
                root_note_offset: 3, // C4
                wind_weight: 0.5,
                water_weight: 0.1,
                birds_weight: 0.5,
                insects_weight: 0.3,
            },
            AudioBiome::RollingMountains => Self {
                scale: scales::AEOLIAN.to_vec(),
                tempo_modifier: 0.8,
                reverb_amount: 0.7,
                intensity_base: 0.4,
                root_note_offset: 5, // D4 - epic
                wind_weight: 0.8,
                water_weight: 0.1,
                birds_weight: 0.3,
                insects_weight: 0.1,
            },
            AudioBiome::MountainPeak => Self {
                scale: scales::AEOLIAN.to_vec(),
                tempo_modifier: 0.65,
                reverb_amount: 0.85,
                intensity_base: 0.45,
                root_note_offset: 7, // E4 - triumphant
                wind_weight: 1.0,
                water_weight: 0.0,
                birds_weight: 0.1, // Eagles only
                insects_weight: 0.0,
            },
            AudioBiome::AlpineMeadow => Self {
                scale: scales::LYDIAN.to_vec(),
                tempo_modifier: 0.85,
                reverb_amount: 0.6,
                intensity_base: 0.35,
                root_note_offset: 4, // C#4 - ethereal
                wind_weight: 0.6,
                water_weight: 0.2, // Mountain streams
                birds_weight: 0.4,
                insects_weight: 0.2,
            },
            AudioBiome::Cave => Self {
                scale: scales::AEOLIAN.to_vec(),
                tempo_modifier: 0.5,
                reverb_amount: 0.95, // Maximum reverb
                intensity_base: 0.2,
                root_note_offset: -7, // D3 - deep, ominous
                wind_weight: 0.1, // Cave wind
                water_weight: 0.4, // Dripping
                birds_weight: 0.0,
                insects_weight: 0.2, // Cave insects
            },
            AudioBiome::Waterfall => Self {
                scale: scales::LYDIAN.to_vec(),
                tempo_modifier: 0.9,
                reverb_amount: 0.7,
                intensity_base: 0.5, // Dramatic
                root_note_offset: 0, // A3
                wind_weight: 0.3,
                water_weight: 1.0,
                birds_weight: 0.3,
                insects_weight: 0.2,
            },
            AudioBiome::Village => Self {
                scale: scales::DORIAN.to_vec(),
                tempo_modifier: 1.0,
                reverb_amount: 0.3,
                intensity_base: 0.25,
                root_note_offset: 0, // A3 - warm, familiar
                wind_weight: 0.2,
                water_weight: 0.0,
                birds_weight: 0.4,
                insects_weight: 0.2,
            },
        }
    }
}

/// Animal encounter threat levels for audio response
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreatLevel {
    None,
    Passive,      // Deer, rabbits - peaceful
    Curious,      // Animals investigating player
    Wary,         // Animals on alert
    Aggressive,   // Active threat
    Combat,       // In combat
    Fleeing,      // Animal fleeing
}

/// Audio events that game systems can trigger
#[derive(Debug, Clone)]
pub enum AudioEvent {
    // Biome transitions
    BiomeEntered(AudioBiome),
    BiomeTransition { from: AudioBiome, to: AudioBiome, blend: f32 },

    // Animal encounters
    AnimalDetected { species: String, threat: ThreatLevel, distance: f32 },
    AnimalCombatStart { species: String },
    AnimalCombatEnd { victory: bool },
    AnimalFleeing { species: String },
    PackHowl,  // Wolf pack coordination

    // NPC/Village
    VillageEntered { population: u32 },
    VillageExited,
    NpcGreeting,
    NpcFarewell,
    TradeStarted,
    TradeCompleted { profit: bool },
    DialogueStarted,
    DialogueEnded,

    // Faction
    FactionReputationChanged { faction: String, positive: bool },
    FactionTerritoryEntered { faction: String, standing: f32 },
    FactionAlliance,
    FactionHostile,

    // Progression
    SkillLevelUp { skill: String },
    DiscoveryMade { category: String, rarity: f32 },
    QuestAccepted,
    QuestCompleted,
    QuestFailed,
    EncyclopediaUnlock { entry_type: String },

    // Environment
    CaveEntered { depth: f32 },
    CaveExited,
    WaterfallNearby { distance: f32 },
    UndergroundWater,

    // Special moments
    SunriseBegins,
    SunsetBegins,
    FirstSnowfall,
    StormApproaching,
}

/// Faction audio themes
#[derive(Debug, Clone)]
pub struct FactionTheme {
    pub scale: Vec<i32>,
    pub tempo_modifier: f32,
    pub root_offset: i32,
    pub intensity_boost: f32,
    pub reverb: f32,
}

impl FactionTheme {
    /// Get theme for a faction by name
    pub fn for_faction(faction_name: &str) -> Self {
        match faction_name.to_lowercase().as_str() {
            "powhatan" | "powhatan confederacy" => Self {
                scale: scales::PENTATONIC_MINOR.to_vec(),
                tempo_modifier: 0.85,
                root_offset: 0, // A3
                intensity_boost: 0.1,
                reverb: 0.5,
            },
            "colonists" | "jamestown" => Self {
                scale: scales::DORIAN.to_vec(),
                tempo_modifier: 1.0,
                root_offset: 2, // B3
                intensity_boost: 0.15,
                reverb: 0.4,
            },
            "croatoan" => Self {
                scale: scales::AEOLIAN.to_vec(),
                tempo_modifier: 0.7,
                root_offset: -5, // E3 - mysterious
                intensity_boost: 0.2,
                reverb: 0.7,
            },
            "pirates" | "privateers" => Self {
                scale: scales::DORIAN.to_vec(),
                tempo_modifier: 1.2,
                root_offset: -2, // G3
                intensity_boost: 0.25,
                reverb: 0.3,
            },
            "spanish" | "spanish empire" => Self {
                scale: scales::AEOLIAN.to_vec(), // Phrygian would be ideal
                tempo_modifier: 0.95,
                root_offset: 3, // C4
                intensity_boost: 0.2,
                reverb: 0.5,
            },
            _ => Self {
                scale: scales::DORIAN.to_vec(),
                tempo_modifier: 1.0,
                root_offset: 0,
                intensity_boost: 0.0,
                reverb: 0.5,
            },
        }
    }
}

/// Audio integration manager - connects all game systems to audio
pub struct AudioEventProcessor {
    current_biome: AudioBiome,
    current_biome_profile: BiomeAudioProfile,
    threat_level: ThreatLevel,
    in_village: bool,
    in_cave: bool,
    active_faction_theme: Option<String>,

    // Transition state
    biome_blend: f32,
    target_biome: Option<AudioBiome>,
    threat_decay_timer: f32,

    // Discovery stinger cooldown
    discovery_cooldown: f32,

    // Volume modifiers
    village_ambience_volume: f32,
    cave_reverb_boost: f32,
}

impl AudioEventProcessor {
    pub fn new() -> Self {
        let initial_biome = AudioBiome::DeciduousForest;
        Self {
            current_biome: initial_biome,
            current_biome_profile: BiomeAudioProfile::for_biome(initial_biome),
            threat_level: ThreatLevel::None,
            in_village: false,
            in_cave: false,
            active_faction_theme: None,
            biome_blend: 0.0,
            target_biome: None,
            threat_decay_timer: 0.0,
            discovery_cooldown: 0.0,
            village_ambience_volume: 0.0,
            cave_reverb_boost: 0.0,
        }
    }

    /// Process an audio event and update the audio system accordingly
    pub fn process_event(&mut self, event: AudioEvent, audio: &mut AudioSystem) {
        match event {
            // Biome events
            AudioEvent::BiomeEntered(biome) => {
                self.target_biome = Some(biome);
                self.biome_blend = 0.0;
            }

            AudioEvent::BiomeTransition { from: _, to, blend } => {
                if blend >= 1.0 {
                    self.current_biome = to;
                    self.current_biome_profile = BiomeAudioProfile::for_biome(to);
                    self.target_biome = None;
                }
                self.biome_blend = blend;
            }

            // Animal events
            AudioEvent::AnimalDetected { species: _, threat, distance } => {
                let new_threat = self.calculate_effective_threat(threat, distance);
                if new_threat as u8 > self.threat_level as u8 {
                    self.threat_level = new_threat;
                    self.threat_decay_timer = 5.0; // 5 seconds before relaxing
                }
                self.update_music_for_threat(audio);
            }

            AudioEvent::AnimalCombatStart { species: _ } => {
                self.threat_level = ThreatLevel::Combat;
                self.threat_decay_timer = 0.0; // No decay during combat
                audio.set_music_state(MusicState::Combat);
            }

            AudioEvent::AnimalCombatEnd { victory } => {
                self.threat_level = ThreatLevel::None;
                if victory {
                    // Triumphant return to exploration
                    audio.set_music_state(MusicState::Discovery);
                } else {
                    audio.set_music_state(MusicState::Tension);
                }
                self.threat_decay_timer = 3.0;
            }

            AudioEvent::AnimalFleeing { species: _ } => {
                if self.threat_level == ThreatLevel::Combat {
                    self.threat_level = ThreatLevel::Wary;
                    audio.set_music_state(MusicState::Exploration);
                }
            }

            AudioEvent::PackHowl => {
                // Dramatic tension spike
                self.threat_level = ThreatLevel::Aggressive;
                self.threat_decay_timer = 8.0;
                audio.soundtrack_config.target_intensity =
                    (audio.soundtrack_config.target_intensity + 0.3).min(0.8);
            }

            // Village events
            AudioEvent::VillageEntered { population } => {
                self.in_village = true;
                self.village_ambience_volume = (population as f32 / 50.0).min(1.0);
                let village_profile = BiomeAudioProfile::for_biome(AudioBiome::Village);
                audio.soundtrack_config.scale = village_profile.scale;
                audio.set_music_state(MusicState::Peaceful);
            }

            AudioEvent::VillageExited => {
                self.in_village = false;
                self.village_ambience_volume = 0.0;
                // Restore biome audio
                audio.soundtrack_config.scale = self.current_biome_profile.scale.clone();
                audio.set_music_state(MusicState::Exploration);
            }

            AudioEvent::TradeStarted => {
                audio.soundtrack_config.target_intensity = 0.2;
            }

            AudioEvent::TradeCompleted { profit } => {
                if profit {
                    audio.soundtrack_config.target_intensity = 0.4;
                }
            }

            AudioEvent::DialogueStarted => {
                // Lower music during dialogue
                audio.soundtrack_config.target_intensity = 0.15;
            }

            AudioEvent::DialogueEnded => {
                audio.soundtrack_config.target_intensity =
                    self.current_biome_profile.intensity_base;
            }

            AudioEvent::NpcGreeting | AudioEvent::NpcFarewell => {
                // Could trigger short melodic stinger
            }

            // Faction events
            AudioEvent::FactionTerritoryEntered { faction, standing } => {
                self.active_faction_theme = Some(faction.clone());
                let theme = FactionTheme::for_faction(&faction);

                // Blend faction theme with current biome
                audio.soundtrack_config.scale = theme.scale;
                audio.soundtrack_config.target_intensity =
                    self.current_biome_profile.intensity_base + theme.intensity_boost;

                // Adjust music state based on standing
                if standing < -0.5 {
                    audio.set_music_state(MusicState::Tension);
                } else if standing > 0.5 {
                    audio.set_music_state(MusicState::Peaceful);
                }
            }

            AudioEvent::FactionReputationChanged { faction: _, positive } => {
                if positive {
                    audio.set_music_state(MusicState::Discovery);
                } else {
                    audio.soundtrack_config.target_intensity += 0.1;
                }
            }

            AudioEvent::FactionAlliance => {
                audio.set_music_state(MusicState::Discovery);
                audio.soundtrack_config.target_intensity = 0.5;
            }

            AudioEvent::FactionHostile => {
                audio.set_music_state(MusicState::Tension);
                audio.soundtrack_config.target_intensity = 0.7;
            }

            // Progression events
            AudioEvent::SkillLevelUp { skill: _ } => {
                if self.discovery_cooldown <= 0.0 {
                    audio.set_music_state(MusicState::Discovery);
                    self.discovery_cooldown = 2.0;
                }
            }

            AudioEvent::DiscoveryMade { category: _, rarity } => {
                if self.discovery_cooldown <= 0.0 {
                    audio.set_music_state(MusicState::Discovery);
                    // Intensity based on rarity
                    audio.soundtrack_config.target_intensity =
                        (0.4 + rarity * 0.4).min(0.9);
                    self.discovery_cooldown = 3.0;
                }
            }

            AudioEvent::QuestAccepted => {
                audio.soundtrack_config.target_intensity += 0.15;
            }

            AudioEvent::QuestCompleted => {
                audio.set_music_state(MusicState::Discovery);
                audio.soundtrack_config.target_intensity = 0.6;
            }

            AudioEvent::QuestFailed => {
                audio.soundtrack_config.scale = scales::AEOLIAN.to_vec();
                audio.soundtrack_config.target_intensity = 0.3;
            }

            AudioEvent::EncyclopediaUnlock { entry_type: _ } => {
                if self.discovery_cooldown <= 0.0 {
                    // Brief discovery moment
                    audio.soundtrack_config.target_intensity = 0.5;
                    self.discovery_cooldown = 1.5;
                }
            }

            // Cave events
            AudioEvent::CaveEntered { depth } => {
                self.in_cave = true;
                let cave_profile = BiomeAudioProfile::for_biome(AudioBiome::Cave);
                audio.soundtrack_config.scale = cave_profile.scale;
                audio.soundtrack_config.reverb = cave_profile.reverb_amount;
                audio.soundtrack_config.tempo_bpm *= cave_profile.tempo_modifier;
                self.cave_reverb_boost = (depth / 50.0).min(0.3);
            }

            AudioEvent::CaveExited => {
                self.in_cave = false;
                audio.soundtrack_config.scale = self.current_biome_profile.scale.clone();
                audio.soundtrack_config.reverb = self.current_biome_profile.reverb_amount;
                self.cave_reverb_boost = 0.0;
            }

            AudioEvent::WaterfallNearby { distance } => {
                let waterfall_intensity = (1.0 - distance / 100.0).max(0.0);
                audio.weather_ambience.target_rain =
                    audio.weather_ambience.target_rain.max(waterfall_intensity * 0.5);
            }

            AudioEvent::UndergroundWater => {
                audio.weather_ambience.target_rain = 0.3; // Dripping sounds
            }

            // Time events
            AudioEvent::SunriseBegins => {
                audio.soundtrack_config.scale = scales::LYDIAN.to_vec();
                audio.soundtrack_config.target_intensity = 0.45;
            }

            AudioEvent::SunsetBegins => {
                audio.soundtrack_config.scale = scales::DORIAN.to_vec();
                audio.soundtrack_config.target_intensity = 0.35;
            }

            AudioEvent::FirstSnowfall => {
                audio.soundtrack_config.target_intensity = 0.4;
                audio.soundtrack_config.tempo_bpm = 50.0;
            }

            AudioEvent::StormApproaching => {
                audio.set_music_state(MusicState::Tension);
                audio.weather_ambience.set_for_weather(WeatherType::Stormy, 0.5);
            }
        }
    }

    /// Calculate effective threat based on distance
    fn calculate_effective_threat(&self, base_threat: ThreatLevel, distance: f32) -> ThreatLevel {
        if distance > 50.0 {
            return ThreatLevel::None;
        }

        match base_threat {
            ThreatLevel::Combat => ThreatLevel::Combat,
            ThreatLevel::Aggressive => {
                if distance < 20.0 { ThreatLevel::Aggressive }
                else if distance < 35.0 { ThreatLevel::Wary }
                else { ThreatLevel::Curious }
            }
            ThreatLevel::Wary => {
                if distance < 25.0 { ThreatLevel::Wary }
                else { ThreatLevel::Curious }
            }
            _ => base_threat,
        }
    }

    /// Update music state based on current threat level
    fn update_music_for_threat(&self, audio: &mut AudioSystem) {
        let state = match self.threat_level {
            ThreatLevel::None | ThreatLevel::Passive => MusicState::Exploration,
            ThreatLevel::Curious => MusicState::Exploration,
            ThreatLevel::Wary => MusicState::Tension,
            ThreatLevel::Aggressive => MusicState::Tension,
            ThreatLevel::Combat => MusicState::Combat,
            ThreatLevel::Fleeing => MusicState::Exploration,
        };
        audio.set_music_state(state);
    }

    /// Update method - call every frame
    pub fn update(&mut self, dt: f32, audio: &mut AudioSystem) {
        // Biome transition blending
        if let Some(target) = self.target_biome {
            self.biome_blend += dt * 0.3; // 3+ second transitions
            if self.biome_blend >= 1.0 {
                self.current_biome = target;
                self.current_biome_profile = BiomeAudioProfile::for_biome(target);
                self.target_biome = None;
                self.biome_blend = 0.0;

                // Apply new biome audio settings
                if !self.in_cave && !self.in_village && self.active_faction_theme.is_none() {
                    audio.soundtrack_config.scale = self.current_biome_profile.scale.clone();
                    audio.soundtrack_config.reverb = self.current_biome_profile.reverb_amount;
                    audio.soundtrack_config.target_intensity = self.current_biome_profile.intensity_base;
                }
            }
        }

        // Threat decay
        if self.threat_decay_timer > 0.0 {
            self.threat_decay_timer -= dt;
            if self.threat_decay_timer <= 0.0 && self.threat_level != ThreatLevel::Combat {
                self.threat_level = ThreatLevel::None;
                self.update_music_for_threat(audio);
            }
        }

        // Discovery cooldown
        if self.discovery_cooldown > 0.0 {
            self.discovery_cooldown -= dt;
        }

        // Apply biome ambient weights to weather ambience
        if !self.in_village && !self.in_cave {
            let profile = &self.current_biome_profile;
            // Modulate ambient sounds based on biome
            audio.weather_ambience.target_wind =
                audio.weather_ambience.target_wind.max(profile.wind_weight * 0.3);
            audio.weather_ambience.target_birds =
                audio.weather_ambience.target_birds * 0.7 + profile.birds_weight * 0.3;
        }

        // Cave reverb boost
        if self.in_cave {
            audio.soundtrack_config.reverb =
                (audio.soundtrack_config.reverb + self.cave_reverb_boost).min(0.95);
        }
    }

    /// Get current biome for external queries
    pub fn current_biome(&self) -> AudioBiome {
        self.current_biome
    }

    /// Check if currently in combat
    pub fn is_in_combat(&self) -> bool {
        self.threat_level == ThreatLevel::Combat
    }

    /// Get current threat level
    pub fn threat_level(&self) -> ThreatLevel {
        self.threat_level
    }
}

impl Default for AudioEventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to convert biome type from WFC crate
pub fn wfc_biome_to_audio(biome_name: &str) -> AudioBiome {
    match biome_name.to_lowercase().as_str() {
        "ocean" => AudioBiome::Ocean,
        "beach" => AudioBiome::Beach,
        "saltmarsh" | "salt_marsh" => AudioBiome::SaltMarsh,
        "coastalscrub" | "coastal_scrub" => AudioBiome::CoastalScrub,
        "grassland" => AudioBiome::Grassland,
        "deciduousforest" | "deciduous_forest" | "forest" => AudioBiome::DeciduousForest,
        "wetland" => AudioBiome::Wetland,
        "river" => AudioBiome::River,
        "foothills" => AudioBiome::Foothills,
        "rollingmountains" | "rolling_mountains" | "mountains" => AudioBiome::RollingMountains,
        "mountainpeak" | "mountain_peak" | "peak" => AudioBiome::MountainPeak,
        "alpinemeadow" | "alpine_meadow" | "alpine" => AudioBiome::AlpineMeadow,
        "cave" => AudioBiome::Cave,
        "waterfall" => AudioBiome::Waterfall,
        "canyonriver" | "canyon_river" | "canyon" => AudioBiome::CanyonRiver,
        _ => AudioBiome::DeciduousForest, // Default fallback
    }
}

/// Helper to map animal species to threat profile
pub fn species_threat_profile(species: &str) -> ThreatLevel {
    match species.to_lowercase().as_str() {
        "black bear" | "blackbear" => ThreatLevel::Wary,
        "eastern cougar" | "easterncougar" | "cougar" | "mountain lion" => ThreatLevel::Aggressive,
        "gray wolf" | "graywolf" | "wolf" => ThreatLevel::Aggressive,
        "timber rattlesnake" | "timberrattlesnake" | "rattlesnake" => ThreatLevel::Wary,
        "american alligator" | "americanalligator" | "alligator" => ThreatLevel::Aggressive,
        "wild boar" | "wildboar" | "boar" => ThreatLevel::Wary,
        "copperhead" => ThreatLevel::Wary,
        "red wolf" | "redwolf" => ThreatLevel::Aggressive,
        "bobcat" => ThreatLevel::Wary,
        "cottonmouth" => ThreatLevel::Wary,
        "deer" | "white-tailed deer" => ThreatLevel::Passive,
        "rabbit" | "eastern cottontail" => ThreatLevel::Passive,
        "turkey" | "wild turkey" => ThreatLevel::Passive,
        "fox" | "red fox" => ThreatLevel::Curious,
        "raccoon" => ThreatLevel::Curious,
        _ => ThreatLevel::Passive,
    }
}
