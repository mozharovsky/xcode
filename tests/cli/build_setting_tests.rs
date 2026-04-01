use super::*;

// ── Phase ensure ─────────────────────────────────────────────

#[test]
fn phase_ensure_dry_run() {
    let out = xcodekit(&[
        "build",
        "phase",
        "ensure",
        &fixture("project.pbxproj"),
        "--target",
        "testproject",
        "--type",
        "sources",
        "--json",
    ]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["uuid"].is_string());
    assert_eq!(json["changed"], false);
}

#[test]
fn phase_ensure_invalid_target() {
    let out = xcodekit(&[
        "build",
        "phase",
        "ensure",
        &fixture("project.pbxproj"),
        "--target",
        "nonexistent",
        "--type",
        "sources",
    ]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("TARGET_NOT_FOUND"));
}

// ── Add script ───────────────────────────────────────────────

#[test]
fn add_script_dry_run() {
    let out = xcodekit(&[
        "build",
        "phase",
        "add-script",
        &fixture("project.pbxproj"),
        "--target",
        "testproject",
        "--name",
        "My Script Phase",
        "--script",
        "echo hello",
        "--json",
    ]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["uuid"].is_string());
    assert_eq!(json["changed"], false);
}

#[test]
fn add_script_custom_shell() {
    let out = xcodekit(&[
        "build",
        "phase",
        "add-script",
        &fixture("project.pbxproj"),
        "--target",
        "testproject",
        "--name",
        "Zsh Script",
        "--script",
        "echo zsh",
        "--shell",
        "/bin/zsh",
        "--json",
    ]);
    assert!(out.status.success());
    assert!(json_stdout(&out)["uuid"].is_string());
}

#[test]
fn add_script_invalid_target() {
    let out = xcodekit(&[
        "build",
        "phase",
        "add-script",
        &fixture("project.pbxproj"),
        "--target",
        "nonexistent",
        "--name",
        "Test",
        "--script",
        "echo test",
    ]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("TARGET_NOT_FOUND"));
}

// ── Build settings ───────────────────────────────────────────

#[test]
fn get_existing() {
    let out = xcodekit(&[
        "build",
        "setting",
        "get",
        &fixture("project.pbxproj"),
        "--target",
        "testproject",
        "--key",
        "PRODUCT_BUNDLE_IDENTIFIER",
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
        "build",
        "setting",
        "get",
        &fixture("project.pbxproj"),
        "--target",
        "testproject",
        "--key",
        "NONEXISTENT_KEY_12345",
    ]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("(not set)"));
}

#[test]
fn set_dry_run() {
    let out = xcodekit(&[
        "build",
        "setting",
        "set",
        &fixture("project.pbxproj"),
        "--target",
        "testproject",
        "--key",
        "SWIFT_VERSION",
        "--value",
        "6.0",
    ]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("dry-run"));
}

#[test]
fn set_dry_run_json() {
    let out = xcodekit(&[
        "build",
        "setting",
        "set",
        &fixture("project.pbxproj"),
        "--target",
        "testproject",
        "--key",
        "SWIFT_VERSION",
        "--value",
        "6.0",
        "--json",
    ]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["changed"], false);
}
