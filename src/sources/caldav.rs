use std::collections::HashMap;

use anyhow::Result;
use chrono::prelude::*;

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

const DATE_TIME_FORMAT: &str = "%Y%m%dT%H%M%S";
const DATE_TIME_FORMAT_WITH_TZ: &str = "%Y%m%dT%H%M%S%Z";

impl CalDavSource {
    pub fn query_tasks(&self) -> Result<Vec<Task>> {
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
                    // Check start due date if this task is ready to be started on
                    let start_due = props
                        .get("DTSTART")
                        .map(|raw| NaiveDateTime::parse_from_str(raw.as_str(), DATE_TIME_FORMAT))
                        .transpose()?
                        .map(|t| Utc.from_utc_datetime(&t));
                    let can_start = if let Some(start_due) = start_due {
                        Utc::now().cmp(&start_due).is_ge()
                    } else {
                        true
                    };
                    if !completed && can_start {
                        if let Some(title) = props.get("SUMMARY") {
                            let title = title
                                .replace("\\\\", "\\")
                                .replace("\\n", "\n")
                                .replace("\\,", ",");
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
                                .map(|raw| {
                                    NaiveDateTime::parse_from_str(raw.as_str(), DATE_TIME_FORMAT)
                                })
                                .transpose()?;

                            let created = props
                                .get("CREATED")
                                .map(|raw| {
                                    NaiveDateTime::parse_from_str(
                                        raw.as_str(),
                                        DATE_TIME_FORMAT_WITH_TZ,
                                    )
                                })
                                .transpose()?;

                            let task = Task {
                                project: c.name().clone(),
                                title: title.clone(),
                                description,
                                due,
                                created,
                                id: props.get("UID").cloned(),
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
