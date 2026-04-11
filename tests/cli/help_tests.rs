use super::*;

#[test]
fn top_level() {
    let out = xcodekit(&["--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("project"));
    assert!(text.contains("target"));
    assert!(text.contains("build"));
    assert!(text.contains("doctor"));
}

#[test]
fn project() {
    let out = xcodekit(&["project", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("inspect"));
    assert!(text.contains("targets"));
    assert!(text.contains("health"));
    assert!(text.contains("dump"));
}

#[test]
fn build_setting() {
    let out = xcodekit(&["build", "setting", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("get"));
    assert!(text.contains("set"));
    assert!(text.contains("remove"));
}
