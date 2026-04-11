use super::*;

#[test]
fn orphans_clean() {
    let out = xcodekit(&["doctor", "orphans", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["orphanedReferenceCount"], 0);
}

#[test]
fn orphans_malformed() {
    let out = xcodekit(&["doctor", "orphans", &fixture("malformed.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["orphanedReferenceCount"].as_u64().unwrap() > 0);
}

#[test]
fn fix_dry_run() {
    let out = xcodekit(&["doctor", "fix", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["removedCount"].is_number());
    assert_eq!(json["changed"], false);
}

#[test]
fn summary() {
    let out = xcodekit(&["doctor", "summary", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["healthy"], true);
    assert!(json["targetCount"].as_u64().unwrap() > 0);
    assert!(json["objectCount"].as_u64().unwrap() > 0);
}
