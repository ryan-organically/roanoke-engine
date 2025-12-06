//! Quest System
//!
//! Manages quests, objectives, and campaign progression.

use super::reputation::Faction;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quest manager handling all active and completed quests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestManager {
    /// All available quests (including completed)
    pub quests: HashMap<String, Quest>,
    /// Currently active quest IDs
    pub active: Vec<String>,
    /// Completed quest IDs
    pub completed: Vec<String>,
    /// Failed quest IDs
    pub failed: Vec<String>,
    /// Main campaign chapter
    pub campaign_chapter: u32,
}

impl QuestManager {
    pub fn new() -> Self {
        let mut manager = Self::default();
        manager.initialize_quests();
        manager
    }

    /// Initialize all quests in the game
    fn initialize_quests(&mut self) {
        // Main Campaign - Chapter 1: Arrival
        self.add_quest(Quest {
            id: "main_01_lost_colony".to_string(),
            title: "The Lost Colony".to_string(),
            description: "Discover what happened to the colonists of Roanoke.".to_string(),
            quest_type: QuestType::MainStory,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("explore_island", "Explore Roanoke Island", ObjectiveType::Explore {
                    location_id: 1,
                    radius: 100.0,
                }),
                QuestObjective::new("find_croatoan", "Find the CROATOAN carving", ObjectiveType::Discover {
                    item: "croatoan_tree".to_string(),
                }),
                QuestObjective::new("speak_elder", "Speak with the village elder", ObjectiveType::TalkTo {
                    npc_id: 1,
                    dialogue_id: "elder_intro".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 100,
                items: vec![("map_fragment".to_string(), 1)],
                reputation: vec![(Faction::NativeCouncil, 25)],
                skill_points: Some(("hunting".to_string(), 50)),
                unlocks: vec!["main_02_proving".to_string()],
            },
            prerequisites: vec![],
            state: QuestState::Available,
            giver_npc: None,
            turn_in_npc: Some(1),
            time_limit: None,
        });

        // Main Campaign - Chapter 1: Proving Yourself
        self.add_quest(Quest {
            id: "main_02_proving".to_string(),
            title: "Proving Yourself".to_string(),
            description: "Earn the trust of the native village by helping with their problems.".to_string(),
            quest_type: QuestType::MainStory,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("hunt_boar", "Hunt wild boar for the village", ObjectiveType::Kill {
                    species: "Wild Boar".to_string(),
                    count: 3,
                    current: 0,
                }),
                QuestObjective::new("gather_herbs", "Gather healing herbs", ObjectiveType::Gather {
                    item: "healing_herb".to_string(),
                    count: 5,
                    current: 0,
                }),
                QuestObjective::new("return_elder", "Return to the elder", ObjectiveType::TalkTo {
                    npc_id: 1,
                    dialogue_id: "elder_proving_complete".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 150,
                items: vec![("hunter_bow".to_string(), 1)],
                reputation: vec![(Faction::NativeCouncil, 50), (Faction::Hunters, 25)],
                skill_points: None,
                unlocks: vec!["main_03_threat".to_string(), "side_hunting_01".to_string()],
            },
            prerequisites: vec!["main_01_lost_colony".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(1),
            turn_in_npc: Some(1),
            time_limit: None,
        });

        // Main Campaign - Chapter 1: The Threat
        self.add_quest(Quest {
            id: "main_03_threat".to_string(),
            title: "Shadows in the Forest".to_string(),
            description: "Investigate reports of dangerous predators threatening the village.".to_string(),
            quest_type: QuestType::MainStory,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("investigate_attack", "Investigate the attack site", ObjectiveType::Explore {
                    location_id: 10,
                    radius: 30.0,
                }),
                QuestObjective::new("track_predator", "Follow the predator's trail", ObjectiveType::Track {
                    species: "Eastern Cougar".to_string(),
                }),
                QuestObjective::new("kill_cougar", "Kill or drive off the cougar", ObjectiveType::Kill {
                    species: "Eastern Cougar".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 250,
                items: vec![("cougar_pelt".to_string(), 1), ("hunter_knife".to_string(), 1)],
                reputation: vec![(Faction::NativeCouncil, 75), (Faction::Hunters, 50)],
                skill_points: Some(("hunting".to_string(), 100)),
                unlocks: vec!["main_04_shaman".to_string()],
            },
            prerequisites: vec!["main_02_proving".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(2), // Warrior chief
            turn_in_npc: Some(2),
            time_limit: None,
        });

        // Side Quest - Hunter's Path
        self.add_quest(Quest {
            id: "side_hunting_01".to_string(),
            title: "The Hunter's Path".to_string(),
            description: "Learn the ways of the hunt from the village hunters.".to_string(),
            quest_type: QuestType::Side,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("track_deer", "Successfully track a deer", ObjectiveType::Track {
                    species: "Deer".to_string(),
                }),
                QuestObjective::new("stealth_kill", "Perform a stealth kill", ObjectiveType::SpecialKill {
                    condition: "stealth".to_string(),
                    count: 1,
                    current: 0,
                }),
                QuestObjective::new("skin_animal", "Skin an animal", ObjectiveType::Gather {
                    item: "animal_pelt".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 100,
                items: vec![("skinning_knife".to_string(), 1)],
                reputation: vec![(Faction::Hunters, 50)],
                skill_points: Some(("hunting".to_string(), 75)),
                unlocks: vec!["side_hunting_02".to_string()],
            },
            prerequisites: vec!["main_02_proving".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(5), // Hunter NPC
            turn_in_npc: Some(5),
            time_limit: None,
        });

        // Side Quest - Archaeology Introduction
        self.add_quest(Quest {
            id: "side_dig_01".to_string(),
            title: "Bones of the Ancients".to_string(),
            description: "The shaman speaks of ancient beasts whose bones hold power.".to_string(),
            quest_type: QuestType::Side,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("find_dig_site", "Find a fossil dig site", ObjectiveType::Explore {
                    location_id: 20,
                    radius: 50.0,
                }),
                QuestObjective::new("extract_fossil", "Extract a fossil", ObjectiveType::Gather {
                    item: "fossil".to_string(),
                    count: 1,
                    current: 0,
                }),
                QuestObjective::new("bring_shaman", "Bring the fossil to the shaman", ObjectiveType::TalkTo {
                    npc_id: 3,
                    dialogue_id: "shaman_fossil".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 75,
                items: vec![("digging_tool".to_string(), 1)],
                reputation: vec![(Faction::Shamans, 50)],
                skill_points: Some(("archaeology".to_string(), 50)),
                unlocks: vec!["side_dig_02".to_string()],
            },
            prerequisites: vec![],
            state: QuestState::Available,
            giver_npc: Some(3), // Shaman
            turn_in_npc: Some(3),
            time_limit: None,
        });

        // Hunting Contract - Repeatable
        self.add_quest(Quest {
            id: "contract_wolf_pack".to_string(),
            title: "Wolf Pack Bounty".to_string(),
            description: "A wolf pack is terrorizing travelers. Eliminate the threat.".to_string(),
            quest_type: QuestType::Contract,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("kill_wolves", "Kill wolves", ObjectiveType::Kill {
                    species: "Gray Wolf".to_string(),
                    count: 5,
                    current: 0,
                }),
                QuestObjective::new("kill_alpha", "Kill the pack alpha", ObjectiveType::SpecialKill {
                    condition: "alpha".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 150,
                items: vec![("wolf_bounty_gold".to_string(), 50)],
                reputation: vec![(Faction::NativeCouncil, 20)],
                skill_points: None,
                unlocks: vec![],
            },
            prerequisites: vec!["main_02_proving".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(2),
            turn_in_npc: Some(2),
            time_limit: Some(3.0), // 3 in-game days
        });

        // Legendary Hunt
        self.add_quest(Quest {
            id: "legendary_ghost_cougar".to_string(),
            title: "The Ghost of the Mountain".to_string(),
            description: "Legends speak of an albino cougar that stalks the peaks. Prove your worth as a legendary hunter.".to_string(),
            quest_type: QuestType::Legendary,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("find_tracks", "Find the Ghost Cougar's tracks", ObjectiveType::Track {
                    species: "Ghost Cougar".to_string(),
                }),
                QuestObjective::new("reach_summit", "Reach the mountain summit", ObjectiveType::Explore {
                    location_id: 100,
                    radius: 20.0,
                }),
                QuestObjective::new("hunt_ghost", "Hunt the Ghost Cougar", ObjectiveType::Kill {
                    species: "Ghost Cougar".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 1000,
                items: vec![("ghost_cougar_pelt".to_string(), 1), ("invisibility_cloak".to_string(), 1)],
                reputation: vec![(Faction::Hunters, 200), (Faction::NativeCouncil, 100)],
                skill_points: Some(("hunting".to_string(), 500)),
                unlocks: vec![],
            },
            prerequisites: vec!["hunting_legendary_hunter".to_string()], // Requires Legendary Hunter skill
            state: QuestState::Locked,
            giver_npc: None, // Self-discovered
            turn_in_npc: None,
            time_limit: None,
        });

        // === CHAPTER 2 - Main Story ===

        self.add_quest(Quest {
            id: "main_04_shaman".to_string(),
            title: "The Shaman's Vision".to_string(),
            description: "The village shaman has had a troubling vision. Seek her wisdom in the Sacred Grove.".to_string(),
            quest_type: QuestType::MainStory,
            chapter: 2,
            objectives: vec![
                QuestObjective::new("visit_grove", "Visit the Sacred Grove at night", ObjectiveType::Explore {
                    location_id: 30,
                    radius: 30.0,
                }),
                QuestObjective::new("speak_shaman", "Speak with the Shaman", ObjectiveType::TalkTo {
                    npc_id: 3,
                    dialogue_id: "shaman_vision".to_string(),
                }),
                QuestObjective::new("gather_spirit_herbs", "Gather spirit herbs for the ritual", ObjectiveType::Gather {
                    item: "spirit_herb".to_string(),
                    count: 3,
                    current: 0,
                }),
                QuestObjective::new("complete_ritual", "Complete the vision ritual", ObjectiveType::Discover {
                    item: "vision_complete".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 300,
                items: vec![("spirit_totem".to_string(), 1)],
                reputation: vec![(Faction::Shamans, 100), (Faction::NativeCouncil, 50)],
                skill_points: Some(("archaeology".to_string(), 100)),
                unlocks: vec!["main_05_darkness".to_string()],
            },
            prerequisites: vec!["main_03_threat".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(3),
            turn_in_npc: Some(3),
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "main_05_darkness".to_string(),
            title: "Darkness Rising".to_string(),
            description: "The vision showed a great darkness threatening the land. Investigate the corrupted areas.".to_string(),
            quest_type: QuestType::MainStory,
            chapter: 2,
            objectives: vec![
                QuestObjective::new("investigate_swamp", "Investigate the darkened swamp", ObjectiveType::Explore {
                    location_id: 40,
                    radius: 100.0,
                }),
                QuestObjective::new("defeat_corrupted", "Defeat corrupted creatures", ObjectiveType::Kill {
                    species: "Corrupted Wolf".to_string(),
                    count: 5,
                    current: 0,
                }),
                QuestObjective::new("find_source", "Find the source of corruption", ObjectiveType::Discover {
                    item: "corruption_source".to_string(),
                }),
                QuestObjective::new("report_council", "Report to the village council", ObjectiveType::TalkTo {
                    npc_id: 1,
                    dialogue_id: "elder_darkness_report".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 400,
                items: vec![("purification_charm".to_string(), 1)],
                reputation: vec![(Faction::NativeCouncil, 100), (Faction::Warriors, 50)],
                skill_points: Some(("hunting".to_string(), 150)),
                unlocks: vec!["main_06_alliance".to_string(), "side_cleanse_01".to_string()],
            },
            prerequisites: vec!["main_04_shaman".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(3),
            turn_in_npc: Some(1),
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "main_06_alliance".to_string(),
            title: "Forging Alliances".to_string(),
            description: "The council believes we need allies from other villages. Travel to negotiate.".to_string(),
            quest_type: QuestType::MainStory,
            chapter: 2,
            objectives: vec![
                QuestObjective::new("travel_north", "Travel to the northern village", ObjectiveType::Explore {
                    location_id: 50,
                    radius: 50.0,
                }),
                QuestObjective::new("speak_chief", "Speak with Chief Running Bear", ObjectiveType::TalkTo {
                    npc_id: 10,
                    dialogue_id: "chief_alliance".to_string(),
                }),
                QuestObjective::new("prove_worth", "Complete their trial of worth", ObjectiveType::SpecialKill {
                    condition: "trial_creature".to_string(),
                    count: 1,
                    current: 0,
                }),
                QuestObjective::new("secure_alliance", "Secure the alliance", ObjectiveType::TalkTo {
                    npc_id: 10,
                    dialogue_id: "chief_alliance_complete".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 500,
                items: vec![("alliance_token".to_string(), 1), ("tribal_armor".to_string(), 1)],
                reputation: vec![(Faction::NativeCouncil, 150), (Faction::NativeVillage(2), 200)],
                skill_points: None,
                unlocks: vec!["main_07_gathering".to_string()],
            },
            prerequisites: vec!["main_05_darkness".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(1),
            turn_in_npc: Some(1),
            time_limit: None,
        });

        // === Extended Side Quests ===

        self.add_quest(Quest {
            id: "side_hunting_02".to_string(),
            title: "Master of the Hunt".to_string(),
            description: "Prove your mastery by hunting dangerous game.".to_string(),
            quest_type: QuestType::Side,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("kill_bear", "Kill a black bear", ObjectiveType::Kill {
                    species: "Black Bear".to_string(),
                    count: 1,
                    current: 0,
                }),
                QuestObjective::new("kill_cougar", "Kill a cougar", ObjectiveType::Kill {
                    species: "Eastern Cougar".to_string(),
                    count: 1,
                    current: 0,
                }),
                QuestObjective::new("pelts", "Collect 5 quality pelts", ObjectiveType::Gather {
                    item: "quality_pelt".to_string(),
                    count: 5,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 200,
                items: vec![("hunter_bow_improved".to_string(), 1)],
                reputation: vec![(Faction::Hunters, 100)],
                skill_points: Some(("hunting".to_string(), 150)),
                unlocks: vec!["side_hunting_03".to_string()],
            },
            prerequisites: vec!["side_hunting_01".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(5),
            turn_in_npc: Some(5),
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "side_hunting_03".to_string(),
            title: "The Alpha Hunter".to_string(),
            description: "Track and defeat pack alphas to prove supreme hunting skill.".to_string(),
            quest_type: QuestType::Side,
            chapter: 2,
            objectives: vec![
                QuestObjective::new("track_alpha", "Track a wolf pack to their den", ObjectiveType::Track {
                    species: "Wolf Pack".to_string(),
                }),
                QuestObjective::new("kill_alphas", "Kill pack alphas", ObjectiveType::SpecialKill {
                    condition: "alpha".to_string(),
                    count: 3,
                    current: 0,
                }),
                QuestObjective::new("trophies", "Collect alpha trophies", ObjectiveType::Gather {
                    item: "alpha_trophy".to_string(),
                    count: 3,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 350,
                items: vec![("alpha_hunter_cloak".to_string(), 1)],
                reputation: vec![(Faction::Hunters, 150), (Faction::Warriors, 50)],
                skill_points: Some(("hunting".to_string(), 200)),
                unlocks: vec!["legendary_fenrir".to_string()],
            },
            prerequisites: vec!["side_hunting_02".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(5),
            turn_in_npc: Some(5),
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "side_dig_02".to_string(),
            title: "Secrets of the Earth".to_string(),
            description: "Excavate ancient sites to uncover prehistoric secrets.".to_string(),
            quest_type: QuestType::Side,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("find_sites", "Discover 3 dig sites", ObjectiveType::Explore {
                    location_id: 21,
                    radius: 30.0,
                }),
                QuestObjective::new("extract_rare", "Extract a rare fossil", ObjectiveType::Gather {
                    item: "rare_fossil".to_string(),
                    count: 1,
                    current: 0,
                }),
                QuestObjective::new("catalog", "Catalog 5 different fossil types", ObjectiveType::Gather {
                    item: "fossil_catalog_entry".to_string(),
                    count: 5,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 150,
                items: vec![("archaeologist_kit".to_string(), 1)],
                reputation: vec![(Faction::Shamans, 75)],
                skill_points: Some(("archaeology".to_string(), 100)),
                unlocks: vec!["side_dig_03".to_string()],
            },
            prerequisites: vec!["side_dig_01".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(3),
            turn_in_npc: Some(3),
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "side_dig_03".to_string(),
            title: "The Megalodon's Tooth".to_string(),
            description: "Rumors speak of a massive shark tooth near the coast.".to_string(),
            quest_type: QuestType::Side,
            chapter: 2,
            objectives: vec![
                QuestObjective::new("coastal_site", "Find the coastal dig site", ObjectiveType::Explore {
                    location_id: 60,
                    radius: 40.0,
                }),
                QuestObjective::new("excavate", "Carefully excavate the site", ObjectiveType::Gather {
                    item: "megalodon_tooth".to_string(),
                    count: 1,
                    current: 0,
                }),
                QuestObjective::new("study", "Have the shaman study the tooth", ObjectiveType::TalkTo {
                    npc_id: 3,
                    dialogue_id: "shaman_megalodon".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 250,
                items: vec![("megalodon_tooth_pendant".to_string(), 1)],
                reputation: vec![(Faction::Shamans, 100)],
                skill_points: Some(("archaeology".to_string(), 150)),
                unlocks: vec!["side_dig_mastodon".to_string()],
            },
            prerequisites: vec!["side_dig_02".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(3),
            turn_in_npc: Some(3),
            time_limit: None,
        });

        // === Village/Social Quests ===

        self.add_quest(Quest {
            id: "side_village_trader".to_string(),
            title: "The Merchant's Request".to_string(),
            description: "The village trader needs rare materials for trading.".to_string(),
            quest_type: QuestType::Side,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("gather_pelts", "Gather quality pelts", ObjectiveType::Gather {
                    item: "quality_pelt".to_string(),
                    count: 10,
                    current: 0,
                }),
                QuestObjective::new("gather_herbs", "Gather medicinal herbs", ObjectiveType::Gather {
                    item: "medicinal_herb".to_string(),
                    count: 15,
                    current: 0,
                }),
                QuestObjective::new("deliver", "Deliver to the trader", ObjectiveType::TalkTo {
                    npc_id: 6,
                    dialogue_id: "trader_delivery".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 100,
                items: vec![("gold_pouch".to_string(), 100)],
                reputation: vec![(Faction::Traders, 100)],
                skill_points: None,
                unlocks: vec!["side_village_trading_route".to_string()],
            },
            prerequisites: vec!["main_02_proving".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(6),
            turn_in_npc: Some(6),
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "side_village_healer".to_string(),
            title: "Healing Hands".to_string(),
            description: "The village healer needs help treating the sick.".to_string(),
            quest_type: QuestType::Side,
            chapter: 1,
            objectives: vec![
                QuestObjective::new("gather_special", "Gather special healing ingredients", ObjectiveType::Gather {
                    item: "spirit_moss".to_string(),
                    count: 5,
                    current: 0,
                }),
                QuestObjective::new("snake_venom", "Obtain snake venom (from rattlesnakes)", ObjectiveType::Gather {
                    item: "snake_venom".to_string(),
                    count: 3,
                    current: 0,
                }),
                QuestObjective::new("assist", "Assist with treatment", ObjectiveType::TalkTo {
                    npc_id: 7,
                    dialogue_id: "healer_assist".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 125,
                items: vec![("healing_poultice".to_string(), 5), ("antivenom".to_string(), 3)],
                reputation: vec![(Faction::NativeCouncil, 50), (Faction::Shamans, 25)],
                skill_points: None,
                unlocks: vec![],
            },
            prerequisites: vec!["main_01_lost_colony".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(7),
            turn_in_npc: Some(7),
            time_limit: None,
        });

        // === Cleansing/Corruption Quests ===

        self.add_quest(Quest {
            id: "side_cleanse_01".to_string(),
            title: "Cleansing the Land".to_string(),
            description: "Help the shaman cleanse corrupted areas.".to_string(),
            quest_type: QuestType::Side,
            chapter: 2,
            objectives: vec![
                QuestObjective::new("cleanse_pool", "Cleanse the corrupted pool", ObjectiveType::Explore {
                    location_id: 41,
                    radius: 20.0,
                }),
                QuestObjective::new("defeat_spirits", "Defeat corrupted spirits", ObjectiveType::Kill {
                    species: "Corrupted Spirit".to_string(),
                    count: 5,
                    current: 0,
                }),
                QuestObjective::new("place_totem", "Place cleansing totem", ObjectiveType::Discover {
                    item: "totem_placed".to_string(),
                }),
            ],
            rewards: QuestRewards {
                experience: 200,
                items: vec![("cleansing_totem".to_string(), 2)],
                reputation: vec![(Faction::Shamans, 75), (Faction::NativeCouncil, 50)],
                skill_points: Some(("archaeology".to_string(), 75)),
                unlocks: vec!["side_cleanse_02".to_string()],
            },
            prerequisites: vec!["main_05_darkness".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(3),
            turn_in_npc: Some(3),
            time_limit: None,
        });

        // === Additional Contracts ===

        self.add_quest(Quest {
            id: "contract_bear_cave".to_string(),
            title: "Bear Cave Clearance".to_string(),
            description: "A bear has moved too close to the village. Remove the threat.".to_string(),
            quest_type: QuestType::Contract,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("find_cave", "Find the bear's cave", ObjectiveType::Explore {
                    location_id: 70,
                    radius: 30.0,
                }),
                QuestObjective::new("kill_bear", "Kill or drive off the bear", ObjectiveType::Kill {
                    species: "Black Bear".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 100,
                items: vec![("bear_bounty_gold".to_string(), 30)],
                reputation: vec![(Faction::NativeCouncil, 15)],
                skill_points: None,
                unlocks: vec![],
            },
            prerequisites: vec!["main_02_proving".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(2),
            turn_in_npc: Some(2),
            time_limit: Some(2.0),
        });

        self.add_quest(Quest {
            id: "contract_alligator".to_string(),
            title: "Swamp Terror".to_string(),
            description: "An unusually large alligator is hunting near the fishing waters.".to_string(),
            quest_type: QuestType::Contract,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("track_gator", "Track the alligator", ObjectiveType::Track {
                    species: "American Alligator".to_string(),
                }),
                QuestObjective::new("kill_gator", "Kill the alligator", ObjectiveType::Kill {
                    species: "American Alligator".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 175,
                items: vec![("gator_bounty_gold".to_string(), 75), ("gator_hide".to_string(), 1)],
                reputation: vec![(Faction::NativeCouncil, 25)],
                skill_points: None,
                unlocks: vec![],
            },
            prerequisites: vec!["main_03_threat".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(2),
            turn_in_npc: Some(2),
            time_limit: Some(3.0),
        });

        self.add_quest(Quest {
            id: "contract_snake_nest".to_string(),
            title: "Snake Nest Clearance".to_string(),
            description: "A rattlesnake nest was found near the children's play area.".to_string(),
            quest_type: QuestType::Contract,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("find_nest", "Find the snake nest", ObjectiveType::Explore {
                    location_id: 71,
                    radius: 15.0,
                }),
                QuestObjective::new("clear_snakes", "Clear out the snakes", ObjectiveType::Kill {
                    species: "Timber Rattlesnake".to_string(),
                    count: 5,
                    current: 0,
                }),
                QuestObjective::new("collect_venom", "Collect venom for medicine", ObjectiveType::Gather {
                    item: "snake_venom".to_string(),
                    count: 2,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 75,
                items: vec![("snake_bounty_gold".to_string(), 25)],
                reputation: vec![(Faction::NativeCouncil, 20)],
                skill_points: None,
                unlocks: vec![],
            },
            prerequisites: vec!["main_01_lost_colony".to_string()],
            state: QuestState::Locked,
            giver_npc: Some(4),
            turn_in_npc: Some(4),
            time_limit: Some(1.0),
        });

        // === Additional Legendary Hunts ===

        self.add_quest(Quest {
            id: "legendary_fenrir".to_string(),
            title: "Fenrir, the Giant Wolf".to_string(),
            description: "The northern hunters speak of a wolf larger than any seen before. Hunt the legend.".to_string(),
            quest_type: QuestType::Legendary,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("northern_forest", "Venture into the deep northern forest", ObjectiveType::Explore {
                    location_id: 101,
                    radius: 50.0,
                }),
                QuestObjective::new("find_fenrir", "Find Fenrir's tracks", ObjectiveType::Track {
                    species: "Fenrir".to_string(),
                }),
                QuestObjective::new("defeat_pack", "Defeat Fenrir's wolf pack", ObjectiveType::Kill {
                    species: "Gray Wolf".to_string(),
                    count: 10,
                    current: 0,
                }),
                QuestObjective::new("hunt_fenrir", "Hunt Fenrir", ObjectiveType::Kill {
                    species: "Fenrir".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 1200,
                items: vec![("fenrir_pelt".to_string(), 1), ("wolf_spirit_token".to_string(), 1)],
                reputation: vec![(Faction::Hunters, 250), (Faction::Warriors, 150)],
                skill_points: Some(("hunting".to_string(), 600)),
                unlocks: vec![],
            },
            prerequisites: vec!["side_hunting_03".to_string()],
            state: QuestState::Locked,
            giver_npc: None,
            turn_in_npc: None,
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "legendary_swamp_king".to_string(),
            title: "The Swamp King".to_string(),
            description: "An ancient alligator rules the deepest swamp. Only the bravest dare face it.".to_string(),
            quest_type: QuestType::Legendary,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("deep_swamp", "Navigate to the deep swamp", ObjectiveType::Explore {
                    location_id: 102,
                    radius: 30.0,
                }),
                QuestObjective::new("track_king", "Track the Swamp King", ObjectiveType::Track {
                    species: "Swamp King".to_string(),
                }),
                QuestObjective::new("hunt_king", "Hunt the Swamp King", ObjectiveType::Kill {
                    species: "Swamp King".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 1100,
                items: vec![("swamp_king_hide".to_string(), 1), ("impenetrable_armor".to_string(), 1)],
                reputation: vec![(Faction::Hunters, 200), (Faction::NativeCouncil, 100)],
                skill_points: Some(("hunting".to_string(), 550)),
                unlocks: vec![],
            },
            prerequisites: vec!["contract_alligator".to_string()],
            state: QuestState::Locked,
            giver_npc: None,
            turn_in_npc: None,
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "legendary_old_silverback".to_string(),
            title: "Old Silverback".to_string(),
            description: "A bear of legend dwells in the ancient cave. Its silver fur is said to grant protection.".to_string(),
            quest_type: QuestType::Legendary,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("ancient_cave", "Find the ancient cave", ObjectiveType::Explore {
                    location_id: 103,
                    radius: 25.0,
                }),
                QuestObjective::new("track_silverback", "Track Old Silverback", ObjectiveType::Track {
                    species: "Old Silverback".to_string(),
                }),
                QuestObjective::new("hunt_silverback", "Hunt Old Silverback", ObjectiveType::Kill {
                    species: "Old Silverback".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 1000,
                items: vec![("silverback_pelt".to_string(), 1), ("bear_spirit_token".to_string(), 1)],
                reputation: vec![(Faction::Hunters, 175), (Faction::Shamans, 100)],
                skill_points: Some(("hunting".to_string(), 500)),
                unlocks: vec![],
            },
            prerequisites: vec!["contract_bear_cave".to_string()],
            state: QuestState::Locked,
            giver_npc: None,
            turn_in_npc: None,
            time_limit: None,
        });

        self.add_quest(Quest {
            id: "legendary_serpent_mother".to_string(),
            title: "The Serpent Mother".to_string(),
            description: "Deep in the hidden grotto lives a massive serpent guarding ancient eggs.".to_string(),
            quest_type: QuestType::Legendary,
            chapter: 0,
            objectives: vec![
                QuestObjective::new("hidden_grotto", "Find the hidden grotto", ObjectiveType::Explore {
                    location_id: 104,
                    radius: 20.0,
                }),
                QuestObjective::new("navigate_nest", "Navigate the serpent's nest", ObjectiveType::Explore {
                    location_id: 105,
                    radius: 10.0,
                }),
                QuestObjective::new("hunt_serpent", "Hunt the Serpent Mother", ObjectiveType::Kill {
                    species: "Serpent Mother".to_string(),
                    count: 1,
                    current: 0,
                }),
            ],
            rewards: QuestRewards {
                experience: 900,
                items: vec![("serpent_mother_skin".to_string(), 1), ("poison_mastery_token".to_string(), 1)],
                reputation: vec![(Faction::Hunters, 150), (Faction::Shamans, 150)],
                skill_points: Some(("hunting".to_string(), 450)),
                unlocks: vec![],
            },
            prerequisites: vec!["contract_snake_nest".to_string()],
            state: QuestState::Locked,
            giver_npc: None,
            turn_in_npc: None,
            time_limit: None,
        });
    }

    /// Add a quest to the manager
    pub fn add_quest(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }

    /// Start a quest
    pub fn start_quest(&mut self, quest_id: &str) -> Result<(), &'static str> {
        let quest = self.quests.get_mut(quest_id).ok_or("Quest not found")?;

        if quest.state != QuestState::Available {
            return Err("Quest not available");
        }

        quest.state = QuestState::Active;
        self.active.push(quest_id.to_string());
        Ok(())
    }

    /// Update quest progress
    pub fn update_progress(&mut self, quest_id: &str, objective_id: &str, progress: u32) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if let Some(obj) = quest.objectives.iter_mut().find(|o| o.id == objective_id) {
                obj.update_progress(progress);
                return obj.is_complete();
            }
        }
        false
    }

    /// Check if all objectives for a quest are complete
    pub fn is_quest_complete(&self, quest_id: &str) -> bool {
        self.quests.get(quest_id)
            .map(|q| q.objectives.iter().all(|o| o.is_complete()))
            .unwrap_or(false)
    }

    /// Complete a quest and get rewards
    pub fn complete_quest(&mut self, quest_id: &str) -> Option<QuestRewards> {
        // First check if quest can be completed
        let (rewards, unlocks) = {
            let quest = self.quests.get_mut(quest_id)?;

            if quest.state != QuestState::Active {
                return None;
            }

            if !quest.objectives.iter().all(|o| o.is_complete()) {
                return None;
            }

            quest.state = QuestState::Completed;
            (quest.rewards.clone(), quest.rewards.unlocks.clone())
        };

        self.active.retain(|id| id != quest_id);
        self.completed.push(quest_id.to_string());

        // Unlock dependent quests
        for unlock_id in unlocks {
            if let Some(unlock_quest) = self.quests.get_mut(&unlock_id) {
                if unlock_quest.state == QuestState::Locked {
                    unlock_quest.state = QuestState::Available;
                }
            }
        }

        Some(rewards)
    }

    /// Fail a quest
    pub fn fail_quest(&mut self, quest_id: &str) {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            quest.state = QuestState::Failed;
            self.active.retain(|id| id != quest_id);
            self.failed.push(quest_id.to_string());
        }
    }

    /// Get available quests for a specific NPC
    pub fn quests_for_npc(&self, npc_id: u32) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.giver_npc == Some(npc_id) && q.state == QuestState::Available)
            .collect()
    }

    /// Get quests ready to turn in to a specific NPC
    pub fn turn_in_for_npc(&self, npc_id: u32) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| {
                q.turn_in_npc == Some(npc_id)
                    && q.state == QuestState::Active
                    && q.objectives.iter().all(|o| o.is_complete())
            })
            .collect()
    }

    /// Process a kill event for relevant quests
    pub fn on_kill(&mut self, species: &str, was_stealth: bool, was_alpha: bool) -> Vec<String> {
        let mut completed_objectives = Vec::new();

        for quest_id in self.active.clone() {
            if let Some(quest) = self.quests.get_mut(&quest_id) {
                for obj in &mut quest.objectives {
                    match &mut obj.objective_type {
                        ObjectiveType::Kill { species: s, count, current } if s == species => {
                            *current += 1;
                            if *current >= *count {
                                completed_objectives.push(format!("{}:{}", quest_id, obj.id));
                            }
                        }
                        ObjectiveType::SpecialKill { condition, count, current } => {
                            let matches = match condition.as_str() {
                                "stealth" => was_stealth,
                                "alpha" => was_alpha,
                                _ => false,
                            };
                            if matches {
                                *current += 1;
                                if *current >= *count {
                                    completed_objectives.push(format!("{}:{}", quest_id, obj.id));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        completed_objectives
    }

    /// Process a gather event for relevant quests
    pub fn on_gather(&mut self, item: &str) -> Vec<String> {
        let mut completed_objectives = Vec::new();

        for quest_id in self.active.clone() {
            if let Some(quest) = self.quests.get_mut(&quest_id) {
                for obj in &mut quest.objectives {
                    if let ObjectiveType::Gather { item: i, count, current } = &mut obj.objective_type {
                        if i == item || item.contains(i.as_str()) {
                            *current += 1;
                            if *current >= *count {
                                completed_objectives.push(format!("{}:{}", quest_id, obj.id));
                            }
                        }
                    }
                }
            }
        }

        completed_objectives
    }

    /// Process location discovery
    pub fn on_location_reached(&mut self, location_id: u64, player_pos: Vec3) -> Vec<String> {
        let mut completed_objectives = Vec::new();

        for quest_id in self.active.clone() {
            if let Some(quest) = self.quests.get_mut(&quest_id) {
                for obj in &mut quest.objectives {
                    if let ObjectiveType::Explore { location_id: loc_id, .. } = &obj.objective_type {
                        if *loc_id == location_id {
                            obj.completed = true;
                            completed_objectives.push(format!("{}:{}", quest_id, obj.id));
                        }
                    }
                }
            }
        }

        completed_objectives
    }
}

/// Quest data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub quest_type: QuestType,
    pub chapter: u32,
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestRewards,
    pub prerequisites: Vec<String>,
    pub state: QuestState,
    pub giver_npc: Option<u32>,
    pub turn_in_npc: Option<u32>,
    pub time_limit: Option<f32>, // In-game days
}

/// Quest type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestType {
    MainStory,  // Campaign progression
    Side,       // Optional story content
    Contract,   // Repeatable bounties
    Legendary,  // Legendary beast hunts
    Discovery,  // Exploration-based
}

/// Quest state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestState {
    Locked,    // Prerequisites not met
    Available, // Can be started
    Active,    // In progress
    Completed, // Successfully finished
    Failed,    // Time ran out or failed condition
}

/// Quest objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub id: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub completed: bool,
    pub optional: bool,
}

impl QuestObjective {
    pub fn new(id: &str, description: &str, objective_type: ObjectiveType) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            objective_type,
            completed: false,
            optional: false,
        }
    }

    pub fn update_progress(&mut self, progress: u32) {
        match &mut self.objective_type {
            ObjectiveType::Kill { current, count, .. } => {
                *current = progress.min(*count);
                self.completed = *current >= *count;
            }
            ObjectiveType::Gather { current, count, .. } => {
                *current = progress.min(*count);
                self.completed = *current >= *count;
            }
            ObjectiveType::SpecialKill { current, count, .. } => {
                *current = progress.min(*count);
                self.completed = *current >= *count;
            }
            _ => {}
        }
    }

    pub fn is_complete(&self) -> bool {
        self.completed || self.optional
    }

    /// Get progress as (current, total)
    pub fn progress(&self) -> (u32, u32) {
        match &self.objective_type {
            ObjectiveType::Kill { current, count, .. } => (*current, *count),
            ObjectiveType::Gather { current, count, .. } => (*current, *count),
            ObjectiveType::SpecialKill { current, count, .. } => (*current, *count),
            _ => (if self.completed { 1 } else { 0 }, 1),
        }
    }
}

/// Types of quest objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveType {
    Kill {
        species: String,
        count: u32,
        current: u32,
    },
    SpecialKill {
        condition: String, // "stealth", "alpha", "no_damage"
        count: u32,
        current: u32,
    },
    Gather {
        item: String,
        count: u32,
        current: u32,
    },
    Explore {
        location_id: u64,
        radius: f32,
    },
    TalkTo {
        npc_id: u32,
        dialogue_id: String,
    },
    Discover {
        item: String,
    },
    Track {
        species: String,
    },
    Craft {
        item: String,
        count: u32,
        current: u32,
    },
    Escort {
        npc_id: u32,
        destination: u64,
    },
    Defend {
        location_id: u64,
        duration: f32,
    },
}

/// Quest rewards
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestRewards {
    pub experience: u32,
    pub items: Vec<(String, u32)>,
    pub reputation: Vec<(Faction, i32)>,
    pub skill_points: Option<(String, u32)>,
    pub unlocks: Vec<String>,
}
