use anyhow::Result;
use chrono::prelude::*;
use eframe::epaint::ahash::HashMap;
use serde::{Deserialize, Serialize};
use ureq::Agent;
use url::Url;

use crate::tasks::Task;

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
                    let completed = props
                        .get("STATUS")
                        .filter(|s| s.as_str() == "COMPLETED")
                        .is_some()
                        || props.contains_key("COMPLETED");
                    if !completed {
                        if let Some(title) = props.get("SUMMARY") {
                            let description: String = props
                                .get("DESCRIPTION")
                                .map(|s| {
                                    s.replace("\\\\", "\\")
                                        .replace("\\n", "\n")
                                        .replace("\\,", ",")
                                })
                                .unwrap_or_default();
                            let due = props
                                .get("DUE")
                                .map(|raw| Utc.datetime_from_str(raw.as_str(), "%Y%m%dT%H%M%S"))
                                .transpose()?;
                            let task = Task {
                                title: title.clone(),
                                description,
                                due,
                            };
                            result.push(task);
                        }
                    }
                }
            }
        }
        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GitHubSource {
    #[serde(skip)]
    agent: ureq::Agent,
    pub token: String,
}

impl Default for GitHubSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            token: Default::default(),
        }
    }
}
impl GitHubSource {
    pub fn query_tasks(&mut self) -> Result<Vec<Task>> {
        let response = self
            .agent
            .get("https://api.github.com/issues")
            .set("Authorization", &format!("Beaer: {}", &self.token))
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set("Accept", "application/vnd.github+json")
            .call()?;
        let body = response.into_string()?;
        dbg!(body);
        Ok(vec![])
    }
}
