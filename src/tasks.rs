use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::sources::CalDavSource;

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct TaskManager {
    tasks: Arc<Mutex<Vec<Task>>>,
    pub sources: Vec<(CalDavSource, bool)>,
    #[serde(skip)]
    last_connection_attempt: Option<std::time::Instant>,
    #[serde(skip)]
    last_error: Arc<Mutex<Option<anyhow::Error>>>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
            sources: Default::default(),
            last_connection_attempt: Default::default(),
            last_error: Default::default(),
        }
    }
}

fn try_get_tasks(sources: &mut Vec<(CalDavSource, bool)>) -> Result<Vec<Task>> {
    let mut result = Vec::default();

    for (source, active) in sources {
        if *active {
            let new_tasks = source.query_tasks()?;
            result.extend(new_tasks);
        }
    }
    Ok(result)
}

impl TaskManager {
    /// Refresh task list in the background
    pub fn refresh(&mut self) {
        {
            let mut tasks = self.tasks.lock().expect("Lock poisoning");
            tasks.clear();
        }

        let mut sources = self.sources.clone();
        let last_error = self.last_error.clone();
        let tasks = self.tasks.clone();

        rayon::spawn(move || match try_get_tasks(&mut sources) {
            Ok(new_tasks) => {
                {
                    let mut tasks = tasks.lock().expect("Lock poisoning");
                    *tasks = new_tasks;
                }
                {
                    let mut last_error = last_error.lock().expect("Lock poisoning");
                    *last_error = None;
                }
            }
            Err(e) => {
                let mut last_error = last_error.lock().expect("Lock poisoning");
                *last_error = Some(e);
            }
        });
    }

    pub fn tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().expect("Lock poisoning");
        tasks.clone()
    }

    pub fn add_caldav_source(&mut self, source: CalDavSource) {
        self.sources.push((source, true));
    }

    pub fn get_and_clear_last_err(&self) -> Option<anyhow::Error> {
        let mut last_error = self.last_error.lock().expect("Lock poisoning");
        let result = last_error.take();
        result
    }
}
