use std::borrow::Cow;
use std::io::Read;

use clap::Args;
use serde::Deserialize;
use xcodekit::project::XcodeProject;
use xcodekit::types::PlistValue;

use crate::output::{self, CliError, ErrorCode};
use crate::resolve::{resolve_group, resolve_target, PhaseType};

#[derive(Args)]
pub struct BatchArgs {
    /// Path to .pbxproj or .xcodeproj
    path: String,
    /// Write changes to disk
    #[arg(long)]
    write: bool,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Deserialize)]
#[serde(tag = "command")]
enum Operation {
    #[serde(rename = "build setting set")]
    BuildSettingSet { target: String, key: String, value: String },
    #[serde(rename = "build setting remove")]
    BuildSettingRemove { target: String, key: String },
    #[serde(rename = "file add")]
    FileAdd { group: String, path: String },
    #[serde(rename = "file remove")]
    FileRemove { file: String },
    #[serde(rename = "group add")]
    GroupAdd { parent: String, name: String },
    #[serde(rename = "group remove")]
    GroupRemove { group: String },
    #[serde(rename = "target rename")]
    TargetRename {
        target: String,
        #[serde(alias = "new-name")]
        new_name: String,
    },
    #[serde(rename = "target create native")]
    TargetCreateNative {
        name: String,
        #[serde(alias = "product-type")]
        product_type: String,
        #[serde(alias = "bundle-id")]
        bundle_id: String,
    },
    #[serde(rename = "target duplicate")]
    TargetDuplicate {
        target: String,
        #[serde(alias = "new-name")]
        new_name: String,
    },
    #[serde(rename = "dependency add")]
    DependencyAdd {
        target: String,
        #[serde(alias = "depends-on")]
        depends_on: String,
    },
    #[serde(rename = "extension embed")]
    ExtensionEmbed { host: String, extension: String },
    #[serde(rename = "framework add")]
    FrameworkAdd { target: String, name: String },
    #[serde(rename = "build phase ensure")]
    BuildPhaseEnsure {
        target: String,
        #[serde(rename = "type")]
        phase_type: PhaseType,
    },
    #[serde(rename = "build phase add script")]
    BuildPhaseAddScript {
        target: String,
        name: String,
        script: String,
        #[serde(default = "default_shell")]
        shell: String,
    },
    #[serde(rename = "spm add remote")]
    SpmAddRemote { url: String, version: String },
    #[serde(rename = "spm add local")]
    SpmAddLocal { path: String },
    #[serde(rename = "spm add product")]
    SpmAddProduct { target: String, product: String, package: String },
}

fn default_shell() -> String {
    "/bin/sh".to_string()
}

pub fn run(args: BatchArgs) -> Result<(), CliError> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| CliError::new(ErrorCode::StdinError, format!("Failed to read stdin: {}", e)))?;

    let operations: Vec<Operation> = serde_json::from_str(&input)
        .map_err(|e| CliError::new(ErrorCode::ParseError, format!("Invalid batch input: {}", e)))?;

    let resolved = output::normalize_project_path(&args.path);
    let mut project = XcodeProject::open(&resolved).map_err(|e| {
        if e.contains("Failed to read") {
            CliError::file_not_found(&args.path)
        } else {
            CliError::parse_error(&e)
        }
    })?;

    let total = operations.len();
    let mut executed = 0;

    for (i, op) in operations.iter().enumerate() {
        execute(&mut project, op).map_err(|e| CliError::new(e.code, format!("Operation {}: {}", i, e.message)))?;
        executed += 1;
    }

    let changed = executed > 0;

    if args.write && changed {
        std::fs::write(&resolved, project.to_pbxproj())
            .map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))?;
    }

    if args.json {
        output::print_json(&serde_json::json!({
            "changed": changed,
            "operationsExecuted": executed,
            "operationsTotal": total,
        }));
    } else {
        println!(
            "Executed {}/{} operations{}",
            executed,
            total,
            if args.write && changed {
                ""
            } else if changed {
                " (dry-run, use --write to save)"
            } else {
                " (no changes)"
            }
        );
    }

    Ok(())
}

