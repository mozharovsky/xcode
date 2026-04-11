use super::*;

#[test]
fn remove_dry_run() {
    let list_out =
        xcodekit(&["object", "list-by-isa", &fixture("project.pbxproj"), "--isa", "PBXFileReference", "--json"]);
    let objects = json_stdout(&list_out)["objects"].as_array().unwrap().clone();
    assert!(!objects.is_empty());
    let file_uuid = objects[0]["uuid"].as_str().unwrap();

    let out = xcodekit(&["file", "remove", &fixture("project.pbxproj"), "--file", file_uuid, "--json"]);
    assert!(out.status.success());
    assert_eq!(json_stdout(&out)["changed"], true);
}

#[test]
fn remove_not_found() {
    let out =
        xcodekit(&["file", "remove", &fixture("project.pbxproj"), "--file", "000000000000000000000000", "--json"]);
    assert!(!out.status.success());
}

#[test]
fn list_files() {
    let out = xcodekit(&["file", "list", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let files = json["files"].as_array().unwrap();
    assert!(!files.is_empty());
    assert!(files[0]["uuid"].is_string());
    assert!(files[0]["path"].is_string());
}

#[test]
fn help() {
    let out = xcodekit(&["file", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("add"));
    assert!(text.contains("remove"));
    assert!(text.contains("list"));
    assert!(text.contains("move"));
    assert!(text.contains("add-to-target"));
    assert!(text.contains("remove-from-target"));
}
