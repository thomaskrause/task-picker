use serde::{Deserialize, Serialize};
use ureq::Agent;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GitLabSource {
    #[serde(skip)]
    agent: Agent,
    pub name: String,
    pub server_url: String,
    pub user_id: String,
    pub token: String,
}

impl Default for GitLabSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            name: "GitLab".to_string(),
            server_url: Default::default(),
            user_id: Default::default(),
            token: Default::default(),
        }
    }
}
