#![allow(deprecated)]
#![allow(warnings)]
use eframe::egui;
use std::collections::{HashMap, HashSet};
use indexmap::IndexMap;
use rand::Rng;
use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/aspects/"]
#[include = "**/*.png"]
struct Aspects;

fn anyhow_to_eframe(e: anyhow::Error) -> eframe::Error {
    eframe::Error::AppCreation(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    )))
}

// Embedded static data
const RECIPES_JSON: &str = include_str!("../data/recipes.json");
const RESEARCH_JSON: &str = include_str!("../data/research.json");
const DEFAULT_SAVE_JSON: &str = include_str!("../saves/deafault-save.json");

fn main() -> eframe::Result<()> {
    let save: Savefile = load_or_create_save();
    let recipes: RecipesFile = serde_json::from_str(RECIPES_JSON).map_err(|e| anyhow_to_eframe(e.into()))?;
    let research: ResearchTree = serde_json::from_str(RESEARCH_JSON).map_err(|e| anyhow_to_eframe(e.into()))?;

    println!("Player: {} [{}]", save.player.Charactername, save.player.Title);
    println!("Level: {}, XP: {}\n", save.player.Level, save.player.Experience);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // Start maximized so it adapts to any screen size dynamically
            .with_maximized(true)
            // Keep a sensible minimum so small screens are usable
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Clicker Game",
        options,
        Box::new(move |_cc| Ok(Box::new(Clicker::from_save_with_data(save, recipes, research)))),
    )
}

fn load_json<T: for<'de> Deserialize<'de>>(file_path: &str) -> Result<T> {
    let data = fs::read_to_string(file_path)?;
    let parsed: T = serde_json::from_str(&data)?;
    Ok(parsed)
}

fn load_research_data(_path: &str) -> Result<ResearchTree> { serde_json::from_str(RESEARCH_JSON).map_err(Into::into) }

fn save_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")))
        .join("save.json")
}

fn load_or_create_save() -> Savefile {
    let path = save_path();
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => {
            let _ = std::fs::write(&path, DEFAULT_SAVE_JSON.as_bytes());
            serde_json::from_str(DEFAULT_SAVE_JSON).unwrap_or_default()
        }
    }
}

fn save_game(app: &Clicker) -> anyhow::Result<()> {
    let mut save = Savefile::default();
    save.inventory.Vis = app.vis;
    save.inventory.crystals = app.crystals.clone();
    save.unlocks = app.unlocks.clone();
    save.upgrades.visClickAmount = app.visClickAmount;
    save.upgrades.crystalClickAmount = app.crystalClickAmount;
    // Persist research progress
    save.unlocked_nodes = app.unlocked_nodes.iter().cloned().collect();
    save.unlocked_recipes = app.unlocked_recipes.iter().cloned().collect();
    save.unlocked_research_tabs = app.unlocked_research_tabs.iter().cloned().collect();
    let json = serde_json::to_vec_pretty(&save)?;
    std::fs::write(save_path(), json)?;
    Ok(())
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
}

