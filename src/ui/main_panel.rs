use eframe::egui;
use crate::scanner::FileSystemScanner;
use std::path::PathBuf;
use rfd::FileDialog;

#[derive(Default)]
pub struct MainPanel {
    pub show_settings: bool,
    pub show_about: bool,
    pub dark_mode: bool,
    scan_path_input: String,
    error_message: Option<String>,
}

impl MainPanel {
    pub fn new() -> Self {
        let default_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .to_string_lossy()
            .to_string();
        Self {
            show_settings: false,
            show_about: false,
            dark_mode: true,
            scan_path_input: default_path,
            error_message: None,
        }
    }

    pub fn render_menu(
        &mut self,
        ui: &mut egui::Ui,
        is_scanning: &mut bool,
        scanner: &mut Option<FileSystemScanner>,
        _selected_path: &mut Option<PathBuf>,
        current_path: &mut PathBuf,
    ) {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("📁 Select Directory...").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.scan_path_input = path.to_string_lossy().to_string();
                        *current_path = path;
                        self.error_message = None;
                    }
                    ui.close();
                }
                if ui.button("📁 Scan Selected Directory").clicked() {
                    let path = PathBuf::from(&self.scan_path_input);
                    if path.exists() {
                        *current_path = path.clone();
                        *scanner = Some(FileSystemScanner::new(path));
                        scanner.as_mut().unwrap().start();
                        *is_scanning = true;
                        self.error_message = None;
                    } else {
                        self.error_message = Some("Path does not exist".to_string());
                    }
                    ui.close();
                }
                if ui.button("📊 Export Report...").clicked() {
                    // TODO
                    ui.close();
                }
                ui.separator();
                if ui.button("🚪 Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.dark_mode, "Dark Mode");
                if self.dark_mode {
                    ui.ctx().set_visuals(egui::Visuals::dark());
                } else {
                    ui.ctx().set_visuals(egui::Visuals::light());
                }
                if ui.button("Reset Layout").clicked() {}
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("🔍 Find Large Files").clicked() {}
                if ui.button("🔄 Find Duplicates").clicked() {}
                if ui.button("🗑️ Cleanup Suggestions").clicked() {}
            });

            ui.menu_button("Help", |ui| {
                if ui.button("📚 Documentation").clicked() {}
                if ui.button("ℹ️ About DiskRay").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙️").clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
        });

        // Path selection panel
        ui.horizontal(|ui| {
            ui.label("Scan path:");
            let response = ui.add(egui::TextEdit::singleline(&mut self.scan_path_input)
                .desired_width(ui.available_width() * 0.7)
                .hint_text("Enter path or click Browse..."));
            if response.changed() {
                self.error_message = None;
            }

            if ui.button("📁 Browse...").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.scan_path_input = path.to_string_lossy().to_string();
                    *current_path = path;
                    self.error_message = None;
                }
            }

            let scan_clicked = ui.button("▶️ Scan").clicked();
            if scan_clicked && !self.scan_path_input.trim().is_empty() {
                let path = PathBuf::from(&self.scan_path_input);
                if path.exists() {
                    *current_path = path.clone();
                    *scanner = Some(FileSystemScanner::new(path));
                    scanner.as_mut().unwrap().start();
                    *is_scanning = true;
                    self.error_message = None;
                } else {
                    self.error_message = Some("Path does not exist".to_string());
                }
            }

            if ui.button("⏹️ Stop").clicked() && *is_scanning {
                if let Some(scanner) = scanner {
                    scanner.stop();
                    *is_scanning = false;
                }
            }
        });

        // Quick disk selection buttons
        ui.horizontal(|ui| {
            ui.label("Quick scan:");
            use sysinfo::Disks;
            let disks = Disks::new_with_refreshed_list();
            for disk in disks.list() {
                let mount_point = disk.mount_point().to_string_lossy().to_string();
                let name = disk.name().to_string_lossy().to_string();
                let button_text = if name.is_empty() { mount_point.clone() } else { name };
                if ui.button(button_text).clicked() {
                    let path = PathBuf::from(&mount_point);
                    if path.exists() {
                        self.scan_path_input = mount_point.clone();
                        *current_path = path.clone();
                        *scanner = Some(FileSystemScanner::new(path));
                        scanner.as_mut().unwrap().start();
                        *is_scanning = true;
                        self.error_message = None;
                    }
                }
            }
            if let Some(home) = dirs::home_dir() {
                if ui.button("🏠 Home").clicked() {
                    self.scan_path_input = home.to_string_lossy().to_string();
                    *current_path = home.clone();
                    *scanner = Some(FileSystemScanner::new(home));
                    scanner.as_mut().unwrap().start();
                    *is_scanning = true;
                    self.error_message = None;
                }
            }
            if let Some(desktop) = dirs::desktop_dir() {
                if ui.button("🖥️ Desktop").clicked() {
                    self.scan_path_input = desktop.to_string_lossy().to_string();
                    *current_path = desktop.clone();
                    *scanner = Some(FileSystemScanner::new(desktop));
                    scanner.as_mut().unwrap().start();
                    *is_scanning = true;
                    self.error_message = None;
                }
            }
        });

        if let Some(ref err) = self.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }

        if self.show_settings { self.render_settings(ui.ctx()); }
        if self.show_about { self.render_about(ui.ctx()); }
    }

    pub fn render_status(
        &self,
        ui: &mut egui::Ui,
        current_path: &PathBuf,
        scan_result: std::sync::Arc<parking_lot::RwLock<Option<crate::scanner::ScanResult>>>,
        is_scanning: bool,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("📁 Current path: {}", current_path.display()));
            ui.separator();

            if is_scanning {
                ui.spinner();
                ui.label("Scanning...");
            } else if let Some(scan_result) = &*scan_result.read() {
                ui.label(format!(
                    "📊 Files: {}, Size: {}",
                    scan_result.file_count,
                    humansize::format_size(scan_result.total_size, humansize::DECIMAL)
                ));
                ui.separator();
                ui.label(format!(
                    "⏱️ Scan time: {:.2}s",
                    scan_result.scan_duration.as_secs_f32()
                ));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("DiskRay v0.3.0");
            });
        });
    }

    fn render_settings(&mut self, ctx: &egui::Context) {
        let mut settings_open = self.show_settings;
        let response = egui::Window::new("Settings")
            .open(&mut settings_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("Theme:");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.dark_mode, true, "Dark");
                            ui.radio_value(&mut self.dark_mode, false, "Light");
                        });
                        ui.end_row();
                        ui.label("Update UI theme:");
                        if ui.button("Apply Theme").clicked() {
                            if self.dark_mode {
                                ctx.set_visuals(egui::Visuals::dark());
                            } else {
                                ctx.set_visuals(egui::Visuals::light());
                            }
                        }
                        ui.end_row();
                    });
                ui.separator();
                let mut should_close = false;
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() { should_close = true; }
                    if ui.button("Cancel").clicked() { should_close = true; }
                });
                should_close
            });
        if response.is_some() {
            self.show_settings = settings_open;
            if let Some(response) = response {
                if let Some(should_close) = response.inner {
                    if should_close { self.show_settings = false; }
                }
            }
        }
    }

    fn render_about(&mut self, ctx: &egui::Context) {
        let mut about_open = self.show_about;
        let response = egui::Window::new("About DiskRay")
            .open(&mut about_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("DiskRay");
                    ui.label("Advanced Disk Space Analyzer");
                    ui.add_space(10.0);
                    ui.label("Version 0.3.0");
                    ui.label("Built with Rust and egui");
                    ui.add_space(20.0);
                    ui.hyperlink("https://github.com/yourusername/diskray");
                    ui.add_space(20.0);
                    ui.label("© 2024 Your Name");
                });
                ui.separator();
                let mut should_close = false;
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() { should_close = true; }
                });
                should_close
            });
        if response.is_some() {
            self.show_about = about_open;
            if let Some(response) = response {
                if let Some(should_close) = response.inner {
                    if should_close { self.show_about = false; }
                }
            }
        }
    }
}