use eframe::egui;

/// Chart view panel
pub struct ChartPanel {}

impl ChartPanel {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("📊 Chart View");
            ui.label("This feature is under development");
            ui.add_space(20.0);
            ui.label("Coming soon: Treemap visualization of disk usage");
        });
    }
}