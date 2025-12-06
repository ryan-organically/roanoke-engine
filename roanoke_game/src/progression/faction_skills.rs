//! Faction Skill Trees
//!
//! Implements the signature skill trees for each faction, with tiered progression
//! and unlock requirements.

use super::faction::{Faction, Standing};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// SKILL TREE STRUCTURE
// ============================================================================

/// Unique identifier for faction skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionSkillId {
    // ========== SPANISH SKILLS ==========
    ConquistadorInitiate,
    SwordAndBuckler,
    ArquebusMastery,
    TercioFormation,
    DuelistsGrace,
    VolleyFire,
    MarkmansPatience,
    SteelTempest,
    ThunderOfGod,
    GoldAndGlory,
    ElConquistador,

    // ========== FRENCH SKILLS ==========
    ApprentiVoyageur,
    FurTrade,
    ForestWisdom,
    MasterTrapper,
    NegotiatorsTongue,
    SilentShadow,
    HerbalistsCraft,
    TradeEmpire,
    OneWithLand,
    SpiritBridge,
    GrandVoyageur,

    // ========== ENGLISH SKILLS ==========
    RoanokeSettler,
    Fortification,
    FrontierSurvival,
    MasterBuilder,
    MilitiaCaptain,
    WildernessScout,
    ColonialFarmer,
    ColonialLeader,
    FrontierMaster,
    NewWorldGovernor,
    LordOfRoanoke,

    // ========== AZTEC SKILLS ==========
    MacehuallinInitiate,
    JaguarWarrior,
    EagleWarrior,
    OcelotlFury,
    ShadowStalker,
    CuauhtliStrike,
    SolarAscension,
    JaguarKnight,
    EagleKnight,
    Cuachicqueh,
    ChampionOfTheSun,

    // ========== POWHATAN SKILLS ==========
    PowhatanNewcomer,
    HuntersPath,
    DiplomatsPath,
    DeerStalker,
    RiverKeeper,
    PeaceWeaver,
    WarChief,
    SpiritWalker,
    ConfederateLord,
    Werowance,
    Mamanatowick,

    // ========== TUSCARORA SKILLS ==========
    FriendOfTuscarora,
    ClanWarrior,
    ClanProvider,
    BearClanFury,
    WolfClanPack,
    TurtleClanWisdom,
    DeerClanGrace,
    TuscaroraWarCaptain,
    ClanMother,
    PeaceChief,
    KeeperOfTheFire,

    // ========== CHEROKEE SKILLS ==========
    CherokeeFriend,
    RedWarPath,
    WhitePeacePath,
    RavenMocker,
    CherokeeWarPriest,
    MedicineWalker,
    BelovedElder,
    RedWarChief,
    WhitePeaceChief,
    FirstBelovedMan,
    UkuOfCherokee,

    // ========== CATAWBA SKILLS ==========
    CatawbaAcquaintance,
    RiverFighter,
    CatawbaTradeMaster,
    RaidLeader,
    WaterAmbush,
    PotteryArtisan,
    MarketManipulator,
    RiverHawk,
    TradeLord,
    EsawChief,
    KingOfCatawba,

    // ========== PAMUNKEY SKILLS ==========
    PamunkeyAccepted,
    RoyalTradition,
    SacredKeeper,
    LineageHeir,
    CornLord,
    TempleGuardian,
    HistoryKeeper,
    RoyalBlood,
    SacredWisdom,
    ParamountHeir,
    BloodOfPowhatan,
}

impl FactionSkillId {
    /// Get the faction this skill belongs to
    pub fn faction(&self) -> Faction {
        use FactionSkillId::*;
        match self {
            ConquistadorInitiate | SwordAndBuckler | ArquebusMastery | TercioFormation
            | DuelistsGrace | VolleyFire | MarkmansPatience | SteelTempest | ThunderOfGod
            | GoldAndGlory | ElConquistador => Faction::Spanish,

            ApprentiVoyageur | FurTrade | ForestWisdom | MasterTrapper | NegotiatorsTongue
            | SilentShadow | HerbalistsCraft | TradeEmpire | OneWithLand | SpiritBridge
            | GrandVoyageur => Faction::French,

            RoanokeSettler | Fortification | FrontierSurvival | MasterBuilder | MilitiaCaptain
            | WildernessScout | ColonialFarmer | ColonialLeader | FrontierMaster
            | NewWorldGovernor | LordOfRoanoke => Faction::English,

            MacehuallinInitiate | JaguarWarrior | EagleWarrior | OcelotlFury | ShadowStalker
            | CuauhtliStrike | SolarAscension | JaguarKnight | EagleKnight | Cuachicqueh
            | ChampionOfTheSun => Faction::Aztec,

            PowhatanNewcomer | HuntersPath | DiplomatsPath | DeerStalker | RiverKeeper
            | PeaceWeaver | WarChief | SpiritWalker | ConfederateLord | Werowance
            | Mamanatowick => Faction::Powhatan,

            FriendOfTuscarora | ClanWarrior | ClanProvider | BearClanFury | WolfClanPack
            | TurtleClanWisdom | DeerClanGrace | TuscaroraWarCaptain | ClanMother | PeaceChief
            | KeeperOfTheFire => Faction::Tuscarora,

            CherokeeFriend | RedWarPath | WhitePeacePath | RavenMocker | CherokeeWarPriest
            | MedicineWalker | BelovedElder | RedWarChief | WhitePeaceChief | FirstBelovedMan
            | UkuOfCherokee => Faction::Cherokee,

            CatawbaAcquaintance | RiverFighter | CatawbaTradeMaster | RaidLeader | WaterAmbush
            | PotteryArtisan | MarketManipulator | RiverHawk | TradeLord | EsawChief
            | KingOfCatawba => Faction::Catawba,

            PamunkeyAccepted | RoyalTradition | SacredKeeper | LineageHeir | CornLord
            | TempleGuardian | HistoryKeeper | RoyalBlood | SacredWisdom | ParamountHeir
            | BloodOfPowhatan => Faction::Pamunkey,
        }
    }

