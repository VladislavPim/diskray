#![windows_subsystem = "windows"]

use eframe::egui;
use diskray::app::DiskRayApp;

fn load_icon() -> Option<egui::IconData> {
    // Встраиваем файл иконки в бинарник
    let icon_bytes = include_bytes!("../assets/diskray.ico");
    // Загружаем изображение из памяти
    let image = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn main() -> Result<(), eframe::Error> {
    let icon = load_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1400.0, 900.0])
        .with_min_inner_size([800.0, 600.0])
        .with_resizable(true)
        .with_title("DiskRay - Disk Space Analyzer");

    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "DiskRay",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(DiskRayApp::new()))
        }),
    )
}