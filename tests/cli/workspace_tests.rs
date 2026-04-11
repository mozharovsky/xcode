use super::*;

#[test]
fn inspect_simple() {
    let out = xcodekit(&["workspace", "inspect", &fixture("workspaces/simple.xcworkspacedata"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let projects = json["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0], "group:App.xcodeproj");
}

#[test]
fn inspect_cocoapods() {
    let out = xcodekit(&["workspace", "inspect", &fixture("workspaces/cocoapods.xcworkspacedata"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let projects = json["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 2);
}

#[test]
fn list_projects() {
    let out = xcodekit(&["workspace", "list-projects", &fixture("workspaces/cocoapods.xcworkspacedata"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["projects"].as_array().unwrap().len(), 2);
}

#[test]
fn inspect_all_location_types() {
    let out = xcodekit(&["workspace", "inspect", &fixture("workspaces/all-location-types.xcworkspacedata"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert!(json["projects"].as_array().unwrap().len() >= 3);
}

#[test]
fn help() {
    let out = xcodekit(&["workspace", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("inspect"));
    assert!(text.contains("list-projects"));
    assert!(text.contains("add-project"));
    assert!(text.contains("remove-project"));
    assert!(text.contains("create"));
}
