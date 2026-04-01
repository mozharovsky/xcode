use std::io::Write;
use std::process::{Command, Stdio};

use super::*;

fn batch(path: &str, ops_json: &str) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_xcodekit");
    let mut child = Command::new(bin)
        .args(["batch", path, "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(ops_json.as_bytes()).unwrap();

    child.wait_with_output().unwrap()
}

#[test]
fn single_set_dry_run() {
    let ops = r#"[{"command": "build setting set", "target": "testproject", "key": "MY_FLAG", "value": "YES"}]"#;
    let out = batch(&fixture("project.pbxproj"), ops);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["operationsExecuted"], 1);
    assert_eq!(json["operationsTotal"], 1);
    assert_eq!(json["changed"], true);
}

#[test]
fn multiple_operations() {
    let ops = r#"[
        {"command": "build setting set", "target": "testproject", "key": "A", "value": "1"},
        {"command": "build setting set", "target": "testproject", "key": "B", "value": "2"},
        {"command": "build setting set", "target": "testproject", "key": "C", "value": "3"}
    ]"#;
    let out = batch(&fixture("project.pbxproj"), ops);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["operationsExecuted"], 3);
    assert_eq!(json["operationsTotal"], 3);
}

#[test]
fn invalid_target_fails() {
    let ops = r#"[{"command": "build setting set", "target": "nonexistent", "key": "A", "value": "1"}]"#;
    let out = batch(&fixture("project.pbxproj"), ops);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("TARGET_NOT_FOUND"));
}

#[test]
fn invalid_json_fails() {
    let out = batch(&fixture("project.pbxproj"), "not json");
    assert!(!out.status.success());
    assert!(stderr(&out).contains("PARSE_ERROR"));
}

#[test]
fn empty_array() {
    let out = batch(&fixture("project.pbxproj"), "[]");
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["operationsExecuted"], 0);
    assert_eq!(json["operationsTotal"], 0);
}

#[test]
fn build_phase_ensure_in_batch() {
    let ops = r#"[{"command": "build phase ensure", "target": "testproject", "type": "sources"}]"#;
    let out = batch(&fixture("project.pbxproj"), ops);
    assert!(out.status.success());
    assert_eq!(json_stdout(&out)["operationsExecuted"], 1);
}

#[test]
fn add_script_in_batch() {
    let ops = r#"[{"command": "build phase add script", "target": "testproject", "name": "Test Script", "script": "echo hello"}]"#;
    let out = batch(&fixture("project.pbxproj"), ops);
    assert!(out.status.success());
    assert_eq!(json_stdout(&out)["operationsExecuted"], 1);
}

#[test]
fn target_duplicate_in_batch() {
    let ops = r#"[{"command": "target duplicate", "target": "testproject", "new-name": "testproject-copy"}]"#;
    let out = batch(&fixture("project.pbxproj"), ops);
    assert!(out.status.success());
    assert_eq!(json_stdout(&out)["operationsExecuted"], 1);
}

#[test]
fn mixed_operations() {
    let ops = r#"[
        {"command": "build setting set", "target": "testproject", "key": "X", "value": "1"},
        {"command": "build phase ensure", "target": "testproject", "type": "resources"},
        {"command": "framework add", "target": "testproject", "name": "UIKit"}
    ]"#;
    let out = batch(&fixture("project.pbxproj"), ops);
    assert!(out.status.success());
    assert_eq!(json_stdout(&out)["operationsExecuted"], 3);
}
