use eframe::egui;
use crate::scanner::{FileSystemScanner, ScanResult};
use crate::analyzer::DiskAnalyzer;
use crate::ui::{MainPanel, TreePanel, DisksPanel};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct DiskRayApp {
    pub scanner: Option<FileSystemScanner>,
    pub scan_result: Arc<RwLock<Option<ScanResult>>>,
    pub analyzer: DiskAnalyzer,
    pub main_panel: MainPanel,
    pub tree_panel: TreePanel,
    pub disks_panel: DisksPanel,
    pub current_path: PathBuf,
    pub is_scanning: bool,
    pub selected_path: Option<PathBuf>,
    pub selected_entry_details: Option<crate::scanner::FileEntry>,
}

impl DiskRayApp {
    pub fn new() -> Self {
        Self {
            scanner: None,
            scan_result: Arc::new(RwLock::new(None)),
            analyzer: DiskAnalyzer::new(),
            main_panel: MainPanel::new(),
            tree_panel: TreePanel::new(),
            disks_panel: DisksPanel::new(),
            current_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            is_scanning: false,
            selected_path: None,
            selected_entry_details: None,
        }
    }

    fn update_selected_entry(&mut self) {
        if let Some(ref path) = self.selected_path {
            if let Some(scan_result) = &*self.scan_result.read() {
                self.selected_entry_details = scan_result.entries.iter()
                    .find(|e| e.path == *path)
                    .cloned();
                return;
            }
        }
        self.selected_entry_details = None;
    }

    fn render_extensions_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Extensions");
        ui.separator();

        if let Some(entry) = &self.selected_entry_details {
            ui.label(format!("Selected: {}", entry.name));
            ui.label(format!("Size: {}", humansize::format_size(entry.size, humansize::DECIMAL)));
            ui.label(format!("Modified: {}", entry.modified.format("%Y-%m-%d %H:%M")));
            if entry.is_directory {
                ui.label(format!("Contains {} items", entry.children.len()));
            }
            ui.separator();
        }

        if let Some(scan_result) = &*self.scan_result.read() {
            let stats = self.analyzer.get_extension_stats(scan_result);
            let total_size = scan_result.total_size;

            ui.label(format!("Total files: {}", scan_result.file_count));
            ui.label(format!("Total size: {}", humansize::format_size(total_size, humansize::DECIMAL)));
            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for stat in stats.iter().take(30) {
                    ui.horizontal(|ui| {
                        ui.label(&stat.extension);
                        let percent = stat.total_size as f64 / total_size as f64 * 100.0;
                        ui.add(egui::ProgressBar::new(percent as f32 / 100.0).desired_width(100.0));
                        ui.label(format!("{:.1}%", percent));
                        ui.label(format!("({})", humansize::format_size(stat.total_size, humansize::DECIMAL)));
                    });
                }
            });
        } else {
            ui.label("No scan data");
        }
    }
}

impl eframe::App for DiskRayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_scanning {
            ctx.request_repaint();
        }
        self.update_scanning(ctx);
        self.update_selected_entry();
        self.render_ui(ctx);
    }
}

impl DiskRayApp {
    fn update_scanning(&mut self, ctx: &egui::Context) {
        if self.is_scanning {
            if let Some(scanner) = &mut self.scanner {
                if scanner.is_finished() {
                    self.is_scanning = false;
                    if let Some(result) = scanner.take_result() {
                        self.analyzer.analyze(&result);
                        self.tree_panel.set_scan_result(Some(result.clone()));
                        *self.scan_result.write() = Some(result);
                        ctx.request_repaint(); // принудительно обновить UI
                    } else {
                        self.tree_panel.set_scan_result(None);
                    }
                }
            }
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.main_panel.render_menu(
                ui,
                &mut self.is_scanning,
                &mut self.scanner,
                &mut self.selected_path,
                &mut self.current_path,
            );
        });

        egui::TopBottomPanel::top("disks_panel").show(ctx, |ui| {
            self.disks_panel.render(ui);
        });

        egui::SidePanel::right("extensions_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                self.render_extensions_panel(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.tree_panel.render(
                ui,
                &mut self.selected_path,
                &mut self.scanner,
                &mut self.is_scanning,
                &mut self.current_path,
            );
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.main_panel.render_status(
                ui,
                &self.current_path,
                self.scan_result.clone(),
                self.is_scanning,
            );
        });
    }
}