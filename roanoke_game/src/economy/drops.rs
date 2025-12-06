//! Dropped Item Entity System
//!
//! Manages items dropped in the world that players can pick up.
//! Dropped items have a visual representation and can despawn over time.

use super::item::{Item, ItemId, Rarity};
use super::loot::DropReward;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Unique identifier for a dropped item in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DropId(pub u64);

impl DropId {
    /// Generate a new unique drop ID
    pub fn generate() -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        Self((timestamp << 24) | (id & 0xFFFFFF))
    }
}

/// A dropped item entity in the world
#[derive(Debug, Clone)]
pub struct DroppedItem {
    pub id: DropId,
    pub item: Item,
    pub position: Vec3,
    pub rotation: f32, // Y-axis rotation for visual variety
    pub spawn_time: Instant,
    pub despawn_timer: f32, // Seconds until despawn (0 = never)
    pub bounce_phase: f32, // For visual bounce animation
    pub glow_intensity: f32, // Based on rarity
    pub is_highlighted: bool, // Player is looking at it
}

impl DroppedItem {
    /// Create a new dropped item
    pub fn new(item: Item, position: Vec3) -> Self {
        let glow = Self::glow_for_rarity(&item.rarity);
        Self {
            id: DropId::generate(),
            item,
            position,
            rotation: rand::random::<f32>() * std::f32::consts::TAU,
            spawn_time: Instant::now(),
            despawn_timer: 300.0, // 5 minutes default
            bounce_phase: rand::random::<f32>() * std::f32::consts::TAU,
            glow_intensity: glow,
            is_highlighted: false,
        }
    }

    /// Create with currency reward instead of item
    pub fn new_currency(wampum: u64, tobacco: u64, position: Vec3) -> Option<Self> {
        if wampum == 0 && tobacco == 0 {
            return None;
        }

        // Create a placeholder item for currency using the proper constructor
        let mut item = Item::new(
            if tobacco > 0 { "tobacco_bundle" } else { "wampum_pouch" },
            &if tobacco > 0 {
                format!("{} Tobacco", tobacco)
            } else {
                format!("{} Wampum", wampum)
            },
            super::item::ItemType::Currency,
            if tobacco > 0 { (tobacco * 10).min(u32::MAX as u64) as u32 } else { wampum.min(u32::MAX as u64) as u32 },
        );
        item.rarity = if wampum >= 100 || tobacco >= 10 { Rarity::Uncommon } else { Rarity::Common };
        item.stack_size = if tobacco > 0 { tobacco.min(999) as u32 } else { wampum.min(999) as u32 };
        item.max_stack = 999;

        Some(Self::new(item, position))
    }

    /// Get glow intensity based on rarity
    fn glow_for_rarity(rarity: &Rarity) -> f32 {
        match rarity {
            Rarity::Crude => 0.0,
            Rarity::Common => 0.1,
            Rarity::Uncommon => 0.3,
            Rarity::Rare => 0.5,
            Rarity::Epic => 0.7,
            Rarity::Legendary => 1.0,
            Rarity::Mythic => 1.2,
            Rarity::Primordial => 1.5,
        }
    }

    /// Get visual color based on rarity
    pub fn color(&self) -> [f32; 3] {
        self.item.rarity.color()
    }

    /// Get visual scale (higher rarity = slightly larger)
    pub fn scale(&self) -> f32 {
        match self.item.rarity {
            Rarity::Crude => 0.3,
            Rarity::Common => 0.35,
            Rarity::Uncommon => 0.4,
            Rarity::Rare => 0.45,
            Rarity::Epic => 0.5,
            Rarity::Legendary => 0.55,
            Rarity::Mythic => 0.6,
            Rarity::Primordial => 0.7,
        }
    }

    /// Get current vertical offset for bounce animation
    pub fn bounce_offset(&self, time: f32) -> f32 {
        let phase = self.bounce_phase + time * 2.0;
        (phase.sin() * 0.1).max(0.0) // Gentle bob, always positive
    }

    /// Check if this drop should be despawned
    pub fn should_despawn(&self) -> bool {
        self.despawn_timer > 0.0 && self.spawn_time.elapsed().as_secs_f32() > self.despawn_timer
    }

    /// Get seconds remaining until despawn
    pub fn time_remaining(&self) -> f32 {
        if self.despawn_timer <= 0.0 {
            f32::INFINITY
        } else {
            (self.despawn_timer - self.spawn_time.elapsed().as_secs_f32()).max(0.0)
        }
    }
}

/// Manages all dropped items in the world
#[derive(Debug)]
pub struct DroppedItemManager {
    drops: HashMap<DropId, DroppedItem>,
    /// Maximum drops to track (oldest get removed)
    max_drops: usize,
    /// Time accumulator for cleanup
    cleanup_timer: f32,
    /// Statistics
    total_spawned: u64,
    total_picked_up: u64,
    total_despawned: u64,
}

