#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::IconData;

fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let mut native_options = eframe::NativeOptions::default();

    if let Ok(icon) = image::load_from_memory_with_format(
        include_bytes!("taskpicker.png"),
        image::ImageFormat::Png,
    ) {
        let icon = icon.to_rgba8();
        let icon_data = IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_raw(),
        };
        native_options.icon_data = Some(icon_data);
    }
    eframe::run_native(
        "Task Picker",
        native_options,
        Box::new(|cc| Box::new(task_picker::TaskPickerApp::new(cc))),
    )
}
