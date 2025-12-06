//! Crew management system
//!
//! Handles ship crew including:
//! - Crew skills and roles
//! - Morale and discipline
//! - Wages and provisions
//! - Crew events

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A crew member on a ship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewMember {
    pub id: u64,
    pub name: String,
    pub role: CrewRole,
    pub skills: CrewSkills,
    pub morale: f32,
    pub health: f32,
    pub loyalty: f32,       // 0-100, affects mutiny chance
    pub wage: u32,          // Monthly wage in shillings
    pub months_served: u32,
    pub origin: CrewOrigin,
}

impl CrewMember {
    pub fn new_random(id: u64, role: CrewRole) -> Self {
        Self {
            id,
            name: generate_name(),
            role,
            skills: CrewSkills::for_role(role),
            morale: 60.0 + rand_float() * 30.0,
            health: 80.0 + rand_float() * 20.0,
            loyalty: 50.0 + rand_float() * 30.0,
            wage: role.base_wage(),
            months_served: 0,
            origin: CrewOrigin::random(),
        }
    }

    pub fn is_effective(&self) -> bool {
        self.health > 30.0 && self.morale > 20.0
    }

    pub fn effectiveness(&self) -> f32 {
        let health_factor = self.health / 100.0;
        let morale_factor = self.morale / 100.0;
        (health_factor * morale_factor).sqrt()
    }

