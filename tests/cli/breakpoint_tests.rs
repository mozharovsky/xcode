use super::*;

#[test]
fn list_from_file() {
    let out = xcodekit(&["breakpoint", "list", &fixture("breakpoints/Breakpoints_v2.xcbkptlist"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let bps = json["breakpoints"].as_array().unwrap();
    assert_eq!(bps.len(), 4);
    assert_eq!(bps[0]["filePath"], "MyApp/ViewController.swift");
    assert_eq!(bps[0]["line"], "42");
    assert_eq!(bps[0]["enabled"], "Yes");
}

#[test]
fn list_file_breakpoint() {
    let out = xcodekit(&["breakpoint", "list", &fixture("breakpoints/Breakpoints_v2.xcbkptlist"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let bps = json["breakpoints"].as_array().unwrap();
    let model_bp = &bps[1];
    assert_eq!(model_bp["filePath"], "MyApp/Model.swift");
    assert_eq!(model_bp["line"], "100");
    assert_eq!(model_bp["enabled"], "No");
    assert_eq!(model_bp["condition"], "count > 10");
}

#[test]
fn list_symbolic_breakpoint() {
    let out = xcodekit(&["breakpoint", "list", &fixture("breakpoints/Breakpoints_v2.xcbkptlist"), "--json"]);
    assert!(out.status.success());
    let json = json_stdout(&out);
    let bps = json["breakpoints"].as_array().unwrap();
    let symbolic = &bps[2];
    assert!(symbolic["filePath"].is_null());
    assert_eq!(symbolic["symbolName"], "objc_exception_throw");
}

#[test]
fn help() {
    let out = xcodekit(&["breakpoint", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("list"));
    assert!(text.contains("add"));
    assert!(text.contains("remove"));
}
