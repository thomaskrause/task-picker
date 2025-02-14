use std::{sync::Once, vec};

use chrono::Days;
use egui_kittest::Harness;

use super::*;

static INIT: Once = Once::new();

#[test]
fn test_render_single_task_with_description() {
    INIT.call_once(|| std::env::set_var("TZ", "CET"));
    let now = Utc.with_ymd_and_hms(2022, 03, 19, 17, 42, 00).unwrap();
    let mut app = TaskPickerApp::default();

    app.overwrite_current_time = Some(now);
    app.app_version = "0.0.0".to_string();

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

    let mut harness = Harness::new(|ctx| {
        ctx.set_theme(egui::Theme::Dark);
        app.init_with_egui_context(ctx);
        app.render(ctx);
    });
    harness.set_size(Vec2::new(800.0, 600.0));
    harness.run_steps(5);
    harness.snapshot("single_task_with_description");
}

#[test]
fn test_render_task_grid() {
    INIT.call_once(|| std::env::set_var("TZ", "CET"));
    let now = Utc.with_ymd_and_hms(2023, 03, 19, 17, 42, 00).unwrap();
    let mut app = TaskPickerApp::default();

    app.overwrite_current_time = Some(now);
    app.app_version = "0.0.0".to_string();

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

    let mut harness = Harness::new(|ctx| {
        ctx.set_theme(egui::Theme::Light);
        app.init_with_egui_context(ctx);
        app.render(ctx);
    });
    harness.set_size(Vec2::new(800.0, 600.0));
    harness.run_steps(5);
    harness.snapshot("task_grid");
}
