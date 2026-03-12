use eframe::egui;
use sysinfo::Disks;
use humansize::{format_size, DECIMAL};

pub struct DisksPanel {
    disks_info: Vec<DiskInfo>,
    last_update: std::time::Instant,
    update_interval: f32,
}

#[derive(Clone)]
struct DiskInfo {
    name: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
    used_space: u64,
    usage_percent: f32,
    disk_type: String,
    file_system: String,
    is_removable: bool,
}

impl DisksPanel {
    pub fn new() -> Self {
        let mut panel = Self {
            disks_info: Vec::new(),
            last_update: std::time::Instant::now(),
            update_interval: 2.0,
        };
        panel.update_disks_info();
        panel
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_update).as_secs_f32() >= self.update_interval {
            self.update_disks_info();
            self.last_update = now;
        }

        ui.heading("💽 Disks Information");
        ui.add_space(5.0);

        if self.disks_info.is_empty() {
            ui.label("No disks found.");
            return;
        }

        let total_space: u64 = self.disks_info.iter().map(|d| d.total_space).sum();
        let total_used: u64 = self.disks_info.iter().map(|d| d.used_space).sum();
        let total_available: u64 = self.disks_info.iter().map(|d| d.available_space).sum();

        ui.horizontal(|ui| {
            ui.label(format!("Total: {}", format_size(total_space, DECIMAL)));
            ui.separator();
            ui.colored_label(egui::Color32::LIGHT_RED, format!("Used: {}", format_size(total_used, DECIMAL)));
            ui.separator();
            ui.colored_label(egui::Color32::LIGHT_GREEN, format!("Available: {}", format_size(total_available, DECIMAL)));
        });

        ui.separator();

        egui::Grid::new("disks_grid")
            .num_columns(7)
            .striped(true)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("Drive"); ui.label("Mount Point"); ui.label("Type"); ui.label("FS"); ui.label("Total"); ui.label("Used"); ui.label("Usage"); ui.end_row();

                for disk in &self.disks_info {
                    let icon = if disk.is_removable { "💾" } else if disk.disk_type == "SSD" { "⚡" } else { "💽" };
                    ui.horizontal(|ui| { ui.label(icon); ui.label(&disk.name); });
                    ui.label(&disk.mount_point);
                    ui.label(&disk.disk_type);
                    ui.label(&disk.file_system);
                    ui.label(format_size(disk.total_space, DECIMAL));
                    ui.label(format_size(disk.used_space, DECIMAL));
                    let usage_color = if disk.usage_percent > 90.0 { egui::Color32::RED } else if disk.usage_percent > 75.0 { egui::Color32::YELLOW } else { egui::Color32::GREEN };
                    ui.horizontal(|ui| {
                        ui.add(egui::ProgressBar::new(disk.usage_percent / 100.0).desired_width(80.0).fill(usage_color));
                        ui.colored_label(usage_color, format!("{:.1}%", disk.usage_percent));
                    });
                    ui.end_row();
                }
            });
    }

    fn update_disks_info(&mut self) {
        self.disks_info.clear();
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let percent = if total > 0 { (used as f64 / total as f64 * 100.0) as f32 } else { 0.0 };
            let kind = match disk.kind() {
                sysinfo::DiskKind::SSD => "SSD",
                sysinfo::DiskKind::HDD => "HDD",
                _ => "Unknown",
            };
            self.disks_info.push(DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent: percent,
                disk_type: kind.to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                is_removable: disk.is_removable(),
            });
        }
        self.disks_info.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    }
}