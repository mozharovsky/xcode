use super::*;

#[test]
fn xcodeproj_directory_resolves() {
    let xcodeproj_dir = format!("{}/test.xcodeproj", FIXTURES_DIR);
    let _ = std::fs::create_dir(&xcodeproj_dir);
    std::fs::copy(
        format!("{}/project.pbxproj", FIXTURES_DIR),
        format!("{}/project.pbxproj", xcodeproj_dir),
    )
    .unwrap();

    let out = xcodekit(&["project", "inspect", &xcodeproj_dir, "--json"]);
    assert!(out.status.success());
    assert!(json_stdout(&out)["targets"].as_array().unwrap().len() > 0);

    let _ = std::fs::remove_dir_all(&xcodeproj_dir);
}
