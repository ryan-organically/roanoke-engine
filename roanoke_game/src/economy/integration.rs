//! Economy Integration Layer
//!
//! Connects the economy system with combat, trading, progression, and game state.

use super::{
    item::{Item, ItemId, ItemType, Rarity, Quality, OriginMethod, KillRecord},
    inventory::{Inventory, InventoryError},
    currency::{Wallet, TransactionType},
    loot::{LootGenerator, LootContext, KillContext, PityTracker, DropReward},
    pipeline::{
        EventPipeline, EconomyEvent, LootDropEvent, LootSource, ItemAddedEvent,
        ItemSource, CurrencyEvent, MetricType, PityEvent, RareDropEvent,
        CurrencyType, PersistenceManager, AnalyticsSummary,
    },
};
use crate::animals::types::AnimalSpecies;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum pending loot notifications
const MAX_PENDING_NOTIFICATIONS: usize = 20;

/// Player's complete economy state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEconomy {
    /// Player's inventory
    pub inventory: Inventory,

    /// Player's wallet (dual currency)
    pub wallet: Wallet,

    /// Pity tracker for loot drops
    pub pity_tracker: PityTracker,

    /// Session statistics
    #[serde(default)]
    pub session_stats: SessionEconomyStats,
}

impl Default for PlayerEconomy {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerEconomy {
    pub fn new() -> Self {
        Self {
            inventory: Inventory::new(),
            wallet: Wallet::new(),
            pity_tracker: PityTracker::default(),
            session_stats: SessionEconomyStats::default(),
        }
    }

    /// Called after loading to rebuild indices
    pub fn on_load(&mut self) {
        self.inventory.rebuild_index();
    }
}

/// Session-specific economy statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionEconomyStats {
    pub session_minutes: u32,
    pub items_looted: u32,
    pub wampum_earned: u64,
    pub tobacco_earned: u64,
    pub rarest_drop: Option<Rarity>,
    pub items_sold: u32,
    pub items_bought: u32,
}

/// Loot drop notification for UI
#[derive(Debug, Clone)]
pub struct LootNotification {
    pub item: Item,
    pub reward: DropReward,
    pub timestamp: f64,
    pub position: Vec3,
    pub is_new_template: bool,
    pub triggered_pity: bool,
}

/// Economy event manager - handles loot drops, notifications, and integration
pub struct EconomyManager {
    /// Loot generator
    loot_generator: LootGenerator,

    /// Pending loot notifications for UI
    pending_notifications: VecDeque<LootNotification>,

    /// Current game time for timestamps
    current_time: f64,

    /// Moon phase (0.0 - 1.0)
    moon_phase: f32,

    /// Weather modifier
    weather_modifier: f32,

    /// Active event multiplier
    event_multiplier: f32,

    /// Event pipeline for validation, transactions, and analytics
    pub event_pipeline: EventPipeline,

    /// Persistence manager for auto-save
    pub persistence_manager: PersistenceManager,
}

impl Default for EconomyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EconomyManager {
    pub fn new() -> Self {
        Self {
            loot_generator: LootGenerator::new(),
            pending_notifications: VecDeque::new(),
            current_time: 0.0,
            moon_phase: 0.5,
            weather_modifier: 1.0,
            event_multiplier: 1.0,
            event_pipeline: EventPipeline::new(),
            persistence_manager: PersistenceManager::new(),
        }
    }

    /// Update time-based modifiers
    pub fn update(&mut self, game_time: f64, moon_phase: f32, weather_modifier: f32) {
        self.current_time = game_time;
        self.moon_phase = moon_phase;
        self.weather_modifier = weather_modifier;
    }

    /// Set active event multiplier
    pub fn set_event_multiplier(&mut self, multiplier: f32) {
        self.event_multiplier = multiplier;
    }

