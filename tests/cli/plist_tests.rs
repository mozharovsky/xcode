use super::*;

fn plist_fixture() -> String {
    let dir = FIXTURES_DIR;
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
fn parse() {
    let out = xcodekit(&["plist", "parse", &plist_fixture()]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["CFBundleName"], "TestApp");
    assert_eq!(json["CFBundleVersion"], "1");
}

#[test]
fn roundtrip() {
    let tmp_json = "/tmp/xcodekit_test_plist.json";
    let tmp_plist = "/tmp/xcodekit_test_output.plist";

    let out = xcodekit(&["plist", "parse", &plist_fixture()]);
    assert!(out.status.success());
    std::fs::write(tmp_json, stdout(&out)).unwrap();

    let out = xcodekit(&["plist", "build", "--input", tmp_json, "--output", tmp_plist]);
    assert!(out.status.success());

    let out = xcodekit(&["plist", "parse", tmp_plist]);
    assert!(out.status.success());
    assert_eq!(json_stdout(&out)["CFBundleName"], "TestApp");

    let _ = std::fs::remove_file(tmp_json);
    let _ = std::fs::remove_file(tmp_plist);
}

#[test]
fn file_not_found() {
    let out = xcodekit(&["plist", "parse", "/nonexistent/file.plist"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("FILE_NOT_FOUND"));
}
