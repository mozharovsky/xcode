use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError};

#[derive(Subcommand)]
pub enum ObjectAction {
    /// Get an object by UUID (shows all properties)
    Get {
        path: String,
        #[arg(long)]
        uuid: String,
        #[arg(long)]
        json: bool,
    },
    /// Get a single property from an object
    #[command(name = "get-property")]
    GetProperty {
        path: String,
        #[arg(long)]
        uuid: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        json: bool,
    },
    /// Set a string property on an object
    #[command(name = "set-property")]
    SetProperty {
        path: String,
        #[arg(long)]
        uuid: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        value: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        json: bool,
    },
    /// List all object UUIDs matching an ISA type
    #[command(name = "list-by-isa")]
    ListByIsa {
        path: String,
        #[arg(long)]
        isa: String,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: ObjectAction) -> Result<(), CliError> {
    match action {
        ObjectAction::Get { path, uuid, json } => {
            let project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let obj = project.get_object(&uuid).ok_or_else(|| CliError::object_not_found(&uuid))?;

            if json {
                let props: serde_json::Map<String, serde_json::Value> = obj.props.iter()
                    .map(|(k, v)| (k.to_string(), serde_json::to_value(v).unwrap_or_default()))
                    .collect();
                output::print_json(&serde_json::json!({
                    "uuid": uuid,
                    "isa": obj.isa,
                    "properties": props,
                }));
            } else {
                println!("{} ({})", uuid, obj.isa);
                for (k, v) in &obj.props {
                    println!("  {} = {:?}", k, v);
                }
            }
            Ok(())
        }

        ObjectAction::GetProperty { path, uuid, key, json } => {
            let project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let value = project.get_object_property(&uuid, &key);

            if json {
                output::print_json(&serde_json::json!({
                    "uuid": uuid,
                    "key": key,
                    "value": value,
                }));
            } else {
                match value {
                    Some(v) => println!("{}", v),
                    None => println!("(not set)"),
                }
            }
            Ok(())
        }

        ObjectAction::SetProperty { path, uuid, key, value, write, json } => {
            let mut project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let ok = project.set_object_property(&uuid, &key, &value);

            if !ok {
                return Err(CliError::object_not_found(&uuid));
            }

            if write {
                std::fs::write(&path, project.to_pbxproj())
                    .map_err(|e| CliError::new("WRITE_FAILED", e.to_string()))?;
            }

            if json {
                output::print_json(&serde_json::json!({ "changed": write }));
            } else {
                println!("Set {}.{} = {}{}", uuid, key, value,
                    if write { "" } else { " (dry-run)" });
            }
            Ok(())
        }

        ObjectAction::ListByIsa { path, isa, json } => {
            let project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let uuids = project.find_objects_by_isa(&isa);

            if json {
                let entries: Vec<_> = uuids.iter()
                    .map(|u| {
                        let name = project.get_object(u)
                            .and_then(|o| o.get_str("name").or(o.get_str("path")))
                            .unwrap_or("")
                            .to_string();
                        serde_json::json!({ "uuid": u, "name": name })
                    })
                    .collect();
                output::print_json(&serde_json::json!({ "objects": entries }));
            } else {
                for u in &uuids {
                    let name = project.get_object(u)
                        .and_then(|o| o.get_str("name").or(o.get_str("path")))
                        .unwrap_or("");
                    println!("{} {}", u, name);
                }
            }
            Ok(())
        }
    }
}
