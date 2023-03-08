use crate::{sources::CalDavSource, TaskProvider, TaskSource};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TaskPickerApp {
    new_task_source: Option<CalDavSource>,
    sources: Vec<(TaskSource, bool)>,
}

impl Default for TaskPickerApp {
    fn default() -> Self {
        Self {
            new_task_source: None,
            sources: Vec::default(),
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
                    ui.label("Label");
                    ui.text_edit_singleline(&mut new_task_source.label);
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
                    ui.text_edit_singleline(&mut new_task_source.password);
                });
            }

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if let Some(new_task_source) = &self.new_task_source {
                        self.sources
                            .push((TaskSource::CalDav(new_task_source.clone()), true));
                    }
                    self.new_task_source = None;
                }
                if ui.button("Discard").clicked() {
                    self.new_task_source = None;
                }
            });
        });
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
                for i in 0..self.sources.len() {
                    let (s, enabled) = &mut self.sources[i];
                    ui.horizontal(|ui| {
                        ui.checkbox(enabled, s.get_label());
                        if ui.small_button("X").clicked() {
                            remove_source = Some(i);
                        }
                    });
                }
                if let Some(i) = remove_source {
                    self.sources.remove(i);
                }

                if ui.button("Add CalDAV").clicked() {
                    self.new_task_source = Some(CalDavSource::default());
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Task Picker");
        });

        if self.new_task_source.is_some() {
            self.add_new_task(ctx);
        }
    }
}
