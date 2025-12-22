//! Storage Container System
//!
//! World-placed containers (chests, crates, barrels) that can store items.
//! Supports multiple container types with different capacities and properties.

use super::item::{Item, ItemId};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a storage container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId(pub u64);

impl ContainerId {
    pub fn generate() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let random = rand::random::<u64>();
        Self(timestamp ^ random)
    }
}

/// Types of storage containers with different properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerType {
    /// Basic wooden chest - 20 slots
    WoodenChest,
    /// Reinforced chest - 30 slots, lockable
    ReinforcedChest,
    /// Small crate - 10 slots
    Crate,
    /// Barrel - 15 slots, liquid-safe
    Barrel,
    /// Large storage trunk - 40 slots
    Trunk,
    /// Personal lockbox - 10 slots, always locked
    Lockbox,
}

impl ContainerType {
    /// Number of storage slots for this container type
    pub fn slot_count(&self) -> usize {
        match self {
            Self::WoodenChest => 20,
            Self::ReinforcedChest => 30,
            Self::Crate => 10,
            Self::Barrel => 15,
            Self::Trunk => 40,
            Self::Lockbox => 10,
        }
    }

    /// Whether this container type can be locked
    pub fn can_lock(&self) -> bool {
        matches!(self, Self::ReinforcedChest | Self::Lockbox | Self::Trunk)
    }

    /// Display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::WoodenChest => "Wooden Chest",
            Self::ReinforcedChest => "Reinforced Chest",
            Self::Crate => "Crate",
            Self::Barrel => "Barrel",
            Self::Trunk => "Storage Trunk",
            Self::Lockbox => "Lockbox",
        }
    }

    /// Model identifier for rendering
    pub fn model_id(&self) -> &'static str {
        match self {
            Self::WoodenChest => "chest_wooden",
            Self::ReinforcedChest => "chest_reinforced",
            Self::Crate => "crate",
            Self::Barrel => "barrel",
            Self::Trunk => "trunk",
            Self::Lockbox => "lockbox",
        }
    }
}

/// A storage container in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageContainer {
    /// Unique ID
    pub id: ContainerId,
    /// Container type
    pub container_type: ContainerType,
    /// World position
    pub position: Vec3,
    /// Rotation in radians (Y-axis)
    pub rotation: f32,
    /// Item slots
    pub slots: Vec<Option<Item>>,
    /// Whether the container is locked
    pub is_locked: bool,
    /// Owner player ID (None = public)
    pub owner: Option<u64>,
    /// Whether the lid is open (for animation)
    pub is_open: bool,
    /// Custom name (optional)
    pub custom_name: Option<String>,
}

impl StorageContainer {
    /// Create a new storage container
    pub fn new(container_type: ContainerType, position: Vec3, rotation: f32) -> Self {
        let slot_count = container_type.slot_count();
        Self {
            id: ContainerId::generate(),
            container_type,
            position,
            rotation,
            slots: vec![None; slot_count],
            is_locked: false,
            owner: None,
            is_open: false,
            custom_name: None,
        }
    }

    /// Create a new container with a specific ID (for loading saves)
    pub fn with_id(id: ContainerId, container_type: ContainerType, position: Vec3, rotation: f32) -> Self {
        let slot_count = container_type.slot_count();
        Self {
            id,
            container_type,
            position,
            rotation,
            slots: vec![None; slot_count],
            is_locked: false,
            owner: None,
            is_open: false,
            custom_name: None,
        }
    }

