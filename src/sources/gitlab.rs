use anyhow::Result;
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
        dbg!(assigned_issues);
        Ok(result)
    }
}
