use eframe::egui;

/// Details view panel
pub struct DetailsPanel {}

impl DetailsPanel {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("📋 Details View");
            ui.label("This feature is under development");
            ui.add_space(20.0);
            ui.label("Coming soon: Detailed file information and statistics");
        });
    }
}