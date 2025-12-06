# Roanoke Economy: Practical Implementation Guide

**Status:** Ready for immediate implementation
**Prerequisites:** Existing trading.rs, player_state.rs, types.rs systems

This document provides copy-paste ready code to implement the economy system TODAY.

---

## Phase 0: What We Have Now

### Current State Analysis

```
EXISTING SYSTEMS
═══════════════════════════════════════════════════════════════════════════════

✅ Trading System (npc/trading.rs)
   • NPC inventories with items
   • Gold-based pricing
   • Trade history tracking
   • Category system (Weapon, Armor, Tool, Material, etc.)

✅ Player Progression (progression/player_state.rs)
   • Skills and achievements
   • Reputation with factions
   • Event logging
   • Statistics tracking (gold_earned, gold_spent)

✅ Animal System (animals/types.rs)
   • Loot tables per species
   • Drop items defined
   • Danger levels (can map to rarity)

❌ MISSING
   • Inventory system (player doesn't store items)
   • Item instances (items are just strings, not objects)
   • Rarity/quality system
   • Provenance tracking
   • Marketplace (player-to-player trading)
   • Dual currency (only gold exists)
```

---

## Phase 1: Core Item System (Week 1)

### 1.1 Create Item Data Structures

**New file:** `roanoke_game/src/economy/mod.rs`

```rust
//! Roanoke Economy System
//!
//! Core economic infrastructure for items, currency, and trading.

pub mod item;
pub mod inventory;
pub mod currency;
pub mod loot;
pub mod marketplace;

pub use item::*;
pub use inventory::*;
pub use currency::*;
pub use loot::*;
```

**New file:** `roanoke_game/src/economy/item.rs`

```rust
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
        let random = fastrand::u64(..);
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
```

### 1.2 Create Inventory System

**New file:** `roanoke_game/src/economy/inventory.rs`

