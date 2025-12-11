//! Resource Gathering System
//!
//! Handles player gathering of natural resources:
//! - Wood (logging trees)
//! - Stone (quarrying)
//! - Iron ore (mining deposits)
//! - Gold (panning rivers, rare deposits)
//!
//! Resources can be delivered to settlements for construction and trade.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use glam::Vec3;
use noise::{NoiseFn, Perlin};

// ============================================================================
// RESOURCE NODE TYPES
// ============================================================================

/// Types of gatherable resources in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatherableResource {
    // Wood types
    Pine,
    Oak,
    Hickory,
    Cedar,
    Walnut,
    Cypress,

    // Stone types
    Granite,
    Limestone,
    Sandstone,
    Slate,

    // Metal ores
    IronOre,
    CopperOre,
    GoldNugget,
    SilverOre,

    // Other
    Clay,
    Flint,
    Coal,
    Salt,
}

impl GatherableResource {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pine => "Pine",
            Self::Oak => "Oak",
            Self::Hickory => "Hickory",
            Self::Cedar => "Cedar",
            Self::Walnut => "Walnut",
            Self::Cypress => "Cypress",
            Self::Granite => "Granite",
            Self::Limestone => "Limestone",
            Self::Sandstone => "Sandstone",
            Self::Slate => "Slate",
            Self::IronOre => "Iron Ore",
            Self::CopperOre => "Copper Ore",
            Self::GoldNugget => "Gold Nugget",
            Self::SilverOre => "Silver Ore",
            Self::Clay => "Clay",
            Self::Flint => "Flint",
            Self::Coal => "Coal",
            Self::Salt => "Salt",
        }
    }

    pub fn category(&self) -> ResourceCategory {
        match self {
            Self::Pine | Self::Oak | Self::Hickory | Self::Cedar
            | Self::Walnut | Self::Cypress => ResourceCategory::Wood,
            Self::Granite | Self::Limestone | Self::Sandstone | Self::Slate => ResourceCategory::Stone,
            Self::IronOre | Self::CopperOre | Self::GoldNugget | Self::SilverOre => ResourceCategory::Ore,
            Self::Clay | Self::Flint | Self::Coal | Self::Salt => ResourceCategory::Mineral,
        }
    }

    /// Time to gather one unit (seconds)
    pub fn gather_time(&self) -> f32 {
        match self.category() {
            ResourceCategory::Wood => 5.0,   // Chopping
            ResourceCategory::Stone => 8.0,  // Quarrying
            ResourceCategory::Ore => 10.0,   // Mining
            ResourceCategory::Mineral => 3.0, // Digging
        }
    }

    /// Base yield per node
    pub fn base_yield(&self) -> u32 {
        match self {
            Self::Pine => 10,
            Self::Oak => 15,
            Self::Hickory => 12,
            Self::Cedar => 8,
            Self::Walnut => 10,
            Self::Cypress => 12,
            Self::Granite => 20,
            Self::Limestone => 25,
            Self::Sandstone => 30,
            Self::Slate => 15,
            Self::IronOre => 8,
            Self::CopperOre => 6,
            Self::GoldNugget => 1,
            Self::SilverOre => 3,
            Self::Clay => 40,
            Self::Flint => 10,
            Self::Coal => 15,
            Self::Salt => 8,
        }
    }

    /// Base value per unit
    pub fn base_value(&self) -> u32 {
        match self {
            Self::Pine => 1,
            Self::Oak => 2,
            Self::Hickory => 2,
            Self::Cedar => 3,
            Self::Walnut => 4,
            Self::Cypress => 3,
            Self::Granite => 2,
            Self::Limestone => 1,
            Self::Sandstone => 1,
            Self::Slate => 3,
            Self::IronOre => 5,
            Self::CopperOre => 4,
            Self::GoldNugget => 50,
            Self::SilverOre => 20,
            Self::Clay => 1,
            Self::Flint => 2,
            Self::Coal => 3,
            Self::Salt => 4,
        }
    }

    /// Tool required to gather efficiently
    pub fn required_tool(&self) -> Option<GatheringTool> {
        match self.category() {
            ResourceCategory::Wood => Some(GatheringTool::Axe),
            ResourceCategory::Stone => Some(GatheringTool::Pickaxe),
            ResourceCategory::Ore => Some(GatheringTool::Pickaxe),
            ResourceCategory::Mineral => Some(GatheringTool::Shovel),
        }
    }

    /// Rarity/spawn weight (higher = more common)
    pub fn spawn_weight(&self) -> f32 {
        match self {
            Self::Pine => 30.0,
            Self::Oak => 25.0,
            Self::Hickory => 15.0,
            Self::Cedar => 10.0,
            Self::Walnut => 8.0,
            Self::Cypress => 12.0, // Swamps
            Self::Granite => 20.0,
            Self::Limestone => 25.0,
            Self::Sandstone => 15.0,
            Self::Slate => 10.0,
            Self::IronOre => 8.0,
            Self::CopperOre => 5.0,
            Self::GoldNugget => 0.5, // Very rare
            Self::SilverOre => 1.0,  // Rare
            Self::Clay => 30.0,
            Self::Flint => 15.0,
            Self::Coal => 10.0,
            Self::Salt => 5.0,
        }
    }

    /// Preferred biome for this resource
    pub fn preferred_biomes(&self) -> Vec<BiomeType> {
        match self {
            Self::Pine => vec![BiomeType::PineForest, BiomeType::Mountain],
            Self::Oak => vec![BiomeType::DeciduousForest, BiomeType::Meadow],
            Self::Hickory => vec![BiomeType::DeciduousForest],
            Self::Cedar => vec![BiomeType::PineForest, BiomeType::Swamp],
            Self::Walnut => vec![BiomeType::DeciduousForest, BiomeType::RiverValley],
            Self::Cypress => vec![BiomeType::Swamp, BiomeType::CoastalMarsh],
            Self::Granite => vec![BiomeType::Mountain, BiomeType::DeciduousForest],
            Self::Limestone => vec![BiomeType::RiverValley, BiomeType::CoastalMarsh],
            Self::Sandstone => vec![BiomeType::CoastalMarsh, BiomeType::Meadow],
            Self::Slate => vec![BiomeType::Mountain],
            Self::IronOre => vec![BiomeType::Mountain, BiomeType::DeciduousForest],
            Self::CopperOre => vec![BiomeType::Mountain],
            Self::GoldNugget => vec![BiomeType::RiverValley, BiomeType::Mountain],
            Self::SilverOre => vec![BiomeType::Mountain],
            Self::Clay => vec![BiomeType::RiverValley, BiomeType::Swamp],
            Self::Flint => vec![BiomeType::DeciduousForest, BiomeType::RiverValley],
            Self::Coal => vec![BiomeType::Mountain, BiomeType::PineForest],
            Self::Salt => vec![BiomeType::CoastalMarsh],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceCategory {
    Wood,
    Stone,
    Ore,
    Mineral,
}

impl ResourceCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Wood => "Wood",
            Self::Stone => "Stone",
            Self::Ore => "Ore",
            Self::Mineral => "Mineral",
        }
    }

    pub fn gathering_verb(&self) -> &'static str {
        match self {
            Self::Wood => "Chopping",
            Self::Stone => "Quarrying",
            Self::Ore => "Mining",
            Self::Mineral => "Digging",
        }
    }
}

