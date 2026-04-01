use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError, ErrorCode};

#[derive(Subcommand)]
pub enum ProjectAction {
    /// Show project summary: targets, object counts, health
    Inspect {
        /// Path to .pbxproj file
        path: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List all targets in the project
    Targets {
        /// Path to .pbxproj file
        path: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Check project health (orphaned references, etc.)
    Health {
        /// Path to .pbxproj file
        path: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Dump the full project structure as JSON
    Dump {
        /// Path to .pbxproj file
        path: String,
    },
}

fn open_project(path: &str) -> Result<XcodeProject, CliError> {
    if path == "-" {
        let (_, content) = output::read_project_input(path)?;
        XcodeProject::from_plist(&content).map_err(|e| CliError::parse_error(&e))
    } else {
        let resolved = output::normalize_project_path(path);
        XcodeProject::open(&resolved).map_err(|e| {
            if e.contains("Failed to read") {
                CliError::file_not_found(path)
            } else {
                CliError::parse_error(&e)
            }
        })
    }
}

pub fn run(action: ProjectAction) -> Result<(), CliError> {
    match action {
        ProjectAction::Inspect { path, json } => inspect(&path, json),
        ProjectAction::Targets { path, json } => targets(&path, json),
        ProjectAction::Health { path, json } => health(&path, json),
        ProjectAction::Dump { path } => dump(&path),
    }
}

fn inspect(path: &str, json: bool) -> Result<(), CliError> {
    let project = open_project(path)?;
    let orphans = project.find_orphaned_references();

    let targets: Vec<_> = project
        .native_targets()
        .iter()
        .map(|t| {
            serde_json::json!({
                "uuid": t.uuid,
                "name": t.get_str("name").unwrap_or(""),
                "productType": t.get_str("productType").unwrap_or(""),
            })
        })
        .collect();

    let objects_count = project.objects().count();

    if json {
        output::print_json(&serde_json::json!({
            "project": {
                "path": path,
                "archiveVersion": project.archive_version,
                "objectVersion": project.object_version,
            },
            "targets": targets,
            "stats": {
                "objectCount": objects_count,
                "orphanedReferenceCount": orphans.len(),
            },
        }));
    } else {
        println!("Project: {}", path);
        println!("  archiveVersion: {}", project.archive_version);
        println!("  objectVersion:  {}", project.object_version);
        println!("  objects:        {}", objects_count);
        println!("  orphans:        {}", orphans.len());
        println!();
        println!("Targets:");
        for t in &targets {
            println!("  {} ({})", t["name"].as_str().unwrap_or(""), t["productType"].as_str().unwrap_or(""));
        }
    }

    Ok(())
}

fn targets(path: &str, json: bool) -> Result<(), CliError> {
    let project = open_project(path)?;

    let targets: Vec<_> = project
        .native_targets()
        .iter()
        .map(|t| {
            serde_json::json!({
                "uuid": t.uuid,
                "name": t.get_str("name").unwrap_or(""),
                "productType": t.get_str("productType").unwrap_or(""),
            })
        })
        .collect();

    if json {
        output::print_json(&serde_json::json!({ "targets": targets }));
    } else {
        for t in &targets {
            println!("{}", t["name"].as_str().unwrap_or(""));
        }
    }

    Ok(())
}

fn health(path: &str, json: bool) -> Result<(), CliError> {
    let project = open_project(path)?;
    let orphans = project.find_orphaned_references();

    if json {
        let orphan_list: Vec<_> = orphans
            .iter()
            .map(|o| {
                serde_json::json!({
                    "referrerUuid": o.referrer_uuid,
                    "referrerIsa": o.referrer_isa,
                    "property": o.property,
                    "orphanUuid": o.orphan_uuid,
                })
            })
            .collect();
        output::print_json(&serde_json::json!({
            "healthy": orphans.is_empty(),
            "orphanedReferenceCount": orphans.len(),
            "orphanedReferences": orphan_list,
        }));
    } else if orphans.is_empty() {
        println!("Project is healthy. No orphaned references found.");
    } else {
        println!("Found {} orphaned reference(s):", orphans.len());
        for o in &orphans {
            println!("  {} > {}.{} > {}", o.referrer_uuid, o.referrer_isa, o.property, o.orphan_uuid);
        }
    }

    Ok(())
}

fn dump(path: &str) -> Result<(), CliError> {
    let content = std::fs::read_to_string(path).map_err(|_| CliError::file_not_found(path))?;
    let plist = xcodekit::parser::parse(&content).map_err(|e| CliError::parse_error(&e))?;
    let json = serde_json::to_value(&plist).map_err(|e| CliError::new(ErrorCode::SerializeError, e.to_string()))?;
    output::print_json(&json);
    Ok(())
}
