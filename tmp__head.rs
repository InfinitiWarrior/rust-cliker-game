#![allow(deprecated)]
#![allow(warnings)]
use eframe::egui;
use std::collections::HashMap;
use rand::Rng;
use std::fs;
use serde::Deserialize;
use anyhow::Result;

fn main() -> eframe::Result<()> {
    let save: Savefile = load_json("saves/save1.json")?;
    let recipes: RecipeFile = load_json("data/recipes.json")?;

    println!("Player: {} [{}]", save.player.Charactername, save.player.Title);
    println!("Level: {}, XP: {}\n", save.player.Level, save.player.Experience);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_position([0.0, 0.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Clicker Game",
        options,
        Box::new(|_cc| Ok(Box::new(Clicker::default()))),
    )
}

fn load_json<T: for <'de> Deserialize<'de>>(file_path: &str) -> Result<T> {
    let data = fs::read_to_string(file_path)?;
    let parsed: T = serde_json::from_str(&data)?;
    Ok(parsed)
}

fn safe_subtract(value: &mut u32, amount: u32) -> bool {
    if *value >= amount {
        *value -= amount;
        true
    } else {
        false
    }
}


enum MenuTab {
    Clicking,
    Smithing,
    Upgrades,
    Quests,
    Equipment,
    Achievements,
    Settings,
    StatBreakDown,
}

enum FirstQuest {
    CollectFireRunes,
    CollectPlasmaRunes,
    Complete,
}

enum SecondQuest {
    CollectWaterRunes,
    CollectMistRunes,
    Complete,
}

enum ForgeQuest {
    BuildForge,
    MakeBasicSword,
    MakeBasicRing,
    MakeAlloys,
    MakeAdvancedSword,
    MakeAdvancedRing,
    Complete,
}

enum AlchemyQuest {
    BuildAlchemyLab,
    MakeHealthPotion,
    MakeManaPotion,
    MakeStaminaPotion,
    MakeAdvancedPotions,
    Complete,
}

enum ThirdQuest {
    BuildShop,
    SellBasicItems,
    SellAdvancedItems,
    Complete,
}

enum EnchantingQuest {
    InfuseBasicItems,
    InfuseAdvancedItems,
    UpgradeInfusions,
    Complete,
}

enum EquipmentQuest {
    EquipBasicGear,
    EquipAdvancedGear,
    Complete,
}

enum BuildingStuffProgress {
    ForgeFoundation,
    ForgeAnvil,
    ForgeFireplace,
    ForgeWalls,
    ForgeRoof,
    StoreFront,
    StoreShelves,
    AlchemyBench,
    AlchemyStand,
    Complete,
}

enum AlchemyPotions {
    Health,
    Mana,
    Stamina,
    Strength,
    Agility,
    Intelligence,
    Complete,
}

enum ForgeFireStages {
    Kindling,
    SmallFire,
    MediumFire,
    LargeFire,
    Inferno,
    EternalFlame,
    Hellflame,
    InfernalHellFlame,
    EternalHellFlame,
    Complete,
}

enum AlchemyWaterfallStages {
    Drip,
    Stream,
    Brook,
    River,
    Cascade,
    Waterfall,
    Ocean,
    Sea,
    Tsunami,
    Complete,
}
enum ForgeAlloys {
    Bronze,
    Iron,
    Steel,
    Mithril,
    Adamantite,
    Runite,
    Dragonite,
    Complete,
}
struct Clicker {
    unlocks: Unlocks,
    essence: u32,
    maxEssence: u32,
    coins: u32,
    souls: u32,
    runeConversionCost: u32,
    alloys: HashMap<String, u32>,
    advRunes: HashMap<String, u32>,
    forgableSwords: HashMap<String, u32>,
    forgableRings: HashMap<String, u32>,
    essenceClickAmount: u32,
    runeClickAmount: u32,
    extraForgeItemChance: u32,
    ForgeItemAmount: u32,
    swordLimit: u32,
    ringLimit: u32,
    runeChance: u32,
    runes: HashMap<String, u32>,
    fireQuest: FirstQuest,
    upgradePrices: UpgradePrices,
    BuildingProgress: BuildingStuffProgress,
    Potions: AlchemyPotions,
    FireStages: ForgeFireStages,
    WaterfallStages: AlchemyWaterfallStages,
    current_tab: MenuTab,
    gatherQuest: SecondQuest,
    forgeQuest: ForgeQuest,
    alchemyQuest: AlchemyQuest,
    shopQuest: ThirdQuest,
    enchantingQuest: EnchantingQuest,
    equipmentQuest: EquipmentQuest,
    runeConversionAmount: u32,
    advRuneConversionAmount: u32,
    autoClickInterval: f32, // seconds between auto-clicks
    autoClickTimer: f32,    // accumulates time
    playTime: f32,         // total playtime in seconds
    
}

#[derive(Deserialize, Debug)]
struct Savefile {
    player: Player,
    inventory: Inventory,
    settings: Settings,
    unlocks: Unlocks,
    progress: Progress,
    upgrades: Upgrades,
}
#[derive(Deserialize, Debug)]
struct Player {
    Charactername: String,
    Title: String,
    Level: u32,
    Experience: u32,
}

#[derive(Deserialize, Debug)]
struct Inventory {
    gold: u32,
    essence: u32,
    crystals: HashMap<String, u32>,
}

#[derive(Deserialize, Debug)]
struct Settings {
    colorScheme: String,
}

#[derive(Deserialize, Debug)]
struct Unlocks {
    advancedRunes: bool,
    essenceConversion: bool,
    forgingBasics: bool,
    alchemyBasics: bool,
    alloyForging: bool,
    autoCliking: bool,
}

#[derive(Deserialize, Debug)]
struct Progress {
    totalClicks: u32,
    totalEssenceEarned: u32,
}

