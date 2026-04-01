use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError};
use crate::resolve::resolve_target;

#[derive(Subcommand)]
pub enum DependencyAction {
    /// Add a dependency from one target to another
    Add {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long, name = "depends-on")]
        depends_on: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: DependencyAction) -> Result<(), CliError> {
    match action {
        DependencyAction::Add { path, target, depends_on, write, json } => {
            let mut project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let target_uuid = resolve_target(&project, &target)?;
            let dep_uuid = resolve_target(&project, &depends_on)?;
            let uuid = project.add_dependency(&target_uuid, &dep_uuid)
                .ok_or_else(|| CliError::new("ADD_FAILED", "Failed to add dependency"))?;

            if write {
                std::fs::write(&path, project.to_pbxproj())
                    .map_err(|e| CliError::new("WRITE_FAILED", e.to_string()))?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Added dependency ({}){}", uuid,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }
    }
}
