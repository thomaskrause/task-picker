use crate::TaskSource;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TaskPickerApp {
    add_task_source: bool,
    sources: Vec<TaskSource>,
}

impl Default for TaskPickerApp {
    fn default() -> Self {
        Self {
            add_task_source: false,
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
                ui.heading("Task Sources");

                if ui.button("Add").clicked() {
                    self.add_task_source = true;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Task Picker");
        });

        if self.add_task_source {
            egui::Window::new("Add Task Source")
                .open(&mut self.add_task_source)
                .show(ctx, |ui| {
                    ui.label("Ups");
                });
        }
    }
}