```rust
//! Player Inventory System
//!
//! Manages player's item storage with slots and equipment.

use super::item::{Item, ItemId, ItemType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum inventory slots
pub const MAX_INVENTORY_SLOTS: usize = 40;
/// Maximum stash slots
pub const MAX_STASH_SLOTS: usize = 100;

/// Equipment slots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipSlot {
    MainHand,
    OffHand,
    Head,
    Chest,
    Legs,
    Feet,
    Hands,
    Accessory1,
    Accessory2,
    Ammo,
}

/// Player inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// Inventory slots (carried items)
    pub slots: Vec<Option<Item>>,

    /// Currently equipped items
    pub equipped: HashMap<EquipSlot, Item>,

    /// Stash storage (home base)
    pub stash: Vec<Option<Item>>,

    /// Quick-access item lookup
    #[serde(skip)]
    item_index: HashMap<ItemId, usize>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: vec![None; MAX_INVENTORY_SLOTS],
            equipped: HashMap::new(),
            stash: vec![None; MAX_STASH_SLOTS],
            item_index: HashMap::new(),
        }
    }

    /// Rebuild the item index
    pub fn rebuild_index(&mut self) {
        self.item_index.clear();
        for (idx, slot) in self.slots.iter().enumerate() {
            if let Some(item) = slot {
                self.item_index.insert(item.id, idx);
            }
        }
    }

    /// Add an item to inventory
    pub fn add_item(&mut self, item: Item) -> Result<usize, InventoryError> {
        // Try to stack with existing
        if item.max_stack > 1 {
            for slot in &mut self.slots {
                if let Some(existing) = slot {
                    if existing.template_id == item.template_id
                        && existing.rarity == item.rarity
                        && existing.stack_size < existing.max_stack
                    {
                        let can_add = existing.max_stack - existing.stack_size;
                        let to_add = item.stack_size.min(can_add);
                        existing.stack_size += to_add;

                        if to_add == item.stack_size {
                            return Ok(self.slots.iter().position(|s| {
                                s.as_ref().map(|i| i.id == existing.id).unwrap_or(false)
                            }).unwrap());
                        }
                    }
                }
            }
        }

        // Find empty slot
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                self.item_index.insert(item.id, idx);
                *slot = Some(item);
                return Ok(idx);
            }
        }

        Err(InventoryError::Full)
    }

    /// Remove an item by ID
    pub fn remove_item(&mut self, id: ItemId) -> Option<Item> {
        if let Some(&idx) = self.item_index.get(&id) {
            self.item_index.remove(&id);
            self.slots[idx].take()
        } else {
            None
        }
    }

    /// Get an item by ID
    pub fn get_item(&self, id: ItemId) -> Option<&Item> {
        self.item_index.get(&id)
            .and_then(|&idx| self.slots[idx].as_ref())
    }

    /// Get mutable item by ID
    pub fn get_item_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        if let Some(&idx) = self.item_index.get(&id) {
            self.slots[idx].as_mut()
        } else {
            None
        }
    }

    /// Equip an item
    pub fn equip(&mut self, id: ItemId, slot: EquipSlot) -> Result<Option<Item>, InventoryError> {
        let item = self.remove_item(id).ok_or(InventoryError::ItemNotFound)?;

        // Validate item can go in slot
        if !can_equip_in_slot(&item, slot) {
            // Put it back
            self.add_item(item)?;
            return Err(InventoryError::InvalidSlot);
        }

        // Unequip existing
        let previous = self.equipped.remove(&slot);

        // Equip new
        self.equipped.insert(slot, item);

        // Add previous to inventory if exists
        if let Some(prev) = previous {
            self.add_item(prev)?;
        }

        Ok(self.equipped.get(&slot).cloned())
    }

    /// Unequip an item
    pub fn unequip(&mut self, slot: EquipSlot) -> Result<(), InventoryError> {
        let item = self.equipped.remove(&slot).ok_or(InventoryError::ItemNotFound)?;
        self.add_item(item)?;
        Ok(())
    }

    /// Get equipped item in slot
    pub fn get_equipped(&self, slot: EquipSlot) -> Option<&Item> {
        self.equipped.get(&slot)
    }

    /// Count items by template ID
    pub fn count_items(&self, template_id: &str) -> u32 {
        self.slots.iter()
            .filter_map(|s| s.as_ref())
            .filter(|i| i.template_id == template_id)
            .map(|i| i.stack_size)
            .sum()
    }

    /// Get total inventory value
    pub fn total_value(&self) -> u64 {
        let inventory_value: u64 = self.slots.iter()
            .filter_map(|s| s.as_ref())
            .map(|i| i.calculate_value() as u64 * i.stack_size as u64)
            .sum();

        let equipped_value: u64 = self.equipped.values()
            .map(|i| i.calculate_value() as u64)
            .sum();

        inventory_value + equipped_value
    }

    /// Get items by type
    pub fn items_by_type(&self, item_type: ItemType) -> Vec<&Item> {
        self.slots.iter()
            .filter_map(|s| s.as_ref())
            .filter(|i| i.item_type == item_type)
            .collect()
    }

    /// Move item to stash
    pub fn stash_item(&mut self, id: ItemId) -> Result<(), InventoryError> {
        let item = self.remove_item(id).ok_or(InventoryError::ItemNotFound)?;

        for slot in &mut self.stash {
            if slot.is_none() {
                *slot = Some(item);
                return Ok(());
            }
        }

        // Stash full, put back in inventory
        self.add_item(item)?;
        Err(InventoryError::StashFull)
    }

    /// Retrieve item from stash
    pub fn unstash_item(&mut self, stash_index: usize) -> Result<(), InventoryError> {
        if stash_index >= self.stash.len() {
            return Err(InventoryError::ItemNotFound);
        }

        let item = self.stash[stash_index].take().ok_or(InventoryError::ItemNotFound)?;
        self.add_item(item)?;
        Ok(())
    }

    /// Get number of free slots
    pub fn free_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_none()).count()
    }

    /// Check if inventory has room
    pub fn has_room(&self) -> bool {
        self.free_slots() > 0
    }
}

/// Check if an item can be equipped in a slot
fn can_equip_in_slot(item: &Item, slot: EquipSlot) -> bool {
    match (item.item_type, slot) {
        (ItemType::Weapon, EquipSlot::MainHand | EquipSlot::OffHand) => true,
        (ItemType::Armor, EquipSlot::Head | EquipSlot::Chest | EquipSlot::Legs | EquipSlot::Feet | EquipSlot::Hands) => true,
        (ItemType::Accessory, EquipSlot::Accessory1 | EquipSlot::Accessory2) => true,
        (ItemType::Ammo, EquipSlot::Ammo) => true,
        (ItemType::Tool, EquipSlot::MainHand | EquipSlot::OffHand) => true,
        _ => false,
    }
}

/// Inventory errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryError {
    Full,
    StashFull,
    ItemNotFound,
    InvalidSlot,
    NotStackable,
    CannotDrop,
}
```

