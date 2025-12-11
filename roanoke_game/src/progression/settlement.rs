//! Settlement and Colony Building System
//!
//! Allows players to establish and grow settlements of different cultural styles:
//! - English colonial settlements (forts, plantations)
//! - Spanish missions and presidios
//! - French trading posts
//! - Native-style villages (if adopted into tribe)
//!
//! Buildings require resources (wood, stone, iron) and laborers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use glam::Vec3;

use super::faction::Faction;
use super::resource_gathering::{GatherableResource, ResourceCategory, GatherResult};
use crate::economy::item::ItemType;

// ============================================================================
// SETTLEMENT TYPES
// ============================================================================

/// Type of settlement based on founding culture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettlementStyle {
    /// English colonial - wooden palisades, thatched cottages
    EnglishColonial,
    /// Spanish mission - adobe/stone, church-centered
    SpanishMission,
    /// Spanish fort - stone walls, military focus
    SpanishPresidio,
    /// French trading post - log cabins, warehouse
    FrenchTradingPost,
    /// Native longhouse village
    NativeVillage,
    /// Mixed/frontier style
    Frontier,
}

impl SettlementStyle {
    pub fn available_buildings(&self) -> Vec<BuildingType> {
        match self {
            Self::EnglishColonial => vec![
                BuildingType::WoodenPalisade,
                BuildingType::Cottage,
                BuildingType::Storehouse,
                BuildingType::Blacksmith,
                BuildingType::Chapel,
                BuildingType::Well,
                BuildingType::Farm,
                BuildingType::LumberMill,
                BuildingType::Dock,
                BuildingType::TownHall,
                BuildingType::Barracks,
                BuildingType::Tavern,
            ],
            Self::SpanishMission => vec![
                BuildingType::AdobeWall,
                BuildingType::MissionChurch,
                BuildingType::Convento,
                BuildingType::Workshop,
                BuildingType::Granary,
                BuildingType::Well,
                BuildingType::Farm,
                BuildingType::Vineyard,
                BuildingType::BellTower,
            ],
            Self::SpanishPresidio => vec![
                BuildingType::StoneWall,
                BuildingType::Watchtower,
                BuildingType::Barracks,
                BuildingType::Armory,
                BuildingType::CommandersQuarters,
                BuildingType::Stable,
                BuildingType::Blacksmith,
                BuildingType::Well,
                BuildingType::Storehouse,
            ],
            Self::FrenchTradingPost => vec![
                BuildingType::LogCabin,
                BuildingType::TradingHouse,
                BuildingType::FurWarehouse,
                BuildingType::Dock,
                BuildingType::Smokehouse,
                BuildingType::Well,
                BuildingType::Tavern,
            ],
            Self::NativeVillage => vec![
                BuildingType::Longhouse,
                BuildingType::CouncilHouse,
                BuildingType::SweatLodge,
                BuildingType::StoragePit,
                BuildingType::DryingRack,
                BuildingType::CornField,
                BuildingType::PalisadeWall,
            ],
            Self::Frontier => vec![
                BuildingType::LogCabin,
                BuildingType::WoodenPalisade,
                BuildingType::Storehouse,
                BuildingType::Well,
                BuildingType::Farm,
                BuildingType::Smokehouse,
                BuildingType::Blacksmith,
            ],
        }
    }

    pub fn default_faction(&self) -> Faction {
        match self {
            Self::EnglishColonial => Faction::English,
            Self::SpanishMission | Self::SpanishPresidio => Faction::Spanish,
            Self::FrenchTradingPost => Faction::French,
            Self::NativeVillage => Faction::Powhatan,
            Self::Frontier => Faction::Independent,
        }
    }
}

// ============================================================================
// BUILDING TYPES
// ============================================================================

/// All building types available in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    // Defensive
    WoodenPalisade,
    StoneWall,
    AdobeWall,
    PalisadeWall,
    Watchtower,
    Gate,

    // Residential
    Cottage,
    LogCabin,
    Longhouse,
    Convento,
    CommandersQuarters,

    // Production
    LumberMill,
    Blacksmith,
    Workshop,
    Smokehouse,
    DryingRack,
    Vineyard,
    Farm,
    CornField,

    // Storage
    Storehouse,
    Granary,
    FurWarehouse,
    StoragePit,
    Armory,

    // Religious/Cultural
    Chapel,
    MissionChurch,
    BellTower,
    CouncilHouse,
    SweatLodge,

    // Military
    Barracks,
    Stable,
    TrainingGround,

    // Trade/Economy
    TradingHouse,
    Dock,
    Market,
    Tavern,

    // Infrastructure
    Well,
    Road,
    Bridge,

    // Administrative
    TownHall,
    GovernorsHouse,
}

