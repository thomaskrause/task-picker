#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod sources;
pub use app::TaskPickerApp;

use serde::{Serialize, Deserialize};
use sources::CalDavSource;

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub title: String,
    pub description: String,
}


#[derive(Serialize, Deserialize)]
#[serde()]
pub enum TaskSource {
    CalDav(CalDavSource)
}

trait TaskProvider {
    fn get_label(&self) -> &str;
    fn get_tasks(&mut self) -> anyhow::Result<Vec<Task>>;
    fn reset_cache(&mut self);
}

impl TaskProvider for TaskSource {
    fn get_tasks(&mut self) -> anyhow::Result<Vec<Task>> {
        match self {
            TaskSource::CalDav(c) => c.get_tasks(),
        }
    }

    fn get_label(&self) -> &str {
        match self {
            TaskSource::CalDav(c) => c.get_label(),
        }
    }

    fn reset_cache(&mut self) {
        match self {
            TaskSource::CalDav(c) => c.reset_cache(),
        }
    }
}