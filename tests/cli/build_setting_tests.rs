use super::*;

#[test]
fn get_existing() {
    let out = xcodekit(&[
        "build", "setting", "get", &fixture("project.pbxproj"),
        "--target", "testproject",
        "--key", "PRODUCT_BUNDLE_IDENTIFIER",
        "--json",
    ]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["key"], "PRODUCT_BUNDLE_IDENTIFIER");
    assert!(json["value"].is_string());
}

#[test]
fn get_nonexistent_key() {
    let out = xcodekit(&[
        "build", "setting", "get", &fixture("project.pbxproj"),
        "--target", "testproject",
        "--key", "NONEXISTENT_KEY_12345",
    ]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("(not set)"));
}

#[test]
fn set_dry_run() {
    let out = xcodekit(&[
        "build", "setting", "set", &fixture("project.pbxproj"),
        "--target", "testproject",
        "--key", "SWIFT_VERSION",
        "--value", "6.0",
    ]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("dry-run"));
}

#[test]
fn set_dry_run_json() {
    let out = xcodekit(&[
        "build", "setting", "set", &fixture("project.pbxproj"),
        "--target", "testproject",
        "--key", "SWIFT_VERSION",
        "--value", "6.0",
        "--json",
    ]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["changed"], false);
}
