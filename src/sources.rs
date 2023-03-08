use anyhow::Result;
use eframe::epaint::ahash::HashMap;
use serde::{Deserialize, Serialize};
use ureq::Agent;
use url::Url;

use crate::{Task, TaskProvider};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CalDavSource {
    #[serde(skip)]
    agent: ureq::Agent,
    pub calendar_name: String,
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
            calendar_name: String::default(),
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
        self.calendar_name.as_str()
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
            let mut result = Vec::default();
            for c in calendars {
                if c.name().as_str() == self.calendar_name {
                    let (todos, _errors) = minicaldav::get_todos(
                        self.agent.clone(),
                        &self.username,
                        &self.password,
                        &c,
                    )?;
                    for t in todos {
                        let props: HashMap<String, String> = t
                            .properties_todo()
                            .into_iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect();
                        if let Some(title) = props.get("SUMMARY") {
                            let description : String = props.get("DESCRIPTION").map(|s| s.to_string()).unwrap_or_default();
                            let task = Task {
                                title: title.clone(),
                                description,
                            };
                            result.push(task);
                        }
                    }
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