impl BuildingType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::WoodenPalisade => "Wooden Palisade",
            Self::StoneWall => "Stone Wall",
            Self::AdobeWall => "Adobe Wall",
            Self::PalisadeWall => "Palisade Wall",
            Self::Watchtower => "Watchtower",
            Self::Gate => "Gate",
            Self::Cottage => "Cottage",
            Self::LogCabin => "Log Cabin",
            Self::Longhouse => "Longhouse",
            Self::Convento => "Convento",
            Self::CommandersQuarters => "Commander's Quarters",
            Self::LumberMill => "Lumber Mill",
            Self::Blacksmith => "Blacksmith",
            Self::Workshop => "Workshop",
            Self::Smokehouse => "Smokehouse",
            Self::DryingRack => "Drying Rack",
            Self::Vineyard => "Vineyard",
            Self::Farm => "Farm",
            Self::CornField => "Corn Field",
            Self::Storehouse => "Storehouse",
            Self::Granary => "Granary",
            Self::FurWarehouse => "Fur Warehouse",
            Self::StoragePit => "Storage Pit",
            Self::Armory => "Armory",
            Self::Chapel => "Chapel",
            Self::MissionChurch => "Mission Church",
            Self::BellTower => "Bell Tower",
            Self::CouncilHouse => "Council House",
            Self::SweatLodge => "Sweat Lodge",
            Self::Barracks => "Barracks",
            Self::Stable => "Stable",
            Self::TrainingGround => "Training Ground",
            Self::TradingHouse => "Trading House",
            Self::Dock => "Dock",
            Self::Market => "Market",
            Self::Tavern => "Tavern",
            Self::Well => "Well",
            Self::Road => "Road",
            Self::Bridge => "Bridge",
            Self::TownHall => "Town Hall",
            Self::GovernorsHouse => "Governor's House",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::WoodenPalisade => "A defensive wall of sharpened wooden stakes",
            Self::StoneWall => "A sturdy stone fortification wall",
            Self::AdobeWall => "Sun-dried brick wall, Spanish colonial style",
            Self::PalisadeWall => "Traditional native defensive barrier",
            Self::Watchtower => "Elevated platform for spotting threats",
            Self::Gate => "Fortified entrance to the settlement",
            Self::Cottage => "Simple English-style dwelling",
            Self::LogCabin => "Frontier-style log dwelling",
            Self::Longhouse => "Traditional multi-family native dwelling",
            Self::Convento => "Living quarters for mission priests",
            Self::CommandersQuarters => "Officer housing in a presidio",
            Self::LumberMill => "Processes timber into planks and beams",
            Self::Blacksmith => "Forges tools, weapons, and hardware",
            Self::Workshop => "General crafting and repair facility",
            Self::Smokehouse => "Preserves meat through smoking",
            Self::DryingRack => "Dries meat and fish for storage",
            Self::Vineyard => "Cultivates grapes for wine",
            Self::Farm => "Produces food crops",
            Self::CornField => "Traditional Three Sisters agriculture",
            Self::Storehouse => "General storage for goods",
            Self::Granary => "Specialized grain storage",
            Self::FurWarehouse => "Storage for pelts and furs",
            Self::StoragePit => "Underground food cache",
            Self::Armory => "Stores weapons and ammunition",
            Self::Chapel => "Small Protestant church",
            Self::MissionChurch => "Catholic mission church",
            Self::BellTower => "Church bell for calling faithful",
            Self::CouncilHouse => "Meeting place for tribal councils",
            Self::SweatLodge => "Ceremonial purification lodge",
            Self::Barracks => "Housing for soldiers",
            Self::Stable => "Shelter and care for horses",
            Self::TrainingGround => "Area for military drills",
            Self::TradingHouse => "Facility for trading goods",
            Self::Dock => "Landing for boats and ships",
            Self::Market => "Open-air trading area",
            Self::Tavern => "Social gathering and lodging",
            Self::Well => "Source of fresh water",
            Self::Road => "Improved path for travel",
            Self::Bridge => "Crossing over water or ravine",
            Self::TownHall => "Administrative center",
            Self::GovernorsHouse => "Residence of colonial governor",
        }
    }

    /// Get construction costs
    pub fn construction_cost(&self) -> ConstructionCost {
        match self {
            // Defensive (per section)
            Self::WoodenPalisade => ConstructionCost {
                wood: 50,
                stone: 0,
                iron: 5,
                labor_days: 2.0,
                special: None,
            },
            Self::StoneWall => ConstructionCost {
                wood: 10,
                stone: 100,
                iron: 10,
                labor_days: 5.0,
                special: None,
            },
            Self::AdobeWall => ConstructionCost {
                wood: 5,
                stone: 0, // Uses adobe bricks
                iron: 0,
                labor_days: 3.0,
                special: Some("Adobe Bricks: 200"),
            },
            Self::PalisadeWall => ConstructionCost {
                wood: 30,
                stone: 0,
                iron: 0,
                labor_days: 1.5,
                special: None,
            },
            Self::Watchtower => ConstructionCost {
                wood: 80,
                stone: 20,
                iron: 15,
                labor_days: 4.0,
                special: None,
            },
            Self::Gate => ConstructionCost {
                wood: 60,
                stone: 10,
                iron: 30,
                labor_days: 3.0,
                special: None,
            },

            // Residential
            Self::Cottage => ConstructionCost {
                wood: 120,
                stone: 30,
                iron: 10,
                labor_days: 7.0,
                special: Some("Thatch: 50"),
            },
            Self::LogCabin => ConstructionCost {
                wood: 150,
                stone: 20,
                iron: 15,
                labor_days: 10.0,
                special: None,
            },
            Self::Longhouse => ConstructionCost {
                wood: 200,
                stone: 0,
                iron: 0,
                labor_days: 14.0,
                special: Some("Bark sheets: 100"),
            },
            Self::Convento => ConstructionCost {
                wood: 50,
                stone: 150,
                iron: 20,
                labor_days: 20.0,
                special: Some("Adobe Bricks: 300"),
            },
            Self::CommandersQuarters => ConstructionCost {
                wood: 80,
                stone: 200,
                iron: 40,
                labor_days: 25.0,
                special: None,
            },

            // Production
            Self::LumberMill => ConstructionCost {
                wood: 200,
                stone: 50,
                iron: 80,
                labor_days: 15.0,
                special: Some("Sawblades: 2"),
            },
            Self::Blacksmith => ConstructionCost {
                wood: 100,
                stone: 80,
                iron: 50,
                labor_days: 12.0,
                special: Some("Forge equipment"),
            },
            Self::Workshop => ConstructionCost {
                wood: 80,
                stone: 40,
                iron: 20,
                labor_days: 8.0,
                special: None,
            },
            Self::Smokehouse => ConstructionCost {
                wood: 60,
                stone: 30,
                iron: 10,
                labor_days: 5.0,
                special: None,
            },
            Self::DryingRack => ConstructionCost {
                wood: 20,
                stone: 0,
                iron: 0,
                labor_days: 0.5,
                special: None,
            },
            Self::Vineyard => ConstructionCost {
                wood: 30,
                stone: 10,
                iron: 5,
                labor_days: 10.0,
                special: Some("Grape vines: 50"),
            },
            Self::Farm => ConstructionCost {
                wood: 40,
                stone: 0,
                iron: 10,
                labor_days: 5.0,
                special: Some("Seeds"),
            },
            Self::CornField => ConstructionCost {
                wood: 10,
                stone: 0,
                iron: 0,
                labor_days: 3.0,
                special: Some("Corn, bean, squash seeds"),
            },

            // Storage
            Self::Storehouse => ConstructionCost {
                wood: 150,
                stone: 30,
                iron: 20,
                labor_days: 10.0,
                special: None,
            },
            Self::Granary => ConstructionCost {
                wood: 100,
                stone: 50,
                iron: 15,
                labor_days: 8.0,
                special: None,
            },
            Self::FurWarehouse => ConstructionCost {
                wood: 120,
                stone: 20,
                iron: 15,
                labor_days: 8.0,
                special: None,
            },
            Self::StoragePit => ConstructionCost {
                wood: 20,
                stone: 0,
                iron: 0,
                labor_days: 2.0,
                special: Some("Bark lining"),
            },
            Self::Armory => ConstructionCost {
                wood: 80,
                stone: 100,
                iron: 60,
                labor_days: 15.0,
                special: None,
            },

            // Religious
            Self::Chapel => ConstructionCost {
                wood: 200,
                stone: 100,
                iron: 30,
                labor_days: 30.0,
                special: Some("Bell: 1"),
            },
            Self::MissionChurch => ConstructionCost {
                wood: 100,
                stone: 300,
                iron: 50,
                labor_days: 60.0,
                special: Some("Bell: 1, Adobe: 500"),
            },
            Self::BellTower => ConstructionCost {
                wood: 50,
                stone: 150,
                iron: 30,
                labor_days: 20.0,
                special: Some("Bell: 1"),
            },
            Self::CouncilHouse => ConstructionCost {
                wood: 150,
                stone: 0,
                iron: 0,
                labor_days: 10.0,
                special: Some("Bark: 80"),
            },
            Self::SweatLodge => ConstructionCost {
                wood: 40,
                stone: 30,
                iron: 0,
                labor_days: 3.0,
                special: None,
            },

            // Military
            Self::Barracks => ConstructionCost {
                wood: 180,
                stone: 80,
                iron: 40,
                labor_days: 20.0,
                special: None,
            },
            Self::Stable => ConstructionCost {
                wood: 150,
                stone: 30,
                iron: 30,
                labor_days: 12.0,
                special: None,
            },
            Self::TrainingGround => ConstructionCost {
                wood: 30,
                stone: 20,
                iron: 10,
                labor_days: 5.0,
                special: None,
            },

            // Trade
            Self::TradingHouse => ConstructionCost {
                wood: 200,
                stone: 50,
                iron: 40,
                labor_days: 18.0,
                special: None,
            },
            Self::Dock => ConstructionCost {
                wood: 250,
                stone: 100,
                iron: 60,
                labor_days: 25.0,
                special: Some("Rope: 100"),
            },
            Self::Market => ConstructionCost {
                wood: 100,
                stone: 20,
                iron: 20,
                labor_days: 8.0,
                special: None,
            },
            Self::Tavern => ConstructionCost {
                wood: 180,
                stone: 60,
                iron: 30,
                labor_days: 15.0,
                special: None,
            },

            // Infrastructure
            Self::Well => ConstructionCost {
                wood: 30,
                stone: 80,
                iron: 20,
                labor_days: 5.0,
                special: None,
            },
            Self::Road => ConstructionCost {
                wood: 10,
                stone: 50,
                iron: 0,
                labor_days: 2.0,
                special: None, // Per section
            },
            Self::Bridge => ConstructionCost {
                wood: 150,
                stone: 80,
                iron: 40,
                labor_days: 15.0,
                special: Some("Rope: 50"),
            },

            // Administrative
            Self::TownHall => ConstructionCost {
                wood: 300,
                stone: 200,
                iron: 60,
                labor_days: 40.0,
                special: Some("Glass: 20"),
            },
            Self::GovernorsHouse => ConstructionCost {
                wood: 250,
                stone: 300,
                iron: 80,
                labor_days: 50.0,
                special: Some("Glass: 30, Furniture"),
            },
        }
    }

    /// Get effects of this building
    pub fn effects(&self) -> Vec<BuildingEffect> {
        match self {
            Self::WoodenPalisade | Self::StoneWall | Self::AdobeWall | Self::PalisadeWall => {
                vec![BuildingEffect::Defense(10)]
            }
            Self::Watchtower => vec![
                BuildingEffect::Defense(5),
                BuildingEffect::VisionRange(50.0),
            ],
            Self::Gate => vec![BuildingEffect::Defense(5)],
            Self::Cottage | Self::LogCabin => vec![BuildingEffect::Housing(4)],
            Self::Longhouse => vec![BuildingEffect::Housing(20)],
            Self::Convento => vec![BuildingEffect::Housing(6), BuildingEffect::Happiness(5)],
            Self::CommandersQuarters => vec![BuildingEffect::Housing(4), BuildingEffect::Defense(2)],
            Self::LumberMill => vec![BuildingEffect::Production(ResourceType::Wood, 2.0)],
            Self::Blacksmith => vec![
                BuildingEffect::Production(ResourceType::Tools, 1.0),
                BuildingEffect::UnlocksBuilding(BuildingType::Armory),
            ],
            Self::Workshop => vec![BuildingEffect::Production(ResourceType::Goods, 1.0)],
            Self::Smokehouse | Self::DryingRack => {
                vec![BuildingEffect::FoodPreservation(0.5)]
            }
            Self::Vineyard => vec![BuildingEffect::Production(ResourceType::Wine, 0.5)],
            Self::Farm | Self::CornField => vec![BuildingEffect::Production(ResourceType::Food, 1.5)],
            Self::Storehouse | Self::Granary | Self::FurWarehouse | Self::StoragePit => {
                vec![BuildingEffect::Storage(500)]
            }
            Self::Armory => vec![
                BuildingEffect::Storage(200),
                BuildingEffect::Defense(5),
            ],
            Self::Chapel | Self::MissionChurch | Self::BellTower => {
                vec![BuildingEffect::Happiness(10), BuildingEffect::Faith(1.0)]
            }
            Self::CouncilHouse => vec![BuildingEffect::Happiness(5), BuildingEffect::Diplomacy(0.1)],
            Self::SweatLodge => vec![BuildingEffect::Healing(0.2)],
            Self::Barracks => vec![
                BuildingEffect::Housing(10),
                BuildingEffect::MilitaryTraining(1.0),
            ],
            Self::Stable => vec![BuildingEffect::Cavalry(5)],
            Self::TrainingGround => vec![BuildingEffect::MilitaryTraining(0.5)],
            Self::TradingHouse | Self::Market => {
                vec![BuildingEffect::TradeBonus(0.2), BuildingEffect::Income(10)]
            }
            Self::Dock => vec![
                BuildingEffect::TradeBonus(0.3),
                BuildingEffect::NavalAccess,
            ],
            Self::Tavern => vec![
                BuildingEffect::Happiness(5),
                BuildingEffect::Income(5),
                BuildingEffect::InformationGathering,
            ],
            Self::Well => vec![BuildingEffect::FreshWater, BuildingEffect::Health(0.1)],
            Self::Road => vec![BuildingEffect::MovementSpeed(0.2)],
            Self::Bridge => vec![BuildingEffect::MovementSpeed(0.3)],
            Self::TownHall => vec![
                BuildingEffect::Administration(1.0),
                BuildingEffect::Happiness(5),
            ],
            Self::GovernorsHouse => vec![
                BuildingEffect::Administration(2.0),
                BuildingEffect::Diplomacy(0.2),
            ],
        }
    }

    /// Buildings that must exist before this one can be built
    pub fn prerequisites(&self) -> Vec<BuildingType> {
        match self {
            Self::LumberMill => vec![Self::Blacksmith],
            Self::Armory => vec![Self::Blacksmith, Self::Storehouse],
            Self::Barracks => vec![Self::Well],
            Self::TownHall => vec![Self::Chapel, Self::Storehouse],
            Self::GovernorsHouse => vec![Self::TownHall],
            Self::Stable => vec![Self::Barracks],
            Self::Dock => vec![Self::Storehouse],
            Self::BellTower => vec![Self::MissionChurch],
            _ => vec![],
        }
    }
}

