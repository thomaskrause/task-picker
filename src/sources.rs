mod caldav;
mod github;
mod gitlab;

pub use caldav::CalDavSource;
pub use github::GitHubSource;
use serde::{Serialize, Deserialize};

#[non_exhaustive]
#[derive(Serialize,Deserialize, Clone)]
pub enum TaskSource {
    CalDav(CalDavSource),
    GitHub(GitHubSource),
}

impl TaskSource {
    pub fn name(&self) -> &str {
        match self {
            TaskSource::CalDav(s) => s.calendar_name.as_str(),
            TaskSource::GitHub(s) => s.name.as_str(),
        }
    }
}