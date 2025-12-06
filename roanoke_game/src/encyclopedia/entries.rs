//! Pre-defined encyclopedia entry data and helpers
//!
//! Contains the full knowledge database for all species,
//! formatted for gradual reveal through the discovery system.

use super::*;

/// Generate a display string for fauna at current discovery tier
pub fn format_fauna_entry(species: AnimalSpecies, knowledge: &FaunaKnowledge, tier: DiscoveryTier) -> String {
    let mut lines = Vec::new();

    match tier {
        DiscoveryTier::Unknown => {
            lines.push("Species: ???".to_string());
            lines.push("No information available.".to_string());
            lines.push("Seek out this creature to learn more.".to_string());
        }
        DiscoveryTier::Sighted => {
            if let Some(name) = &knowledge.common_name {
                lines.push(format!("Common Name: {}", name));
            }
            if let Some(size) = &knowledge.size_category {
                lines.push(format!("Size: {:?}", size));
            }
            lines.push(String::new());
            lines.push("Observe longer to learn more about this creature.".to_string());
        }
        DiscoveryTier::Observed => {
            if let Some(name) = &knowledge.common_name {
                lines.push(format!("Common Name: {}", name));
            }
            if let Some(size) = &knowledge.size_category {
                lines.push(format!("Size: {:?}", size));
            }
            if let Some(diet) = &knowledge.diet {
                lines.push(format!("Diet: {:?}", diet));
            }
            if let Some(activity) = &knowledge.activity_pattern {
                lines.push(format!("Activity: {:?}", activity));
            }
            if let Some(habitats) = &knowledge.habitats {
                lines.push(format!("Habitats: {}", habitats.join(", ")));
            }
            lines.push(String::new());
            lines.push("Continue studying to uncover weaknesses and uses.".to_string());
        }
        DiscoveryTier::Studied => {
            if let Some(name) = &knowledge.common_name {
                lines.push(format!("Common Name: {}", name));
            }
            if let Some(size) = &knowledge.size_category {
                lines.push(format!("Size: {:?}", size));
            }
            if let Some(diet) = &knowledge.diet {
                lines.push(format!("Diet: {:?}", diet));
            }
            if let Some(activity) = &knowledge.activity_pattern {
                lines.push(format!("Activity: {:?}", activity));
            }
            if let Some(habitats) = &knowledge.habitats {
                lines.push(format!("Habitats: {}", habitats.join(", ")));
            }
            lines.push(String::new());
            if let Some(danger) = &knowledge.danger_level {
                lines.push(format!("Danger Level: {}/10", danger));
            }
            if let Some(attacks) = &knowledge.attack_descriptions {
                lines.push(format!("Attacks: {}", attacks.join(", ")));
            }
            if let Some(weaknesses) = &knowledge.weaknesses {
                lines.push(format!("Weaknesses: {}", weaknesses.join(", ")));
            }
            if let Some(loot) = &knowledge.loot_items {
                lines.push(format!("Yields: {}", loot.join(", ")));
            }
            lines.push(String::new());
            lines.push("Achieve mastery to unlock full naturalist knowledge.".to_string());
        }
        DiscoveryTier::Mastered => {
            if let Some(name) = &knowledge.common_name {
                lines.push(format!("Common Name: {}", name));
            }
            if let Some(sci_name) = &knowledge.scientific_name {
                lines.push(format!("Scientific Name: {}", sci_name));
            }
            if let Some(size) = &knowledge.size_category {
                lines.push(format!("Size: {:?}", size));
            }
            if let Some(diet) = &knowledge.diet {
                lines.push(format!("Diet: {:?}", diet));
            }
            if let Some(activity) = &knowledge.activity_pattern {
                lines.push(format!("Activity: {:?}", activity));
            }
            if let Some(habitats) = &knowledge.habitats {
                lines.push(format!("Habitats: {}", habitats.join(", ")));
            }
            lines.push(String::new());
            if let Some(danger) = &knowledge.danger_level {
                lines.push(format!("Danger Level: {}/10", danger));
            }
            if let Some(attacks) = &knowledge.attack_descriptions {
                lines.push(format!("Attacks: {}", attacks.join(", ")));
            }
            if let Some(weaknesses) = &knowledge.weaknesses {
                lines.push(format!("Weaknesses: {}", weaknesses.join(", ")));
            }
            if let Some(loot) = &knowledge.loot_items {
                lines.push(format!("Yields: {}", loot.join(", ")));
            }
            lines.push(String::new());
            if let Some(notes) = &knowledge.behavioral_notes {
                lines.push("Naturalist Notes:".to_string());
                lines.push(notes.clone());
            }
            if let Some(bonus) = &knowledge.tracking_bonus {
                lines.push(String::new());
                lines.push(format!("Tracking Bonus: +{}%", (bonus * 100.0) as i32));
            }
        }
    }

    lines.join("\n")
}