/// Biome types for resource spawning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    DeciduousForest,
    PineForest,
    Swamp,
    CoastalMarsh,
    Meadow,
    RiverValley,
    Mountain,
}

// ============================================================================
// GATHERING TOOLS
// ============================================================================

/// Tools that affect gathering efficiency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatheringTool {
    Axe,
    Pickaxe,
    Shovel,
    Pan, // For gold panning
}

impl GatheringTool {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Axe => "Axe",
            Self::Pickaxe => "Pickaxe",
            Self::Shovel => "Shovel",
            Self::Pan => "Pan",
        }
    }

    /// Efficiency multiplier for different tool qualities
    pub fn efficiency_for_quality(&self, quality: ToolQuality) -> f32 {
        let base = match self {
            Self::Axe => 1.0,
            Self::Pickaxe => 1.0,
            Self::Shovel => 1.0,
            Self::Pan => 0.8,
        };

        base * quality.multiplier()
    }
}

/// Quality levels for gathering tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolQuality {
    Makeshift,  // Stone/bone tools
    Basic,      // Iron tools
    Good,       // Steel tools
    Fine,       // Well-crafted steel
    Masterwork, // Expert craftsmanship
}

impl ToolQuality {
    pub fn multiplier(&self) -> f32 {
        match self {
            Self::Makeshift => 0.5,
            Self::Basic => 1.0,
            Self::Good => 1.5,
            Self::Fine => 2.0,
            Self::Masterwork => 3.0,
        }
    }
}

