use std::borrow::Cow;

use clap::Subcommand;
use xcodekit::project::XcodeProject;
use xcodekit::types::PlistValue;

use crate::output::{self, CliError};
use crate::resolve::resolve_target;

#[derive(Subcommand)]
pub enum BuildAction {
    /// Manage build settings
    Setting {
        #[command(subcommand)]
        action: SettingAction,
    },
    /// Manage build phases
    Phase {
        #[command(subcommand)]
        action: PhaseAction,
    },
}

#[derive(Subcommand)]
pub enum SettingAction {
    /// Get a build setting value
    Get {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        json: bool,
    },
    /// Set a build setting value
    Set {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        value: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Remove a build setting
    Remove {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum PhaseAction {
    /// Ensure a build phase exists on a target
    Ensure {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long, name = "type")]
        phase_type: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Add a file to a build phase
    #[command(name = "add-file")]
    AddFile {
        path: String,
        #[arg(long)]
        phase: String,
        #[arg(long, name = "file-ref")]
        file_ref: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
}

fn open(path: &str) -> Result<XcodeProject, CliError> {
    let resolved = crate::output::normalize_project_path(path);
    XcodeProject::open(&resolved).map_err(|e| {
        if e.contains("Failed to read") { CliError::file_not_found(path) }
        else { CliError::parse_error(&e) }
    })
}

fn save(project: &XcodeProject, path: &str) -> Result<(), CliError> {
    std::fs::write(path, project.to_pbxproj())
        .map_err(|e| CliError::new("WRITE_FAILED", e.to_string()))
}

fn map_phase_type(name: &str) -> String {
    match name.to_lowercase().as_str() {
        "sources" => "PBXSourcesBuildPhase",
        "frameworks" => "PBXFrameworksBuildPhase",
        "resources" => "PBXResourcesBuildPhase",
        "headers" => "PBXHeadersBuildPhase",
        "copyfiles" | "copy-files" => "PBXCopyFilesBuildPhase",
        "shellscript" | "shell-script" => "PBXShellScriptBuildPhase",
        other => other,
    }
    .to_string()
}

pub fn run(action: BuildAction) -> Result<(), CliError> {
    match action {
        BuildAction::Setting { action } => run_setting(action),
        BuildAction::Phase { action } => run_phase(action),
    }
}

fn run_setting(action: SettingAction) -> Result<(), CliError> {
    match action {
        SettingAction::Get { path, target, key, json } => {
            let project = open(&path)?;
            let uuid = resolve_target(&project, &target)?;
            let value = project.get_build_setting(&uuid, &key);

            if json {
                let v = value
                    .map(|v| serde_json::to_value(&v).unwrap_or_default())
                    .unwrap_or(serde_json::Value::Null);
                output::print_json(&serde_json::json!({ "key": key, "value": v }));
            } else {
                match value {
                    Some(v) => println!("{}", v.as_str().unwrap_or(&format!("{:?}", v))),
                    None => println!("(not set)"),
                }
            }
            Ok(())
        }

        SettingAction::Set { path, target, key, value, write, json } => {
            let mut project = open(&path)?;
            let uuid = resolve_target(&project, &target)?;
            project.set_build_setting(&uuid, &key, PlistValue::String(Cow::Owned(value.clone())));

            if write { save(&project, &path)?; }

            if json {
                output::print_json(&serde_json::json!({ "changed": write }));
            } else {
                println!("Set {} = {}{}",
                    key, value,
                    if write { "" } else { " (dry-run, use --write to save)" });
            }
            Ok(())
        }

        SettingAction::Remove { path, target, key, write, json } => {
            let mut project = open(&path)?;
            let uuid = resolve_target(&project, &target)?;
            project.remove_build_setting(&uuid, &key);

            if write { save(&project, &path)?; }

            if json {
                output::print_json(&serde_json::json!({ "changed": write }));
            } else {
                println!("Removed {}{}",
                    key,
                    if write { "" } else { " (dry-run, use --write to save)" });
            }
            Ok(())
        }
    }
}

fn run_phase(action: PhaseAction) -> Result<(), CliError> {
    match action {
        PhaseAction::Ensure { path, target, phase_type, write, json } => {
            let mut project = open(&path)?;
            let target_uuid = resolve_target(&project, &target)?;
            let isa = map_phase_type(&phase_type);
            let uuid = project.ensure_build_phase(&target_uuid, &isa)
                .ok_or_else(|| CliError::new("PHASE_FAILED", "Failed to ensure build phase"))?;

            if write { save(&project, &path)?; }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Build phase {} ({}){}", isa, uuid,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        PhaseAction::AddFile { path, phase, file_ref, write, json } => {
            let mut project = open(&path)?;
            let uuid = project.add_build_file(&phase, &file_ref)
                .ok_or_else(|| CliError::new("ADD_FAILED", "Failed to add build file"))?;

            if write { save(&project, &path)?; }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Added build file {}{}", uuid,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }
    }
}
