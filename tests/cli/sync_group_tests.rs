use super::*;

#[test]
fn list_on_non_xcode16_project() {
    let out = xcodekit(&[
        "sync", "group", "list", &fixture("project.pbxproj"),
        "--target", "testproject", "--json",
    ]);
    assert!(out.status.success());
    assert!(json_stdout(&out)["paths"].as_array().unwrap().is_empty());
}
