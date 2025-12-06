//! Trading System
//!
//! Handles NPC shops, bartering, and economic interactions.

use crate::progression::reputation::{Faction, ReputationLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trading system manager
#[derive(Debug, Clone, Default)]
pub struct TradingSystem {
    /// NPC inventories
    pub inventories: HashMap<u32, TradeInventory>,
    /// Active trade session
    pub active_trade: Option<ActiveTrade>,
    /// Trade history
    pub history: Vec<TradeRecord>,
}

impl TradingSystem {
    pub fn new() -> Self {
        let mut system = Self::default();
        system.initialize_inventories();
        system
    }

    /// Initialize NPC trade inventories
    fn initialize_inventories(&mut self) {
        // Village Elder - Basic supplies
        self.inventories.insert(1, TradeInventory {
            npc_id: 1,
            name: "Village Elder's Goods".to_string(),
            items: vec![
                TradeItem::new("dried_meat", "Dried Venison", 5, 3, ItemCategory::Food),
                TradeItem::new("healing_herbs", "Healing Herbs", 10, 5, ItemCategory::Medicine),
                TradeItem::new("water_skin", "Water Skin", 15, 2, ItemCategory::Equipment),
                TradeItem::new("flint", "Flint Stones", 3, 10, ItemCategory::Crafting),
            ],
            restocks_daily: true,
            faction: Faction::NativeCouncil,
            required_reputation: ReputationLevel::Neutral,
        });

        // Hunter NPC - Hunting gear
        self.inventories.insert(5, TradeInventory {
            npc_id: 5,
            name: "Hunter's Supplies".to_string(),
            items: vec![
                TradeItem::new("hunting_bow", "Hunting Bow", 50, 1, ItemCategory::Weapon),
                TradeItem::new("arrows", "Bone Arrows", 2, 20, ItemCategory::Ammo),
                TradeItem::new("skinning_knife", "Skinning Knife", 25, 2, ItemCategory::Tool),
                TradeItem::new("trap_kit", "Simple Snare Kit", 15, 5, ItemCategory::Tool),
                TradeItem::new("bait_meat", "Bait (Raw Meat)", 3, 10, ItemCategory::Consumable),
                TradeItem::new("deer_call", "Deer Call", 20, 1, ItemCategory::Tool),
                TradeItem::new("wolf_pelt", "Wolf Pelt", 30, 0, ItemCategory::Material), // Buying only
            ],
            restocks_daily: true,
            faction: Faction::Hunters,
            required_reputation: ReputationLevel::Friendly,
        });

        // Shaman - Mystical items
        self.inventories.insert(3, TradeInventory {
            npc_id: 3,
            name: "Shaman's Sacred Items".to_string(),
            items: vec![
                TradeItem::new("antivenom", "Antivenom Poultice", 25, 3, ItemCategory::Medicine),
                TradeItem::new("spirit_totem", "Spirit Totem", 100, 1, ItemCategory::Artifact),
                TradeItem::new("bone_talisman", "Bone Talisman", 75, 2, ItemCategory::Artifact),
                TradeItem::new("tongue_stone_amulet", "Tongue Stone Amulet", 150, 1, ItemCategory::Artifact),
                TradeItem::new("healing_salve", "Greater Healing Salve", 40, 2, ItemCategory::Medicine),
                // Buying fossils
                TradeItem::new("megalodon_tooth_small", "Megalodon Tooth (Small)", 20, 0, ItemCategory::Fossil),
                TradeItem::new("megalodon_tooth_large", "Megalodon Tooth (Large)", 80, 0, ItemCategory::Fossil),
                TradeItem::new("mastodon_bone", "Mastodon Bone", 50, 0, ItemCategory::Fossil),
            ],
            restocks_daily: false, // Special items don't restock
            faction: Faction::Shamans,
            required_reputation: ReputationLevel::Friendly,
        });

        // Trader NPC - General goods and currency exchange
        self.inventories.insert(6, TradeInventory {
            npc_id: 6,
            name: "Trader's Wares".to_string(),
            items: vec![
                TradeItem::new("rope", "Hemp Rope", 5, 10, ItemCategory::Crafting),
                TradeItem::new("iron_ingot", "Iron Ingot", 20, 5, ItemCategory::Crafting),
                TradeItem::new("cloth", "Woven Cloth", 8, 8, ItemCategory::Crafting),
                TradeItem::new("torch", "Torch", 3, 10, ItemCategory::Equipment),
                TradeItem::new("backpack", "Leather Backpack", 35, 2, ItemCategory::Equipment),
                TradeItem::new("bedroll", "Bedroll", 20, 2, ItemCategory::Equipment),
                // European goods (rare)
                TradeItem::new("steel_knife", "Steel Knife", 100, 1, ItemCategory::Weapon),
                TradeItem::new("musket", "Musket", 500, 1, ItemCategory::Weapon),
                TradeItem::new("gunpowder", "Gunpowder", 50, 3, ItemCategory::Ammo),
            ],
            restocks_daily: true,
            faction: Faction::Traders,
            required_reputation: ReputationLevel::Neutral,
        });

        // Warrior Chief - Military equipment
        self.inventories.insert(2, TradeInventory {
            npc_id: 2,
            name: "War Chief's Arsenal".to_string(),
            items: vec![
                TradeItem::new("war_club", "War Club", 40, 2, ItemCategory::Weapon),
                TradeItem::new("tomahawk", "Tomahawk", 60, 2, ItemCategory::Weapon),
                TradeItem::new("war_bow", "War Bow", 80, 1, ItemCategory::Weapon),
                TradeItem::new("war_arrows", "Flint War Arrows", 4, 15, ItemCategory::Ammo),
                TradeItem::new("leather_armor", "Leather Armor", 100, 1, ItemCategory::Armor),
                TradeItem::new("war_paint", "War Paint", 15, 5, ItemCategory::Consumable),
            ],
            restocks_daily: false,
            faction: Faction::Warriors,
            required_reputation: ReputationLevel::Respected,
        });
    }

    /// Start a trade with an NPC
    pub fn start_trade(&mut self, npc_id: u32, player_reputation: ReputationLevel) -> Result<&TradeInventory, &'static str> {
        let inventory = self.inventories.get(&npc_id).ok_or("NPC doesn't trade")?;

        if player_reputation < inventory.required_reputation {
            return Err("Insufficient reputation to trade");
        }

        self.active_trade = Some(ActiveTrade {
            npc_id,
            player_offer: Vec::new(),
            npc_offer: Vec::new(),
            player_gold: 0,
            npc_gold: 0,
        });

        Ok(inventory)
    }

    /// Add item to player's offer
    pub fn add_to_player_offer(&mut self, item: String, quantity: u32) {
        if let Some(trade) = &mut self.active_trade {
            if let Some(existing) = trade.player_offer.iter_mut().find(|(i, _)| i == &item) {
                existing.1 += quantity;
            } else {
                trade.player_offer.push((item, quantity));
            }
        }
    }

    /// Add item to NPC's offer (what player is buying)
    pub fn add_to_npc_offer(&mut self, item: String, quantity: u32) {
        if let Some(trade) = &mut self.active_trade {
            if let Some(existing) = trade.npc_offer.iter_mut().find(|(i, _)| i == &item) {
                existing.1 += quantity;
            } else {
                trade.npc_offer.push((item, quantity));
            }
        }
    }

    /// Calculate trade value
    pub fn calculate_trade_value(&self, reputation_modifier: f32) -> TradeBalance {
        let Some(trade) = &self.active_trade else {
            return TradeBalance::default();
        };

        let Some(inventory) = self.inventories.get(&trade.npc_id) else {
            return TradeBalance::default();
        };

        // Calculate player offer value
        let player_value: u32 = trade.player_offer.iter()
            .filter_map(|(item, qty)| {
                inventory.items.iter()
                    .find(|i| &i.id == item)
                    .map(|i| i.base_price * qty)
            })
            .sum::<u32>() + trade.player_gold;

        // Calculate NPC offer value (with reputation modifier)
        let npc_value: u32 = trade.npc_offer.iter()
            .filter_map(|(item, qty)| {
                inventory.items.iter()
                    .find(|i| &i.id == item)
                    .map(|i| ((i.base_price as f32) * reputation_modifier) as u32 * qty)
            })
            .sum::<u32>() + trade.npc_gold;

        TradeBalance {
            player_value,
            npc_value,
            is_fair: player_value >= npc_value,
            difference: if player_value >= npc_value {
                player_value - npc_value
            } else {
                npc_value - player_value
            },
        }
    }

    /// Execute the trade
    pub fn execute_trade(&mut self, reputation_modifier: f32) -> Result<TradeResult, &'static str> {
        let balance = self.calculate_trade_value(reputation_modifier);

        if !balance.is_fair {
            return Err("Trade is not fair - offer more");
        }

        let trade = self.active_trade.take().ok_or("No active trade")?;

        // Record trade
        self.history.push(TradeRecord {
            npc_id: trade.npc_id,
            player_gave: trade.player_offer.clone(),
            player_received: trade.npc_offer.clone(),
            gold_exchanged: trade.player_gold as i32 - trade.npc_gold as i32,
        });

        // Update NPC inventory
        if let Some(inventory) = self.inventories.get_mut(&trade.npc_id) {
            for (item_id, qty) in &trade.npc_offer {
                if let Some(item) = inventory.items.iter_mut().find(|i| &i.id == item_id) {
                    item.stock = item.stock.saturating_sub(*qty);
                }
            }
            // Add player's sold items to NPC stock (for resale)
            for (item_id, qty) in &trade.player_offer {
                if let Some(item) = inventory.items.iter_mut().find(|i| &i.id == item_id) {
                    item.stock += qty;
                }
            }
        }

        Ok(TradeResult {
            items_received: trade.npc_offer,
            items_given: trade.player_offer,
            gold_received: trade.npc_gold,
            gold_given: trade.player_gold,
            change: balance.difference,
        })
    }

    /// Cancel current trade
    pub fn cancel_trade(&mut self) {
        self.active_trade = None;
    }

    /// Check if player can afford an item
    pub fn can_afford(&self, npc_id: u32, item_id: &str, player_gold: u32, reputation_modifier: f32) -> bool {
        self.inventories.get(&npc_id)
            .and_then(|inv| inv.items.iter().find(|i| i.id == item_id))
            .map(|item| {
                let price = ((item.base_price as f32) * reputation_modifier) as u32;
                player_gold >= price && item.stock > 0
            })
            .unwrap_or(false)
    }

    /// Quick buy (direct purchase without bartering)
    pub fn quick_buy(&mut self, npc_id: u32, item_id: &str, quantity: u32, reputation_modifier: f32) -> Result<(String, u32, u32), &'static str> {
        let inventory = self.inventories.get_mut(&npc_id).ok_or("NPC doesn't trade")?;
        let item = inventory.items.iter_mut()
            .find(|i| i.id == item_id)
            .ok_or("Item not found")?;

        if item.stock < quantity {
            return Err("Not enough stock");
        }

        let price = ((item.base_price as f32) * reputation_modifier) as u32 * quantity;
        item.stock -= quantity;

        Ok((item_id.to_string(), quantity, price))
    }

    /// Quick sell (direct sale without bartering)
    pub fn quick_sell(&mut self, npc_id: u32, item_id: &str, quantity: u32, reputation_modifier: f32) -> Result<u32, &'static str> {
        let inventory = self.inventories.get_mut(&npc_id).ok_or("NPC doesn't trade")?;

        // Find or create item entry (NPC can buy even if they don't normally sell it)
        let base_price = inventory.items.iter()
            .find(|i| i.id == item_id)
            .map(|i| i.base_price)
            .unwrap_or(5); // Default buy price for unknown items

        // Sell price is typically lower than buy price
        let sell_price = ((base_price as f32) * 0.6 * reputation_modifier) as u32 * quantity;

        // Add to NPC stock
        if let Some(item) = inventory.items.iter_mut().find(|i| i.id == item_id) {
            item.stock += quantity;
        }

        Ok(sell_price)
    }

    /// Restock all daily inventories
    pub fn daily_restock(&mut self) {
        for inventory in self.inventories.values_mut() {
            if inventory.restocks_daily {
                for item in &mut inventory.items {
                    // Restock to base amount
                    item.stock = item.stock.max(item.base_stock());
                }
            }
        }
    }
}