    /// Process an animal kill and generate loot
    pub fn process_animal_kill(
        &mut self,
        economy: &mut PlayerEconomy,
        species: AnimalSpecies,
        position: Vec3,
        was_stealth: bool,
        was_perfect: bool,
        was_critical: bool,
        weapon_used: &str,
        kill_time_seconds: f32,
        player_health_remaining: f32,
        hunting_skill_level: u32,
        luck_stat: f32,
    ) -> Vec<LootNotification> {
        let mut notifications = Vec::new();

        // Build loot context
        let context = LootContext {
            luck: luck_stat,
            skill_level: hunting_skill_level,
            session_minutes: economy.session_stats.session_minutes,
            karma: economy.pity_tracker.karma_accumulated,
            kill_context: Some(KillContext {
                species_name: species.name().to_string(),
                was_stealth,
                was_perfect,
                was_critical,
                weapon_used: weapon_used.to_string(),
                kill_time_seconds,
                player_health_remaining,
            }),
            moon_phase: self.moon_phase,
            weather_modifier: self.weather_modifier,
            event_multiplier: self.event_multiplier,
        };

        // Check for pity before dropping
        let had_pity = economy.pity_tracker.check_pity().is_some();

        // Generate loot for each item in species loot table
        for loot_id in species.loot() {
            if let Some((mut item, reward)) = self.loot_generator.generate_drop(
                loot_id,
                &context,
                &mut economy.pity_tracker,
            ) {
                // Set origin location
                item.provenance.origin_location = Some([position.x, position.y, position.z]);

                // Check if this is a new template for this player
                let is_new_template = !economy.inventory.all_items()
                    .any(|i| i.template_id == item.template_id);

                // Add to inventory
                if economy.inventory.add_item(item.clone()).is_ok() {
                    // Add currency rewards
                    if reward.wampum > 0 {
                        economy.wallet.receive_wampum(
                            reward.wampum,
                            TransactionType::LootDrop,
                            &format!("Loot from {}", species.name()),
                        );
                        economy.session_stats.wampum_earned += reward.wampum;
                    }

                    if reward.tobacco > 0 {
                        economy.wallet.add_tobacco(reward.tobacco);
                        economy.session_stats.tobacco_earned += reward.tobacco;
                    }

                    // Update session stats
                    economy.session_stats.items_looted += 1;

                    // Track rarest drop
                    if economy.session_stats.rarest_drop.map(|r| item.rarity > r).unwrap_or(true) {
                        economy.session_stats.rarest_drop = Some(item.rarity);
                    }

                    // Create notification
                    let notification = LootNotification {
                        item: item.clone(),
                        reward: reward.clone(),
                        timestamp: self.current_time,
                        position,
                        is_new_template,
                        triggered_pity: had_pity && item.rarity >= Rarity::Rare,
                    };

                    notifications.push(notification.clone());

                    // Queue for UI
                    self.pending_notifications.push_back(notification);
                    if self.pending_notifications.len() > MAX_PENDING_NOTIFICATIONS {
                        self.pending_notifications.pop_front();
                    }
                }
            }
        }

        notifications
    }

    /// Process a fossil extraction
    pub fn process_fossil_extraction(
        &mut self,
        economy: &mut PlayerEconomy,
        fossil_type: &str,
        site_name: &str,
        position: Vec3,
        extraction_quality: u8,
        archaeology_skill_level: u32,
        luck_stat: f32,
    ) -> Option<LootNotification> {
        let context = LootContext {
            luck: luck_stat,
            skill_level: archaeology_skill_level,
            session_minutes: economy.session_stats.session_minutes,
            karma: economy.pity_tracker.karma_accumulated,
            kill_context: None,
            moon_phase: self.moon_phase,
            weather_modifier: self.weather_modifier,
            event_multiplier: self.event_multiplier,
        };

        let (mut item, reward) = self.loot_generator.generate_drop(
            fossil_type,
            &context,
            &mut economy.pity_tracker,
        )?;

        // Override quality with extraction quality
        item.quality = Quality::new(extraction_quality);
        item.provenance.origin_method = OriginMethod::ArchaeologyDig {
            site_name: site_name.to_string(),
        };
        item.provenance.origin_location = Some([position.x, position.y, position.z]);

        let is_new_template = !economy.inventory.all_items()
            .any(|i| i.template_id == item.template_id);

        economy.inventory.add_item(item.clone()).ok()?;

        if reward.wampum > 0 {
            economy.wallet.receive_wampum(
                reward.wampum,
                TransactionType::LootDrop,
                &format!("Fossil: {}", fossil_type),
            );
            economy.session_stats.wampum_earned += reward.wampum;
        }

        economy.session_stats.items_looted += 1;

        let notification = LootNotification {
            item: item.clone(),
            reward,
            timestamp: self.current_time,
            position,
            is_new_template,
            triggered_pity: false,
        };

        self.pending_notifications.push_back(notification.clone());
        Some(notification)
    }

