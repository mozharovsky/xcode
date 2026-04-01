use std::borrow::Cow;

use clap::Subcommand;
use xcodekit::project::XcodeProject;
use xcodekit::types::PlistValue;

use crate::output::{self, CliError, ErrorCode};
use crate::resolve::resolve_target;
use crate::resolve::PhaseType;

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
        #[arg(long = "type", value_enum)]
        phase_type: PhaseType,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Add a run script build phase to a target
    #[command(name = "add-script")]
    AddScript {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        script: String,
        #[arg(long, default_value = "/bin/sh")]
        shell: String,
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
        if e.contains("Failed to read") {
            CliError::file_not_found(path)
        } else {
            CliError::parse_error(&e)
        }
    })
}

fn save(project: &XcodeProject, path: &str) -> Result<(), CliError> {
    std::fs::write(path, project.to_pbxproj()).map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))
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
                let v = value.map(|v| serde_json::to_value(&v).unwrap_or_default()).unwrap_or(serde_json::Value::Null);
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

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "changed": write }));
            } else {
                println!("Set {} = {}{}", key, value, if write { "" } else { " (dry-run, use --write to save)" });
            }
            Ok(())
        }

        SettingAction::Remove { path, target, key, write, json } => {
            let mut project = open(&path)?;
            let uuid = resolve_target(&project, &target)?;
            project.remove_build_setting(&uuid, &key);

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "changed": write }));
            } else {
                println!("Removed {}{}", key, if write { "" } else { " (dry-run, use --write to save)" });
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
            let isa = phase_type.as_isa();
            let uuid = project
                .ensure_build_phase(&target_uuid, isa)
                .ok_or_else(|| CliError::new(ErrorCode::PhaseFailed, "Failed to ensure build phase"))?;

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Build phase {} ({}){}", isa, uuid, if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        PhaseAction::AddScript { path, target, name, script, shell, write, json } => {
            let mut project = open(&path)?;
            let target_uuid = resolve_target(&project, &target)?;
            let uuid = project
                .add_run_script_phase(&target_uuid, &name, &script, Some(&shell))
                .ok_or_else(|| CliError::new(ErrorCode::PhaseFailed, "Failed to add run script phase"))?;

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Added run script phase '{}' ({}){}", name, uuid, if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        PhaseAction::AddFile { path, phase, file_ref, write, json } => {
            let mut project = open(&path)?;
            let uuid = project
                .add_build_file(&phase, &file_ref)
                .ok_or_else(|| CliError::new(ErrorCode::AddFailed, "Failed to add build file"))?;

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Added build file {}{}", uuid, if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }
    }
}