struct Clicker {
    unlocks: Unlocks,
    vis: u32,
    maxVis: u32,
    crystals: IndexMap<String, u32>,
    visClickAmount: u32,
    crystalClickAmount: u32,
    runeChance: u32,
    upgradePrices: UpgradePrices,
    current_tab: MenuTab,
    autoClickInterval: f32,
    autoClickTimer: f32,
    playTime: f32,
    autosave_timer: f32,
    // Thauminomicon state
    skills: Vec<SkillNode>,
    cam_offset: egui::Vec2,
    cam_zoom: f32,
    // Data
    recipes: RecipesFile,
    // Cached textures for crystal icons
    textures: HashMap<String, egui::TextureHandle>,
    research: ResearchTree,
    current_research_tab: String,
    unlocked_research_tabs: HashSet<String>,
    unlocked_nodes: HashSet<String>,
    // Recipes unlocked via research: item ids like "gelum", "metallum"
    unlocked_recipes: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct Savefile {
    player: Player,
    inventory: Inventory,
    settings: Settings,
    unlocks: Unlocks,
    progress: Progress,
    upgrades: Upgrades,

    // NEW: what to persist about research/thauminomicon
    unlocked_nodes: Vec<String>,         // list of node IDs
    unlocked_recipes: Vec<String>,       // recipe ids unlocked by research
    unlocked_research_tabs: Vec<String>, // research tabs unlocked
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Player {
    Charactername: String,
    Title: String,
    Level: u32,
    Experience: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Inventory {
    Vis: u32,
    crystals: IndexMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Settings {
    colorScheme: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct Unlocks {
    advancedRunes: bool,
    secondary_crystals: bool,
    tertiary_crystals: bool,
    quaternary_crystals: bool,
    visConversion: bool,
    autoCliking: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Progress {
    totalClicks: u32,
    totalVisEarned: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Upgrades {
    visClickAmount: u32,
    crystalClickAmount: u32,
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

// Research data-driven system
#[derive(Deserialize, Clone)]
struct ResearchNode {
    id: String,
    name: String,
    description: String,
    x: f32,
    y: f32,
    cost: IndexMap<String, u32>,
    prerequisites: Vec<String>,
    unlocks: Option<Vec<String>>,
    unlocks_nodes: Option<Vec<String>>,
    unlocks_menu: Option<String>,
}

type ResearchTree = IndexMap<String, Vec<ResearchNode>>; // category -> nodes

impl Default for Clicker {
    fn default() -> Self {
        let crystals = IndexMap::new();
        Self {
            vis: 0,
            maxVis: 50,
            visClickAmount: 50,
            crystalClickAmount: 1,
            runeChance: 50,
            crystals,
            autoClickInterval: 30.0,
            autoClickTimer: 0.0,
            playTime: 0.0,
            autosave_timer: 0.0,
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
            // Data-driven research now provides nodes; keep legacy skills empty
            skills: Vec::new(),
            cam_offset: egui::vec2(0.0, 0.0),
            cam_zoom: 1.0,
            recipes: RecipesFile { crystals: IndexMap::new() },
            textures: HashMap::new(),
            research: IndexMap::new(),
            current_research_tab: "Crystallography".to_string(),
            unlocked_research_tabs: {
                let mut s = HashSet::new();
                s.insert("Crystallography".to_string());
                s
            },
            unlocked_nodes: HashSet::new(),
            unlocked_recipes: HashSet::new(),
        }
    }
}


// Unlock result types
#[derive(Debug)]
enum UnlockError {
    NotFound,
    AlreadyUnlocked,
    PrerequisitesMissing(Vec<String>),
    InsufficientVis { needed: u32, have: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnlockOutcome {
    Unlocked,
    AlreadyUnlocked,
}

impl Clicker {
    fn from_save_with_data(save: Savefile, recipes: RecipesFile, research: ResearchTree) -> Self {
        let mut clicker_default = Clicker::default();
        // Restore inventory state
        clicker_default.crystals = save.inventory.crystals;
        clicker_default.vis = save.inventory.Vis;
        clicker_default.recipes = recipes;
        clicker_default.research = research;
        // Initialize from saved upgrades
        clicker_default.visClickAmount = save.upgrades.visClickAmount;
        // Populate runtime sets from save vectors
        clicker_default.unlocked_nodes = save.unlocked_nodes.into_iter().collect();
        clicker_default.unlocked_recipes = save.unlocked_recipes.into_iter().collect();
        clicker_default.unlocked_research_tabs = save.unlocked_research_tabs.into_iter().collect();
        // Ensure at least one research tab is unlocked
        if clicker_default.unlocked_research_tabs.is_empty() {
            if let Some((first_tab, _)) = clicker_default.research.iter().next() {
                clicker_default.unlocked_research_tabs.insert(first_tab.clone());
                clicker_default.current_research_tab = first_tab.clone();
            }
        } else if !clicker_default.unlocked_research_tabs.contains(&clicker_default.current_research_tab) {
            // pick a valid current tab
            if let Some(tab) = clicker_default.unlocked_research_tabs.iter().next() {
                clicker_default.current_research_tab = tab.clone();
            }
        }
        clicker_default
    }

        fn get_crystal_icon(&mut self, ctx: &egui::Context, name: &str) -> Option<&egui::TextureHandle> {
        if self.textures.contains_key(name) {
            return self.textures.get(name);
        }

        // Try embedded first
        let path = format!("{}.png", name);
        if let Some(file) = Aspects::get(&path) {
            if let Ok(image) = image::load_from_memory(&file.data) {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
                let tex = ctx.load_texture(
                    format!("crystal_{}", name),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.textures.insert(name.to_string(), tex);
                return self.textures.get(name);
            }
        }

        // Fallback for dev mode
        let try_paths = [
            format!("assets/aspects/{}.png", name),
            format!("assets/apsects/{}.png", name),
        ];
        for path in try_paths.iter() {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(image) = image::load_from_memory(&bytes) {
                    let rgba = image.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = rgba.as_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
                    let tex = ctx.load_texture(
                        format!("crystal_{}", name),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );
                    self.textures.insert(name.to_string(), tex);
                    return self.textures.get(name);
                }
            }
        }

        None
    }

    // Research system helpers
    fn can_unlock_node(&self, id: &str) -> bool {
        if self.unlocked_nodes.contains(id) {
            return false;
        }
        // find node
        let node = self
            .research
            .values()
            .flat_map(|v| v.iter())
            .find(|n| n.id == id);
        let Some(node) = node else { return false; };
        node.prerequisites.iter().all(|pre| self.unlocked_nodes.contains(pre))
    }

    fn can_afford_cost(&self, cost: &IndexMap<String, u32>) -> bool {
        for (k, &amt) in cost.iter() {
            match k.as_str() {
                "Vis" => { if self.vis < amt { return false; } }
                // Souls removed from the game; ignore any legacy Soul cost keys
                "Soul" | "Souls" => { /* ignore */ }
                _ => {
                    if self.crystals.get(k).copied().unwrap_or(0) < amt { return false; }
                }
            }
        }
        true
    }

    fn spend_cost(&mut self, cost: &IndexMap<String, u32>) {
        for (k, &amt) in cost.iter() {
            match k.as_str() {
                "Vis" => { self.vis = self.vis.saturating_sub(amt); }
                // Souls removed from the game; ignore any legacy Soul cost keys
                "Soul" | "Souls" => { /* ignore */ }
                _ => {
                    if let Some(v) = self.crystals.get_mut(k) { *v = v.saturating_sub(amt); }
                }
            }
        }
    }

    fn apply_unlocks(&mut self, unlocks: &[String]) {
        for u in unlocks {
            match u.as_str() {
                "secondary_crystals" => self.unlocks.secondary_crystals = true,
                "tertiary_crystals" => self.unlocks.tertiary_crystals = true,
                "quaternary_crystals" => self.unlocks.quaternary_crystals = true,
                "vis_conversion" => self.unlocks.visConversion = true,
                "auto_clicking" => self.unlocks.autoCliking = true,
                "advancedRunes" => self.unlocks.advancedRunes = true,
                _ => {
                    if let Some(rest) = u.strip_prefix("recipe:") {
                        self.unlocked_recipes.insert(rest.to_string());
                    }
                }
            }
        }
    }

    fn unlock_node(&mut self, id: &str) -> Result<UnlockOutcome, UnlockError> {
        if self.unlocked_nodes.contains(id) {
            return Err(UnlockError::AlreadyUnlocked);
        }
        // Locate node (and its category) for reading data
        let (cat_key, idx) = {
            let mut found: Option<(String, usize)> = None;
            for (cat, nodes) in &self.research {
                if let Some(i) = nodes.iter().position(|n| n.id == id) {
                    found = Some((cat.clone(), i));
                    break;
                }
            }
            found.ok_or(UnlockError::NotFound)?
        };
        let node = &self.research.get(&cat_key).unwrap()[idx];
        // Clone dynamic fields to avoid holding an immutable borrow across mutation
        let cost = node.cost.clone();
        let unlocks = node.unlocks.clone();
        let unlocks_menu = node.unlocks_menu.clone();

        // Prerequisites
        let missing: Vec<String> = node
            .prerequisites
            .iter()
            .filter(|pre| !self.unlocked_nodes.contains(pre.as_str()))
            .cloned()
            .collect();
        if !missing.is_empty() { return Err(UnlockError::PrerequisitesMissing(missing)); }

        // Cost
        if !self.can_afford_cost(&cost) {
            // derive needed vis/souls is complex; return generic insufficient vis with current vis for simplicity
            return Err(UnlockError::InsufficientVis { needed: 0, have: self.vis });
        }
        self.spend_cost(&cost);

        // Mark unlocked and reward
        self.unlocked_nodes.insert(id.to_string());

        // Apply unlocks
        if let Some(unlocks) = &unlocks { self.apply_unlocks(&unlocks); }
        if let Some(tab) = &unlocks_menu { self.unlocked_research_tabs.insert(tab.clone()); }

        Ok(UnlockOutcome::Unlocked)
    }

    fn show_research_book(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Thauminomicon").color(egui::Color32::WHITE));
        ui.separator();

        // Fallback: if current tab is missing (e.g., mismatched name), pick the first available
        if !self.research.is_empty() && !self.research.contains_key(&self.current_research_tab) {
            if let Some((first_tab, _)) = self.research.iter().next() {
                self.current_research_tab = first_tab.clone();
                self.unlocked_research_tabs.insert(first_tab.clone());
            }
        }

        if self.research.is_empty() {
            ui.colored_label(egui::Color32::LIGHT_RED, "No research data found. Ensure data/research.json exists and loads correctly.");
            return;
        }

        // Tabs for research categories
        ui.horizontal(|ui| {
            for tab in self.unlocked_research_tabs.clone().into_iter() {
                if ui.add(styled_tab(&tab)).clicked() { self.current_research_tab = tab; }
            }
        });

        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            let canvas_size = egui::vec2(3000.0, 3000.0);
            let (rect, _response) = ui.allocate_exact_size(canvas_size, egui::Sense::drag());
            let painter = ui.painter_at(rect);

            // Zoom + pan controls
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            let ctrl = ui.input(|i| i.modifiers.ctrl);
            if ctrl && scroll_delta.abs() > 0.0 {
                let zoom_factor = (1.0 + (scroll_delta * 0.001)).clamp(0.5, 1.5);
                self.cam_zoom = (self.cam_zoom * zoom_factor).clamp(0.25, 3.0);
            }
            let pointer_delta = ui.input(|i| i.pointer.delta());
            let middle_down = ui.input(|i| i.pointer.middle_down());
            if middle_down { self.cam_offset += pointer_delta; }

            let to_screen = |p: egui::Pos2| -> egui::Pos2 {
                egui::pos2(
                    rect.min.x + self.cam_offset.x + p.x * self.cam_zoom,
                    rect.min.y + self.cam_offset.y + p.y * self.cam_zoom,
                )
            };
            let node_size = egui::vec2(180.0, 64.0) * self.cam_zoom;

            let nodes = match self.research.get(&self.current_research_tab) { Some(v) => v, None => return };

            // Show all nodes in the current tab; color/animation indicates state.

            // Draw edge shafts first (under nodes) and collect arrowheads to draw on top
            let mut arrowheads: Vec<(egui::Pos2, egui::Pos2, egui::Pos2, egui::Stroke)> = Vec::new();
            for n in nodes.iter() {
                for pre in &n.prerequisites {
                    if let Some(pnode) = nodes.iter().find(|pn| &pn.id == pre) {
                        let a = to_screen(egui::pos2(pnode.x, pnode.y));
                        let b_center = to_screen(egui::pos2(n.x, n.y));
                        let stroke = egui::Stroke { width: 2.0, color: egui::Color32::DARK_GRAY };
                        let dir = b_center - a;
                        let len = dir.length();
                        if len > 1.0 {
                            let u = dir / len;
                            let arrow_len = 12.0 * self.cam_zoom.max(0.5);
                            let arrow_half_w = 6.0 * self.cam_zoom.max(0.5);
                            // Compute intersection with child node rectangle to place arrow just before it
                            let hx = (node_size.x * 0.5).max(1.0);
                            let hy = (node_size.y * 0.5).max(1.0);
                            let ux = u.x.abs();
                            let uy = u.y.abs();
                            let tx = if ux > 1e-6 { hx / ux } else { f32::INFINITY };
                            let ty = if uy > 1e-6 { hy / uy } else { f32::INFINITY };
                            let t_edge = tx.min(ty);
                            // Edge point on the child rect boundary (from center along -u)
                            let edge_point = b_center - u * t_edge;
                            let tip_inset = 4.0 * self.cam_zoom.max(0.5);
                            let tip = edge_point - u * tip_inset;
                            let base = tip - u * arrow_len;
                            let perp = egui::vec2(-u.y, u.x);
                            let left = base + perp * arrow_half_w;
                            let right = base - perp * arrow_half_w;
                            // main shaft up to base of arrow (so head sits on top of node)
                            painter.line_segment([a, base], stroke);
                            arrowheads.push((tip, left, right, stroke));
                        } else {
                            painter.line_segment([a, b_center], stroke);
                        }
                    }
                }
            }

            let pointer_pos = ui.ctx().pointer_latest_pos();
            let mut clicked: Option<String> = None;
            for n in nodes.iter() {
                let mut center = to_screen(egui::pos2(n.x, n.y));
                let mut size = node_size;
                let unlocked = self.unlocked_nodes.contains(&n.id);
                let unlockable = !unlocked && self.can_unlock_node(&n.id) && self.can_afford_cost(&n.cost);
                if unlockable {
                    let t = ui.ctx().input(|i| i.time as f32);
                    let scale = 1.05 + 0.02 * (t * 3.5).sin();
                    size *= scale;
                }
                let rect_node = egui::Rect::from_center_size(center, size);
                let color = if unlocked { egui::Color32::from_rgb(50,190,90) } else if unlockable { egui::Color32::from_rgb(60,140,220) } else { egui::Color32::from_gray(50) };
                painter.rect_filled(rect_node, egui::Rounding::same(10), color);
                painter.rect_stroke(rect_node, egui::Rounding::same(10), egui::Stroke{width:2.0, color: egui::Color32::BLACK}, egui::StrokeKind::Outside);
                painter.text(rect_node.center(), egui::Align2::CENTER_CENTER, &n.name, egui::FontId::proportional(14.0*self.cam_zoom), egui::Color32::WHITE);
                if let Some(pp) = pointer_pos { if rect_node.contains(pp) {
                    egui::containers::show_tooltip_for(ui.ctx(), ui.layer_id(), egui::Id::new(format!("node_tt_{}", n.id)), &rect_node, |ui: &mut egui::Ui| {
                        ui.label(&n.description);
                        if !n.cost.is_empty() { ui.label(format!("Cost: {:?}", n.cost)); }
                    });
                    if unlockable && ui.input(|i| i.pointer.primary_clicked()) { clicked = Some(n.id.clone()); }
                }}
            }
            if let Some(id) = clicked { let _ = self.unlock_node(&id); }
            // Draw arrowheads on top of nodes so they are visible
            for (tip, left, right, stroke) in arrowheads {
                painter.line_segment([tip, left], stroke);
                painter.line_segment([tip, right], stroke);
            }
        });
    }

    fn show_recipes(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Recipes").color(egui::Color32::WHITE));
        ui.separator();
        if self.recipes.crystals.is_empty() {
            ui.colored_label(egui::Color32::LIGHT_RED, "No recipes loaded. Check data/recipes.json");
            return;
        }
        let category_unlocked = |cat: &str, unlocks: &Unlocks| -> bool {
            match cat {
                "secondary" => unlocks.secondary_crystals,
                "tertiary" => unlocks.tertiary_crystals,
                "quaternary" => unlocks.quaternary_crystals,
                _ => true,
            }
        };

        // Clone the recipes to avoid borrowing self while rendering and loading textures
        let recipes_snapshot = self.recipes.crystals.clone();
        for (category, items) in recipes_snapshot {
            if !category_unlocked(category.as_str(), &self.unlocks) { continue; }
            ui.label(egui::RichText::new(&category).strong().color(egui::Color32::LIGHT_BLUE));
            ui.horizontal_wrapped(|ui| {
                for (name, costs) in items.iter() {
                    // Card styling
                    let (rect, _resp) = ui.allocate_exact_size(egui::vec2(220.0, 110.0), egui::Sense::hover());
                    let painter = ui.painter();
                    painter.rect_filled(rect, egui::Rounding::same(8), egui::Color32::from_rgb(40,40,50));
                    painter.rect_stroke(
                        rect,
                        egui::Rounding::same(8),
                        egui::Stroke{width:1.0, color: egui::Color32::DARK_GRAY},
                        egui::StrokeKind::Outside,
                    );
                    // Contents
                    let mut y = rect.min.y + 8.0;
                    // Icon + name
                    if let Some(tex) = self.get_crystal_icon(ui.ctx(), name) {
                        let img_size = egui::vec2(20.0, 20.0);
                        let img_rect = egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 8.0), img_size);
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                        painter.image(tex.id(), img_rect, uv, egui::Color32::WHITE);
                    }
                    painter.text(rect.min + egui::vec2(36.0, 12.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(16.0), egui::Color32::WHITE);
                    y += 28.0;
                    // Costs with icons per required crystal/resource
                    let mut x = rect.min.x + 8.0;
                    let icon_size = egui::vec2(16.0, 16.0);
                    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    for (req, amt) in costs.iter() {
                        if req == "Soul" || req == "Souls" { continue; }
                        // Try an icon for the requirement (crystal). For Vis/Soul a text fallback is used.
                        if let Some(tex) = self.get_crystal_icon(ui.ctx(), req) {
                            let img_rect = egui::Rect::from_min_size(egui::pos2(x, y), icon_size);
                            painter.image(tex.id(), img_rect, uv, egui::Color32::WHITE);
                            x += icon_size.x + 4.0;
                            painter.text(egui::pos2(x, y + 2.0), egui::Align2::LEFT_TOP, format!("x{}", amt), egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
                            x += 28.0; // space before next requirement
                        } else {
                            // No icon: show text "req xamt"
                            let label = format!("{} x{}", req, amt);
                            painter.text(egui::pos2(x, y + 2.0), egui::Align2::LEFT_TOP, &label, egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
                            x += (label.len() as f32) * 7.5 + 12.0;
                        }
                    }
                }
            });
            ui.separator();
        }
    }

    fn can_unlock(&self, id: &str) -> bool {
        let node = match self.skills.iter().find(|n| n.id == id) {
            Some(n) => n,
            None => return false,
        };
        if node.unlocked {
            return false;
        }
        // All prerequisites must be unlocked
        node.prerequisites.iter().all(|pre_id| {
            self.skills.iter().any(|n| n.id == *pre_id && n.unlocked)
        })
    }

    fn unlock_skill(&mut self, id: &str) -> Result<UnlockOutcome, UnlockError> {
        // Locate node index for atomic mutation
        let idx = self
            .skills
            .iter()
            .position(|n| n.id == id)
            .ok_or(UnlockError::NotFound)?;

        if self.skills[idx].unlocked {
            return Err(UnlockError::AlreadyUnlocked);
        }

        // Verify prerequisites; collect any missing for better diagnostics
        let missing: Vec<String> = self.skills[idx]
            .prerequisites
            .iter()
            .filter(|pre_id| !self.skills.iter().any(|n| n.id == **pre_id && n.unlocked))
            .map(|s| s.to_string())
            .collect();
        if !missing.is_empty() {
            return Err(UnlockError::PrerequisitesMissing(missing));
        }

        // Handle costs atomically before mutating state
        match id {
            "essence_control" => {
                let needed = 50;
                if self.vis < needed {
                    return Err(UnlockError::InsufficientVis { needed, have: self.vis });
                }
                self.vis -= needed;
                self.skills[idx].unlocked = true;
                self.unlocks.secondary_crystals = true;
            }
            _ => {
                self.skills[idx].unlocked = true;
            }
        }

        Ok(UnlockOutcome::Unlocked)
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
        // Souls removed

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
                    *val += self.crystalClickAmount;
                } else {
                    // In case save didn't have the key, insert it
                    self.crystals.insert(chosen.to_string(), 1);
                }
            }
        }

        // Secondary crystals crafting (cards)
        if self.unlocks.secondary_crystals {
            ui.separator();
            ui.label(egui::RichText::new("secondary crystals").color(egui::Color32::LIGHT_BLUE));
            if let Some(items) = self.recipes.crystals.get("secondary").cloned() {
                let gallery: Vec<(String, IndexMap<String, u32>)> = items.into_iter().collect();
                ui.horizontal_wrapped(|ui| {
                    for (name, costs) in gallery.iter() {
                        // affordability
                        let mut can_afford = true;
                        for (req, amt) in costs.iter() {
                            if self.crystals.get(req.as_str()).copied().unwrap_or(0) < *amt {
                                can_afford = false; break;
                            }
                        }
                        // Card
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(220.0, 110.0), egui::Sense::click());
                        let painter = ui.painter();
                        let bg = if can_afford { egui::Color32::from_rgb(40,50,60) } else { egui::Color32::from_rgb(30,30,35) };
                        painter.rect_filled(rect, egui::Rounding::same(8), bg);
                        painter.rect_stroke(rect, egui::Rounding::same(8), egui::Stroke{width:1.0, color: egui::Color32::DARK_GRAY}, egui::StrokeKind::Outside);
                        // icon + name
                        let mut y = rect.min.y + 8.0;
                        if let Some(tex) = self.get_crystal_icon(ui.ctx(), name) {
                            let img_size = egui::vec2(20.0,20.0);
                            let img_rect = egui::Rect::from_min_size(rect.min + egui::vec2(8.0,8.0), img_size);
                            let uv = egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0));
                            painter.image(tex.id(), img_rect, uv, egui::Color32::WHITE);
                        }
                        painter.text(rect.min + egui::vec2(36.0, 12.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(16.0), egui::Color32::WHITE);
                        y += 28.0;
                        // costs with icons
                        let mut x = rect.min.x + 8.0;
                        let icon_size = egui::vec2(16.0,16.0);
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0));
                        for (req, amt) in costs.iter() {
                            if let Some(tex) = self.get_crystal_icon(ui.ctx(), req) {
                                let img_rect = egui::Rect::from_min_size(egui::pos2(x,y), icon_size);
                                painter.image(tex.id(), img_rect, uv, egui::Color32::WHITE);
                                x += icon_size.x + 4.0;
                                let t = format!("x{}", amt);
                                painter.text(egui::pos2(x,y+2.0), egui::Align2::LEFT_TOP, &t, egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
                                x += 28.0;
                            } else {
                                let label = format!("{} x{}", req, amt);
                                painter.text(egui::pos2(x, y+2.0), egui::Align2::LEFT_TOP, &label, egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
                                x += (label.len() as f32)*7.5 + 12.0;
                            }
                        }
                        if can_afford && resp.clicked() {
                            // spend and craft
                            for (req, amt) in costs.iter() {
                                if let Some(entry) = self.crystals.get_mut(req.as_str()) {
                                    *entry = entry.saturating_sub(*amt);
                                }
                            }
                            *self.crystals.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                });
            }
        }

        // Tertiary crystals crafting (cards)
        if self.unlocks.tertiary_crystals {
            ui.separator();
            ui.label(egui::RichText::new("tertiary crystals").color(egui::Color32::LIGHT_BLUE));
            if let Some(items) = self.recipes.crystals.get("tertiary").cloned() {
                let gallery: Vec<(String, IndexMap<String, u32>)> = items.into_iter().collect();
                ui.horizontal_wrapped(|ui| {
                    for (name, costs) in gallery.iter() {
                        let mut can_afford = true;
                        for (req, amt) in costs.iter() {
                            if self.crystals.get(req.as_str()).copied().unwrap_or(0) < *amt { can_afford = false; break; }
                        }
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(220.0, 110.0), egui::Sense::click());
                        let painter = ui.painter();
                        let bg = if can_afford { egui::Color32::from_rgb(40,50,60) } else { egui::Color32::from_rgb(30,30,35) };
                        painter.rect_filled(rect, egui::Rounding::same(8), bg);
                        painter.rect_stroke(rect, egui::Rounding::same(8), egui::Stroke{width:1.0, color: egui::Color32::DARK_GRAY}, egui::StrokeKind::Outside);
                        let mut y = rect.min.y + 8.0;
                        if let Some(tex) = self.get_crystal_icon(ui.ctx(), name) {
                            let img_size = egui::vec2(20.0,20.0);
                            let img_rect = egui::Rect::from_min_size(rect.min + egui::vec2(8.0,8.0), img_size);
                            let uv = egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0));
                            painter.image(tex.id(), img_rect, uv, egui::Color32::WHITE);
                        }
                        painter.text(rect.min + egui::vec2(36.0, 12.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(16.0), egui::Color32::WHITE);
                        y += 28.0;
                        let mut x = rect.min.x + 8.0;
                        let icon_size = egui::vec2(16.0,16.0);
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0));
                        for (req, amt) in costs.iter() {
                            if let Some(tex) = self.get_crystal_icon(ui.ctx(), req) {
                                let img_rect = egui::Rect::from_min_size(egui::pos2(x,y), icon_size);
                                painter.image(tex.id(), img_rect, uv, egui::Color32::WHITE);
                                x += icon_size.x + 4.0;
                                let t = format!("x{}", amt);
                                painter.text(egui::pos2(x,y+2.0), egui::Align2::LEFT_TOP, &t, egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
                                x += 28.0;
                            } else {
                                let label = format!("{} x{}", req, amt);
                                painter.text(egui::pos2(x, y+2.0), egui::Align2::LEFT_TOP, &label, egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
                                x += (label.len() as f32)*7.5 + 12.0;
                            }
                        }
                        if can_afford && resp.clicked() {
                            for (req, amt) in costs.iter() { if let Some(e) = self.crystals.get_mut(req.as_str()) { *e = e.saturating_sub(*amt); } }
                            *self.crystals.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                });
            }
        }

        // Quaternary crystals crafting (cards)
        if self.unlocks.quaternary_crystals {
            ui.separator();
            ui.label(egui::RichText::new("quaternary crystals").color(egui::Color32::LIGHT_BLUE));
            if let Some(items) = self.recipes.crystals.get("quaternary").cloned() {
                let gallery: Vec<(String, IndexMap<String, u32>)> = items.into_iter().collect();
                ui.horizontal_wrapped(|ui| {
                    for (name, costs) in gallery.iter() {
                        let mut can_afford = true;
                        for (req, amt) in costs.iter() { if self.crystals.get(req.as_str()).copied().unwrap_or(0) < *amt { can_afford = false; break; } }
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(220.0, 110.0), egui::Sense::click());
                        let painter = ui.painter();
                        let bg = if can_afford { egui::Color32::from_rgb(40,50,60) } else { egui::Color32::from_rgb(30,30,35) };
                        painter.rect_filled(rect, egui::Rounding::same(8), bg);
                        painter.rect_stroke(rect, egui::Rounding::same(8), egui::Stroke{width:1.0, color: egui::Color32::DARK_GRAY}, egui::StrokeKind::Outside);
                        let mut y = rect.min.y + 8.0;
                        if let Some(tex) = self.get_crystal_icon(ui.ctx(), name) {
                            let img_size = egui::vec2(20.0,20.0);
                            let img_rect = egui::Rect::from_min_size(rect.min + egui::vec2(8.0,8.0), img_size);
                            let uv = egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0));
                            painter.image(tex.id(), img_rect, uv, egui::Color32::WHITE);
                        }
                        painter.text(rect.min + egui::vec2(36.0, 12.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(16.0), egui::Color32::WHITE);
                        y += 28.0;
                        let mut x = rect.min.x + 8.0;
                        let icon_size = egui::vec2(16.0,16.0);
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0));
                        for (req, amt) in costs.iter() {
                            if let Some(tex) = self.get_crystal_icon(ui.ctx(), req) {
                                let img_rect = egui::Rect::from_min_size(egui::pos2(x,y), icon_size);
                                painter.image(tex.id(), img_rect, uv, egui::Color32::WHITE);
                                x += icon_size.x + 4.0;
                                let t = format!("x{}", amt);
                                painter.text(egui::pos2(x,y+2.0), egui::Align2::LEFT_TOP, &t, egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
                                x += 28.0;
                            } else {
                                let label = format!("{} x{}", req, amt);
                                painter.text(egui::pos2(x, y+2.0), egui::Align2::LEFT_TOP, &label, egui::FontId::proportional(14.0), egui::Color32::LIGHT_GRAY);
                                x += (label.len() as f32)*7.5 + 12.0;
                            }
                        }
                        if can_afford && resp.clicked() {
                            for (req, amt) in costs.iter() { if let Some(e) = self.crystals.get_mut(req.as_str()) { *e = e.saturating_sub(*amt); } }
                            *self.crystals.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                });
            }
        }
    }

    fn show_upgrades(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Upgrades Menu").color(egui::Color32::WHITE));
        ui.label(egui::RichText::new("Purchase upgrades to enhance clicks or crafting.").color(egui::Color32::WHITE));

        // Upgrade 1 removed (used souls)

        // Upgrade 2: Vis Capacity
        if ui.add_enabled(self.vis >= 50, styled_button("Upgrade Vis Capacity (+50) (50 Vis)")).clicked() {
            if safe_subtract(&mut self.vis, 50) {
                self.maxVis += 50;
            }
        }

        // Upgrade 3: Crystal Click Amount
        if ui.add_enabled(self.vis >= 200, styled_button("Upgrade Crystal Click Amount (+1) (200 Vis)")).clicked() {
            if safe_subtract(&mut self.vis, 200) {
                self.crystalClickAmount += 1;
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
        ui.separator();
        if ui.add(styled_button("Save Game")).clicked() {
            let _ = save_game(self);
        }
        if ui.add(styled_button("Save and Exit")).clicked() {
            let _ = save_game(self);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
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
        // autosave every 60 seconds
        let dt = ctx.input(|i| i.unstable_dt);
        self.autosave_timer += dt;
        if self.autosave_timer >= 60.0 {
            let _ = save_game(self);
            self.autosave_timer = 0.0;
        }

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
                // Avoid borrowing self immutably while calling a mutable method
                let crystal_list: Vec<(String, u32)> = self
                    .crystals
                    .iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
                for (crystal, amount) in crystal_list {
                    ui.horizontal(|ui| {
                        if let Some(tex) = self.get_crystal_icon(ui.ctx(), &crystal) {
                            ui.add(egui::Image::new((tex.id(), egui::vec2(18.0, 18.0))));
                        }
                        ui.label(
                            egui::RichText::new(format!("{}: {}", crystal, amount))
                                .color(egui::Color32::WHITE),
                        );
                    });
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
                    MenuTab::Thauminomicon => self.show_research_book(ui),
                    MenuTab::Equipment => self.show_equipment(ui),
                    MenuTab::Achievements => self.show_achievements(ui),
                    MenuTab::Settings => self.show_settings(ui),
                }
            });
    }
}








