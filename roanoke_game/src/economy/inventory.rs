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

    /// Rebuild the item index (call after loading)
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
            for (idx, slot) in self.slots.iter_mut().enumerate() {
                if let Some(existing) = slot {
                    if existing.template_id == item.template_id
                        && existing.rarity == item.rarity
                        && existing.stack_size < existing.max_stack
                    {
                        let can_add = existing.max_stack - existing.stack_size;
                        let to_add = item.stack_size.min(can_add);
                        existing.stack_size += to_add;

                        if to_add == item.stack_size {
                            return Ok(idx);
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

    /// Get all items as iterator
    pub fn all_items(&self) -> impl Iterator<Item = &Item> {
        self.slots.iter().filter_map(|s| s.as_ref())
    }

    /// Get slot by index
    pub fn get_slot(&self, index: usize) -> Option<&Item> {
        self.slots.get(index).and_then(|s| s.as_ref())
    }

    /// Swap two slots
    pub fn swap_slots(&mut self, a: usize, b: usize) -> Result<(), InventoryError> {
        if a >= self.slots.len() || b >= self.slots.len() {
            return Err(InventoryError::InvalidSlot);
        }
        self.slots.swap(a, b);
        self.rebuild_index();
        Ok(())
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

impl std::fmt::Display for InventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "Inventory is full"),
            Self::StashFull => write!(f, "Stash is full"),
            Self::ItemNotFound => write!(f, "Item not found"),
            Self::InvalidSlot => write!(f, "Invalid equipment slot for this item"),
            Self::NotStackable => write!(f, "Item is not stackable"),
            Self::CannotDrop => write!(f, "Cannot drop this item"),
        }
    }
}

impl std::error::Error for InventoryError {}
