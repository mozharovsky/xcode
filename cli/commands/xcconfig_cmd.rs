use clap::Subcommand;
use xcodekit::xcconfig::XCConfig;

use crate::output::{self, CliError, ErrorCode};

#[derive(Subcommand)]
pub enum XCConfigAction {
    /// Parse an xcconfig file and dump as JSON
    Parse {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Flatten xcconfig to resolved key-value pairs
    Flatten {
        path: String,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: XCConfigAction) -> Result<(), CliError> {
    match action {
        XCConfigAction::Parse { path, json } => {
            let config = XCConfig::from_file(&path).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            if json {
                let val = serde_json::to_value(&config)
                    .map_err(|e| CliError::new(ErrorCode::SerializeError, e.to_string()))?;
                output::print_json(&val);
            } else {
                for entry in &config.entries {
                    match entry {
                        xcodekit::xcconfig::XCConfigEntry::Setting { key, value, conditions } => {
                            if conditions.is_empty() {
                                println!("{} = {}", key, value);
                            } else {
                                let conds: Vec<String> =
                                    conditions.iter().map(|c| format!("[{}={}]", c.key, c.value)).collect();
                                println!("{}{} = {}", key, conds.join(""), value);
                            }
                        }
                        xcodekit::xcconfig::XCConfigEntry::Include { path, optional } => {
                            if *optional {
                                println!("#include? \"{}\"", path);
                            } else {
                                println!("#include \"{}\"", path);
                            }
                        }
                        xcodekit::xcconfig::XCConfigEntry::Comment { text } => {
                            println!("// {}", text);
                        }
                    }
                }
            }
            Ok(())
        }

        XCConfigAction::Flatten { path, json } => {
            let config = XCConfig::from_file(&path).map_err(|e| CliError::new(ErrorCode::ParseError, e))?;
            let flat = config.flatten();
            if json {
                output::print_json(&flat);
            } else {
                for (k, v) in &flat {
                    println!("{} = {}", k, v);
                }
            }
            Ok(())
        }
    }
}
