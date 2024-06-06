use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use json::JsonValue;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::tasks::Task;

use super::GITHUB_ICON;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GitHubSource {
    #[serde(skip)]
    agent: ureq::Agent,
    pub name: String,
    pub server_url: String,
    pub token: String,
}

impl Default for GitHubSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            token: Default::default(),
            name: "GitHub".to_string(),
            server_url: "https://api.github.com".to_string(),
        }
    }
}
impl GitHubSource {
    pub fn query_tasks(&self, secret: Option<&str>) -> Result<Vec<Task>> {
        let mut result = Vec::default();

        let mut request = self
            .agent
            .get(&format!("{}/issues", self.server_url))
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set("Accept", "application/vnd.github+json");
        if let Some(secret) = secret {
            request = request.set("Authorization", &format!("Bearer {}", secret))
        }
        let response = request.call()?;
        let body = response.into_string()?;
        let assigned_issues = json::parse(&body)?;
        if let JsonValue::Array(assigned_issues) = assigned_issues {
            for issue in assigned_issues {
                if let JsonValue::Object(issue) = issue {
                    if Some("open") == issue["state"].as_str() {
                        let project = if let JsonValue::Object(repo) = &issue["repository"] {
                            repo["full_name"]
                                .as_str()
                                .context("Missing 'full_name' field for issue")?
                        } else {
                            "GitHub"
                        };

                        let title = issue["title"]
                            .as_str()
                            .context("Missing 'title' field for issue")?;
                        let url = issue["html_url"]
                            .as_str()
                            .context("Missing 'html_url' field for issue")?;

                        let created: Option<DateTime<Utc>> = issue["created_at"]
                            .as_str()
                            .map(|d| DateTime::parse_from_str(d, "%+"))
                            .transpose()?
                            .map(|d| d.into());

                        let due: Option<DateTime<Utc>> =
                            if let JsonValue::Object(milestone) = &issue["milestone"] {
                                milestone["due_on"]
                                    .as_str()
                                    .map(|d| DateTime::parse_from_str(d, "%+"))
                                    .transpose()?
                                    .map(|d| d.into())
                            } else {
                                None
                            };

                        let task = Task {
                            project: format!("{} {}", GITHUB_ICON, project),
                            title: title.to_string(),
                            description: url.to_string(),
                            due,
                            created,
                            id: Some(url.to_string()),
                        };
                        result.push(task);
                    }
                }
            }
        }
        Ok(result)
    }
}
