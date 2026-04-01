use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError, ErrorCode};
use crate::resolve::resolve_target;

#[derive(Subcommand)]
pub enum SyncAction {
    /// Manage file system sync groups (Xcode 16+)
    Group {
        #[command(subcommand)]
        action: SyncGroupAction,
    },
}

#[derive(Subcommand)]
pub enum SyncGroupAction {
    /// Add a file system sync group to a target
    Add {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long, name = "sync-path")]
        sync_path: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// List file system sync group paths for a target
    List {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: SyncAction) -> Result<(), CliError> {
    match action {
        SyncAction::Group { action } => run_group(action),
    }
}

fn run_group(action: SyncGroupAction) -> Result<(), CliError> {
    match action {
        SyncGroupAction::Add { path, target, sync_path, write, json } => {
            let mut project = XcodeProject::open(&crate::output::normalize_project_path(&path))
                .map_err(|e| CliError::parse_error(&e))?;
            let target_uuid = resolve_target(&project, &target)?;
            let uuid = project
                .add_file_system_sync_group(&target_uuid, &sync_path)
                .map_err(|e| CliError::new(ErrorCode::AddFailed, e))?;

            if write {
                std::fs::write(&path, project.to_pbxproj())
                    .map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Added sync group '{}' ({}){}", sync_path, uuid, if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        SyncGroupAction::List { path, target, json } => {
            let project = XcodeProject::open(&crate::output::normalize_project_path(&path))
                .map_err(|e| CliError::parse_error(&e))?;
            let target_uuid = resolve_target(&project, &target)?;
            let paths = project.get_target_sync_group_paths(&target_uuid);

            if json {
                output::print_json(&serde_json::json!({ "paths": paths }));
            } else if paths.is_empty() {
                println!("No sync groups");
            } else {
                for p in &paths {
                    println!("{}", p);
                }
            }
            Ok(())
        }
    }
}
