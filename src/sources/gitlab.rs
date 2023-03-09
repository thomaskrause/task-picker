use anyhow::{Result, Context};
use json::JsonValue;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::tasks::Task;

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
    pub fn query_tasks(&mut self) -> Result<Vec<Task>> {
        let mut result = Vec::default();

        let request = self
            .agent
            .get(&format!(
                "{}/issues?state=opened&assignee_username={}",
                self.server_url, self.user_name
            ))
            .set("PRIVATE-TOKEN", &self.token);
        let response = request.call()?;
        let body = response.into_string()?;
        let assigned_issues = json::parse(&body)?;

        if let JsonValue::Array(assigned_issues) = assigned_issues {
            for issue in assigned_issues {
                if let JsonValue::Object(issue) = issue {
                    
                    let project = if let JsonValue::String(project_id) = issue
                        .get("project_id")
                        .context("Missing 'project_id' field for issue")?
                    {
                        project_id
                    } else {
                        "GitLab"
                    };

                    let title = issue
                        .get("title")
                        .context("Missing 'title' field for issue")?
                        .as_str()
                        .unwrap_or_default();
                    let url = issue
                        .get("web_url")
                        .context("Missing 'web_url' field for issue")?
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
        Ok(result)
    }
}