/// Generate a display string for flora at current discovery tier
pub fn format_flora_entry(species: FloraSpecies, knowledge: &FloraKnowledge, tier: DiscoveryTier) -> String {
    let mut lines = Vec::new();

    match tier {
        DiscoveryTier::Unknown => {
            lines.push("Species: ???".to_string());
            lines.push("No information available.".to_string());
            lines.push("Find this plant to begin your study.".to_string());
        }
        DiscoveryTier::Sighted => {
            if let Some(name) = &knowledge.common_name {
                lines.push(format!("Common Name: {}", name));
            }
            if let Some(category) = &knowledge.category {
                lines.push(format!("Type: {:?}", category));
            }
            lines.push(String::new());
            lines.push("Spend more time studying to learn about its uses.".to_string());
        }
        DiscoveryTier::Observed => {
            if let Some(name) = &knowledge.common_name {
                lines.push(format!("Common Name: {}", name));
            }
            if let Some(category) = &knowledge.category {
                lines.push(format!("Type: {:?}", category));
            }
            if let Some(seasons) = &knowledge.seasons {
                let season_strs: Vec<_> = seasons.iter().map(|s| format!("{:?}", s)).collect();
                lines.push(format!("Growing Seasons: {}", season_strs.join(", ")));
            }
            if let Some(habitats) = &knowledge.habitats {
                lines.push(format!("Found In: {}", habitats.join(", ")));
            }
            lines.push(String::new());
            lines.push("Continue studying to learn about edibility and medicinal uses.".to_string());
        }
        DiscoveryTier::Studied => {
            if let Some(name) = &knowledge.common_name {
                lines.push(format!("Common Name: {}", name));
            }
            if let Some(category) = &knowledge.category {
                lines.push(format!("Type: {:?}", category));
            }
            if let Some(seasons) = &knowledge.seasons {
                let season_strs: Vec<_> = seasons.iter().map(|s| format!("{:?}", s)).collect();
                lines.push(format!("Growing Seasons: {}", season_strs.join(", ")));
            }
            if let Some(habitats) = &knowledge.habitats {
                lines.push(format!("Found In: {}", habitats.join(", ")));
            }
            lines.push(String::new());
            if let Some(edibility) = &knowledge.edibility {
                lines.push(format!("Edibility: {:?}", edibility));
            }
            if let Some(toxicity) = &knowledge.toxicity {
                lines.push(format!("Toxicity: {:?}", toxicity.level));
                if !toxicity.symptoms.is_empty() {
                    lines.push(format!("Symptoms: {}", toxicity.symptoms.join(", ")));
                }
            }
            if let Some(medicinal) = &knowledge.medicinal_uses {
                if !medicinal.is_empty() {
                    lines.push(format!("Medicinal Uses: {}", medicinal.join(", ")));
                }
            }
            if let Some(harvest) = &knowledge.harvest_info {
                lines.push(format!("Harvest: {}", harvest));
            }
            lines.push(String::new());
            lines.push("Achieve mastery for full botanical knowledge.".to_string());
        }
        DiscoveryTier::Mastered => {
            if let Some(name) = &knowledge.common_name {
                lines.push(format!("Common Name: {}", name));
            }
            if let Some(sci_name) = &knowledge.scientific_name {
                lines.push(format!("Scientific Name: {}", sci_name));
            }
            if let Some(category) = &knowledge.category {
                lines.push(format!("Type: {:?}", category));
            }
            if let Some(seasons) = &knowledge.seasons {
                let season_strs: Vec<_> = seasons.iter().map(|s| format!("{:?}", s)).collect();
                lines.push(format!("Growing Seasons: {}", season_strs.join(", ")));
            }
            if let Some(habitats) = &knowledge.habitats {
                lines.push(format!("Found In: {}", habitats.join(", ")));
            }
            lines.push(String::new());
            if let Some(edibility) = &knowledge.edibility {
                lines.push(format!("Edibility: {:?}", edibility));
            }
            if let Some(toxicity) = &knowledge.toxicity {
                lines.push(format!("Toxicity: {:?}", toxicity.level));
                if !toxicity.symptoms.is_empty() {
                    lines.push(format!("Symptoms: {}", toxicity.symptoms.join(", ")));
                }
            }
            if let Some(medicinal) = &knowledge.medicinal_uses {
                if !medicinal.is_empty() {
                    lines.push(format!("Medicinal Uses: {}", medicinal.join(", ")));
                }
            }
            if let Some(harvest) = &knowledge.harvest_info {
                lines.push(format!("Harvest: {}", harvest));
            }
            lines.push(String::new());
            if let Some(notes) = &knowledge.botanical_notes {
                lines.push("Botanical Notes:".to_string());
                lines.push(notes.clone());
            }
            if let Some(tips) = &knowledge.cultivation_tips {
                lines.push(String::new());
                lines.push("Cultivation:".to_string());
                lines.push(tips.clone());
            }
        }
    }

    lines.join("\n")
}

