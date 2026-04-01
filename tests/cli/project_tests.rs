use super::*;

#[test]
fn inspect_human_output() {
    let out = xcodekit(&["project", "inspect", &fixture("project.pbxproj")]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("archiveVersion: 1"));
    assert!(text.contains("objectVersion:  46"));
    assert!(text.contains("testproject"));
}

#[test]
fn inspect_json_output() {
    let out = xcodekit(&["project", "inspect", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["project"]["archiveVersion"], 1);
    assert_eq!(json["project"]["objectVersion"], 46);
    assert!(json["targets"].as_array().unwrap().len() > 0);
    assert!(json["stats"]["objectCount"].as_u64().unwrap() > 0);
}

#[test]
fn inspect_file_not_found() {
    let out = xcodekit(&["project", "inspect", "nonexistent.pbxproj", "--json"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("FILE_NOT_FOUND"));
}

#[test]
fn targets_human() {
    let out = xcodekit(&["project", "targets", &fixture("project.pbxproj")]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("testproject"));
}

#[test]
fn targets_json() {
    let out = xcodekit(&["project", "targets", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let targets = json["targets"].as_array().unwrap();
    assert!(!targets.is_empty());
    assert!(targets[0]["uuid"].is_string());
    assert!(targets[0]["name"].is_string());
}

#[test]
fn health_clean() {
    let out = xcodekit(&["project", "health", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["healthy"], true);
    assert_eq!(json["orphanedReferenceCount"], 0);
}

#[test]
fn health_malformed() {
    let out = xcodekit(&["project", "health", &fixture("malformed.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["healthy"], false);
    assert!(json["orphanedReferenceCount"].as_u64().unwrap() > 0);
}

#[test]
fn dump_valid_json() {
    let out = xcodekit(&["project", "dump", &fixture("project.pbxproj")]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["archiveVersion"].is_number());
    assert!(json["objects"].is_object());
}
