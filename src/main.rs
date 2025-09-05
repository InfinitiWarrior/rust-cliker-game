use eframe::egui;

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

struct Clicker {
    essence: u32,
}

impl Default for Clicker {
    fn default() -> Self {
        Self { essence: 0 }
    }
}

impl eframe::App for Clicker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(egui::RichText::new(format!("essence: {}", self.essence)).color(egui::Color32::WHITE));

            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Conjure resources").color(egui::Color32::BLACK),
                    )
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
                    .min_size([120.0, 50.0].into()),
                )
                .clicked()
            {
                self.essence += 1;
            }
        });
    }
}