/// Cost to construct a building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionCost {
    pub wood: u32,
    pub stone: u32,
    pub iron: u32,
    pub labor_days: f32,
    pub special: Option<&'static str>,
}

/// Effects a building provides
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BuildingEffect {
    Defense(i32),
    Housing(u32),
    Storage(u32),
    Production(ResourceType, f32),
    Happiness(i32),
    Health(f32),
    Faith(f32),
    TradeBonus(f32),
    Income(u32),
    VisionRange(f32),
    MilitaryTraining(f32),
    Cavalry(u32),
    FoodPreservation(f32),
    FreshWater,
    NavalAccess,
    InformationGathering,
    MovementSpeed(f32),
    Administration(f32),
    Diplomacy(f32),
    Healing(f32),
    UnlocksBuilding(BuildingType),
}

/// Resource types for production
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Wood,
    Stone,
    Iron,
    Gold,
    Food,
    Furs,
    Tools,
    Goods,
    Wine,
    Tobacco,
}

// ============================================================================
// SETTLEMENT INSTANCE
// ============================================================================

/// A settlement in the game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub id: u32,
    pub name: String,
    pub position: [f32; 3],
    pub style: SettlementStyle,
    pub faction: Faction,
    pub founded: f64, // game time

    /// All buildings in this settlement
    pub buildings: Vec<Building>,

    /// Population
    pub population: u32,
    pub max_population: u32,

    /// Resources stored
    pub resources: SettlementResources,

    /// Defense rating
    pub defense: i32,

    /// Happiness/morale (0-100)
    pub happiness: i32,

    /// Health/disease level (0-100)
    pub health: i32,

    /// Active construction projects
    pub construction_queue: Vec<ConstructionProject>,

    /// Territory radius this settlement controls
    pub control_radius: f32,

    /// Is this the player's settlement?
    pub player_owned: bool,
}