/// Get a hint about where to find a species
pub fn get_fauna_location_hint(species: AnimalSpecies) -> &'static str {
    match species {
        AnimalSpecies::BlackBear => "Roams forests and mountain foothills, especially near berry patches",
        AnimalSpecies::EasternCougar => "Stalks the deep forest and rocky mountain terrain",
        AnimalSpecies::GrayWolf => "Hunts in packs across forests and open plains",
        AnimalSpecies::TimberRattlesnake => "Basks on sunny rocks in forest clearings",
        AnimalSpecies::AmericanAlligator => "Lurks in swamps, rivers, and coastal marshes",
        AnimalSpecies::WildBoar => "Roots through forest underbrush and swampy areas",
        AnimalSpecies::Copperhead => "Hides among leaf litter in deciduous forests",
        AnimalSpecies::RedWolf => "Rare; seeks out swamps and coastal plain forests",
        AnimalSpecies::Bobcat => "Prowls forests and rocky terrain at twilight",
        AnimalSpecies::Cottonmouth => "Swims through swamps and waterways",
    }
}

/// Get the encyclopedia icon/silhouette for unknown species (mystery shape)
pub fn get_mystery_silhouette(species: AnimalSpecies) -> &'static str {
    match species {
        AnimalSpecies::BlackBear | AnimalSpecies::WildBoar => "large_quadruped",
        AnimalSpecies::EasternCougar | AnimalSpecies::Bobcat => "feline",
        AnimalSpecies::GrayWolf | AnimalSpecies::RedWolf => "canine",
        AnimalSpecies::TimberRattlesnake | AnimalSpecies::Copperhead | AnimalSpecies::Cottonmouth => "serpent",
        AnimalSpecies::AmericanAlligator => "reptile_large",
    }
}

/// Statistics for encyclopedia completion tracking
#[derive(Debug, Clone, Default)]
pub struct EncyclopediaStats {
    pub fauna_sighted: u32,
    pub fauna_observed: u32,
    pub fauna_studied: u32,
    pub fauna_mastered: u32,
    pub flora_sighted: u32,
    pub flora_observed: u32,
    pub flora_studied: u32,
    pub flora_mastered: u32,
    pub total_observation_time: f32,
    pub rarest_discovery: Option<String>,
    pub most_observed: Option<String>,
}

impl EncyclopediaStats {
    pub fn from_encyclopedia(encyclopedia: &Encyclopedia) -> Self {
        let mut stats = Self::default();
        let mut max_obs_time = 0.0f32;
        let mut max_obs_name = None;

        for (species, entry) in &encyclopedia.fauna {
            match entry.tier {
                DiscoveryTier::Sighted => stats.fauna_sighted += 1,
                DiscoveryTier::Observed => stats.fauna_observed += 1,
                DiscoveryTier::Studied => stats.fauna_studied += 1,
                DiscoveryTier::Mastered => stats.fauna_mastered += 1,
                _ => {}
            }
            stats.total_observation_time += entry.observation_time;

            if entry.observation_time > max_obs_time {
                max_obs_time = entry.observation_time;
                max_obs_name = Some(species.name().to_string());
            }
        }

        for (species, entry) in &encyclopedia.flora {
            match entry.tier {
                DiscoveryTier::Sighted => stats.flora_sighted += 1,
                DiscoveryTier::Observed => stats.flora_observed += 1,
                DiscoveryTier::Studied => stats.flora_studied += 1,
                DiscoveryTier::Mastered => stats.flora_mastered += 1,
                _ => {}
            }
            stats.total_observation_time += entry.observation_time;

            if entry.observation_time > max_obs_time {
                max_obs_time = entry.observation_time;
                max_obs_name = Some(species.name().to_string());
            }
        }

        stats.most_observed = max_obs_name;
        stats
    }

    pub fn total_discovered(&self) -> u32 {
        self.fauna_sighted + self.fauna_observed + self.fauna_studied + self.fauna_mastered
            + self.flora_sighted + self.flora_observed + self.flora_studied + self.flora_mastered
    }

    pub fn total_mastered(&self) -> u32 {
        self.fauna_mastered + self.flora_mastered
    }
}
