//! Horse Training System
//!
//! Skill development and specialization for tamed horses.
//! Training improves horse abilities and unlocks new capabilities.

use super::entity::Horse;
use super::types::HorseSpecies;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Training skills that can be developed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingSkill {
    // === Movement Skills ===
    /// Base speed improvement
    Speed,
    /// Maximum stamina
    Endurance,
    /// Acceleration and responsiveness
    Agility,
    /// Swimming ability
    Swimming,
    /// Jumping obstacles
    Jumping,
    /// Terrain navigation
    SureFooted,

    // === Work Skills ===
    /// Pulling power for plowing/wagons
    Strength,
    /// Carrying capacity
    PackHorse,
    /// Working in harness
    Harness,
    /// Field work efficiency
    Plowing,

    // === Combat Skills ===
    /// Bravery in battle
    WarHorse,
    /// Trampling attacks
    Charging,
    /// Kicks and rearing
    Defensive,
    /// Tolerance of weapons/armor
    ArmorTraining,

    // === Obedience Skills ===
    /// Response to voice commands
    VoiceCommand,
    /// Response to leg/rein cues
    RidingCues,
    /// Ground manners
    GroundManners,
    /// Standing still when needed
    Patience,
    /// Recall/come when called
    Recall,

    // === Specialty Skills ===
    /// Herding livestock
    Herding,
    /// Long distance travel
    LongRide,
    /// Navigating difficult terrain
    MountainTrail,
    /// Marsh/swamp navigation
    WetlandTrail,
    /// Beach/coastal travel
    CoastalTrail,
    /// Racing performance
    Racing,
}

