use std::{
    cmp::Ordering,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::sources::TaskSource;

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub project: String,
    pub title: String,
    pub description: String,
    pub due: Option<NaiveDateTime>,
    pub created: Option<NaiveDateTime>,
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
    pub sources: Vec<(TaskSource, bool)>,
    #[serde(skip)]
    last_error: Arc<Mutex<Option<anyhow::Error>>>,
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

fn try_get_tasks(sources: &mut Vec<(TaskSource, bool)>) -> Result<Vec<Task>> {
    let mut result = Vec::default();

    for (source, active) in sources {
        if *active {
            let new_tasks = match source {
                TaskSource::CalDav(s) => s.query_tasks()?,
                TaskSource::GitHub(s) => s.query_tasks()?,
                TaskSource::GitLab(s) => s.query_tasks()?,
            };
            result.extend(new_tasks);
        }
    }

    // Show the tasks that are due next first. Tasks without due date are sorted
    // by their creation date (oldest first).
    result.sort_by(|a, b| {
        let by_due_date = compare_optional(&a.due, &b.due);

        if by_due_date == Ordering::Equal {
            compare_optional(&a.created, &b.created)
        } else {
            by_due_date
        }
    });

    Ok(result)
}

impl TaskManager {
    /// Refresh task list in the background
    pub fn refresh(&mut self) {
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

    pub fn add_source(&mut self, source: TaskSource) {
        self.sources.push((source, true));
    }

    pub fn get_and_clear_last_err(&self) -> Option<anyhow::Error> {
        let mut last_error = self.last_error.lock().expect("Lock poisoning");
        last_error.take()
    }
}
