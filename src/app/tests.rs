use std::{path::PathBuf, sync::Once, vec};

use chrono::Days;
use egui_screenshot_testing::TestBackend;
use serde::Serialize;

use super::*;

#[derive(Serialize)]
struct Info {
    actual_file: PathBuf,
}

static INIT: Once = Once::new();

#[test]
fn test_render_single_task_with_description() {
    INIT.call_once(|| std::env::set_var("TZ", "CET"));
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

    let mut backend = TestBackend::new("src/app/tests/expected", "src/app/tests/actual", |_ctx| {});
    backend.assert_screenshot_after_n_frames(
        "single_task_with_description.png",
        (800, 600),
        5,
        |ctx| {
            app.render(ctx);
        },
    );
}

#[test]
fn test_render_task_grid() {
    INIT.call_once(|| std::env::set_var("TZ", "CET"));
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

    let mut backend = TestBackend::new("src/app/tests/expected", "src/app/tests/actual", |ctx| {
        app.init_with_egui_context(ctx)
    });
    backend.assert_screenshot_after_n_frames("task_grid.png", (800, 600), 2, |ctx| {
        app.render(ctx);
    });
}