impl TrainingSkill {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Speed => "Speed",
            Self::Endurance => "Endurance",
            Self::Agility => "Agility",
            Self::Swimming => "Swimming",
            Self::Jumping => "Jumping",
            Self::SureFooted => "Sure-Footed",
            Self::Strength => "Strength",
            Self::PackHorse => "Pack Horse",
            Self::Harness => "Harness Training",
            Self::Plowing => "Plowing",
            Self::WarHorse => "War Horse",
            Self::Charging => "Charging",
            Self::Defensive => "Defensive Combat",
            Self::ArmorTraining => "Armor Training",
            Self::VoiceCommand => "Voice Commands",
            Self::RidingCues => "Riding Cues",
            Self::GroundManners => "Ground Manners",
            Self::Patience => "Patience",
            Self::Recall => "Recall",
            Self::Herding => "Herding",
            Self::LongRide => "Long Ride",
            Self::MountainTrail => "Mountain Trail",
            Self::WetlandTrail => "Wetland Trail",
            Self::CoastalTrail => "Coastal Trail",
            Self::Racing => "Racing",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Speed => "Improves maximum gallop speed",
            Self::Endurance => "Increases stamina and recovery rate",
            Self::Agility => "Better turning and acceleration",
            Self::Swimming => "Faster and more confident in water",
            Self::Jumping => "Higher and farther jumps",
            Self::SureFooted => "Less stumbling on rough terrain",
            Self::Strength => "More pulling power for work",
            Self::PackHorse => "Increased carrying capacity",
            Self::Harness => "Works well in harness with wagons",
            Self::Plowing => "Efficient at field work",
            Self::WarHorse => "Stays calm in combat situations",
            Self::Charging => "Powerful charge attacks",
            Self::Defensive => "Effective kicks and rearing",
            Self::ArmorTraining => "Comfortable wearing barding",
            Self::VoiceCommand => "Responds to voice commands",
            Self::RidingCues => "Subtle leg and rein response",
            Self::GroundManners => "Behaves well on lead",
            Self::Patience => "Stands quietly when needed",
            Self::Recall => "Comes when whistled for",
            Self::Herding => "Helps manage livestock",
            Self::LongRide => "Maintains pace over long distances",
            Self::MountainTrail => "Navigates mountain paths safely",
            Self::WetlandTrail => "Moves through swamps easily",
            Self::CoastalTrail => "Comfortable on beaches and dunes",
            Self::Racing => "Optimized for speed competitions",
        }
    }

    /// Get the category of this skill
    pub fn category(&self) -> SkillCategory {
        match self {
            Self::Speed | Self::Endurance | Self::Agility |
            Self::Swimming | Self::Jumping | Self::SureFooted
                => SkillCategory::Movement,

            Self::Strength | Self::PackHorse | Self::Harness | Self::Plowing
                => SkillCategory::Work,

            Self::WarHorse | Self::Charging | Self::Defensive | Self::ArmorTraining
                => SkillCategory::Combat,

            Self::VoiceCommand | Self::RidingCues | Self::GroundManners |
            Self::Patience | Self::Recall
                => SkillCategory::Obedience,

            Self::Herding | Self::LongRide | Self::MountainTrail |
            Self::WetlandTrail | Self::CoastalTrail | Self::Racing
                => SkillCategory::Specialty,
        }
    }

    /// Get experience required per level
    pub fn xp_per_level(&self) -> u32 {
        match self.category() {
            SkillCategory::Movement => 100,
            SkillCategory::Work => 120,
            SkillCategory::Combat => 150,
            SkillCategory::Obedience => 80,
            SkillCategory::Specialty => 200,
        }
    }

    /// Get natural aptitude bonus for a species
    pub fn species_aptitude(&self, species: HorseSpecies) -> f32 {
        match (self, species) {
            // Banker Horse - coastal specialist
            (Self::CoastalTrail, HorseSpecies::BankerHorse) => 1.5,
            (Self::Swimming, HorseSpecies::BankerHorse) => 1.3,
            (Self::Endurance, HorseSpecies::BankerHorse) => 1.2,

            // Carolina Marsh Tacky - wetland specialist
            (Self::WetlandTrail, HorseSpecies::CarolinaMarshTacky) => 1.5,
            (Self::Swimming, HorseSpecies::CarolinaMarshTacky) => 1.4,
            (Self::SureFooted, HorseSpecies::CarolinaMarshTacky) => 1.3,

            // Colonial Spanish - versatile
            (Self::RidingCues, HorseSpecies::ColonialSpanish) => 1.3,
            (Self::VoiceCommand, HorseSpecies::ColonialSpanish) => 1.3,
            (Self::Agility, HorseSpecies::ColonialSpanish) => 1.2,

            // Chincoteague Pony - island survival
            (Self::Swimming, HorseSpecies::ChincoteaguePony) => 1.5,
            (Self::CoastalTrail, HorseSpecies::ChincoteaguePony) => 1.3,
            (Self::SureFooted, HorseSpecies::ChincoteaguePony) => 1.2,

            // Virginia Draught - work horse
            (Self::Strength, HorseSpecies::VirginiaDraught) => 1.5,
            (Self::Plowing, HorseSpecies::VirginiaDraught) => 1.5,
            (Self::Harness, HorseSpecies::VirginiaDraught) => 1.4,
            (Self::PackHorse, HorseSpecies::VirginiaDraught) => 1.3,
            (Self::Patience, HorseSpecies::VirginiaDraught) => 1.3,

            // Chickasaw - speed
            (Self::Speed, HorseSpecies::Chickasaw) => 1.5,
            (Self::Racing, HorseSpecies::Chickasaw) => 1.5,
            (Self::Agility, HorseSpecies::Chickasaw) => 1.3,
            (Self::Endurance, HorseSpecies::Chickasaw) => 1.2,

            // Default
            _ => 1.0,
        }
    }

    /// Iterator over all skills
    pub fn all() -> impl Iterator<Item = TrainingSkill> {
        [
            Self::Speed, Self::Endurance, Self::Agility,
            Self::Swimming, Self::Jumping, Self::SureFooted,
            Self::Strength, Self::PackHorse, Self::Harness, Self::Plowing,
            Self::WarHorse, Self::Charging, Self::Defensive, Self::ArmorTraining,
            Self::VoiceCommand, Self::RidingCues, Self::GroundManners,
            Self::Patience, Self::Recall,
            Self::Herding, Self::LongRide, Self::MountainTrail,
            Self::WetlandTrail, Self::CoastalTrail, Self::Racing,
        ].into_iter()
    }
}

