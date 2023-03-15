use std::{
    cmp::Ordering,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use eframe::epaint::ahash::HashMap;
use serde::{Deserialize, Serialize};

use crate::sources::TaskSource;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub project: String,
    pub title: String,
    pub description: String,
    pub due: Option<DateTime<Utc>>,
    pub created: Option<DateTime<Utc>>,
    pub id: Option<String>,
}

impl Task {
    pub fn get_id(&self) -> String {
        // Use provided ID or fall back to an auto-generated one
        self.id
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format!("{}/{}", self.project, self.title))
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TaskManager {
    tasks: Arc<Mutex<Vec<Task>>>,
    sources: Vec<(TaskSource, bool)>,
    #[serde(skip)]
    error_by_source: Arc<Mutex<HashMap<String, anyhow::Error>>>,
}

fn compare_optional<T: Ord>(a: &Option<T>, b: &Option<T>) -> Ordering {
    if let (Some(a), Some(b)) = (a, b) {
        a.cmp(b)
    } else if a.is_none() {
        if b.is_none() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    } else {
        Ordering::Less
    }
}

impl TaskManager {
    /// Refresh task list in the background
    pub fn refresh<F>(&mut self, finish_callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let sources = self.sources.clone();
        let error_by_source = self.error_by_source.clone();
        let tasks = self.tasks.clone();

        rayon::spawn(move || {
            // Query tasks for each source and collect errors when they occur, but
            // proceed with the next source.
            let mut new_tasks = Vec::default();
            let mut new_errors = HashMap::default();
            for (source, active) in &sources {
                if *active {
                    let tasks_for_source = match source {
                        TaskSource::CalDav(s) => s.query_tasks(),
                        TaskSource::GitHub(s) => s.query_tasks(),
                        TaskSource::GitLab(s) => s.query_tasks(),
                    };
                    match tasks_for_source {
                        Ok(tasks_for_source) => new_tasks.extend(tasks_for_source),
                        Err(e) => {
                            new_errors.insert(source.name().to_string(), e);
                        }
                    }
                }
            }

            // Show the tasks that are due next first. Tasks without due date are sorted
            // by their creation date (oldest first).
            new_tasks.sort_by(|a, b| {
                let by_due_date = compare_optional(&a.due, &b.due);

                if by_due_date == Ordering::Equal {
                    compare_optional(&a.created, &b.created)
                } else {
                    by_due_date
                }
            });

            {
                let mut tasks = tasks.lock().expect("Lock poisoning");
                *tasks = new_tasks;
            }
            {
                let mut error_by_source = error_by_source.lock().expect("Lock poisoning");
                *error_by_source = new_errors;
            }

            finish_callback();
        });
    }

    pub fn tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().expect("Lock poisoning");
        tasks.clone()
    }

    pub fn sources(&self) -> &Vec<(TaskSource, bool)> {
        &self.sources
    }

    pub fn source_ref_mut(&mut self, idx: usize) -> &mut (TaskSource, bool) {
        &mut self.sources[idx]
    }

    /// Adds a new resource or replaces an existing one if a source with the
    /// same name already exists.
    pub fn add_or_replace_source(&mut self, source: TaskSource) {
        let existing = self
            .sources
            .binary_search_by(|(probe, _)| probe.name().cmp(source.name()));
        match existing {
            Ok(i) => self.sources[i].0 = source,
            Err(i) => self.sources.insert(i, (source, true)),
        };
    }

    pub fn remove_source(&mut self, idx: usize) -> (TaskSource, bool) {
        self.sources.remove(idx)
    }

    pub fn get_and_clear_last_err(&self, source: &str) -> Option<anyhow::Error> {
        let mut error_by_source = self.error_by_source.lock().expect("Lock poisoning");
        error_by_source.remove(source)
    }
}
