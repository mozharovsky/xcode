use std::io::Write;
use std::process::{Command, Stdio};

use super::*;

#[test]
fn project_inspect_from_stdin() {
    let content = std::fs::read_to_string(fixture("project.pbxproj")).unwrap();

    let bin = env!("CARGO_BIN_EXE_xcodekit");
    let mut child = Command::new(bin)
        .args(&["project", "inspect", "-", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(content.as_bytes()).unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["targets"].as_array().unwrap().len() > 0);
}
