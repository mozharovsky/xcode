use clap::Subcommand;
use xcodekit::breakpoints::BreakpointBucket;

use crate::output::{self, CliError, ErrorCode};

#[derive(Subcommand)]
pub enum BreakpointAction {
    /// List breakpoints from a .xcodeproj or breakpoint file
    List {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Add a file breakpoint
    Add {
        path: String,
        #[arg(long)]
        file: String,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        json: bool,
    },
    /// Remove a breakpoint by UUID
    Remove {
        path: String,
        #[arg(long)]
        uuid: String,
        #[arg(long)]
        json: bool,
    },
}

fn load_bucket(path: &str) -> Result<(BreakpointBucket, String), CliError> {
    if path.ends_with(".xcbkptlist") {
        let bucket = BreakpointBucket::from_file(path).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
        Ok((bucket, path.to_string()))
    } else {
        let files = BreakpointBucket::find_breakpoint_files(path);
        if files.is_empty() {
            return Err(CliError::new(ErrorCode::FileNotFound, "No breakpoint files found"));
        }
        let file = &files[0];
        let bucket = BreakpointBucket::from_file(file).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
        Ok((bucket, file.clone()))
    }
}

pub fn run(action: BreakpointAction) -> Result<(), CliError> {
    match action {
        BreakpointAction::List { path, json } => {
            let (bucket, _) = load_bucket(&path)?;
            let bps = bucket.list_breakpoints();
            if json {
                let entries: Vec<_> = bps
                    .iter()
                    .map(|bp| {
                        serde_json::json!({
                            "uuid": bp.uuid,
                            "filePath": bp.file_path,
                            "line": bp.starting_line_number,
                            "enabled": bp.should_be_enabled,
                            "condition": bp.condition,
                            "symbolName": bp.symbol_name,
                        })
                    })
                    .collect();
                output::print_json(&serde_json::json!({ "breakpoints": entries }));
            } else {
                if bps.is_empty() {
                    println!("No breakpoints");
                }
                for bp in &bps {
                    let loc = match (&bp.file_path, &bp.starting_line_number) {
                        (Some(f), Some(l)) => format!("{}:{}", f, l),
                        (None, _) => bp.symbol_name.as_deref().unwrap_or("(unknown)").to_string(),
                        _ => "(unknown)".to_string(),
                    };
                    let enabled = bp.should_be_enabled.as_deref().unwrap_or("?");
                    println!("{} [{}] {}", bp.uuid.as_deref().unwrap_or("?"), enabled, loc);
                }
            }
            Ok(())
        }

        BreakpointAction::Add { path, file, line, json } => {
            let (mut bucket, file_path) = load_bucket(&path)?;
            let uuid_str = xcodekit::project::uuid::generate_dashed_uuid(&format!("{}:{}", file, line));
            bucket.add_file_breakpoint(&uuid_str, &file, line);
            bucket.save(&file_path).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "uuid": uuid_str, "changed": true }));
            } else {
                println!("Added breakpoint at {}:{} ({})", file, line, uuid_str);
            }
            Ok(())
        }

        BreakpointAction::Remove { path, uuid, json } => {
            let (mut bucket, file_path) = load_bucket(&path)?;
            let removed = bucket.remove_breakpoint(&uuid);
            if !removed {
                return Err(CliError::new(ErrorCode::ObjectNotFound, format!("Breakpoint '{}' not found", uuid)));
            }
            bucket.save(&file_path).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Removed breakpoint {}", uuid);
            }
            Ok(())
        }
    }
}