impl Settlement {
    pub fn new(
        id: u32,
        name: &str,
        position: [f32; 3],
        style: SettlementStyle,
        faction: Faction,
        game_time: f64,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            position,
            style,
            faction,
            founded: game_time,
            buildings: Vec::new(),
            population: 0,
            max_population: 0,
            resources: SettlementResources::default(),
            defense: 0,
            happiness: 50,
            health: 80,
            construction_queue: Vec::new(),
            control_radius: 100.0,
            player_owned: false,
        }
    }

    /// Add a completed building
    pub fn add_building(&mut self, building: Building) {
        // Apply building effects
        for effect in building.building_type.effects() {
            self.apply_effect(&effect);
        }
        self.buildings.push(building);
    }

    fn apply_effect(&mut self, effect: &BuildingEffect) {
        match effect {
            BuildingEffect::Defense(d) => self.defense += d,
            BuildingEffect::Housing(h) => self.max_population += h,
            BuildingEffect::Storage(s) => self.resources.max_storage += s,
            BuildingEffect::Happiness(h) => self.happiness = (self.happiness + h).clamp(0, 100),
            BuildingEffect::Health(h) => self.health = (self.health as f32 + h * 10.0) as i32,
            _ => {} // Other effects handled elsewhere
        }
    }

    /// Start construction of a building
    pub fn start_construction(
        &mut self,
        building_type: BuildingType,
        position: [f32; 3],
    ) -> Result<(), ConstructionError> {
        // Check prerequisites
        for prereq in building_type.prerequisites() {
            if !self.has_building(prereq) {
                return Err(ConstructionError::MissingPrerequisite(prereq));
            }
        }

        // Check resources
        let cost = building_type.construction_cost();
        if self.resources.wood < cost.wood {
            return Err(ConstructionError::InsufficientWood);
        }
        if self.resources.stone < cost.stone {
            return Err(ConstructionError::InsufficientStone);
        }
        if self.resources.iron < cost.iron {
            return Err(ConstructionError::InsufficientIron);
        }

        // Deduct resources
        self.resources.wood -= cost.wood;
        self.resources.stone -= cost.stone;
        self.resources.iron -= cost.iron;

        // Add to queue
        self.construction_queue.push(ConstructionProject {
            building_type,
            position,
            progress: 0.0,
            total_days: cost.labor_days,
            workers_assigned: 1,
        });

        Ok(())
    }

    /// Update construction progress
    pub fn update_construction(&mut self, delta_days: f32) {
        let mut completed = Vec::new();

        for (idx, project) in self.construction_queue.iter_mut().enumerate() {
            // Progress based on workers
            let work_done = delta_days * project.workers_assigned as f32;
            project.progress += work_done;

            if project.progress >= project.total_days {
                completed.push(idx);
            }
        }

        // Complete buildings (in reverse to preserve indices)
        for idx in completed.into_iter().rev() {
            let project = self.construction_queue.remove(idx);
            let building = Building {
                id: self.buildings.len() as u32,
                building_type: project.building_type,
                position: project.position,
                health: 100,
                level: 1,
            };
            self.add_building(building);
        }
    }

    /// Check if settlement has a building type
    pub fn has_building(&self, building_type: BuildingType) -> bool {
        self.buildings.iter().any(|b| b.building_type == building_type)
    }

    /// Count buildings of a type
    pub fn count_buildings(&self, building_type: BuildingType) -> usize {
        self.buildings.iter().filter(|b| b.building_type == building_type).count()
    }

    /// Get total production of a resource type
    pub fn get_production(&self, resource: ResourceType) -> f32 {
        let mut total = 0.0;
        for building in &self.buildings {
            for effect in building.building_type.effects() {
                if let BuildingEffect::Production(r, amount) = effect {
                    if r == resource {
                        total += amount;
                    }
                }
            }
        }
        total
    }

    /// Daily update
    pub fn daily_update(&mut self) {
        // Food consumption
        let food_consumed = self.population / 2;
        self.resources.food = self.resources.food.saturating_sub(food_consumed);

        // Starvation affects happiness and health
        if self.resources.food == 0 && self.population > 0 {
            self.happiness = (self.happiness - 10).max(0);
            self.health = (self.health - 5).max(0);

            // Population loss from starvation
            if self.happiness < 20 {
                self.population = self.population.saturating_sub(1);
            }
        }

        // Production
        self.resources.food += self.get_production(ResourceType::Food) as u32;
        self.resources.wood += self.get_production(ResourceType::Wood) as u32;

        // Cap at storage
        self.resources.cap_at_storage();
    }
}