    /// Get the display name (custom or default)
    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or_else(|| self.container_type.display_name())
    }

    /// Add an item to the container
    pub fn add_item(&mut self, item: Item) -> Result<usize, StorageError> {
        // Try to stack with existing items first
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
                *slot = Some(item);
                return Ok(idx);
            }
        }

        Err(StorageError::Full)
    }

    /// Remove an item by slot index
    pub fn remove_item(&mut self, slot_index: usize) -> Option<Item> {
        if slot_index < self.slots.len() {
            self.slots[slot_index].take()
        } else {
            None
        }
    }

    /// Remove an item by ID
    pub fn remove_item_by_id(&mut self, id: ItemId) -> Option<Item> {
        for slot in &mut self.slots {
            if let Some(item) = slot {
                if item.id == id {
                    return slot.take();
                }
            }
        }
        None
    }

    /// Get an item by slot index
    pub fn get_item(&self, slot_index: usize) -> Option<&Item> {
        self.slots.get(slot_index).and_then(|s| s.as_ref())
    }

    /// Get mutable item by slot index
    pub fn get_item_mut(&mut self, slot_index: usize) -> Option<&mut Item> {
        self.slots.get_mut(slot_index).and_then(|s| s.as_mut())
    }

    /// Count items by template ID
    pub fn count_items(&self, template_id: &str) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|i| i.template_id == template_id)
            .map(|i| i.stack_size)
            .sum()
    }

    /// Get number of free slots
    pub fn free_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_none()).count()
    }

    /// Check if container is empty
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|s| s.is_none())
    }

    /// Check if container is full
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }

    /// Total number of items (counting stacks)
    pub fn total_items(&self) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|i| i.stack_size)
            .sum()
    }

    /// Get all items as iterator
    pub fn all_items(&self) -> impl Iterator<Item = &Item> {
        self.slots.iter().filter_map(|s| s.as_ref())
    }

    /// Swap two slots
    pub fn swap_slots(&mut self, a: usize, b: usize) -> Result<(), StorageError> {
        if a >= self.slots.len() || b >= self.slots.len() {
            return Err(StorageError::InvalidSlot);
        }
        self.slots.swap(a, b);
        Ok(())
    }

    /// Check if player can access (unlock check)
    pub fn can_access(&self, player_id: Option<u64>) -> bool {
        if !self.is_locked {
            return true;
        }
        match (self.owner, player_id) {
            (None, _) => true,
            (Some(owner), Some(player)) => owner == player,
            (Some(_), None) => false,
        }
    }

    /// Open the container (for animation)
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Close the container (for animation)
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Distance from a point
    pub fn distance_from(&self, point: Vec3) -> f32 {
        self.position.distance(point)
    }
}

/// Storage-related errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    Full,
    Locked,
    InvalidSlot,
    NotFound,
    NotOwner,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "Container is full"),
            Self::Locked => write!(f, "Container is locked"),
            Self::InvalidSlot => write!(f, "Invalid slot"),
            Self::NotFound => write!(f, "Container not found"),
            Self::NotOwner => write!(f, "Not the owner of this container"),
        }
    }
}

impl std::error::Error for StorageError {}