// ============================================================================
// RESOURCE NODES
// ============================================================================

/// A gatherable resource node in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: u64,
    pub resource: GatherableResource,
    pub position: [f32; 3],
    /// Remaining amount to gather
    pub remaining: u32,
    /// Original capacity
    pub capacity: u32,
    /// Quality affects yield
    pub quality: f32, // 0.5 - 2.0
    /// Visual scale
    pub scale: f32,
    /// Regenerates over time?
    pub regenerating: bool,
    /// Time since last gathered (for regeneration)
    pub last_gathered: f32,
}

impl ResourceNode {
    pub fn new(id: u64, resource: GatherableResource, position: [f32; 3]) -> Self {
        let base_yield = resource.base_yield();
        let quality = 0.8 + rand_float() * 0.4; // 0.8 - 1.2
        let capacity = (base_yield as f32 * quality) as u32;

        Self {
            id,
            resource,
            position,
            remaining: capacity,
            capacity,
            quality,
            scale: 0.9 + rand_float() * 0.2,
            regenerating: matches!(
                resource.category(),
                ResourceCategory::Wood | ResourceCategory::Mineral
            ),
            last_gathered: 0.0,
        }
    }

    /// Check if node is depleted
    pub fn is_depleted(&self) -> bool {
        self.remaining == 0
    }

    /// Gather resources from this node
    pub fn gather(&mut self, tool_efficiency: f32, gather_time: f32) -> GatherResult {
        if self.is_depleted() {
            return GatherResult {
                resource: self.resource,
                amount: 0,
                quality: self.quality,
                node_depleted: true,
            };
        }

        // Calculate yield based on tool and time spent
        let base_gather = (gather_time / self.resource.gather_time()).floor() as u32;
        let amount = ((base_gather as f32 * tool_efficiency * self.quality) as u32)
            .min(self.remaining)
            .max(1);

        self.remaining = self.remaining.saturating_sub(amount);
        self.last_gathered = 0.0;

        GatherResult {
            resource: self.resource,
            amount,
            quality: self.quality,
            node_depleted: self.remaining == 0,
        }
    }

    /// Update node (regeneration)
    pub fn update(&mut self, delta_days: f32) {
        if !self.regenerating || self.remaining >= self.capacity {
            return;
        }

        self.last_gathered += delta_days;

        // Regenerate after 7 days
        if self.last_gathered > 7.0 {
            let regen_rate = match self.resource.category() {
                ResourceCategory::Wood => 0.1,    // Trees regrow slowly
                ResourceCategory::Mineral => 0.2, // Clay/etc refills
                _ => 0.0,                         // Stone/ore don't regenerate
            };

            let regen = (self.capacity as f32 * regen_rate * delta_days) as u32;
            self.remaining = (self.remaining + regen).min(self.capacity);
        }
    }

    /// Get display name with quality
    pub fn display_name(&self) -> String {
        let quality_desc = if self.quality > 1.3 {
            "Rich "
        } else if self.quality > 1.1 {
            "Good "
        } else if self.quality < 0.7 {
            "Poor "
        } else {
            ""
        };

        format!("{}{}", quality_desc, self.resource.name())
    }
}

/// Result of a gathering action
#[derive(Debug, Clone)]
pub struct GatherResult {
    pub resource: GatherableResource,
    pub amount: u32,
    pub quality: f32,
    pub node_depleted: bool,
}

// ============================================================================
// RESOURCE MANAGER
// ============================================================================

/// Manages all resource nodes in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGatheringManager {
    /// All resource nodes by chunk
    nodes_by_chunk: HashMap<(i32, i32), Vec<ResourceNode>>,
    /// Next node ID
    next_node_id: u64,
    /// Noise generator for procedural spawning
    #[serde(skip)]
    spawn_noise: Option<Perlin>,
    /// Player's current gathering progress
    pub current_gathering: Option<GatheringProgress>,
    /// Statistics
    pub stats: GatheringStats,
}

