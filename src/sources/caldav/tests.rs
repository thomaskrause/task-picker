use chrono::Datelike;

use super::parse_caldav_date;

#[test]
fn test_date_without_time() {
    let t = parse_caldav_date("20240412").unwrap();
    assert_eq!(t.date_naive().year(), 2024);
    assert_eq!(t.date_naive().month0(), 3);
}