/// NPC trade inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInventory {
    pub npc_id: u32,
    pub name: String,
    pub items: Vec<TradeItem>,
    pub restocks_daily: bool,
    pub faction: Faction,
    pub required_reputation: ReputationLevel,
}

/// Tradeable item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeItem {
    pub id: String,
    pub name: String,
    pub base_price: u32,
    pub stock: u32,
    pub category: ItemCategory,
}

impl TradeItem {
    pub fn new(id: &str, name: &str, base_price: u32, stock: u32, category: ItemCategory) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            base_price,
            stock,
            category,
        }
    }

    pub fn base_stock(&self) -> u32 {
        match self.category {
            ItemCategory::Consumable | ItemCategory::Ammo => 20,
            ItemCategory::Food | ItemCategory::Medicine => 10,
            ItemCategory::Crafting => 15,
            ItemCategory::Tool | ItemCategory::Equipment => 5,
            ItemCategory::Weapon | ItemCategory::Armor => 2,
            ItemCategory::Artifact => 1,
            ItemCategory::Material | ItemCategory::Fossil => 0, // Only from player sales
        }
    }
}

/// Item categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Tool,
    Equipment,
    Consumable,
    Food,
    Medicine,
    Crafting,
    Material,
    Ammo,
    Artifact,
    Fossil,
}

