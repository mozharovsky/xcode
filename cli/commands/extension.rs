use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError, ErrorCode};
use crate::resolve::resolve_target;

#[derive(Subcommand)]
pub enum ExtensionAction {
    /// Embed an extension target into a host app target
    Embed {
        path: String,
        #[arg(long)]
        host: String,
        #[arg(long)]
        extension: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: ExtensionAction) -> Result<(), CliError> {
    match action {
        ExtensionAction::Embed { path, host, extension, write, json } => {
            let mut project = XcodeProject::open(&crate::output::normalize_project_path(&path))
                .map_err(|e| CliError::parse_error(&e))?;
            let host_uuid = resolve_target(&project, &host)?;
            let ext_uuid = resolve_target(&project, &extension)?;
            let uuid = project
                .embed_extension(&host_uuid, &ext_uuid)
                .ok_or_else(|| CliError::new(ErrorCode::EmbedFailed, "Failed to embed extension"))?;

            if write {
                std::fs::write(&path, project.to_pbxproj())
                    .map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Embedded extension ({}){}", uuid, if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }
    }
}
