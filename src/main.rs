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
    essence: u32,
    coins: u32,
    souls: u32,
    runes: HashMap<String, u32>,
    current_tab: MenuTab,
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
            runes,
            current_tab: MenuTab::Clicking,
        }
    }
}

// Helper function for main action buttons
fn styled_button(label: &str) -> egui::Button {
    egui::Button::new(label)
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
        .min_size([150.0, 50.0].into())
}

// Helper function for tab buttons (slightly smaller)
fn styled_tab(label: &str) -> egui::Button {
    egui::Button::new(label)
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
        .min_size([120.0, 40.0].into())
}

impl Clicker {
    fn show_clicking(&mut self, ui: &mut egui::Ui) {
        ui.heading("Clicking Menu");
        ui.label(format!("Essence: {}", self.essence));

        // Display runes
        ui.horizontal(|ui| {
            for (rune, amount) in &self.runes {
                ui.label(format!("{}: {}", rune, amount));
            }
        });

        // Clicking button
        if ui.add(styled_button("Conjure resources")).clicked() {
            self.essence += 1;

            let mut rng = rand::thread_rng();

            if rng.gen_range(0..100) < 50 {
                let keys: Vec<String> = self.runes.keys().cloned().collect();
                let chosen = &keys[rng.gen_range(0..keys.len())];
                *self.runes.get_mut(chosen.as_str()).unwrap() += 1;
            }
        }
    }

    fn show_smithing(&mut self, ui: &mut egui::Ui) {
        ui.heading("Smithing Menu");
        ui.label(format!("Coins: {}", self.coins));

        if ui.add_enabled(self.coins >= 10, styled_button("Buy Upgrade (10 coins)")).clicked() {
            self.coins -= 10;
            // TODO: apply upgrade effects
        }
    }

    fn show_upgrades(&mut self, ui: &mut egui::Ui) {
        ui.heading("Upgrades Menu");
        ui.label("Purchase upgrades to enhance clicks or crafting.");
    }

    fn show_quests(&mut self, ui: &mut egui::Ui) {
        ui.heading("Quests Menu");
        ui.label("Turn in runes for rewards.");

        if let Some(&fire) = self.runes.get("Fire Rune") {
            if ui.add_enabled(fire >= 10, styled_button("Turn in 10 Fire Runes")).clicked() {
                *self.runes.get_mut("Fire Rune").unwrap() -= 10;
                self.souls += 1;
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

