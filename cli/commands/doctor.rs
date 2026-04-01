use clap::Subcommand;
use xcodekit::project::XcodeProject;

use crate::output::{self, CliError};

#[derive(Subcommand)]
pub enum DoctorAction {
    /// Find orphaned references (UUIDs referenced but not present)
    Orphans {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Show a project health summary
    Summary {
        path: String,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: DoctorAction) -> Result<(), CliError> {
    match action {
        DoctorAction::Orphans { path, json } => {
            let project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let orphans = project.find_orphaned_references();

            if json {
                let list: Vec<_> = orphans
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
                    "orphanedReferenceCount": orphans.len(),
                    "orphanedReferences": list,
                }));
            } else if orphans.is_empty() {
                println!("No orphaned references found.");
            } else {
                println!("Found {} orphaned reference(s):", orphans.len());
                for o in &orphans {
                    println!(
                        "  {} > {}.{} > {}",
                        o.referrer_uuid, o.referrer_isa, o.property, o.orphan_uuid
                    );
                }
            }
            Ok(())
        }

        DoctorAction::Summary { path, json } => {
            let project = XcodeProject::open(&crate::output::normalize_project_path(&path)).map_err(|e| CliError::parse_error(&e))?;
            let orphans = project.find_orphaned_references();
            let target_count = project.native_targets().len();
            let object_count = project.objects().count();

            if json {
                output::print_json(&serde_json::json!({
                    "healthy": orphans.is_empty(),
                    "targetCount": target_count,
                    "objectCount": object_count,
                    "orphanedReferenceCount": orphans.len(),
                }));
            } else {
                println!("Targets:  {}", target_count);
                println!("Objects:  {}", object_count);
                println!("Orphans:  {}", orphans.len());
                println!("Health:   {}", if orphans.is_empty() { "OK" } else { "ISSUES FOUND" });
            }
            Ok(())
        }
    }
}
