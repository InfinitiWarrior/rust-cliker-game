#![allow(deprecated)]
#![allow(warnings)]
use eframe::egui;
use std::collections::HashMap;
use indexmap::IndexMap;
use rand::Rng;
use std::fs;
use serde::Deserialize;
use anyhow::Result;

fn anyhow_to_eframe(e: anyhow::Error) -> eframe::Error {
    eframe::Error::AppCreation(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    )))
}

fn main() -> eframe::Result<()> {
    let save: Savefile = load_json("saves/save1.json").map_err(anyhow_to_eframe)?;
    let recipes: RecipesFile = load_json("data/recipes.json").map_err(anyhow_to_eframe)?;

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
        Box::new(move |_cc| Ok(Box::new(Clicker::from_save_with_recipes(save, recipes)))),
    )
}

fn load_json<T: for<'de> Deserialize<'de>>(file_path: &str) -> Result<T> {
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
    Gathering,
    Upgrades,
    Thauminomicon,
    Equipment,
    Achievements,
    Settings,
    StatBreakDown,
}

struct Clicker {
    unlocks: Unlocks,
    vis: u32,
    maxVis: u32,
    souls: u32,
    crystals: IndexMap<String, u32>,
    visClickAmount: u32,
    runeChance: u32,
    upgradePrices: UpgradePrices,
    current_tab: MenuTab,
    autoClickInterval: f32,
    autoClickTimer: f32,
    playTime: f32,
    // Thauminomicon state
    skills: Vec<SkillNode>,
    cam_offset: egui::Vec2,
    cam_zoom: f32,
    // Data
    recipes: RecipesFile,
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
    Vis: u32,
    crystals: IndexMap<String, u32>,
}

#[derive(Deserialize, Debug)]
struct Settings {
    colorScheme: String,
}

#[derive(Deserialize, Debug)]
struct Unlocks {
    advancedRunes: bool,
    secondary_crystals: bool,
    tertiary_crystals: bool,
    quaternary_crystals: bool,
    visConversion: bool,
    autoCliking: bool,
}

#[derive(Deserialize, Debug)]
struct Progress {
    totalClicks: u32,
    totalVisEarned: u32,
}

#[derive(Deserialize, Debug)]
struct Upgrades {
    #[serde(alias = "clickPower")]
    visClickAmount: u32,
    autoClicker: u32,
}

#[derive(Deserialize, Debug)]
struct RecipesFile {
    // crystals.category -> item -> cost_map (preserve JSON order)
    crystals: IndexMap<String, IndexMap<String, IndexMap<String, u32>>>,
}

struct UpgradePrices {
    vis_capacity: u32,
    vis_click_amount_cost: u32,
    vis_conversion_amount_cost: u32,
    auto_click_interval: u32,
}

#[derive(Clone)]
struct SkillNode {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    position: egui::Pos2,
    unlocked: bool,
    prerequisites: Vec<&'static str>,
}

impl Default for Clicker {
    fn default() -> Self {
        let crystals = IndexMap::new();
        Self {
            vis: 0,
            maxVis: 50,
            souls: 0,
            visClickAmount: 1,
            runeChance: 50,
            crystals,
            autoClickInterval: 30.0,
            autoClickTimer: 0.0,
            playTime: 0.0,
            unlocks: Unlocks {
                advancedRunes: false,
                secondary_crystals: false,
                tertiary_crystals: false,
                quaternary_crystals: false,
                visConversion: false,
                autoCliking: false,
            },
            upgradePrices: UpgradePrices {
                vis_capacity: 100,
                vis_click_amount_cost: 200,
                vis_conversion_amount_cost: 300,
                auto_click_interval: 800,
            },
            current_tab: MenuTab::Gathering,
            // Thauminomicon default
            skills: vec![
                SkillNode {
                    id: "basic_crystals",
                    name: "Basic Crystals",
                    description: "Foundational Crystals and minor inscriptions.",
                    position: egui::pos2(300.0, 300.0),
                    unlocked: true,
                    prerequisites: vec![],
                },
                SkillNode {
                    id: "essence_control",
                    name: "Essence Control",
                    description: "Harness and shape raw vis.",
                    position: egui::pos2(600.0, 360.0),
                    unlocked: false,
                    prerequisites: vec!["basic_crystals"],
                },
                SkillNode {
                    id: "advanced_crystals",
                    name: "Advanced Crystals",
                    description: "Complex runic patterns and bindings.",
                    position: egui::pos2(900.0, 300.0),
                    unlocked: false,
                    prerequisites: vec!["essence_control"],
                },
                SkillNode {
                    id: "forging",
                    name: "Forging",
                    description: "Imbue metals with magic.",
                    position: egui::pos2(900.0, 480.0),
                    unlocked: false,
                    prerequisites: vec!["essence_control"],
                },
                SkillNode {
                    id: "alchemy_basics",
                    name: "Alchemy Basics",
                    description: "Distill and combine essences.",
                    position: egui::pos2(1200.0, 300.0),
                    unlocked: false,
                    prerequisites: vec!["advanced_crystals"],
                },
            ],
            cam_offset: egui::vec2(0.0, 0.0),
            cam_zoom: 1.0,
            recipes: RecipesFile { crystals: IndexMap::new() },
        }
    }
}

