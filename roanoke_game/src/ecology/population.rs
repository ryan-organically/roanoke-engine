//! Population dynamics modeling
//!
//! Implements realistic population ecology including:
//! - Lotka-Volterra predator-prey dynamics
//! - Allee effects (minimum viable population)
//! - Density-dependent growth
//! - Migration and dispersal

use serde::{Deserialize, Serialize};
use crate::animals::types::AnimalSpecies;

/// Detailed population model for a species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationModel {
    pub species: AnimalSpecies,
    pub adults: u32,
    pub juveniles: u32,
    pub age_structure: AgeStructure,

    // Vital rates
    pub fecundity: f32,           // Offspring per adult female per year
    pub juvenile_survival: f32,    // Probability of juvenile surviving to adult
    pub adult_survival: f32,       // Annual adult survival rate

    // Population dynamics
    pub carrying_capacity: u32,
    pub growth_rate: f32,          // Intrinsic growth rate (r)
    pub allee_threshold: u32,      // Minimum viable population

    // External pressures
    pub hunting_mortality: f32,
    pub disease_mortality: f32,
    pub starvation_mortality: f32,
}

impl PopulationModel {
    pub fn new(species: AnimalSpecies, initial_pop: u32, carrying_capacity: u32) -> Self {
        let (fecundity, juv_surv, adult_surv, allee) = species.life_history_traits();

        Self {
            species,
            adults: (initial_pop as f32 * 0.7) as u32,
            juveniles: (initial_pop as f32 * 0.3) as u32,
            age_structure: AgeStructure::Stable,
            fecundity,
            juvenile_survival: juv_surv,
            adult_survival: adult_surv,
            carrying_capacity,
            growth_rate: calculate_intrinsic_growth(fecundity, adult_surv),
            allee_threshold: allee,
            hunting_mortality: 0.0,
            disease_mortality: 0.0,
            starvation_mortality: 0.0,
        }
    }

    pub fn total_population(&self) -> u32 {
        self.adults + self.juveniles
    }

    /// Update population over a time step (in days)
    pub fn update(&mut self, delta_days: f32) {
        let dt = delta_days / 365.0; // Convert to years

        // Check for Allee effect (population too small to sustain)
        if self.total_population() < self.allee_threshold {
            // Population decline due to difficulty finding mates, etc.
            let decline_rate = 0.2 * dt;
            self.adults = (self.adults as f32 * (1.0 - decline_rate)) as u32;
            self.juveniles = (self.juveniles as f32 * (1.0 - decline_rate)) as u32;
            self.age_structure = AgeStructure::Declining;
            return;
        }

        // Logistic growth model
        let n = self.total_population() as f32;
        let k = self.carrying_capacity as f32;
        let r = self.growth_rate;

        // dN/dt = rN(1 - N/K)
        let growth_factor = 1.0 - n / k;

        // Reproduction (only from adults)
        let births = (self.adults as f32 * self.fecundity * 0.5 * growth_factor * dt) as u32;
        self.juveniles += births;

        // Maturation (juveniles -> adults)
        let maturing = (self.juveniles as f32 * self.juvenile_survival * dt * 2.0) as u32;
        self.juveniles = self.juveniles.saturating_sub(maturing);
        self.adults += maturing;

        // Natural mortality
        let adult_deaths = (self.adults as f32 * (1.0 - self.adult_survival) * dt) as u32;
        let juv_deaths = (self.juveniles as f32 * (1.0 - self.juvenile_survival) * dt) as u32;

        self.adults = self.adults.saturating_sub(adult_deaths);
        self.juveniles = self.juveniles.saturating_sub(juv_deaths);

        // Additional mortality factors
        let hunting_deaths = (self.adults as f32 * self.hunting_mortality * dt) as u32;
        let disease_deaths = (self.total_population() as f32 * self.disease_mortality * dt) as u32;
        let starvation_deaths = (self.total_population() as f32 * self.starvation_mortality * dt) as u32;

        self.adults = self.adults.saturating_sub(hunting_deaths);
        self.adults = self.adults.saturating_sub(disease_deaths / 2);
        self.juveniles = self.juveniles.saturating_sub(disease_deaths / 2);
        self.adults = self.adults.saturating_sub(starvation_deaths / 2);
        self.juveniles = self.juveniles.saturating_sub(starvation_deaths / 2);

        // Update age structure assessment
        self.update_age_structure();

        // Decay external mortality factors
        self.hunting_mortality *= 0.95;
        self.disease_mortality *= 0.98;
        self.starvation_mortality *= 0.9;
    }

    fn update_age_structure(&mut self) {
        let adult_ratio = self.adults as f32 / (self.total_population().max(1) as f32);

        self.age_structure = if self.total_population() < self.allee_threshold {
            AgeStructure::Declining
        } else if adult_ratio > 0.8 {
            AgeStructure::Aging
        } else if adult_ratio < 0.4 {
            AgeStructure::Growing
        } else {
            AgeStructure::Stable
        };
    }

    /// Apply hunting pressure
    pub fn apply_hunting(&mut self, kills: u32) {
        self.adults = self.adults.saturating_sub(kills);
        self.hunting_mortality += 0.1 * kills as f32;
    }