/// Categories of skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    Movement,
    Work,
    Combat,
    Obedience,
    Specialty,
}

impl SkillCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Movement => "Movement",
            Self::Work => "Work",
            Self::Combat => "Combat",
            Self::Obedience => "Obedience",
            Self::Specialty => "Specialty",
        }
    }

    pub fn skills(&self) -> Vec<TrainingSkill> {
        TrainingSkill::all()
            .filter(|s| s.category() == *self)
            .collect()
    }
}

/// Skill level (1-10)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SkillLevel(pub u8);

impl SkillLevel {
    pub const MIN: SkillLevel = SkillLevel(0);
    pub const MAX: SkillLevel = SkillLevel(10);

    pub fn new(level: u8) -> Self {
        Self(level.min(10))
    }

    pub fn level(&self) -> u8 {
        self.0
    }

    /// Get bonus multiplier for this level
    pub fn bonus(&self) -> f32 {
        self.0 as f32 * 0.05 // 5% per level, max 50%
    }

    /// Get display stars
    pub fn stars(&self) -> &'static str {
        match self.0 {
            0 => "",
            1 => "*",
            2 => "**",
            3 => "***",
            4 => "****",
            5 => "*****",
            6 => "******",
            7 => "*******",
            8 => "********",
            9 => "*********",
            10 => "**********",
            _ => "**********",
        }
    }

    /// Get tier name
    pub fn tier_name(&self) -> &'static str {
        match self.0 {
            0 => "Untrained",
            1..=2 => "Novice",
            3..=4 => "Trained",
            5..=6 => "Skilled",
            7..=8 => "Expert",
            9..=10 => "Master",
            _ => "Master",
        }
    }
}

/// Individual skill progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProgress {
    pub skill: TrainingSkill,
    pub level: SkillLevel,
    pub experience: u32,
    pub times_trained: u32,
}

impl SkillProgress {
    pub fn new(skill: TrainingSkill) -> Self {
        Self {
            skill,
            level: SkillLevel::MIN,
            experience: 0,
            times_trained: 0,
        }
    }

    /// Add experience and check for level up
    pub fn add_experience(&mut self, amount: u32, species_aptitude: f32) -> bool {
        let adjusted_amount = (amount as f32 * species_aptitude) as u32;
        self.experience += adjusted_amount;
        self.times_trained += 1;

        let xp_needed = self.skill.xp_per_level() * (self.level.0 as u32 + 1);
        if self.experience >= xp_needed && self.level.0 < 10 {
            self.level = SkillLevel::new(self.level.0 + 1);
            self.experience -= xp_needed;
            return true;
        }
        false
    }

    /// Get progress to next level (0.0-1.0)
    pub fn progress_to_next(&self) -> f32 {
        if self.level.0 >= 10 {
            return 1.0;
        }
        let xp_needed = self.skill.xp_per_level() * (self.level.0 as u32 + 1);
        self.experience as f32 / xp_needed as f32
    }
}

/// Collection of all training skills for a horse
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrainingSkills {
    skills: HashMap<TrainingSkill, SkillProgress>,
}

impl TrainingSkills {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Get skill level (None if never trained)
    pub fn get_skill(&self, skill: TrainingSkill) -> Option<SkillLevel> {
        self.skills.get(&skill).map(|p| p.level)
    }

    /// Get skill progress
    pub fn get_progress(&self, skill: TrainingSkill) -> Option<&SkillProgress> {
        self.skills.get(&skill)
    }

