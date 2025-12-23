//! Item System
//!
//! Every item in the game is represented by an Item struct with full provenance.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique identifier for an item instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

impl ItemId {
    pub fn generate() -> Self {
        // Combine timestamp with random component
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let random = rand::random::<u64>();
        Self(timestamp ^ random)
    }
}

/// Item rarity tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Crude = 0,
    Common = 1,
    Uncommon = 2,
    Rare = 3,
    Epic = 4,
    Legendary = 5,
    Mythic = 6,
    Primordial = 7,
}

impl Rarity {
    /// Get the color for this rarity (RGB)
    pub fn color(&self) -> [f32; 3] {
        match self {
            Self::Crude => [0.5, 0.5, 0.5],      // Gray
            Self::Common => [1.0, 1.0, 1.0],     // White
            Self::Uncommon => [0.2, 0.8, 0.2],   // Green
            Self::Rare => [0.2, 0.4, 1.0],       // Blue
            Self::Epic => [0.6, 0.2, 0.8],       // Purple
            Self::Legendary => [1.0, 0.6, 0.0],  // Orange
            Self::Mythic => [1.0, 0.2, 0.2],     // Red
            Self::Primordial => [0.1, 0.1, 0.1], // Black/Gold
        }
    }

    /// Base value multiplier
    pub fn value_multiplier(&self) -> f32 {
        match self {
            Self::Crude => 0.25,
            Self::Common => 1.0,
            Self::Uncommon => 3.0,
            Self::Rare => 10.0,
            Self::Epic => 35.0,
            Self::Legendary => 150.0,
            Self::Mythic => 750.0,
            Self::Primordial => 5000.0,
        }
    }

    /// Drop rate for this rarity (base, before modifiers)
    pub fn base_drop_rate(&self) -> f64 {
        match self {
            Self::Crude => 0.45,
            Self::Common => 0.30,
            Self::Uncommon => 0.15,
            Self::Rare => 0.065,
            Self::Epic => 0.025,
            Self::Legendary => 0.008,
            Self::Mythic => 0.0018,
            Self::Primordial => 0.0002,
        }
    }
}

/// Item quality (0-100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quality(pub u8);

impl Quality {
    pub fn new(value: u8) -> Self {
        Self(value.min(100))
    }

    /// Quality descriptor
    pub fn descriptor(&self) -> &'static str {
        match self.0 {
            0..=10 => "Ruined",
            11..=25 => "Poor",
            26..=40 => "Adequate",
            41..=60 => "Fine",
            61..=80 => "Superior",
            81..=95 => "Exceptional",
            96..=100 => "Perfect",
            _ => "Unknown",
        }
    }

    /// Stat multiplier (0.5 to 1.5)
    pub fn stat_multiplier(&self) -> f32 {
        0.5 + (self.0 as f32 / 100.0)
    }
}

impl Default for Quality {
    fn default() -> Self {
        Self(50)
    }
}

/// Item categories (matches existing ItemCategory in trading.rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    // Equipment
    Weapon,
    Armor,
    Tool,
    Accessory,

    // Consumables
    Food,
    Medicine,
    Consumable,

    // Materials
    Material,
    Crafting,
    Ammo,

    // Special
    Artifact,
    Fossil,
    Quest,
    Currency,
}

/// The core item instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique instance ID
    pub id: ItemId,

    /// Base template ID (e.g., "bear_pelt", "hunting_bow")
    pub template_id: String,

    /// Display name (can be customized for named items)
    pub name: String,

    /// Item type
    pub item_type: ItemType,

    /// Rarity tier
    pub rarity: Rarity,

    /// Quality score (0-100)
    pub quality: Quality,

    /// Current durability (None = no durability)
    pub durability: Option<Durability>,

    /// Stack size (1 for non-stackable)
    pub stack_size: u32,

    /// Maximum stack size
    pub max_stack: u32,

    /// Base value in Wampum
    pub base_value: u32,

    /// Optional prefix modifier
    pub prefix: Option<ItemPrefix>,

    /// Optional suffix modifier
    pub suffix: Option<ItemSuffix>,

    /// Full provenance chain
    pub provenance: ItemProvenance,

    /// Custom data (JSON for flexibility)
    pub custom_data: Option<String>,

    /// Is this item tradeable?
    pub tradeable: bool,

    /// Binding type
    pub binding: BindingType,
}

impl Item {
    /// Create a new item from a template
    pub fn new(template_id: &str, name: &str, item_type: ItemType, base_value: u32) -> Self {
        Self {
            id: ItemId::generate(),
            template_id: template_id.to_string(),
            name: name.to_string(),
            item_type,
            rarity: Rarity::Common,
            quality: Quality::default(),
            durability: None,
            stack_size: 1,
            max_stack: 1,
            base_value,
            prefix: None,
            suffix: None,
            provenance: ItemProvenance::new(),
            custom_data: None,
            tradeable: true,
            binding: BindingType::Unbound,
        }
    }