### 1.3 Create Currency System

**New file:** `roanoke_game/src/economy/currency.rs`

```rust
//! Dual Currency System
//!
//! Wampum (WPM) - Utility currency, earned through play
//! Tobacco (TBC) - Premium currency, deflationary store of value

use serde::{Deserialize, Serialize};

/// Player wallet containing both currencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wallet {
    /// Wampum - the utility currency
    pub wampum: u64,

    /// Tobacco - the premium currency
    pub tobacco: u64,

    /// Lifetime earnings
    pub lifetime_wampum_earned: u64,
    pub lifetime_tobacco_earned: u64,

    /// Lifetime spending
    pub lifetime_wampum_spent: u64,
    pub lifetime_tobacco_spent: u64,
}

impl Wallet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add wampum to wallet
    pub fn add_wampum(&mut self, amount: u64) {
        self.wampum = self.wampum.saturating_add(amount);
        self.lifetime_wampum_earned = self.lifetime_wampum_earned.saturating_add(amount);
    }

    /// Spend wampum (returns false if insufficient)
    pub fn spend_wampum(&mut self, amount: u64) -> bool {
        if self.wampum >= amount {
            self.wampum -= amount;
            self.lifetime_wampum_spent = self.lifetime_wampum_spent.saturating_add(amount);
            true
        } else {
            false
        }
    }

    /// Add tobacco to wallet
    pub fn add_tobacco(&mut self, amount: u64) {
        self.tobacco = self.tobacco.saturating_add(amount);
        self.lifetime_tobacco_earned = self.lifetime_tobacco_earned.saturating_add(amount);
    }

    /// Spend tobacco (returns false if insufficient)
    pub fn spend_tobacco(&mut self, amount: u64) -> bool {
        if self.tobacco >= amount {
            self.tobacco -= amount;
            self.lifetime_tobacco_spent = self.lifetime_tobacco_spent.saturating_add(amount);
            true
        } else {
            false
        }
    }

    /// Get net worth in wampum-equivalent
    pub fn net_worth(&self, tbc_to_wpm_rate: u64) -> u64 {
        self.wampum + (self.tobacco * tbc_to_wpm_rate)
    }
}

/// Currency conversion rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRates {
    /// How much WPM for 1 TBC
    pub tbc_to_wpm: u64,

    /// Minimum WPM per TBC (floor)
    pub tbc_floor: u64,

    /// Conversion tax (burned, deflationary)
    pub conversion_tax_percent: f32,
}

impl Default for ExchangeRates {
    fn default() -> Self {
        Self {
            tbc_to_wpm: 1000, // 1 TBC = 1000 WPM
            tbc_floor: 500,   // Never below 500 WPM per TBC
            conversion_tax_percent: 5.0, // 5% burned on conversion
        }
    }
}

impl ExchangeRates {
    /// Convert wampum to tobacco
    pub fn wpm_to_tbc(&self, wampum: u64) -> (u64, u64) {
        let before_tax = wampum / self.tbc_to_wpm;
        let tax = (before_tax as f32 * (self.conversion_tax_percent / 100.0)) as u64;
        let after_tax = before_tax.saturating_sub(tax);
        (after_tax, tax)
    }

    /// Convert tobacco to wampum
    pub fn tbc_to_wpm(&self, tobacco: u64) -> (u64, u64) {
        let before_tax = tobacco * self.tbc_to_wpm;
        let tax = (before_tax as f32 * (self.conversion_tax_percent / 100.0)) as u64;
        let after_tax = before_tax.saturating_sub(tax);
        (after_tax, tax)
    }
}

/// Currency transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyTransaction {
    pub transaction_type: TransactionType,
    pub wampum_amount: i64,   // Positive = gained, negative = spent
    pub tobacco_amount: i64,
    pub timestamp: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    LootDrop,
    NpcSale,
    NpcPurchase,
    PlayerTrade,
    CurrencyConversion,
    QuestReward,
    RepairCost,
    FastTravel,
    MarketplaceFee,
    Sacrifice,
}
```

