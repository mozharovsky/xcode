use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError};
use crate::resolve::resolve_group;

#[derive(Subcommand)]
pub enum GroupAction {
    /// Add a new group as a child of a parent group
    Add {
        path: String,
        #[arg(long)]
        parent: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Remove a group from the project
    Remove {
        path: String,
        #[arg(long)]
        group: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// List children of a group
    #[command(name = "list-children")]
    ListChildren {
        path: String,
        #[arg(long)]
        group: String,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: GroupAction) -> Result<(), CliError> {
    match action {
        GroupAction::Add { path, parent, name, write, json } => {
            let mut project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let parent_uuid = resolve_group(&project, &parent)?;
            let uuid = project.add_group(&parent_uuid, &name)
                .ok_or_else(|| CliError::new("ADD_FAILED", "Failed to add group"))?;

            if write {
                std::fs::write(&path, project.to_pbxproj())
                    .map_err(|e| CliError::new("WRITE_FAILED", e.to_string()))?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!("Added group '{}' ({}){}", name, uuid,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        GroupAction::Remove { path, group, write, json } => {
            let mut project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let group_uuid = resolve_group(&project, &group)?;
            let changed = project.remove_group(&group_uuid);

            if !changed {
                return Err(CliError::new("REMOVE_FAILED", format!("Failed to remove group '{}'", group)));
            }

            if write {
                let resolved = crate::output::normalize_project_path(&path);
                std::fs::write(&resolved, project.to_pbxproj())
                    .map_err(|e| CliError::new("WRITE_FAILED", e.to_string()))?;
            }

            if json {
                output::print_json(&serde_json::json!({ "changed": changed }));
            } else {
                println!("Removed group '{}'{}", group,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        GroupAction::ListChildren { path, group, json } => {
            let project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let group_uuid = resolve_group(&project, &group)?;
            let children = project.get_group_children(&group_uuid);

            let entries: Vec<_> = children.iter()
                .map(|uuid| {
                    let name = project.get_object(uuid)
                        .and_then(|o| o.get_str("name").or(o.get_str("path")))
                        .unwrap_or("")
                        .to_string();
                    let isa = project.get_object(uuid).map(|o| o.isa.as_str()).unwrap_or("");
                    serde_json::json!({ "uuid": uuid, "name": name, "isa": isa })
                })
                .collect();

            if json {
                output::print_json(&serde_json::json!({ "children": entries }));
            } else {
                for e in &entries {
                    println!("{} ({})", e["name"].as_str().unwrap_or(""), e["isa"].as_str().unwrap_or(""));
                }
            }
            Ok(())
        }
    }
}
