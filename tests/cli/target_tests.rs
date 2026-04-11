use super::*;

// ── Duplicate target ─────────────────────────────────────────

#[test]
fn duplicate_dry_run() {
    let out = xcodekit(&[
        "target",
        "duplicate",
        &fixture("project.pbxproj"),
        "--target",
        "testproject",
        "--new-name",
        "testproject-copy",
        "--json",
    ]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["uuid"].is_string());
    assert_eq!(json["changed"], false);
}

#[test]
fn duplicate_invalid_target() {
    let out = xcodekit(&[
        "target",
        "duplicate",
        &fixture("project.pbxproj"),
        "--target",
        "nonexistent",
        "--new-name",
        "copy",
    ]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("TARGET_NOT_FOUND"));
}

#[test]
fn duplicate_multitarget() {
    let out = xcodekit(&[
        "target",
        "duplicate",
        &fixture("project-multitarget.pbxproj"),
        "--target",
        "multitarget",
        "--new-name",
        "multitarget-clone",
        "--json",
    ]);
    assert!(out.status.success());
    assert!(json_stdout(&out)["uuid"].is_string());
}

// ── Remove ──────────────────────────────────────────────────

#[test]
fn remove_dry_run() {
    let out = xcodekit(&["target", "remove", &fixture("project.pbxproj"), "--target", "testproject", "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["changed"], false);
}

#[test]
fn remove_invalid_target() {
    let out = xcodekit(&["target", "remove", &fixture("project.pbxproj"), "--target", "nonexistent"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("TARGET_NOT_FOUND"));
}

// ── List / Show ──────────────────────────────────────────────

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
    let out = xcodekit(&["target", "show", &fixture("project.pbxproj"), "--target", "testproject", "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["name"], "testproject");
    assert!(json["uuid"].is_string());
}

#[test]
fn show_not_found() {
    let out = xcodekit(&["target", "show", &fixture("project.pbxproj"), "--target", "nonexistent", "--json"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("TARGET_NOT_FOUND"));
}
