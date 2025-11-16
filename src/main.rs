use clap::Parser;
use dirtidy::cli::{OrganizeCommand, run_cli_with_config};
use dirtidy::output::OutputFormatter;
use std::path::PathBuf;

/// A directory organization and cleanup utility.
///
/// Automatically organizes files in a directory into category-based subdirectories.
/// Supports dry-run mode for safe previewing and undo functionality to revert changes.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
struct Args {
    /// Path to the directory to organize
    #[arg(value_name = "DIRECTORY")]
    directory: PathBuf,

    /// Undo the previous organization
    #[arg(long, conflicts_with = "dry_run")]
    undo: bool,

    /// Simulate the organization without making changes
    #[arg(long, short = 'n')]
    dry_run: bool,

    /// Path to configuration file
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let command = if args.undo {
        OrganizeCommand::Undo
    } else {
        OrganizeCommand::Organize {
            dry_run: args.dry_run,
        }
    };

    let config_path_ref = args.config.as_deref();

    if let Err(e) = run_cli_with_config(command, &args.directory, config_path_ref) {
        OutputFormatter::error(&e);
        std::process::exit(1);
    }
}