    /// Get the tier of this skill (1-6)
    pub fn tier(&self) -> u8 {
        use FactionSkillId::*;
        match self {
            // Tier 1
            ConquistadorInitiate | ApprentiVoyageur | RoanokeSettler | MacehuallinInitiate
            | PowhatanNewcomer | FriendOfTuscarora | CherokeeFriend | CatawbaAcquaintance
            | PamunkeyAccepted => 1,

            // Tier 2
            SwordAndBuckler | ArquebusMastery | FurTrade | ForestWisdom | Fortification
            | FrontierSurvival | JaguarWarrior | EagleWarrior | HuntersPath | DiplomatsPath
            | ClanWarrior | ClanProvider | RedWarPath | WhitePeacePath | RiverFighter
            | CatawbaTradeMaster | RoyalTradition | SacredKeeper => 2,

            // Tier 3
            TercioFormation | DuelistsGrace | VolleyFire | MarkmansPatience | MasterTrapper
            | NegotiatorsTongue | SilentShadow | HerbalistsCraft | MasterBuilder
            | MilitiaCaptain | WildernessScout | ColonialFarmer | OcelotlFury | ShadowStalker
            | CuauhtliStrike | SolarAscension | DeerStalker | RiverKeeper | PeaceWeaver
            | WarChief | BearClanFury | WolfClanPack | TurtleClanWisdom | DeerClanGrace
            | RavenMocker | CherokeeWarPriest | MedicineWalker | BelovedElder | RaidLeader
            | WaterAmbush | PotteryArtisan | MarketManipulator | LineageHeir | CornLord
            | TempleGuardian | HistoryKeeper => 3,

            // Tier 4
            SteelTempest | ThunderOfGod | TradeEmpire | OneWithLand | ColonialLeader
            | FrontierMaster | JaguarKnight | EagleKnight | SpiritWalker | ConfederateLord
            | TuscaroraWarCaptain | ClanMother | RedWarChief | WhitePeaceChief | RiverHawk
            | TradeLord | RoyalBlood | SacredWisdom => 4,

            // Tier 5
            GoldAndGlory | SpiritBridge | NewWorldGovernor | Cuachicqueh | Werowance
            | PeaceChief | FirstBelovedMan | EsawChief | ParamountHeir => 5,

            // Tier 6 (Ultimate)
            ElConquistador | GrandVoyageur | LordOfRoanoke | ChampionOfTheSun | Mamanatowick
            | KeeperOfTheFire | UkuOfCherokee | KingOfCatawba | BloodOfPowhatan => 6,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        use FactionSkillId::*;
        match self {
            // Spanish
            ConquistadorInitiate => "Conquistador Initiate",
            SwordAndBuckler => "Sword & Buckler",
            ArquebusMastery => "Arquebus Mastery",
            TercioFormation => "Tercio Formation",
            DuelistsGrace => "Duelist's Grace",
            VolleyFire => "Volley Fire",
            MarkmansPatience => "Marksman's Patience",
            SteelTempest => "Steel Tempest",
            ThunderOfGod => "Thunder of God",
            GoldAndGlory => "Gold & Glory",
            ElConquistador => "El Conquistador",

            // French
            ApprentiVoyageur => "Apprenti Voyageur",
            FurTrade => "Fur Trade",
            ForestWisdom => "Forest Wisdom",
            MasterTrapper => "Master Trapper",
            NegotiatorsTongue => "Negotiator's Tongue",
            SilentShadow => "Silent Shadow",
            HerbalistsCraft => "Herbalist's Craft",
            TradeEmpire => "Trade Empire",
            OneWithLand => "One With the Land",
            SpiritBridge => "Spirit Bridge",
            GrandVoyageur => "Grand Voyageur",

            // English
            RoanokeSettler => "Roanoke Settler",
            Fortification => "Fortification",
            FrontierSurvival => "Frontier Survival",
            MasterBuilder => "Master Builder",
            MilitiaCaptain => "Militia Captain",
            WildernessScout => "Wilderness Scout",
            ColonialFarmer => "Colonial Farmer",
            ColonialLeader => "Colonial Leader",
            FrontierMaster => "Frontier Master",
            NewWorldGovernor => "New World Governor",
            LordOfRoanoke => "Lord of Roanoke",

            // Aztec
            MacehuallinInitiate => "Macehualtin Initiate",
            JaguarWarrior => "Jaguar Warrior",
            EagleWarrior => "Eagle Warrior",
            OcelotlFury => "Ocelotl Fury",
            ShadowStalker => "Shadow Stalker",
            CuauhtliStrike => "Cuauhtli Strike",
            SolarAscension => "Solar Ascension",
            JaguarKnight => "Jaguar Knight",
            EagleKnight => "Eagle Knight",
            Cuachicqueh => "Cuachicqueh (Shorn One)",
            ChampionOfTheSun => "Champion of the Sun",

            // Powhatan
            PowhatanNewcomer => "Newcomer",
            HuntersPath => "Hunter's Path",
            DiplomatsPath => "Diplomat's Path",
            DeerStalker => "Deer Stalker",
            RiverKeeper => "River Keeper",
            PeaceWeaver => "Peace Weaver",
            WarChief => "War Chief",
            SpiritWalker => "Spirit Walker",
            ConfederateLord => "Confederate Lord",
            Werowance => "Werowance (Chief)",
            Mamanatowick => "Mamanatowick",

            // Tuscarora
            FriendOfTuscarora => "Friend of Tuscarora",
            ClanWarrior => "Clan Warrior",
            ClanProvider => "Clan Provider",
            BearClanFury => "Bear Clan Fury",
            WolfClanPack => "Wolf Clan Pack",
            TurtleClanWisdom => "Turtle Clan Wisdom",
            DeerClanGrace => "Deer Clan Grace",
            TuscaroraWarCaptain => "War Captain",
            ClanMother => "Clan Mother",
            PeaceChief => "Peace Chief",
            KeeperOfTheFire => "Keeper of the Fire",

            // Cherokee
            CherokeeFriend => "Cherokee Friend",
            RedWarPath => "Red War Path",
            WhitePeacePath => "White Peace Path",
            RavenMocker => "Raven Mocker",
            CherokeeWarPriest => "War Priest (Didanawisgi)",
            MedicineWalker => "Medicine Walker",
            BelovedElder => "Beloved Elder",
            RedWarChief => "Red War Chief",
            WhitePeaceChief => "White Peace Chief",
            FirstBelovedMan => "First Beloved Man",
            UkuOfCherokee => "Uku of the Cherokee",

            // Catawba
            CatawbaAcquaintance => "Catawba Acquaintance",
            RiverFighter => "River Fighter",
            CatawbaTradeMaster => "Trade Master",
            RaidLeader => "Raid Leader",
            WaterAmbush => "Water Ambush",
            PotteryArtisan => "Pottery Artisan",
            MarketManipulator => "Market Manipulator",
            RiverHawk => "River Hawk",
            TradeLord => "Trade Lord",
            EsawChief => "Esaw Chief",
            KingOfCatawba => "King of the Catawba",

            // Pamunkey
            PamunkeyAccepted => "Pamunkey Accepted",
            RoyalTradition => "Royal Tradition",
            SacredKeeper => "Sacred Keeper",
            LineageHeir => "Lineage Heir",
            CornLord => "Corn Lord",
            TempleGuardian => "Temple Guardian",
            HistoryKeeper => "History Keeper",
            RoyalBlood => "Royal Blood",
            SacredWisdom => "Sacred Wisdom",
            ParamountHeir => "Paramount Heir",
            BloodOfPowhatan => "Blood of Powhatan",
        }
    }
}

/// Definition of a skill node in the tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSkill {
    pub id: FactionSkillId,
    pub name: &'static str,
    pub description: &'static str,
    pub tier: u8,
    pub faction: Faction,
    pub prerequisites: Vec<FactionSkillId>,
    pub required_standing: Standing,
    pub unlock_condition: UnlockCondition,
    pub effects: Vec<SkillEffect>,
    pub skill_point_cost: u32,
}

/// Conditions that must be met to unlock a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockCondition {
    /// No special condition (just prerequisites and standing)
    None,
    /// Must kill a certain number of enemies
    KillCount { count: u32, target: Option<String> },
    /// Must complete specific quests
    QuestComplete { quest_ids: Vec<String> },
    /// Must accumulate wealth
    WealthAmount { amount: u32 },
    /// Must survive for duration
    SurvivalDays { days: u32, location: Option<String> },
    /// Must craft items
    CraftCount { count: u32, item_type: Option<String> },
    /// Must trade successfully
    TradeCount { count: u32 },
    /// Must discover locations
    DiscoveryCount { count: u32 },
    /// Must defend settlements
    DefenseCount { count: u32 },
    /// Must capture enemies
    CaptureCount { count: u32 },
    /// Must win ritual combat
    RitualCombatWins { count: u32 },
    /// Must complete vision quest
    VisionQuest,
    /// Special achievements
    Achievement { achievement_id: String },
}