---

## Phase 2: Loot Generation System (Week 2)

### 2.1 Loot Drop System

**New file:** `roanoke_game/src/economy/loot.rs`

```rust
//! Loot Generation System
//!
//! Generates items with rarity, quality, and modifiers based on context.

use super::item::*;
use super::currency::Wallet;
use crate::animals::types::AnimalSpecies;
use serde::{Deserialize, Serialize};

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
    pub species: AnimalSpecies,
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
            Rarity::Legendary | Rarity::Mythic | Rarity::Primordial => {
                self.drops_since_legendary = 0;
                self.lifetime_legendary += 1;
            }
            Rarity::Mythic => {
                self.lifetime_mythic += 1;
            }
            Rarity::Primordial => {
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
                species: kill.species.name().to_string(),
            };
            item.provenance.kill_record = Some(KillRecord {
                species: kill.species.name().to_string(),
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
        let base_roll: f64 = fastrand::f64();

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
        let base_mean = 35.0 + (context.skill_level as f32 * 0.3);
        let luck_bonus = context.luck * 20.0;
        let std_dev = 15.0 - (context.skill_level as f32 * 0.05);

        // Simple normal distribution approximation
        let u1: f32 = fastrand::f32();
        let u2: f32 = fastrand::f32();
        let normal = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();

        let quality = (base_mean + luck_bonus + normal * std_dev).clamp(0.0, 100.0) as u8;
        Quality(quality)
    }

    fn should_have_prefix(&self, rarity: Rarity) -> bool {
        let chance = match rarity {
            Rarity::Crude | Rarity::Common => 0.0,
            Rarity::Uncommon => 0.10,
            Rarity::Rare => 0.30,
            Rarity::Epic => 0.60,
            Rarity::Legendary => 0.90,
            Rarity::Mythic | Rarity::Primordial => 1.0,
        };
        fastrand::f32() < chance
    }

    fn should_have_suffix(&self, rarity: Rarity) -> bool {
        let chance = match rarity {
            Rarity::Crude | Rarity::Common => 0.0,
            Rarity::Uncommon => 0.05,
            Rarity::Rare => 0.20,
            Rarity::Epic => 0.50,
            Rarity::Legendary => 0.80,
            Rarity::Mythic | Rarity::Primordial => 1.0,
        };
        fastrand::f32() < chance
    }

    fn roll_prefix(&self, min_rarity: Rarity) -> Option<ItemPrefix> {
        let eligible: Vec<_> = self.prefixes.iter()
            .filter(|p| p.min_rarity <= min_rarity)
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let idx = fastrand::usize(..eligible.len());
        Some(eligible[idx].to_prefix())
    }

    fn roll_suffix(&self, min_rarity: Rarity) -> Option<ItemSuffix> {
        let eligible: Vec<_> = self.suffixes.iter()
            .filter(|s| s.min_rarity <= min_rarity)
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let idx = fastrand::usize(..eligible.len());
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
        // TBC only drops from high rarity
        match item.rarity {
            Rarity::Legendary if fastrand::f32() < 0.10 => 10,
            Rarity::Mythic if fastrand::f32() < 0.25 => 100,
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
        // Hunting drops
        self.templates.push(ItemTemplate::new("bear_pelt", "Bear Pelt", ItemType::Material, 30, 10));
        self.templates.push(ItemTemplate::new("bear_meat", "Bear Meat", ItemType::Food, 15, 20));
        self.templates.push(ItemTemplate::new("wolf_pelt", "Wolf Pelt", ItemType::Material, 20, 10));
        self.templates.push(ItemTemplate::new("wolf_meat", "Wolf Meat", ItemType::Food, 10, 20));
        self.templates.push(ItemTemplate::new("boar_hide", "Boar Hide", ItemType::Material, 15, 10));
        self.templates.push(ItemTemplate::new("boar_meat", "Boar Meat", ItemType::Food, 8, 20));
        self.templates.push(ItemTemplate::new("alligator_hide", "Alligator Hide", ItemType::Material, 40, 10));
        self.templates.push(ItemTemplate::new("cougar_pelt", "Cougar Pelt", ItemType::Material, 35, 10));
        self.templates.push(ItemTemplate::new("snake_skin", "Snake Skin", ItemType::Material, 10, 10));
        self.templates.push(ItemTemplate::new("venom_gland", "Venom Gland", ItemType::Crafting, 25, 5));
        self.templates.push(ItemTemplate::new("claws", "Claws", ItemType::Crafting, 12, 10));
        self.templates.push(ItemTemplate::new("fangs", "Fangs", ItemType::Crafting, 15, 10));
        self.templates.push(ItemTemplate::new("tusks", "Tusks", ItemType::Crafting, 20, 5));

        // Weapons
        self.templates.push(ItemTemplate::weapon("hunting_bow", "Hunting Bow", 50, 100));
        self.templates.push(ItemTemplate::weapon("war_bow", "War Bow", 80, 150));
        self.templates.push(ItemTemplate::weapon("skinning_knife", "Skinning Knife", 25, 80));
        self.templates.push(ItemTemplate::weapon("tomahawk", "Tomahawk", 60, 120));
        self.templates.push(ItemTemplate::weapon("war_club", "War Club", 40, 100));
        self.templates.push(ItemTemplate::weapon("spear", "Spear", 45, 100));

        // Fossils
        self.templates.push(ItemTemplate::new("megalodon_tooth_small", "Megalodon Tooth (Small)", ItemType::Fossil, 50, 1));
        self.templates.push(ItemTemplate::new("megalodon_tooth_large", "Megalodon Tooth (Large)", ItemType::Fossil, 150, 1));
        self.templates.push(ItemTemplate::new("mastodon_bone", "Mastodon Bone", ItemType::Fossil, 100, 1));
        self.templates.push(ItemTemplate::new("trilobite", "Trilobite", ItemType::Fossil, 200, 1));
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
struct ItemTemplate {
    id: String,
    name: String,
    item_type: ItemType,
    base_value: u32,
    max_stack: u32,
    durability: Option<u32>,
}

impl ItemTemplate {
    fn new(id: &str, name: &str, item_type: ItemType, base_value: u32, max_stack: u32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            item_type,
            base_value,
            max_stack,
            durability: None,
        }
    }

    fn weapon(id: &str, name: &str, base_value: u32, durability: u32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            item_type: ItemType::Weapon,
            base_value,
            max_stack: 1,
            durability: Some(durability),
        }
    }

    fn instantiate(&self, rarity: Rarity, quality: Quality) -> Item {
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
```

