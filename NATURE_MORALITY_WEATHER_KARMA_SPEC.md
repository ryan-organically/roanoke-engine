# Nature Morality, Weather Karma & Supernatural Consequences

## Roanoke Engine - Spiritual Ecosystem Framework

This document specifies the interconnected systems of nature morality, spiritual consequences, and weather-based karma that reflect the beliefs of both Native peoples and superstitious colonial settlers.

---

## Table of Contents

1. [Overview](#overview)
2. [Nature Morality System](#nature-morality-system)
3. [Karma Accumulation](#karma-accumulation)
4. [Weather Consequences](#weather-consequences)
5. [Storm & Hurricane System](#storm--hurricane-system)
6. [Supernatural Events](#supernatural-events)
7. [Spirit Animals & Omens](#spirit-animals--omens)
8. [Redemption Mechanics](#redemption-mechanics)
9. [Integration with Other Systems](#integration-with-other-systems)
10. [Data Structures](#data-structures)

---

## Overview

### Design Philosophy

In the 16th century, both Native Americans and European colonists saw the natural world as spiritually alive. The Powhatan believed that mistreating nature invited the wrath of spirits. European settlers carried superstitions about omens and divine punishment. This system makes those beliefs tangibly real.

**The land remembers what you do.**

### Core Concepts

| Concept | Description |
|---------|-------------|
| Nature Balance | A meter tracking harmony vs. exploitation |
| Karma Events | Weather and wildlife respond to player actions |
| Spirit Wrath | Extreme negative karma triggers supernatural events |
| Omens | Warning signs before major consequences |
| Redemption | Ways to restore balance |
| Blessings | Rewards for respectful behavior |

### Thematic Pillars

1. **Actions Have Consequences**: Every kill, every tree felled, every plant harvested matters
2. **The Spirits Are Watching**: Invisible forces track your behavior
3. **Balance Can Be Restored**: Bad karma is not permanent
4. **Respect Is Rewarded**: Living in harmony grants blessings
5. **Ignorance Is Partly Excused**: Not knowing makes it less severe
6. **Excess Is Punished**: Taking more than you need angers the spirits

---

## Nature Morality System

### The Balance Meter

```rust
// morality/balance.rs

/// The player's relationship with the natural world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatureBalance {
    /// Current balance: -100 (destroyer) to +100 (guardian)
    pub value: f32,

    /// Historical low point
    pub lowest_point: f32,

    /// Historical high point
    pub highest_point: f32,

    /// Accumulated positive actions
    pub positive_karma: f32,

    /// Accumulated negative actions
    pub negative_karma: f32,

    /// Current tier
    pub tier: BalanceTier,

    /// Pending karma events
    pub pending_events: Vec<KarmaEvent>,

    /// Active effects from current balance
    pub active_effects: Vec<BalanceEffect>,

    /// Time since last major transgression
    pub healing_timer: f32,

    /// Spirit attention level (high = more supernatural events)
    pub spirit_attention: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalanceTier {
    // Negative tiers
    Destroyer,         // -100 to -75: Nature actively fights you
    Despoiler,         // -75 to -50: Severe penalties, storms
    Exploiter,         // -50 to -25: Moderate penalties, animal hostility
    Taker,             // -25 to -10: Minor penalties, fewer resources

    // Neutral
    Neutral,           // -10 to +10: No special effects

    // Positive tiers
    Respectful,        // +10 to +25: Minor bonuses, animal trust
    Protector,         // +25 to +50: Moderate bonuses, spirit guidance
    Guardian,          // +50 to +75: Major bonuses, spirit allies
    OneWithNature,     // +75 to +100: Supernatural harmony
}

impl BalanceTier {
    pub fn from_value(value: f32) -> Self {
        match value {
            v if v <= -75.0 => Self::Destroyer,
            v if v <= -50.0 => Self::Despoiler,
            v if v <= -25.0 => Self::Exploiter,
            v if v <= -10.0 => Self::Taker,
            v if v < 10.0 => Self::Neutral,
            v if v < 25.0 => Self::Respectful,
            v if v < 50.0 => Self::Protector,
            v if v < 75.0 => Self::Guardian,
            _ => Self::OneWithNature,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Destroyer => "Destroyer of Nature",
            Self::Despoiler => "Despoiler of the Land",
            Self::Exploiter => "Exploiter",
            Self::Taker => "Taker",
            Self::Neutral => "Wanderer",
            Self::Respectful => "Respectful Traveler",
            Self::Protector => "Protector of Nature",
            Self::Guardian => "Guardian of the Wild",
            Self::OneWithNature => "One With Nature",
        }
    }
}
```

### Action Karma Values

```rust
/// Actions and their karma impact
#[derive(Debug, Clone)]
pub struct KarmaAction {
    pub action_type: ActionType,
    pub base_impact: f32,       // Base karma change
    pub modifiers: Vec<KarmaModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    // === NEGATIVE ACTIONS ===
    // Killing
    KillAnimal,
    KillDocileAnimal,
    KillPregnantAnimal,
    KillYoungAnimal,
    KillRareAnimal,
    KillSpiritAnimal,
    OverkillWaste,          // Kill without harvesting

    // Harvesting
    OverharvharvPlant,      // Take all from area
    DestroyRarePlant,
    HarvestOutOfSeason,
    HarvestWithoutNeed,

    // Environment
    ClearCutArea,           // Remove many trees
    PollutWater,
    BurnForest,
    DisturbNest,
    DisturbDen,
    DestroySacredSite,

    // === POSITIVE ACTIONS ===
    // Conservation
    SparePrey,              // Let wounded animal go
    ProtectYoung,
    PlantTree,
    PlantSeeds,
    FeedAnimal,
    HealAnimal,

    // Respect
    PrayBeforeHunt,
    LeaveOffering,
    HarvestResponsibly,     // Leave some behind
    UseEntireKill,          // Waste nothing
    FollowTaboo,

    // Sacred
    ProtectSacredSite,
    ParticipateInCeremony,
    ReleaseCaptiveAnimal,
}

pub fn get_base_karma(action: ActionType) -> f32 {
    match action {
        // Severe negative
        ActionType::KillSpiritAnimal => -25.0,
        ActionType::DestroySacredSite => -20.0,
        ActionType::BurnForest => -15.0,
        ActionType::KillPregnantAnimal => -12.0,
        ActionType::KillYoungAnimal => -10.0,

        // Moderate negative
        ActionType::ClearCutArea => -8.0,
        ActionType::OverkillWaste => -5.0,
        ActionType::KillDocileAnimal => -3.0,
        ActionType::OverharvestPlant => -3.0,
        ActionType::DisturbNest => -4.0,
        ActionType::DisturbDen => -5.0,
        ActionType::KillRareAnimal => -6.0,

        // Minor negative
        ActionType::KillAnimal => -1.0,  // Hunting is natural
        ActionType::HarvestOutOfSeason => -2.0,
        ActionType::HarvestWithoutNeed => -1.5,
        ActionType::DestroyRarePlant => -3.0,
        ActionType::PollutWater => -4.0,

        // Minor positive
        ActionType::HarvestResponsibly => 0.5,
        ActionType::UseEntireKill => 1.0,
        ActionType::PlantSeeds => 0.5,
        ActionType::FeedAnimal => 0.5,
        ActionType::FollowTaboo => 1.0,

        // Moderate positive
        ActionType::PlantTree => 2.0,
        ActionType::SparePrey => 2.0,
        ActionType::ProtectYoung => 3.0,
        ActionType::HealAnimal => 2.5,
        ActionType::PrayBeforeHunt => 1.5,
        ActionType::LeaveOffering => 2.0,

        // Significant positive
        ActionType::ReleaseCaptiveAnimal => 4.0,
        ActionType::ProtectSacredSite => 5.0,
        ActionType::ParticipateInCeremony => 6.0,
    }
}
```

### Karma Modifiers

Context affects karma impact:

```rust
#[derive(Debug, Clone)]
pub enum KarmaModifier {
    // Knowledge modifiers
    KnewBetter(f32),        // Encyclopedia knowledge multiplier
    NativeWarning(f32),     // Was warned by NPC
    Ignorance(f32),         // Didn't know (reduces penalty)

    // Need modifiers
    Starving(f32),          // Was starving (reduces penalty for killing for food)
    Plenty(f32),            // Had plenty (increases penalty)
    SurvivalNeed(f32),      // Genuine survival situation

    // Method modifiers
    CleanKill(f32),         // Quick, merciful kill
    CruelKill(f32),         // Prolonged suffering
    WastedKill(f32),        // Didn't use the body

    // Target modifiers
    SacredSpecies(f32),     // Species has spiritual significance
    Endangered(f32),        // Rare species
    Pest(f32),              // Harmful species (reduces penalty)

    // Location modifiers
    SacredGround(f32),      // Near sacred site
    ProtectedArea(f32),     // Marked as protected
    WasteLand(f32),         // Already damaged area

    // Time modifiers
    SacredTime(f32),        // During ceremony or sacred period
    WrongSeason(f32),       // Out of season

    // Repetition modifiers
    FirstOffense(f32),      // First time
    RepeatOffender(f32),    // Done this before
    SerialKiller(f32),      // Excessive killing
}

impl NatureBalance {
    pub fn apply_action(&mut self, action: ActionType, modifiers: &[KarmaModifier]) {
        let base = get_base_karma(action);

        let mut final_impact = base;

        for modifier in modifiers {
            match modifier {
                KarmaModifier::KnewBetter(mult) => final_impact *= mult,
                KarmaModifier::Ignorance(mult) => final_impact *= mult,
                KarmaModifier::Starving(mult) => final_impact *= mult,
                KarmaModifier::CleanKill(mult) => final_impact *= mult,
                KarmaModifier::CruelKill(mult) => final_impact *= mult,
                KarmaModifier::SacredSpecies(mult) => final_impact *= mult,
                KarmaModifier::SacredGround(mult) => final_impact *= mult,
                KarmaModifier::RepeatOffender(mult) => final_impact *= mult,
                _ => {}
            }
        }

        // Track accumulation
        if final_impact < 0.0 {
            self.negative_karma += final_impact.abs();
            self.spirit_attention += final_impact.abs() * 0.1;
        } else {
            self.positive_karma += final_impact;
        }

        // Apply to balance
        self.value = (self.value + final_impact).clamp(-100.0, 100.0);

        // Update tier
        let new_tier = BalanceTier::from_value(self.value);
        if new_tier != self.tier {
            self.on_tier_change(self.tier, new_tier);
            self.tier = new_tier;
        }

        // Track extremes
        if self.value < self.lowest_point {
            self.lowest_point = self.value;
        }
        if self.value > self.highest_point {
            self.highest_point = self.value;
        }

        // Check for karma events
        self.check_karma_events();
    }
}
```

---

## Karma Accumulation

### Tracking Systems

```rust
/// Detailed tracking of player's nature interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarmaLedger {
    // Kill tracking
    pub animals_killed: HashMap<AnimalSpecies, u32>,
    pub total_kills: u32,
    pub wasteful_kills: u32,
    pub mercy_given: u32,

    // Harvest tracking
    pub plants_harvested: HashMap<FloraSpecies, u32>,
    pub plants_over_harvested: u32,
    pub seeds_planted: u32,
    pub trees_felled: u32,
    pub trees_planted: u32,

    // Sacred tracking
    pub sacred_sites_desecrated: u32,
    pub sacred_sites_protected: u32,
    pub offerings_made: u32,
    pub taboos_broken: u32,
    pub taboos_honored: u32,

    // Environmental
    pub fires_started: u32,
    pub water_polluted: u32,
    pub nests_disturbed: u32,

    // Timeline
    pub recent_actions: VecDeque<TimestampedAction>,
    pub worst_transgression: Option<WorstAction>,
    pub best_deed: Option<BestAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedAction {
    pub action: ActionType,
    pub timestamp: f64,
    pub karma_impact: f32,
    pub location: Vec3,
    pub details: String,
}

impl KarmaLedger {
    /// Check if player is becoming a serial killer of a species
    pub fn is_hunting_excessively(&self, species: AnimalSpecies) -> bool {
        let count = self.animals_killed.get(&species).copied().unwrap_or(0);
        let def = get_animal_def(species);

        // More than 10 of common species, 5 of uncommon, 2 of rare
        match def.rarity {
            Rarity::Common => count > 10,
            Rarity::Uncommon => count > 5,
            Rarity::Rare => count > 2,
            Rarity::VeryRare => count > 1,
        }
    }

    /// Check for recent killing spree
    pub fn recent_killing_spree(&self, window_seconds: f64) -> Option<u32> {
        let now = get_game_time();
        let recent_kills = self.recent_actions.iter()
            .filter(|a| now - a.timestamp < window_seconds)
            .filter(|a| matches!(a.action, ActionType::KillAnimal | ActionType::KillDocileAnimal))
            .count() as u32;

        if recent_kills >= 5 { Some(recent_kills) } else { None }
    }
}
```

### Karma Thresholds

```rust
/// Thresholds that trigger events
pub struct KarmaThresholds {
    // Negative thresholds
    pub minor_transgression: f32,      // -15: Warning omen
    pub moderate_transgression: f32,   // -35: Weather change
    pub major_transgression: f32,      // -55: Storm summoned
    pub severe_transgression: f32,     // -75: Spirit wrath
    pub ultimate_transgression: f32,   // -90: Catastrophe

    // Positive thresholds
    pub minor_blessing: f32,           // +15: Small luck
    pub moderate_blessing: f32,        // +35: Spirit aid
    pub major_blessing: f32,           // +55: Nature ally
    pub supreme_blessing: f32,         // +75: Spirit guardian
    pub transcendence: f32,            // +95: One with nature
}

impl NatureBalance {
    pub fn check_karma_events(&mut self) {
        // Check for negative events
        if self.value <= -15.0 && !self.has_pending_event(EventType::Warning) {
            self.queue_event(KarmaEvent::warning_omen());
        }

        if self.value <= -35.0 && !self.has_pending_event(EventType::WeatherShift) {
            self.queue_event(KarmaEvent::weather_darkens());
        }

        if self.value <= -55.0 && !self.has_pending_event(EventType::Storm) {
            self.queue_event(KarmaEvent::storm_summoned());
        }

        if self.value <= -75.0 && !self.has_pending_event(EventType::SpiritWrath) {
            self.queue_event(KarmaEvent::spirit_wrath());
        }

        if self.value <= -90.0 && !self.has_pending_event(EventType::Catastrophe) {
            self.queue_event(KarmaEvent::catastrophe());
        }

        // Check for positive events
        if self.value >= 25.0 && !self.has_pending_event(EventType::Blessing) {
            self.queue_event(KarmaEvent::nature_blessing());
        }

        if self.value >= 50.0 && !self.has_pending_event(EventType::SpiritGuide) {
            self.queue_event(KarmaEvent::spirit_guide_appears());
        }

        if self.value >= 75.0 && !self.has_pending_event(EventType::SpiritAlly) {
            self.queue_event(KarmaEvent::gain_spirit_ally());
        }
    }
}
```

---

## Weather Consequences

### Weather as Punishment

The weather itself responds to player actions:

```rust
// weather_karma.rs

/// Weather modification based on karma
pub struct KarmaWeatherSystem {
    pub base_weather: WeatherSystem,
    pub karma_influence: f32,        // How much karma affects weather
    pub storm_charge: f32,           // Building toward storm
    pub hurricane_threat: f32,       // Building toward hurricane
    pub drought_days: u32,           // Days of punishing sun
}

impl KarmaWeatherSystem {
    pub fn update(&mut self, balance: &NatureBalance, dt: f32) {
        let karma = balance.value;

        // Negative karma makes bad weather more likely
        if karma < -25.0 {
            self.storm_charge += (-karma / 100.0) * dt * 0.01;

            // Extreme negative karma builds hurricane
            if karma < -60.0 {
                self.hurricane_threat += (-karma - 60.0) / 40.0 * dt * 0.005;
            }
        }

        // Positive karma clears weather
        if karma > 25.0 {
            self.storm_charge = (self.storm_charge - dt * 0.02).max(0.0);
            self.hurricane_threat = (self.hurricane_threat - dt * 0.01).max(0.0);
        }

        // Trigger events at thresholds
        if self.storm_charge >= 1.0 {
            self.trigger_karma_storm(balance);
            self.storm_charge = 0.0;
        }

        if self.hurricane_threat >= 1.0 {
            self.trigger_karma_hurricane(balance);
            self.hurricane_threat = 0.0;
        }
    }

    fn trigger_karma_storm(&mut self, balance: &NatureBalance) {
        // Storm severity based on karma level
        let severity = match balance.tier {
            BalanceTier::Destroyer => StormSeverity::Catastrophic,
            BalanceTier::Despoiler => StormSeverity::Severe,
            BalanceTier::Exploiter => StormSeverity::Moderate,
            _ => StormSeverity::Minor,
        };

        self.base_weather.summon_storm(Storm {
            severity,
            duration: 3600.0 * (1.0 + (-balance.value / 50.0)),
            lightning_rate: severity.lightning_rate(),
            wind_speed: severity.wind_speed(),
            rainfall_rate: severity.rainfall_rate(),
            is_supernatural: true,
            karma_source: true,
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StormSeverity {
    Minor,          // Brief rain, low wind
    Moderate,       // Heavy rain, thunder
    Severe,         // Dangerous lightning, flooding risk
    Catastrophic,   // Hurricane-force, destruction
}

impl StormSeverity {
    pub fn wind_speed(&self) -> f32 {
        match self {
            Self::Minor => 15.0,
            Self::Moderate => 35.0,
            Self::Severe => 60.0,
            Self::Catastrophic => 100.0,
        }
    }

    pub fn lightning_rate(&self) -> f32 {
        match self {
            Self::Minor => 0.01,
            Self::Moderate => 0.05,
            Self::Severe => 0.15,
            Self::Catastrophic => 0.3,
        }
    }

    pub fn damage_per_second(&self) -> f32 {
        match self {
            Self::Minor => 0.0,
            Self::Moderate => 0.0,
            Self::Severe => 0.5,
            Self::Catastrophic => 2.0,
        }
    }
}
```

### Weather Effects on Gameplay

```rust
/// Active weather effects during karma storms
#[derive(Debug, Clone)]
pub struct KarmaStormEffects {
    // Environmental
    pub visibility: f32,          // 0.0 = blind, 1.0 = clear
    pub movement_penalty: f32,    // Speed reduction
    pub hearing_penalty: f32,     // Reduced sound detection

    // Damage
    pub exposure_damage: f32,     // Damage from being outside
    pub lightning_chance: f32,    // Chance of direct strike

    // Gameplay
    pub animals_flee: bool,       // Animals run from storm
    pub animals_aggressive: bool, // Predators empowered
    pub plants_wither: bool,      // Plants take damage
    pub fires_start: bool,        // Lightning starts fires

    // Supernatural
    pub spirit_manifestations: bool,
    pub ghost_sightings: bool,
    pub omen_frequency: f32,
}

impl KarmaStormEffects {
    pub fn from_severity(severity: StormSeverity, is_supernatural: bool) -> Self {
        let base = match severity {
            StormSeverity::Minor => Self {
                visibility: 0.8,
                movement_penalty: 0.1,
                hearing_penalty: 0.2,
                exposure_damage: 0.0,
                lightning_chance: 0.001,
                animals_flee: false,
                animals_aggressive: false,
                plants_wither: false,
                fires_start: false,
                spirit_manifestations: false,
                ghost_sightings: false,
                omen_frequency: 0.0,
            },
            StormSeverity::Moderate => Self {
                visibility: 0.5,
                movement_penalty: 0.2,
                hearing_penalty: 0.4,
                exposure_damage: 0.1,
                lightning_chance: 0.005,
                animals_flee: true,
                animals_aggressive: false,
                plants_wither: false,
                fires_start: false,
                spirit_manifestations: false,
                ghost_sightings: false,
                omen_frequency: 0.1,
            },
            StormSeverity::Severe => Self {
                visibility: 0.3,
                movement_penalty: 0.4,
                hearing_penalty: 0.6,
                exposure_damage: 0.5,
                lightning_chance: 0.02,
                animals_flee: true,
                animals_aggressive: true,
                plants_wither: true,
                fires_start: true,
                spirit_manifestations: is_supernatural,
                ghost_sightings: is_supernatural,
                omen_frequency: 0.3,
            },
            StormSeverity::Catastrophic => Self {
                visibility: 0.1,
                movement_penalty: 0.6,
                hearing_penalty: 0.8,
                exposure_damage: 2.0,
                lightning_chance: 0.05,
                animals_flee: false, // Too scared to move
                animals_aggressive: true,
                plants_wither: true,
                fires_start: true,
                spirit_manifestations: true,
                ghost_sightings: true,
                omen_frequency: 0.8,
            },
        };

        if is_supernatural {
            // Supernatural storms have enhanced effects
            Self {
                spirit_manifestations: true,
                ghost_sightings: true,
                omen_frequency: base.omen_frequency * 2.0,
                ..base
            }
        } else {
            base
        }
    }
}
```

### Bad Luck Weather Events

Specific weather events punish bad karma:

```rust
#[derive(Debug, Clone)]
pub enum BadLuckWeatherEvent {
    /// Lightning strikes near player
    CloseCallLightning {
        distance: f32,
        warning_time: f32,
    },

    /// Flash flood threatens area
    FlashFlood {
        affected_area: AABB,
        water_level: f32,
        duration: f32,
    },

    /// Fog rolls in suddenly, disorienting
    SupernaturalFog {
        visibility: f32,
        causes_hallucinations: bool,
    },

    /// Unnatural cold snap
    SuddenFrost {
        temperature_drop: f32,
        crop_damage: f32,
    },

    /// Drought specifically targeting player's area
    LocalizedDrought {
        affected_radius: f32,
        water_sources_dry: bool,
    },

    /// Perpetual overcast following player
    DarkCloud {
        duration: f32,
        mood_penalty: f32,
    },

    /// Wind that seems to follow and hinder
    HeadWind {
        direction: Vec3,
        strength: f32,
        follows_player: bool,
    },

    /// Rain that only falls on player
    TargetedRain {
        duration: f32,
        equipment_damage: f32,
    },
}

impl BadLuckWeatherEvent {
    pub fn choose_for_karma(karma: f32) -> Option<Self> {
        if karma > -15.0 { return None; }

        let severity = (-karma / 20.0).floor() as u32;
        let roll = rand::random::<f32>();

        match severity {
            0..=1 => {
                // Minor annoyances
                if roll < 0.3 {
                    Some(Self::HeadWind {
                        direction: Vec3::ZERO, // Calculated to oppose player
                        strength: 15.0,
                        follows_player: true,
                    })
                } else if roll < 0.6 {
                    Some(Self::TargetedRain {
                        duration: 300.0,
                        equipment_damage: 0.01,
                    })
                } else {
                    Some(Self::DarkCloud {
                        duration: 1800.0,
                        mood_penalty: 0.1,
                    })
                }
            },
            2..=3 => {
                // Moderate punishment
                if roll < 0.3 {
                    Some(Self::SupernaturalFog {
                        visibility: 0.3,
                        causes_hallucinations: false,
                    })
                } else if roll < 0.6 {
                    Some(Self::CloseCallLightning {
                        distance: 20.0,
                        warning_time: 1.5,
                    })
                } else {
                    Some(Self::SuddenFrost {
                        temperature_drop: 15.0,
                        crop_damage: 0.3,
                    })
                }
            },
            _ => {
                // Severe events
                if roll < 0.3 {
                    Some(Self::FlashFlood {
                        affected_area: AABB::around_player(50.0),
                        water_level: 1.5,
                        duration: 120.0,
                    })
                } else if roll < 0.6 {
                    Some(Self::SupernaturalFog {
                        visibility: 0.1,
                        causes_hallucinations: true,
                    })
                } else {
                    Some(Self::LocalizedDrought {
                        affected_radius: 200.0,
                        water_sources_dry: true,
                    })
                }
            },
        }
    }
}
```

---

## Storm & Hurricane System

### Enhanced Weather Types

```rust
// weather/storms.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    // Base weather
    Clear,
    PartlyCloudy,
    Overcast,
    LightRain,
    Fog,

    // Storms
    Thunderstorm,
    SevereThunderstorm,
    Tornado,
    Hurricane,

    // Seasonal
    Snow,
    Blizzard,
    IceStorm,
    HeatWave,
    Drought,

    // Supernatural
    BloodMoon,
    SpiritStorm,
    UnnatualCalm,
}

/// Full storm data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storm {
    pub storm_type: StormType,
    pub severity: StormSeverity,

    // Position and movement
    pub center: Vec3,
    pub radius: f32,
    pub heading: Vec3,
    pub speed: f32,

    // Intensity
    pub wind_speed: f32,
    pub rainfall_rate: f32,
    pub lightning_rate: f32,

    // Timing
    pub duration: f32,
    pub elapsed: f32,
    pub buildup_time: f32,
    pub dissipate_time: f32,

    // Effects
    pub effects: StormEffects,

    // Karma source
    pub is_supernatural: bool,
    pub karma_source: bool,
    pub can_be_appeased: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StormType {
    Thunderstorm,
    Hurricane,
    Tornado,
    Blizzard,
    SpiritStorm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormEffects {
    // Visual
    pub sky_darkness: f32,
    pub cloud_height: f32,
    pub rain_density: f32,
    pub lightning_color: Vec3,

    // Audio
    pub thunder_frequency: f32,
    pub wind_volume: f32,
    pub rain_volume: f32,

    // Gameplay
    pub visibility_multiplier: f32,
    pub movement_multiplier: f32,
    pub projectile_accuracy: f32,
    pub fire_suppression: bool,
    pub outdoor_damage: f32,
}
```

### Hurricane System

```rust
/// Full hurricane simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hurricane {
    pub name: String,  // Named hurricanes for atmosphere
    pub category: HurricaneCategory,

    // Position
    pub eye_position: Vec3,
    pub eye_radius: f32,
    pub outer_radius: f32,

    // Movement
    pub track: Vec<Vec3>,  // Predicted path
    pub current_speed: f32,
    pub heading: f32,

    // Intensity
    pub sustained_winds: f32,
    pub peak_gusts: f32,
    pub storm_surge: f32,
    pub rainfall_inches_per_hour: f32,

    // State
    pub phase: HurricanePhase,
    pub time_until_landfall: Option<f32>,
    pub is_karma_hurricane: bool,

    // Damage tracking
    pub trees_destroyed: u32,
    pub structures_damaged: u32,
    pub flooding_areas: Vec<FloodZone>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HurricaneCategory {
    TropicalDepression,  // < 39 mph
    TropicalStorm,       // 39-73 mph
    Category1,           // 74-95 mph
    Category2,           // 96-110 mph
    Category3,           // 111-129 mph (Major)
    Category4,           // 130-156 mph (Major)
    Category5,           // 157+ mph (Major)
}

impl HurricaneCategory {
    pub fn from_wind_speed(mph: f32) -> Self {
        match mph {
            w if w < 39.0 => Self::TropicalDepression,
            w if w < 74.0 => Self::TropicalStorm,
            w if w < 96.0 => Self::Category1,
            w if w < 111.0 => Self::Category2,
            w if w < 130.0 => Self::Category3,
            w if w < 157.0 => Self::Category4,
            _ => Self::Category5,
        }
    }

    pub fn storm_surge(&self) -> f32 {
        match self {
            Self::TropicalDepression => 0.0,
            Self::TropicalStorm => 1.0,
            Self::Category1 => 4.0,
            Self::Category2 => 6.0,
            Self::Category3 => 9.0,
            Self::Category4 => 13.0,
            Self::Category5 => 18.0,
        }
    }

    pub fn damage_description(&self) -> &'static str {
        match self {
            Self::TropicalDepression => "Minimal",
            Self::TropicalStorm => "Minor flooding",
            Self::Category1 => "Some damage to structures",
            Self::Category2 => "Extensive damage",
            Self::Category3 => "Devastating damage",
            Self::Category4 => "Catastrophic damage",
            Self::Category5 => "Total destruction",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HurricanePhase {
    Forming,         // Organizing offshore
    Approaching,     // Moving toward land
    Landfall,        // Hitting shore
    Eyewall,         // Worst part overhead
    Eye,             // Calm center
    BackEyewall,     // Second eyewall passage
    Weakening,       // Moving inland, losing strength
    Dissipating,     // Breaking apart
}

impl Hurricane {
    pub fn update(&mut self, dt: f32, terrain: &Terrain) {
        // Move hurricane
        let movement = Vec3::new(
            self.heading.cos() * self.current_speed * dt,
            0.0,
            self.heading.sin() * self.current_speed * dt,
        );
        self.eye_position += movement;

        // Update phase
        self.update_phase(terrain);

        // Apply effects
        self.apply_environmental_damage(terrain, dt);
    }

    fn update_phase(&mut self, terrain: &Terrain) {
        let distance_to_land = terrain.distance_to_shore(self.eye_position);

        self.phase = match distance_to_land {
            d if d > 100.0 => HurricanePhase::Approaching,
            d if d > 0.0 => {
                self.time_until_landfall = Some(d / self.current_speed);
                HurricanePhase::Approaching
            },
            d if d > -self.eye_radius => HurricanePhase::Landfall,
            _ => {
                // Over land, weakening
                self.sustained_winds *= 0.99;  // Lose strength over land
                if self.sustained_winds < 39.0 {
                    HurricanePhase::Dissipating
                } else {
                    HurricanePhase::Weakening
                }
            }
        };
    }

    pub fn get_wind_at_position(&self, pos: Vec3) -> Vec3 {
        let to_eye = self.eye_position - pos;
        let distance = to_eye.length();

        if distance < self.eye_radius {
            // In the eye - calm
            return Vec3::ZERO;
        }

        if distance > self.outer_radius {
            // Outside storm
            return Vec3::ZERO;
        }

        // Cyclonic winds (counterclockwise in Northern Hemisphere)
        let tangent = Vec3::new(-to_eye.z, 0.0, to_eye.x).normalize();

        // Inward spiral component
        let inward = to_eye.normalize() * 0.2;

        // Wind speed peaks at eyewall
        let eyewall_distance = self.eye_radius * 1.5;
        let speed_factor = if distance < eyewall_distance {
            distance / eyewall_distance
        } else {
            1.0 - (distance - eyewall_distance) / (self.outer_radius - eyewall_distance)
        };

        (tangent + inward) * self.sustained_winds * speed_factor
    }

    fn apply_environmental_damage(&mut self, terrain: &mut Terrain, dt: f32) {
        // Tree damage in high wind zones
        let damage_radius = self.eye_radius * 3.0;
        // Trees within damage radius have chance to be destroyed

        // Flooding in low areas
        if self.phase == HurricanePhase::Landfall || self.phase == HurricanePhase::Eyewall {
            // Storm surge flooding near coast
        }

        // Structure damage
        // Buildings take damage based on wind speed and construction
    }
}
```

### Warning Signs Before Storms

```rust
/// Omens that precede karma storms
#[derive(Debug, Clone)]
pub enum StormOmen {
    /// Animals behave strangely
    AnimalUnrest {
        affected_species: Vec<AnimalSpecies>,
        behavior: AnimalBehavior,
    },

    /// Sky turns unnatural colors
    SkyColor {
        color: Vec3,
        description: String,
    },

    /// Birds fly in unusual patterns
    BirdMurmuration {
        direction: Vec3,
        density: f32,
    },

    /// Insects swarm
    InsectSwarm {
        species: InsectType,
        intensity: f32,
    },

    /// Water behaves strangely
    WaterDisturbance {
        disturbance_type: WaterDisturbance,
    },

    /// Plants react
    PlantReaction {
        reaction: PlantReaction,
    },

    /// Supernatural signs
    SpiritSign {
        sign_type: SpiritSign,
        visibility: f32,
    },
}

impl StormOmen {
    pub fn get_warning_message(&self) -> &'static str {
        match self {
            Self::AnimalUnrest { .. } => "The animals are restless. Something approaches.",
            Self::SkyColor { .. } => "The sky turns an unnatural hue...",
            Self::BirdMurmuration { .. } => "Birds wheel and cry in great numbers, fleeing to the east.",
            Self::InsectSwarm { .. } => "Insects swarm in agitation. Nature is disturbed.",
            Self::WaterDisturbance { .. } => "The water churns without wind. The spirits are angry.",
            Self::PlantReaction { .. } => "The plants fold inward, as if cowering from what comes.",
            Self::SpiritSign { .. } => "A chill runs through you. The spirits demand attention.",
        }
    }

    pub fn time_until_storm(&self) -> f32 {
        match self {
            Self::AnimalUnrest { .. } => 3600.0,      // 1 hour
            Self::SkyColor { .. } => 1800.0,          // 30 minutes
            Self::BirdMurmuration { .. } => 2700.0,   // 45 minutes
            Self::InsectSwarm { .. } => 1200.0,       // 20 minutes
            Self::WaterDisturbance { .. } => 900.0,   // 15 minutes
            Self::PlantReaction { .. } => 600.0,      // 10 minutes
            Self::SpiritSign { .. } => 300.0,         // 5 minutes
        }
    }
}
```

---

## Supernatural Events

### Spirit Manifestations

At extreme negative karma, supernatural entities appear:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpiritManifestation {
    /// Ghostly animal that cannot be killed
    GhostAnimal {
        species: AnimalSpecies,
        represents: GhostIntent,
        behavior: GhostBehavior,
    },

    /// The spirit of a killed animal haunts player
    VengefulSpirit {
        original_kill: AnimalSpecies,
        kill_time: f64,
        anger_level: f32,
        effects: Vec<HauntingEffect>,
    },

    /// Nature spirit materializes
    NatureSpirit {
        spirit_type: NatureSpiritType,
        disposition: SpiritDisposition,
        message: Option<String>,
    },

    /// Ancestral spirit appears with warning
    AncestralWarning {
        warning_type: WarningType,
        urgency: f32,
    },

    /// The forest itself seems hostile
    LivingForest {
        effects: Vec<ForestHostilityEffect>,
        duration: f32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum GhostIntent {
    Warning,        // Trying to warn player
    Accusation,     // Reminding of transgression
    Vengeance,      // Actively hostile
    Guidance,       // Leading somewhere
}

#[derive(Debug, Clone, Copy)]
pub enum GhostBehavior {
    Follows,            // Follows player silently
    Blocks,             // Appears in player's path
    Attacks,            // Deals supernatural damage
    LeadsTo(Vec3),      // Guides to location
    Vanishes,           // Appears and disappears
}

#[derive(Debug, Clone)]
pub enum HauntingEffect {
    Whispers,           // Hear sounds of dying animal
    Visions,            // Flash images of the kill
    NightTerrors,       // Disturbed sleep
    AnimalHostility,    // Animals of that species are aggressive
    BadLuck,            // Minor mishaps
    HealthDrain,        // Slow health loss
}

#[derive(Debug, Clone, Copy)]
pub enum NatureSpiritType {
    ForestSpirit,
    WaterSpirit,
    StormSpirit,
    AnimalGuardian(AnimalSpecies),
    PlantSpirit(FloraSpecies),
    AncientOne,
}

#[derive(Debug, Clone)]
pub enum ForestHostilityEffect {
    TreesBlock,         // Trees seem to move to block path
    VinesGrab,          // Vines slow movement
    ThornsDamage,       // Take damage from plants
    LostEffect,         // Navigation becomes difficult
    DarkAmbient,        // Lighting dims unnaturally
    CreepingShadows,    // Things move in peripheral vision
}
```

### Curse Mechanics

```rust
/// Curses applied for severe transgressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curse {
    pub curse_type: CurseType,
    pub source: CurseSource,
    pub severity: f32,
    pub duration: Option<f32>,      // None = until lifted
    pub effects: Vec<CurseEffect>,
    pub lift_conditions: Vec<LiftCondition>,
    pub time_active: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum CurseType {
    HuntersCurse,       // Animals always know where you are
    WastrelsCurse,      // Food spoils quickly
    DestroyersCurse,    // Plants wither around you
    BloodCurse,         // Wounds heal slowly
    StormFollower,      // Bad weather follows you
    LonelyCurse,        // NPCs distrust you
    SpiritMarked,       // Supernatural entities target you
}

#[derive(Debug, Clone)]
pub enum CurseEffect {
    StatModifier { stat: Stat, modifier: f32 },
    SkillPenalty { skill: SkillType, penalty: f32 },
    ResourceDecay { resource: ResourceType, rate: f32 },
    AnimalBehavior { behavior: AnimalHostility },
    WeatherAttraction { weather_type: WeatherType },
    SpiritVisibility { visibility: f32 },
    NPCReputation { faction: Faction, penalty: f32 },
}

#[derive(Debug, Clone)]
pub enum LiftCondition {
    TimeElapsed(f32),
    KarmaRestored(f32),
    Offering(OfferingType),
    Ceremony(CeremonyType),
    PilgrimageTo(Vec3),
    SaveAnimals(u32),
    PlantTrees(u32),
    FastForDays(u32),
}
```

---

## Spirit Animals & Omens

### Omen System

```rust
/// Omens the player can observe and interpret
#[derive(Debug, Clone)]
pub struct Omen {
    pub omen_type: OmenType,
    pub interpretation: OmenMeaning,
    pub observed_at: f64,
    pub location: Vec3,
    pub player_noticed: bool,
    pub correctly_interpreted: bool,
}

#[derive(Debug, Clone)]
pub enum OmenType {
    // Animal omens
    DeadAnimalCrossing,
    UnusualAnimalBehavior(AnimalSpecies),
    AlbinoSighting,
    PredatorYieldsToPlayer,
    AnimalsStaring,

    // Bird omens
    CirclingVultures,
    OwlDaytime,
    RavenCalls,
    BirdsMurmuration,
    EagleSighting,

    // Weather omens
    RainbowAppears,
    RingAroundMoon,
    RedSunset,
    GreenFlash,
    StrangeCloud,

    // Nature omens
    FlowerBloomingWrongSeason,
    TreeFallsNearby,
    WaterRunsBackward,
    UnseasonalBlooming,

    // Supernatural
    ColdSpot,
    WhisperOnWind,
    ShadowWithNoSource,
    LightOrb,
    FaceInNature,
}

#[derive(Debug, Clone)]
pub enum OmenMeaning {
    GoodFortune { duration: f32 },
    BadFortune { duration: f32 },
    WarningOfDanger { danger_type: DangerType, time_until: f32 },
    SpiritApproval,
    SpiritDisapproval,
    DeathOmen,
    ChangesComing,
    SacredPresence,
    GuidanceOffered { direction: Vec3 },
}

impl Omen {
    pub fn generate_for_karma(karma: f32, location: Vec3) -> Option<Self> {
        let omen_chance = if karma < -25.0 {
            0.01 * (-karma / 25.0)  // More omens at worse karma
        } else if karma > 25.0 {
            0.005 * (karma / 25.0)  // Fewer positive omens
        } else {
            0.002  // Rare at neutral karma
        };

        if rand::random::<f32>() > omen_chance {
            return None;
        }

        let omen_type = if karma < -50.0 {
            // Bad omens
            match rand::random::<u32>() % 10 {
                0 => OmenType::DeadAnimalCrossing,
                1 => OmenType::CirclingVultures,
                2 => OmenType::OwlDaytime,
                3 => OmenType::RavenCalls,
                4 => OmenType::ColdSpot,
                5 => OmenType::WhisperOnWind,
                6 => OmenType::ShadowWithNoSource,
                7 => OmenType::TreeFallsNearby,
                8 => OmenType::AnimalsStaring,
                _ => OmenType::RedSunset,
            }
        } else if karma > 50.0 {
            // Good omens
            match rand::random::<u32>() % 8 {
                0 => OmenType::RainbowAppears,
                1 => OmenType::EagleSighting,
                2 => OmenType::AlbinoSighting,
                3 => OmenType::LightOrb,
                4 => OmenType::PredatorYieldsToPlayer,
                5 => OmenType::FlowerBloomingWrongSeason,
                6 => OmenType::GreenFlash,
                _ => OmenType::UnusualAnimalBehavior(AnimalSpecies::WhiteTailedDeer),
            }
        } else {
            // Neutral omens
            match rand::random::<u32>() % 5 {
                0 => OmenType::StrangeCloud,
                1 => OmenType::RingAroundMoon,
                2 => OmenType::BirdsMurmuration,
                _ => OmenType::UnseasonalBlooming,
            }
        };

        let interpretation = Self::interpret(&omen_type, karma);

        Some(Self {
            omen_type,
            interpretation,
            observed_at: get_game_time(),
            location,
            player_noticed: false,
            correctly_interpreted: false,
        })
    }

    fn interpret(omen_type: &OmenType, karma: f32) -> OmenMeaning {
        match omen_type {
            OmenType::CirclingVultures => OmenMeaning::WarningOfDanger {
                danger_type: DangerType::Death,
                time_until: 3600.0,
            },
            OmenType::RainbowAppears => OmenMeaning::GoodFortune { duration: 7200.0 },
            OmenType::RavenCalls => OmenMeaning::ChangesComing,
            OmenType::EagleSighting => OmenMeaning::SpiritApproval,
            OmenType::OwlDaytime => OmenMeaning::DeathOmen,
            OmenType::ShadowWithNoSource => OmenMeaning::SpiritDisapproval,
            OmenType::LightOrb => OmenMeaning::SacredPresence,
            OmenType::AlbinoSighting => OmenMeaning::GoodFortune { duration: 86400.0 },
            _ => OmenMeaning::ChangesComing,
        }
    }
}
```

### Spirit Animal Companions

At high positive karma, spirit animals may bond with player:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiritCompanion {
    pub species: AnimalSpecies,
    pub name: Option<String>,       // Revealed over time
    pub bond_strength: f32,         // 0.0 - 1.0
    pub visibility: SpiritVisibility,
    pub abilities: Vec<SpiritAbility>,
    pub personality: SpiritPersonality,
    pub granted_blessings: Vec<SpiritBlessing>,
}

#[derive(Debug, Clone, Copy)]
pub enum SpiritVisibility {
    Invisible,          // Only sense their presence
    Glimpses,           // Occasional appearances
    PartiallyVisible,   // Translucent form
    FullyVisible,       // Can be seen clearly
    Manifest,           // Can interact physically
}

#[derive(Debug, Clone)]
pub enum SpiritAbility {
    // Guidance
    Tracking { range: f32 },                    // Highlights prey
    DangerSense { radius: f32 },                // Warns of threats
    PathFinding { to: PathDestination },        // Shows way
    WeatherPrediction { hours_ahead: f32 },     // Forecasts weather

    // Protection
    WardAgainst(AnimalSpecies),                 // Animals avoid
    HealthRegeneration { rate: f32 },           // Slow healing
    PoisonResistance { resistance: f32 },       // Poison protection
    StormShelter,                               // Reduced storm damage

    // Enhancement
    StealthBonus { bonus: f32 },                // Harder to detect
    SpeedBonus { bonus: f32 },                  // Faster movement
    StaminaBonus { bonus: f32 },                // More endurance
    NightVision,                                // See in dark

    // Supernatural
    SpiritCommunication,                        // Understand spirit messages
    CrossWorlds,                                // Access spirit locations
    Prophecy,                                   // Glimpse future events
}

#[derive(Debug, Clone)]
pub enum SpiritBlessing {
    LuckInHunt,         // Better hunting outcomes
    SafePassage,        // Reduced random encounters
    NatureAffinity,     // Faster karma recovery
    ElementalShield,    // Weather resistance
    AnimalKinship,      // Animals less hostile
    SpiritSight,        // See supernatural entities
}
```

---

## Redemption Mechanics

### Restoring Balance

```rust
/// Ways to restore positive karma
#[derive(Debug, Clone)]
pub enum RedemptionAction {
    // Immediate actions
    Offering {
        offering_type: OfferingType,
        location: OfferingLocation,
        karma_value: f32,
    },

    Prayer {
        duration: f32,
        at_sacred_site: bool,
        karma_value: f32,
    },

    // Sustained actions
    FastFromMeat {
        days: u32,
        karma_per_day: f32,
    },

    ProtectWildlife {
        animals_saved: u32,
        karma_per_save: f32,
    },

    PlantRestoration {
        plants_planted: u32,
        karma_per_plant: f32,
    },

    // Quest-like redemption
    Pilgrimage {
        destination: Vec3,
        trials: Vec<PilgrimageTrial>,
        karma_reward: f32,
    },

    Ceremony {
        ceremony_type: CeremonyType,
        requirements: Vec<CeremonyRequirement>,
        karma_reward: f32,
    },

    SpiritQuest {
        quest_giver: NatureSpiritType,
        objectives: Vec<QuestObjective>,
        karma_reward: f32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum OfferingType {
    Tobacco,            // +2 karma
    Corn,               // +1 karma
    AnimalPart,         // Variable
    CraftedItem,        // Variable
    RareHerb,           // +3 karma
    PreciousStone,      // +4 karma
    FirstKill,          // +5 karma (first of hunt)
}

#[derive(Debug, Clone, Copy)]
pub enum OfferingLocation {
    SacredSite,         // 2x karma
    WaterSource,        // 1.5x karma
    AncientTree,        // 1.5x karma
    AnimalDen,          // 1.5x for that species
    HighPlace,          // 1.25x karma
    Anywhere,           // 1x karma
}

#[derive(Debug, Clone)]
pub enum CeremonyType {
    Purification {
        requires: Vec<PurificationStep>,
        duration_hours: f32,
    },

    Appeasement {
        target_spirit: NatureSpiritType,
        offerings_required: Vec<OfferingType>,
    },

    RenewalOfBalance {
        participants: u32,  // NPCs needed
        location: Vec3,
    },

    SpiritMeeting {
        invoked_spirit: NatureSpiritType,
        ritual_items: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum PilgrimageTrial {
    FastUntilArrival,
    NoKilling,
    NoHarvesting,
    TravelByNight,
    MakeOfferingsAlong,
    SurviveStorm,
    FaceFear,
}

impl NatureBalance {
    pub fn process_redemption(&mut self, action: &RedemptionAction) {
        let karma_gained = match action {
            RedemptionAction::Offering { offering_type, location, karma_value } => {
                let location_mult = location.multiplier();
                karma_value * location_mult
            },

            RedemptionAction::Prayer { duration, at_sacred_site, karma_value } => {
                let site_mult = if *at_sacred_site { 2.0 } else { 1.0 };
                let duration_mult = (duration / 300.0).min(3.0);  // Up to 15 min
                karma_value * site_mult * duration_mult
            },

            RedemptionAction::FastFromMeat { days, karma_per_day } => {
                *days as f32 * karma_per_day
            },

            RedemptionAction::Pilgrimage { karma_reward, .. } => *karma_reward,
            RedemptionAction::Ceremony { karma_reward, .. } => *karma_reward,
            RedemptionAction::SpiritQuest { karma_reward, .. } => *karma_reward,

            _ => 0.0,
        };

        self.value = (self.value + karma_gained).min(100.0);
        self.positive_karma += karma_gained;

        // Check for curse lifting
        self.check_curse_conditions();

        // May trigger positive event
        if self.value > 0.0 && self.was_negative() {
            self.on_balance_restored();
        }
    }
}
```

### Natural Decay/Recovery

```rust
impl NatureBalance {
    pub fn update(&mut self, dt: f32) {
        // Karma slowly drifts toward zero (natural forgetting)
        let decay_rate = 0.001;  // Per second

        if self.value < 0.0 {
            self.value = (self.value + decay_rate * dt).min(0.0);
        } else if self.value > 0.0 {
            self.value = (self.value - decay_rate * dt * 0.5).max(0.0);  // Slower positive decay
        }

        // Spirit attention fades over time
        self.spirit_attention = (self.spirit_attention - 0.0001 * dt).max(0.0);

        // Healing timer for major transgressions
        if self.value < -50.0 {
            self.healing_timer = 0.0;  // Reset on severe negative
        } else {
            self.healing_timer += dt;
        }

        // Process pending events
        self.process_pending_events(dt);

        // Update active effects
        self.update_effects(dt);
    }
}
```

---

## Integration with Other Systems

### Hunting Integration

```rust
/// Apply karma to hunting actions
pub fn on_animal_killed(
    balance: &mut NatureBalance,
    species: AnimalSpecies,
    kill_context: &KillContext,
    encyclopedia: &Encyclopedia,
) {
    let mut modifiers = vec![];

    // Knowledge modifier
    let knowledge = encyclopedia.get_entry_tier(species);
    modifiers.push(KarmaModifier::KnewBetter(match knowledge {
        DiscoveryTier::Unknown => 0.5,
        DiscoveryTier::Sighted => 0.75,
        DiscoveryTier::Observed => 1.0,
        DiscoveryTier::Studied => 1.25,
        DiscoveryTier::Mastered => 1.5,
    }));

    // Context modifiers
    if kill_context.player_starving {
        modifiers.push(KarmaModifier::Starving(0.5));
    }

    if kill_context.was_attacked_first {
        modifiers.push(KarmaModifier::SelfDefense(0.3));
    }

    if kill_context.clean_kill {
        modifiers.push(KarmaModifier::CleanKill(0.8));
    }

    // Species modifiers
    if is_sacred_species(species) {
        modifiers.push(KarmaModifier::SacredSpecies(2.0));
    }

    if kill_context.near_sacred_site {
        modifiers.push(KarmaModifier::SacredGround(1.5));
    }

    // Apply
    let action = if is_docile(species) {
        ActionType::KillDocileAnimal
    } else {
        ActionType::KillAnimal
    };

    balance.apply_action(action, &modifiers);
}

/// Apply karma to harvesting
pub fn on_plant_harvested(
    balance: &mut NatureBalance,
    species: FloraSpecies,
    harvest_context: &HarvestContext,
) {
    let mut modifiers = vec![];

    // Check if over-harvesting
    if harvest_context.depleted_area {
        modifiers.push(KarmaModifier::OverHarvest(1.5));
    }

    // Check if left some behind
    if harvest_context.left_some {
        balance.apply_action(ActionType::HarvestResponsibly, &modifiers);
        return;
    }

    // Rare plant penalty
    if is_rare_plant(species) {
        modifiers.push(KarmaModifier::RareSpecies(1.5));
    }

    balance.apply_action(ActionType::OverharvestPlant, &modifiers);
}
```

### Weather Integration

```rust
/// Weather system responds to karma
impl WeatherSystem {
    pub fn update_with_karma(&mut self, balance: &NatureBalance, dt: f32) {
        // Karma influences weather probability
        let karma = balance.value;

        if karma < -25.0 {
            // Bias toward bad weather
            self.storm_probability += (-karma / 100.0) * 0.01 * dt;
            self.clear_probability *= 0.99;
        } else if karma > 25.0 {
            // Bias toward good weather
            self.storm_probability *= 0.99;
            self.clear_probability += (karma / 100.0) * 0.01 * dt;
        }

        // Check for karma-triggered weather
        if let Some(event) = BadLuckWeatherEvent::choose_for_karma(karma) {
            self.queue_event(event);
        }

        // Karma storms
        if balance.tier <= BalanceTier::Exploiter {
            if rand::random::<f32>() < 0.001 * (-karma / 25.0) * dt {
                self.trigger_karma_storm(balance.tier);
            }
        }
    }

    fn trigger_karma_storm(&mut self, tier: BalanceTier) {
        let severity = match tier {
            BalanceTier::Destroyer => StormSeverity::Catastrophic,
            BalanceTier::Despoiler => StormSeverity::Severe,
            BalanceTier::Exploiter => StormSeverity::Moderate,
            _ => StormSeverity::Minor,
        };

        let storm = Storm {
            storm_type: StormType::SpiritStorm,
            severity,
            is_supernatural: true,
            karma_source: true,
            can_be_appeased: true,
            ..Storm::new()
        };

        self.summon_storm(storm);
    }
}
```

---

## Data Structures

### Main System

```rust
// karma/mod.rs

pub struct KarmaSystem {
    pub balance: NatureBalance,
    pub ledger: KarmaLedger,
    pub active_curses: Vec<Curse>,
    pub active_blessings: Vec<Blessing>,
    pub spirit_companion: Option<SpiritCompanion>,
    pub observed_omens: Vec<Omen>,
    pub active_manifestations: Vec<SpiritManifestation>,
    pub redemption_progress: HashMap<String, RedemptionProgress>,
}

impl KarmaSystem {
    pub fn new() -> Self {
        Self {
            balance: NatureBalance::default(),
            ledger: KarmaLedger::default(),
            active_curses: vec![],
            active_blessings: vec![],
            spirit_companion: None,
            observed_omens: vec![],
            active_manifestations: vec![],
            redemption_progress: HashMap::new(),
        }
    }

    pub fn update(&mut self, dt: f32, weather: &mut WeatherSystem) {
        // Update balance
        self.balance.update(dt);

        // Update curses/blessings
        self.update_effects(dt);

        // Process omens
        self.process_omens(dt);

        // Update manifestations
        self.update_manifestations(dt);

        // Weather integration
        weather.update_with_karma(&self.balance, dt);

        // Spirit companion
        if let Some(companion) = &mut self.spirit_companion {
            companion.update(dt, &self.balance);
        }
    }

    pub fn on_action(&mut self, action: ActionType, context: &ActionContext) {
        // Record in ledger
        self.ledger.record(action, context);

        // Apply to balance
        let modifiers = context.get_modifiers();
        self.balance.apply_action(action, &modifiers);

        // Check for new effects
        self.check_curse_triggers();
        self.check_blessing_triggers();
    }

    pub fn get_active_effects(&self) -> Vec<ActiveEffect> {
        let mut effects = vec![];

        for curse in &self.active_curses {
            effects.extend(curse.effects.iter().cloned().map(ActiveEffect::Curse));
        }

        for blessing in &self.active_blessings {
            effects.extend(blessing.effects.iter().cloned().map(ActiveEffect::Blessing));
        }

        if let Some(companion) = &self.spirit_companion {
            for blessing in &companion.granted_blessings {
                effects.push(ActiveEffect::SpiritBlessing(blessing.clone()));
            }
        }

        effects
    }
}
```

### Save/Load

```rust
#[derive(Serialize, Deserialize)]
pub struct KarmaSaveData {
    pub balance: NatureBalance,
    pub ledger: KarmaLedger,
    pub curses: Vec<Curse>,
    pub blessings: Vec<Blessing>,
    pub companion: Option<SpiritCompanion>,
    pub omens_seen: u32,
    pub manifestations_witnessed: u32,
    pub redemption_acts: u32,
}
```

---

## Implementation Priority

### Phase 1: Core Balance
- [ ] NatureBalance struct
- [ ] Karma action values
- [ ] Tier system
- [ ] Basic modifiers

### Phase 2: Weather Link
- [ ] Karma-influenced weather probability
- [ ] Storm severity based on karma
- [ ] Bad luck weather events

### Phase 3: Consequences
- [ ] Warning omens
- [ ] Curse application
- [ ] Blessing application

### Phase 4: Supernatural
- [ ] Spirit manifestations
- [ ] Ghost animals
- [ ] Haunting effects

### Phase 5: Redemption
- [ ] Offering system
- [ ] Prayer mechanics
- [ ] Ceremony system

### Phase 6: Spirit Companion
- [ ] Companion bonding
- [ ] Spirit abilities
- [ ] Visibility progression

### Phase 7: Integration
- [ ] Hunting karma
- [ ] Harvesting karma
- [ ] Encyclopedia knowledge links
- [ ] NPC reaction to karma

---

*The spirits of Roanoke watch all who walk these lands. Treat the forest with respect, and it shall be your ally. Despoil it, and face the wrath of nature itself.*
