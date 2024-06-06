mod caldav;
mod github;
mod gitlab;
mod openproject;

pub use caldav::CalDavSource;
pub use github::GitHubSource;
pub use gitlab::GitLabSource;
use keyring::Entry;
pub use openproject::OpenProjectSource;

use serde::{Deserialize, Serialize};

pub const CALDAV_ICON: &str = egui_phosphor::regular::CALENDAR;
pub const GITHUB_ICON: &str = egui_phosphor::regular::GITHUB_LOGO;
pub const GITLAB_ICON: &str = egui_phosphor::regular::GITLAB_LOGO;
pub const OPENPROJECT_ICON: &str = egui_phosphor::regular::INFINITY;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
pub enum TaskSource {
    CalDav(CalDavSource),
    GitHub(GitHubSource),
    GitLab(GitLabSource),
    OpenProject(OpenProjectSource),
}

impl TaskSource {
    pub fn name(&self) -> &str {
        match self {
            TaskSource::CalDav(s) => s.calendar_name.as_str(),
            TaskSource::GitHub(s) => s.name.as_str(),
            TaskSource::GitLab(s) => s.name.as_str(),
            TaskSource::OpenProject(s) => s.name.as_str(),
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            TaskSource::CalDav(_) => "CalDAV",
            TaskSource::GitHub(_) => "GitHub",
            TaskSource::GitLab(_) => "GitLab",
            TaskSource::OpenProject(_) => "OpenProject",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            TaskSource::CalDav(_) => CALDAV_ICON,
            TaskSource::GitHub(_) => GITHUB_ICON,
            TaskSource::GitLab(_) => GITLAB_ICON,
            TaskSource::OpenProject(_) => OPENPROJECT_ICON,
        }
    }

    /// Returns the secret (e.g. a password or a token) for this task source.
    pub fn secret(&self) -> Option<String> {
        let keyring_entry = Entry::new("task-picker", self.name()).ok()?;
        let secret = keyring_entry.get_password().ok()?;
        Some(secret)
    }
}