---

## Phase 3: Integration Points (Week 3)

### 3.1 Update Player State

**Edit:** `roanoke_game/src/progression/player_state.rs`

Add to PlayerProgression struct:
```rust
use crate::economy::{Inventory, Wallet, PityTracker};

// Add these fields to PlayerProgression:
pub inventory: Inventory,
pub wallet: Wallet,
pub pity_tracker: PityTracker,
```

### 3.2 Hook into Animal Combat

**Edit:** `roanoke_game/src/animals/combat.rs`

On animal death:
```rust
use crate::economy::{LootGenerator, LootContext, KillContext};

fn on_animal_killed(
    species: AnimalSpecies,
    player: &mut PlayerProgression,
    was_stealth: bool,
    was_perfect: bool,
    was_critical: bool,
    weapon: &str,
    kill_time: f32,
    player_health: f32,
) {
    let loot_gen = LootGenerator::new();

    let context = LootContext {
        luck: player.calculate_luck(),
        skill_level: player.hunting.effective_level(),
        session_minutes: player.current_session_minutes(),
        karma: player.pity_tracker.karma_accumulated,
        kill_context: Some(KillContext {
            species,
            was_stealth,
            was_perfect,
            was_critical,
            weapon_used: weapon.to_string(),
            kill_time_seconds: kill_time,
            player_health_remaining: player_health,
        }),
        moon_phase: get_moon_phase(),
        weather_modifier: get_weather_modifier(),
        event_multiplier: get_active_event_multiplier(),
    };

    // Generate drops for each loot item
    for loot_id in species.loot() {
        if let Some((item, reward)) = loot_gen.generate_drop(loot_id, &context, &mut player.pity_tracker) {
            // Add to inventory
            if player.inventory.add_item(item).is_ok() {
                // Add currency rewards
                player.wallet.add_wampum(reward.wampum);
                player.wallet.add_tobacco(reward.tobacco);
            }
        }
    }
}
```

