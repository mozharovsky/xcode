use super::*;

#[test]
fn list_on_spm_project() {
    let out = xcodekit(&["spm", "list", &fixture("006-spm.pbxproj"), "--json"]);
    assert!(out.status.success());
    let packages = json_stdout(&out)["packages"].as_array().unwrap().clone();
    assert!(!packages.is_empty());
    assert!(packages[0]["location"].as_str().unwrap().contains("supabase"));
}

#[test]
fn list_on_non_spm_project() {
    let out = xcodekit(&["spm", "list", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    assert!(json_stdout(&out)["packages"].as_array().unwrap().is_empty());
}

#[test]
fn add_remote_dry_run() {
    let out = xcodekit(&[
        "spm",
        "add-remote",
        &fixture("project.pbxproj"),
        "--url",
        "https://github.com/apple/swift-collections",
        "--version",
        "1.0.0",
        "--json",
    ]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["uuid"].is_string());
    assert_eq!(json["changed"], true);
}

#[test]
fn add_local_dry_run() {
    let out =
        xcodekit(&["spm", "add-local", &fixture("project.pbxproj"), "--package-path", "../Packages/MyLib", "--json"]);
    assert!(out.status.success());
    assert!(json_stdout(&out)["uuid"].is_string());
}

#[test]
fn help() {
    let out = xcodekit(&["spm", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("add-remote"));
    assert!(text.contains("add-local"));
    assert!(text.contains("add-product"));
    assert!(text.contains("remove-product"));
    assert!(text.contains("list"));
}
