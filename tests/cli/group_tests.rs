use super::*;

#[test]
fn list_children() {
    let out = xcodekit(&["project", "inspect", &fixture("project.pbxproj"), "--json"]);
    assert!(out.status.success());
    let _json = json_stdout(&out);
}

#[test]
fn remove_not_found() {
    let out = xcodekit(&[
        "group", "remove", &fixture("project.pbxproj"),
        "--group", "NonexistentGroup", "--json",
    ]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("GROUP_NOT_FOUND"));
}

#[test]
fn help() {
    let out = xcodekit(&["group", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("add"));
    assert!(text.contains("remove"));
    assert!(text.contains("list-children"));
}
