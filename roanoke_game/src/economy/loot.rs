//! Loot Generation System
//!
//! Generates items with rarity, quality, and modifiers based on context.

use super::item::*;
use serde::{Deserialize, Serialize};
use rand::Rng;

/// Loot generation context
#[derive(Debug, Clone)]
pub struct LootContext {
    /// Player's current luck stat
    pub luck: f32,

    /// Relevant skill level (0-100)
    pub skill_level: u32,

    /// Minutes in current session (for pity)
    pub session_minutes: u32,

    /// Player's karma (pity accumulation)
    pub karma: f32,

    /// Is this from a kill?
    pub kill_context: Option<KillContext>,

    /// Moon phase (0.0 = new, 1.0 = full)
    pub moon_phase: f32,

    /// Current weather modifier
    pub weather_modifier: f32,

    /// Active event multiplier
    pub event_multiplier: f32,
}

impl Default for LootContext {
    fn default() -> Self {
        Self {
            luck: 0.0,
            skill_level: 1,
            session_minutes: 0,
            karma: 0.0,
            kill_context: None,
            moon_phase: 0.5,
            weather_modifier: 1.0,
            event_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KillContext {
    pub species_name: String,
    pub was_stealth: bool,
    pub was_perfect: bool,
    pub was_critical: bool,
    pub weapon_used: String,
    pub kill_time_seconds: f32,
    pub player_health_remaining: f32,
}

/// Pity system tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PityTracker {
    pub karma_accumulated: f32,
    pub drops_since_rare: u32,
    pub drops_since_epic: u32,
    pub drops_since_legendary: u32,

    // Lifetime stats
    pub lifetime_drops: u64,
    pub lifetime_legendary: u32,
    pub lifetime_mythic: u32,
    pub lifetime_primordial: u32,
}

impl PityTracker {
    /// Hard pity thresholds
    const RARE_PITY: u32 = 50;
    const EPIC_PITY: u32 = 200;
    const LEGENDARY_PITY: u32 = 1000;

    /// Update after a drop
    pub fn record_drop(&mut self, rarity: Rarity) {
        self.lifetime_drops += 1;

        match rarity {
            Rarity::Rare | Rarity::Epic | Rarity::Legendary | Rarity::Mythic | Rarity::Primordial => {
                self.drops_since_rare = 0;
            }
            _ => {
                self.drops_since_rare += 1;
            }
        }

        match rarity {
            Rarity::Epic | Rarity::Legendary | Rarity::Mythic | Rarity::Primordial => {
                self.drops_since_epic = 0;
            }
            _ => {
                self.drops_since_epic += 1;
            }
        }

        match rarity {
            Rarity::Legendary => {
                self.drops_since_legendary = 0;
                self.lifetime_legendary += 1;
            }
            Rarity::Mythic => {
                self.drops_since_legendary = 0;
                self.lifetime_mythic += 1;
            }
            Rarity::Primordial => {
                self.drops_since_legendary = 0;
                self.lifetime_primordial += 1;
            }
            _ => {
                self.drops_since_legendary += 1;
            }
        }

        // Karma adjustment
        let expected_value = 0.5; // Middle rarity
        let actual_value = rarity as u32 as f32 / 7.0;
        let deficit = expected_value - actual_value;

        if deficit > 0.0 {
            self.karma_accumulated += deficit * 0.01;
        } else {
            self.karma_accumulated = (self.karma_accumulated - 0.02).max(0.0);
        }
    }

    /// Check if hard pity triggers
    pub fn check_pity(&self) -> Option<Rarity> {
        if self.drops_since_legendary >= Self::LEGENDARY_PITY {
            Some(Rarity::Legendary)
        } else if self.drops_since_epic >= Self::EPIC_PITY {
            Some(Rarity::Epic)
        } else if self.drops_since_rare >= Self::RARE_PITY {
            Some(Rarity::Rare)
        } else {
            None
        }
    }

    /// Get karma modifier for drop calculation
    pub fn karma_modifier(&self) -> f32 {
        self.karma_accumulated.min(0.10) // Max 10% bonus from karma
    }
}

/// The loot generator
pub struct LootGenerator {
    /// Prefix definitions
    prefixes: Vec<PrefixDef>,
    /// Suffix definitions
    suffixes: Vec<SuffixDef>,
    /// Item templates
    templates: Vec<ItemTemplate>,
}

impl Default for LootGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl LootGenerator {
    pub fn new() -> Self {
        let mut gen = Self {
            prefixes: Vec::new(),
            suffixes: Vec::new(),
            templates: Vec::new(),
        };
        gen.initialize_prefixes();
        gen.initialize_suffixes();
        gen.initialize_templates();
        gen
    }

    /// Generate a loot drop
    pub fn generate_drop(
        &self,
        template_id: &str,
        context: &LootContext,
        pity: &mut PityTracker,
    ) -> Option<(Item, DropReward)> {
        let template = self.templates.iter().find(|t| t.id == template_id)?;

        // Check pity first
        let forced_rarity = pity.check_pity();

        // Calculate rarity
        let rarity = if let Some(r) = forced_rarity {
            r
        } else {
            self.roll_rarity(context, pity)
        };

        // Calculate quality
        let quality = self.roll_quality(context);

        // Create base item
        let mut item = template.instantiate(rarity, quality);

        // Roll for prefix (higher rarity = higher chance)
        if self.should_have_prefix(rarity) {
            if let Some(prefix) = self.roll_prefix(rarity) {
                item.prefix = Some(prefix);
            }
        }

        // Roll for suffix (higher rarity = higher chance)
        if self.should_have_suffix(rarity) {
            if let Some(suffix) = self.roll_suffix(rarity) {
                item.suffix = Some(suffix);
            }
        }

        // Set provenance
        if let Some(kill) = &context.kill_context {
            item.provenance.origin_method = OriginMethod::HuntingDrop {
                species: kill.species_name.clone(),
            };
            item.provenance.kill_record = Some(KillRecord {
                species: kill.species_name.clone(),
                was_stealth: kill.was_stealth,
                was_perfect: kill.was_perfect,
                was_critical: kill.was_critical,
                weapon_used: kill.weapon_used.clone(),
                kill_time_seconds: kill.kill_time_seconds,
                player_health_remaining: kill.player_health_remaining,
            });
        }

        // Calculate currency reward
        let wampum = self.calculate_wampum_reward(&item, context);
        let tobacco = self.calculate_tobacco_reward(&item, context);

        // Update pity tracker
        pity.record_drop(rarity);

        Some((item, DropReward { wampum, tobacco }))
    }

    /// Roll for rarity
    fn roll_rarity(&self, context: &LootContext, pity: &PityTracker) -> Rarity {
        let mut rng = rand::thread_rng();
        let base_roll: f64 = rng.gen();

        // Apply modifiers
        let luck_mod = context.luck as f64 * 0.05;
        let skill_mod = (context.skill_level as f64 / 100.0) * 0.15;
        let session_pity = (context.session_minutes as f64 * 0.001).min(0.10);
        let karma_mod = pity.karma_modifier() as f64;

        let celestial_mod = if context.moon_phase > 0.8 {
            0.05 // Full moon bonus
        } else {
            0.0
        };

        let modified_roll = base_roll
            - luck_mod
            - skill_mod
            - session_pity
            - karma_mod
            - celestial_mod;

        let final_roll = (modified_roll * context.event_multiplier as f64).clamp(0.0, 1.0);

        // Determine rarity from roll
        if final_roll <= 0.0002 {
            Rarity::Primordial
        } else if final_roll <= 0.002 {
            Rarity::Mythic
        } else if final_roll <= 0.01 {
            Rarity::Legendary
        } else if final_roll <= 0.035 {
            Rarity::Epic
        } else if final_roll <= 0.10 {
            Rarity::Rare
        } else if final_roll <= 0.25 {
            Rarity::Uncommon
        } else if final_roll <= 0.55 {
            Rarity::Common
        } else {
            Rarity::Crude
        }
    }

    /// Roll for quality (0-100)
    fn roll_quality(&self, context: &LootContext) -> Quality {
        let mut rng = rand::thread_rng();
        let base_mean = 35.0 + (context.skill_level as f32 * 0.3);
        let luck_bonus = context.luck * 20.0;
        let std_dev = 15.0 - (context.skill_level as f32 * 0.05);

        // Simple normal distribution approximation using Box-Muller
        let u1: f32 = rng.gen();
        let u2: f32 = rng.gen();
        let normal = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();

        let quality = (base_mean + luck_bonus + normal * std_dev).clamp(0.0, 100.0) as u8;
        Quality(quality)
    }

    fn should_have_prefix(&self, rarity: Rarity) -> bool {
        let mut rng = rand::thread_rng();
        let chance = match rarity {
            Rarity::Crude | Rarity::Common => 0.0,
            Rarity::Uncommon => 0.10,
            Rarity::Rare => 0.30,
            Rarity::Epic => 0.60,
            Rarity::Legendary => 0.90,
            Rarity::Mythic | Rarity::Primordial => 1.0,
        };
        rng.gen::<f32>() < chance
    }

    fn should_have_suffix(&self, rarity: Rarity) -> bool {
        let mut rng = rand::thread_rng();
        let chance = match rarity {
            Rarity::Crude | Rarity::Common => 0.0,
            Rarity::Uncommon => 0.05,
            Rarity::Rare => 0.20,
            Rarity::Epic => 0.50,
            Rarity::Legendary => 0.80,
            Rarity::Mythic | Rarity::Primordial => 1.0,
        };
        rng.gen::<f32>() < chance
    }

    fn roll_prefix(&self, min_rarity: Rarity) -> Option<ItemPrefix> {
        let mut rng = rand::thread_rng();
        let eligible: Vec<_> = self.prefixes.iter()
            .filter(|p| p.min_rarity <= min_rarity)
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let idx = rng.gen_range(0..eligible.len());
        Some(eligible[idx].to_prefix())
    }

    fn roll_suffix(&self, min_rarity: Rarity) -> Option<ItemSuffix> {
        let mut rng = rand::thread_rng();
        let eligible: Vec<_> = self.suffixes.iter()
            .filter(|s| s.min_rarity <= min_rarity)
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let idx = rng.gen_range(0..eligible.len());
        Some(eligible[idx].to_suffix())
    }

    fn calculate_wampum_reward(&self, item: &Item, context: &LootContext) -> u64 {
        let base = match item.rarity {
            Rarity::Crude => 5,
            Rarity::Common => 20,
            Rarity::Uncommon => 100,
            Rarity::Rare => 500,
            Rarity::Epic => 2500,
            Rarity::Legendary => 15000,
            Rarity::Mythic => 100000,
            Rarity::Primordial => 1000000,
        };

        let quality_mult = item.quality.stat_multiplier();

        let kill_mult = context.kill_context.as_ref().map(|k| {
            let mut m = 1.0;
            if k.was_stealth { m += 0.25; }
            if k.was_perfect { m += 0.50; }
            if k.was_critical { m += 0.15; }
            m
        }).unwrap_or(1.0);

        (base as f32 * quality_mult * kill_mult) as u64
    }

    fn calculate_tobacco_reward(&self, item: &Item, _context: &LootContext) -> u64 {
        let mut rng = rand::thread_rng();
        // TBC only drops from high rarity
        match item.rarity {
            Rarity::Legendary if rng.gen::<f32>() < 0.10 => 10,
            Rarity::Mythic if rng.gen::<f32>() < 0.25 => 100,
            Rarity::Primordial => 1000,
            _ => 0,
        }
    }

    fn initialize_prefixes(&mut self) {
        // Tier 1 prefixes (Common+)
        self.prefixes.push(PrefixDef::new("chilled", "Chilled", 1, Rarity::Common, vec![
            ItemEffect::DamagePercent(0.05),
        ]));
        self.prefixes.push(PrefixDef::new("sharp", "Sharp", 1, Rarity::Common, vec![
            ItemEffect::Damage(5.0),
        ]));
        self.prefixes.push(PrefixDef::new("lucky", "Lucky", 1, Rarity::Common, vec![
            ItemEffect::Luck(0.02),
        ]));

        // Tier 2 prefixes (Rare+)
        self.prefixes.push(PrefixDef::new("blazing", "Blazing", 2, Rarity::Rare, vec![
            ItemEffect::Damage(15.0),
            ItemEffect::DamagePercent(0.10),
        ]));
        self.prefixes.push(PrefixDef::new("hunters", "Hunter's", 2, Rarity::Rare, vec![
            ItemEffect::HuntingXp(0.15),
            ItemEffect::CritChance(0.05),
        ]));

        // Tier 3 prefixes (Epic+)
        self.prefixes.push(PrefixDef::new("volcanic", "Volcanic", 3, Rarity::Epic, vec![
            ItemEffect::Damage(30.0),
            ItemEffect::DamagePercent(0.20),
        ]));
        self.prefixes.push(PrefixDef::new("divine", "Divine", 3, Rarity::Epic, vec![
            ItemEffect::Health(50.0),
            ItemEffect::Luck(0.10),
        ]));

        // Tier 4 prefixes (Legendary+)
        self.prefixes.push(PrefixDef::new("godtouched", "Godtouched", 4, Rarity::Legendary, vec![
            ItemEffect::DamagePercent(0.35),
            ItemEffect::DropRateBonus(0.10),
        ]));
        self.prefixes.push(PrefixDef::new("primordial", "Primordial", 4, Rarity::Mythic, vec![
            ItemEffect::DamagePercent(0.50),
            ItemEffect::Luck(0.25),
            ItemEffect::DropRateBonus(0.20),
        ]));
    }

    fn initialize_suffixes(&mut self) {
        // Tier 1
        self.suffixes.push(SuffixDef::new("strength", "Strength", 1, Rarity::Common, vec![
            ItemEffect::Damage(3.0),
        ]));
        self.suffixes.push(SuffixDef::new("hunter", "the Hunter", 1, Rarity::Common, vec![
            ItemEffect::HuntingXp(0.05),
        ]));

        // Tier 2
        self.suffixes.push(SuffixDef::new("slaying", "Slaying", 2, Rarity::Rare, vec![
            ItemEffect::DamagePercent(0.15),
        ]));
        self.suffixes.push(SuffixDef::new("fortune", "Fortune", 2, Rarity::Rare, vec![
            ItemEffect::Luck(0.08),
            ItemEffect::GoldFind(0.15),
        ]));

        // Tier 3
        self.suffixes.push(SuffixDef::new("titan", "the Titan", 3, Rarity::Epic, vec![
            ItemEffect::Damage(25.0),
            ItemEffect::Health(30.0),
        ]));

        // Tier 4
        self.suffixes.push(SuffixDef::new("legend", "Legend", 4, Rarity::Legendary, vec![
            ItemEffect::DamagePercent(0.25),
            ItemEffect::Luck(0.15),
        ]));
        self.suffixes.push(SuffixDef::new("roanoke", "Roanoke", 4, Rarity::Mythic, vec![
            ItemEffect::DamagePercent(0.40),
            ItemEffect::DropRateBonus(0.15),
        ]));
    }

    fn initialize_templates(&mut self) {
        // =========== ANIMAL LOOT (matches AnimalSpecies::loot()) ===========

        // Black Bear
        self.templates.push(ItemTemplate::new("bear_pelt", "Bear Pelt", ItemType::Material, 30, 10));
        self.templates.push(ItemTemplate::new("bear_meat", "Bear Meat", ItemType::Food, 15, 20));
        self.templates.push(ItemTemplate::new("bear_fat", "Bear Fat", ItemType::Crafting, 20, 10));

        // Eastern Cougar
        self.templates.push(ItemTemplate::new("cougar_pelt", "Cougar Pelt", ItemType::Material, 35, 10));
        self.templates.push(ItemTemplate::new("cougar_meat", "Cougar Meat", ItemType::Food, 18, 20));

        // Gray Wolf / Red Wolf
        self.templates.push(ItemTemplate::new("wolf_pelt", "Wolf Pelt", ItemType::Material, 20, 10));
        self.templates.push(ItemTemplate::new("wolf_meat", "Wolf Meat", ItemType::Food, 10, 20));
        self.templates.push(ItemTemplate::new("red_wolf_pelt", "Red Wolf Pelt", ItemType::Material, 25, 10));
        self.templates.push(ItemTemplate::new("teeth", "Animal Teeth", ItemType::Crafting, 8, 20));

        // Wild Boar
        self.templates.push(ItemTemplate::new("boar_hide", "Boar Hide", ItemType::Material, 15, 10));
        self.templates.push(ItemTemplate::new("boar_meat", "Boar Meat", ItemType::Food, 8, 20));
        self.templates.push(ItemTemplate::new("tusks", "Tusks", ItemType::Crafting, 20, 5));

        // American Alligator
        self.templates.push(ItemTemplate::new("alligator_hide", "Alligator Hide", ItemType::Material, 40, 10));
        self.templates.push(ItemTemplate::new("alligator_meat", "Alligator Meat", ItemType::Food, 22, 20));

        // Snakes (Rattlesnake, Copperhead, Cottonmouth)
        self.templates.push(ItemTemplate::new("snake_skin", "Snake Skin", ItemType::Material, 10, 10));
        self.templates.push(ItemTemplate::new("venom_gland", "Venom Gland", ItemType::Crafting, 25, 5));
        self.templates.push(ItemTemplate::new("rattles", "Rattlesnake Rattles", ItemType::Crafting, 15, 10));

        // Bobcat
        self.templates.push(ItemTemplate::new("bobcat_pelt", "Bobcat Pelt", ItemType::Material, 22, 10));
        self.templates.push(ItemTemplate::new("bobcat_meat", "Bobcat Meat", ItemType::Food, 12, 20));

        // Shared animal parts
        self.templates.push(ItemTemplate::new("claws", "Claws", ItemType::Crafting, 12, 10));
        self.templates.push(ItemTemplate::new("fangs", "Fangs", ItemType::Crafting, 15, 10));

        // =========== ADDITIONAL GAME (future expansion) ===========

        // Deer/Elk drops
        self.templates.push(ItemTemplate::new("deer_hide", "Deer Hide", ItemType::Material, 18, 10));
        self.templates.push(ItemTemplate::new("venison", "Venison", ItemType::Food, 12, 20));
        self.templates.push(ItemTemplate::new("antlers", "Antlers", ItemType::Crafting, 25, 5));

        // Small game
        self.templates.push(ItemTemplate::new("rabbit_pelt", "Rabbit Pelt", ItemType::Material, 5, 20));
        self.templates.push(ItemTemplate::new("rabbit_meat", "Rabbit Meat", ItemType::Food, 4, 20));
        self.templates.push(ItemTemplate::new("turkey_feathers", "Turkey Feathers", ItemType::Crafting, 3, 50));
        self.templates.push(ItemTemplate::new("turkey_meat", "Turkey Meat", ItemType::Food, 8, 20));

        // Weapons
        self.templates.push(ItemTemplate::weapon("hunting_bow", "Hunting Bow", 50, 100));
        self.templates.push(ItemTemplate::weapon("war_bow", "War Bow", 80, 150));
        self.templates.push(ItemTemplate::weapon("skinning_knife", "Skinning Knife", 25, 80));
        self.templates.push(ItemTemplate::weapon("tomahawk", "Tomahawk", 60, 120));
        self.templates.push(ItemTemplate::weapon("war_club", "War Club", 40, 100));
        self.templates.push(ItemTemplate::weapon("spear", "Spear", 45, 100));
        self.templates.push(ItemTemplate::weapon("atlatl", "Atlatl", 55, 90));
        self.templates.push(ItemTemplate::weapon("blowgun", "Blowgun", 35, 60));

        // Fossils
        self.templates.push(ItemTemplate::new("megalodon_tooth_small", "Megalodon Tooth (Small)", ItemType::Fossil, 50, 1));
        self.templates.push(ItemTemplate::new("megalodon_tooth_large", "Megalodon Tooth (Large)", ItemType::Fossil, 150, 1));
        self.templates.push(ItemTemplate::new("mastodon_bone", "Mastodon Bone", ItemType::Fossil, 100, 1));
        self.templates.push(ItemTemplate::new("trilobite", "Trilobite", ItemType::Fossil, 200, 1));
        self.templates.push(ItemTemplate::new("ammonite", "Ammonite", ItemType::Fossil, 75, 1));
        self.templates.push(ItemTemplate::new("shark_tooth_fossil", "Shark Tooth Fossil", ItemType::Fossil, 30, 5));

        // Artifacts
        self.templates.push(ItemTemplate::new("arrowhead_stone", "Stone Arrowhead", ItemType::Artifact, 15, 10));
        self.templates.push(ItemTemplate::new("pottery_shard", "Pottery Shard", ItemType::Artifact, 10, 20));
        self.templates.push(ItemTemplate::new("bone_tool", "Bone Tool", ItemType::Artifact, 25, 5));
        self.templates.push(ItemTemplate::new("shell_bead", "Shell Bead", ItemType::Artifact, 8, 50));
    }

    /// Get a template by ID
    pub fn get_template(&self, template_id: &str) -> Option<&ItemTemplate> {
        self.templates.iter().find(|t| t.id == template_id)
    }

    /// Get all templates
    pub fn all_templates(&self) -> &[ItemTemplate] {
        &self.templates
    }
}

/// Prefix definition
struct PrefixDef {
    id: String,
    name: String,
    tier: u8,
    min_rarity: Rarity,
    effects: Vec<ItemEffect>,
}

impl PrefixDef {
    fn new(id: &str, name: &str, tier: u8, min_rarity: Rarity, effects: Vec<ItemEffect>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            tier,
            min_rarity,
            effects,
        }
    }

    fn to_prefix(&self) -> ItemPrefix {
        ItemPrefix {
            id: self.id.clone(),
            name: self.name.clone(),
            tier: self.tier,
            effects: self.effects.clone(),
        }
    }
}

/// Suffix definition
struct SuffixDef {
    id: String,
    name: String,
    tier: u8,
    min_rarity: Rarity,
    effects: Vec<ItemEffect>,
}

impl SuffixDef {
    fn new(id: &str, name: &str, tier: u8, min_rarity: Rarity, effects: Vec<ItemEffect>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            tier,
            min_rarity,
            effects,
        }
    }