    /// Process quest reward
    pub fn grant_quest_reward(
        &mut self,
        economy: &mut PlayerEconomy,
        quest_id: &str,
        item_template_id: &str,
        guaranteed_rarity: Option<Rarity>,
        wampum_bonus: u64,
        tobacco_bonus: u64,
    ) -> Option<Item> {
        let context = LootContext::default();

        let (mut item, _) = self.loot_generator.generate_drop(
            item_template_id,
            &context,
            &mut economy.pity_tracker,
        )?;

        // Override rarity if guaranteed
        if let Some(rarity) = guaranteed_rarity {
            item.rarity = rarity;
        }

        item.provenance.origin_method = OriginMethod::QuestReward {
            quest_id: quest_id.to_string(),
        };

        economy.inventory.add_item(item.clone()).ok()?;

        // Grant bonus currency
        if wampum_bonus > 0 {
            economy.wallet.receive_wampum(
                wampum_bonus,
                TransactionType::QuestReward,
                &format!("Quest: {}", quest_id),
            );
        }

        if tobacco_bonus > 0 {
            economy.wallet.add_tobacco(tobacco_bonus);
        }

        Some(item)
    }

    /// Get and clear pending notifications
    pub fn drain_notifications(&mut self) -> Vec<LootNotification> {
        self.pending_notifications.drain(..).collect()
    }

    /// Peek at pending notifications without clearing
    pub fn peek_notifications(&self) -> impl Iterator<Item = &LootNotification> {
        self.pending_notifications.iter()
    }

    /// Get the loot generator for direct access
    pub fn loot_generator(&self) -> &LootGenerator {
        &self.loot_generator
    }

    // === PIPELINE INTEGRATION ===

    /// Record a loot drop event in the pipeline
    pub fn record_loot_drop(
        &mut self,
        species: &str,
        position: [f32; 3],
        items: Vec<ItemId>,
        wampum: u64,
        tobacco: u64,
        was_legendary: bool,
    ) {
        let event_id = self.event_pipeline.next_event_id();
        let timestamp = EventPipeline::timestamp();

        // Capture count before moving items
        let item_count = items.len();

        let event = EconomyEvent::LootDropped(LootDropEvent {
            event_id,
            timestamp,
            source_type: LootSource::AnimalKill {
                species: species.to_string(),
                was_legendary,
            },
            source_id: format!("kill_{}", event_id),
            position,
            items,
            total_wampum: wampum,
            total_tobacco: tobacco,
        });

        self.event_pipeline.queue_event(event);

        // Record in analytics
        self.event_pipeline.analytics_mut().record_simple(MetricType::ItemsLooted, item_count as f64);
        self.event_pipeline.analytics_mut().record_simple(MetricType::WampumEarned, wampum as f64);
        if tobacco > 0 {
            self.event_pipeline.analytics_mut().record_simple(MetricType::TobaccoEarned, tobacco as f64);
        }

        // Mark dirty for auto-save
        self.persistence_manager.mark_dirty();
    }

    /// Record a rare drop occurrence
    pub fn record_rare_drop(&mut self, item: &Item, species: &str, was_pity: bool) {
        let event_id = self.event_pipeline.next_event_id();
        let timestamp = EventPipeline::timestamp();

        let event = EconomyEvent::RareDropOccurred(RareDropEvent {
            event_id,
            timestamp,
            item_id: item.id,
            rarity: item.rarity,
            source: LootSource::AnimalKill {
                species: species.to_string(),
                was_legendary: item.rarity >= Rarity::Legendary,
            },
            was_pity,
        });

        self.event_pipeline.queue_event(event);

        if item.rarity >= Rarity::Legendary {
            self.event_pipeline.analytics_mut().record_simple(MetricType::LegendaryDrops, 1.0);
        } else {
            self.event_pipeline.analytics_mut().record_simple(MetricType::RareDrops, 1.0);
        }
    }