/// Individual building instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: u32,
    pub building_type: BuildingType,
    pub position: [f32; 3],
    pub health: i32,
    pub level: u32,
}

/// Resources stored in a settlement
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettlementResources {
    pub wood: u32,
    pub stone: u32,
    pub iron: u32,
    pub gold: u32,
    pub food: u32,
    pub furs: u32,
    pub tools: u32,
    pub max_storage: u32,
}

impl SettlementResources {
    pub fn cap_at_storage(&mut self) {
        let max = self.max_storage;
        self.wood = self.wood.min(max);
        self.stone = self.stone.min(max);
        self.iron = self.iron.min(max);
        self.food = self.food.min(max);
        self.furs = self.furs.min(max);
    }

    /// Add gathered resources to settlement storage
    pub fn add_gathered(&mut self, result: &GatherResult) {
        let amount = result.amount;

        match result.resource.category() {
            ResourceCategory::Wood => {
                self.wood = self.wood.saturating_add(amount);
            }
            ResourceCategory::Stone => {
                self.stone = self.stone.saturating_add(amount);
            }
            ResourceCategory::Ore => {
                match result.resource {
                    GatherableResource::IronOre => {
                        self.iron = self.iron.saturating_add(amount);
                    }
                    GatherableResource::GoldNugget => {
                        self.gold = self.gold.saturating_add(amount);
                    }
                    GatherableResource::CopperOre | GatherableResource::SilverOre => {
                        // Store as iron equivalent for now
                        self.iron = self.iron.saturating_add(amount / 2);
                    }
                    _ => {}
                }
            }
            ResourceCategory::Mineral => {
                // Minerals stored as generic materials
                // Could expand storage types later
            }
        }

        self.cap_at_storage();
    }

