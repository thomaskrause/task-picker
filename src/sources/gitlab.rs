use anyhow::{Ok, Result};
use chrono::{DateTime, NaiveDate, Utc};
use json::JsonValue;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::tasks::Task;

use super::GITLAB_ICON;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GitLabSource {
    #[serde(skip)]
    agent: Agent,
    pub name: String,
    pub server_url: String,
    pub user_name: String,
    pub token: String,
}

impl Default for GitLabSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            name: "GitLab".to_string(),
            server_url: "https://gitlab.com/api/v4/".to_string(),
            user_name: Default::default(),
            token: Default::default(),
        }
    }
}

impl GitLabSource {
    fn query_todos(&self, secret: Option<&str>) -> Result<Vec<Task>> {
        let mut request = self
            .agent
            .get(&format!("{}/todos?state=pending", self.server_url,));
        if let Some(secret) = secret {
            request = request.set("PRIVATE-TOKEN", secret);
        }
        let response = request.call()?;
        let body = response.into_string()?;
        let all_todos = json::parse(&body)?;

        let mut result = Vec::default();

        if let JsonValue::Array(all_todos) = all_todos {
            for todo in all_todos {
                let project = todo["project"]["name_with_namespace"]
                    .as_str()
                    .unwrap_or(&self.name);

                let title = todo["body"].as_str().unwrap_or_default();

                // Work items sometimes have a woring "target_url, but the
                // "web_url" of the target is correct and should be prefered.
                let url = if let Some(target_url) = todo["target"]["web_url"].as_str() {
                    target_url
                } else {
                    todo["target_url"].as_str().unwrap_or_default()
                };

                let due: Option<DateTime<Utc>> = todo["target"]["due_date"]
                    .as_str()
                    .map(|due_date| NaiveDate::parse_from_str(due_date, "%Y-%m-%d"))
                    .transpose()?
                    .and_then(|due_date| due_date.and_hms_opt(0, 0, 0))
                    .map(|due_date| DateTime::from_naive_utc_and_offset(due_date, Utc));

                let created: Option<DateTime<Utc>> = todo["created_at"]
                    .as_str()
                    .map(|d| DateTime::parse_from_str(d, "%+"))
                    .transpose()?
                    .map(|d| d.into());

                let task = Task {
                    project: format!("{} {}", GITLAB_ICON, project),
                    title: title.to_string(),
                    description: url.to_string(),
                    due,
                    created,
                    id: Some(url.to_string()),
                };
                result.push(task);
            }
        }
        Ok(result)
    }

    pub fn query_tasks(&self, secret: Option<&str>) -> Result<Vec<Task>> {
        let mut result = Vec::default();

        let todos = self.query_todos(secret)?;
        result.extend(todos);

        Ok(result)
    }
}
