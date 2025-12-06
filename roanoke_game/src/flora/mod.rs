//! Flora and Plant System
//!
//! A naturalistic plant system modeling the flora of colonial-era Virginia/Carolina region.
//! Includes growth cycles, harvesting, medicinal uses, and toxicology based on real botany.

pub mod growth;
pub mod harvest;
pub mod medicinal;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::encyclopedia::{PlantCategory, Season, Edibility, ToxicityInfo, ToxicityLevel};

/// All plant species in the game, based on historical Virginia/Carolina flora
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FloraSpecies {
    // TREES (15 species)
    WhiteOak,
    RedOak,
    LoblollyPine,
    EasternRedCedar,
    BlackWalnut,
    Sweetgum,
    TulipPoplar,
    AmericanSycamore,
    BaldCypress,
    SouthernMagnolia,
    EasternHemlock,
    AmericanBeech,
    Hickory,
    BlackCherry,
    Sassafras,

    // SHRUBS (10 species)
    Elderberry,
    Blueberry,
    Huckleberry,
    Blackberry,
    WildRose,
    Sumac,
    WitchHazel,
    Bayberry,
    Spicebush,
    Pawpaw,

    // HERBS & MEDICINALS (20 species)
    Ginseng,           // Valuable medicinal
    Goldenseal,        // Antiseptic
    Bloodroot,         // Dye and medicine (toxic)
    WildGinger,        // Digestive aid
    JewelWeed,         // Poison ivy remedy
    Boneset,           // Fever reducer
    Yarrow,            // Wound treatment
    Plantain,          // Common healing herb
    Mullein,           // Respiratory aid
    Comfrey,           // Bone healing
    Echinacea,         // Immune boost
    BlackCohosh,       // Women's medicine
    Lobelia,           // Respiratory (toxic in quantity)
    JimsonWeed,        // Hallucinogenic/deadly
    Foxglove,          // Heart medicine (deadly)
    Pokeweed,          // Edible young shoots (mature plant toxic)
    Mayapple,          // Medicinal root (fruit edible when ripe)
    JackInThePulpit,   // Native use, extremely acrid
    WildLettuce,       // Pain relief/sedative
    Snakeroot,         // Various uses (some species toxic)

    // EDIBLE PLANTS (15 species)
    WildOnion,
    Ramps,             // Wild leek
    Cattail,           // Many edible parts
    WildRice,
    Persimmon,         // Fruit tree
    Muscadine,         // Wild grape
    Maypop,            // Passion fruit
    WildStrawberry,
    Serviceberry,
    Mulberry,
    Hickorynut,
    Chestnut,
    Acorn,             // Requires processing
    WildCarrot,        // Distinguish from poison hemlock!
    Crabapple,

    // FUNGI (10 species)
    Chanterelle,
    ChickenOfTheWoods,
    HensOfTheWoods,
    Morel,
    Puffball,
    Oyster,
    DestroyingAngel,   // Deadly
    DeathCap,          // Deadly
    FalseMorel,        // Toxic
    JackOLantern,      // Toxic lookalike

    // AQUATIC/WETLAND (5 species)
    WildCelery,
    Watercress,
    Arrowhead,         // Wapato - edible tuber
    Pickerelweed,
    SpatterdockLily,

    // CROPS (Colonial era) (10 species)
    Tobacco,
    Maize,
    Squash,
    Beans,
    Sunflower,
    Pumpkin,
    Watermelon,
    Cotton,
    Indigo,
    Sugarcane,
}

