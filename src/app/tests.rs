use std::{path::PathBuf, sync::Once, vec};

use chrono::Days;
use egui::{CentralPanel, Pos2};
use egui_skia::{EguiSkia, RasterizeOptions};
use mockall::predicate::*;
use serde::Serialize;
use skia_safe::Surface;
use visual_hash::HasherConfig;

use super::*;

#[derive(Serialize)]
struct Info {
    actual_file: PathBuf,
}

fn assert_eq_screenshot(expected_file_name: &str, surface: &mut Surface) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let mut output_file_rel = PathBuf::from("src/app/tests/expected");
    output_file_rel.push(expected_file_name);
    let output_file = manifest_dir.join(&output_file_rel);

    // Write out the screenshot to a file that is removed if test ist successful
    let mut actual_file_rel = PathBuf::from("src/app/tests/actual");
    actual_file_rel.push(expected_file_name);

    let actual_file = manifest_dir.join(&actual_file_rel);
    std::fs::create_dir_all(&actual_file.parent().unwrap()).unwrap();

    let actual_image_skia = surface.image_snapshot();
    let skia_data = actual_image_skia
        .encode_to_data(skia_safe::EncodedImageFormat::PNG)
        .unwrap();
    std::fs::write(&actual_file, skia_data.as_bytes()).unwrap();

    if std::env::var("UPDATE_EXPECT").is_ok() {
        // Write current snapshot to to expected path
        let data = actual_image_skia
            .encode_to_data(skia_safe::EncodedImageFormat::PNG)
            .unwrap();
        std::fs::write(&output_file, data.as_bytes()).unwrap();
    }

    // Read in expected image from file
    let expected_image = image::io::Reader::open(&output_file)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    let actual_image = image::io::Reader::open(&actual_file)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    // Compare images using a visual hash
    let hasher = HasherConfig::default().to_hasher();
    let expected_hash = hasher.hash_image(&expected_image);
    let actual_hash = hasher.hash_image(&actual_image);

    let dist = actual_hash.dist(&expected_hash);
    assert!(
        dist == 0,
        "{} != {}",
        actual_file_rel.to_string_lossy(),
        output_file_rel.to_string_lossy(),
    );

    // Remove the created file
    std::fs::remove_file(actual_file).unwrap();
}

static INIT: Once = Once::new();

pub struct TestBackend {
    backend: EguiSkia,
    pub app: TaskPickerApp,
}

impl TestBackend {
    pub fn new(app: TaskPickerApp) -> Self {
        INIT.call_once(|| std::env::set_var("TZ", "CET"));
        let backend = EguiSkia::default();
        app.init_with_egui_context(&backend.egui_ctx);
        TestBackend { backend, app }
    }

    pub fn assert_screenshot_after_n_frames(
        &mut self,
        expected_file_name: &str,
        window_size: (i32, i32),
        n: usize,
    ) {
        let RasterizeOptions { pixels_per_point } = Default::default();

        let mut surface =
            Surface::new_raster_n32_premul(window_size).expect("Failed to create surface");
        let input = egui::RawInput {
            screen_rect: Some(
                [
                    Pos2::default(),
                    Pos2::new(surface.width() as f32, surface.height() as f32),
                ]
                .into(),
            ),
            pixels_per_point: Some(pixels_per_point),
            ..Default::default()
        };

        for _ in 0..n {
            self.backend.run(input.clone(), |ctx| {
                CentralPanel::default().show(ctx, |_ui| {
                    self.app.render(ctx);
                });
            });
        }

        self.backend.paint(surface.canvas());
        assert_eq_screenshot(expected_file_name, &mut surface);
    }
}

#[test]
fn test_render_single_task_with_description() {
    let now = Utc.with_ymd_and_hms(2022, 03, 19, 17, 42, 00).unwrap();
    let mut app = TaskPickerApp::default();

    app.settings.dark_mode = true;
    app.overwrite_current_time = Some(now);

    let task = Task {
        project: format!("{} family", CALDAV_ICON),
        title: "Buy presents".to_string(),
        description: "They should be surprising.\n\nBut not that surprising!".to_string(),
        due: Some(Utc.with_ymd_and_hms(2022, 12, 24, 20, 0, 0).unwrap()),
        created: Some(Utc.with_ymd_and_hms(2022, 09, 1, 12, 24, 30).unwrap()),
        id: None,
    };
    app.task_manager.expect_tasks().return_const(vec![task]);
    app.task_manager.expect_sources().return_const(vec![]);
    app.task_manager.expect_refresh().return_const(());

    let mut backend = TestBackend::new(app);
    backend.assert_screenshot_after_n_frames("single_task_with_description.png", (800, 600), 5);
}

#[test]
fn test_render_task_grid() {
    let now = Utc.with_ymd_and_hms(2023, 03, 19, 17, 42, 00).unwrap();
    let mut app = TaskPickerApp::default();
    app.settings.dark_mode = false;
    app.overwrite_current_time = Some(now);

    let task_relaxed = Task {
        project: "project".to_string(),
        title: "Far away".to_string(),
        description: "http://example.com".to_string(),
        due: now.checked_add_days(Days::new(20)),
        created: now.checked_sub_days(Days::new(10)),
        id: Some("task_relaxed".to_string()),
    };
    let task_due_tomorrow = Task {
        project: "project".to_string(),
        title: "Due Tomorrow".to_string(),
        description: "http://example.com".to_string(),
        due: Some(Utc.with_ymd_and_hms(2023, 03, 20, 20, 42, 00).unwrap()),
        created: now.checked_sub_days(Days::new(10)),
        id: Some("task_due_tomorrow".to_string()),
    };

    let task_due_today = Task {
        project: "project".to_string(),
        title: "Due Today".to_string(),
        description: "http://example.com".to_string(),
        due: Some(Utc.with_ymd_and_hms(2023, 03, 19, 19, 42, 00).unwrap()),
        created: now.checked_sub_days(Days::new(10)),
        id: Some("task_due_today".to_string()),
    };

    let tasks = vec![task_due_today, task_due_tomorrow, task_relaxed];

    app.task_manager.expect_tasks().return_const(tasks);
    app.task_manager.expect_sources().return_const(vec![]);
    app.task_manager.expect_refresh().return_const(());

    let mut backend = TestBackend::new(app);
    backend.assert_screenshot_after_n_frames("task_grid.png", (800, 600), 2);
}
