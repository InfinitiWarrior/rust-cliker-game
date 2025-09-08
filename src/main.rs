#![allow(deprecated)]
#![allow(warnings)]
use eframe::egui;
use std::collections::HashMap;
use rand::Rng;

fn main() -> eframe::Result<()> {
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

enum MenuTab {
    Clicking,
    Smithing,
    Upgrades,
    Quests,
    Equipment,
    Achievements,
    Settings,
    Collections,
    StatBreakDown,
}

enum FirstQuest {
    CollectFireRunes,
    CollectPlasmaRunes,
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

struct Clicker {
    unlocks: Unlocks,
    essence: u32,
    coins: u32,
    souls: u32,
    advRunes: HashMap<String, u32>,
    essenceAmount: u32,
    runeChance: u32,
    runes: HashMap<String, u32>,
    fireQuest: FirstQuest,
    BuildingProgress: BuildingStuffProgress,
    Alloys: ForgeAlloys,
    Potions: AlchemyPotions,
    FireStages: ForgeFireStages,
    WaterfallStages: AlchemyWaterfallStages,
    current_tab: MenuTab,
}

struct Unlocks {
    advancedRunes: bool,
}

impl Default for Clicker {
    fn default() -> Self {
        let mut runes = HashMap::new();
        for rune in ["Fire", "Water", "Earth", "Air"] {
            runes.insert(format!("{} Rune", rune), 0);
        }
        let mut advRunes = HashMap::new();
        for advRune in ["Plasma", "Mist", "Metal", "Gust"] {
            advRunes.insert(format!("{} Rune", advRune), 0);
        }
        Self {
            essence: 0,
            coins: 0,
            souls: 0,
            essenceAmount: 1,
            runeChance: 50,
            runes,
            advRunes,
            fireQuest: FirstQuest::CollectFireRunes,
            unlocks: Unlocks {
                advancedRunes: false,
            },
            current_tab: MenuTab::Clicking,
        }
    }
}

// Helper function for main action buttons
fn styled_button(label: &str) -> egui::Button {
    egui::Button::new(
        egui::RichText::new(label).color(egui::Color32::BLACK)
    )
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
        .min_size([150.0, 50.0].into())
}

// Helper function for tab buttons (slightly smaller)
fn styled_tab(label: &str) -> egui::Button {
    egui::Button::new(
        egui::RichText::new(label).color(egui::Color32::BLACK)
    )
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
        .min_size([120.0, 40.0].into())
}

impl Clicker {
    fn show_clicking(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Clicking Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Essence: {}", self.essence)).color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Souls: {}", self.souls)).color(egui::Color32::WHITE));
        // Display runes
        ui.horizontal(|ui| {
            for (rune, amount) in &self.runes {
                ui.label(egui::RichText::new(format!("{}: {}", rune, amount)).color(egui::Color32::WHITE));
            }
        });
        // Display advanced runes
        ui.horizontal(|ui| {
            for (rune, amount) in &self.advRunes {
                ui.label(egui::RichText::new(format!("{}: {}", rune, amount)).color(egui::Color32::WHITE));
            }
        });

        // Clicking button
        if ui.add(styled_button("Conjure resources")).clicked() {
            self.essence += self.essenceAmount;

            let mut rng = rand::thread_rng();

            if rng.gen_range(0..100) < self.runeChance {
                let keys: Vec<String> = self.runes.keys().cloned().collect();
                let chosen = &keys[rng.gen_range(0..keys.len())];
                *self.runes.get_mut(chosen.as_str()).unwrap() += 1;
            }
        }
        ui.horizontal(|ui| {
            if let Some(&fire) = self.runes.get("Fire Rune") {
                if ui.add_enabled(fire >= 5 && self.unlocks.advancedRunes == true, styled_button("Make Plasma")).clicked() {
                    *self.runes.get_mut("Fire Rune").unwrap() -= 5;
                    *self.advRunes.get_mut("Plasma Rune").unwrap() += 1;
                }
            }
            if let Some(&air) = self.runes.get("Air Rune") {
                if ui.add_enabled(air >= 5 && self.unlocks.advancedRunes == true, styled_button("Make Gust")).clicked() {
                    *self.runes.get_mut("Air Rune").unwrap() -= 5;
                    *self.advRunes.get_mut("Gust Rune").unwrap() += 1;
                }
            }
            if let Some(&earth) = self.runes.get("Earth Rune") {
                if ui.add_enabled(earth >= 5 && self.unlocks.advancedRunes == true, styled_button("Make Metal")).clicked() {
                    *self.runes.get_mut("Earth Rune").unwrap() -= 5;
                    *self.advRunes.get_mut("Metal Rune").unwrap() += 1;
                }
            }
            if let Some(&water) = self.runes.get("Water Rune") {
                if ui.add_enabled(water >= 5 && self.unlocks.advancedRunes == true, styled_button("Make Mist")).clicked() {
                    *self.runes.get_mut("Water Rune").unwrap() -= 5;
                    *self.advRunes.get_mut("Mist Rune").unwrap() += 1;
                }
            }
        });
    }

    fn show_smithing(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Smithing Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Coins: {}", self.coins)).color(egui::Color32::WHITE));
        match self.BuildingProgress {
            BuildingProgress::ForgeFoundation => {
                if let Some(&metal) = self.advRunes.get("Metal Rune") {
                    if ui.add_enabled(metal >= 5, styled_button("Buy Upgrade 5 Metal Runes")).clicked() {
                        *self.advRunes.get_mut("Metal Rune").unwrap() -= 5;
                    }
                }
            }
            BuildingProgress::ForgeAnvil => {
                if let Some(&metal) = self.advRunes.get("Metal Rune") {
                    if ui.add_enabled(metal >= 10, styled_button("Buy Upgrade 10 Metal Runes")).clicked() {
                        *self.advRunes.get_mut("Metal Rune").unwrap() -= 10;
                    }
                }
            }
            BuildingProgress::ForgeFireplace => {
                if let Some(&metal) = self.advRunes.get("Metal Rune") {
                    if ui.add_enabled(metal >= 15, styled_button("Buy Upgrade 15 Metal Runes")).clicked() {
                        *self.advRunes.get_mut("Metal Rune").unwrap() -= 15;
                    }
                }
            }
            BuildingProgress::ForgeWalls => {
                if let Some(&metal) = self.advRunes.get("Metal Rune") {
                    if ui.add_enabled(metal >= 20, styled_button("Buy Upgrade 20 Metal Runes")).clicked() {
                        *self.advRunes.get_mut("Metal Rune").unwrap() -= 20;
                    }
                }
            }
            BuildingProgress::ForgeRoof => {
                if let Some(&metal) = self.advRunes.get("Metal Rune") {
                    if ui.add_enabled(metal >= 25, styled_button("Buy Upgrade 25 Metal Runes")).clicked() {
                        *self.advRunes.get_mut("Metal Rune").unwrap() -= 25;
                    }
                }
            }
        }
        // Placeholder for smithing actions


    }

    fn show_upgrades(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Upgrades Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Purchase upgrades to enhance clicks or crafting.").color(egui::Color32::WHITE));

        if ui.add_enabled(self.essence >= 30, styled_button("Buy Upgrade (30 essence)")).clicked() {
            self.essence -= 30;
            self.essenceAmount += 1;
        }
    }

    fn show_quests(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Quests Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Turn in runes for rewards.").color(egui::Color32::WHITE));
        match self.fireQuest {
            FirstQuest::CollectFireRunes => {
                if let Some(&fire) = self.runes.get("Fire Rune") {
                    if ui.add_enabled(fire >= 10, styled_button("Turn in 10 Fire Runes")).clicked() {
                        *self.runes.get_mut("Fire Rune").unwrap() -= 10;
                        self.unlocks.advancedRunes = true;
                        self.fireQuest = FirstQuest::CollectPlasmaRunes;
                    }
                } 
            }
            FirstQuest::CollectPlasmaRunes => {
                if let Some(&plasma) = self.advRunes.get("Plasma Rune") {
                    if ui.add_enabled(plasma >= 1, styled_button("Turn in 1 Plasma Runes")).clicked() {
                        *self.advRunes.get_mut("Plasma Rune").unwrap() -= 1;
                        self.runeChance = 150;
                        self.fireQuest = FirstQuest::Complete;
                    }
                }
            }
            FirstQuest::Complete => {
                //hopes and prayers
            }
        }

    }
    fn show_equipment(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Equipment Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Manage your equipment here.").color(egui::Color32::WHITE));
        // Placeholder for equipment management
    }
    fn show_achievements(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Achievements Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Track your achievements here.").color(egui::Color32::WHITE));
        // Placeholder for achievements tracking
    }
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Settings Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Adjust your game settings here.").color(egui::Color32::WHITE));
        // Placeholder for settings adjustments
    }
    fn show_collections(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Collections Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("View your collections here.").color(egui::Color32::WHITE));
        // Placeholder for collections viewing
    }
    fn show_stat_breakdown(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Stat Break Down Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Detailed stats of your progress.").color(egui::Color32::WHITE));
        // Placeholder for detailed stats
    }
    
}

