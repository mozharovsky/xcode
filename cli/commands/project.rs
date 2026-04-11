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
    /// List all file references in the project
    #[command(name = "list-files")]
    ListFiles {
        /// Path to .pbxproj file
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Show groups only (no files)
    #[command(name = "list-groups")]
    ListGroups {
        /// Path to .pbxproj file
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Show the project group tree
    #[command(name = "list-tree")]
    ListTree {
        /// Path to .pbxproj file
        path: String,
        #[arg(long)]
        json: bool,
    },
}

fn open_project(path: &str) -> Result<XcodeProject, CliError> {
    let _timer = output::verbose_timer("open project");
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
        ProjectAction::ListFiles { path, json } => list_files(&path, json),
        ProjectAction::ListGroups { path, json } => list_groups(&path, json),
        ProjectAction::ListTree { path, json } => list_tree(&path, json),
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

fn list_files(path: &str, json: bool) -> Result<(), CliError> {
    let project = open_project(path)?;
    let files = project.list_all_files();

    if json {
        let entries: Vec<_> = files
            .iter()
            .map(|(uuid, fpath, ftype)| serde_json::json!({ "uuid": uuid, "path": fpath, "fileType": ftype }))
            .collect();
        output::print_json(&serde_json::json!({ "files": entries }));
    } else if files.is_empty() {
        println!("No file references");
    } else {
        for (_, fpath, ftype) in &files {
            println!("{} ({})", fpath, ftype);
        }
    }
    Ok(())
}

fn list_groups(path: &str, json: bool) -> Result<(), CliError> {
    let project = open_project(path)?;
    let groups: Vec<_> = project
        .objects_by_isa("PBXGroup")
        .iter()
        .map(|g| {
            let name = g.get_str("name").or_else(|| g.get_str("path")).unwrap_or("").to_string();
            let child_count = g.get_array("children").map(|a| a.len()).unwrap_or(0);
            serde_json::json!({ "uuid": g.uuid, "name": name, "childCount": child_count })
        })
        .collect();

    if json {
        output::print_json(&serde_json::json!({ "groups": groups }));
    } else if groups.is_empty() {
        println!("No groups");
    } else {
        for g in &groups {
            println!("{} ({} children)", g["name"].as_str().unwrap_or(""), g["childCount"]);
        }
    }
    Ok(())
}

fn list_tree(path: &str, json: bool) -> Result<(), CliError> {
    let project = open_project(path)?;
    let main_group = project
        .main_group_uuid()
        .ok_or_else(|| CliError::new(ErrorCode::ObjectNotFound, "No main group found".to_string()))?;
    let tree = project.build_group_tree(&main_group);

    if json {
        output::print_json(&tree);
    } else {
        println!("{}", serde_json::to_string_pretty(&tree).unwrap_or_else(|_| "{}".to_string()));
    }
    Ok(())
}
