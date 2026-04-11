use super::*;

#[test]
fn parse_simple() {
    let out = xcodekit(&["xcconfig", "parse", &fixture("xcconfigs/simple.xcconfig"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let entries = json["entries"].as_array().unwrap();
    let settings: Vec<_> = entries.iter().filter(|e| e["type"] == "setting").collect();
    assert_eq!(settings.len(), 3);
    assert!(settings.iter().any(|s| s["key"] == "PRODUCT_NAME" && s["value"] == "MyApp"));
    assert!(settings.iter().any(|s| s["key"] == "SWIFT_VERSION" && s["value"] == "5.0"));
}

#[test]
fn parse_conditional() {
    let out = xcodekit(&["xcconfig", "parse", &fixture("xcconfigs/conditional.xcconfig"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let entries = json["entries"].as_array().unwrap();
    let with_conditions: Vec<_> = entries
        .iter()
        .filter(|e| e["type"] == "setting" && e["conditions"].as_array().map(|a| !a.is_empty()).unwrap_or(false))
        .collect();
    assert!(!with_conditions.is_empty());
    let first_cond = &with_conditions[0]["conditions"].as_array().unwrap()[0];
    assert_eq!(first_cond["key"], "sdk");
}

#[test]
fn parse_with_include() {
    let out = xcodekit(&["xcconfig", "parse", &fixture("xcconfigs/optional-include.xcconfig"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let entries = json["entries"].as_array().unwrap();
    let includes: Vec<_> = entries.iter().filter(|e| e["type"] == "include").collect();
    assert!(!includes.is_empty());
}

#[test]
fn flatten_simple() {
    let out = xcodekit(&["xcconfig", "flatten", &fixture("xcconfigs/simple.xcconfig"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    assert_eq!(json["PRODUCT_NAME"], "MyApp");
    assert_eq!(json["SWIFT_VERSION"], "5.0");
    assert_eq!(json["PRODUCT_BUNDLE_IDENTIFIER"], "com.example.myapp");
}

#[test]
fn help() {
    let out = xcodekit(&["xcconfig", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("parse"));
    assert!(text.contains("flatten"));
}