/// Effects provided by a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffect {
    /// Passive stat modifier
    StatModifier {
        stat: SkillStat,
        modifier: f32,
        condition: Option<EffectCondition>,
    },
    /// Unlock ability
    UnlockAbility { ability_id: String },
    /// Unlock weapon access
    UnlockWeapon { weapon_id: String },
    /// Unlock crafting recipe
    UnlockRecipe { recipe_id: String },
    /// Command followers
    FollowerCommand { max_followers: u8 },
    /// Access to special locations
    LocationAccess { location_type: String },
    /// Trade modifier
    TradeBonus { multiplier: f32 },
    /// Reputation gain modifier
    ReputationModifier { factions: Vec<Faction>, multiplier: f32 },
    /// Resource gathering bonus
    GatheringBonus { resource: String, multiplier: f32 },
    /// Companion summon
    CompanionUnlock { companion_type: String, count: u8 },
    /// Title grant
    TitleGrant { title: String },
    /// Special passive
    SpecialPassive { description: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillStat {
    MeleeDamage,
    RangedDamage,
    Health,
    Stamina,
    Speed,
    Stealth,
    DetectionRange,
    BlockEffectiveness,
    ParryWindow,
    ReloadSpeed,
    DrawSpeed,
    TrackingRange,
    HealingEffectiveness,
    CraftingQuality,
    TradePrice,
    ReputationGain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectCondition {
    NearWater,
    InForest,
    InMountain,
    AtNight,
    AtDay,
    NearAllies,
    NearSacredSite,
    BelowHealthPercent(f32),
    WhileCrouched,
    WithSpecificWeapon(String),
}

// ============================================================================
// SKILL TREE DATA
// ============================================================================

/// Get all skills for a faction
pub fn get_faction_skill_tree(faction: Faction) -> Vec<FactionSkill> {
    match faction {
        Faction::Spanish => get_spanish_skills(),
        Faction::French => get_french_skills(),
        Faction::English => get_english_skills(),
        Faction::Aztec => get_aztec_skills(),
        Faction::Powhatan => get_powhatan_skills(),
        Faction::Tuscarora => get_tuscarora_skills(),
        Faction::Cherokee => get_cherokee_skills(),
        Faction::Catawba => get_catawba_skills(),
        Faction::Pamunkey => get_pamunkey_skills(),
        Faction::Independent | Faction::Wildlife => vec![],
    }
}

fn get_spanish_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: ConquistadorInitiate,
            name: "Conquistador Initiate",
            description: "Join the Spanish faction and gain access to their weapons and training",
            tier: 1,
            faction: Faction::Spanish,
            prerequisites: vec![],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::LocationAccess { location_type: "spanish_armory".into() },
                SkillEffect::UnlockRecipe { recipe_id: "paper_cartridge".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: SwordAndBuckler,
            name: "Sword & Buckler",
            description: "Master the classic Spanish fighting style with sword and small shield",
            tier: 2,
            faction: Faction::Spanish,
            prerequisites: vec![ConquistadorInitiate],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 25, target: None },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::BlockEffectiveness, modifier: 0.10, condition: None },
                SkillEffect::StatModifier { stat: SkillStat::ParryWindow, modifier: 0.2, condition: None },
                SkillEffect::UnlockAbility { ability_id: "buckler_bash".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: ArquebusMastery,
            name: "Arquebus Mastery",
            description: "Become proficient with Spanish firearms",
            tier: 2,
            faction: Faction::Spanish,
            prerequisites: vec![ConquistadorInitiate],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 15, target: Some("firearm".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::RangedDamage, modifier: 0.15, condition: None },
                SkillEffect::StatModifier { stat: SkillStat::ReloadSpeed, modifier: 0.15, condition: None },
                SkillEffect::UnlockRecipe { recipe_id: "paper_cartridge".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: TercioFormation,
            name: "Tercio Formation",
            description: "Fight effectively alongside allies in formation",
            tier: 3,
            faction: Faction::Spanish,
            prerequisites: vec![SwordAndBuckler],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::DefenseCount { count: 3 },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::Health, modifier: 0.20, condition: Some(EffectCondition::NearAllies) },
                SkillEffect::FollowerCommand { max_followers: 4 },
                SkillEffect::UnlockAbility { ability_id: "hold_the_line".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: DuelistsGrace,
            name: "Duelist's Grace",
            description: "Excel in one-on-one combat with elegant swordplay",
            tier: 3,
            faction: Faction::Spanish,
            prerequisites: vec![SwordAndBuckler],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::RitualCombatWins { count: 10 },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::Speed, modifier: 0.25, condition: None },
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.40, condition: None },
                SkillEffect::UnlockAbility { ability_id: "estocada".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: VolleyFire,
            name: "Volley Fire",
            description: "Coordinate devastating ranged attacks with allies",
            tier: 3,
            faction: Faction::Spanish,
            prerequisites: vec![ArquebusMastery],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::KillCount { count: 3, target: Some("penetration_kill".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::RangedDamage, modifier: 0.50, condition: Some(EffectCondition::NearAllies) },
                SkillEffect::UnlockAbility { ability_id: "fire_by_rank".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: MarkmansPatience,
            name: "Marksman's Patience",
            description: "Perfect your aim for devastating precision shots",
            tier: 3,
            faction: Faction::Spanish,
            prerequisites: vec![ArquebusMastery],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 20, target: Some("headshot".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::RangedDamage, modifier: 1.0, condition: Some(EffectCondition::WhileCrouched) },
                SkillEffect::SpecialPassive { description: "Hold breath 8s instead of 4s".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: SteelTempest,
            name: "Steel Tempest",
            description: "Become a whirlwind of steel in melee combat",
            tier: 4,
            faction: Faction::Spanish,
            prerequisites: vec![TercioFormation, DuelistsGrace],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::KillCount { count: 50, target: Some("spanish_steel".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.15, condition: None },
                SkillEffect::UnlockAbility { ability_id: "whirlwind".into() },
                SkillEffect::SpecialPassive { description: "Toledo steel never degrades below 50%".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: ThunderOfGod,
            name: "Thunder of God",
            description: "Your firearms strike terror into enemies",
            tier: 4,
            faction: Faction::Spanish,
            prerequisites: vec![VolleyFire, MarkmansPatience],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::KillCount { count: 1, target: Some("legendary_animal_firearm".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::RangedDamage, modifier: 0.20, condition: None },
                SkillEffect::UnlockAbility { ability_id: "terrifying_shot".into() },
                SkillEffect::UnlockRecipe { recipe_id: "incendiary_rounds".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: GoldAndGlory,
            name: "Gold & Glory",
            description: "Your nose for treasure is unmatched",
            tier: 5,
            faction: Faction::Spanish,
            prerequisites: vec![SteelTempest, ThunderOfGod],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::WealthAmount { amount: 5000 },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Sense buried treasure within 50m".into() },
                SkillEffect::TradeBonus { multiplier: 1.50 },
                SkillEffect::LocationAccess { location_type: "spanish_legendary_shop".into() },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: ElConquistador,
            name: "El Conquistador",
            description: "You are the ultimate Spanish warrior, feared across the land",
            tier: 6,
            faction: Faction::Spanish,
            prerequisites: vec![GoldAndGlory],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["seven_cities".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "El Conquistador".into() },
                SkillEffect::UnlockWeapon { weapon_id: "gilded_morion".into() },
                SkillEffect::UnlockAbility { ability_id: "conquerors_presence".into() },
                SkillEffect::FollowerCommand { max_followers: 8 },
                SkillEffect::SpecialPassive { description: "Establish Spanish outposts anywhere".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

fn get_french_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: ApprentiVoyageur,
            name: "Apprenti Voyageur",
            description: "Begin your journey as a French woodsman and trader",
            tier: 1,
            faction: Faction::French,
            prerequisites: vec![],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::UnlockRecipe { recipe_id: "basic_trap".into() },
                SkillEffect::SpecialPassive { description: "Canoe handling unlocked".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: FurTrade,
            name: "Fur Trade",
            description: "Master the lucrative fur trade",
            tier: 2,
            faction: Faction::French,
            prerequisites: vec![ApprentiVoyageur],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::TradeCount { count: 50 },
            effects: vec![
                SkillEffect::GatheringBonus { resource: "pelts".into(), multiplier: 1.20 },
                SkillEffect::TradeBonus { multiplier: 1.20 },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: ForestWisdom,
            name: "Forest Wisdom",
            description: "Learn to survive indefinitely in the wilderness",
            tier: 2,
            faction: Faction::French,
            prerequisites: vec![ApprentiVoyageur],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::SurvivalDays { days: 10, location: Some("wilderness".into()) },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Natural shelter construction".into() },
                SkillEffect::SpecialPassive { description: "Edible plants highlighted".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: MasterTrapper,
            name: "Master Trapper",
            description: "Your traps are deadly and efficient",
            tier: 3,
            faction: Faction::French,
            prerequisites: vec![FurTrade],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::CraftCount { count: 100, item_type: Some("trap".into()) },
            effects: vec![
                SkillEffect::UnlockRecipe { recipe_id: "steel_trap".into() },
                SkillEffect::SpecialPassive { description: "Traps reset automatically once".into() },
                SkillEffect::StatModifier { stat: SkillStat::Stealth, modifier: 0.30, condition: None },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: NegotiatorsTongue,
            name: "Negotiator's Tongue",
            description: "Your diplomatic skills open all doors",
            tier: 3,
            faction: Faction::French,
            prerequisites: vec![FurTrade],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::TradeCount { count: 20 },
            effects: vec![
                SkillEffect::TradeBonus { multiplier: 1.15 },
                SkillEffect::UnlockAbility { ability_id: "trade_parley".into() },
                SkillEffect::ReputationModifier { factions: vec![], multiplier: 2.0 },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: SilentShadow,
            name: "Silent Shadow",
            description: "Move through the forest like a ghost",
            tier: 3,
            faction: Faction::French,
            prerequisites: vec![ForestWisdom],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::SurvivalDays { days: 30, location: Some("undetected".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::Stealth, modifier: 0.60, condition: None },
                SkillEffect::StatModifier { stat: SkillStat::Speed, modifier: 1.0, condition: Some(EffectCondition::WhileCrouched) },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: HerbalistsCraft,
            name: "Herbalist's Craft",
            description: "Master the healing arts of the forest",
            tier: 3,
            faction: Faction::French,
            prerequisites: vec![ForestWisdom],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::CraftCount { count: 50, item_type: Some("medicine".into()) },
            effects: vec![
                SkillEffect::GatheringBonus { resource: "plants".into(), multiplier: 2.0 },
                SkillEffect::UnlockRecipe { recipe_id: "coureurs_tonic".into() },
                SkillEffect::StatModifier { stat: SkillStat::HealingEffectiveness, modifier: 0.50, condition: None },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: TradeEmpire,
            name: "Trade Empire",
            description: "Build a network of trading posts",
            tier: 4,
            faction: Faction::French,
            prerequisites: vec![MasterTrapper, NegotiatorsTongue],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::WealthAmount { amount: 3000 },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Establish trading posts".into() },
                SkillEffect::FollowerCommand { max_followers: 2 },
                SkillEffect::LocationAccess { location_type: "french_black_market".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: OneWithLand,
            name: "One With the Land",
            description: "The forest itself sustains you",
            tier: 4,
            faction: Faction::French,
            prerequisites: vec![SilentShadow, HerbalistsCraft],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::SurvivalDays { days: 120, location: None },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::Health, modifier: 0.01, condition: Some(EffectCondition::InForest) },
                SkillEffect::CompanionUnlock { companion_type: "wild_animal".into(), count: 1 },
                SkillEffect::SpecialPassive { description: "Immune to poison".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: SpiritBridge,
            name: "Spirit Bridge",
            description: "You are a bridge between Native and European worlds",
            tier: 5,
            faction: Faction::French,
            prerequisites: vec![TradeEmpire, OneWithLand],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "allied_3_native".into() },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Participate in Native ceremonies".into() },
                SkillEffect::SpecialPassive { description: "Learn one skill from any Native tree".into() },
                SkillEffect::ReputationModifier {
                    factions: vec![Faction::Powhatan, Faction::Tuscarora, Faction::Cherokee, Faction::Catawba, Faction::Pamunkey],
                    multiplier: 1.50
                },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: GrandVoyageur,
            name: "Grand Voyageur",
            description: "You are a legend among the coureurs des bois",
            tier: 6,
            faction: Faction::French,
            prerequisites: vec![SpiritBridge],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["northwest_passage".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "Grand Voyageur".into() },
                SkillEffect::UnlockWeapon { weapon_id: "coureurs_capote".into() },
                SkillEffect::UnlockAbility { ability_id: "spirit_walk".into() },
                SkillEffect::SpecialPassive { description: "All waterways revealed on map".into() },
                SkillEffect::SpecialPassive { description: "Fast travel between water locations".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

fn get_english_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: RoanokeSettler,
            name: "Roanoke Settler",
            description: "Begin as a colonist of the Roanoke settlement",
            tier: 1,
            faction: Faction::English,
            prerequisites: vec![],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::LocationAccess { location_type: "colonial_storage".into() },
                SkillEffect::SpecialPassive { description: "Request basic supplies from ships".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: Fortification,
            name: "Fortification",
            description: "Learn to build defensive structures",
            tier: 2,
            faction: Faction::English,
            prerequisites: vec![RoanokeSettler],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::CraftCount { count: 10, item_type: Some("structure".into()) },
            effects: vec![
                SkillEffect::UnlockRecipe { recipe_id: "palisade".into() },
                SkillEffect::StatModifier { stat: SkillStat::CraftingQuality, modifier: 0.25, condition: None },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: FrontierSurvival,
            name: "Frontier Survival",
            description: "Learn to survive in the wilderness",
            tier: 2,
            faction: Faction::English,
            prerequisites: vec![RoanokeSettler],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::SurvivalDays { days: 5, location: Some("wilderness".into()) },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Basic hunting unlocked".into() },
                SkillEffect::SpecialPassive { description: "Water purification with fire".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: MasterBuilder,
            name: "Master Builder",
            description: "Construct advanced colonial structures",
            tier: 3,
            faction: Faction::English,
            prerequisites: vec![Fortification],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::CraftCount { count: 1, item_type: Some("home".into()) },
            effects: vec![
                SkillEffect::UnlockRecipe { recipe_id: "stone_walls".into() },
                SkillEffect::UnlockRecipe { recipe_id: "watchtower".into() },
                SkillEffect::SpecialPassive { description: "Structures provide comfort bonus".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: MilitiaCaptain,
            name: "Militia Captain",
            description: "Lead colonial militia in defense",
            tier: 3,
            faction: Faction::English,
            prerequisites: vec![Fortification],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::DefenseCount { count: 3 },
            effects: vec![
                SkillEffect::FollowerCommand { max_followers: 6 },
                SkillEffect::StatModifier { stat: SkillStat::RangedDamage, modifier: 0.20, condition: Some(EffectCondition::NearAllies) },
                SkillEffect::UnlockAbility { ability_id: "rally".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: WildernessScout,
            name: "Wilderness Scout",
            description: "Explore and map the frontier",
            tier: 3,
            faction: Faction::English,
            prerequisites: vec![FrontierSurvival],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::DiscoveryCount { count: 20 },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::DetectionRange, modifier: 0.50, condition: None },
                SkillEffect::SpecialPassive { description: "Mark locations for others".into() },
                SkillEffect::SpecialPassive { description: "Fast travel to outposts".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: ColonialFarmer,
            name: "Colonial Farmer",
            description: "Master European agricultural techniques",
            tier: 3,
            faction: Faction::English,
            prerequisites: vec![FrontierSurvival],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::CraftCount { count: 100, item_type: Some("crop".into()) },
            effects: vec![
                SkillEffect::GatheringBonus { resource: "crops".into(), multiplier: 1.50 },
                SkillEffect::UnlockRecipe { recipe_id: "european_crops".into() },
                SkillEffect::SpecialPassive { description: "Livestock breeding unlocked".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: ColonialLeader,
            name: "Colonial Leader",
            description: "Become a leader of the settlement",
            tier: 4,
            faction: Faction::English,
            prerequisites: vec![MasterBuilder, MilitiaCaptain],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "population_20".into() },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Assign NPC jobs and schedules".into() },
                SkillEffect::SpecialPassive { description: "Settlement generates passive resources".into() },
                SkillEffect::SpecialPassive { description: "Establish trade agreements".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: FrontierMaster,
            name: "Frontier Master",
            description: "The wilderness holds no secrets from you",
            tier: 4,
            faction: Faction::English,
            prerequisites: vec![WildernessScout, ColonialFarmer],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::DiscoveryCount { count: 3 },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::Speed, modifier: 0.50, condition: None },
                SkillEffect::SpecialPassive { description: "Establish remote outposts".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: NewWorldGovernor,
            name: "New World Governor",
            description: "Govern multiple colonial settlements",
            tier: 5,
            faction: Faction::English,
            prerequisites: vec![ColonialLeader, FrontierMaster],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "control_3_settlements".into() },
            effects: vec![
                SkillEffect::TitleGrant { title: "Governor".into() },
                SkillEffect::SpecialPassive { description: "Negotiate treaties with Natives".into() },
                SkillEffect::SpecialPassive { description: "Monthly supply ships include rare items".into() },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: LordOfRoanoke,
            name: "Lord of Roanoke",
            description: "You are the undisputed leader of the colony",
            tier: 6,
            faction: Faction::English,
            prerequisites: vec![NewWorldGovernor],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["lost_colony".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "Lord of Roanoke".into() },
                SkillEffect::UnlockRecipe { recipe_id: "governors_mansion".into() },
                SkillEffect::SpecialPassive { description: "Declare war or peace with any faction".into() },
                SkillEffect::SpecialPassive { description: "Settlers arrive monthly".into() },
                SkillEffect::LocationAccess { location_type: "crown_armory".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

fn get_aztec_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: MacehuallinInitiate,
            name: "Macehualtin Initiate",
            description: "Prove yourself worthy to join the Aztec warrior tradition",
            tier: 1,
            faction: Faction::Aztec,
            prerequisites: vec![],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::KillCount { count: 20, target: Some("spanish".into()) },
            effects: vec![
                SkillEffect::LocationAccess { location_type: "aztec_armory".into() },
                SkillEffect::SpecialPassive { description: "Learn Nahuatl language".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: JaguarWarrior,
            name: "Jaguar Warrior",
            description: "Join the elite Jaguar warrior society",
            tier: 2,
            faction: Faction::Aztec,
            prerequisites: vec![MacehuallinInitiate],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::CaptureCount { count: 4 },
            effects: vec![
                SkillEffect::UnlockWeapon { weapon_id: "jaguar_armor".into() },
                SkillEffect::UnlockAbility { ability_id: "jaguar_pounce".into() },
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.30, condition: Some(EffectCondition::AtNight) },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: EagleWarrior,
            name: "Eagle Warrior",
            description: "Join the elite Eagle warrior society",
            tier: 2,
            faction: Faction::Aztec,
            prerequisites: vec![MacehuallinInitiate],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 4, target: Some("unharmed".into()) },
            effects: vec![
                SkillEffect::UnlockWeapon { weapon_id: "eagle_armor".into() },
                SkillEffect::UnlockAbility { ability_id: "eagle_dive".into() },
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.30, condition: Some(EffectCondition::AtDay) },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: OcelotlFury,
            name: "Ocelotl Fury",
            description: "Embrace the primal fury of the jaguar",
            tier: 3,
            faction: Faction::Aztec,
            prerequisites: vec![JaguarWarrior],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::KillCount { count: 30, target: Some("stealth".into()) },
            effects: vec![
                SkillEffect::UnlockAbility { ability_id: "jaguar_rage".into() },
                SkillEffect::StatModifier { stat: SkillStat::Health, modifier: 0.20, condition: None },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: ShadowStalker,
            name: "Shadow Stalker",
            description: "Hunt your prey through shadows",
            tier: 3,
            faction: Faction::Aztec,
            prerequisites: vec![JaguarWarrior],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 10, target: Some("fleeing".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::TrackingRange, modifier: 0.50, condition: None },
                SkillEffect::StatModifier { stat: SkillStat::Speed, modifier: 0.20, condition: None },
                SkillEffect::SpecialPassive { description: "Wounded targets cannot sprint".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: CuauhtliStrike,
            name: "Cuauhtli Strike",
            description: "Strike from above like the eagle",
            tier: 3,
            faction: Faction::Aztec,
            prerequisites: vec![EagleWarrior],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 10, target: Some("aerial".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::RangedDamage, modifier: 0.40, condition: None },
                SkillEffect::SpecialPassive { description: "Diving attacks stun for 2s".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: SolarAscension,
            name: "Solar Ascension",
            description: "Draw power from the sun itself",
            tier: 3,
            faction: Faction::Aztec,
            prerequisites: vec![EagleWarrior],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::KillCount { count: 20, target: Some("golden_hour".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.15, condition: Some(EffectCondition::AtDay) },
                SkillEffect::SpecialPassive { description: "Immune to fire damage".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: JaguarKnight,
            name: "Jaguar Knight",
            description: "Become a knight of the Jaguar order",
            tier: 4,
            faction: Faction::Aztec,
            prerequisites: vec![OcelotlFury, ShadowStalker],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::CaptureCount { count: 1 },
            effects: vec![
                SkillEffect::TitleGrant { title: "Jaguar Knight".into() },
                SkillEffect::CompanionUnlock { companion_type: "jaguar".into(), count: 3 },
                SkillEffect::UnlockAbility { ability_id: "call_of_the_hunt".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: EagleKnight,
            name: "Eagle Knight",
            description: "Become a knight of the Eagle order",
            tier: 4,
            faction: Faction::Aztec,
            prerequisites: vec![CuauhtliStrike, SolarAscension],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::KillCount { count: 1, target: Some("atlatl_80m".into()) },
            effects: vec![
                SkillEffect::TitleGrant { title: "Eagle Knight".into() },
                SkillEffect::CompanionUnlock { companion_type: "eagle".into(), count: 1 },
                SkillEffect::UnlockAbility { ability_id: "eagles_cry".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: Cuachicqueh,
            name: "Cuachicqueh",
            description: "Become one of the Shorn Ones, the most elite warriors",
            tier: 5,
            faction: Faction::Aztec,
            prerequisites: vec![JaguarKnight, EagleKnight],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::RitualCombatWins { count: 50 },
            effects: vec![
                SkillEffect::TitleGrant { title: "Cuachicqueh".into() },
                SkillEffect::SpecialPassive { description: "Cannot retreat from combat".into() },
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.50, condition: Some(EffectCondition::BelowHealthPercent(0.30)) },
                SkillEffect::FollowerCommand { max_followers: 10 },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: ChampionOfTheSun,
            name: "Champion of the Sun",
            description: "You are the chosen champion of Huitzilopochtli",
            tier: 6,
            faction: Faction::Aztec,
            prerequisites: vec![Cuachicqueh],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["fifth_sun".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "Champion of the Sun".into() },
                SkillEffect::UnlockWeapon { weapon_id: "macuahuitl_quetzalcoatl".into() },
                SkillEffect::UnlockAbility { ability_id: "wrath_of_huitzilopochtli".into() },
                SkillEffect::SpecialPassive { description: "Sacrifice enemies for powerful buffs".into() },
                SkillEffect::SpecialPassive { description: "Lead Aztec hidden city".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

fn get_powhatan_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: PowhatanNewcomer,
            name: "Newcomer",
            description: "Gain acceptance in Powhatan territory",
            tier: 1,
            faction: Faction::Powhatan,
            prerequisites: vec![],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::LocationAccess { location_type: "powhatan_village".into() },
                SkillEffect::SpecialPassive { description: "Basic Algonquian phrases".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: HuntersPath,
            name: "Hunter's Path",
            description: "Learn the hunting ways of the Powhatan",
            tier: 2,
            faction: Faction::Powhatan,
            prerequisites: vec![PowhatanNewcomer],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::TradeCount { count: 20 },
            effects: vec![
                SkillEffect::LocationAccess { location_type: "hunting_grounds".into() },
                SkillEffect::UnlockRecipe { recipe_id: "powhatan_arrows".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: DiplomatsPath,
            name: "Diplomat's Path",
            description: "Learn the political ways of the confederacy",
            tier: 2,
            faction: Faction::Powhatan,
            prerequisites: vec![PowhatanNewcomer],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["diplomatic_5".into()] },
            effects: vec![
                SkillEffect::LocationAccess { location_type: "chiefs_longhouse".into() },
                SkillEffect::SpecialPassive { description: "Political dialogue options".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: DeerStalker,
            name: "Deer Stalker",
            description: "Master hunting deer in confederacy lands",
            tier: 3,
            faction: Faction::Powhatan,
            prerequisites: vec![HuntersPath],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 50, target: Some("deer".into()) },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Deer visible through vegetation".into() },
                SkillEffect::StatModifier { stat: SkillStat::Stealth, modifier: 1.0, condition: Some(EffectCondition::InForest) },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: RiverKeeper,
            name: "River Keeper",
            description: "Master the tidewater ways",
            tier: 3,
            faction: Faction::Powhatan,
            prerequisites: vec![HuntersPath],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::TradeCount { count: 100 },
            effects: vec![
                SkillEffect::UnlockRecipe { recipe_id: "fish_trap".into() },
                SkillEffect::SpecialPassive { description: "Navigate rivers at night safely".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: PeaceWeaver,
            name: "Peace Weaver",
            description: "Prevent conflicts through diplomacy",
            tier: 3,
            faction: Faction::Powhatan,
            prerequisites: vec![DiplomatsPath],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "prevent_3_conflicts".into() },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Arrange alliance marriages".into() },
                SkillEffect::ReputationModifier { factions: vec![], multiplier: 0.50 },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: WarChief,
            name: "War Chief",
            description: "Lead Powhatan warriors in battle",
            tier: 3,
            faction: Faction::Powhatan,
            prerequisites: vec![DiplomatsPath],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::DefenseCount { count: 5 },
            effects: vec![
                SkillEffect::FollowerCommand { max_followers: 10 },
                SkillEffect::UnlockAbility { ability_id: "war_paint".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: SpiritWalker,
            name: "Spirit Walker",
            description: "Commune with the spirits of the land",
            tier: 4,
            faction: Faction::Powhatan,
            prerequisites: vec![DeerStalker, RiverKeeper],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::VisionQuest,
            effects: vec![
                SkillEffect::SpecialPassive { description: "Animal spirits warn of danger".into() },
                SkillEffect::UnlockAbility { ability_id: "spirit_guide".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: ConfederateLord,
            name: "Confederate Lord",
            description: "Gain influence over multiple tribes",
            tier: 4,
            faction: Faction::Powhatan,
            prerequisites: vec![PeaceWeaver, WarChief],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "unite_5_tribes".into() },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Weekly tribute from allied tribes".into() },
                SkillEffect::SpecialPassive { description: "Veto power over confederacy decisions".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: Werowance,
            name: "Werowance",
            description: "Become a tribal chief",
            tier: 5,
            faction: Faction::Powhatan,
            prerequisites: vec![SpiritWalker, ConfederateLord],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::TitleGrant { title: "Werowance".into() },
                SkillEffect::SpecialPassive { description: "Personal village with 30 inhabitants".into() },
                SkillEffect::SpecialPassive { description: "Perform sacred ceremonies".into() },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: Mamanatowick,
            name: "Mamanatowick",
            description: "Become the paramount chief of the confederacy",
            tier: 6,
            faction: Faction::Powhatan,
            prerequisites: vec![Werowance],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["unification".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "Mamanatowick".into() },
                SkillEffect::UnlockWeapon { weapon_id: "powhatans_war_club".into() },
                SkillEffect::UnlockAbility { ability_id: "voice_of_the_land".into() },
                SkillEffect::SpecialPassive { description: "Negotiate with English as equal sovereign".into() },
                SkillEffect::SpecialPassive { description: "Found new tribes in unclaimed territory".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

// Stub implementations for remaining factions - following same pattern
fn get_tuscarora_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: FriendOfTuscarora,
            name: "Friend of Tuscarora",
            description: "Gain acceptance among the Tuscarora people",
            tier: 1,
            faction: Faction::Tuscarora,
            prerequisites: vec![],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::LocationAccess { location_type: "tuscarora_village".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: ClanWarrior,
            name: "Clan Warrior",
            description: "Join a warrior clan of the Tuscarora",
            tier: 2,
            faction: Faction::Tuscarora,
            prerequisites: vec![FriendOfTuscarora],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::RitualCombatWins { count: 1 },
            effects: vec![
                SkillEffect::LocationAccess { location_type: "clan_markings".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: ClanProvider,
            name: "Clan Provider",
            description: "Support your clan through agriculture",
            tier: 2,
            faction: Faction::Tuscarora,
            prerequisites: vec![FriendOfTuscarora],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::TradeCount { count: 50 },
            effects: vec![
                SkillEffect::UnlockRecipe { recipe_id: "three_sisters_seeds".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: BearClanFury,
            name: "Bear Clan Fury",
            description: "Join the Bear Clan and gain their strength",
            tier: 3,
            faction: Faction::Tuscarora,
            prerequisites: vec![ClanWarrior],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::KillCount { count: 1, target: Some("bear_solo".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.25, condition: Some(EffectCondition::BelowHealthPercent(0.50)) },
                SkillEffect::SpecialPassive { description: "Cannot be knocked down".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: WolfClanPack,
            name: "Wolf Clan Pack",
            description: "Join the Wolf Clan and hunt with wolves",
            tier: 3,
            faction: Faction::Tuscarora,
            prerequisites: vec![ClanWarrior],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "hunt_with_wolves_10".into() },
            effects: vec![
                SkillEffect::CompanionUnlock { companion_type: "wolf".into(), count: 3 },
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.15, condition: Some(EffectCondition::NearAllies) },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: TurtleClanWisdom,
            name: "Turtle Clan Wisdom",
            description: "Join the Turtle Clan and learn ancient crafts",
            tier: 3,
            faction: Faction::Tuscarora,
            prerequisites: vec![ClanProvider],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::CraftCount { count: 20, item_type: None },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::CraftingQuality, modifier: 0.30, condition: None },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: DeerClanGrace,
            name: "Deer Clan Grace",
            description: "Join the Deer Clan and gain their swiftness",
            tier: 3,
            faction: Faction::Tuscarora,
            prerequisites: vec![ClanProvider],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::SurvivalDays { days: 30, location: Some("no_deer_hunt".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::Speed, modifier: 0.25, condition: None },
                SkillEffect::SpecialPassive { description: "Deer will not flee from you".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: TuscaroraWarCaptain,
            name: "War Captain",
            description: "Lead Tuscarora war parties",
            tier: 4,
            faction: Faction::Tuscarora,
            prerequisites: vec![BearClanFury, WolfClanPack],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::DefenseCount { count: 3 },
            effects: vec![
                SkillEffect::FollowerCommand { max_followers: 15 },
                SkillEffect::UnlockAbility { ability_id: "war_drum".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: ClanMother,
            name: "Clan Mother",
            description: "Become a Clan Mother with civil authority",
            tier: 4,
            faction: Faction::Tuscarora,
            prerequisites: vec![TurtleClanWisdom, DeerClanGrace],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::SurvivalDays { days: 365, location: None },
            effects: vec![
                SkillEffect::TitleGrant { title: "Clan Mother".into() },
                SkillEffect::SpecialPassive { description: "Select and depose war chiefs".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: PeaceChief,
            name: "Peace Chief",
            description: "Become a Peace Chief of the Tuscarora",
            tier: 5,
            faction: Faction::Tuscarora,
            prerequisites: vec![TuscaroraWarCaptain, ClanMother],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "negotiate_peace_2".into() },
            effects: vec![
                SkillEffect::TitleGrant { title: "Peace Chief".into() },
                SkillEffect::SpecialPassive { description: "Diplomatic immunity in all villages".into() },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: KeeperOfTheFire,
            name: "Keeper of the Fire",
            description: "Become the supreme civil authority of the Tuscarora",
            tier: 6,
            faction: Faction::Tuscarora,
            prerequisites: vec![PeaceChief],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["joining_of_nations".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "Keeper of the Fire".into() },
                SkillEffect::SpecialPassive { description: "Tuscarora join Iroquois Confederacy".into() },
                SkillEffect::UnlockAbility { ability_id: "great_law".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

fn get_cherokee_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: CherokeeFriend,
            name: "Cherokee Friend",
            description: "Gain acceptance among the Cherokee people",
            tier: 1,
            faction: Faction::Cherokee,
            prerequisites: vec![],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::LocationAccess { location_type: "cherokee_town".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: RedWarPath,
            name: "Red War Path",
            description: "Walk the path of war",
            tier: 2,
            faction: Faction::Cherokee,
            prerequisites: vec![CherokeeFriend],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 10, target: None },
            effects: vec![
                SkillEffect::UnlockWeapon { weapon_id: "red_war_paint".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: WhitePeacePath,
            name: "White Peace Path",
            description: "Walk the path of peace and healing",
            tier: 2,
            faction: Faction::Cherokee,
            prerequisites: vec![CherokeeFriend],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["diplomatic".into()] },
            effects: vec![
                SkillEffect::LocationAccess { location_type: "council_meeting".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: RavenMocker,
            name: "Raven Mocker",
            description: "Become a feared supernatural warrior",
            tier: 3,
            faction: Faction::Cherokee,
            prerequisites: vec![RedWarPath],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::KillCount { count: 20, target: Some("scalp".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::Health, modifier: 5.0, condition: None },
                SkillEffect::SpecialPassive { description: "Enemies have reduced morale".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: CherokeeWarPriest,
            name: "War Priest",
            description: "Perform battle rituals for your warriors",
            tier: 3,
            faction: Faction::Cherokee,
            prerequisites: vec![RedWarPath],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::SurvivalDays { days: 7, location: Some("fasting".into()) },
            effects: vec![
                SkillEffect::UnlockAbility { ability_id: "battle_ritual".into() },
                SkillEffect::UnlockAbility { ability_id: "enemy_curse".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: MedicineWalker,
            name: "Medicine Walker",
            description: "Master Cherokee herbal medicine",
            tier: 3,
            faction: Faction::Cherokee,
            prerequisites: vec![WhitePeacePath],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::CraftCount { count: 30, item_type: Some("medicine".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::HealingEffectiveness, modifier: 1.0, condition: None },
                SkillEffect::SpecialPassive { description: "Cure diseases".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: BelovedElder,
            name: "Beloved Elder",
            description: "Gain the wisdom and respect of an elder",
            tier: 3,
            faction: Faction::Cherokee,
            prerequisites: vec![WhitePeacePath],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "age_50_or_wisdom".into() },
            effects: vec![
                SkillEffect::TitleGrant { title: "Beloved Elder".into() },
                SkillEffect::SpecialPassive { description: "Grant sanctuary to fugitives".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: RedWarChief,
            name: "Red War Chief",
            description: "Become a war chief of the Cherokee",
            tier: 4,
            faction: Faction::Cherokee,
            prerequisites: vec![RavenMocker, CherokeeWarPriest],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::DefenseCount { count: 5 },
            effects: vec![
                SkillEffect::TitleGrant { title: "Red War Chief".into() },
                SkillEffect::FollowerCommand { max_followers: 20 },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: WhitePeaceChief,
            name: "White Peace Chief",
            description: "Become a peace chief with civil authority",
            tier: 4,
            faction: Faction::Cherokee,
            prerequisites: vec![MedicineWalker, BelovedElder],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::SurvivalDays { days: 365, location: Some("peace".into()) },
            effects: vec![
                SkillEffect::TitleGrant { title: "White Peace Chief".into() },
                SkillEffect::SpecialPassive { description: "Veto war declarations".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: FirstBelovedMan,
            name: "First Beloved Man",
            description: "Unite the Red and White paths",
            tier: 5,
            faction: Faction::Cherokee,
            prerequisites: vec![RedWarChief, WhitePeaceChief],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "unite_paths".into() },
            effects: vec![
                SkillEffect::TitleGrant { title: "First Beloved Man".into() },
                SkillEffect::SpecialPassive { description: "Authority in all Cherokee towns".into() },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: UkuOfCherokee,
            name: "Uku of the Cherokee",
            description: "Become the high priest-chief of the Cherokee nation",
            tier: 6,
            faction: Faction::Cherokee,
            prerequisites: vec![FirstBelovedMan],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["eternal_flame".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "Uku".into() },
                SkillEffect::UnlockAbility { ability_id: "voice_of_ancestors".into() },
                SkillEffect::SpecialPassive { description: "Keeper of the Eternal Flame".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

fn get_catawba_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: CatawbaAcquaintance,
            name: "Catawba Acquaintance",
            description: "Make contact with the Catawba people",
            tier: 1,
            faction: Faction::Catawba,
            prerequisites: vec![],
            required_standing: Standing::Neutral,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::LocationAccess { location_type: "catawba_village".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: RiverFighter,
            name: "River Fighter",
            description: "Learn to fight on and near the water",
            tier: 2,
            faction: Faction::Catawba,
            prerequisites: vec![CatawbaAcquaintance],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 5, target: Some("water".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.20, condition: Some(EffectCondition::NearWater) },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: CatawbaTradeMaster,
            name: "Trade Master",
            description: "Learn the Catawba trade networks",
            tier: 2,
            faction: Faction::Catawba,
            prerequisites: vec![CatawbaAcquaintance],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::TradeCount { count: 10 },
            effects: vec![
                SkillEffect::TradeBonus { multiplier: 1.15 },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: RaidLeader,
            name: "Raid Leader",
            description: "Lead Catawba raiding parties",
            tier: 3,
            faction: Faction::Catawba,
            prerequisites: vec![RiverFighter],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::DefenseCount { count: 1 },
            effects: vec![
                SkillEffect::FollowerCommand { max_followers: 8 },
                SkillEffect::SpecialPassive { description: "Captive taking unlocked".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: WaterAmbush,
            name: "Water Ambush",
            description: "Master ambush tactics from the water",
            tier: 3,
            faction: Faction::Catawba,
            prerequisites: vec![RiverFighter],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::KillCount { count: 15, target: Some("water_ambush".into()) },
            effects: vec![
                SkillEffect::StatModifier { stat: SkillStat::MeleeDamage, modifier: 0.75, condition: Some(EffectCondition::NearWater) },
                SkillEffect::SpecialPassive { description: "Hide underwater with reed".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: PotteryArtisan,
            name: "Pottery Artisan",
            description: "Master the famous Catawba pottery",
            tier: 3,
            faction: Faction::Catawba,
            prerequisites: vec![CatawbaTradeMaster],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::CraftCount { count: 50, item_type: Some("pottery".into()) },
            effects: vec![
                SkillEffect::TradeBonus { multiplier: 3.0 },
                SkillEffect::UnlockRecipe { recipe_id: "master_pottery".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: MarketManipulator,
            name: "Market Manipulator",
            description: "Control trade in the region",
            tier: 3,
            faction: Faction::Catawba,
            prerequisites: vec![CatawbaTradeMaster],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "control_trade".into() },
            effects: vec![
                SkillEffect::SpecialPassive { description: "Set prices at Catawba markets".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: RiverHawk,
            name: "River Hawk",
            description: "Dominate the rivers as a feared raider",
            tier: 4,
            faction: Faction::Catawba,
            prerequisites: vec![RaidLeader, WaterAmbush],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::SurvivalDays { days: 365, location: Some("river".into()) },
            effects: vec![
                SkillEffect::TitleGrant { title: "River Hawk".into() },
                SkillEffect::StatModifier { stat: SkillStat::Speed, modifier: 0.50, condition: Some(EffectCondition::NearWater) },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: TradeLord,
            name: "Trade Lord",
            description: "Build a trade empire",
            tier: 4,
            faction: Faction::Catawba,
            prerequisites: vec![PotteryArtisan, MarketManipulator],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "trade_posts_3".into() },
            effects: vec![
                SkillEffect::TitleGrant { title: "Trade Lord".into() },
                SkillEffect::SpecialPassive { description: "Passive income from trade routes".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: EsawChief,
            name: "Esaw Chief",
            description: "Become a chief of a Catawba town",
            tier: 5,
            faction: Faction::Catawba,
            prerequisites: vec![RiverHawk, TradeLord],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::TitleGrant { title: "Esaw Chief".into() },
                SkillEffect::SpecialPassive { description: "Control a Catawba town".into() },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: KingOfCatawba,
            name: "King of the Catawba",
            description: "Become the paramount ruler of all Catawba",
            tier: 6,
            faction: Faction::Catawba,
            prerequisites: vec![EsawChief],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["river_empire".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "King of the Catawba".into() },
                SkillEffect::UnlockWeapon { weapon_id: "river_kings_mace".into() },
                SkillEffect::UnlockAbility { ability_id: "flood_the_land".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

fn get_pamunkey_skills() -> Vec<FactionSkill> {
    use FactionSkillId::*;
    vec![
        FactionSkill {
            id: PamunkeyAccepted,
            name: "Pamunkey Accepted",
            description: "Gain acceptance among the royal Pamunkey tribe",
            tier: 1,
            faction: Faction::Pamunkey,
            prerequisites: vec![],
            required_standing: Standing::Friendly,
            unlock_condition: UnlockCondition::None,
            effects: vec![
                SkillEffect::LocationAccess { location_type: "pamunkey_village".into() },
            ],
            skill_point_cost: 1,
        },
        FactionSkill {
            id: RoyalTradition,
            name: "Royal Tradition",
            description: "Learn the royal protocols",
            tier: 2,
            faction: Faction::Pamunkey,
            prerequisites: vec![PamunkeyAccepted],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["royal_ceremony".into()] },
            effects: vec![
                SkillEffect::UnlockWeapon { weapon_id: "royal_regalia".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: SacredKeeper,
            name: "Sacred Keeper",
            description: "Learn the sacred knowledge of the confederacy",
            tier: 2,
            faction: Faction::Pamunkey,
            prerequisites: vec![PamunkeyAccepted],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::VisionQuest,
            effects: vec![
                SkillEffect::LocationAccess { location_type: "temple".into() },
            ],
            skill_point_cost: 2,
        },
        FactionSkill {
            id: LineageHeir,
            name: "Lineage Heir",
            description: "Be recognized as a potential paramount",
            tier: 3,
            faction: Faction::Pamunkey,
            prerequisites: vec![RoyalTradition],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["succession_trials".into()] },
            effects: vec![
                SkillEffect::FollowerCommand { max_followers: 2 },
                SkillEffect::SpecialPassive { description: "Tribute collection rights".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: CornLord,
            name: "Corn Lord",
            description: "Control food distribution for the confederacy",
            tier: 3,
            faction: Faction::Pamunkey,
            prerequisites: vec![RoyalTradition],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::SurvivalDays { days: 90, location: Some("food_surplus".into()) },
            effects: vec![
                SkillEffect::GatheringBonus { resource: "crops".into(), multiplier: 1.50 },
                SkillEffect::SpecialPassive { description: "Famine immunity for settlements".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: TempleGuardian,
            name: "Temple Guardian",
            description: "Protect the sacred temples",
            tier: 3,
            faction: Faction::Pamunkey,
            prerequisites: vec![SacredKeeper],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::DefenseCount { count: 1 },
            effects: vec![
                SkillEffect::LocationAccess { location_type: "sacred_weapons".into() },
                SkillEffect::StatModifier { stat: SkillStat::Health, modifier: 5.0, condition: Some(EffectCondition::NearSacredSite) },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: HistoryKeeper,
            name: "History Keeper",
            description: "Learn the complete history of the confederacy",
            tier: 3,
            faction: Faction::Pamunkey,
            prerequisites: vec![SacredKeeper],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "learn_history".into() },
            effects: vec![
                SkillEffect::SpecialPassive { description: "All confederacy locations known".into() },
            ],
            skill_point_cost: 3,
        },
        FactionSkill {
            id: RoyalBlood,
            name: "Royal Blood",
            description: "Be recognized as having royal blood",
            tier: 4,
            faction: Faction::Pamunkey,
            prerequisites: vec![LineageHeir, CornLord],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "paramount_recognition".into() },
            effects: vec![
                SkillEffect::TitleGrant { title: "Royal Blood".into() },
                SkillEffect::FollowerCommand { max_followers: 10 },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: SacredWisdom,
            name: "Sacred Wisdom",
            description: "Complete all sacred rituals",
            tier: 4,
            faction: Faction::Pamunkey,
            prerequisites: vec![TempleGuardian, HistoryKeeper],
            required_standing: Standing::Allied,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["all_rituals".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "Sacred Wisdom".into() },
                SkillEffect::SpecialPassive { description: "Perform ceremonies".into() },
            ],
            skill_point_cost: 5,
        },
        FactionSkill {
            id: ParamountHeir,
            name: "Paramount Heir",
            description: "Be recognized as heir to the paramount chief",
            tier: 5,
            faction: Faction::Pamunkey,
            prerequisites: vec![RoyalBlood, SacredWisdom],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::Achievement { achievement_id: "paramount_endorsement".into() },
            effects: vec![
                SkillEffect::TitleGrant { title: "Paramount Heir".into() },
                SkillEffect::FollowerCommand { max_followers: 20 },
            ],
            skill_point_cost: 8,
        },
        FactionSkill {
            id: BloodOfPowhatan,
            name: "Blood of Powhatan",
            description: "Become the new Mamanatowick",
            tier: 6,
            faction: Faction::Pamunkey,
            prerequisites: vec![ParamountHeir],
            required_standing: Standing::BloodBond,
            unlock_condition: UnlockCondition::QuestComplete { quest_ids: vec!["succession".into()] },
            effects: vec![
                SkillEffect::TitleGrant { title: "Blood of Powhatan".into() },
                SkillEffect::UnlockWeapon { weapon_id: "feathered_crown".into() },
                SkillEffect::UnlockAbility { ability_id: "unite_the_people".into() },
                SkillEffect::SpecialPassive { description: "Supreme authority over confederacy".into() },
            ],
            skill_point_cost: 15,
        },
    ]
}

// ============================================================================
// PLAYER FACTION SKILLS STATE
// ============================================================================

/// Player's unlocked faction skills
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerFactionSkills {
    /// Unlocked skills per faction
    pub unlocked: HashMap<Faction, HashSet<FactionSkillId>>,
    /// Skill points available per faction
    pub skill_points: HashMap<Faction, u32>,
    /// Currently active faction (for primary bonuses)
    pub primary_faction: Option<Faction>,
}

impl PlayerFactionSkills {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a skill is unlocked
    pub fn has_skill(&self, skill_id: FactionSkillId) -> bool {
        let faction = skill_id.faction();
        self.unlocked
            .get(&faction)
            .map(|skills| skills.contains(&skill_id))
            .unwrap_or(false)
    }

    /// Check if player can unlock a skill
    pub fn can_unlock(&self, skill: &FactionSkill, standing: Standing) -> bool {
        // Check standing requirement
        if standing < skill.required_standing {
            return false;
        }

        // Check prerequisites
        for prereq in &skill.prerequisites {
            if !self.has_skill(*prereq) {
                return false;
            }
        }

        // Check skill points
        let points = self.skill_points.get(&skill.faction).copied().unwrap_or(0);
        if points < skill.skill_point_cost {
            return false;
        }

        // Check not already unlocked
        !self.has_skill(skill.id)
    }

    /// Unlock a skill (assumes can_unlock returned true)
    pub fn unlock_skill(&mut self, skill: &FactionSkill) -> bool {
        if !self.can_unlock(skill, skill.required_standing) {
            return false;
        }

        // Deduct points
        if let Some(points) = self.skill_points.get_mut(&skill.faction) {
            *points -= skill.skill_point_cost;
        }

        // Add to unlocked
        self.unlocked
            .entry(skill.faction)
            .or_default()
            .insert(skill.id);

        true
    }

    /// Add skill points for a faction
    pub fn add_points(&mut self, faction: Faction, amount: u32) {
        *self.skill_points.entry(faction).or_default() += amount;
    }

    /// Get total unlocked skills for a faction
    pub fn skill_count(&self, faction: Faction) -> usize {
        self.unlocked.get(&faction).map(|s| s.len()).unwrap_or(0)
    }

    /// Get highest tier unlocked for a faction
    pub fn highest_tier(&self, faction: Faction) -> u8 {
        self.unlocked
            .get(&faction)
            .map(|skills| skills.iter().map(|s| s.tier()).max().unwrap_or(0))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_tier() {
        assert_eq!(FactionSkillId::ConquistadorInitiate.tier(), 1);
        assert_eq!(FactionSkillId::ElConquistador.tier(), 6);
        assert_eq!(FactionSkillId::GoldAndGlory.tier(), 5);
    }

    #[test]
    fn test_skill_faction() {
        assert_eq!(FactionSkillId::ConquistadorInitiate.faction(), Faction::Spanish);
        assert_eq!(FactionSkillId::ApprentiVoyageur.faction(), Faction::French);
        assert_eq!(FactionSkillId::Mamanatowick.faction(), Faction::Powhatan);
    }

    #[test]
    fn test_player_skills_unlock() {
        let mut player_skills = PlayerFactionSkills::new();
        player_skills.add_points(Faction::Spanish, 10);

        let skills = get_spanish_skills();
        let initiate = skills.iter().find(|s| s.id == FactionSkillId::ConquistadorInitiate).unwrap();

        assert!(player_skills.can_unlock(initiate, Standing::Neutral));
        assert!(player_skills.unlock_skill(initiate));
        assert!(player_skills.has_skill(FactionSkillId::ConquistadorInitiate));
    }
}
