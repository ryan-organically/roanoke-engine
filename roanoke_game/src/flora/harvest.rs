//! Harvesting system for flora
//!
//! Handles item drops, quality determination, and resource gathering from plants.

use serde::{Deserialize, Serialize};
use super::{FloraSpecies, growth::HarvestQuality};
use crate::encyclopedia::PlantCategory;

/// Item produced from harvesting a plant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestItem {
    pub item_id: &'static str,
    pub name: &'static str,
    pub quantity: u32,
    pub quality: HarvestQuality,
    pub properties: ItemProperties,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemProperties {
    /// Base trade value
    pub value: u32,
    /// Weight in pounds
    pub weight: f32,
    /// Stack limit
    pub max_stack: u32,
    /// Spoilage time in game days (None = doesn't spoil)
    pub spoils_in: Option<f32>,
    /// Can be eaten directly
    pub is_food: bool,
    /// Food value if edible
    pub food_value: Option<FoodValue>,
    /// Is this a crafting material
    pub is_material: bool,
    /// Is this medicinal
    pub is_medicine: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FoodValue {
    pub hunger: f32,
    pub thirst: f32,
    pub health: f32, // Can be negative for toxic plants
}

/// Get harvest drops for a species
pub fn get_harvest_drops(species: FloraSpecies, quality: HarvestQuality) -> Vec<HarvestItem> {
    let base_items = get_base_drops(species);

    base_items
        .into_iter()
        .map(|mut item| {
            // Adjust quantity based on quality
            item.quantity = match quality {
                HarvestQuality::Poor => (item.quantity as f32 * 0.5).max(1.0) as u32,
                HarvestQuality::Average => item.quantity,
                HarvestQuality::Good => (item.quantity as f32 * 1.5) as u32,
                HarvestQuality::Prime => item.quantity * 2,
            };
            item.quality = quality;
            item
        })
        .collect()
}

fn get_base_drops(species: FloraSpecies) -> Vec<HarvestItem> {
    match species {
        // Berries
        FloraSpecies::Blueberry => vec![HarvestItem {
            item_id: "blueberries",
            name: "Blueberries",
            quantity: 8,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 2,
                weight: 0.1,
                max_stack: 50,
                spoils_in: Some(5.0),
                is_food: true,
                food_value: Some(FoodValue { hunger: 5.0, thirst: 3.0, health: 2.0 }),
                is_material: false,
                is_medicine: false,
            },
        }],

        FloraSpecies::Blackberry => vec![HarvestItem {
            item_id: "blackberries",
            name: "Blackberries",
            quantity: 10,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 2,
                weight: 0.1,
                max_stack: 50,
                spoils_in: Some(4.0),
                is_food: true,
                food_value: Some(FoodValue { hunger: 5.0, thirst: 4.0, health: 1.0 }),
                is_material: false,
                is_medicine: false,
            },
        }],

        FloraSpecies::Elderberry => vec![
            HarvestItem {
                item_id: "elderberries",
                name: "Elderberries",
                quantity: 12,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 3,
                    weight: 0.1,
                    max_stack: 50,
                    spoils_in: Some(3.0),
                    is_food: false, // Must be cooked
                    food_value: None,
                    is_material: true,
                    is_medicine: true,
                },
            },
            HarvestItem {
                item_id: "elderflowers",
                name: "Elderflowers",
                quantity: 5,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 5,
                    weight: 0.05,
                    max_stack: 30,
                    spoils_in: Some(2.0),
                    is_food: false,
                    food_value: None,
                    is_material: true,
                    is_medicine: true,
                },
            },
        ],

        // Medicinal herbs
        FloraSpecies::Ginseng => vec![HarvestItem {
            item_id: "ginseng_root",
            name: "Ginseng Root",
            quantity: 1,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 50,
                weight: 0.2,
                max_stack: 10,
                spoils_in: Some(30.0), // Dries well
                is_food: false,
                food_value: None,
                is_material: true,
                is_medicine: true,
            },
        }],

        FloraSpecies::Goldenseal => vec![HarvestItem {
            item_id: "goldenseal_root",
            name: "Goldenseal Root",
            quantity: 1,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 40,
                weight: 0.15,
                max_stack: 10,
                spoils_in: Some(20.0),
                is_food: false,
                food_value: None,
                is_material: true,
                is_medicine: true,
            },
        }],

        FloraSpecies::Yarrow => vec![
            HarvestItem {
                item_id: "yarrow_leaves",
                name: "Yarrow Leaves",
                quantity: 5,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 5,
                    weight: 0.05,
                    max_stack: 30,
                    spoils_in: Some(7.0),
                    is_food: false,
                    food_value: None,
                    is_material: true,
                    is_medicine: true,
                },
            },
            HarvestItem {
                item_id: "yarrow_flowers",
                name: "Yarrow Flowers",
                quantity: 3,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 8,
                    weight: 0.03,
                    max_stack: 30,
                    spoils_in: Some(5.0),
                    is_food: false,
                    food_value: None,
                    is_material: true,
                    is_medicine: true,
                },
            },
        ],

        FloraSpecies::Plantain => vec![HarvestItem {
            item_id: "plantain_leaves",
            name: "Plantain Leaves",
            quantity: 8,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 1,
                weight: 0.05,
                max_stack: 50,
                spoils_in: Some(3.0),
                is_food: true,
                food_value: Some(FoodValue { hunger: 2.0, thirst: 1.0, health: 5.0 }),
                is_material: true,
                is_medicine: true,
            },
        }],

        // Edible mushrooms
        FloraSpecies::Chanterelle => vec![HarvestItem {
            item_id: "chanterelle",
            name: "Chanterelle Mushroom",
            quantity: 3,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 15,
                weight: 0.2,
                max_stack: 20,
                spoils_in: Some(4.0),
                is_food: true,
                food_value: Some(FoodValue { hunger: 10.0, thirst: 0.0, health: 0.0 }),
                is_material: false,
                is_medicine: false,
            },
        }],

        FloraSpecies::Morel => vec![HarvestItem {
            item_id: "morel",
            name: "Morel Mushroom",
            quantity: 2,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 25,
                weight: 0.1,
                max_stack: 20,
                spoils_in: Some(3.0),
                is_food: true,
                food_value: Some(FoodValue { hunger: 8.0, thirst: 0.0, health: 0.0 }),
                is_material: false,
                is_medicine: false,
            },
        }],

        // Deadly mushrooms - still harvestable for... purposes
        FloraSpecies::DestroyingAngel | FloraSpecies::DeathCap => vec![HarvestItem {
            item_id: "deadly_mushroom",
            name: "Deadly Mushroom",
            quantity: 1,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 30, // Valuable to certain buyers
                weight: 0.1,
                max_stack: 10,
                spoils_in: Some(5.0),
                is_food: true, // Technically edible... once
                food_value: Some(FoodValue { hunger: 5.0, thirst: 0.0, health: -200.0 }),
                is_material: true,
                is_medicine: false,
            },
        }],

        // Trees - yield varies by part harvested
        FloraSpecies::WhiteOak | FloraSpecies::RedOak => vec![
            HarvestItem {
                item_id: "oak_bark",
                name: "Oak Bark",
                quantity: 3,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 2,
                    weight: 0.5,
                    max_stack: 20,
                    spoils_in: None,
                    is_food: false,
                    food_value: None,
                    is_material: true,
                    is_medicine: true, // Tannins
                },
            },
            HarvestItem {
                item_id: "acorns",
                name: "Acorns",
                quantity: 10,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 1,
                    weight: 0.1,
                    max_stack: 50,
                    spoils_in: Some(60.0), // Long storage
                    is_food: true,
                    food_value: Some(FoodValue { hunger: 3.0, thirst: -2.0, health: 0.0 }),
                    is_material: true,
                    is_medicine: false,
                },
            },
        ],

        FloraSpecies::Sassafras => vec![
            HarvestItem {
                item_id: "sassafras_root",
                name: "Sassafras Root",
                quantity: 2,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 10,
                    weight: 0.3,
                    max_stack: 20,
                    spoils_in: Some(30.0),
                    is_food: false,
                    food_value: None,
                    is_material: true,
                    is_medicine: true,
                },
            },
            HarvestItem {
                item_id: "sassafras_leaves",
                name: "Sassafras Leaves",
                quantity: 5,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 3,
                    weight: 0.05,
                    max_stack: 30,
                    spoils_in: Some(7.0),
                    is_food: false,
                    food_value: None,
                    is_material: true, // File powder for gumbo
                    is_medicine: false,
                },
            },
        ],

        // Aquatic
        FloraSpecies::Cattail => vec![
            HarvestItem {
                item_id: "cattail_shoots",
                name: "Cattail Shoots",
                quantity: 4,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 2,
                    weight: 0.2,
                    max_stack: 20,
                    spoils_in: Some(3.0),
                    is_food: true,
                    food_value: Some(FoodValue { hunger: 8.0, thirst: 2.0, health: 0.0 }),
                    is_material: false,
                    is_medicine: false,
                },
            },
            HarvestItem {
                item_id: "cattail_pollen",
                name: "Cattail Pollen",
                quantity: 2,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 5,
                    weight: 0.1,
                    max_stack: 30,
                    spoils_in: Some(30.0),
                    is_food: true, // Flour substitute
                    food_value: Some(FoodValue { hunger: 5.0, thirst: -1.0, health: 0.0 }),
                    is_material: true,
                    is_medicine: false,
                },
            },
            HarvestItem {
                item_id: "cattail_fluff",
                name: "Cattail Fluff",
                quantity: 3,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 1,
                    weight: 0.02,
                    max_stack: 50,
                    spoils_in: None,
                    is_food: false,
                    food_value: None,
                    is_material: true, // Tinder, insulation
                    is_medicine: false,
                },
            },
        ],

        // Toxic plants - harvestable for crafting/alchemy
        FloraSpecies::Foxglove => vec![HarvestItem {
            item_id: "foxglove_leaves",
            name: "Foxglove Leaves",
            quantity: 4,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 15,
                weight: 0.05,
                max_stack: 20,
                spoils_in: Some(5.0),
                is_food: false,
                food_value: None,
                is_material: true,
                is_medicine: true, // Digitalis source
            },
        }],

        FloraSpecies::JimsonWeed => vec![
            HarvestItem {
                item_id: "jimsonweed_seeds",
                name: "Jimsonweed Seeds",
                quantity: 10,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 20,
                    weight: 0.05,
                    max_stack: 50,
                    spoils_in: None,
                    is_food: false,
                    food_value: None,
                    is_material: true,
                    is_medicine: false, // Too dangerous
                },
            },
            HarvestItem {
                item_id: "jimsonweed_leaves",
                name: "Jimsonweed Leaves",
                quantity: 5,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 10,
                    weight: 0.05,
                    max_stack: 30,
                    spoils_in: Some(5.0),
                    is_food: false,
                    food_value: None,
                    is_material: true,
                    is_medicine: false,
                },
            },
        ],

        // Crops
        FloraSpecies::Tobacco => vec![HarvestItem {
            item_id: "tobacco_leaves",
            name: "Tobacco Leaves",
            quantity: 8,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 10,
                weight: 0.2,
                max_stack: 30,
                spoils_in: Some(14.0),
                is_food: false,
                food_value: None,
                is_material: true,
                is_medicine: false,
            },
        }],

        FloraSpecies::Maize => vec![
            HarvestItem {
                item_id: "corn_ears",
                name: "Corn Ears",
                quantity: 6,
                quality: HarvestQuality::Average,
                properties: ItemProperties {
                    value: 3,
                    weight: 0.3,
                    max_stack: 20,
                    spoils_in: Some(14.0),
                    is_food: true,
                    food_value: Some(FoodValue { hunger: 15.0, thirst: -2.0, health: 0.0 }),
                    is_material: true,
                    is_medicine: false,
                },
            },
        ],

        // Default for unspecified plants
        _ => vec![HarvestItem {
            item_id: "plant_material",
            name: "Plant Material",
            quantity: 2,
            quality: HarvestQuality::Average,
            properties: ItemProperties {
                value: 1,
                weight: 0.1,
                max_stack: 50,
                spoils_in: Some(7.0),
                is_food: false,
                food_value: None,
                is_material: true,
                is_medicine: false,
            },
        }],
    }
}

/// Calculate foraging skill experience from a harvest
pub fn calculate_foraging_xp(species: FloraSpecies, quality: HarvestQuality) -> u32 {
    let base_xp = match species.rarity() {
        super::Rarity::VeryRare => 50,
        super::Rarity::Rare => 25,
        super::Rarity::Uncommon => 15,
        super::Rarity::Common => 8,
        super::Rarity::Abundant => 3,
    };

    let quality_mult = match quality {
        HarvestQuality::Poor => 0.5,
        HarvestQuality::Average => 1.0,
        HarvestQuality::Good => 1.5,
        HarvestQuality::Prime => 2.0,
    };

    (base_xp as f32 * quality_mult) as u32
}
