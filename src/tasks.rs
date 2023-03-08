use std::{
    cmp::Ordering,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::sources::{CalDavSource, GitHubSource};

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub project: String,
    pub title: String,
    pub description: String,
    pub due: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TaskManager {
    tasks: Arc<Mutex<Vec<Task>>>,
    pub caldav_sources: Vec<(CalDavSource, bool)>,
    pub github_source: Option<(GitHubSource, bool)>,
    #[serde(skip)]
    last_connection_attempt: Option<std::time::Instant>,
    #[serde(skip)]
    last_error: Arc<Mutex<Option<anyhow::Error>>>,
}

fn try_get_tasks(
    sources: &mut Vec<(CalDavSource, bool)>,
    gh_source: Option<&mut (GitHubSource, bool)>,
) -> Result<Vec<Task>> {
    let mut result = Vec::default();

    for (source, active) in sources {
        if *active {
            let new_tasks = source.query_tasks()?;
            result.extend(new_tasks);
        }
    }

    if let Some((gh_source, active)) = gh_source {
        if *active {
            let new_tasks = gh_source.query_tasks()?;
            result.extend(new_tasks);
        }
    }

    // Show the tasks that are due next first
    result.sort_by(|a, b| {
        if let (Some(a), Some(b)) = (a.due, b.due) {
            a.cmp(&b)
        } else if a.due.is_none() {
            if b.due.is_none() {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        } else {
            Ordering::Less
        }
    });

    Ok(result)
}

impl TaskManager {
    /// Refresh task list in the background
    pub fn refresh(&mut self) {
        let mut caldav_sources = self.caldav_sources.clone();
        let mut gh_source = self.github_source.clone();
        let last_error = self.last_error.clone();
        let tasks = self.tasks.clone();

        rayon::spawn(
            move || match try_get_tasks(&mut caldav_sources, gh_source.as_mut()) {
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
                    {
                        let mut tasks = tasks.lock().expect("Lock poisoning");
                        tasks.clear();
                    }
                    {
                        let mut last_error = last_error.lock().expect("Lock poisoning");
                        *last_error = Some(e);
                    }
                }
            },
        );
    }

    pub fn tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().expect("Lock poisoning");
        tasks.clone()
    }

    pub fn add_caldav_source(&mut self, source: CalDavSource) {
        self.caldav_sources.push((source, true));
    }

    pub fn set_github_source(&mut self, source: GitHubSource) {
        self.github_source = Some((source, true));
    }

    pub fn get_and_clear_last_err(&self) -> Option<anyhow::Error> {
        let mut last_error = self.last_error.lock().expect("Lock poisoning");
        last_error.take()
    }
}
