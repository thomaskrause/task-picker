use anyhow::{Context, Result};
use chrono::prelude::*;
use eframe::epaint::ahash::HashMap;
use json::JsonValue;
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
        let mut result = Vec::default();

        let request = self
            .agent
            .get("https://api.github.com/issues")
            .set("Authorization", &format!("Bearer {}", &self.token))
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set("Accept", "application/vnd.github+json");
        let response = request.call()?;
        let body = response.into_string()?;
        let assigned_issues = json::parse(&body)?;
        if let JsonValue::Array(assigned_issues) = assigned_issues {
            for issue in assigned_issues {
                if let JsonValue::Object(issue) = issue {
                    if Some("open")
                        == issue
                            .get("state")
                            .context("Missing 'state' field for issue")?
                            .as_str()
                    {
                        let title = issue.get("title").context("Missing 'title' field for issue")?.as_str().unwrap_or_default();
                        let url = issue.get("html_url").context("Missing 'html_url' field for issue")?.as_str().unwrap_or_default();
                        let task = Task {
                            title: title.to_string(),
                            description: url.to_string(),
                            due: None,
                        };
                        result.push(task);
                    }
                }
            }
        }
        Ok(result)
    }
}
