use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError};
use crate::resolve::resolve_group;

#[derive(Subcommand)]
pub enum FileAction {
    /// Add a file reference to a group
    Add {
        path: String,
        #[arg(long)]
        group: String,
        #[arg(long, name = "file-path")]
        file_path: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: FileAction) -> Result<(), CliError> {
    match action {
        FileAction::Add { path, group, file_path, write, json } => {
            let mut project = XcodeProject::open(&crate::output::normalize_project_path(&path))
                .map_err(|e| CliError::parse_error(&e))?;
            let group_uuid = resolve_group(&project, &group)?;
            let uuid = project.add_file(&group_uuid, &file_path)
                .ok_or_else(|| CliError::new("ADD_FAILED", "Failed to add file"))?;

            if write {
                std::fs::write(&path, project.to_pbxproj())
                    .map_err(|e| CliError::new("WRITE_FAILED", e.to_string()))?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Added file '{}' ({}){}", file_path, uuid,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }
    }
}