/// Active trade session
#[derive(Debug, Clone)]
pub struct ActiveTrade {
    pub npc_id: u32,
    pub player_offer: Vec<(String, u32)>,
    pub npc_offer: Vec<(String, u32)>,
    pub player_gold: u32,
    pub npc_gold: u32,
}

/// Trade balance calculation
#[derive(Debug, Clone, Default)]
pub struct TradeBalance {
    pub player_value: u32,
    pub npc_value: u32,
    pub is_fair: bool,
    pub difference: u32,
}

/// Trade execution result
#[derive(Debug, Clone)]
pub struct TradeResult {
    pub items_received: Vec<(String, u32)>,
    pub items_given: Vec<(String, u32)>,
    pub gold_received: u32,
    pub gold_given: u32,
    pub change: u32,
}

/// Trade history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub npc_id: u32,
    pub player_gave: Vec<(String, u32)>,
    pub player_received: Vec<(String, u32)>,
    pub gold_exchanged: i32,
}

/// Trade offer for barter system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOffer {
    pub items: Vec<(String, u32)>,
    pub gold: u32,
}

impl TradeOffer {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            gold: 0,
        }
    }

    pub fn add_item(&mut self, item: String, quantity: u32) {
        if let Some((_, qty)) = self.items.iter_mut().find(|(i, _)| i == &item) {
            *qty += quantity;
        } else {
            self.items.push((item, quantity));
        }
    }

    pub fn add_gold(&mut self, amount: u32) {
        self.gold += amount;
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.gold = 0;
    }
}
