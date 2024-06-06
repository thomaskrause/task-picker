use anyhow::{anyhow, Context, Result};
use base64::prelude::*;
use chrono::{DateTime, Local, NaiveDate, Utc};
use json::{array, JsonValue};
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::tasks::Task;

use super::OPENPROJECT_ICON;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct OpenProjectSource {
    #[serde(skip)]
    agent: Agent,
    pub name: String,
    pub server_url: String,
    pub token: String,
}

impl Default for OpenProjectSource {
    fn default() -> Self {
        Self {
            agent: Agent::new(),
            name: "OpenProject".to_string(),
            server_url: "https://community.openproject.org".to_string(),
            token: Default::default(),
        }
    }
}

impl OpenProjectSource {
    fn create_task(&self, work_package: &JsonValue) -> Result<Option<Task>> {
        if let JsonValue::Object(work_package) = work_package {
            let title = work_package["subject"].as_str().unwrap_or("<unknown>");
            let id = work_package["id"]
                .as_i64()
                .context("'id' field in response is not an integer")?;
            let url = format!("{}/work_packages/{id}/activity", self.server_url);

            let project = work_package["_links"]["project"]["title"].to_string();

            let created = if let Some(c) = work_package["createdAt"].as_str() {
                let created_utc: DateTime<Utc> = DateTime::parse_from_rfc3339(c)?.into();
                Some(created_utc)
            } else {
                None
            };

            let start: Option<DateTime<Utc>> = work_package["startDate"]
                .as_str()
                .map(|due_date| NaiveDate::parse_from_str(due_date, "%Y-%m-%d"))
                .transpose()?
                .and_then(|due_date| due_date.and_hms_opt(0, 0, 0))
                .map(|due_date| DateTime::from_naive_utc_and_offset(due_date, Utc));

            let can_start = if let Some(start) = start {
                let start: DateTime<Local> = DateTime::from(start);
                Local::now().cmp(&start).is_ge()
            } else {
                true
            };

            if can_start {
                let due: Option<DateTime<Utc>> = work_package["dueDate"]
                    .as_str()
                    .map(|due_date| NaiveDate::parse_from_str(due_date, "%Y-%m-%d"))
                    .transpose()?
                    .and_then(|due_date| due_date.and_hms_opt(0, 0, 0))
                    .map(|due_date| DateTime::from_naive_utc_and_offset(due_date, Utc));

                let t = Task {
                    project: format!("{} {}", OPENPROJECT_ICON, project),
                    title: title.to_string(),
                    description: url,
                    due,
                    created,
                    id: Some(id.to_string()),
                };
                Ok(Some(t))
            } else {
                Ok(None)
            }
        } else {
            Err(anyhow!("Response is not a JSON object"))
        }
    }

    pub fn query_tasks(&self, secret: Option<&str>) -> Result<Vec<Task>> {
        let mut result = Vec::default();

        let basic_auth = secret.map(|secret| format!("apikey:{}", &secret));

        // Query all statuses that count as "closed".
        let mut request = self
            .agent
            .get(&format!("{}/api/v3/statuses", self.server_url));
        if let Some(basic_auth) = &basic_auth {
            request = request.set(
                "Authorization",
                &format!("Basic {}", &BASE64_STANDARD.encode(basic_auth.clone())),
            )
        }
        let response = request.call()?;
        let body = response.into_string()?;
        let closed_statuses =
            if let JsonValue::Array(elements) = &json::parse(&body)?["_embedded"]["elements"] {
                elements
                    .iter()
                    .filter(|e| e["isClosed"].as_bool().unwrap_or(false))
                    .filter_map(|e| e["id"].as_usize())
                    .collect()
            } else {
                Vec::default()
            };

        // Get the user ID for the provided acccess token
        let mut request = self
            .agent
            .get(&format!("{}/api/v3/users/me", self.server_url));
        if let Some(basic_auth) = &basic_auth {
            request = request.set(
                "Authorization",
                &format!("Basic {}", &BASE64_STANDARD.encode(basic_auth.clone())),
            )
        }
        let response = request.call()?;
        let body = response.into_string()?;

        let user_id = json::parse(&body)?["id"].as_usize().unwrap_or(0);
        // Filter by work packages that are assigned to the use and are not closed
        let filter_param = array! [
            {"assignee": {"operator": "=", "values": [user_id]}},
            {"type": {"operator": "=", "values": [1]}},
            {"status": {"operator": "!", "values": closed_statuses.clone()}}
        ];

        let mut request = self
            .agent
            .get(&format!("{}/api/v3/work_packages", self.server_url))
            .query("filters", &filter_param.to_string());
        if let Some(basic_auth) = &basic_auth {
            request = request.set(
                "Authorization",
                &format!("Basic {}", &BASE64_STANDARD.encode(basic_auth.clone())),
            )
        }
        let response = request.call()?;
        let body = response.into_string()?;
        let work_package_collection = json::parse(&body)?;

        if let JsonValue::Array(elements) = &work_package_collection["_embedded"]["elements"] {
            for e in elements {
                if let Some(task) = self.create_task(e)? {
                    result.push(task);
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests;