impl Clicker {
    fn from_save_with_recipes(save: Savefile, recipes: RecipesFile) -> Self {
        let mut clicker_default = Clicker::default();
        clicker_default.crystals = save.inventory.crystals;
        clicker_default.recipes = recipes;
        // Initialize from saved upgrades
        clicker_default.visClickAmount = save.upgrades.visClickAmount;
        clicker_default
    }

    fn can_unlock(&self, id: &str) -> bool {
        if let Some(node) = self.skills.iter().find(|n| n.id == id) {
            if node.unlocked {
                return false; // already unlocked
            }
            for pre in &node.prerequisites {
                if let Some(req) = self.skills.iter().find(|n| &n.id == pre) {
                    if !req.unlocked {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    fn unlock_skill(&mut self, id: &str) {
        if !self.can_unlock(id) {
            return;
        }

        match id {
            // Essence Control costs 50 vis to unlock
            "essence_control" => {
                if self.vis >= 50 {
                    self.vis -= 50;
                    if let Some(n) = self.skills.iter_mut().find(|n| n.id == id) {
                        n.unlocked = true;
                    }
                    // Unlock secondary crystal crafting
                    self.unlocks.secondary_crystals = true;
                }
            }
            // Other nodes are free for now
            _ => {
                if let Some(n) = self.skills.iter_mut().find(|n| n.id == id) {
                    n.unlocked = true;
                }
            }
        }
    }

    fn show_thauminomicon(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Thauminomicon").color(egui::Color32::WHITE));
        ui.separator();

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let canvas_size = egui::vec2(3000.0, 3000.0);
                let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::drag());
                let painter = ui.painter_at(rect);

                // Input handling: zoom with Ctrl+Wheel, pan with middle-mouse drag
                let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                let ctrl = ui.input(|i| i.modifiers.ctrl);
                if ctrl && scroll_delta.abs() > 0.0 {
                    let zoom_factor = (1.0 + (scroll_delta * 0.001)).clamp(0.5, 1.5);
                    self.cam_zoom = (self.cam_zoom * zoom_factor).clamp(0.25, 3.0);
                }
                let pointer_delta = ui.input(|i| i.pointer.delta());
                let middle_down = ui.input(|i| i.pointer.middle_down());
                if middle_down {
                    self.cam_offset += pointer_delta;
                }

                // Helpers
                let to_screen = |p: egui::Pos2| -> egui::Pos2 {
                    egui::pos2(
                        rect.min.x + self.cam_offset.x + p.x * self.cam_zoom,
                        rect.min.y + self.cam_offset.y + p.y * self.cam_zoom,
                    )
                };
                let node_size = egui::vec2(180.0, 64.0) * self.cam_zoom;

                // Draw edges first
                for node in &self.skills {
                    for pre in &node.prerequisites {
                        if let Some(req) = self.skills.iter().find(|n| &n.id == pre) {
                            let a = to_screen(req.position);
                            let b = to_screen(node.position);
                            let col = if req.unlocked { egui::Color32::from_rgb(80, 200, 120) } else { egui::Color32::GRAY };
                            painter.line_segment([a, b], egui::Stroke { width: 2.0, color: col });
                        }
                    }
                }

                                // Draw nodes with enlarge/vibrate for unlockable and robust hover detection
                let mut clicked_id: Option<String> = None;
                let pointer_pos = ui.ctx().pointer_latest_pos();
                for node in &self.skills {
                    // Base center and size
                    let mut center = to_screen(node.position);
                    let mut size = node_size;
                    let can_unlock = self.can_unlock(node.id);

                    // Enlarge/vibrate effect for unlockable nodes
                    if can_unlock {
                        let t = ui.ctx().input(|i| i.time as f32);
                        let scale = 1.08 + 0.02 * (t * 3.5).sin();
                        size *= scale;
                        let phase = (node.id.as_bytes()[0] as f32) * 0.37;
                        let jiggle = egui::vec2(
                            (t * 9.0 + phase).sin() * 1.5 * self.cam_zoom,
                            (t * 11.0 + phase * 0.7).cos() * 1.5 * self.cam_zoom,
                        );
                        center += jiggle;
                    }

                    let rect_node = egui::Rect::from_center_size(center, size);
                    let color = if node.unlocked {
                        egui::Color32::from_rgb(50, 190, 90)
                    } else if can_unlock {
                        egui::Color32::from_rgb(60, 140, 220)
                    } else {
                        egui::Color32::from_gray(50)
                    };

                    painter.rect_filled(rect_node, egui::Rounding::same(10), color);
                    painter.rect_stroke(
                        rect_node,
                        egui::Rounding::same(10),
                        egui::Stroke { width: 2.0, color: egui::Color32::BLACK },
                        egui::StrokeKind::Outside,
                    );
                    painter.text(
                        rect_node.center(),
                        egui::Align2::CENTER_CENTER,
                        node.name,
                        egui::FontId::proportional(16.0 * self.cam_zoom),
                        egui::Color32::WHITE,
                    );

                    if let Some(pp) = pointer_pos {
                        if rect_node.contains(pp) {
                            egui::containers::show_tooltip_for(
                                ui.ctx(),
                                ui.layer_id(),
                                egui::Id::new(format!("node_tt_{}", node.id)),
                                &rect_node,
                                |ui: &mut egui::Ui| {
                                    ui.label(node.description);
                                    if node.id == "essence_control" {
                                        ui.separator();
                                        ui.label(format!("Cost: 50 Vis (you have {})", self.vis));
                                    }
                                },
                            );
                            if ui.input(|i| i.pointer.primary_clicked()) {
                                clicked_id = Some(node.id.to_string());
                            }
                        }
                    }
                }if let Some(id) = clicked_id {
                    self.unlock_skill(&id);
                }
            });
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
    fn show_gathering(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Gather Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Vis: {}/{}", self.vis, self.maxVis)).color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Souls: {}", self.souls)).color(egui::Color32::WHITE));

        // Secondary crystals crafting (from recipes.json), unlocked by Essence Control
        if self.unlocks.secondary_crystals {
            ui.separator();
            ui.label(egui::RichText::new("secondary crystals").color(egui::Color32::LIGHT_BLUE));
            if let Some(categories) = self.recipes.crystals.get("secondary") {
                // Render buttons left-to-right and wrap when needed
                ui.horizontal_wrapped(|ui| {
                    for (name, costs) in categories {
                        // Check if we can afford costs
                        let mut can_afford = true;
                        for (req, &amt) in costs {
                            if self.crystals.get(req).copied().unwrap_or(0) < amt {
                                can_afford = false;
                                break;
                            }
                        }
                        let label = format!("Make {}", name);
                        if ui.add_enabled(can_afford, styled_button(&label)).clicked() {
                            // Deduct inputs
                            for (req, &amt) in costs {
                                if let Some(entry) = self.crystals.get_mut(req) {
                                    *entry = entry.saturating_sub(amt);
                                }
                            }
                            // Add output
                            *self.crystals.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                });
            }
        }


        // Clicking button
        if ui.add(styled_button("Conjure resources")).clicked() {
            self.vis = (self.vis + self.visClickAmount).min(self.maxVis);
            let mut rng = rand::thread_rng();

            // Crystal gain: with the same chance as runes, add exactly one base crystal
            if rng.gen_range(0..100) < self.runeChance {
                let base_crystals = ["aer", "aqua", "ignis", "ordo", "perditio", "terra"];
                let chosen_idx = rng.gen_range(0..base_crystals.len());
                let chosen = base_crystals[chosen_idx];
                if let Some(val) = self.crystals.get_mut(chosen) {
                    *val += 1;
                } else {
                    // In case save didn't have the key, insert it
                    self.crystals.insert(chosen.to_string(), 1);
                }
            }
        }

    }

    fn show_upgrades(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Upgrades Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Purchase upgrades to enhance clicks or crafting.").color(egui::Color32::WHITE));

        // Upgrade 1: Soul to vis Conversion
        if ui.add_enabled(self.souls >= 1, styled_button("Upgrade Vis Click Amount (1 soul)")).clicked() {
            if safe_subtract(&mut self.souls, 1) {
                self.visClickAmount += 1;
            }
        }

        // Upgrade 2: vis Capacity
        if ui.add_enabled(self.vis >= 50, styled_button("Upgrade Vis Capacity (+50) (50 Vis)")).clicked() {
            if safe_subtract(&mut self.vis, 50) {
                self.maxVis += 50;
            }
        }

        // Upgrade 3: Vis Click Amount
        if ui.add_enabled(self.vis >= 200, styled_button("Upgrade Vis Click Amount (+1) (200 Vis)")).clicked() {
            if safe_subtract(&mut self.vis, 200) {
                self.visClickAmount += 1;
            }
        }

        // Upgrade 4: Auto Clicking
        if ui.add_enabled(self.unlocks.autoCliking, styled_button("Upgrade Auto Click Interval (-0.5s)")).clicked() {
            if self.autoClickInterval >= 1.0 {
                self.autoClickInterval -= 0.5;
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
    fn show_stat_breakdown(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Stat Break Down Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Detailed stats of your progress.").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Vis Limit {}", self.maxVis)).color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Vis per Click {}", self.visClickAmount)).color(egui::Color32::WHITE));
        ui.label(egui::RichText::new(format!("Rune Chance {}", self.runeChance)).color(egui::Color32::WHITE));
        // Placeholder for detailed stats
    }
}

impl eframe::App for Clicker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // inside update(...) where you have access to `ctx`
        if self.unlocks.autoCliking {
            // request continuous repaints so update() runs each frame
            ctx.request_repaint();
            // read delta time (seconds since last frame)
            let dt = ctx.input(|i| i.unstable_dt); // this is what you used; keep it if it compiles
            // If you use a different egui version where unstable_dt is Option<f32>, use:
            // let dt = ctx.input(|i| i.unstable_dt).unwrap_or(0.0);
            self.autoClickTimer += dt;
            if self.autoClickTimer >= self.autoClickInterval {
                self.autoClickTimer -= self.autoClickInterval;
                self.vis = (self.vis + self.visClickAmount).min(self.maxVis);
                
            }
        }

        let bg_color = match self.current_tab {
            MenuTab::Gathering => egui::Color32::from_rgb(40, 40, 80),
            MenuTab::Upgrades => egui::Color32::from_rgb(30, 60, 30),
            MenuTab::Thauminomicon => egui::Color32::from_rgb(20, 20, 30),
            MenuTab::Equipment => egui::Color32::from_rgb(30, 30, 60),
            MenuTab::Achievements => egui::Color32::from_rgb(80, 40, 40),
            MenuTab::Settings => egui::Color32::from_rgb(50, 30, 70),
            MenuTab::StatBreakDown => egui::Color32::from_rgb(30, 70, 30),
        };
        // Top menu tabs
        egui::TopBottomPanel::top("menu_panel")
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(bg_color.r(), bg_color.g(), bg_color.b())) // Background color of the tab bar
                    .inner_margin(egui::Margin::same(5)) // Optional padding
            )
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(styled_tab("Gather/Convert")).clicked() {
                    self.current_tab = MenuTab::Gathering;
                }
                if ui.add(styled_tab("Upgrades")).clicked() {
                    self.current_tab = MenuTab::Upgrades;
                }
                if ui.add(styled_tab("Thauminomicon")).clicked() {
                    self.current_tab = MenuTab::Thauminomicon;
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
                if ui.add(styled_tab("Stat Break Down")).clicked() {
                    self.current_tab = MenuTab::StatBreakDown;
                }
            });
        });

        // Always-visible right side panel (inventory)
        egui::SidePanel::right("inventory_panel")
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("Inventory").color(egui::Color32::WHITE));
                ui.separator();
                ui.label(egui::RichText::new("Crystals").color(egui::Color32::WHITE));
                for (crystal, amount) in &self.crystals {
                    ui.label(egui::RichText::new(format!("{}: {}", crystal, amount)).color(egui::Color32::WHITE));
                }
            });

        // Central panel content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(bg_color.r(), bg_color.g(), bg_color.b()))
            )
            .show(ctx, |ui| {
                match self.current_tab {
                    MenuTab::Gathering => self.show_gathering(ui),
                    MenuTab::Upgrades => self.show_upgrades(ui),
                    MenuTab::Thauminomicon => self.show_thauminomicon(ui),
                    MenuTab::Equipment => self.show_equipment(ui),
                    MenuTab::Achievements => self.show_achievements(ui),
                    MenuTab::Settings => self.show_settings(ui),
                    MenuTab::StatBreakDown => self.show_stat_breakdown(ui),
                }
            });
    }
}




