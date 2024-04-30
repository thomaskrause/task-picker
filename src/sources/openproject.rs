use anyhow::{Context, Result};
use base64::prelude::*;
use json::{array, JsonValue};
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
            server_url: "https://community.openproject.org".to_string(),
            user_id: String::default(),
            token: Default::default(),
        }
    }
}

impl OpenProjectSource {
    pub fn query_tasks(&self) -> Result<Vec<Task>> {
        let mut result = Vec::default();

        // TODO: make the type and status configurable.
        let filter_param = array! [
            {"assignee": {"operator": "=", "values": [self.user_id.clone()]}},
            {"type": {"operator": "=", "values": [1]}},
            {"status": {"operator": "!", "values": [12]}}
        ];
        let basic_auth = format!("apikey:{}", &self.token);

        let request = self
            .agent
            .get(&format!("{}/api/v3/work_packages", self.server_url))
            .query("filters", &filter_param.to_string())
            .set(
                "Authorization",
                &format!("Basic {}", &BASE64_STANDARD.encode(basic_auth)),
            );
        let response = request.call()?;
        let body = response.into_string()?;

        if let JsonValue::Object(work_package_collection) = json::parse(&body)? {
            if let Some(JsonValue::Object(embedded)) = work_package_collection.get("_embedded") {
                if let Some(JsonValue::Array(elements)) = embedded.get("elements") {
                    for e in elements {
                        if let JsonValue::Object(e) = e {
                            let title = e
                                .get("subject")
                                .and_then(|subject| subject.as_str())
                                .unwrap_or("<unknown>");
                            let id = e
                                .get("id")
                                .context("Missing 'id' field in response.")?
                                .as_i64()
                                .context("'id' field in response is not an integer")?;
                            let url = format!("{}/work_packages/{id}/activity", self.server_url);
                            // TODO: extract time information and description
                            let t = Task {
                                project: self.name.clone(),
                                title: title.to_string(),
                                description: url,
                                due: None,
                                created: None,
                                id: Some(id.to_string()),
                            };
                            result.push(t);
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}