fn execute(project: &mut XcodeProject, op: &Operation) -> Result<(), CliError> {
    match op {
        Operation::BuildSettingSet { target, key, value } => {
            let uuid = resolve_target(project, target)?;
            project.set_build_setting(&uuid, key, PlistValue::String(Cow::Owned(value.clone())));
            Ok(())
        }
        Operation::BuildSettingRemove { target, key } => {
            let uuid = resolve_target(project, target)?;
            project.remove_build_setting(&uuid, key);
            Ok(())
        }
        Operation::FileAdd { group, path } => {
            let uuid = resolve_group(project, group)?;
            project.add_file(&uuid, path).ok_or_else(|| CliError::new(ErrorCode::AddFailed, "Failed to add file"))?;
            Ok(())
        }
        Operation::FileRemove { file } => {
            if !project.remove_file(file) {
                return Err(CliError::new(ErrorCode::RemoveFailed, format!("File '{}' not found", file)));
            }
            Ok(())
        }
        Operation::GroupAdd { parent, name } => {
            let uuid = resolve_group(project, parent)?;
            project.add_group(&uuid, name).ok_or_else(|| CliError::new(ErrorCode::AddFailed, "Failed to add group"))?;
            Ok(())
        }
        Operation::GroupRemove { group } => {
            let uuid = resolve_group(project, group)?;
            project.remove_group(&uuid);
            Ok(())
        }
        Operation::TargetRename { target, new_name } => {
            let uuid = resolve_target(project, target)?;
            let old_name = project.get_target_name(&uuid).unwrap_or_default();
            project.rename_target(&uuid, &old_name, new_name);
            Ok(())
        }
        Operation::TargetCreateNative { name, product_type, bundle_id } => {
            project
                .create_native_target(name, product_type, bundle_id)
                .ok_or_else(|| CliError::new(ErrorCode::CreateFailed, "Failed to create target"))?;
            Ok(())
        }
        Operation::TargetDuplicate { target, new_name } => {
            let uuid = resolve_target(project, target)?;
            project
                .duplicate_target(&uuid, new_name)
                .ok_or_else(|| CliError::new(ErrorCode::DuplicateFailed, "Failed to duplicate target"))?;
            Ok(())
        }
        Operation::DependencyAdd { target, depends_on } => {
            let t = resolve_target(project, target)?;
            let d = resolve_target(project, depends_on)?;
            project
                .add_dependency(&t, &d)
                .ok_or_else(|| CliError::new(ErrorCode::AddFailed, "Failed to add dependency"))?;
            Ok(())
        }
        Operation::ExtensionEmbed { host, extension } => {
            let h = resolve_target(project, host)?;
            let e = resolve_target(project, extension)?;
            project
                .embed_extension(&h, &e)
                .ok_or_else(|| CliError::new(ErrorCode::EmbedFailed, "Failed to embed extension"))?;
            Ok(())
        }
        Operation::FrameworkAdd { target, name } => {
            let uuid = resolve_target(project, target)?;
            project
                .add_framework(&uuid, name)
                .ok_or_else(|| CliError::new(ErrorCode::AddFailed, "Failed to add framework"))?;
            Ok(())
        }
        Operation::BuildPhaseEnsure { target, phase_type } => {
            let uuid = resolve_target(project, target)?;
            let isa = phase_type.as_isa();
            project
                .ensure_build_phase(&uuid, isa)
                .ok_or_else(|| CliError::new(ErrorCode::PhaseFailed, "Failed to ensure build phase"))?;
            Ok(())
        }
        Operation::BuildPhaseAddScript { target, name, script, shell } => {
            let uuid = resolve_target(project, target)?;
            project
                .add_run_script_phase(&uuid, name, script, Some(shell))
                .ok_or_else(|| CliError::new(ErrorCode::PhaseFailed, "Failed to add run script phase"))?;
            Ok(())
        }
        Operation::SpmAddRemote { url, version } => {
            project
                .add_remote_swift_package(url, version)
                .ok_or_else(|| CliError::new(ErrorCode::AddFailed, "Failed to add package"))?;
            Ok(())
        }
        Operation::SpmAddLocal { path } => {
            project
                .add_local_swift_package(path)
                .ok_or_else(|| CliError::new(ErrorCode::AddFailed, "Failed to add package"))?;
            Ok(())
        }
        Operation::SpmAddProduct { target, product, package } => {
            let uuid = resolve_target(project, target)?;
            project
                .add_swift_package_product(&uuid, product, package)
                .ok_or_else(|| CliError::new(ErrorCode::AddFailed, "Failed to add product"))?;
            Ok(())
        }
    }
}