impl Default for ResourceGatheringManager {
    fn default() -> Self {
        Self::new(12345)
    }
}

impl ResourceGatheringManager {
    pub fn new(seed: u32) -> Self {
        Self {
            nodes_by_chunk: HashMap::new(),
            next_node_id: 0,
            spawn_noise: Some(Perlin::new(seed)),
            current_gathering: None,
            stats: GatheringStats::default(),
        }
    }

    /// Generate resource nodes for a chunk
    pub fn generate_chunk(&mut self, chunk_x: i32, chunk_z: i32, biome: BiomeType) {
        let chunk_key = (chunk_x, chunk_z);
        if self.nodes_by_chunk.contains_key(&chunk_key) {
            return;
        }

        let mut nodes = Vec::new();
        let chunk_size = 64.0;
        let chunk_world_x = chunk_x as f32 * chunk_size;
        let chunk_world_z = chunk_z as f32 * chunk_size;

        // Get eligible resources for this biome
        let eligible_resources: Vec<GatherableResource> = [
            GatherableResource::Pine,
            GatherableResource::Oak,
            GatherableResource::Hickory,
            GatherableResource::Cedar,
            GatherableResource::Walnut,
            GatherableResource::Cypress,
            GatherableResource::Granite,
            GatherableResource::Limestone,
            GatherableResource::Sandstone,
            GatherableResource::Slate,
            GatherableResource::IronOre,
            GatherableResource::CopperOre,
            GatherableResource::GoldNugget,
            GatherableResource::SilverOre,
            GatherableResource::Clay,
            GatherableResource::Flint,
            GatherableResource::Coal,
            GatherableResource::Salt,
        ]
        .into_iter()
        .filter(|r| r.preferred_biomes().contains(&biome))
        .collect();

        if eligible_resources.is_empty() {
            self.nodes_by_chunk.insert(chunk_key, nodes);
            return;
        }

        // Spawn density based on biome
        let density = match biome {
            BiomeType::DeciduousForest | BiomeType::PineForest => 0.4,
            BiomeType::Mountain => 0.3,
            BiomeType::RiverValley => 0.3,
            BiomeType::Swamp | BiomeType::CoastalMarsh => 0.2,
            BiomeType::Meadow => 0.15,
        };

        // Generate nodes procedurally
        if let Some(ref noise) = self.spawn_noise {
            let num_attempts = (density * 20.0) as usize;

            for i in 0..num_attempts {
                // Position within chunk
                let local_x = ((chunk_x * 1000 + i as i32 * 7) % 1000) as f32 / 1000.0 * chunk_size;
                let local_z = ((chunk_z * 1000 + i as i32 * 13) % 1000) as f32 / 1000.0 * chunk_size;
                let world_x = chunk_world_x + local_x;
                let world_z = chunk_world_z + local_z;

                // Use noise to determine if resource spawns here
                let spawn_value = noise.get([
                    world_x as f64 * 0.02,
                    world_z as f64 * 0.02,
                    i as f64 * 0.1,
                ]);

                if spawn_value > 0.3 {
                    // Select resource based on weights
                    let total_weight: f32 = eligible_resources.iter().map(|r| r.spawn_weight()).sum();
                    let mut roll = rand_float() * total_weight;

                    for resource in &eligible_resources {
                        roll -= resource.spawn_weight();
                        if roll <= 0.0 {
                            let node = ResourceNode::new(
                                self.next_node_id,
                                *resource,
                                [world_x, 0.0, world_z], // Y will be set by terrain
                            );
                            self.next_node_id += 1;
                            nodes.push(node);
                            break;
                        }
                    }
                }
            }
        }

        self.nodes_by_chunk.insert(chunk_key, nodes);
    }