    /// Record pity trigger
    pub fn record_pity_trigger(&mut self, rarity: Rarity, drops_since: u32, karma: f32) {
        let event_id = self.event_pipeline.next_event_id();
        let timestamp = EventPipeline::timestamp();

        let event = EconomyEvent::PityTriggered(PityEvent {
            event_id,
            timestamp,
            rarity_guaranteed: rarity,
            drops_since_last: drops_since,
            karma_accumulated: karma,
        });

        self.event_pipeline.queue_event(event);
        self.event_pipeline.analytics_mut().record_simple(MetricType::PityTriggers, 1.0);
    }

    /// Process all pending events and return them
    pub fn process_pipeline_events(&mut self) -> Vec<EconomyEvent> {
        self.event_pipeline.process_events()
    }

    /// Get analytics summary
    pub fn get_analytics_summary(&self) -> AnalyticsSummary {
        self.event_pipeline.analytics().summary()
    }

    /// Check if auto-save is needed
    pub fn should_auto_save(&self) -> bool {
        self.persistence_manager.should_auto_save()
    }

    /// Mark save complete
    pub fn on_save_complete(&mut self) {
        self.persistence_manager.on_save_complete();
    }

    /// Cleanup old pipeline data
    pub fn cleanup_pipeline(&mut self) {
        self.event_pipeline.cleanup();
    }

    /// Get transaction statistics
    pub fn get_transaction_stats(&self) -> super::pipeline::TransactionStats {
        self.event_pipeline.transaction_stats()
    }

    /// Get error count for monitoring
    pub fn get_pipeline_error_count(&self) -> usize {
        self.event_pipeline.error_count()
    }
}

/// Trading integration - connects old trading system with new economy
pub mod trading_bridge {
    use super::*;
    use crate::npc::trading::{TradingSystem, TradeItem, ItemCategory};

    /// Convert old ItemCategory to new ItemType
    pub fn category_to_type(category: ItemCategory) -> ItemType {
        match category {
            ItemCategory::Weapon => ItemType::Weapon,
            ItemCategory::Armor => ItemType::Armor,
            ItemCategory::Tool => ItemType::Tool,
            ItemCategory::Equipment => ItemType::Accessory,
            ItemCategory::Consumable => ItemType::Consumable,
            ItemCategory::Food => ItemType::Food,
            ItemCategory::Medicine => ItemType::Medicine,
            ItemCategory::Crafting => ItemType::Crafting,
            ItemCategory::Material => ItemType::Material,
            ItemCategory::Ammo => ItemType::Ammo,
            ItemCategory::Artifact => ItemType::Artifact,
            ItemCategory::Fossil => ItemType::Fossil,
        }
    }

    /// Convert TradeItem to economy Item
    pub fn trade_item_to_item(trade_item: &TradeItem, quantity: u32) -> Item {
        let mut item = Item::new(
            &trade_item.id,
            &trade_item.name,
            category_to_type(trade_item.category),
            trade_item.base_price,
        );
        item.stack_size = quantity;
        item.max_stack = 20; // Default for tradeable items
        item.provenance.origin_method = OriginMethod::NpcPurchase { npc_id: 0 };
        item
    }

    /// Process an NPC purchase using new economy system
    pub fn buy_from_npc(
        economy: &mut PlayerEconomy,
        trading: &mut TradingSystem,
        npc_id: u32,
        item_id: &str,
        quantity: u32,
        reputation_modifier: f32,
    ) -> Result<Item, &'static str> {
        // Get price from trading system
        let (bought_id, bought_qty, price) = trading.quick_buy(
            npc_id,
            item_id,
            quantity,
            reputation_modifier,
        )?;

        // Check if player can afford
        if !economy.wallet.spend_wampum(price as u64) {
            // Rollback the NPC inventory change
            if let Some(inv) = trading.inventories.get_mut(&npc_id) {
                if let Some(item) = inv.items.iter_mut().find(|i| i.id == bought_id) {
                    item.stock += bought_qty;
                }
            }
            return Err("Not enough wampum");
        }

        // Get the trade item for conversion
        let trade_item = trading.inventories.get(&npc_id)
            .and_then(|inv| inv.items.iter().find(|i| i.id == item_id))
            .ok_or("Item not found")?
            .clone();

