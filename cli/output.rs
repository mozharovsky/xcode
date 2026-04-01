use std::io::Read;
use std::path::Path;

use serde::Serialize;

/// Normalize a path: if it ends with `.xcodeproj`, append `/project.pbxproj`.
pub fn normalize_project_path(path: &str) -> String {
    if Path::new(path).extension().and_then(|e| e.to_str()) == Some("xcodeproj") {
        format!("{}/project.pbxproj", path)
    } else {
        path.to_string()
    }
}

/// Read project content from a path or stdin (when path is "-").
pub fn read_project_input(path: &str) -> Result<(String, String), CliError> {
    if path == "-" {
        let mut content = String::new();
        std::io::stdin()
            .read_to_string(&mut content)
            .map_err(|e| CliError::new("STDIN_ERROR", format!("Failed to read stdin: {}", e)))?;
        Ok(("<stdin>".to_string(), content))
    } else {
        let resolved = normalize_project_path(path);
        let content = std::fs::read_to_string(&resolved).map_err(|_| CliError::file_not_found(path))?;
        Ok((resolved, content))
    }
}

pub fn print_json<T: Serialize>(value: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
    );
}

pub fn print_error(err: &CliError) {
    let json = serde_json::json!({
        "error": {
            "code": err.code,
            "message": err.message,
        }
    });
    eprintln!("{}", serde_json::to_string_pretty(&json).unwrap());
}

#[derive(Debug)]
pub struct CliError {
    pub code: String,
    pub message: String,
}

impl CliError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        CliError {
            code: code.to_string(),
            message: message.into(),
        }
    }

    pub fn file_not_found(path: &str) -> Self {
        Self::new("FILE_NOT_FOUND", format!("File not found: {}", path))
    }

    pub fn parse_error(detail: &str) -> Self {
        Self::new("PARSE_ERROR", format!("Failed to parse project: {}", detail))
    }

    pub fn target_not_found(query: &str) -> Self {
        Self::new("TARGET_NOT_FOUND", format!("Target '{}' was not found", query))
    }

    pub fn group_not_found(query: &str) -> Self {
        Self::new("GROUP_NOT_FOUND", format!("Group '{}' was not found", query))
    }

    pub fn object_not_found(uuid: &str) -> Self {
        Self::new("OBJECT_NOT_FOUND", format!("Object '{}' was not found", uuid))
    }
}