    /// Apply disease outbreak
    pub fn apply_disease(&mut self, severity: f32) {
        self.disease_mortality = (self.disease_mortality + severity).min(0.5);
    }

    /// Apply food scarcity
    pub fn apply_starvation(&mut self, severity: f32) {
        self.starvation_mortality = (self.starvation_mortality + severity).min(0.3);
    }

    /// Get population status
    pub fn status(&self) -> PopulationStatus {
        let ratio = self.total_population() as f32 / self.carrying_capacity as f32;

        if self.total_population() == 0 {
            PopulationStatus::Extinct
        } else if self.total_population() < self.allee_threshold {
            PopulationStatus::Critical
        } else if ratio < 0.25 {
            PopulationStatus::Endangered
        } else if ratio < 0.5 {
            PopulationStatus::Vulnerable
        } else if ratio > 1.2 {
            PopulationStatus::Overabundant
        } else {
            PopulationStatus::Stable
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgeStructure {
    Growing,   // More juveniles, population expanding
    Stable,    // Balanced age distribution
    Aging,     // More adults, reproduction declining
    Declining, // Population falling, may go extinct
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopulationStatus {
    Extinct,      // Gone
    Critical,     // Below Allee threshold
    Endangered,   // Very low numbers
    Vulnerable,   // Below healthy levels
    Stable,       // Healthy population
    Overabundant, // Above carrying capacity
}

impl PopulationStatus {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Extinct => "Locally extinct - no individuals remain",
            Self::Critical => "Critically endangered - population may not recover",
            Self::Endangered => "Endangered - numbers are dangerously low",
            Self::Vulnerable => "Vulnerable - population is stressed",
            Self::Stable => "Stable - healthy population levels",
            Self::Overabundant => "Overabundant - exceeds sustainable levels",
        }
    }

    pub fn conservation_priority(&self) -> u8 {
        match self {
            Self::Extinct => 0,
            Self::Critical => 5,
            Self::Endangered => 4,
            Self::Vulnerable => 3,
            Self::Stable => 1,
            Self::Overabundant => 2,
        }
    }
}

fn calculate_intrinsic_growth(fecundity: f32, adult_survival: f32) -> f32 {
    // Simplified calculation of r from life history traits
    (fecundity * adult_survival).ln().max(0.01)
}

impl AnimalSpecies {
    /// Get life history traits: (fecundity, juvenile_survival, adult_survival, allee_threshold)
    pub fn life_history_traits(&self) -> (f32, f32, f32, u32) {
        match self {
            // Large carnivores: low fecundity, high survival, need larger populations
            Self::BlackBear => (2.0, 0.7, 0.85, 4),
            Self::EasternCougar => (2.5, 0.6, 0.8, 3),

            // Pack hunters: moderate fecundity, social requirements
            Self::GrayWolf => (5.0, 0.5, 0.75, 6),
            Self::RedWolf => (4.0, 0.5, 0.7, 4),

            // Solitary hunters: low-moderate fecundity
            Self::Bobcat => (3.0, 0.6, 0.8, 3),

            // Reptiles: high fecundity, low juvenile survival
            Self::AmericanAlligator => (35.0, 0.1, 0.95, 6),
            Self::TimberRattlesnake => (8.0, 0.3, 0.85, 5),
            Self::Copperhead => (6.0, 0.35, 0.8, 6),
            Self::Cottonmouth => (7.0, 0.3, 0.82, 5),

            // Prolific breeders
            Self::WildBoar => (8.0, 0.6, 0.7, 4),
        }
    }
}

/// Predator-prey interaction model
#[derive(Debug, Clone)]
pub struct PredatorPreyModel {
    pub predator_species: AnimalSpecies,
    pub prey_species: AnimalSpecies,
    pub attack_rate: f32,     // How effectively predator finds prey
    pub conversion_rate: f32, // How efficiently prey becomes predator biomass
}

impl PredatorPreyModel {
    pub fn new(predator: AnimalSpecies, prey: AnimalSpecies) -> Self {
        // Set interaction rates based on species
        let (attack, conversion) = match (predator, prey) {
            (AnimalSpecies::GrayWolf, AnimalSpecies::WildBoar) => (0.01, 0.1),
            (AnimalSpecies::EasternCougar, AnimalSpecies::WildBoar) => (0.008, 0.12),
            (AnimalSpecies::Bobcat, AnimalSpecies::WildBoar) => (0.005, 0.08), // Less effective
            (AnimalSpecies::AmericanAlligator, _) => (0.015, 0.15), // Efficient predator
            _ => (0.005, 0.1),
        };

        Self {
            predator_species: predator,
            prey_species: prey,
            attack_rate: attack,
            conversion_rate: conversion,
        }
    }

    /// Calculate predation rate using Lotka-Volterra equations
    pub fn calculate_predation(&self, predator_pop: u32, prey_pop: u32) -> (i32, i32) {
        let alpha = self.attack_rate;
        let beta = self.conversion_rate;

        // Prey killed
        let kills = (alpha * predator_pop as f32 * prey_pop as f32) as i32;

        // Predator growth from kills
        let predator_growth = (beta * kills as f32) as i32;

        (-kills, predator_growth) // (prey change, predator change)
    }
}
