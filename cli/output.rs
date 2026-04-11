use std::fmt;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use serde::Serialize;

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(v: bool) {
    VERBOSE.store(v, Ordering::Relaxed);
}

#[allow(dead_code)]
pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

#[allow(dead_code)]
pub fn verbose_log(msg: &str) {
    if is_verbose() {
        eprintln!("[xcodekit] {}", msg);
    }
}

pub fn verbose_timer(label: &str) -> VerboseTimer {
    VerboseTimer { label: label.to_string(), start: Instant::now() }
}

pub struct VerboseTimer {
    label: String,
    start: Instant,
}

impl Drop for VerboseTimer {
    fn drop(&mut self) {
        if is_verbose() {
            let elapsed = self.start.elapsed();
            eprintln!("[xcodekit] {} in {:.1}ms", self.label, elapsed.as_secs_f64() * 1000.0);
        }
    }
}

/// Normalize a path: if it ends with `.xcodeproj`, append `/project.pbxproj`.
pub fn normalize_project_path(path: &str) -> String {
    if Path::new(path).extension().and_then(|e| e.to_str()) == Some("xcodeproj") {
        format!("{}/project.pbxproj", path)
    } else {
        path.to_string()
    }
}

/// Auto-detect a .xcodeproj or .pbxproj in the current directory.
#[allow(dead_code)]
pub fn auto_detect_project() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    if let Ok(entries) = std::fs::read_dir(&cwd) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("xcodeproj") {
                let pbxproj = path.join("project.pbxproj");
                if pbxproj.exists() {
                    return Some(pbxproj.to_string_lossy().to_string());
                }
            }
        }
    }
    if cwd.join("project.pbxproj").exists() {
        return Some(cwd.join("project.pbxproj").to_string_lossy().to_string());
    }
    None
}

/// Save with optional .backup file creation.
pub fn save_with_backup(path: &str, content: &str) -> Result<(), CliError> {
    let backup_path = format!("{}.backup", path);
    if Path::new(path).exists() {
        std::fs::copy(path, &backup_path)
            .map_err(|e| CliError::new(ErrorCode::WriteFailed, format!("Failed to create backup: {}", e)))?;
        verbose_log(&format!("Backup saved to {}", backup_path));
    }
    std::fs::write(path, content).map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))
}

/// Read project content from a path or stdin (when path is "-").
pub fn read_project_input(path: &str) -> Result<(String, String), CliError> {
    if path == "-" {
        let mut content = String::new();
        std::io::stdin()
            .read_to_string(&mut content)
            .map_err(|e| CliError::new(ErrorCode::StdinError, format!("Failed to read stdin: {}", e)))?;
        Ok(("<stdin>".to_string(), content))
    } else {
        let resolved = normalize_project_path(path);
        let content = std::fs::read_to_string(&resolved).map_err(|_| CliError::file_not_found(path))?;
        Ok((resolved, content))
    }
}

pub fn print_json<T: Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string()));
}

pub fn print_error(err: &CliError) {
    let json = serde_json::json!({
        "error": {
            "code": err.code.to_string(),
            "message": err.message,
        }
    });
    eprintln!("{}", serde_json::to_string_pretty(&json).unwrap());
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    FileNotFound,
    ParseError,
    TargetNotFound,
    GroupNotFound,
    ObjectNotFound,
    WriteFailed,
    AddFailed,
    RemoveFailed,
    CreateFailed,
    DuplicateFailed,
    PhaseFailed,
    EmbedFailed,
    StdinError,
    SerializeError,
    BuildError,
    PackageNotFound,
    AmbiguousReference,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::FileNotFound => "FILE_NOT_FOUND",
            Self::ParseError => "PARSE_ERROR",
            Self::TargetNotFound => "TARGET_NOT_FOUND",
            Self::GroupNotFound => "GROUP_NOT_FOUND",
            Self::ObjectNotFound => "OBJECT_NOT_FOUND",
            Self::WriteFailed => "WRITE_FAILED",
            Self::AddFailed => "ADD_FAILED",
            Self::RemoveFailed => "REMOVE_FAILED",
            Self::CreateFailed => "CREATE_FAILED",
            Self::DuplicateFailed => "DUPLICATE_FAILED",
            Self::PhaseFailed => "PHASE_FAILED",
            Self::EmbedFailed => "EMBED_FAILED",
            Self::StdinError => "STDIN_ERROR",
            Self::SerializeError => "SERIALIZE_ERROR",
            Self::BuildError => "BUILD_ERROR",
            Self::PackageNotFound => "PACKAGE_NOT_FOUND",
            Self::AmbiguousReference => "AMBIGUOUS_REFERENCE",
        })
    }
}

#[derive(Debug)]
pub struct CliError {
    pub code: ErrorCode,
    pub message: String,
}

impl CliError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        CliError { code, message: message.into() }
    }

    pub fn file_not_found(path: &str) -> Self {
        Self::new(ErrorCode::FileNotFound, format!("File not found: {}", path))
    }

    pub fn parse_error(detail: &str) -> Self {
        Self::new(ErrorCode::ParseError, format!("Failed to parse project: {}", detail))
    }

    pub fn target_not_found(query: &str) -> Self {
        Self::new(ErrorCode::TargetNotFound, format!("Target '{}' was not found", query))
    }

    pub fn group_not_found(query: &str) -> Self {
        Self::new(ErrorCode::GroupNotFound, format!("Group '{}' was not found", query))
    }

    pub fn object_not_found(uuid: &str) -> Self {
        Self::new(ErrorCode::ObjectNotFound, format!("Object '{}' was not found", uuid))
    }
}
