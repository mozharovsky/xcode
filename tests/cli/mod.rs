use std::process::Command;

pub const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

pub fn xcodekit(args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_xcodekit");
    Command::new(bin)
        .args(args)
        .output()
        .expect("failed to run xcodekit")
}

pub fn fixture(name: &str) -> String {
    format!("{}/{}", FIXTURES_DIR, name)
}

pub fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

pub fn json_stdout(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_str(&stdout(output)).expect("stdout is not valid JSON")
}

mod project_tests;
mod target_tests;
mod build_setting_tests;
mod doctor_tests;
mod object_tests;
mod spm_tests;
mod plist_tests;
mod file_tests;
mod group_tests;
mod sync_group_tests;
mod stdin_tests;
mod version_tests;
mod path_tests;
mod help_tests;
