use super::*;

#[test]
fn list_targets() {
    let out = xcodekit(&["target", "list", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let targets = json["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["name"], "testproject");
}

#[test]
fn list_multitarget() {
    let out = xcodekit(&["target", "list", &fixture("project-multitarget.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["targets"].as_array().unwrap().len() > 1);
}

#[test]
fn show_by_name() {
    let out = xcodekit(&[
        "target", "show", &fixture("project.pbxproj"),
        "--target", "testproject", "--json",
    ]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["name"], "testproject");
    assert!(json["uuid"].is_string());
}

#[test]
fn show_not_found() {
    let out = xcodekit(&[
        "target", "show", &fixture("project.pbxproj"),
        "--target", "nonexistent", "--json",
    ]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("TARGET_NOT_FOUND"));
}