    /// Add raw resource by type
    pub fn add_resource(&mut self, resource: GatherableResource, amount: u32) {
        match resource.category() {
            ResourceCategory::Wood => self.wood = self.wood.saturating_add(amount),
            ResourceCategory::Stone => self.stone = self.stone.saturating_add(amount),
            ResourceCategory::Ore => {
                match resource {
                    GatherableResource::IronOre => self.iron = self.iron.saturating_add(amount),
                    GatherableResource::GoldNugget => self.gold = self.gold.saturating_add(amount),
                    _ => self.iron = self.iron.saturating_add(amount / 2),
                }
            }
            ResourceCategory::Mineral => {}
        }
        self.cap_at_storage();
    }

    /// Check if settlement has enough resources for construction
    pub fn can_afford(&self, cost: &ConstructionCost) -> bool {
        self.wood >= cost.wood && self.stone >= cost.stone && self.iron >= cost.iron
    }

    /// Get total resource count
    pub fn total(&self) -> u32 {
        self.wood + self.stone + self.iron + self.gold + self.food + self.furs + self.tools
    }
}

/// Active construction project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionProject {
    pub building_type: BuildingType,
    pub position: [f32; 3],
    pub progress: f32,      // days completed
    pub total_days: f32,    // total days needed
    pub workers_assigned: u32,
}

