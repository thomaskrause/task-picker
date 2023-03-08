use anyhow::Result;
use serde::{Deserialize, Serialize};
use ureq::Agent;
use url::Url;

use crate::{Task, TaskProvider};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CalDavSource {
    #[serde(skip)]
    agent: ureq::Agent,
    pub label: String,
    pub username: String,
    pub password: String,
    pub base_url: String,
    #[serde(skip)]
    last_connection_attempt: Option<std::time::Instant>,
    #[serde(skip)]
    cached_tasks: Vec<Task>,
}

impl Default for CalDavSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            label: String::default(),
            username: String::default(),
            password: String::default(),
            base_url: String::default(),
            cached_tasks: Vec::default(),
            last_connection_attempt: None,
        }
    }
}

impl TaskProvider for CalDavSource {
    fn get_label(&self) -> &str {
        self.label.as_str()
    }

    fn get_tasks(&mut self) -> Result<Vec<Task>> {
        // Only get values each 2 minutes
        let mut use_cache = self.last_connection_attempt.is_some();
        if let Some(last_connection_attempt) = self.last_connection_attempt {
            let seconds_since_last_attempt = last_connection_attempt
                .duration_since(std::time::Instant::now())
                .as_secs();
            if seconds_since_last_attempt > 120 {
                use_cache = false;
            }
        }
        self.last_connection_attempt = Some(std::time::Instant::now());

        if use_cache {
            Ok(self.cached_tasks.clone())
        } else {
            let base_url = Url::parse(&self.base_url)?;
            let calendars = minicaldav::get_calendars(
                self.agent.clone(),
                &self.username,
                &self.password,
                &base_url,
            )?;
            let unknown_string = String::from("UNKNOWN");
            let mut result = Vec::default();
            for c in calendars {
                let (todos, _errors) =
                    minicaldav::get_todos(self.agent.clone(), &self.username, &self.password, &c)?;
                for t in todos {
                    let title = t.get("title").unwrap_or(&unknown_string);
                    let task = Task {
                        title: title.clone(),
                    };
                    result.push(task);
                }
            }
            self.cached_tasks = result.clone();
            Ok(result)
        }
    }

    fn reset_cache(&mut self) {
        self.last_connection_attempt = None;
        self.cached_tasks.clear();
    }
}