    /// Get resource nodes near a position
    pub fn get_nodes_near(&self, position: [f32; 3], radius: f32) -> Vec<&ResourceNode> {
        let chunk_size = 64.0;
        let center_chunk_x = (position[0] / chunk_size).floor() as i32;
        let center_chunk_z = (position[2] / chunk_size).floor() as i32;

        let mut nearby = Vec::new();
        let radius_sq = radius * radius;

        for dx in -1..=1 {
            for dz in -1..=1 {
                let chunk_key = (center_chunk_x + dx, center_chunk_z + dz);
                if let Some(nodes) = self.nodes_by_chunk.get(&chunk_key) {
                    for node in nodes {
                        let dist_sq = (node.position[0] - position[0]).powi(2)
                            + (node.position[2] - position[2]).powi(2);
                        if dist_sq <= radius_sq && !node.is_depleted() {
                            nearby.push(node);
                        }
                    }
                }
            }
        }

        nearby
    }

    /// Get closest gatherable node to player
    pub fn get_closest_node(
        &self,
        position: [f32; 3],
        max_distance: f32,
    ) -> Option<(u64, &ResourceNode, f32)> {
        let nearby = self.get_nodes_near(position, max_distance);

        nearby
            .into_iter()
            .map(|node| {
                let dist = ((node.position[0] - position[0]).powi(2)
                    + (node.position[2] - position[2]).powi(2))
                .sqrt();
                (node.id, node, dist)
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
    }

    /// Start gathering from a node
    pub fn start_gathering(
        &mut self,
        node_id: u64,
        tool: Option<GatheringTool>,
        tool_quality: ToolQuality,
    ) -> bool {
        // Find the node
        let node = self.find_node_mut(node_id);
        if node.is_none() {
            return false;
        }

        let node = node.unwrap();
        if node.is_depleted() {
            return false;
        }

        let resource = node.resource;
        let required_tool = resource.required_tool();

        // Calculate efficiency
        let efficiency = if let Some(req_tool) = required_tool {
            if tool == Some(req_tool) {
                tool_quality.multiplier()
            } else if tool.is_some() {
                // Wrong tool, reduced efficiency
                tool_quality.multiplier() * 0.3
            } else {
                // No tool, very slow
                0.2
            }
        } else {
            // No tool required
            tool_quality.multiplier()
        };

        self.current_gathering = Some(GatheringProgress {
            node_id,
            resource,
            progress: 0.0,
            gather_time: resource.gather_time() / efficiency,
            efficiency,
        });

        true
    }

    /// Update gathering progress
    pub fn update_gathering(&mut self, delta_seconds: f32) -> Option<GatherResult> {
        let progress = self.current_gathering.as_mut()?;

        progress.progress += delta_seconds;

        if progress.progress >= progress.gather_time {
            let node_id = progress.node_id;
            let efficiency = progress.efficiency;
            let gather_time = progress.gather_time;

            self.current_gathering = None;

            // Perform the gather
            if let Some(node) = self.find_node_mut(node_id) {
                let result = node.gather(efficiency, gather_time);

                // Update stats
                self.stats.total_gathered += result.amount;
                match result.resource.category() {
                    ResourceCategory::Wood => self.stats.wood_gathered += result.amount,
                    ResourceCategory::Stone => self.stats.stone_gathered += result.amount,
                    ResourceCategory::Ore => self.stats.ore_gathered += result.amount,
                    ResourceCategory::Mineral => self.stats.minerals_gathered += result.amount,
                }

                return Some(result);
            }
        }

        None
    }

    /// Cancel current gathering
    pub fn cancel_gathering(&mut self) {
        self.current_gathering = None;
    }

    /// Find a node by ID (mutable)
    fn find_node_mut(&mut self, node_id: u64) -> Option<&mut ResourceNode> {
        for nodes in self.nodes_by_chunk.values_mut() {
            for node in nodes {
                if node.id == node_id {
                    return Some(node);
                }
            }
        }
        None
    }

    /// Update all nodes (regeneration)
    pub fn update(&mut self, delta_days: f32) {
        for nodes in self.nodes_by_chunk.values_mut() {
            for node in nodes {
                node.update(delta_days);
            }
        }
    }

    /// Get gathering progress percentage
    pub fn get_gathering_progress(&self) -> Option<f32> {
        self.current_gathering.as_ref().map(|p| {
            (p.progress / p.gather_time * 100.0).min(100.0)
        })
    }
}

/// Current gathering progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatheringProgress {
    pub node_id: u64,
    pub resource: GatherableResource,
    pub progress: f32,
    pub gather_time: f32,
    pub efficiency: f32,
}

/// Statistics for resource gathering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatheringStats {
    pub total_gathered: u32,
    pub wood_gathered: u32,
    pub stone_gathered: u32,
    pub ore_gathered: u32,
    pub minerals_gathered: u32,
    pub nodes_depleted: u32,
}

