use clap::Subcommand;
use xcodekit::scheme::Scheme;

use crate::output::{self, CliError, ErrorCode};

#[derive(Subcommand)]
pub enum SchemeAction {
    /// List schemes in a .xcodeproj
    List {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Show scheme details as JSON
    Show {
        path: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        json: bool,
    },
    /// Create a new scheme for a target
    Create {
        path: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        json: bool,
    },
    /// Set an environment variable on the launch action
    #[command(name = "set-env")]
    SetEnv {
        path: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        value: String,
        #[arg(long)]
        json: bool,
    },
    /// Add a launch argument
    #[command(name = "add-arg")]
    AddArg {
        path: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        arg: String,
        #[arg(long)]
        json: bool,
    },
    /// Add a build target to the scheme
    #[command(name = "add-build-target")]
    AddBuildTarget {
        path: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        target: String,
        #[arg(long, name = "blueprint-id")]
        blueprint_id: String,
        #[arg(long, name = "buildable-name")]
        buildable_name: String,
        #[arg(long)]
        container: String,
        #[arg(long)]
        json: bool,
    },
    /// Duplicate a scheme under a new name
    Duplicate {
        path: String,
        #[arg(long)]
        scheme: String,
        #[arg(long, name = "new-name")]
        new_name: String,
        #[arg(long)]
        json: bool,
    },
    /// Remove a scheme file
    Remove {
        path: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        json: bool,
    },
    /// Set build configuration for run/archive actions
    #[command(name = "set-config")]
    SetConfig {
        path: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        archive: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Enable code coverage on the test action
    #[command(name = "enable-coverage")]
    EnableCoverage {
        path: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        json: bool,
    },
}

fn scheme_path(xcodeproj: &str, name: &str) -> String {
    format!("{}/xcshareddata/xcschemes/{}.xcscheme", xcodeproj.trim_end_matches('/'), name)
}

pub fn run(action: SchemeAction) -> Result<(), CliError> {
    match action {
        SchemeAction::List { path, json } => {
            let schemes = Scheme::list_schemes(&path);
            if json {
                let entries: Vec<_> =
                    schemes.iter().map(|(name, path)| serde_json::json!({ "name": name, "path": path })).collect();
                output::print_json(&serde_json::json!({ "schemes": entries }));
            } else if schemes.is_empty() {
                println!("No schemes found");
            } else {
                for (name, _) in &schemes {
                    println!("{}", name);
                }
            }
            Ok(())
        }

        SchemeAction::Show { path, scheme, json } => {
            let scheme_file = scheme_path(&path, &scheme);
            let s = Scheme::from_file(&scheme_file).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            if json {
                let val =
                    serde_json::to_value(&s).map_err(|e| CliError::new(ErrorCode::SerializeError, e.to_string()))?;
                output::print_json(&val);
            } else {
                let targets = s.get_build_targets();
                println!("Scheme: {}", scheme);
                if let Some(ref la) = s.launch_action {
                    if let Some(ref cfg) = la.build_configuration {
                        println!("  Launch config: {}", cfg);
                    }
                }
                println!("  Build targets: {}", targets.len());
                for t in &targets {
                    println!(
                        "    - {} ({})",
                        t.blueprint_name.as_deref().unwrap_or("?"),
                        t.buildable_name.as_deref().unwrap_or("?")
                    );
                }
            }
            Ok(())
        }

        SchemeAction::Create { path, name, target, json } => {
            let container = format!(
                "container:{}",
                std::path::Path::new(&path).file_name().and_then(|f| f.to_str()).unwrap_or(&path)
            );
            let s = Scheme::create_for_target(&target, "", &format!("{}.app", target), &container);
            let scheme_file = scheme_path(&path, &name);
            let dir = std::path::Path::new(&scheme_file).parent().unwrap();
            std::fs::create_dir_all(dir).map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))?;
            s.save(&scheme_file).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "created": name, "path": scheme_file }));
            } else {
                println!("Created scheme '{}' at {}", name, scheme_file);
            }
            Ok(())
        }

        SchemeAction::SetEnv { path, scheme, key, value, json } => {
            let scheme_file = scheme_path(&path, &scheme);
            let mut s = Scheme::from_file(&scheme_file).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            s.set_env_var(&key, &value, true);
            s.save(&scheme_file).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Set {}={} on scheme '{}'", key, value, scheme);
            }
            Ok(())
        }

        SchemeAction::AddArg { path, scheme, arg, json } => {
            let scheme_file = scheme_path(&path, &scheme);
            let mut s = Scheme::from_file(&scheme_file).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            s.add_launch_arg(&arg);
            s.save(&scheme_file).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Added launch argument '{}' to scheme '{}'", arg, scheme);
            }
            Ok(())
        }

        SchemeAction::AddBuildTarget { path, scheme, target, blueprint_id, buildable_name, container, json } => {
            let scheme_file = scheme_path(&path, &scheme);
            let mut s = Scheme::from_file(&scheme_file).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            s.add_build_target(&blueprint_id, &buildable_name, &target, &container);
            s.save(&scheme_file).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Added build target '{}' to scheme '{}'", target, scheme);
            }
            Ok(())
        }

        SchemeAction::Duplicate { path, scheme, new_name, json } => {
            let scheme_file = scheme_path(&path, &scheme);
            let s = Scheme::from_file(&scheme_file).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            let dup = s.duplicate();
            let new_file = scheme_path(&path, &new_name);
            let dir = std::path::Path::new(&new_file).parent().unwrap();
            std::fs::create_dir_all(dir).map_err(|e| CliError::new(ErrorCode::WriteFailed, e.to_string()))?;
            dup.save(&new_file).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "created": new_name, "path": new_file }));
            } else {
                println!("Duplicated scheme '{}' as '{}' at {}", scheme, new_name, new_file);
            }
            Ok(())
        }

        SchemeAction::Remove { path, scheme, json } => {
            let scheme_file = scheme_path(&path, &scheme);
            std::fs::remove_file(&scheme_file).map_err(|e| CliError::new(ErrorCode::RemoveFailed, e.to_string()))?;
            if json {
                output::print_json(&serde_json::json!({ "removed": scheme }));
            } else {
                println!("Removed scheme '{}'", scheme);
            }
            Ok(())
        }

        SchemeAction::SetConfig { path, scheme, run, archive, json } => {
            let scheme_file = scheme_path(&path, &scheme);
            let mut s = Scheme::from_file(&scheme_file).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            s.set_build_configuration(run.as_deref(), archive.as_deref());
            s.save(&scheme_file).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Updated build configuration for scheme '{}'", scheme);
            }
            Ok(())
        }

        SchemeAction::EnableCoverage { path, scheme, json } => {
            let scheme_file = scheme_path(&path, &scheme);
            let mut s = Scheme::from_file(&scheme_file).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            s.set_test_coverage(true);
            s.save(&scheme_file).map_err(|e| CliError::new(ErrorCode::WriteFailed, e))?;
            if json {
                output::print_json(&serde_json::json!({ "changed": true }));
            } else {
                println!("Enabled code coverage on scheme '{}'", scheme);
            }
            Ok(())
        }
    }
}
