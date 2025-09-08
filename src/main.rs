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
}

struct Clicker {
    unlocks: Unlocks,
    essence: u32,
    coins: u32,
    souls: u32,
    essenceAmount: u32,
    runeChance: u32,
    runes: HashMap<String, u32>,
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
        Self {
            essence: 0,
            coins: 0,
            souls: 0,
            essenceAmount: 1,
            runeChance: 50,
            runes,
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
                    self.souls += 1;
                }
            }
            if let Some(&air) = self.runes.get("Air Rune") {
                if ui.add_enabled(air >= 5 && self.unlocks.advancedRunes == true, styled_button("Make Gust")).clicked() {
                    *self.runes.get_mut("Air Rune").unwrap() -= 5;
                    self.souls += 1;
                }
            }
            if let Some(&earth) = self.runes.get("Earth Rune") {
                if ui.add_enabled(earth >= 5 && self.unlocks.advancedRunes == true, styled_button("Make Metal")).clicked() {
                    *self.runes.get_mut("earth Rune").unwrap() -= 5;
                    self.souls += 1;
                }
            }
            if let Some(&water) = self.runes.get("Water Rune") {
                if ui.add_enabled(water >= 5 && self.unlocks.advancedRunes == true, styled_button("Make Mist")).clicked() {
                    *self.runes.get_mut("Water Rune").unwrap() -= 5;
                    self.souls += 1;
                }
            }
        });
    }

    fn show_smithing(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Smithing Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Coins: {}", self.coins)).color(egui::Color32::WHITE));

        if ui.add_enabled(self.coins >= 10, styled_button("Buy Upgrade (10 coins)")).clicked() {
            self.coins -= 10;
            // TODO: apply upgrade effects
        }
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

        if let Some(&fire) = self.runes.get("Fire Rune") {
            if ui.add_enabled(fire >= 10 && self.unlocks.advancedRunes == false, styled_button("Turn in 10 Fire Runes")).clicked() {
                *self.runes.get_mut("Fire Rune").unwrap() -= 10;
                self.unlocks.advancedRunes = true;
            }
        }
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
            });
        });

        // Central panel content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                MenuTab::Clicking => self.show_clicking(ui),
                MenuTab::Smithing => self.show_smithing(ui),
                MenuTab::Upgrades => self.show_upgrades(ui),
                MenuTab::Quests => self.show_quests(ui),
            }
        });
    }
}