    /// Add experience to a skill
    pub fn train(&mut self, skill: TrainingSkill, xp: u32, species: HorseSpecies) -> bool {
        let aptitude = skill.species_aptitude(species);
        let progress = self.skills
            .entry(skill)
            .or_insert_with(|| SkillProgress::new(skill));
        progress.add_experience(xp, aptitude)
    }

    /// Get total skill points invested
    pub fn total_skill_points(&self) -> u32 {
        self.skills.values()
            .map(|p| p.level.0 as u32)
            .sum()
    }

    /// Get skills by category
    pub fn skills_in_category(&self, category: SkillCategory) -> Vec<&SkillProgress> {
        self.skills.values()
            .filter(|p| p.skill.category() == category)
            .collect()
    }

    /// Get highest level in a category
    pub fn highest_in_category(&self, category: SkillCategory) -> SkillLevel {
        self.skills.values()
            .filter(|p| p.skill.category() == category)
            .map(|p| p.level)
            .max()
            .unwrap_or(SkillLevel::MIN)
    }
}

/// Training exercises that develop skills
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingExercise {
    // Movement exercises
    SprintDrills,
    EnduranceRide,
    AgilityWeave,
    WaterCrossing,
    JumpPractice,
    RoughTerrainWalk,

    // Work exercises
    PullWeight,
    CarryPacks,
    HarnessWork,
    FieldWork,

    // Combat exercises
    BattleDesensitization,
    ChargePractice,
    DefenseTraining,
    ArmorFitting,

    // Obedience exercises
    VoiceDrills,
    CuePractice,
    LeadWork,
    StandingStill,
    RecallPractice,

    // Specialty exercises
    HerdingPractice,
    LongDistanceRide,
    MountainTraining,
    SwampTraining,
    BeachTraining,
    RacePractice,
}

