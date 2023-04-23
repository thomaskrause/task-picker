# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- GitLab issues with a `due_date` were not parsed because it only provides a
  date, not a time. Add default time to date to make it compatible with our
  internal format.
- Avoid dependency to outdated time crate by disabling the "oldtime" feature in
  chrono.

### Changed

- GitLab source now uses the TODO list instead of assigned issues or merge
  requests. This is much more flexible, since anything can be a TODO item, e.g.
  the new "work items" which are sub-task like. Using the TODO-endpoint also
  means our also corresponds with the TODO page on GitLab. A disadvantage is
  that even when an issue is closed, you still have to mark the TODO as done.

## [0.2.0] - 2023-03-15

### Added

- GitLab source now also includes assigned merge requests as tasks.

### Fixed

- Update view after refreshing the tasks in the background without needing any
  other user interaction like mouse movement.
- Do not gray-out other tasks if the selected task has vanished.

## [0.1.0] - 2023-03-13

Initial release