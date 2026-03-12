use eframe::egui;
use crate::scanner::FileEntry;
use crate::scanner::FileSystemScanner;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::scanner::ScanResult;

pub struct TreePanel {
    expanded_dirs: HashMap<PathBuf, bool>,
    scan_result: Option<ScanResult>,
}

impl TreePanel {
    pub fn new() -> Self {
        Self {
            expanded_dirs: HashMap::new(),
            scan_result: None,
        }
    }

    pub fn set_scan_result(&mut self, result: Option<ScanResult>) {
        self.scan_result = result;
        self.expanded_dirs.clear();
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        selected_path: &mut Option<PathBuf>,
        scanner: &mut Option<FileSystemScanner>,
        is_scanning: &mut bool,
        current_path: &mut PathBuf,
    ) {
        egui::TopBottomPanel::top("tree_panel_header")
            .exact_height(40.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("🌳 File Tree");
                    ui.label(format!("Path: {}", current_path.display()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::TextEdit::singleline(&mut String::new())
                            .hint_text("Search...")
                            .desired_width(200.0));
                    });
                });
            });

        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                // Разделяем заимствования: берём ссылку на scan_result и мутабельную ссылку на expanded_dirs
                let scan_result_ref = self.scan_result.as_ref();
                let expanded_dirs = &mut self.expanded_dirs;

                if let Some(scan_result) = scan_result_ref {
                    let entry_map: HashMap<_, _> = scan_result.entries
                        .iter()
                        .map(|e| (e.path.clone(), e))
                        .collect();

                    let root_entries: Vec<&FileEntry> = scan_result.entries
                        .iter()
                        .filter(|e| {
                            e.parent.as_ref().map_or(true, |parent| {
                                parent == &scan_result.root_path
                            })
                        })
                        .collect();

                    let total_size = scan_result.total_size;

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            for entry in root_entries {
                                Self::render_tree_node(
                                    ui,
                                    entry,
                                    &entry_map,
                                    selected_path,
                                    total_size,
                                    expanded_dirs,
                                );
                            }
                        });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading("No Scan Data");
                        ui.label("Select a directory to start analyzing disk usage");
                        ui.add_space(20.0);
                        if ui.button("📁 Scan Current Directory").clicked() {
                            *scanner = Some(FileSystemScanner::new(current_path.clone()));
                            scanner.as_mut().unwrap().start();
                            *is_scanning = true;
                        }
                    });
                }
            });
    }

    fn render_tree_node(
        ui: &mut egui::Ui,
        entry: &FileEntry,
        entry_map: &HashMap<PathBuf, &FileEntry>,
        selected_path: &mut Option<PathBuf>,
        total_size: u64,
        expanded_dirs: &mut HashMap<PathBuf, bool>,
    ) {
        let is_expanded = expanded_dirs.get(&entry.path).copied().unwrap_or(false);
        let is_selected = Some(&entry.path) == selected_path.as_ref();

        let response = ui.selectable_label(is_selected, Self::format_entry(entry, total_size));

        if response.clicked() {
            *selected_path = Some(entry.path.clone());
        }

        if response.double_clicked() && entry.is_directory {
            let new_state = !is_expanded;
            expanded_dirs.insert(entry.path.clone(), new_state);
        }

        if entry.is_directory && is_expanded {
            ui.indent(egui::Id::new(&entry.path), |ui| {
                let mut children: Vec<&FileEntry> = entry.children
                    .iter()
                    .filter_map(|path| entry_map.get(path))
                    .copied()
                    .collect();

                children.sort_by(|a, b| b.size.cmp(&a.size));

                for child in children {
                    Self::render_tree_node(
                        ui,
                        child,
                        entry_map,
                        selected_path,
                        total_size,
                        expanded_dirs,
                    );
                }
            });
        }
    }

    fn format_entry(entry: &FileEntry, total_size: u64) -> String {
        let size_str = humansize::format_size(entry.size, humansize::DECIMAL);
        let icon = if entry.is_directory { "📁" } else { "📄" };
        let percent = if total_size > 0 {
            entry.size as f64 / total_size as f64 * 100.0
        } else {
            0.0
        };
        format!("{} {} - {} ({:.1}%)", icon, entry.name, size_str, percent)
    }
}