// ============================================================================
// LUMBER CAMP / MINING OUTPOST
// ============================================================================

/// A resource gathering camp (automated gathering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatheringCamp {
    pub id: u32,
    pub name: String,
    pub position: [f32; 3],
    pub camp_type: GatheringCampType,
    pub workers: u32,
    pub max_workers: u32,
    pub efficiency: f32,
    pub storage: u32,
    pub max_storage: u32,
    /// Resources gathered but not yet delivered
    pub pending_resources: HashMap<GatherableResource, u32>,
    /// Connected settlement for delivery
    pub connected_settlement: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatheringCampType {
    LumberCamp,
    Quarry,
    Mine,
    GoldPanningCamp,
}

impl GatheringCamp {
    pub fn new(id: u32, name: &str, position: [f32; 3], camp_type: GatheringCampType) -> Self {
        Self {
            id,
            name: name.to_string(),
            position,
            camp_type,
            workers: 0,
            max_workers: match camp_type {
                GatheringCampType::LumberCamp => 10,
                GatheringCampType::Quarry => 8,
                GatheringCampType::Mine => 6,
                GatheringCampType::GoldPanningCamp => 4,
            },
            efficiency: 1.0,
            storage: 0,
            max_storage: 500,
            pending_resources: HashMap::new(),
            connected_settlement: None,
        }
    }

    /// Daily resource production
    pub fn daily_production(&self) -> HashMap<GatherableResource, u32> {
        let mut production = HashMap::new();

        if self.workers == 0 {
            return production;
        }

        let base_amount = (self.workers as f32 * self.efficiency * 5.0) as u32;

        match self.camp_type {
            GatheringCampType::LumberCamp => {
                production.insert(GatherableResource::Oak, base_amount);
            }
            GatheringCampType::Quarry => {
                production.insert(GatherableResource::Granite, base_amount);
            }
            GatheringCampType::Mine => {
                production.insert(GatherableResource::IronOre, base_amount / 2);
                // Small chance of other ores
                if rand_float() < 0.1 {
                    production.insert(GatherableResource::CopperOre, base_amount / 4);
                }
            }
            GatheringCampType::GoldPanningCamp => {
                // Very low gold yield
                if rand_float() < 0.3 {
                    production.insert(GatherableResource::GoldNugget, 1);
                }
            }
        }

        production
    }

    /// Update camp (daily tick)
    pub fn daily_update(&mut self) {
        let production = self.daily_production();

        for (resource, amount) in production {
            let entry = self.pending_resources.entry(resource).or_insert(0);
            *entry = (*entry + amount).min(self.max_storage);
            self.storage = self.pending_resources.values().sum();
        }
    }

    /// Collect pending resources
    pub fn collect(&mut self) -> HashMap<GatherableResource, u32> {
        let collected = self.pending_resources.clone();
        self.pending_resources.clear();
        self.storage = 0;
        collected
    }

    /// Connect camp to a settlement for automatic delivery
    pub fn connect_to_settlement(&mut self, settlement_id: u32) {
        self.connected_settlement = Some(settlement_id);
    }

    /// Disconnect from settlement
    pub fn disconnect(&mut self) {
        self.connected_settlement = None;
    }

    /// Get camp type display name
    pub fn type_name(&self) -> &'static str {
        match self.camp_type {
            GatheringCampType::LumberCamp => "Lumber Camp",
            GatheringCampType::Quarry => "Stone Quarry",
            GatheringCampType::Mine => "Mine",
            GatheringCampType::GoldPanningCamp => "Gold Panning Camp",
        }
    }

    /// Get storage utilization percentage
    pub fn storage_percentage(&self) -> f32 {
        (self.storage as f32 / self.max_storage as f32 * 100.0).min(100.0)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn rand_float() -> f32 {
    // Simple PRNG - in real implementation, use proper RNG
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}