impl FloraSpecies {
    pub fn name(&self) -> &'static str {
        match self {
            // Trees
            Self::WhiteOak => "White Oak",
            Self::RedOak => "Red Oak",
            Self::LoblollyPine => "Loblolly Pine",
            Self::EasternRedCedar => "Eastern Red Cedar",
            Self::BlackWalnut => "Black Walnut",
            Self::Sweetgum => "Sweetgum",
            Self::TulipPoplar => "Tulip Poplar",
            Self::AmericanSycamore => "American Sycamore",
            Self::BaldCypress => "Bald Cypress",
            Self::SouthernMagnolia => "Southern Magnolia",
            Self::EasternHemlock => "Eastern Hemlock",
            Self::AmericanBeech => "American Beech",
            Self::Hickory => "Hickory",
            Self::BlackCherry => "Black Cherry",
            Self::Sassafras => "Sassafras",
            // Shrubs
            Self::Elderberry => "Elderberry",
            Self::Blueberry => "Blueberry",
            Self::Huckleberry => "Huckleberry",
            Self::Blackberry => "Blackberry",
            Self::WildRose => "Wild Rose",
            Self::Sumac => "Sumac",
            Self::WitchHazel => "Witch Hazel",
            Self::Bayberry => "Bayberry",
            Self::Spicebush => "Spicebush",
            Self::Pawpaw => "Pawpaw",
            // Herbs
            Self::Ginseng => "American Ginseng",
            Self::Goldenseal => "Goldenseal",
            Self::Bloodroot => "Bloodroot",
            Self::WildGinger => "Wild Ginger",
            Self::JewelWeed => "Jewelweed",
            Self::Boneset => "Boneset",
            Self::Yarrow => "Yarrow",
            Self::Plantain => "Common Plantain",
            Self::Mullein => "Great Mullein",
            Self::Comfrey => "Comfrey",
            Self::Echinacea => "Purple Coneflower",
            Self::BlackCohosh => "Black Cohosh",
            Self::Lobelia => "Indian Tobacco",
            Self::JimsonWeed => "Jimsonweed",
            Self::Foxglove => "Foxglove",
            Self::Pokeweed => "Pokeweed",
            Self::Mayapple => "Mayapple",
            Self::JackInThePulpit => "Jack-in-the-Pulpit",
            Self::WildLettuce => "Wild Lettuce",
            Self::Snakeroot => "White Snakeroot",
            // Edibles
            Self::WildOnion => "Wild Onion",
            Self::Ramps => "Ramps",
            Self::Cattail => "Cattail",
            Self::WildRice => "Wild Rice",
            Self::Persimmon => "Persimmon",
            Self::Muscadine => "Muscadine Grape",
            Self::Maypop => "Maypop",
            Self::WildStrawberry => "Wild Strawberry",
            Self::Serviceberry => "Serviceberry",
            Self::Mulberry => "Red Mulberry",
            Self::Hickorynut => "Hickory Nut",
            Self::Chestnut => "American Chestnut",
            Self::Acorn => "Acorn",
            Self::WildCarrot => "Wild Carrot",
            Self::Crabapple => "Wild Crabapple",
            // Fungi
            Self::Chanterelle => "Chanterelle",
            Self::ChickenOfTheWoods => "Chicken of the Woods",
            Self::HensOfTheWoods => "Hen of the Woods",
            Self::Morel => "Morel",
            Self::Puffball => "Giant Puffball",
            Self::Oyster => "Oyster Mushroom",
            Self::DestroyingAngel => "Destroying Angel",
            Self::DeathCap => "Death Cap",
            Self::FalseMorel => "False Morel",
            Self::JackOLantern => "Jack O'Lantern",
            // Aquatic
            Self::WildCelery => "Wild Celery",
            Self::Watercress => "Watercress",
            Self::Arrowhead => "Arrowhead",
            Self::Pickerelweed => "Pickerelweed",
            Self::SpatterdockLily => "Spatterdock",
            // Crops
            Self::Tobacco => "Tobacco",
            Self::Maize => "Maize",
            Self::Squash => "Squash",
            Self::Beans => "Beans",
            Self::Sunflower => "Sunflower",
            Self::Pumpkin => "Pumpkin",
            Self::Watermelon => "Watermelon",
            Self::Cotton => "Cotton",
            Self::Indigo => "Indigo",
            Self::Sugarcane => "Sugarcane",
        }
    }

    /// Parse species from name string
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase();
        // Get all variants and match by name
        Self::all_species().into_iter().find(|species| {
            species.name().to_lowercase() == name_lower ||
            species.name().to_lowercase().replace(' ', "") == name_lower.replace(' ', "")
        })
    }

    /// Get all species variants
    pub fn all_species() -> Vec<Self> {
        vec![
            // Trees
            Self::WhiteOak, Self::RedOak, Self::LoblollyPine, Self::EasternRedCedar,
            Self::BlackWalnut, Self::Sweetgum, Self::TulipPoplar, Self::AmericanSycamore,
            Self::BaldCypress, Self::SouthernMagnolia, Self::EasternHemlock, Self::AmericanBeech,
            Self::Hickory, Self::BlackCherry, Self::Sassafras,
            // Shrubs
            Self::Elderberry, Self::Blueberry, Self::Huckleberry, Self::Blackberry,
            Self::WildRose, Self::Sumac, Self::WitchHazel, Self::Bayberry,
            Self::Spicebush, Self::Pawpaw,
            // Herbs
            Self::Ginseng, Self::Goldenseal, Self::Bloodroot, Self::WildGinger,
            Self::JewelWeed, Self::Boneset, Self::Yarrow, Self::Plantain,
            Self::Mullein, Self::Comfrey, Self::Echinacea, Self::BlackCohosh,
            Self::Lobelia, Self::JimsonWeed, Self::Foxglove, Self::Pokeweed,
            Self::Mayapple, Self::JackInThePulpit, Self::WildLettuce, Self::Snakeroot,
            // Edibles
            Self::WildOnion, Self::Ramps, Self::Cattail, Self::WildRice,
            Self::Persimmon, Self::Muscadine, Self::Maypop, Self::WildStrawberry,
            Self::Serviceberry, Self::Mulberry, Self::Hickorynut, Self::Chestnut,
            Self::Acorn, Self::WildCarrot, Self::Crabapple,
            // Mushrooms
            Self::Chanterelle, Self::ChickenOfTheWoods, Self::HensOfTheWoods,
            Self::Morel, Self::Puffball, Self::Oyster, Self::DestroyingAngel,
            Self::DeathCap, Self::FalseMorel, Self::JackOLantern,
            // Aquatic
            Self::WildCelery, Self::Watercress, Self::Arrowhead,
            Self::Pickerelweed, Self::SpatterdockLily,
            // Crops
            Self::Tobacco, Self::Maize, Self::Squash, Self::Beans,
            Self::Sunflower, Self::Pumpkin, Self::Watermelon, Self::Cotton,
            Self::Indigo, Self::Sugarcane,
        ]
    }

    pub fn scientific_name(&self) -> &'static str {
        match self {
            Self::WhiteOak => "Quercus alba",
            Self::RedOak => "Quercus rubra",
            Self::LoblollyPine => "Pinus taeda",
            Self::EasternRedCedar => "Juniperus virginiana",
            Self::BlackWalnut => "Juglans nigra",
            Self::Sweetgum => "Liquidambar styraciflua",
            Self::TulipPoplar => "Liriodendron tulipifera",
            Self::AmericanSycamore => "Platanus occidentalis",
            Self::BaldCypress => "Taxodium distichum",
            Self::SouthernMagnolia => "Magnolia grandiflora",
            Self::EasternHemlock => "Tsuga canadensis",
            Self::AmericanBeech => "Fagus grandifolia",
            Self::Hickory => "Carya spp.",
            Self::BlackCherry => "Prunus serotina",
            Self::Sassafras => "Sassafras albidum",
            Self::Elderberry => "Sambucus nigra",
            Self::Blueberry => "Vaccinium corymbosum",
            Self::Huckleberry => "Gaylussacia baccata",
            Self::Blackberry => "Rubus allegheniensis",
            Self::WildRose => "Rosa virginiana",
            Self::Sumac => "Rhus typhina",
            Self::WitchHazel => "Hamamelis virginiana",
            Self::Bayberry => "Myrica pensylvanica",
            Self::Spicebush => "Lindera benzoin",
            Self::Pawpaw => "Asimina triloba",
            Self::Ginseng => "Panax quinquefolius",
            Self::Goldenseal => "Hydrastis canadensis",
            Self::Bloodroot => "Sanguinaria canadensis",
            Self::WildGinger => "Asarum canadense",
            Self::JewelWeed => "Impatiens capensis",
            Self::Boneset => "Eupatorium perfoliatum",
            Self::Yarrow => "Achillea millefolium",
            Self::Plantain => "Plantago major",
            Self::Mullein => "Verbascum thapsus",
            Self::Comfrey => "Symphytum officinale",
            Self::Echinacea => "Echinacea purpurea",
            Self::BlackCohosh => "Actaea racemosa",
            Self::Lobelia => "Lobelia inflata",
            Self::JimsonWeed => "Datura stramonium",
            Self::Foxglove => "Digitalis purpurea",
            Self::Pokeweed => "Phytolacca americana",
            Self::Mayapple => "Podophyllum peltatum",
            Self::JackInThePulpit => "Arisaema triphyllum",
            Self::WildLettuce => "Lactuca virosa",
            Self::Snakeroot => "Ageratina altissima",
            Self::WildOnion => "Allium cernuum",
            Self::Ramps => "Allium tricoccum",
            Self::Cattail => "Typha latifolia",
            Self::WildRice => "Zizania aquatica",
            Self::Persimmon => "Diospyros virginiana",
            Self::Muscadine => "Vitis rotundifolia",
            Self::Maypop => "Passiflora incarnata",
            Self::WildStrawberry => "Fragaria virginiana",
            Self::Serviceberry => "Amelanchier arborea",
            Self::Mulberry => "Morus rubra",
            Self::Hickorynut => "Carya ovata",
            Self::Chestnut => "Castanea dentata",
            Self::Acorn => "Quercus spp.",
            Self::WildCarrot => "Daucus carota",
            Self::Crabapple => "Malus coronaria",
            Self::Chanterelle => "Cantharellus cibarius",
            Self::ChickenOfTheWoods => "Laetiporus sulphureus",
            Self::HensOfTheWoods => "Grifola frondosa",
            Self::Morel => "Morchella esculenta",
            Self::Puffball => "Calvatia gigantea",
            Self::Oyster => "Pleurotus ostreatus",
            Self::DestroyingAngel => "Amanita bisporigera",
            Self::DeathCap => "Amanita phalloides",
            Self::FalseMorel => "Gyromitra esculenta",
            Self::JackOLantern => "Omphalotus olearius",
            Self::WildCelery => "Vallisneria americana",
            Self::Watercress => "Nasturtium officinale",
            Self::Arrowhead => "Sagittaria latifolia",
            Self::Pickerelweed => "Pontederia cordata",
            Self::SpatterdockLily => "Nuphar advena",
            Self::Tobacco => "Nicotiana tabacum",
            Self::Maize => "Zea mays",
            Self::Squash => "Cucurbita spp.",
            Self::Beans => "Phaseolus vulgaris",
            Self::Sunflower => "Helianthus annuus",
            Self::Pumpkin => "Cucurbita pepo",
            Self::Watermelon => "Citrullus lanatus",
            Self::Cotton => "Gossypium hirsutum",
            Self::Indigo => "Indigofera tinctoria",
            Self::Sugarcane => "Saccharum officinarum",
        }
    }

    pub fn category(&self) -> PlantCategory {
        match self {
            Self::WhiteOak | Self::RedOak | Self::LoblollyPine | Self::EasternRedCedar
            | Self::BlackWalnut | Self::Sweetgum | Self::TulipPoplar | Self::AmericanSycamore
            | Self::BaldCypress | Self::SouthernMagnolia | Self::EasternHemlock
            | Self::AmericanBeech | Self::Hickory | Self::BlackCherry | Self::Sassafras
            | Self::Persimmon | Self::Mulberry | Self::Chestnut => PlantCategory::Tree,

            Self::Elderberry | Self::Blueberry | Self::Huckleberry | Self::Blackberry
            | Self::WildRose | Self::Sumac | Self::WitchHazel | Self::Bayberry
            | Self::Spicebush | Self::Pawpaw => PlantCategory::Shrub,

            Self::Ginseng | Self::Goldenseal | Self::Bloodroot | Self::WildGinger
            | Self::JewelWeed | Self::Boneset | Self::Yarrow | Self::Plantain
            | Self::Mullein | Self::Comfrey | Self::Echinacea | Self::BlackCohosh
            | Self::Lobelia | Self::JimsonWeed | Self::Foxglove | Self::Pokeweed
            | Self::Mayapple | Self::JackInThePulpit | Self::WildLettuce | Self::Snakeroot
            | Self::WildOnion | Self::Ramps | Self::WildStrawberry | Self::WildCarrot => PlantCategory::Herb,

            Self::Chanterelle | Self::ChickenOfTheWoods | Self::HensOfTheWoods
            | Self::Morel | Self::Puffball | Self::Oyster | Self::DestroyingAngel
            | Self::DeathCap | Self::FalseMorel | Self::JackOLantern => PlantCategory::Fungus,

            Self::Cattail | Self::WildRice | Self::WildCelery | Self::Watercress
            | Self::Arrowhead | Self::Pickerelweed | Self::SpatterdockLily => PlantCategory::Aquatic,

            Self::Muscadine | Self::Maypop => PlantCategory::Vine,

            Self::Tobacco | Self::Maize | Self::Squash | Self::Beans | Self::Sunflower
            | Self::Pumpkin | Self::Watermelon | Self::Cotton | Self::Indigo
            | Self::Sugarcane => PlantCategory::Crop,

            Self::Serviceberry | Self::Hickorynut | Self::Acorn | Self::Crabapple => PlantCategory::Tree,
        }
    }

    pub fn growing_seasons(&self) -> &'static [Season] {
        match self.category() {
            PlantCategory::Tree => &[Season::Spring, Season::Summer, Season::Fall],
            PlantCategory::Shrub => &[Season::Spring, Season::Summer],
            PlantCategory::Herb => match self {
                Self::Ginseng | Self::Goldenseal => &[Season::Spring],
                Self::Bloodroot => &[Season::Spring],
                _ => &[Season::Spring, Season::Summer],
            },
            PlantCategory::Fungus => match self {
                Self::Morel => &[Season::Spring],
                Self::Chanterelle => &[Season::Summer, Season::Fall],
                _ => &[Season::Summer, Season::Fall],
            },
            PlantCategory::Aquatic => &[Season::Spring, Season::Summer],
            PlantCategory::Vine => &[Season::Summer, Season::Fall],
            PlantCategory::Crop => &[Season::Spring, Season::Summer, Season::Fall],
            _ => &[Season::Spring, Season::Summer],
        }
    }

    pub fn habitats(&self) -> &'static [FloraHabitat] {
        match self {
            // Trees
            Self::WhiteOak | Self::RedOak | Self::Hickory | Self::AmericanBeech =>
                &[FloraHabitat::DeciduousForest, FloraHabitat::MixedForest],
            Self::LoblollyPine | Self::EasternRedCedar =>
                &[FloraHabitat::PineForest, FloraHabitat::MixedForest, FloraHabitat::CoastalPlain],
            Self::BaldCypress => &[FloraHabitat::Swamp, FloraHabitat::Wetland],
            Self::SouthernMagnolia => &[FloraHabitat::DeciduousForest, FloraHabitat::CoastalPlain],
            Self::Sassafras => &[FloraHabitat::ForestEdge, FloraHabitat::DeciduousForest],

            // Wetland/Aquatic
            Self::Cattail | Self::WildRice | Self::Arrowhead | Self::Pickerelweed =>
                &[FloraHabitat::Wetland, FloraHabitat::Riverbank, FloraHabitat::Swamp],
            Self::Watercress => &[FloraHabitat::StreamBed, FloraHabitat::Riverbank],

            // Medicinals often in specific areas
            Self::Ginseng | Self::Goldenseal =>
                &[FloraHabitat::DeciduousForest, FloraHabitat::RichCove],
            Self::Bloodroot | Self::Mayapple => &[FloraHabitat::DeciduousForest],
            Self::JewelWeed => &[FloraHabitat::Wetland, FloraHabitat::StreamBed],

            // Common herbs
            Self::Plantain | Self::Yarrow | Self::Mullein =>
                &[FloraHabitat::Meadow, FloraHabitat::Disturbed, FloraHabitat::Roadside],

            // Toxic plants
            Self::JimsonWeed | Self::Pokeweed =>
                &[FloraHabitat::Disturbed, FloraHabitat::Roadside, FloraHabitat::FieldEdge],
            Self::Foxglove => &[FloraHabitat::ForestEdge, FloraHabitat::Meadow],

            // Fungi
            Self::Chanterelle | Self::HensOfTheWoods => &[FloraHabitat::DeciduousForest],
            Self::ChickenOfTheWoods => &[FloraHabitat::DeciduousForest, FloraHabitat::MixedForest],
            Self::Morel => &[FloraHabitat::DeciduousForest, FloraHabitat::Disturbed],
            Self::DestroyingAngel | Self::DeathCap => &[FloraHabitat::DeciduousForest, FloraHabitat::MixedForest],

            // Berries
            Self::Blueberry | Self::Huckleberry => &[FloraHabitat::PineForest, FloraHabitat::AcidSoil],
            Self::Blackberry => &[FloraHabitat::ForestEdge, FloraHabitat::Disturbed, FloraHabitat::Meadow],
            Self::Elderberry => &[FloraHabitat::Wetland, FloraHabitat::ForestEdge],

            // Crops
            Self::Tobacco | Self::Maize | Self::Squash | Self::Beans | Self::Sunflower
            | Self::Pumpkin | Self::Watermelon | Self::Cotton | Self::Indigo | Self::Sugarcane =>
                &[FloraHabitat::Cultivated],

            _ => &[FloraHabitat::DeciduousForest, FloraHabitat::Meadow],
        }
    }

    pub fn edibility(&self) -> Edibility {
        match self {
            // Safe to eat raw
            Self::Blueberry | Self::Huckleberry | Self::Blackberry | Self::WildStrawberry
            | Self::Muscadine | Self::Mulberry | Self::Serviceberry | Self::Watercress
            | Self::Ramps | Self::WildOnion => Edibility::Safe,

            // Need cooking/processing
            Self::Elderberry => Edibility::CookRequired, // Raw berries mildly toxic
            Self::Cattail | Self::Arrowhead | Self::WildRice | Self::Acorn | Self::Maize
            | Self::Squash | Self::Beans | Self::Pumpkin => Edibility::CookRequired,

            // Edible mushrooms
            Self::Chanterelle | Self::ChickenOfTheWoods | Self::HensOfTheWoods
            | Self::Morel | Self::Puffball | Self::Oyster => Edibility::CookRequired,

            // Medicinal primary use
            Self::Ginseng | Self::Goldenseal | Self::Yarrow | Self::Echinacea
            | Self::Boneset | Self::Mullein | Self::Comfrey | Self::WitchHazel
            | Self::Sassafras | Self::Spicebush => Edibility::Medicinal,

            // Toxic
            Self::Bloodroot | Self::Lobelia | Self::Pokeweed | Self::JackInThePulpit
            | Self::FalseMorel | Self::JackOLantern | Self::Snakeroot => Edibility::Toxic,

            // Deadly
            Self::JimsonWeed | Self::Foxglove | Self::DestroyingAngel | Self::DeathCap => Edibility::Deadly,

            // Most trees not directly edible
            _ => Edibility::Toxic,
        }
    }

    pub fn toxicity_info(&self) -> Option<ToxicityInfo> {
        match self {
            Self::Bloodroot => Some(ToxicityInfo {
                level: ToxicityLevel::Moderate,
                symptoms: vec![
                    "Nausea".to_string(),
                    "Vomiting".to_string(),
                    "Skin irritation from sap".to_string(),
                ],
                onset_time: "15-30 minutes".to_string(),
            }),
            Self::Lobelia => Some(ToxicityInfo {
                level: ToxicityLevel::Moderate,
                symptoms: vec![
                    "Nausea".to_string(),
                    "Sweating".to_string(),
                    "Convulsions in high doses".to_string(),
                ],
                onset_time: "20-60 minutes".to_string(),
            }),
            Self::JimsonWeed => Some(ToxicityInfo {
                level: ToxicityLevel::Lethal,
                symptoms: vec![
                    "Hallucinations".to_string(),
                    "Dilated pupils".to_string(),
                    "Rapid heartbeat".to_string(),
                    "Hyperthermia".to_string(),
                    "Seizures".to_string(),
                    "Death".to_string(),
                ],
                onset_time: "30-60 minutes".to_string(),
            }),
            Self::Foxglove => Some(ToxicityInfo {
                level: ToxicityLevel::Lethal,
                symptoms: vec![
                    "Nausea".to_string(),
                    "Irregular heartbeat".to_string(),
                    "Visual disturbances".to_string(),
                    "Heart failure".to_string(),
                ],
                onset_time: "15-30 minutes".to_string(),
            }),
            Self::DestroyingAngel | Self::DeathCap => Some(ToxicityInfo {
                level: ToxicityLevel::Lethal,
                symptoms: vec![
                    "Delayed onset (6-12 hours)".to_string(),
                    "Severe abdominal pain".to_string(),
                    "Vomiting and diarrhea".to_string(),
                    "Apparent recovery".to_string(),
                    "Liver and kidney failure".to_string(),
                    "Death in 3-7 days".to_string(),
                ],
                onset_time: "6-12 hours".to_string(),
            }),
            Self::Pokeweed => Some(ToxicityInfo {
                level: ToxicityLevel::Moderate,
                symptoms: vec![
                    "Nausea and vomiting".to_string(),
                    "Abdominal cramps".to_string(),
                    "Diarrhea".to_string(),
                    "Convulsions in severe cases".to_string(),
                ],
                onset_time: "1-2 hours".to_string(),
            }),
            Self::Snakeroot => Some(ToxicityInfo {
                level: ToxicityLevel::Severe,
                symptoms: vec![
                    "Trembling".to_string(),
                    "Weakness".to_string(),
                    "Nausea".to_string(),
                    "Milk sickness if consumed through dairy".to_string(),
                ],
                onset_time: "Hours to days".to_string(),
            }),
            Self::FalseMorel => Some(ToxicityInfo {
                level: ToxicityLevel::Severe,
                symptoms: vec![
                    "Nausea".to_string(),
                    "Diarrhea".to_string(),
                    "Liver damage".to_string(),
                    "Death in severe cases".to_string(),
                ],
                onset_time: "6-12 hours".to_string(),
            }),
            Self::JackOLantern => Some(ToxicityInfo {
                level: ToxicityLevel::Moderate,
                symptoms: vec![
                    "Severe cramps".to_string(),
                    "Vomiting".to_string(),
                    "Diarrhea".to_string(),
                ],
                onset_time: "30 minutes - 2 hours".to_string(),
            }),
            _ => None,
        }
    }

    pub fn medicinal_uses(&self) -> Vec<String> {
        match self {
            Self::Ginseng => vec![
                "Energy and stamina boost".to_string(),
                "Immune system support".to_string(),
                "Stress reduction".to_string(),
            ],
            Self::Goldenseal => vec![
                "Antiseptic for wounds".to_string(),
                "Digestive aid".to_string(),
                "Eye wash".to_string(),
            ],
            Self::Yarrow => vec![
                "Stops bleeding".to_string(),
                "Fever reduction".to_string(),
                "Wound poultice".to_string(),
            ],
            Self::Plantain => vec![
                "Insect bite relief".to_string(),
                "Wound healing".to_string(),
                "Drawing poultice".to_string(),
            ],
            Self::JewelWeed => vec![
                "Poison ivy remedy".to_string(),
                "Insect bite relief".to_string(),
            ],
            Self::Boneset => vec![
                "Fever reducer".to_string(),
                "Flu treatment".to_string(),
            ],
            Self::Mullein => vec![
                "Respiratory aid".to_string(),
                "Cough suppressant".to_string(),
                "Ear oil".to_string(),
            ],
            Self::Comfrey => vec![
                "Bone healing".to_string(),
                "Wound poultice".to_string(),
            ],
            Self::Echinacea => vec![
                "Immune boost".to_string(),
                "Cold prevention".to_string(),
            ],
            Self::WitchHazel => vec![
                "Astringent".to_string(),
                "Skin treatment".to_string(),
                "Bruise relief".to_string(),
            ],
            Self::WildGinger => vec![
                "Digestive aid".to_string(),
                "Nausea relief".to_string(),
            ],
            Self::Sassafras => vec![
                "Blood tonic".to_string(),
                "Spring tonic tea".to_string(),
            ],
            Self::WildLettuce => vec![
                "Pain relief".to_string(),
                "Sleep aid".to_string(),
                "Anxiety reduction".to_string(),
            ],
            Self::BlackCohosh => vec![
                "Women's health".to_string(),
                "Menstrual relief".to_string(),
            ],
            Self::Elderberry => vec![
                "Cold and flu treatment".to_string(),
                "Immune support".to_string(),
            ],
            _ => Vec::new(),
        }
    }

    pub fn harvest_description(&self) -> &'static str {
        match self.category() {
            PlantCategory::Tree => "Harvest bark, nuts, or sap depending on season",
            PlantCategory::Shrub => "Harvest berries when ripe, stems in dormancy",
            PlantCategory::Herb => "Harvest leaves and flowers at peak bloom, roots in fall",
            PlantCategory::Fungus => "Harvest entire fruiting body when mature",
            PlantCategory::Aquatic => "Harvest roots, shoots, or seeds by season",
            PlantCategory::Vine => "Harvest fruit when ripe in late summer/fall",
            PlantCategory::Crop => "Harvest at maturity according to crop type",
            _ => "Harvest appropriate plant parts by season",
        }
    }

    pub fn botanical_notes(&self) -> &'static str {
        match self {
            Self::Ginseng => "Slow-growing perennial requiring 5-10 years to mature. Distinctive palmately compound leaves with 5 leaflets. Red berries in fall. Found in rich, shaded coves.",
            Self::Sassafras => "Aromatic tree with three distinct leaf shapes on same plant: unlobed, mitten-shaped, and three-lobed. Root bark used for tea and flavoring.",
            Self::Bloodroot => "Early spring ephemeral with pure white flowers. Red-orange sap used historically as dye. All parts toxic if ingested.",
            Self::Pawpaw => "Largest edible fruit native to North America. Tropical-tasting custard-like flesh. Grows in understory of rich forests.",
            Self::BaldCypress => "Deciduous conifer of southern swamps. Distinctive 'knees' rise from root system. Extremely rot-resistant wood prized for construction.",
            Self::DestroyingAngel => "Pure white mushroom easily confused with edible species. Contains amatoxins that destroy liver cells. One cap can kill an adult.",
            Self::Morel => "Highly prized edible with distinctive honeycomb cap. Appears in spring, often near dead elms. Must be cooked thoroughly.",
            Self::Cattail => "Every part useful: young shoots edible, pollen for flour, roots starchy, leaves for weaving, fluff for insulation and tinder.",
            _ => "A native species of the Virginia and Carolina regions.",
        }
    }

    pub fn cultivation_tips(&self) -> &'static str {
        match self.category() {
            PlantCategory::Tree => "Most trees require years to establish. Select appropriate site for soil and light requirements.",
            PlantCategory::Shrub => "Plant in appropriate soil conditions. Many berry shrubs prefer acidic soil.",
            PlantCategory::Herb => "Many medicinal herbs can be cultivated in gardens. Match light and moisture requirements.",
            PlantCategory::Fungus => "Cannot be traditionally cultivated. Learn habitat preferences for wild harvesting.",
            PlantCategory::Aquatic => "Require standing or flowing water. Plant in pond margins or stream edges.",
            PlantCategory::Vine => "Require support structure. Prune annually for best fruit production.",
            PlantCategory::Crop => "Follow traditional planting calendar. Three Sisters (corn, beans, squash) grown together.",
            _ => "Study wild growing conditions and attempt to replicate.",
        }
    }

    /// Iterator over all flora species
    pub fn all() -> impl Iterator<Item = FloraSpecies> {
        [
            Self::WhiteOak, Self::RedOak, Self::LoblollyPine, Self::EasternRedCedar,
            Self::BlackWalnut, Self::Sweetgum, Self::TulipPoplar, Self::AmericanSycamore,
            Self::BaldCypress, Self::SouthernMagnolia, Self::EasternHemlock, Self::AmericanBeech,
            Self::Hickory, Self::BlackCherry, Self::Sassafras,
            Self::Elderberry, Self::Blueberry, Self::Huckleberry, Self::Blackberry,
            Self::WildRose, Self::Sumac, Self::WitchHazel, Self::Bayberry,
            Self::Spicebush, Self::Pawpaw,
            Self::Ginseng, Self::Goldenseal, Self::Bloodroot, Self::WildGinger,
            Self::JewelWeed, Self::Boneset, Self::Yarrow, Self::Plantain,
            Self::Mullein, Self::Comfrey, Self::Echinacea, Self::BlackCohosh,
            Self::Lobelia, Self::JimsonWeed, Self::Foxglove, Self::Pokeweed,
            Self::Mayapple, Self::JackInThePulpit, Self::WildLettuce, Self::Snakeroot,
            Self::WildOnion, Self::Ramps, Self::Cattail, Self::WildRice,
            Self::Persimmon, Self::Muscadine, Self::Maypop, Self::WildStrawberry,
            Self::Serviceberry, Self::Mulberry, Self::Hickorynut, Self::Chestnut,
            Self::Acorn, Self::WildCarrot, Self::Crabapple,
            Self::Chanterelle, Self::ChickenOfTheWoods, Self::HensOfTheWoods,
            Self::Morel, Self::Puffball, Self::Oyster, Self::DestroyingAngel,
            Self::DeathCap, Self::FalseMorel, Self::JackOLantern,
            Self::WildCelery, Self::Watercress, Self::Arrowhead, Self::Pickerelweed,
            Self::SpatterdockLily,
            Self::Tobacco, Self::Maize, Self::Squash, Self::Beans, Self::Sunflower,
            Self::Pumpkin, Self::Watermelon, Self::Cotton, Self::Indigo, Self::Sugarcane,
        ].into_iter()
    }

    /// Get rarity (affects spawn frequency)
    pub fn rarity(&self) -> Rarity {
        match self {
            Self::Ginseng | Self::Goldenseal => Rarity::VeryRare,
            Self::Morel | Self::Chanterelle | Self::BlackCohosh => Rarity::Rare,
            Self::Pawpaw | Self::Ramps | Self::WildGinger => Rarity::Uncommon,
            Self::Blueberry | Self::Blackberry | Self::Elderberry => Rarity::Common,
            Self::WhiteOak | Self::RedOak | Self::LoblollyPine => Rarity::Abundant,
            Self::Plantain | Self::Yarrow | Self::Mullein => Rarity::Abundant,
            _ => Rarity::Common,
        }
    }

    /// Get trade value (in colonial currency units)
    pub fn trade_value(&self) -> u32 {
        match self.rarity() {
            Rarity::VeryRare => 50,
            Rarity::Rare => 25,
            Rarity::Uncommon => 10,
            Rarity::Common => 3,
            Rarity::Abundant => 1,
        }
    }

    /// Get base harvest yield quantity
    pub fn harvest_yield_base(&self) -> u32 {
        match self.category() {
            PlantCategory::Tree => 5,        // Bark, sap, etc
            PlantCategory::Shrub => 3,       // Berries, leaves
            PlantCategory::Herb => 2,        // Leaves, roots
            PlantCategory::Fungus => 1,      // Individual fungi
            PlantCategory::Fern => 2,        // Fern fronds
            PlantCategory::Moss => 1,        // Moss patches
            PlantCategory::Aquatic => 2,     // Water plants
            PlantCategory::Vine => 4,        // Grapes, etc
            PlantCategory::Grass => 1,       // Grass blades
            PlantCategory::Crop => 5,        // Cultivated crops
        }
    }

    /// Get primary medicinal effect
    pub fn primary_effect(&self) -> Option<crate::flora::medicinal::PlantEffect> {
        use crate::flora::medicinal::PlantEffect;
        match self {
            Self::Ginseng => Some(PlantEffect::StaminaRestore),
            Self::Goldenseal => Some(PlantEffect::WoundCleanse),
            Self::Yarrow => Some(PlantEffect::Healing),
            Self::Plantain => Some(PlantEffect::Healing),
            Self::Mullein => Some(PlantEffect::PainRelief),
            Self::Comfrey => Some(PlantEffect::BoneHealing),
            Self::Echinacea => Some(PlantEffect::ImmuneBoost),
            Self::JewelWeed => Some(PlantEffect::PoisonResist),
            Self::Boneset => Some(PlantEffect::FeverReduction),
            Self::WildGinger => Some(PlantEffect::DigestionAid),
            Self::WildLettuce => Some(PlantEffect::SleepAid),
            _ => None,
        }
    }

    /// Check if plant is toxic
    pub fn is_toxic(&self) -> bool {
        matches!(
            self,
            Self::JimsonWeed
                | Self::Foxglove
                | Self::DestroyingAngel
                | Self::DeathCap
                | Self::Bloodroot
                | Self::Snakeroot
                | Self::FalseMorel
                | Self::JackOLantern
        )
    }

    /// Check if preparation is required
    pub fn requires_preparation(&self) -> bool {
        matches!(
            self,
            Self::Acorn           // Leach tannins
                | Self::Pokeweed  // Only young shoots edible
                | Self::JackInThePulpit // Extremely acrid raw
                | Self::Lobelia   // Toxic in quantity
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloraHabitat {
    DeciduousForest,
    PineForest,
    MixedForest,
    ForestEdge,
    RichCove,
    Swamp,
    Wetland,
    Riverbank,
    StreamBed,
    Meadow,
    Disturbed,
    Roadside,
    FieldEdge,
    CoastalPlain,
    AcidSoil,
    Cultivated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Abundant,
    Common,
    Uncommon,
    Rare,
    VeryRare,
}

impl Rarity {
    pub fn spawn_weight(&self) -> f32 {
        match self {
            Self::Abundant => 1.0,
            Self::Common => 0.6,
            Self::Uncommon => 0.3,
            Self::Rare => 0.1,
            Self::VeryRare => 0.03,
        }
    }
}

// === FLORA MANAGER FOR PIPELINE INTEGRATION ===

use glam::Vec3;

/// Manages flora instances and growth across the world
#[derive(Debug)]
pub struct FloraManager {
    seed: u32,
    /// Plants the player has harvested/interacted with
    known_plants: Vec<FloraSpecies>,
    /// Current day in growing cycle
    current_day: u32,
}

impl FloraManager {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            known_plants: Vec::new(),
            current_day: 1,
        }
    }

    /// Update plant growth based on modifier and time
    pub fn update_growth(&mut self, _modifier: f32, _game_hour: f32) {
        // Growth updates happen daily, not hourly
    }

    /// Advance one day for all plants
    pub fn advance_day(&mut self) {
        self.current_day += 1;
    }

    /// Attempt to harvest a plant at a position
    pub fn harvest(
        &mut self,
        species: FloraSpecies,
        _position: Vec3,
        quality_modifier: f32,
    ) -> Option<HarvestYield> {
        // Record that we've interacted with this species
        if !self.known_plants.contains(&species) {
            self.known_plants.push(species);
        }

        // Calculate harvest yield
        let base_yield = species.harvest_yield_base();
        let quantity = (base_yield as f32 * quality_modifier).ceil() as u32;

        Some(HarvestYield {
            species,
            quantity,
            quality: quality_modifier,
        })
    }

    /// Get list of known plant species for save data
    pub fn get_known_plants(&self) -> Vec<String> {
        self.known_plants.iter().map(|s| format!("{:?}", s)).collect()
    }

    /// Restore known plants from save data
    pub fn restore_known_plants(&mut self, plants: &[String]) {
        self.known_plants.clear();
        for name in plants {
            if let Some(species) = FloraSpecies::from_name(name) {
                self.known_plants.push(species);
            }
        }
    }
}

/// Result of harvesting a plant
#[derive(Debug, Clone)]
pub struct HarvestYield {
    pub species: FloraSpecies,
    pub quantity: u32,
    pub quality: f32,
}
