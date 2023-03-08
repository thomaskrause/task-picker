use anyhow::Result;
use eframe::epaint::ahash::HashMap;
use serde::{Deserialize, Serialize};
use ureq::Agent;
use url::Url;

use crate::tasks::Task;

#[derive(Serialize, Deserialize)]
#[serde()]
pub enum TaskSource {
    CalDav(CalDavSource),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CalDavSource {
    #[serde(skip)]
    agent: ureq::Agent,
    pub calendar_name: String,
    pub username: String,
    pub password: String,
    pub base_url: String,
}

impl Default for CalDavSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            calendar_name: String::default(),
            username: String::default(),
            password: String::default(),
            base_url: String::default(),
        }
    }
}

impl CalDavSource {
    pub fn query_tasks(&mut self) -> Result<Vec<Task>> {
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
                let (todos, _errors) =
                    minicaldav::get_todos(self.agent.clone(), &self.username, &self.password, &c)?;
                for t in todos {
                    let props: HashMap<String, String> = t
                        .properties_todo()
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect();
                    if let Some(title) = props.get("SUMMARY") {
                        let description: String = props
                            .get("DESCRIPTION")
                            .map(|s| s.to_string())
                            .unwrap_or_default();
                        let task = Task {
                            title: title.clone(),
                            description,
                        };
                        result.push(task);
                    }
                }
            }
        }
        Ok(result)
    }
}