/// Manages all storage containers in the world
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageManager {
    /// All containers by ID
    containers: HashMap<ContainerId, StorageContainer>,
    /// Spatial index: chunk coord -> container IDs
    #[serde(skip)]
    spatial_index: HashMap<(i32, i32), Vec<ContainerId>>,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
            spatial_index: HashMap::new(),
        }
    }

    /// Calculate chunk coordinate from world position
    fn chunk_coord(pos: Vec3) -> (i32, i32) {
        let chunk_size = 64.0; // Match game chunk size
        (
            (pos.x / chunk_size).floor() as i32,
            (pos.z / chunk_size).floor() as i32,
        )
    }

    /// Add a new container to the world
    pub fn add_container(&mut self, container: StorageContainer) -> ContainerId {
        let id = container.id;
        let chunk = Self::chunk_coord(container.position);

        self.containers.insert(id, container);
        self.spatial_index.entry(chunk).or_default().push(id);

        id
    }

    /// Spawn a new container at a position
    pub fn spawn_container(
        &mut self,
        container_type: ContainerType,
        position: Vec3,
        rotation: f32,
    ) -> ContainerId {
        let container = StorageContainer::new(container_type, position, rotation);
        self.add_container(container)
    }

    /// Remove a container by ID
    pub fn remove_container(&mut self, id: ContainerId) -> Option<StorageContainer> {
        if let Some(container) = self.containers.remove(&id) {
            let chunk = Self::chunk_coord(container.position);
            if let Some(ids) = self.spatial_index.get_mut(&chunk) {
                ids.retain(|&cid| cid != id);
            }
            Some(container)
        } else {
            None
        }
    }

    /// Get a container by ID
    pub fn get(&self, id: ContainerId) -> Option<&StorageContainer> {
        self.containers.get(&id)
    }

    /// Get a mutable container by ID
    pub fn get_mut(&mut self, id: ContainerId) -> Option<&mut StorageContainer> {
        self.containers.get_mut(&id)
    }

    /// Find containers near a position
    pub fn containers_near(&self, position: Vec3, radius: f32) -> Vec<&StorageContainer> {
        let mut result = Vec::new();
        let radius_sq = radius * radius;

        // Check nearby chunks
        let center_chunk = Self::chunk_coord(position);
        let chunk_radius = (radius / 64.0).ceil() as i32 + 1;

        for dx in -chunk_radius..=chunk_radius {
            for dz in -chunk_radius..=chunk_radius {
                let chunk = (center_chunk.0 + dx, center_chunk.1 + dz);
                if let Some(ids) = self.spatial_index.get(&chunk) {
                    for &id in ids {
                        if let Some(container) = self.containers.get(&id) {
                            let dist_sq = container.position.distance_squared(position);
                            if dist_sq <= radius_sq {
                                result.push(container);
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Find the nearest container to a position within max distance
    pub fn nearest_container(&self, position: Vec3, max_distance: f32) -> Option<&StorageContainer> {
        self.containers_near(position, max_distance)
            .into_iter()
            .min_by(|a, b| {
                a.distance_from(position)
                    .partial_cmp(&b.distance_from(position))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Find the nearest container that the player can interact with
    pub fn nearest_interactable(
        &self,
        position: Vec3,
        player_id: Option<u64>,
        max_distance: f32,
    ) -> Option<&StorageContainer> {
        self.containers_near(position, max_distance)
            .into_iter()
            .filter(|c| c.can_access(player_id))
            .min_by(|a, b| {
                a.distance_from(position)
                    .partial_cmp(&b.distance_from(position))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get all container IDs
    pub fn all_ids(&self) -> impl Iterator<Item = ContainerId> + '_ {
        self.containers.keys().copied()
    }

    /// Get all containers
    pub fn all_containers(&self) -> impl Iterator<Item = &StorageContainer> {
        self.containers.values()
    }

    /// Get all containers mutably
    pub fn all_containers_mut(&mut self) -> impl Iterator<Item = &mut StorageContainer> {
        self.containers.values_mut()
    }

    /// Total number of containers
    pub fn count(&self) -> usize {
        self.containers.len()
    }

    /// Rebuild spatial index (call after loading)
    pub fn rebuild_spatial_index(&mut self) {
        self.spatial_index.clear();
        for (&id, container) in &self.containers {
            let chunk = Self::chunk_coord(container.position);
            self.spatial_index.entry(chunk).or_default().push(id);
        }
    }

    /// Get containers in a specific chunk
    pub fn containers_in_chunk(&self, chunk_x: i32, chunk_z: i32) -> Vec<&StorageContainer> {
        self.spatial_index
            .get(&(chunk_x, chunk_z))
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.containers.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_creation() {
        let container = StorageContainer::new(
            ContainerType::WoodenChest,
            Vec3::new(10.0, 0.0, 20.0),
            0.0,
        );

        assert_eq!(container.slots.len(), 20);
        assert!(container.is_empty());
        assert!(!container.is_full());
        assert_eq!(container.free_slots(), 20);
    }

    #[test]
    fn test_container_manager() {
        let mut manager = StorageManager::new();

        let id = manager.spawn_container(
            ContainerType::Crate,
            Vec3::new(100.0, 0.0, 100.0),
            0.0,
        );

        assert_eq!(manager.count(), 1);
        assert!(manager.get(id).is_some());

        let nearby = manager.containers_near(Vec3::new(100.0, 0.0, 100.0), 10.0);
        assert_eq!(nearby.len(), 1);
    }
}
