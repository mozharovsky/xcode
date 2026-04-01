use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError, ErrorCode};
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
    /// Remove a file reference from the project
    Remove {
        path: String,
        #[arg(long)]
        file: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
}

fn open(path: &str) -> Result<XcodeProject, CliError> {
    XcodeProject::open(&crate::output::normalize_project_path(path)).map_err(|e| CliError::parse_error(&e))
}

fn save(project: &XcodeProject, path: &str) -> Result<(), CliError> {
    let resolved = crate::output::normalize_project_path(path);
    std::fs::write(&resolved, project.to_pbxproj()).map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))
}

pub fn run(action: FileAction) -> Result<(), CliError> {
    match action {
        FileAction::Add { path, group, file_path, write, json } => {
            let mut project = open(&path)?;
            let group_uuid = resolve_group(&project, &group)?;
            let uuid = project.add_file(&group_uuid, &file_path).map_err(|e| CliError::new(ErrorCode::AddFailed, e))?;

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": true }));
            } else {
                println!("Added file '{}' ({}){}", file_path, uuid, if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        FileAction::Remove { path, file, write, json } => {
            let mut project = open(&path)?;

            let file_uuid = resolve_file_ref(&project, &file)?;
            project.remove_file(&file_uuid).map_err(|e| CliError::new(ErrorCode::RemoveFailed, e))?;

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Removed file '{}'{}", file, if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }
    }
}

fn resolve_file_ref(project: &XcodeProject, query: &str) -> Result<String, CliError> {
    if query.len() == 24 && query.chars().all(|c| c.is_ascii_hexdigit()) {
        if project.get_object(query).is_some() {
            return Ok(query.to_string());
        }
        return Err(CliError::new(ErrorCode::ObjectNotFound, format!("File reference '{}' not found", query)));
    }

    let matches: Vec<_> = project
        .objects_by_isa("PBXFileReference")
        .iter()
        .filter(|f| {
            f.get_str("name") == Some(query)
                || f.get_str("path") == Some(query)
                || f.get_str("path").map(|p| p.ends_with(query)).unwrap_or(false)
        })
        .map(|f| f.uuid.clone())
        .collect();

    match matches.len() {
        0 => Err(CliError::new(ErrorCode::ObjectNotFound, format!("No file reference matching '{}'", query))),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(CliError::new(ErrorCode::AmbiguousReference, format!("Multiple files matched '{}'", query))),
    }
}
