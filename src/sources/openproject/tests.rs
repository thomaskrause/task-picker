use crate::sources::OPENPROJECT_ICON;

use super::OpenProjectSource;

#[test]
fn parse_open_project_task() {
    let json_body = r#"
    {
        "derivedStartDate": null,
        "derivedDueDate": null,
        "spentTime": "PT0S",
        "laborCosts": "0,00 EUR",
        "materialCosts": "0,00 EUR",
        "overallCosts": "0,00 EUR",
        "_type": "WorkPackage",
        "id": 33013,
        "lockVersion": 3,
        "subject": "Test title",
        "description": {
          "format": "markdown",
          "raw": "This is a *description*.",
          "html": "<p>This is a <em>description</em></p>"
        },
        "scheduleManually": false,
        "startDate": "2024-05-01",
        "dueDate": null,
        "estimatedTime": null,
        "derivedEstimatedTime": null,
        "derivedRemainingTime": null,
        "duration": null,
        "ignoreNonWorkingDays": false,
        "percentageDone": null,
        "derivedPercentageDone": null,
        "createdAt": "2024-05-10T05:36:41.859Z",
        "updatedAt": "2024-05-30T12:18:44.591Z",
        "readonly": false,
        "customField4": null,
        "customField3": null,
        "_links": {
            "project": {"href": "/api/v3/projects/1337", "title": "Test project"}
        }
      }
    "#;

    let work_package = json::parse(&json_body).unwrap();

    let source = OpenProjectSource::default();

    let task = source.create_task(&work_package).unwrap();
    assert_eq!(true, task.is_some());
    let task = task.unwrap();

    assert_eq!("Test title", task.title);
    assert_eq!(
        "https://community.openproject.org/work_packages/33013/activity",
        task.description
    );
    assert_eq!(format!("{} Test project", OPENPROJECT_ICON), task.project);
}
