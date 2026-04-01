/// CLI integration tests.
///
/// Run the xcodekit binary as a subprocess and verify output, exit codes,
/// and JSON structure against real fixture files.
use std::process::Command;

const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn xcodekit(args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_xcodekit");
    Command::new(bin).args(args).output().expect("failed to run xcodekit")
}

fn fixture(name: &str) -> String {
    format!("{}/{}", FIXTURES_DIR, name)
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn json_stdout(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_str(&stdout(output)).expect("stdout is not valid JSON")
}

// ── project inspect ────────────────────────────────────────────────

mod project_inspect {
    use super::*;

    #[test]
    fn human_output() {
        let out = xcodekit(&["project", "inspect", &fixture("project.pbxproj")]);
        assert!(out.status.success());
        let text = stdout(&out);
        assert!(text.contains("archiveVersion: 1"));
        assert!(text.contains("objectVersion:  46"));
        assert!(text.contains("testproject"));
    }

    #[test]
    fn json_output() {
        let out = xcodekit(&["project", "inspect", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["project"]["archiveVersion"], 1);
        assert_eq!(json["project"]["objectVersion"], 46);
        assert!(json["targets"].as_array().unwrap().len() > 0);
        assert!(json["stats"]["objectCount"].as_u64().unwrap() > 0);
    }

    #[test]
    fn file_not_found() {
        let out = xcodekit(&["project", "inspect", "nonexistent.pbxproj", "--json"]);
        assert!(!out.status.success());
        let err_text = stderr(&out);
        assert!(err_text.contains("FILE_NOT_FOUND"));
    }
}

// ── project targets ────────────────────────────────────────────────

mod project_targets {
    use super::*;

    #[test]
    fn human_output() {
        let out = xcodekit(&["project", "targets", &fixture("project.pbxproj")]);
        assert!(out.status.success());
        assert!(stdout(&out).contains("testproject"));
    }

    #[test]
    fn json_output() {
        let out = xcodekit(&["project", "targets", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        let targets = json["targets"].as_array().unwrap();
        assert!(!targets.is_empty());
        assert!(targets[0]["uuid"].is_string());
        assert!(targets[0]["name"].is_string());
        assert!(targets[0]["productType"].is_string());
    }
}

// ── project health ─────────────────────────────────────────────────

mod project_health {
    use super::*;

    #[test]
    fn clean_project() {
        let out = xcodekit(&["project", "health", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["healthy"], true);
        assert_eq!(json["orphanedReferenceCount"], 0);
    }

    #[test]
    fn malformed_project() {
        let out = xcodekit(&["project", "health", &fixture("malformed.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["healthy"], false);
        assert!(json["orphanedReferenceCount"].as_u64().unwrap() > 0);
        let orphans = json["orphanedReferences"].as_array().unwrap();
        assert!(orphans.iter().any(|o| o["orphanUuid"] == "3E1C2299F05049539341855D"));
    }
}

// ── project dump ───────────────────────────────────────────────────

mod project_dump {
    use super::*;

    #[test]
    fn produces_valid_json() {
        let out = xcodekit(&["project", "dump", &fixture("project.pbxproj")]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert!(json["archiveVersion"].is_number());
        assert!(json["objects"].is_object());
    }
}

// ── target list ────────────────────────────────────────────────────

mod target_list {
    use super::*;

    #[test]
    fn lists_targets() {
        let out = xcodekit(&["target", "list", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        let targets = json["targets"].as_array().unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0]["name"], "testproject");
    }

    #[test]
    fn multitarget_project() {
        let out = xcodekit(&["target", "list", &fixture("project-multitarget.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        let targets = json["targets"].as_array().unwrap();
        assert!(targets.len() > 1);
    }
}

// ── target show ────────────────────────────────────────────────────

mod target_show {
    use super::*;

    #[test]
    fn by_name() {
        let out = xcodekit(&[
            "target",
            "show",
            &fixture("project.pbxproj"),
            "--target",
            "testproject",
            "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["name"], "testproject");
        assert!(json["uuid"].is_string());
    }

    #[test]
    fn not_found() {
        let out = xcodekit(&[
            "target",
            "show",
            &fixture("project.pbxproj"),
            "--target",
            "nonexistent",
            "--json",
        ]);
        assert!(!out.status.success());
        let err = stderr(&out);
        assert!(err.contains("TARGET_NOT_FOUND"));
    }
}

// ── build setting ──────────────────────────────────────────────────

mod build_setting {
    use super::*;

    #[test]
    fn get_existing() {
        let out = xcodekit(&[
            "build",
            "setting",
            "get",
            &fixture("project.pbxproj"),
            "--target",
            "testproject",
            "--key",
            "PRODUCT_BUNDLE_IDENTIFIER",
            "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["key"], "PRODUCT_BUNDLE_IDENTIFIER");
        assert!(json["value"].is_string());
    }

    #[test]
    fn get_nonexistent_key() {
        let out = xcodekit(&[
            "build",
            "setting",
            "get",
            &fixture("project.pbxproj"),
            "--target",
            "testproject",
            "--key",
            "NONEXISTENT_KEY_12345",
        ]);
        assert!(out.status.success());
        assert!(stdout(&out).contains("(not set)"));
    }

    #[test]
    fn set_dry_run() {
        let out = xcodekit(&[
            "build",
            "setting",
            "set",
            &fixture("project.pbxproj"),
            "--target",
            "testproject",
            "--key",
            "SWIFT_VERSION",
            "--value",
            "6.0",
        ]);
        assert!(out.status.success());
        let text = stdout(&out);
        assert!(text.contains("dry-run"));
        assert!(text.contains("SWIFT_VERSION"));
    }

    #[test]
    fn set_dry_run_json() {
        let out = xcodekit(&[
            "build",
            "setting",
            "set",
            &fixture("project.pbxproj"),
            "--target",
            "testproject",
            "--key",
            "SWIFT_VERSION",
            "--value",
            "6.0",
            "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["changed"], false);
    }
}

// ── doctor ─────────────────────────────────────────────────────────

mod doctor {
    use super::*;

    #[test]
    fn orphans_clean() {
        let out = xcodekit(&["doctor", "orphans", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["orphanedReferenceCount"], 0);
    }

    #[test]
    fn orphans_malformed() {
        let out = xcodekit(&["doctor", "orphans", &fixture("malformed.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert!(json["orphanedReferenceCount"].as_u64().unwrap() > 0);
    }

    #[test]
    fn summary() {
        let out = xcodekit(&["doctor", "summary", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["healthy"], true);
        assert!(json["targetCount"].as_u64().unwrap() > 0);
        assert!(json["objectCount"].as_u64().unwrap() > 0);
    }
}

// ── group ──────────────────────────────────────────────────────────

mod group {
    use super::*;

    #[test]
    fn list_children_by_uuid() {
        let out = xcodekit(&["target", "list", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());

        let out = xcodekit(&["project", "inspect", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());
        let _json = json_stdout(&out);
    }
}

// ── object ─────────────────────────────────────────────────────────

mod object {
    use super::*;

    #[test]
    fn get_by_uuid() {
        let list_out = xcodekit(&["target", "list", &fixture("project.pbxproj"), "--json"]);
        let json = json_stdout(&list_out);
        let uuid = json["targets"][0]["uuid"].as_str().unwrap();

        let out = xcodekit(&["object", "get", &fixture("project.pbxproj"), "--uuid", uuid, "--json"]);
        assert!(out.status.success());
        let obj = json_stdout(&out);
        assert_eq!(obj["isa"], "PBXNativeTarget");
    }

    #[test]
    fn get_property() {
        let list_out = xcodekit(&["target", "list", &fixture("project.pbxproj"), "--json"]);
        let uuid = json_stdout(&list_out)["targets"][0]["uuid"]
            .as_str()
            .unwrap()
            .to_string();

        let out = xcodekit(&[
            "object",
            "get-property",
            &fixture("project.pbxproj"),
            "--uuid",
            &uuid,
            "--key",
            "name",
            "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["value"], "testproject");
    }

    #[test]
    fn not_found() {
        let out = xcodekit(&[
            "object",
            "get",
            &fixture("project.pbxproj"),
            "--uuid",
            "000000000000000000000000",
            "--json",
        ]);
        assert!(!out.status.success());
        assert!(stderr(&out).contains("OBJECT_NOT_FOUND"));
    }

    #[test]
    fn list_by_isa() {
        let out = xcodekit(&[
            "object",
            "list-by-isa",
            &fixture("project.pbxproj"),
            "--isa",
            "PBXGroup",
            "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        let objects = json["objects"].as_array().unwrap();
        assert!(!objects.is_empty());
    }
}

// ── sync group ─────────────────────────────────────────────────────

mod sync_group {
    use super::*;

    #[test]
    fn list_on_non_xcode16_project() {
        let out = xcodekit(&[
            "sync",
            "group",
            "list",
            &fixture("project.pbxproj"),
            "--target",
            "testproject",
            "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert!(json["paths"].as_array().unwrap().is_empty());
    }
}

// ── help ───────────────────────────────────────────────────────────

mod help {
    use super::*;

    #[test]
    fn top_level_help() {
        let out = xcodekit(&["--help"]);
        assert!(out.status.success());
        let text = stdout(&out);
        assert!(text.contains("project"));
        assert!(text.contains("target"));
        assert!(text.contains("build"));
        assert!(text.contains("doctor"));
    }

    #[test]
    fn project_help() {
        let out = xcodekit(&["project", "--help"]);
        assert!(out.status.success());
        let text = stdout(&out);
        assert!(text.contains("inspect"));
        assert!(text.contains("targets"));
        assert!(text.contains("health"));
        assert!(text.contains("dump"));
    }

    #[test]
    fn build_setting_help() {
        let out = xcodekit(&["build", "setting", "--help"]);
        assert!(out.status.success());
        let text = stdout(&out);
        assert!(text.contains("get"));
        assert!(text.contains("set"));
        assert!(text.contains("remove"));
    }
}

// ── version ────────────────────────────────────────────────────────

mod version {
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
        let json = json_stdout(&out);
        assert!(json["version"].is_string());
    }
}

// ── path normalization ─────────────────────────────────────────────

// ── spm ────────────────────────────────────────────────────────────

mod spm {
    use super::*;

    #[test]
    fn list_on_spm_project() {
        let out = xcodekit(&["spm", "list", &fixture("006-spm.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        let packages = json["packages"].as_array().unwrap();
        assert!(!packages.is_empty());
        assert!(packages[0]["location"].as_str().unwrap().contains("supabase"));
    }

    #[test]
    fn list_on_non_spm_project() {
        let out = xcodekit(&["spm", "list", &fixture("project.pbxproj"), "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert!(json["packages"].as_array().unwrap().is_empty());
    }

    #[test]
    fn add_remote_dry_run() {
        let out = xcodekit(&[
            "spm", "add-remote", &fixture("project.pbxproj"),
            "--url", "https://github.com/apple/swift-collections",
            "--version", "1.0.0",
            "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert!(json["uuid"].is_string());
        assert_eq!(json["changed"], true);
    }

    #[test]
    fn add_local_dry_run() {
        let out = xcodekit(&[
            "spm", "add-local", &fixture("project.pbxproj"),
            "--package-path", "../Packages/MyLib",
            "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert!(json["uuid"].is_string());
    }

    #[test]
    fn help() {
        let out = xcodekit(&["spm", "--help"]);
        assert!(out.status.success());
        let text = stdout(&out);
        assert!(text.contains("add-remote"));
        assert!(text.contains("add-local"));
        assert!(text.contains("add-product"));
        assert!(text.contains("remove-product"));
        assert!(text.contains("list"));
    }
}

// ── plist ──────────────────────────────────────────────────────────

mod plist {
    use super::*;

    fn plist_fixture() -> String {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
        let path = format!("{}/test_info.plist", dir);
        if !std::path::Path::new(&path).exists() {
            std::fs::write(&path, r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>TestApp</string>
	<key>CFBundleVersion</key>
	<string>1</string>
</dict>
</plist>"#).unwrap();
        }
        path
    }

    #[test]
    fn parse_plist() {
        let out = xcodekit(&["plist", "parse", &plist_fixture()]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["CFBundleName"], "TestApp");
        assert_eq!(json["CFBundleVersion"], "1");
    }

    #[test]
    fn roundtrip_plist() {
        let tmp_json = "/tmp/xcodekit_test_plist.json";
        let tmp_plist = "/tmp/xcodekit_test_output.plist";

        // Parse to JSON
        let out = xcodekit(&["plist", "parse", &plist_fixture()]);
        assert!(out.status.success());
        std::fs::write(tmp_json, stdout(&out)).unwrap();

        // Build back to plist
        let out = xcodekit(&["plist", "build", "--input", tmp_json, "--output", tmp_plist]);
        assert!(out.status.success());

        // Re-parse and verify
        let out = xcodekit(&["plist", "parse", tmp_plist]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["CFBundleName"], "TestApp");

        let _ = std::fs::remove_file(tmp_json);
        let _ = std::fs::remove_file(tmp_plist);
    }

    #[test]
    fn file_not_found() {
        let out = xcodekit(&["plist", "parse", "/nonexistent/file.plist"]);
        assert!(!out.status.success());
        assert!(stderr(&out).contains("FILE_NOT_FOUND"));
    }
}

// ── file remove ────────────────────────────────────────────────────

mod file_remove {
    use super::*;

    #[test]
    fn remove_dry_run() {
        // Find a file reference UUID first
        let list_out = xcodekit(&[
            "object", "list-by-isa", &fixture("project.pbxproj"),
            "--isa", "PBXFileReference", "--json",
        ]);
        let json = json_stdout(&list_out);
        let objects = json["objects"].as_array().unwrap();
        assert!(!objects.is_empty());
        let file_uuid = objects[0]["uuid"].as_str().unwrap();

        let out = xcodekit(&[
            "file", "remove", &fixture("project.pbxproj"),
            "--file", file_uuid, "--json",
        ]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert_eq!(json["changed"], true);
    }

    #[test]
    fn remove_not_found() {
        let out = xcodekit(&[
            "file", "remove", &fixture("project.pbxproj"),
            "--file", "000000000000000000000000", "--json",
        ]);
        assert!(!out.status.success());
    }

    #[test]
    fn help() {
        let out = xcodekit(&["file", "--help"]);
        assert!(out.status.success());
        let text = stdout(&out);
        assert!(text.contains("add"));
        assert!(text.contains("remove"));
    }
}

// ── group remove ───────────────────────────────────────────────────

mod group_remove {
    use super::*;

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
}

// ── stdin ──────────────────────────────────────────────────────────

mod stdin_support {
    use super::*;
    use std::process::{Command, Stdio};

    #[test]
    fn project_inspect_from_stdin() {
        let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
        let content = std::fs::read_to_string(format!("{}/project.pbxproj", fixtures)).unwrap();

        let bin = env!("CARGO_BIN_EXE_xcodekit");
        let mut child = Command::new(bin)
            .args(&["project", "inspect", "-", "--json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        use std::io::Write;
        child.stdin.take().unwrap().write_all(content.as_bytes()).unwrap();

        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());

        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(json["targets"].as_array().unwrap().len() > 0);
    }
}

// ── path normalization ─────────────────────────────────────────────

mod path_normalization {
    use super::*;

    #[test]
    fn xcodeproj_directory_resolves() {
        // Create a temporary .xcodeproj directory with a project.pbxproj symlink
        let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
        let xcodeproj_dir = format!("{}/test.xcodeproj", fixtures);
        let _ = std::fs::create_dir(&xcodeproj_dir);
        std::fs::copy(
            format!("{}/project.pbxproj", fixtures),
            format!("{}/project.pbxproj", xcodeproj_dir),
        )
        .unwrap();

        let out = xcodekit(&["project", "inspect", &xcodeproj_dir, "--json"]);
        assert!(out.status.success());
        let json = json_stdout(&out);
        assert!(json["targets"].as_array().unwrap().len() > 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(&xcodeproj_dir);
    }
}
