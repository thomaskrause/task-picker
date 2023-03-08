use std::time::{Duration, Instant};

use crate::{sources::CalDavSource, tasks::TaskManager};
use egui::{ScrollArea, TextEdit, Ui};
use egui_notify::{Toast, Toasts};
use itertools::Itertools;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TaskPickerApp {
    task_manager: TaskManager,
    refresh_rate: Duration,
    #[serde(skip)]
    last_refreshed: Instant,
    #[serde(skip)]
    messages: Toasts,
    #[serde(skip)]
    new_task_source: Option<CalDavSource>,
}

impl Default for TaskPickerApp {
    fn default() -> Self {
        let mut task_manager = TaskManager::default();
        task_manager.refresh();
        Self {
            task_manager: TaskManager::default(),
            refresh_rate: Duration::from_secs(15),
            last_refreshed: Instant::now(),
            new_task_source: None,
            messages: Toasts::default(),
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
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl TaskPickerApp {
    fn add_new_task(&mut self, ctx: &egui::Context) {
        egui::Window::new("Add Task Source").show(ctx, |ui| {
            if let Some(new_task_source) = &mut self.new_task_source {
                ui.horizontal(|ui| {
                    ui.label("Calendar Name");
                    ui.text_edit_singleline(&mut new_task_source.calendar_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Base Url");
                    ui.text_edit_singleline(&mut new_task_source.base_url);
                });

                ui.horizontal(|ui| {
                    ui.label("User Name");
                    ui.text_edit_singleline(&mut new_task_source.username);
                });
                ui.horizontal(|ui| {
                    ui.label("Password");
                    ui.add(TextEdit::singleline(&mut new_task_source.password).password(true));
                });
            }

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if let Some(new_task_source) = &self.new_task_source {
                        self.task_manager.add_caldav_source(new_task_source.clone());
                    }
                    self.new_task_source = None;
                }
                if ui.button("Discard").clicked() {
                    self.new_task_source = None;
                }
            });
        });
    }

    fn render_tasks(&mut self, _ctx: &egui::Context, ui: &mut Ui) {
        // Create a grid layout where each row can show up to 5 tasks
        egui::Grid::new("task-grid").num_columns(5).show(ui, |ui| {
            // Get all tasks for all active source
            let mut task_counter = 0;
            for task in self.task_manager.tasks() {
                ui.group(|ui| {
                    ui.set_max_width(150.0);
                    ui.style_mut().wrap = Some(true);

                    ui.vertical(|ui| {
                        ui.heading(&task.title);
                        let mut description = task.description.clone();
                        description.truncate(50);
                        ui.label(description.replace("\\\\n", "\n"));
                    });
                });
                task_counter += 1;
                if task_counter % 5 == 0 {
                    ui.end_row();
                }
            }
        });
    }

    fn trigger_refresh(&mut self) {
        self.last_refreshed = Instant::now();
        self.task_manager.refresh();
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
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::right("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Sources");

                let mut remove_source = None;
                let mut refresh = false;

                for i in 0..self.task_manager.sources.len() {
                    let (s, enabled) = &mut self.task_manager.sources[i];
                    ui.horizontal(|ui| {
                        if ui.checkbox(enabled, &s.calendar_name).changed() {
                            refresh = true;
                        }
                        if ui.small_button("X").clicked() {
                            remove_source = Some(i);
                            refresh = true;
                        }
                    });
                }
                if let Some(i) = remove_source {
                    self.task_manager.sources.remove(i);
                }
                if refresh {
                    self.trigger_refresh();
                }

                if ui.button("Add CalDAV").clicked() {
                    self.new_task_source = Some(CalDavSource::default());
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.messages.show(ctx);

            ui.heading("Task Picker");

            if ui.button("Refresh").clicked() {
                self.trigger_refresh();
                let mut msg = Toast::info("Refreshing task list in the background");
                msg.set_duration(Some(Duration::from_secs(1)));
                self.messages.add(msg);
            }

            ScrollArea::vertical().show(ui, |ui| self.render_tasks(ctx, ui));
        });

        if self.new_task_source.is_some() {
            self.add_new_task(ctx);
            self.trigger_refresh();
        } else if self
            .last_refreshed
            .elapsed()
            .cmp(&self.refresh_rate)
            .is_gt()
        {
            self.trigger_refresh();
        }

        if let Some(err) = &self.task_manager.get_and_clear_last_err() {
            let message = err
                .to_string()
                .chars()
                .chunks(50)
                .into_iter()
                .map(|c| c.collect::<String>())
                .join("\n");
            self.messages.error(message);
        }
    }
}