        // Create economy item
        let mut item = trade_item_to_item(&trade_item, bought_qty);
        item.provenance.origin_method = OriginMethod::NpcPurchase { npc_id };

        // Add to inventory
        economy.inventory.add_item(item.clone())
            .map_err(|_| "Inventory full")?;

        // Record transaction
        economy.wallet.record_transaction(super::super::currency::CurrencyTransaction {
            transaction_type: TransactionType::NpcPurchase,
            wampum_amount: -(price as i64),
            tobacco_amount: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: format!("Bought {} x{}", trade_item.name, bought_qty),
        });

        economy.session_stats.items_bought += bought_qty;

        Ok(item)
    }

    /// Process an NPC sale using new economy system
    pub fn sell_to_npc(
        economy: &mut PlayerEconomy,
        trading: &mut TradingSystem,
        npc_id: u32,
        item_id: ItemId,
        quantity: u32,
        reputation_modifier: f32,
    ) -> Result<u64, &'static str> {
        // Get item from inventory
        let item = economy.inventory.get_item(item_id)
            .ok_or("Item not found in inventory")?
            .clone();

        if item.stack_size < quantity {
            return Err("Not enough items to sell");
        }

        // Calculate sale price
        let base_price = item.calculate_value();
        let sale_price = ((base_price as f32) * 0.6 * reputation_modifier) as u64 * quantity as u64;

        // Remove from inventory or reduce stack
        if item.stack_size == quantity {
            economy.inventory.remove_item(item_id);
        } else {
            if let Some(inv_item) = economy.inventory.get_item_mut(item_id) {
                inv_item.stack_size -= quantity;
            }
        }

        // Add to wallet
        economy.wallet.receive_wampum(
            sale_price,
            TransactionType::NpcSale,
            &format!("Sold {} x{}", item.name, quantity),
        );

        // Update NPC inventory (they now have this item)
        let _ = trading.quick_sell(npc_id, &item.template_id, quantity, reputation_modifier);

        economy.session_stats.items_sold += quantity;

        Ok(sale_price)
    }
}

/// Combat result with economy integration
#[derive(Debug, Clone)]
pub struct CombatLootResult {
    pub species: AnimalSpecies,
    pub position: Vec3,
    pub was_stealth: bool,
    pub was_perfect: bool,
    pub was_critical: bool,
    pub loot: Vec<LootNotification>,
    pub total_wampum: u64,
    pub total_tobacco: u64,
}

/// Extension trait for easy integration with existing combat results
pub trait CombatEconomyExt {
    fn process_loot(
        &self,
        manager: &mut EconomyManager,
        economy: &mut PlayerEconomy,
        weapon_used: &str,
        kill_time: f32,
        player_health: f32,
        hunting_level: u32,
        luck: f32,
    ) -> CombatLootResult;
}

impl CombatEconomyExt for crate::animals::combat::AnimalAttackResult {
    fn process_loot(
        &self,
        manager: &mut EconomyManager,
        economy: &mut PlayerEconomy,
        weapon_used: &str,
        kill_time: f32,
        player_health: f32,
        hunting_level: u32,
        luck: f32,
    ) -> CombatLootResult {
        if !self.killed {
            return CombatLootResult {
                species: self.species,
                position: self.position,
                was_stealth: self.was_stealth,
                was_perfect: false,
                was_critical: self.was_critical,
                loot: vec![],
                total_wampum: 0,
                total_tobacco: 0,
            };
        }

        let notifications = manager.process_animal_kill(
            economy,
            self.species,
            self.position,
            self.was_stealth,
            false, // was_perfect - could be calculated from damage/time
            self.was_critical,
            weapon_used,
            kill_time,
            player_health,
            hunting_level,
            luck,
        );

        let total_wampum: u64 = notifications.iter().map(|n| n.reward.wampum).sum();
        let total_tobacco: u64 = notifications.iter().map(|n| n.reward.tobacco).sum();

        CombatLootResult {
            species: self.species,
            position: self.position,
            was_stealth: self.was_stealth,
            was_perfect: false,
            was_critical: self.was_critical,
            loot: notifications,
            total_wampum,
            total_tobacco,
        }
    }
}
