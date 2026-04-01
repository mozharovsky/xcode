use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError};
use crate::resolve::resolve_target;

#[derive(Subcommand)]
pub enum SpmAction {
    /// Add a remote Swift package
    #[command(name = "add-remote")]
    AddRemote {
        path: String,
        #[arg(long)]
        url: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Add a local Swift package
    #[command(name = "add-local")]
    AddLocal {
        path: String,
        #[arg(long, name = "package-path")]
        package_path: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Add a package product to a target
    #[command(name = "add-product")]
    AddProduct {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        product: String,
        #[arg(long)]
        package: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Remove a package product from a target
    #[command(name = "remove-product")]
    RemoveProduct {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        product: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// List all Swift packages in the project
    List {
        path: String,
        #[arg(long)]
        json: bool,
    },
}

fn open(path: &str) -> Result<XcodeProject, CliError> {
    let resolved = crate::output::normalize_project_path(path);
    XcodeProject::open(&resolved).map_err(|e| CliError::parse_error(&e))
}

fn save(project: &XcodeProject, path: &str) -> Result<(), CliError> {
    let resolved = crate::output::normalize_project_path(path);
    std::fs::write(&resolved, project.to_pbxproj())
        .map_err(|e| CliError::new("WRITE_FAILED", e.to_string()))
}

fn resolve_package(project: &XcodeProject, query: &str) -> Result<String, CliError> {
    if query.len() == 24 && query.chars().all(|c| c.is_ascii_hexdigit()) && project.get_object(query).is_some() {
        return Ok(query.to_string());
    }

    let packages = project.list_swift_packages();
    let matches: Vec<_> = packages.iter()
        .filter(|(_, _, loc)| loc.contains(query) || loc.ends_with(query))
        .collect();

    match matches.len() {
        0 => Err(CliError::new("PACKAGE_NOT_FOUND", format!("No package matching '{}'", query))),
        1 => Ok(matches[0].0.clone()),
        _ => Err(CliError::new("AMBIGUOUS_REFERENCE", format!("Multiple packages matched '{}'", query))),
    }
}

pub fn run(action: SpmAction) -> Result<(), CliError> {
    match action {
        SpmAction::AddRemote { path, url, version, write, json } => {
            let mut project = open(&path)?;
            let uuid = project.add_remote_swift_package(&url, &version)
                .ok_or_else(|| CliError::new("ADD_FAILED", "Failed to add package"))?;

            if write { save(&project, &path)?; }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": true }));
            } else {
                println!("Added remote package '{}' @ {} ({}){}", url, version, uuid,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        SpmAction::AddLocal { path, package_path, write, json } => {
            let mut project = open(&path)?;
            let uuid = project.add_local_swift_package(&package_path)
                .ok_or_else(|| CliError::new("ADD_FAILED", "Failed to add package"))?;

            if write { save(&project, &path)?; }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": true }));
            } else {
                println!("Added local package '{}' ({}){}", package_path, uuid,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        SpmAction::AddProduct { path, target, product, package, write, json } => {
            let mut project = open(&path)?;
            let target_uuid = resolve_target(&project, &target)?;
            let package_uuid = resolve_package(&project, &package)?;
            let uuid = project.add_swift_package_product(&target_uuid, &product, &package_uuid)
                .ok_or_else(|| CliError::new("ADD_FAILED", "Failed to add product"))?;

            if write { save(&project, &path)?; }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": true }));
            } else {
                println!("Added product '{}' to target '{}' ({}){}", product, target, uuid,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        SpmAction::RemoveProduct { path, target, product, write, json } => {
            let mut project = open(&path)?;
            let target_uuid = resolve_target(&project, &target)?;
            let changed = project.remove_swift_package_product(&target_uuid, &product);

            if !changed {
                return Err(CliError::new("REMOVE_FAILED", format!("Product '{}' not found on target '{}'", product, target)));
            }

            if write { save(&project, &path)?; }

            if json {
                output::print_json(&serde_json::json!({ "changed": changed }));
            } else {
                println!("Removed product '{}' from target '{}'{}", product, target,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        SpmAction::List { path, json } => {
            let project = open(&path)?;
            let packages = project.list_swift_packages();

            if json {
                let entries: Vec<_> = packages.iter()
                    .map(|(uuid, isa, location)| serde_json::json!({
                        "uuid": uuid,
                        "type": isa,
                        "location": location,
                    }))
                    .collect();
                output::print_json(&serde_json::json!({ "packages": entries }));
            } else if packages.is_empty() {
                println!("No Swift packages");
            } else {
                for (_, isa, location) in &packages {
                    let kind = if isa.contains("Remote") { "remote" } else { "local" };
                    println!("{} ({})", location, kind);
                }
            }
            Ok(())
        }
    }
}