impl ConstructionProject {
    pub fn completion_percentage(&self) -> f32 {
        (self.progress / self.total_days * 100.0).min(100.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructionError {
    MissingPrerequisite(BuildingType),
    InsufficientWood,
    InsufficientStone,
    InsufficientIron,
    InsufficientWorkers,
    InvalidLocation,
    AlreadyBuilding,
}

// ============================================================================
// SETTLEMENT MANAGER
// ============================================================================

/// Manages all settlements in the game
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettlementManager {
    pub settlements: HashMap<u32, Settlement>,
    next_settlement_id: u32,
    pub player_settlement: Option<u32>,
}

impl SettlementManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Found a new settlement
    pub fn found_settlement(
        &mut self,
        name: &str,
        position: [f32; 3],
        style: SettlementStyle,
        faction: Faction,
        player_owned: bool,
        game_time: f64,
    ) -> u32 {
        let id = self.next_settlement_id;
        self.next_settlement_id += 1;

        let mut settlement = Settlement::new(id, name, position, style, faction, game_time);
        settlement.player_owned = player_owned;

        // Start with basic resources
        settlement.resources = SettlementResources {
            wood: 200,
            stone: 50,
            iron: 20,
            gold: 0,
            food: 100,
            furs: 0,
            tools: 10,
            max_storage: 500,
        };

        // Starting population
        settlement.population = 10;
        settlement.max_population = 10;

        if player_owned {
            self.player_settlement = Some(id);
        }

        self.settlements.insert(id, settlement);
        id
    }

    /// Get settlement at position
    pub fn get_settlement_at(&self, position: [f32; 3]) -> Option<&Settlement> {
        self.settlements.values().find(|s| {
            let dx = position[0] - s.position[0];
            let dz = position[2] - s.position[2];
            (dx * dx + dz * dz).sqrt() <= s.control_radius
        })
    }

    /// Get mutable settlement
    pub fn get_settlement_mut(&mut self, id: u32) -> Option<&mut Settlement> {
        self.settlements.get_mut(&id)
    }

    /// Get player's settlement
    pub fn get_player_settlement(&self) -> Option<&Settlement> {
        self.player_settlement.and_then(|id| self.settlements.get(&id))
    }

    /// Get mutable player settlement
    pub fn get_player_settlement_mut(&mut self) -> Option<&mut Settlement> {
        self.player_settlement.and_then(|id| self.settlements.get_mut(&id))
    }

    /// Update all settlements
    pub fn update(&mut self, delta_days: f32) {
        for settlement in self.settlements.values_mut() {
            settlement.update_construction(delta_days);

            // Daily updates
            if delta_days >= 1.0 {
                settlement.daily_update();
            }
        }
    }

    /// Get settlements belonging to a faction
    pub fn get_faction_settlements(&self, faction: Faction) -> Vec<&Settlement> {
        self.settlements.values().filter(|s| s.faction == faction).collect()
    }

    /// Deliver gathered resources to a settlement
    pub fn deliver_resources(&mut self, settlement_id: u32, result: &GatherResult) -> bool {
        if let Some(settlement) = self.settlements.get_mut(&settlement_id) {
            settlement.resources.add_gathered(result);
            true
        } else {
            false
        }
    }

    /// Deliver gathered resources to player's settlement
    pub fn deliver_to_player(&mut self, result: &GatherResult) -> bool {
        if let Some(id) = self.player_settlement {
            self.deliver_resources(id, result)
        } else {
            false
        }
    }

    /// Add raw resources to a settlement
    pub fn add_resources_to_settlement(
        &mut self,
        settlement_id: u32,
        resource: GatherableResource,
        amount: u32,
    ) -> bool {
        if let Some(settlement) = self.settlements.get_mut(&settlement_id) {
            settlement.resources.add_resource(resource, amount);
            true
        } else {
            false
        }
    }

    /// Get nearest settlement to a position
    pub fn get_nearest_settlement(&self, position: [f32; 3]) -> Option<(u32, &Settlement, f32)> {
        self.settlements
            .iter()
            .map(|(id, s)| {
                let dist = ((s.position[0] - position[0]).powi(2)
                    + (s.position[2] - position[2]).powi(2))
                .sqrt();
                (*id, s, dist)
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
    }

    /// Get nearest settlement owned by player
    pub fn get_nearest_player_settlement(&self, position: [f32; 3]) -> Option<(u32, &Settlement, f32)> {
        self.settlements
            .iter()
            .filter(|(_, s)| s.player_owned)
            .map(|(id, s)| {
                let dist = ((s.position[0] - position[0]).powi(2)
                    + (s.position[2] - position[2]).powi(2))
                .sqrt();
                (*id, s, dist)
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
    }

    /// Check if player can afford a building
    pub fn can_player_afford(&self, building_type: BuildingType) -> bool {
        if let Some(settlement) = self.get_player_settlement() {
            settlement.resources.can_afford(&building_type.construction_cost())
        } else {
            false
        }
    }

    /// Get total resources across all player settlements
    pub fn get_player_total_resources(&self) -> SettlementResources {
        let mut total = SettlementResources::default();
        for settlement in self.settlements.values().filter(|s| s.player_owned) {
            total.wood += settlement.resources.wood;
            total.stone += settlement.resources.stone;
            total.iron += settlement.resources.iron;
            total.gold += settlement.resources.gold;
            total.food += settlement.resources.food;
            total.furs += settlement.resources.furs;
            total.tools += settlement.resources.tools;
        }
        total
    }
}
