use super::*;

#[test]
fn human_output() {
    let out = xcodekit(&["version"]);
    assert!(out.status.success());
    assert!(stdout(&out).starts_with("xcodekit "));
}

#[test]
fn json_output() {
    let out = xcodekit(&["version", "--json"]);
    assert!(out.status.success());
    assert!(json_stdout(&out)["version"].is_string());
}