    /// Update crew member state
    pub fn update(&mut self, conditions: &ShipConditions) {
        // Health affected by conditions
        if conditions.food_quality < 0.5 {
            self.health -= 0.5;
        }
        if conditions.water_available < 0.5 {
            self.health -= 1.0;
        }
        if conditions.overcrowded {
            self.health -= 0.2;
        }

        // Morale affected by various factors
        if conditions.wages_paid {
            self.morale += 2.0;
            self.loyalty += 0.5;
        } else {
            self.morale -= 5.0;
            self.loyalty -= 2.0;
        }

        if conditions.recent_victory {
            self.morale += 10.0;
            self.loyalty += 3.0;
        }

        if conditions.recent_defeat {
            self.morale -= 15.0;
            self.loyalty -= 5.0;
        }

        if conditions.captain_popular {
            self.morale += 1.0;
            self.loyalty += 1.0;
        }

        // Clamp values
        self.morale = self.morale.clamp(0.0, 100.0);
        self.health = self.health.clamp(0.0, 100.0);
        self.loyalty = self.loyalty.clamp(0.0, 100.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrewRole {
    Captain,
    FirstMate,
    Quartermaster,
    Boatswain,
    Navigator,
    Surgeon,
    Gunner,
    Carpenter,
    Cook,
    Sailor,
    Marine,
}

impl CrewRole {
    pub fn base_wage(&self) -> u32 {
        match self {
            Self::Captain => 100,
            Self::FirstMate => 60,
            Self::Quartermaster => 50,
            Self::Boatswain => 40,
            Self::Navigator => 45,
            Self::Surgeon => 50,
            Self::Gunner => 30,
            Self::Carpenter => 35,
            Self::Cook => 25,
            Self::Sailor => 20,
            Self::Marine => 25,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Captain => "Commander of the vessel",
            Self::FirstMate => "Second in command, manages crew",
            Self::Quartermaster => "Handles supplies and discipline",
            Self::Boatswain => "Maintains rigging and sails",
            Self::Navigator => "Charts course and reads stars",
            Self::Surgeon => "Tends to wounded and sick",
            Self::Gunner => "Operates and maintains cannon",
            Self::Carpenter => "Repairs hull damage",
            Self::Cook => "Prepares meals for crew",
            Self::Sailor => "General deck work",
            Self::Marine => "Ship's soldier for combat",
        }
    }

    pub fn required_per_100_crew(&self) -> u32 {
        match self {
            Self::Captain => 1,
            Self::FirstMate => 1,
            Self::Quartermaster => 1,
            Self::Boatswain => 2,
            Self::Navigator => 1,
            Self::Surgeon => 1,
            Self::Gunner => 10,
            Self::Carpenter => 2,
            Self::Cook => 2,
            Self::Sailor => 60,
            Self::Marine => 20,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CrewSkills {
    pub sailing: f32,    // 0-1
    pub gunnery: f32,
    pub combat: f32,
    pub navigation: f32,
    pub repair: f32,
    pub medicine: f32,
}

impl CrewSkills {
    pub fn for_role(role: CrewRole) -> Self {
        match role {
            CrewRole::Captain => Self {
                sailing: 0.8,
                gunnery: 0.5,
                combat: 0.6,
                navigation: 0.7,
                repair: 0.3,
                medicine: 0.2,
            },
            CrewRole::Navigator => Self {
                sailing: 0.6,
                navigation: 0.9,
                ..Default::default()
            },
            CrewRole::Gunner => Self {
                gunnery: 0.8,
                combat: 0.4,
                ..Default::default()
            },
            CrewRole::Surgeon => Self {
                medicine: 0.9,
                ..Default::default()
            },
            CrewRole::Carpenter => Self {
                repair: 0.9,
                sailing: 0.3,
                ..Default::default()
            },
            CrewRole::Marine => Self {
                combat: 0.8,
                gunnery: 0.4,
                ..Default::default()
            },
            CrewRole::Sailor => Self {
                sailing: 0.6,
                combat: 0.3,
                repair: 0.3,
                ..Default::default()
            },
            _ => Self {
                sailing: 0.5,
                combat: 0.3,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrewOrigin {
    English,
    Welsh,
    Scottish,
    Irish,
    Dutch,
    French,
    Spanish,
    Portuguese,
    African,
    NativeAmerican,
    Mixed,
}

impl CrewOrigin {
    pub fn random() -> Self {
        let roll = (rand_float() * 11.0) as u32;
        match roll {
            0 => Self::English,
            1 => Self::Welsh,
            2 => Self::Scottish,
            3 => Self::Irish,
            4 => Self::Dutch,
            5 => Self::French,
            6 => Self::Spanish,
            7 => Self::Portuguese,
            8 => Self::African,
            9 => Self::NativeAmerican,
            _ => Self::Mixed,
        }
    }
}

/// Conditions affecting crew
#[derive(Debug, Clone, Default)]
pub struct ShipConditions {
    pub food_quality: f32,
    pub water_available: f32,
    pub overcrowded: bool,
    pub wages_paid: bool,
    pub recent_victory: bool,
    pub recent_defeat: bool,
    pub captain_popular: bool,
}

/// Crew roster for a ship
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrewRoster {
    pub members: Vec<CrewMember>,
    pub next_id: u64,
    pub total_wages_owed: u32,
    pub provisions_remaining: u32,  // Days worth
    pub water_remaining: u32,       // Days worth
}

impl CrewRoster {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new crew member
    pub fn recruit(&mut self, role: CrewRole) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        self.members.push(CrewMember::new_random(id, role));
        id
    }

    /// Get total crew count
    pub fn count(&self) -> u32 {
        self.members.len() as u32
    }

    /// Get effective crew (healthy and not mutinous)
    pub fn effective_count(&self) -> u32 {
        self.members.iter().filter(|m| m.is_effective()).count() as u32
    }

    /// Get average morale
    pub fn average_morale(&self) -> f32 {
        if self.members.is_empty() {
            return 50.0;
        }
        self.members.iter().map(|m| m.morale).sum::<f32>() / self.members.len() as f32
    }

    /// Get average experience
    pub fn average_experience(&self) -> f32 {
        if self.members.is_empty() {
            return 0.3;
        }

        // Experience based on skills and months served
        let total: f32 = self.members
            .iter()
            .map(|m| {
                let skill_avg = (m.skills.sailing + m.skills.gunnery + m.skills.combat) / 3.0;
                let service_bonus = (m.months_served as f32 / 24.0).min(0.3);
                skill_avg + service_bonus
            })
            .sum();

        (total / self.members.len() as f32).min(1.0)
    }

    /// Calculate total monthly wages
    pub fn monthly_wages(&self) -> u32 {
        self.members.iter().map(|m| m.wage).sum()
    }

    /// Pay wages (returns true if successful)
    pub fn pay_wages(&mut self, treasury: &mut u32) -> bool {
        let total = self.monthly_wages();
        if *treasury >= total {
            *treasury -= total;
            self.total_wages_owed = 0;
            for member in &mut self.members {
                member.months_served += 1;
            }
            true
        } else {
            self.total_wages_owed += total;
            false
        }
    }

    /// Check for mutiny conditions
    pub fn mutiny_risk(&self) -> f32 {
        let avg_loyalty: f32 = if self.members.is_empty() {
            50.0
        } else {
            self.members.iter().map(|m| m.loyalty).sum::<f32>() / self.members.len() as f32
        };

        let morale_factor = 1.0 - (self.average_morale() / 100.0);
        let loyalty_factor = 1.0 - (avg_loyalty / 100.0);
        let wage_factor = if self.total_wages_owed > self.monthly_wages() { 0.3 } else { 0.0 };

        (morale_factor * 0.3 + loyalty_factor * 0.4 + wage_factor).min(1.0)
    }

    /// Remove dead or deserted crew
    pub fn cleanup_crew(&mut self) {
        self.members.retain(|m| m.health > 0.0 && m.loyalty > 5.0);
    }

    /// Get crew with specific role
    pub fn get_by_role(&self, role: CrewRole) -> Vec<&CrewMember> {
        self.members.iter().filter(|m| m.role == role).collect()
    }

    /// Get skill level for a specific skill (best among crew)
    pub fn best_skill(&self, skill: impl Fn(&CrewSkills) -> f32) -> f32 {
        self.members
            .iter()
            .map(|m| skill(&m.skills) * m.effectiveness())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Consume provisions
    pub fn consume_daily(&mut self) {
        let crew_count = self.count();
        let food_consumed = crew_count / 10 + 1;
        let water_consumed = crew_count / 8 + 1;

        self.provisions_remaining = self.provisions_remaining.saturating_sub(food_consumed);
        self.water_remaining = self.water_remaining.saturating_sub(water_consumed);
    }
}

/// Crew event that can occur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewEvent {
    pub event_type: CrewEventType,
    pub affected_crew: Vec<u64>,
    pub description: String,
    pub morale_change: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrewEventType {
    // Positive
    ShantyNight,      // Crew sings, +morale
    SuccessfulHunt,   // Fresh food, +morale +health
    LandSighted,      // Relief, +morale
    PrizeShared,      // Loot distributed, +morale +loyalty

    // Negative
    Scurvy,           // Illness, -health
    Accident,         // Injury
    Fight,            // Crew conflict
    Desertion,        // Crew member leaves
    Theft,            // Supplies stolen
    DisciplineProblem,

    // Neutral
    NewSkillLearned,  // Crew improves
    Rumor,            // Affects morale randomly
}

impl CrewEventType {
    pub fn random_positive() -> Self {
        match (rand_float() * 4.0) as u32 {
            0 => Self::ShantyNight,
            1 => Self::SuccessfulHunt,
            2 => Self::LandSighted,
            _ => Self::PrizeShared,
        }
    }

    pub fn random_negative() -> Self {
        match (rand_float() * 6.0) as u32 {
            0 => Self::Scurvy,
            1 => Self::Accident,
            2 => Self::Fight,
            3 => Self::Desertion,
            4 => Self::Theft,
            _ => Self::DisciplineProblem,
        }
    }
}

fn generate_name() -> String {
    let first_names = [
        "John", "William", "Thomas", "Robert", "James", "Richard", "Edward",
        "Henry", "George", "Charles", "Jack", "Samuel", "Benjamin", "Patrick",
        "Michael", "Daniel", "Francis", "Peter", "Christopher", "Nicholas",
    ];

    let last_names = [
        "Smith", "Jones", "Brown", "Wilson", "Taylor", "Johnson", "White",
        "Martin", "Anderson", "Clark", "Walker", "Hall", "Young", "King",
        "Wright", "Hill", "Scott", "Green", "Adams", "Baker",
    ];

    let first = first_names[(rand_float() * first_names.len() as f32) as usize];
    let last = last_names[(rand_float() * last_names.len() as f32) as usize];

    format!("{} {}", first, last)
}

fn rand_float() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}
