use std::process;

use clap::{Parser, Subcommand};

mod commands;
mod output;
mod resolve;

#[derive(Parser)]
#[command(name = "xcodekit", about = "Native Xcode project automation for AI agents, CI, and developer tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect and query Xcode projects
    Project {
        #[command(subcommand)]
        action: commands::project::ProjectAction,
    },
    /// Execute multiple operations in a single parse/save cycle
    Batch(commands::batch::BatchArgs),
    /// List, show, rename, and create targets
    Target {
        #[command(subcommand)]
        action: commands::target::TargetAction,
    },
    /// Get, set, and remove build settings
    Build {
        #[command(subcommand)]
        action: commands::build_setting::BuildAction,
    },
    /// Add files to groups
    File {
        #[command(subcommand)]
        action: commands::file::FileAction,
    },
    /// Manage groups
    Group {
        #[command(subcommand)]
        action: commands::group::GroupAction,
    },
    /// Add frameworks to targets
    Framework {
        #[command(subcommand)]
        action: commands::framework::FrameworkAction,
    },
    /// Manage target dependencies
    Dependency {
        #[command(subcommand)]
        action: commands::dependency::DependencyAction,
    },
    /// Embed extensions into host targets
    Extension {
        #[command(subcommand)]
        action: commands::extension::ExtensionAction,
    },
    /// Validate and diagnose project health
    Doctor {
        #[command(subcommand)]
        action: commands::doctor::DoctorAction,
    },
    /// Manage Swift Package Manager dependencies
    Spm {
        #[command(subcommand)]
        action: commands::spm::SpmAction,
    },
    /// Parse and build plist files (entitlements, Info.plist)
    Plist {
        #[command(subcommand)]
        action: commands::plist::PlistAction,
    },
    /// Manage Xcode 16+ file system sync groups
    Sync {
        #[command(subcommand)]
        action: commands::sync_group::SyncAction,
    },
    /// Low-level object access (advanced)
    Object {
        #[command(subcommand)]
        action: commands::object::ObjectAction,
    },
    /// Print version information
    Version {
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Batch(args) => commands::batch::run(args),
        Command::Project { action } => commands::project::run(action),
        Command::Target { action } => commands::target::run(action),
        Command::Build { action } => commands::build_setting::run(action),
        Command::File { action } => commands::file::run(action),
        Command::Group { action } => commands::group::run(action),
        Command::Framework { action } => commands::framework::run(action),
        Command::Dependency { action } => commands::dependency::run(action),
        Command::Extension { action } => commands::extension::run(action),
        Command::Doctor { action } => commands::doctor::run(action),
        Command::Spm { action } => commands::spm::run(action),
        Command::Plist { action } => commands::plist::run(action),
        Command::Sync { action } => commands::sync_group::run(action),
        Command::Object { action } => commands::object::run(action),
        Command::Version { json } => {
            if json {
                output::print_json(&serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                }));
            } else {
                println!("xcodekit {}", env!("CARGO_PKG_VERSION"));
            }
            Ok(())
        }
    };

    if let Err(e) = result {
        output::print_error(&e);
        process::exit(1);
    }
}