impl TrainingExercise {
    /// Get skills developed by this exercise
    pub fn develops_skills(&self) -> &'static [(TrainingSkill, u32)] {
        match self {
            Self::SprintDrills => &[(TrainingSkill::Speed, 30), (TrainingSkill::Agility, 10)],
            Self::EnduranceRide => &[(TrainingSkill::Endurance, 25), (TrainingSkill::LongRide, 15)],
            Self::AgilityWeave => &[(TrainingSkill::Agility, 30), (TrainingSkill::RidingCues, 10)],
            Self::WaterCrossing => &[(TrainingSkill::Swimming, 35)],
            Self::JumpPractice => &[(TrainingSkill::Jumping, 30), (TrainingSkill::Agility, 10)],
            Self::RoughTerrainWalk => &[(TrainingSkill::SureFooted, 25), (TrainingSkill::Patience, 10)],

            Self::PullWeight => &[(TrainingSkill::Strength, 30), (TrainingSkill::Harness, 15)],
            Self::CarryPacks => &[(TrainingSkill::PackHorse, 35), (TrainingSkill::Endurance, 10)],
            Self::HarnessWork => &[(TrainingSkill::Harness, 30), (TrainingSkill::Patience, 15)],
            Self::FieldWork => &[(TrainingSkill::Plowing, 35), (TrainingSkill::Strength, 15)],

            Self::BattleDesensitization => &[(TrainingSkill::WarHorse, 30), (TrainingSkill::Patience, 10)],
            Self::ChargePractice => &[(TrainingSkill::Charging, 35), (TrainingSkill::Speed, 10)],
            Self::DefenseTraining => &[(TrainingSkill::Defensive, 30), (TrainingSkill::WarHorse, 10)],
            Self::ArmorFitting => &[(TrainingSkill::ArmorTraining, 35), (TrainingSkill::Patience, 10)],

            Self::VoiceDrills => &[(TrainingSkill::VoiceCommand, 35)],
            Self::CuePractice => &[(TrainingSkill::RidingCues, 35)],
            Self::LeadWork => &[(TrainingSkill::GroundManners, 30), (TrainingSkill::Patience, 10)],
            Self::StandingStill => &[(TrainingSkill::Patience, 35), (TrainingSkill::GroundManners, 10)],
            Self::RecallPractice => &[(TrainingSkill::Recall, 35), (TrainingSkill::VoiceCommand, 10)],

            Self::HerdingPractice => &[(TrainingSkill::Herding, 35), (TrainingSkill::Agility, 10)],
            Self::LongDistanceRide => &[(TrainingSkill::LongRide, 30), (TrainingSkill::Endurance, 20)],
            Self::MountainTraining => &[(TrainingSkill::MountainTrail, 35), (TrainingSkill::SureFooted, 15)],
            Self::SwampTraining => &[(TrainingSkill::WetlandTrail, 35), (TrainingSkill::Swimming, 10)],
            Self::BeachTraining => &[(TrainingSkill::CoastalTrail, 35), (TrainingSkill::SureFooted, 10)],
            Self::RacePractice => &[(TrainingSkill::Racing, 35), (TrainingSkill::Speed, 15)],
        }
    }

    /// Get time required for exercise (in game seconds)
    pub fn duration(&self) -> f32 {
        match self {
            Self::SprintDrills => 120.0,
            Self::EnduranceRide => 600.0,
            Self::AgilityWeave => 180.0,
            Self::WaterCrossing => 240.0,
            Self::JumpPractice => 200.0,
            Self::RoughTerrainWalk => 300.0,
            Self::PullWeight => 180.0,
            Self::CarryPacks => 300.0,
            Self::HarnessWork => 240.0,
            Self::FieldWork => 600.0,
            Self::BattleDesensitization => 300.0,
            Self::ChargePractice => 180.0,
            Self::DefenseTraining => 200.0,
            Self::ArmorFitting => 180.0,
            Self::VoiceDrills => 150.0,
            Self::CuePractice => 180.0,
            Self::LeadWork => 150.0,
            Self::StandingStill => 120.0,
            Self::RecallPractice => 180.0,
            Self::HerdingPractice => 400.0,
            Self::LongDistanceRide => 900.0,
            Self::MountainTraining => 500.0,
            Self::SwampTraining => 400.0,
            Self::BeachTraining => 400.0,
            Self::RacePractice => 200.0,
        }
    }

    /// Get stamina cost for exercise
    pub fn stamina_cost(&self) -> f32 {
        match self {
            Self::SprintDrills => 40.0,
            Self::EnduranceRide => 60.0,
            Self::AgilityWeave => 30.0,
            Self::WaterCrossing => 35.0,
            Self::JumpPractice => 35.0,
            Self::RoughTerrainWalk => 25.0,
            Self::PullWeight => 45.0,
            Self::CarryPacks => 40.0,
            Self::HarnessWork => 35.0,
            Self::FieldWork => 50.0,
            Self::BattleDesensitization => 20.0,
            Self::ChargePractice => 45.0,
            Self::DefenseTraining => 30.0,
            Self::ArmorFitting => 15.0,
            Self::VoiceDrills => 10.0,
            Self::CuePractice => 20.0,
            Self::LeadWork => 15.0,
            Self::StandingStill => 5.0,
            Self::RecallPractice => 25.0,
            Self::HerdingPractice => 50.0,
            Self::LongDistanceRide => 70.0,
            Self::MountainTraining => 55.0,
            Self::SwampTraining => 45.0,
            Self::BeachTraining => 40.0,
            Self::RacePractice => 50.0,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::SprintDrills => "Sprint Drills",
            Self::EnduranceRide => "Endurance Ride",
            Self::AgilityWeave => "Agility Weave",
            Self::WaterCrossing => "Water Crossing",
            Self::JumpPractice => "Jump Practice",
            Self::RoughTerrainWalk => "Rough Terrain Walk",
            Self::PullWeight => "Pull Weight",
            Self::CarryPacks => "Carry Packs",
            Self::HarnessWork => "Harness Work",
            Self::FieldWork => "Field Work",
            Self::BattleDesensitization => "Battle Desensitization",
            Self::ChargePractice => "Charge Practice",
            Self::DefenseTraining => "Defense Training",
            Self::ArmorFitting => "Armor Fitting",
            Self::VoiceDrills => "Voice Drills",
            Self::CuePractice => "Cue Practice",
            Self::LeadWork => "Lead Work",
            Self::StandingStill => "Standing Still",
            Self::RecallPractice => "Recall Practice",
            Self::HerdingPractice => "Herding Practice",
            Self::LongDistanceRide => "Long Distance Ride",
            Self::MountainTraining => "Mountain Training",
            Self::SwampTraining => "Swamp Training",
            Self::BeachTraining => "Beach Training",
            Self::RacePractice => "Race Practice",
        }
    }
}

