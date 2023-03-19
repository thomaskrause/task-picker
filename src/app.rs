use std::time::{Duration, Instant};

use crate::{
    sources::{CalDavSource, GitHubSource, GitLabSource, TaskSource},
    tasks::{Task, TaskManager},
};
use chrono::prelude::*;
use eframe::epaint::ahash::HashSet;
use egui::{
    Color32, Context, Layout, RichText, ScrollArea, Slider, Style, TextEdit, Ui, Vec2, Visuals,
};
use egui_notify::{Toast, Toasts};
use ellipse::Ellipse;
use itertools::Itertools;
use log::error;
use minicaldav::Error;
use ureq::ErrorKind;

const BOX_WIDTH: f32 = 220.0;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct Settings {
    refresh_rate_seconds: u64,
    dark_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            refresh_rate_seconds: 15,
            dark_mode: false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TaskPickerApp {
    task_manager: TaskManager,
    selected_task: Option<String>,
    settings: Settings,
    #[serde(skip)]
    last_refreshed: Instant,
    #[serde(skip)]
    messages: Toasts,
    #[serde(skip)]
    edit_source: Option<TaskSource>,
    #[serde(skip)]
    existing_edit_source: bool,
    #[serde(skip)]
    connection_error_for_source: HashSet<String>,
}

impl Default for TaskPickerApp {
    fn default() -> Self {
        let settings = Settings::default();
        Self {
            task_manager: TaskManager::default(),
            selected_task: None,
            last_refreshed: Instant::now()
                .checked_sub(Duration::from_secs(settings.refresh_rate_seconds))
                .unwrap_or_else(Instant::now),
            settings,
            edit_source: None,
            messages: Toasts::default(),
            existing_edit_source: false,
            connection_error_for_source: HashSet::default(),
        }
    }
}

impl TaskPickerApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let app = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            TaskPickerApp::default()
        };

        if app.settings.dark_mode {
            cc.egui_ctx.set_visuals(Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(Visuals::light());
        }

        app
    }

    fn edit_source(&mut self, ctx: &egui::Context) {
        let window_title = if let Some(source) = &self.edit_source {
            format!("{} source", source.type_name())
        } else {
            "Source".to_string()
        };
        egui::Window::new(window_title).show(ctx, |ui| {
            if let Some(source) = &mut self.edit_source {
                match source {
                    TaskSource::CalDav(source) => {
                        ui.horizontal(|ui| {
                            ui.label("Calendar Name");
                            if self.existing_edit_source {
                                ui.label(&source.calendar_name);
                            } else {
                                ui.text_edit_singleline(&mut source.calendar_name);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Base Url");
                            ui.text_edit_singleline(&mut source.base_url);
                        });

                        ui.horizontal(|ui| {
                            ui.label("User Name");
                            ui.text_edit_singleline(&mut source.username);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Password");
                            ui.add(TextEdit::singleline(&mut source.password).password(true));
                        });
                    }
                    TaskSource::GitHub(source) => {
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            if self.existing_edit_source {
                                ui.label(&source.name);
                            } else {
                                ui.text_edit_singleline(&mut source.name);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Server URL");
                            ui.text_edit_singleline(&mut source.server_url);
                        });
                        ui.horizontal(|ui| {
                            ui.label("API Token");
                            ui.text_edit_singleline(&mut source.token);
                        });
                    }
                    TaskSource::GitLab(source) => {
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            if self.existing_edit_source {
                                ui.label(&source.name);
                            } else {
                                ui.text_edit_singleline(&mut source.name);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Server URL");
                            ui.text_edit_singleline(&mut source.server_url);
                        });
                        ui.horizontal(|ui| {
                            ui.label("User ID");
                            ui.text_edit_singleline(&mut source.user_name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("API Token");
                            ui.text_edit_singleline(&mut source.token);
                        });
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if let Some(source) = &self.edit_source {
                            self.task_manager.add_or_replace_source(source.clone());
                        }
                        self.edit_source = None;
                        self.trigger_refresh(true, ctx.clone());
                    }
                    if ui.button("Discard").clicked() {
                        self.edit_source = None;
                    }
                });
            }
        });
    }

    fn render_single_task(&mut self, ui: &mut Ui, task: Task, now: DateTime<Utc>) {
        let mut group = egui::Frame::group(ui.style());
        let overdue = task.due.filter(|d| d.cmp(&now).is_le()).is_some();
        if Some(task.get_id()) == self.selected_task {
            group.fill = ui.visuals().selection.bg_fill;
        } else if overdue {
            group.fill = ui.visuals().error_fg_color;
        }
        group.show(ui, |ui| {
            let size = Vec2::new(BOX_WIDTH, 250.0);
            ui.set_min_size(size);
            ui.set_max_size(size);
            ui.style_mut().wrap = Some(true);

            ui.vertical(|ui| {
                let task_is_selected = Some(task.get_id()) == self.selected_task;

                ui.vertical_centered_justified(|ui| {
                    let caption = if task_is_selected {
                        "Deselect"
                    } else {
                        "Select"
                    };
                    if ui.button(caption).clicked() {
                        if Some(task.get_id()) == self.selected_task {
                            // Already selected, deselect
                            self.selected_task = None;
                        } else {
                            // Select this task
                            self.selected_task = Some(task.get_id());
                        }
                    }
                });
                if !task_is_selected {
                    if overdue {
                        ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                    } else if self.selected_task.is_some() {
                        // Another task is selected. Make this task less visible
                        ui.visuals_mut().override_text_color = Some(ui.visuals().weak_text_color());
                    }
                }
                ui.heading(task.title.as_str().truncate_ellipse(80));

                ui.label(task.project.as_str());

                if let Some(due_utc) = &task.due {
                    // Convert to local time for display
                    let due_local: DateTime<Local> = due_utc.with_timezone(&Local);
                    let mut due_label =
                        RichText::new(format!("Due: {}", due_local.format("%a, %d %b %Y %H:%M")));
                    if !overdue {
                        // Mark the number of days with the color.
                        // If the task is overdue, the background
                        // already be red, an no further highlight
                        // is necessary.
                        let hours_to_finish = due_utc.signed_duration_since(now).num_hours();
                        if hours_to_finish < 24 {
                            due_label = due_label.color(ui.visuals().error_fg_color);
                        } else if hours_to_finish < 48 {
                            due_label = due_label.color(ui.visuals().warn_fg_color);
                        };
                    }
                    ui.label(due_label);
                }
                if let Some(created) = &task.created {
                    ui.label(format!("Created: {}", created.format("%a, %d %b %Y %H:%M")));
                }
                ui.separator();
                if task.description.starts_with("https://") {
                    ui.hyperlink_to(
                        task.description.as_str().truncate_ellipse(100),
                        task.description.as_str(),
                    );
                } else {
                    ui.label(task.description.as_str().truncate_ellipse(100));
                }
            });
        });
    }

    fn render_all_tasks(&mut self, _ctx: &egui::Context, ui: &mut Ui, now: DateTime<Utc>,) {
        let box_width_with_spacing = BOX_WIDTH + (2.0 * ui.style().spacing.item_spacing.x);
        let ratio = (ui.available_width() - 5.0) / (box_width_with_spacing);
        let columns = (ratio.floor() as usize).max(1);

        let all_tasks = self.task_manager.tasks();
        // Check that the selection is valid and unselect if the task does not
        // exists
        if let Some(selection) = self.selected_task.clone() {
            if !all_tasks.iter().any(|t| t.get_id() == selection) {
                self.selected_task = None;
            }
        }

        // Create a grid layout where each row can show up to 5 tasks
        egui::Grid::new("task-grid")
            .num_columns(columns)
            .show(ui, |ui| {
                // Get all tasks for all active source
                let mut task_counter = 0;
                for task in all_tasks {
                    self.render_single_task(ui, task, now);
                    task_counter += 1;
                    if task_counter % columns == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn trigger_refresh(&mut self, manually_triggered: bool, ctx: Context) {
        self.last_refreshed = Instant::now();
        self.connection_error_for_source.clear();

        self.task_manager.refresh(move || {
            ctx.request_repaint();
        });

        if manually_triggered {
            let mut msg = Toast::info("Refreshing task list in the background");
            msg.set_duration(Some(Duration::from_secs(1)));
            self.messages.add(msg);
        }
    }
}

impl eframe::App for TaskPickerApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.separator();

                let style: Style = (*ui.ctx().style()).clone();
                let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
                if let Some(visuals) = new_visuals {
                    if visuals.dark_mode {
                        self.settings.dark_mode = true;
                    } else {
                        self.settings.dark_mode = false;
                    }
                    ui.ctx().set_visuals(visuals);
                }

                ui.separator();

                ui.add(
                    Slider::new(&mut self.settings.refresh_rate_seconds, 5..=120)
                        .text("Refresh Rate (seconds)"),
                );
            });
        });

        egui::SidePanel::left("side_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Sources");

                let mut remove_source = None;
                let mut edit_source = None;
                let mut refresh = false;

                for i in 0..self.task_manager.sources().len() {
                    let (s, enabled) = &mut self.task_manager.source_ref_mut(i);
                    let source_name = s.name();
                    ui.horizontal(|ui| {
                        let source_checkbox = ui.checkbox(enabled, source_name);
                        if source_checkbox.changed() {
                            refresh = true;
                        }
                        source_checkbox.context_menu(|ui| {
                            if ui.button("Edit").clicked() {
                                edit_source = Some(i);
                                refresh = true;
                                ui.close_menu();
                            }
                            if ui.button("Remove").clicked() {
                                remove_source = Some(i);
                                refresh = true;
                                ui.close_menu();
                            }
                        });
                        if self.connection_error_for_source.contains(source_name) {
                            ui.label("📵").on_hover_ui(|ui| {
                                ui.label("Not Connected");
                            });
                        }
                    });
                }
                if let Some(i) = remove_source {
                    self.task_manager.remove_source(i);
                } else if let Some(i) = edit_source {
                    self.edit_source = Some(self.task_manager.sources()[i].0.clone());
                    self.existing_edit_source = true;
                }

                if refresh {
                    self.trigger_refresh(true, ctx.clone());
                }

                ui.separator();
                ui.label("Add source");

                ui.horizontal_wrapped(|ui| {
                    if ui.button("CalDAV").clicked() {
                        self.existing_edit_source = false;
                        self.edit_source = Some(TaskSource::CalDav(CalDavSource::default()));
                    }
                    if ui.button("GitHub").clicked() {
                        self.existing_edit_source = false;
                        self.edit_source = Some(TaskSource::GitHub(GitHubSource::default()));
                    }
                    if ui.button("GitLab").clicked() {
                        self.existing_edit_source = false;
                        self.edit_source = Some(TaskSource::GitLab(GitLabSource::default()));
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.messages.show(ctx);

            ui.horizontal(|ui| {
                ui.heading("Tasks");

                if ui.button("Refresh").clicked() {
                    self.trigger_refresh(true, ctx.clone());
                }
            });
            ScrollArea::vertical().show(ui, |ui| self.render_all_tasks(ctx, ui, Utc::now()));
        });

        if self.edit_source.is_some() {
            self.edit_source(ctx);
        } else if self
            .last_refreshed
            .elapsed()
            .cmp(&Duration::from_secs(self.settings.refresh_rate_seconds))
            .is_gt()
        {
            self.trigger_refresh(false, ctx.clone());
        }

        for (source, active) in self.task_manager.sources() {
            if *active {
                let source_name = source.name();
                if let Some(err) = self.task_manager.get_and_clear_last_err(source.name()) {
                    if is_dns_error(&err) {
                        // DNS errors indicate a connection problem on our side
                        self.connection_error_for_source
                            .insert(source_name.to_string());
                    } else {
                        error!("Error querying source \"{source_name}\". {}", &err);
                        let shortened_message = err
                            .to_string()
                            .chars()
                            .chunks(50)
                            .into_iter()
                            .map(|c| c.collect::<String>())
                            .join("\n");
                        self.messages
                            .error(format!("[{source_name}]\n{shortened_message}"));
                    }
                }
            }
        }

        // Make sure we will be called after the refresh rate has been expired
        ctx.request_repaint_after(Duration::from_secs(self.settings.refresh_rate_seconds));
    }
}

fn is_dns_error(err: &anyhow::Error) -> bool {
    if let Some(Error::Ical(caldav_err)) = err.downcast_ref::<minicaldav::Error>() {
        // The errors only transport the string, so we have to search the error
        // message for a matching string
        caldav_err.starts_with("Transport(Transport { kind: Dns,")
    } else if let Some(transport_err) = err.downcast_ref::<ureq::Error>() {
        ErrorKind::Dns == transport_err.kind()
    } else {
        false
    }
}

#[cfg(test)]
mod tests;