impl eframe::App for Clicker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu tabs
        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(styled_tab("Clicking")).clicked() {
                    self.current_tab = MenuTab::Clicking;
                }
                if ui.add(styled_tab("Smithing")).clicked() {
                    self.current_tab = MenuTab::Smithing;
                }
                if ui.add(styled_tab("Upgrades")).clicked() {
                    self.current_tab = MenuTab::Upgrades;
                }
                if ui.add(styled_tab("Quests")).clicked() {
                    self.current_tab = MenuTab::Quests;
                }
                if ui.add(styled_tab("Equipment")).clicked() {
                    self.current_tab = MenuTab::Equipment;
                }
                if ui.add(styled_tab("Achievements")).clicked() {
                    self.current_tab = MenuTab::Achievements;
                }
                if ui.add(styled_tab("Settings")).clicked() {
                    self.current_tab = MenuTab::Settings;
                }
                if ui.add(styled_tab("Collections")).clicked() {
                    self.current_tab = MenuTab::Collections;
                }
                if ui.add(styled_tab("Stat Break Down")).clicked() {
                    self.current_tab = MenuTab::StatBreakDown;
                }
            });
        });

        // Central panel content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                MenuTab::Clicking => self.show_clicking(ui),
                MenuTab::Smithing => self.show_smithing(ui),
                MenuTab::Upgrades => self.show_upgrades(ui),
                MenuTab::Quests => self.show_quests(ui),
                MenuTab::Equipment => self.show_equipment(ui),
                MenuTab::Achievements => self.show_achievements(ui),
                MenuTab::Settings => self.show_settings(ui),
                MenuTab::Collections => self.show_collections(ui),
                MenuTab::StatBreakDown => self.show_stat_breakdown(ui),
            }
        });
    }
}

