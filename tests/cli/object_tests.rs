use super::*;

#[test]
fn get_by_uuid() {
    let list_out = xcodekit(&["target", "list", &fixture("project.pbxproj"), "--json"]);
    let uuid = json_stdout(&list_out)["targets"][0]["uuid"].as_str().unwrap().to_string();

    let out = xcodekit(&["object", "get", &fixture("project.pbxproj"), "--uuid", &uuid, "--json"]);
    assert!(out.status.success());
    assert_eq!(json_stdout(&out)["isa"], "PBXNativeTarget");
}

#[test]
fn get_property() {
    let list_out = xcodekit(&["target", "list", &fixture("project.pbxproj"), "--json"]);
    let uuid = json_stdout(&list_out)["targets"][0]["uuid"].as_str().unwrap().to_string();

    let out =
        xcodekit(&["object", "get-property", &fixture("project.pbxproj"), "--uuid", &uuid, "--key", "name", "--json"]);
    assert!(out.status.success());
    assert_eq!(json_stdout(&out)["value"], "testproject");
}

#[test]
fn not_found() {
    let out = xcodekit(&["object", "get", &fixture("project.pbxproj"), "--uuid", "000000000000000000000000", "--json"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("OBJECT_NOT_FOUND"));
}

#[test]
fn list_by_isa() {
    let out = xcodekit(&["object", "list-by-isa", &fixture("project.pbxproj"), "--isa", "PBXGroup", "--json"]);
    assert!(out.status.success());
    assert!(!json_stdout(&out)["objects"].as_array().unwrap().is_empty());
}
