use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError, ErrorCode};
use crate::resolve::resolve_target;

#[derive(Subcommand)]
pub enum FrameworkAction {
    /// Add a system framework to a target
    Add {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: FrameworkAction) -> Result<(), CliError> {
    match action {
        FrameworkAction::Add { path, target, name, write, json } => {
            let mut project = XcodeProject::open(&crate::output::normalize_project_path(&path))
                .map_err(|e| CliError::parse_error(&e))?;
            let target_uuid = resolve_target(&project, &target)?;
            let uuid =
                project.add_framework(&target_uuid, &name).map_err(|e| CliError::new(ErrorCode::AddFailed, e))?;

            if write {
                std::fs::write(&path, project.to_pbxproj())
                    .map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Added framework '{}' ({}){}", name, uuid, if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }
    }
}
