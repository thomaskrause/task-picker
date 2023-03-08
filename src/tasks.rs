use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

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
    pub refresh_rate: Duration,
    #[serde(skip)]
    last_connection_attempt: Option<std::time::Instant>,
    #[serde(skip)]
    pub last_error: Option<anyhow::Error>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
            sources: Default::default(),
            refresh_rate: Duration::from_secs(10),
            last_connection_attempt: Default::default(),
            last_error: Default::default(),
        }
    }
}

impl TaskManager {
    pub fn refresh(&mut self) {
        if let Err(e) = self.try_refresh() {
            self.last_error = Some(e);
        } else {
            self.last_error = None;
        }
    }

    fn try_refresh(&mut self) -> Result<()> {
        let mut result = Vec::default();

        for (source, active) in &mut self.sources {
            if *active {
                let new_tasks = source.query_tasks()?;
                result.extend(new_tasks);
            }
        }
        let mut tasks = self.tasks.lock().expect("Lock poisoning");
        *tasks = result;

        Ok(())
    }

    pub fn tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().expect("Lock poisoning");
        tasks.clone()
    }

    pub fn add_caldav_source(&mut self, source: CalDavSource) {
        self.sources.push((source, true));
    }
}