/// Training session manager
#[derive(Debug)]
pub struct TrainingSystem {
    /// Current active training session
    pub active_session: Option<TrainingSession>,
}

impl Default for TrainingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TrainingSystem {
    pub fn new() -> Self {
        Self {
            active_session: None,
        }
    }

    /// Start a training session
    pub fn start_training(&mut self, horse: &mut Horse, exercise: TrainingExercise) -> Result<(), &'static str> {
        if self.active_session.is_some() {
            return Err("Already in a training session");
        }

        if horse.stamina < exercise.stamina_cost() {
            return Err("Horse doesn't have enough stamina");
        }

        if horse.encephalon.needs.rest < 0.2 {
            return Err("Horse is too tired to train");
        }

        self.active_session = Some(TrainingSession {
            exercise,
            progress: 0.0,
            duration: exercise.duration(),
            started: true,
        });

        Ok(())
    }

    /// Update training progress
    pub fn update(&mut self, horse: &mut Horse, dt: f32) -> Option<TrainingResult> {
        let session = self.active_session.as_mut()?;

        session.progress += dt;

        // Drain stamina during training
        let stamina_drain = session.exercise.stamina_cost() / session.duration * dt;
        horse.stamina = (horse.stamina - stamina_drain).max(0.0);

        if session.progress >= session.duration {
            // Training complete!
            let result = self.complete_training(horse);
            self.active_session = None;
            return Some(result);
        }

        None
    }

    /// Complete a training session
    fn complete_training(&self, horse: &mut Horse) -> TrainingResult {
        let session = self.active_session.as_ref().unwrap();
        let skills = session.exercise.develops_skills();

        let mut levels_gained = Vec::new();

        for (skill, base_xp) in skills {
            // Modify XP by bond level
            let bond_bonus = 1.0 + horse.bond_level * 0.5;
            let xp = (*base_xp as f32 * bond_bonus) as u32;

            if horse.training_skills.train(*skill, xp, horse.species) {
                levels_gained.push(*skill);
            }
        }

        // Training improves bond
        horse.bond_level = (horse.bond_level + 0.01).min(1.0);

        // Add horse experience
        horse.add_experience(50);

        TrainingResult {
            exercise: session.exercise,
            skills_trained: skills.iter().map(|(s, _)| *s).collect(),
            levels_gained,
            xp_gained: skills.iter().map(|(_, xp)| *xp).sum(),
        }
    }

    /// Cancel current training
    pub fn cancel(&mut self) {
        self.active_session = None;
    }

    /// Get training progress (0.0-1.0)
    pub fn progress(&self) -> f32 {
        self.active_session.as_ref()
            .map(|s| s.progress / s.duration)
            .unwrap_or(0.0)
    }
}

/// Active training session
#[derive(Debug, Clone)]
pub struct TrainingSession {
    pub exercise: TrainingExercise,
    pub progress: f32,
    pub duration: f32,
    pub started: bool,
}

/// Result of completed training
#[derive(Debug, Clone)]
pub struct TrainingResult {
    pub exercise: TrainingExercise,
    pub skills_trained: Vec<TrainingSkill>,
    pub levels_gained: Vec<TrainingSkill>,
    pub xp_gained: u32,
}
