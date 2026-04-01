use clap::Subcommand;

use crate::output::{self, CliError};

#[derive(Subcommand)]
pub enum PlistAction {
    /// Parse a plist file (XML or binary) and output as JSON
    Parse {
        /// Path to .plist, .entitlements, or Info.plist file
        path: String,
    },
    /// Build a plist XML file from JSON input
    Build {
        /// Path to JSON input file
        #[arg(long)]
        input: String,
        /// Path to output plist file
        #[arg(long)]
        output: String,
    },
}

pub fn run(action: PlistAction) -> Result<(), CliError> {
    match action {
        PlistAction::Parse { path } => {
            let content =
                std::fs::read_to_string(&path).map_err(|_| CliError::file_not_found(&path))?;
            let value = xcodekit::plist_xml::parse_plist(&content)
                .map_err(|e| CliError::new("PARSE_ERROR", e))?;
            output::print_json(&value);
            Ok(())
        }
        PlistAction::Build { input, output: out } => {
            let json_str =
                std::fs::read_to_string(&input).map_err(|_| CliError::file_not_found(&input))?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| CliError::new("PARSE_ERROR", format!("Invalid JSON: {}", e)))?;
            let plist_str = xcodekit::plist_xml::build_plist(&value)
                .map_err(|e| CliError::new("BUILD_ERROR", e))?;
            std::fs::write(&out, plist_str)
                .map_err(|e| CliError::new("WRITE_FAILED", e.to_string()))?;
            println!("Written to {}", out);
            Ok(())
        }
    }
}
