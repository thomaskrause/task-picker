use anyhow::Result;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::tasks::Task;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct OpenProjectSource {
    #[serde(skip)]
    agent: Agent,
    pub name: String,
    pub server_url: String,
    pub user_id: String,
    pub token: String,
}

impl Default for OpenProjectSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            name: "OpenProject".to_string(),
            server_url: "https://community.openproject.org/api/v3/".to_string(),
            user_id: String::default(),
            token: Default::default(),
        }
    }
}

impl OpenProjectSource {
    pub fn query_tasks(&self) -> Result<Vec<Task>> {
        let mut result = Vec::default();

        todo!();

        Ok(result)
    }
}
