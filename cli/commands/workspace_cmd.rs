use clap::Subcommand;
use xcodekit::workspace::Workspace;

use crate::output::{self, CliError, ErrorCode};

#[derive(Subcommand)]
pub enum WorkspaceAction {
    /// Inspect workspace contents
    Inspect {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// List projects in a workspace
    #[command(name = "list-projects")]
    ListProjects {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Add a project to a workspace
    #[command(name = "add-project")]
    AddProject {
        path: String,
        #[arg(long, name = "project-path")]
        project_path: String,
        #[arg(long)]
        json: bool,
    },
    /// Remove a project from a workspace
    #[command(name = "remove-project")]
    RemoveProject {
        path: String,
        #[arg(long, name = "project-path")]
        project_path: String,
        #[arg(long)]
        json: bool,
    },
    /// Create a new empty workspace
    Create {
        path: String,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: WorkspaceAction) -> Result<(), CliError> {
    match action {
        WorkspaceAction::Inspect { path, json } => {
            let ws = Workspace::from_file(&path).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            let projects = ws.get_project_paths();
            if json {
                output::print_json(&serde_json::json!({
                    "version": ws.version,
                    "projects": projects,
                    "itemCount": ws.items.len(),
                }));
            } else {
                println!("Workspace (version {})", ws.version.as_deref().unwrap_or("?"));
                for p in &projects {
                    println!("  {}", p);
                }
            }
            Ok(())
        }

        WorkspaceAction::ListProjects { path, json } => {
            let ws = Workspace::from_file(&path).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            let projects = ws.get_project_paths();
            if json {
                output::print_json(&serde_json::json!({ "projects": projects }));
            } else {
                for p in &projects {
                    println!("{}", p);
                }
            }
            Ok(())
        }

        WorkspaceAction::AddProject { path, project_path, json } => {
            let mut ws = Workspace::from_file(&path).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            let location = format!("group:{}", project_path);
            ws.add_project(&location);
            ws.save(&path).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Added {} to workspace", project_path);
            }
            Ok(())
        }

        WorkspaceAction::RemoveProject { path, project_path, json } => {
            let mut ws = Workspace::from_file(&path).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            let location = format!("group:{}", project_path);
            let removed = ws.remove_project(&location);
            if !removed {
                return Err(CliError::new(
                    ErrorCode::ObjectNotFound,
                    format!("Project '{}' not found in workspace", project_path),
                ));
            }
            ws.save(&path).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Removed {} from workspace", project_path);
            }
            Ok(())
        }

        WorkspaceAction::Create { path, json } => {
            let ws = Workspace::create_empty();
            ws.save(&path).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "created": path }));
            } else {
                println!("Created workspace at {}", path);
            }
            Ok(())
        }
    }
}