impl Default for DroppedItemManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DroppedItemManager {
    pub fn new() -> Self {
        Self {
            drops: HashMap::with_capacity(100),
            max_drops: 200,
            cleanup_timer: 0.0,
            total_spawned: 0,
            total_picked_up: 0,
            total_despawned: 0,
        }
    }

    /// Spawn a dropped item at the given position
    pub fn spawn_drop(&mut self, item: Item, position: Vec3) -> DropId {
        let drop = DroppedItem::new(item, position);
        let id = drop.id;
        self.drops.insert(id, drop);
        self.total_spawned += 1;

        // Remove oldest if over limit
        self.enforce_limit();

        id
    }

    /// Spawn currency drop (returns None if both values are 0)
    pub fn spawn_currency(&mut self, wampum: u64, tobacco: u64, position: Vec3) -> Option<DropId> {
        let drop = DroppedItem::new_currency(wampum, tobacco, position)?;
        let id = drop.id;
        self.drops.insert(id, drop);
        self.total_spawned += 1;
        self.enforce_limit();
        Some(id)
    }

    /// Spawn multiple items at a position with slight scatter
    pub fn spawn_loot_pile(&mut self, items: Vec<Item>, position: Vec3, wampum: u64, tobacco: u64) -> Vec<DropId> {
        let mut ids = Vec::with_capacity(items.len() + 2);

        for (i, item) in items.into_iter().enumerate() {
            // Add some scatter
            let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
            let offset = Vec3::new(angle.cos() * 0.5, 0.0, angle.sin() * 0.5);
            let pos = position + offset;
            ids.push(self.spawn_drop(item, pos));
        }

        // Spawn currency if any
        if let Some(id) = self.spawn_currency(wampum, tobacco, position) {
            ids.push(id);
        }

        ids
    }

    /// Get a drop by ID
    pub fn get(&self, id: DropId) -> Option<&DroppedItem> {
        self.drops.get(&id)
    }

    /// Get a mutable drop by ID
    pub fn get_mut(&mut self, id: DropId) -> Option<&mut DroppedItem> {
        self.drops.get_mut(&id)
    }

    /// Remove a drop (when picked up)
    pub fn pickup(&mut self, id: DropId) -> Option<DroppedItem> {
        let drop = self.drops.remove(&id);
        if drop.is_some() {
            self.total_picked_up += 1;
        }
        drop
    }

    /// Get all drops within radius of a position
    pub fn drops_near(&self, center: Vec3, radius: f32) -> Vec<&DroppedItem> {
        let r2 = radius * radius;
        self.drops
            .values()
            .filter(|drop| (drop.position - center).length_squared() <= r2)
            .collect()
    }

    /// Get the closest drop to a position within range
    pub fn closest_drop(&self, center: Vec3, max_range: f32) -> Option<&DroppedItem> {
        let r2 = max_range * max_range;
        self.drops
            .values()
            .filter(|drop| (drop.position - center).length_squared() <= r2)
            .min_by(|a, b| {
                let da = (a.position - center).length_squared();
                let db = (b.position - center).length_squared();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Update all drops (despawn expired ones)
    pub fn update(&mut self, delta: f32) {
        self.cleanup_timer += delta;

        // Cleanup every second
        if self.cleanup_timer >= 1.0 {
            self.cleanup_timer = 0.0;

            let before = self.drops.len();
            self.drops.retain(|_, drop| !drop.should_despawn());
            let removed = before - self.drops.len();
            self.total_despawned += removed as u64;
        }
    }

    /// Get all active drops
    pub fn all_drops(&self) -> impl Iterator<Item = &DroppedItem> {
        self.drops.values()
    }

    /// Get drop count
    pub fn count(&self) -> usize {
        self.drops.len()
    }

    /// Enforce max drop limit by removing oldest
    fn enforce_limit(&mut self) {
        while self.drops.len() > self.max_drops {
            // Find and remove oldest drop
            if let Some(&oldest_id) = self.drops
                .iter()
                .min_by_key(|(_, d)| d.spawn_time)
                .map(|(id, _)| id)
            {
                self.drops.remove(&oldest_id);
                self.total_despawned += 1;
            } else {
                break;
            }
        }
    }

    /// Clear highlight on all drops
    pub fn clear_highlights(&mut self) {
        for drop in self.drops.values_mut() {
            drop.is_highlighted = false;
        }
    }

    /// Set highlight on a specific drop
    pub fn set_highlighted(&mut self, id: DropId, highlighted: bool) {
        if let Some(drop) = self.drops.get_mut(&id) {
            drop.is_highlighted = highlighted;
        }
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (self.total_spawned, self.total_picked_up, self.total_despawned)
    }
}