### 3.3 Add to main.rs Game State

```rust
use crate::economy::{LootGenerator, Inventory, Wallet};

// In your main GameState struct:
pub struct GameState {
    // ... existing fields ...
    pub loot_generator: LootGenerator,
}

// Initialize in new():
loot_generator: LootGenerator::new(),
```

---

## Phase 4: Immediate Next Steps

### This Week's Tasks

1. **Create directory structure:**
   ```bash
   mkdir -p roanoke_game/src/economy
   ```

2. **Create the module files:**
   - `roanoke_game/src/economy/mod.rs`
   - `roanoke_game/src/economy/item.rs`
   - `roanoke_game/src/economy/inventory.rs`
   - `roanoke_game/src/economy/currency.rs`
   - `roanoke_game/src/economy/loot.rs`

3. **Add to Cargo.toml:**
   ```toml
   [dependencies]
   fastrand = "2.0"  # Fast PRNG for loot rolls
   ```

4. **Add module to main:**
   ```rust
   // In roanoke_game/src/main.rs or lib.rs
   mod economy;
   ```

5. **Test the loot generator:**
   ```rust
   #[test]
   fn test_loot_generation() {
       let gen = LootGenerator::new();
       let ctx = LootContext::default();
       let mut pity = PityTracker::default();

       for _ in 0..100 {
           if let Some((item, reward)) = gen.generate_drop("bear_pelt", &ctx, &mut pity) {
               println!("{} ({:?}) - {} WPM", item.full_name(), item.rarity, reward.wampum);
           }
       }
   }
   ```

---

## Summary: Implementation Priority

```
IMMEDIATE (This Week)
═══════════════════════════════════════════════════════════════════════════════
□ Create economy module directory
□ Implement Item and ItemId structs
□ Implement Inventory system
□ Implement Wallet (dual currency)
□ Implement basic LootGenerator

NEXT WEEK
═══════════════════════════════════════════════════════════════════════════════
□ Hook loot generation to animal kills
□ Add inventory UI
□ Add wallet display
□ Save/load inventory state

WEEK 3
═══════════════════════════════════════════════════════════════════════════════
□ NPC trading integration
□ Item comparison UI
□ Drop notifications
□ Rarity visual effects

WEEK 4
═══════════════════════════════════════════════════════════════════════════════
□ Crafting system
□ Item upgrade/repair
□ Marketplace foundation
□ Player-to-player trading
```

---

*Document Version: 1.0*
*Ready for Implementation: Yes*
*Estimated Total Time: 4 weeks to MVP*