    /// Calculate total value considering all modifiers
    pub fn calculate_value(&self) -> u32 {
        let base = self.base_value as f32;
        let rarity_mult = self.rarity.value_multiplier();
        let quality_mult = self.quality.stat_multiplier();

        let prefix_mult = self.prefix.as_ref().map(|p| p.value_multiplier()).unwrap_or(1.0);
        let suffix_mult = self.suffix.as_ref().map(|s| s.value_multiplier()).unwrap_or(1.0);

        let provenance_mult = self.provenance.value_multiplier();

        (base * rarity_mult * quality_mult * prefix_mult * suffix_mult * provenance_mult) as u32
    }

    /// Get full display name with prefix/suffix
    pub fn full_name(&self) -> String {
        let mut name = String::new();

        if let Some(prefix) = &self.prefix {
            name.push_str(&prefix.name);
            name.push(' ');
        }

        name.push_str(&self.name);

        if let Some(suffix) = &self.suffix {
            name.push_str(" of ");
            name.push_str(&suffix.name);
        }

        name
    }

    /// Check if item is at maximum durability
    pub fn is_pristine(&self) -> bool {
        self.durability.as_ref().map(|d| d.current == d.maximum).unwrap_or(true)
    }

    /// Apply durability damage
    pub fn damage(&mut self, amount: u32) -> bool {
        if let Some(dur) = &mut self.durability {
            dur.current = dur.current.saturating_sub(amount);
            dur.current == 0
        } else {
            false
        }
    }

    /// Repair item (returns true if repair was possible)
    pub fn repair(&mut self, skill_level: u32) -> bool {
        if let Some(dur) = &mut self.durability {
            if dur.repair_count >= 10 {
                return false; // Too many repairs
            }

            let repair_amount = 20 + (skill_level / 5);
            dur.current = (dur.current + repair_amount).min(dur.maximum);

            // Each repair reduces max durability
            let degradation = 1 + (dur.repair_count / 3);
            dur.maximum = dur.maximum.saturating_sub(degradation);
            dur.repair_count += 1;

            true
        } else {
            false
        }
    }
}

/// Durability tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Durability {
    pub current: u32,
    pub maximum: u32,
    pub original_maximum: u32,
    pub repair_count: u32,
}

impl Durability {
    pub fn new(max: u32) -> Self {
        Self {
            current: max,
            maximum: max,
            original_maximum: max,
            repair_count: 0,
        }
    }

    pub fn percent(&self) -> f32 {
        if self.maximum == 0 {
            0.0
        } else {
            self.current as f32 / self.maximum as f32
        }
    }
}

/// Item binding types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingType {
    Unbound,           // Freely tradeable
    BindOnEquip,       // Becomes bound when equipped
    BindOnPickup,      // Bound immediately
    BindOnUse,         // Bound after first use
    AccountBound,      // Tradeable between own characters
    Soulbound,         // Never tradeable
}

/// Item prefix (magical modifier)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPrefix {
    pub id: String,
    pub name: String,
    pub tier: u8,           // 1-4
    pub effects: Vec<ItemEffect>,
}

impl ItemPrefix {
    pub fn value_multiplier(&self) -> f32 {
        1.0 + (self.tier as f32 * 0.25) // +25% per tier
    }
}

/// Item suffix (magical modifier)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSuffix {
    pub id: String,
    pub name: String,
    pub tier: u8,           // 1-4
    pub effects: Vec<ItemEffect>,
}

impl ItemSuffix {
    pub fn value_multiplier(&self) -> f32 {
        1.0 + (self.tier as f32 * 0.20) // +20% per tier
    }
}

/// Stat/effect modifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemEffect {
    // Flat bonuses
    Damage(f32),
    Armor(f32),
    Health(f32),
    Stamina(f32),

    // Percentage bonuses
    DamagePercent(f32),
    AttackSpeed(f32),
    MovementSpeed(f32),
    CritChance(f32),

    // Skill bonuses
    HuntingXp(f32),
    ArchaeologyXp(f32),
    Luck(f32),

    // Special effects
    LifeSteal(f32),
    PoisonDamage(f32),
    BleedDamage(f32),

    // Resource bonuses
    GatherYield(f32),
    GoldFind(f32),
    DropRateBonus(f32),
}

