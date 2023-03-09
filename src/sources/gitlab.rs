use anyhow::{Context, Ok, Result};
use chrono::NaiveDate;
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
    fn get_project_name(&self, project_id: usize) -> Result<Option<String>> {
        let request = self
            .agent
            .get(&format!("{}/projects/{}", self.server_url, project_id))
            .set("PRIVATE-TOKEN", &self.token);
        let response = request.call()?;
        let body = response.into_string()?;
        let project = json::parse(&body)?;
        if let JsonValue::Object(project) = project {
            let project_name = project
                .get("name_with_namespace")
                .context("Missing field 'name_with_namespace' on project JSON")?
                .as_str()
                .context("'name_with_namespace' is not a string")?;
            Ok(Some(project_name.to_string()))
        } else {
            Ok(None)
        }
    }

    fn extract_issues_for_page(&self, page: usize) -> Result<Vec<Task>> {
        let mut result = Vec::default();
        let request = self
            .agent
            .get(&format!(
                "{}/issues?page={}&state=opened&assignee_username={}",
                self.server_url, page, self.user_name
            ))
            .set("PRIVATE-TOKEN", &self.token);
        let response = request.call()?;
        let body = response.into_string()?;
        let assigned_issues = json::parse(&body)?;

        if let JsonValue::Array(assigned_issues) = assigned_issues {
            for issue in assigned_issues {
                if let JsonValue::Object(issue) = issue {
                    let project_id = issue
                        .get("project_id")
                        .context("Missing 'project_id' field for issue")?
                        .as_usize()
                        .context("'project_id' is not a number")?;
                    let project = self
                        .get_project_name(project_id)?
                        .unwrap_or_else(|| self.name.clone());

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

                    let due = issue
                        .get("due_date")
                        .and_then(|due_date| due_date.as_str())
                        .map(|due_date| NaiveDate::parse_from_str(due_date, "%Y-%m-%d"))
                        .transpose()?
                        .and_then(|due_date| due_date.and_hms_opt(0, 0, 0));

                    let task = Task {
                        project,
                        title: title.to_string(),
                        description: url.to_string(),
                        due,
                    };
                    result.push(task);
                }
            }
        }
        Ok(result)
    }

    pub fn query_tasks(&self) -> Result<Vec<Task>> {
        let mut result = Vec::default();

        for page in 1.. {
            let paged_result = self.extract_issues_for_page(page)?;
            if paged_result.is_empty() {
                break;
            } else {
                result.extend(paged_result);
            }
        }

        Ok(result)
    }
}
