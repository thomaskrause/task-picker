use std::path::PathBuf;

use egui::CentralPanel;
use predicates::{
    path::{eq_file, BinaryFilePredicate},
    prelude::*,
};
use serde::Serialize;
use skia_safe::Surface;

use super::*;

#[derive(Serialize)]
struct Info {
    actual_file: PathBuf,
}

fn eq_screenshot(expected_file_name: &str, surface: &mut Surface) -> BinaryFilePredicate {
    let mut output_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    output_file.push("src/app/tests/expected");
    output_file.push(expected_file_name);
    if std::env::var("UPDATE_EXPECT").is_ok() {
        // Write current snapshot to to expected path
        let image = surface.image_snapshot();
        let data = image
            .encode_to_data(skia_safe::EncodedImageFormat::PNG)
            .unwrap();

        std::fs::write(&output_file, data.as_bytes()).unwrap();
    }

    // Compare with the expected file
    eq_file(&output_file)
}

fn assert_screenshot(expected_file_name: &str, window_size: (i32, i32), ctx: impl FnMut(&Context)) {
    let mut surface = egui_skia::rasterize(window_size, ctx, None);
    let p = eq_screenshot(expected_file_name, &mut surface);

    assert_eq!(
        true,
        p.eval(
            surface
                .image_snapshot()
                .encode_to_data(skia_safe::EncodedImageFormat::PNG)
                .unwrap()
                .as_bytes()
        )
    )
}

#[test]
fn test_render_single_task() {
    assert_screenshot("single_task.png", (250, 300), |ctx| {
        CentralPanel::default().show(ctx, |ui| {
            let mut app = TaskPickerApp::default();

            let task = Task {
                project: "project".to_string(),
                title: "Any task".to_string(),
                description: "Has a description".to_string(),
                due: None,
                created: None,
                id: None,
            };

            app.render_single_task(ui, task);
        });
    });
}
