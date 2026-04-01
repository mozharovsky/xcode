use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError, ErrorCode};
use crate::resolve::resolve_target;

#[derive(Subcommand)]
pub enum TargetAction {
    /// List all native targets
    List {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Show details for a specific target
    Show {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        json: bool,
    },
    /// Rename a target (cascades to groups, product refs, proxies)
    Rename {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long, name = "new-name")]
        new_name: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Create a new native target with Debug/Release configurations
    #[command(name = "create-native")]
    CreateNative {
        path: String,
        #[arg(long)]
        name: String,
        #[arg(long, name = "product-type")]
        product_type: String,
        #[arg(long, name = "bundle-id")]
        bundle_id: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// Duplicate a target under a new name (deep-clones configs and phases)
    Duplicate {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long, name = "new-name")]
        new_name: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// List targets embedded in a host target
    #[command(name = "list-embedded")]
    ListEmbedded {
        path: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        json: bool,
    },
}

fn open(path: &str) -> Result<XcodeProject, CliError> {
    let resolved = crate::output::normalize_project_path(path);
    XcodeProject::open(&resolved).map_err(|e| {
        if e.contains("Failed to read") {
            CliError::file_not_found(path)
        } else {
            CliError::parse_error(&e)
        }
    })
}

fn save(project: &XcodeProject, path: &str) -> Result<(), CliError> {
    let output = project.to_pbxproj();
    std::fs::write(path, output).map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))
}

pub fn run(action: TargetAction) -> Result<(), CliError> {
    match action {
        TargetAction::List { path, json } => {
            let project = open(&path)?;
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

        TargetAction::Show { path, target, json } => {
            let project = open(&path)?;
            let uuid = resolve_target(&project, &target)?;
            let obj = project.get_object(&uuid).ok_or_else(|| CliError::target_not_found(&target))?;

            if json {
                let props: serde_json::Map<String, serde_json::Value> = obj
                    .props
                    .iter()
                    .map(|(k, v)| (k.to_string(), serde_json::to_value(v).unwrap_or_default()))
                    .collect();
                output::print_json(&serde_json::json!({
                    "uuid": uuid,
                    "isa": obj.isa,
                    "name": obj.get_str("name"),
                    "productType": obj.get_str("productType"),
                    "properties": props,
                }));
            } else {
                println!("{} ({})", obj.get_str("name").unwrap_or(""), obj.isa);
                println!("  UUID: {}", uuid);
                if let Some(pt) = obj.get_str("productType") {
                    println!("  productType: {}", pt);
                }
                if let Some(pn) = obj.get_str("productName") {
                    println!("  productName: {}", pn);
                }
            }
            Ok(())
        }

        TargetAction::Rename { path, target, new_name, write, json } => {
            let mut project = open(&path)?;
            let uuid = resolve_target(&project, &target)?;
            let old_name = project.get_target_name(&uuid).unwrap_or_default();

            project.rename_target(&uuid, &old_name, &new_name);

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "changed": write }));
            } else {
                println!(
                    "Renamed target '{}' to '{}'{}",
                    old_name,
                    new_name,
                    if write { "" } else { " (dry-run, use --write to save)" }
                );
            }
            Ok(())
        }

        TargetAction::CreateNative { path, name, product_type, bundle_id, write, json } => {
            let mut project = open(&path)?;
            let uuid = project
                .create_native_target(&name, &product_type, &bundle_id)
                .ok_or_else(|| CliError::new(ErrorCode::CreateFailed, "Failed to create target"))?;

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid, "changed": write }));
            } else {
                println!(
                    "Created target '{}' ({}){}",
                    name,
                    uuid,
                    if write { "" } else { " (dry-run, use --write to save)" }
                );
            }
            Ok(())
        }

        TargetAction::Duplicate { path, target, new_name, write, json } => {
            let mut project = open(&path)?;
            let uuid = resolve_target(&project, &target)?;
            let new_uuid = project
                .duplicate_target(&uuid, &new_name)
                .ok_or_else(|| CliError::new(ErrorCode::DuplicateFailed, "Failed to duplicate target"))?;

            if write {
                save(&project, &path)?;
            }

            if json {
                output::print_json(&serde_json::json!({ "uuid": new_uuid, "changed": write }));
            } else {
                println!(
                    "Duplicated target as '{}' ({}){}",
                    new_name,
                    new_uuid,
                    if write { "" } else { " (dry-run, use --write to save)" }
                );
            }
            Ok(())
        }

        TargetAction::ListEmbedded { path, target, json } => {
            let project = open(&path)?;
            let uuid = resolve_target(&project, &target)?;
            let embedded = project.get_embedded_targets(&uuid);

            let entries: Vec<_> = embedded
                .iter()
                .map(|u| {
                    let name = project.get_target_name(u).unwrap_or_default();
                    serde_json::json!({ "uuid": u, "name": name })
                })
                .collect();

            if json {
                output::print_json(&serde_json::json!({ "embedded": entries }));
            } else if entries.is_empty() {
                println!("No embedded targets");
            } else {
                for e in &entries {
                    println!("{}", e["name"].as_str().unwrap_or(""));
                }
            }
            Ok(())
        }
    }
}
