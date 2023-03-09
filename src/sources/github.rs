use anyhow::{Context, Result};
use json::JsonValue;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::tasks::Task;

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
    pub fn query_tasks(&self) -> Result<Vec<Task>> {
        let mut result = Vec::default();

        let request = self
            .agent
            .get(&format!("{}/issues", self.server_url))
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
                        let project = if let JsonValue::Object(repo) = issue
                            .get("repository")
                            .context("Missing 'repository' field for issue")?
                        {
                            repo.get("full_name")
                                .context("Missing 'full_name' field for issue")?
                                .as_str()
                                .unwrap_or_default()
                        } else {
                            "GitHub"
                        };

                        let title = issue
                            .get("title")
                            .context("Missing 'title' field for issue")?
                            .as_str()
                            .unwrap_or_default();
                        let url = issue
                            .get("html_url")
                            .context("Missing 'html_url' field for issue")?
                            .as_str()
                            .unwrap_or_default();
                        let task = Task {
                            project: project.to_string(),
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
