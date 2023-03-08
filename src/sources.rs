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
}

impl Default for CalDavSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            label: String::default(),
            username: String::default(),
            password: String::default(),
            base_url: String::default(),
        }
    }
}

impl TaskProvider for CalDavSource {
    fn get_label(&self) -> &str {
        self.label.as_str()
    }

    fn get_tasks(&self) -> Result<Vec<Task>> {
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
        Ok(result)
    }
}