/// Full provenance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemProvenance {
    /// Player who first obtained this item
    pub first_owner_id: Option<u64>,
    pub first_owner_name: Option<String>,

    /// When item was created (Unix timestamp)
    pub created_at: u64,

    /// Where item was obtained
    pub origin_location: Option<[f32; 3]>,

    /// How item was obtained
    pub origin_method: OriginMethod,

    /// Kill details (for hunting drops)
    pub kill_record: Option<KillRecord>,

    /// Trade history (last 10 trades)
    pub trade_history: Vec<TradeRecord>,

    /// Total value traded over lifetime
    pub lifetime_trade_value: u64,

    /// Number of times traded
    pub times_traded: u32,

    /// Special flags
    pub is_first_of_type: bool,     // First of this type on server
    pub is_world_first: bool,       // World first achievement
    pub is_event_drop: bool,        // From special event
    pub event_name: Option<String>,
}

impl ItemProvenance {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            first_owner_id: None,
            first_owner_name: None,
            created_at: now,
            origin_location: None,
            origin_method: OriginMethod::Unknown,
            kill_record: None,
            trade_history: Vec::new(),
            lifetime_trade_value: 0,
            times_traded: 0,
            is_first_of_type: false,
            is_world_first: false,
            is_event_drop: false,
            event_name: None,
        }
    }

    /// Set the first owner
    pub fn set_first_owner(&mut self, player_id: u64, player_name: &str) {
        if self.first_owner_id.is_none() {
            self.first_owner_id = Some(player_id);
            self.first_owner_name = Some(player_name.to_string());
        }
    }

    /// Record a trade
    pub fn record_trade(&mut self, buyer_id: u64, buyer_name: &str, price: u64) {
        self.trade_history.push(TradeRecord {
            buyer_id,
            buyer_name: buyer_name.to_string(),
            price,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // Keep only last 10
        if self.trade_history.len() > 10 {
            self.trade_history.remove(0);
        }

        self.times_traded += 1;
        self.lifetime_trade_value += price;
    }

    /// Calculate provenance value multiplier
    pub fn value_multiplier(&self) -> f32 {
        let mut mult = 1.0;

        // First of type bonus
        if self.is_first_of_type {
            mult += 0.25;
        }

        // World first bonus
        if self.is_world_first {
            mult += 0.50;
        }

        // Event drop bonus
        if self.is_event_drop {
            mult += 0.10;
        }

        // Perfect kill bonus
        if let Some(kill) = &self.kill_record {
            if kill.was_perfect {
                mult += 0.15;
            }
            if kill.was_stealth {
                mult += 0.10;
            }
        }

        // Age bonus (older items slightly more valuable)
        let age_days = self.age_days();
        mult += (age_days as f32 / 365.0) * 0.05; // +5% per year, max +25%
        mult = mult.min(1.5); // Cap total provenance bonus at 50%

        mult
    }

    /// Get age in days
    pub fn age_days(&self) -> u32 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        ((now - self.created_at) / 86400) as u32
    }
}

impl Default for ItemProvenance {
    fn default() -> Self {
        Self::new()
    }
}

/// How an item was obtained
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OriginMethod {
    Unknown,
    HuntingDrop { species: String },
    ArchaeologyDig { site_name: String },
    Crafted { crafter_id: u64 },
    QuestReward { quest_id: String },
    NpcPurchase { npc_id: u32 },
    Trade { from_player_id: u64 },
    WorldDrop { location_name: String },
    EventReward { event_id: String },
    ChestLoot { chest_tier: u32 },
}

/// Kill record for hunting drops
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillRecord {
    pub species: String,
    pub was_stealth: bool,
    pub was_perfect: bool,
    pub was_critical: bool,
    pub weapon_used: String,
    pub kill_time_seconds: f32,
    pub player_health_remaining: f32,
}

/// Trade record for provenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub buyer_id: u64,
    pub buyer_name: String,
    pub price: u64,
    pub timestamp: u64,
}

// ============================================================================
// ITEM TEMPLATES - Common item creation helpers
// ============================================================================

/// Create a stick item (for campfire building)
pub fn create_stick() -> Item {
    let mut item = Item::new("stick", "Stick", ItemType::Material, 1);
    item.rarity = Rarity::Crude;
    item.stack_size = 1;
    item.max_stack = 50;
    item
}

/// Create multiple sticks
pub fn create_sticks(count: u32) -> Item {
    let mut item = create_stick();
    item.stack_size = count.min(item.max_stack);
    item
}

/// Create a pebble item (for campfire ring)
pub fn create_pebble() -> Item {
    let mut item = Item::new("pebble", "Pebble", ItemType::Material, 1);
    item.rarity = Rarity::Crude;
    item.stack_size = 1;
    item.max_stack = 50;
    item
}

/// Create multiple pebbles
pub fn create_pebbles(count: u32) -> Item {
    let mut item = create_pebble();
    item.stack_size = count.min(item.max_stack);
    item
}

/// Create kindling (dry leaves/grass for fire starting)
pub fn create_kindling() -> Item {
    let mut item = Item::new("kindling", "Kindling", ItemType::Material, 1);
    item.rarity = Rarity::Crude;
    item.stack_size = 1;
    item.max_stack = 30;
    item
}