    fn to_suffix(&self) -> ItemSuffix {
        ItemSuffix {
            id: self.id.clone(),
            name: self.name.clone(),
            tier: self.tier,
            effects: self.effects.clone(),
        }
    }
}

/// Item template for generation
pub struct ItemTemplate {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub base_value: u32,
    pub max_stack: u32,
    pub durability: Option<u32>,
}

impl ItemTemplate {
    pub fn new(id: &str, name: &str, item_type: ItemType, base_value: u32, max_stack: u32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            item_type,
            base_value,
            max_stack,
            durability: None,
        }
    }

    pub fn weapon(id: &str, name: &str, base_value: u32, durability: u32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            item_type: ItemType::Weapon,
            base_value,
            max_stack: 1,
            durability: Some(durability),
        }
    }

    pub fn instantiate(&self, rarity: Rarity, quality: Quality) -> Item {
        let mut item = Item::new(&self.id, &self.name, self.item_type, self.base_value);
        item.rarity = rarity;
        item.quality = quality;
        item.max_stack = self.max_stack;
        item.stack_size = 1;

        if let Some(dur) = self.durability {
            item.durability = Some(Durability::new(dur));
        }

        item
    }
}

/// Reward from a drop
#[derive(Debug, Clone)]
pub struct DropReward {
    pub wampum: u64,
    pub tobacco: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loot_generation() {
        let gen = LootGenerator::new();
        let ctx = LootContext::default();
        let mut pity = PityTracker::default();

        // Generate 100 drops
        let mut rarity_counts = std::collections::HashMap::new();
        for _ in 0..100 {
            if let Some((item, reward)) = gen.generate_drop("bear_pelt", &ctx, &mut pity) {
                *rarity_counts.entry(item.rarity).or_insert(0) += 1;
                assert!(reward.wampum > 0);
            }
        }

        // Should have mostly common/crude
        assert!(rarity_counts.get(&Rarity::Crude).unwrap_or(&0) +
                rarity_counts.get(&Rarity::Common).unwrap_or(&0) > 50);
    }

    #[test]
    fn test_pity_system() {
        let mut pity = PityTracker::default();

        // Simulate many low-rarity drops
        for _ in 0..50 {
            pity.record_drop(Rarity::Crude);
        }

        // Pity should trigger rare
        assert_eq!(pity.check_pity(), Some(Rarity::Rare));
    }

    #[test]
    fn test_item_value_calculation() {
        let mut item = Item::new("test", "Test Item", ItemType::Material, 100);
        item.rarity = Rarity::Rare;
        item.quality = Quality(75);

        let value = item.calculate_value();
        // Base 100 * Rare 10.0 * Quality 1.25 = 1250
        assert!(value > 1000);
    }
}
