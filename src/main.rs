use dirtidy::cli::{OrganizeCommand, run_cli};
use std::env;
use std::path::Path;

fn main() {
    println!("Welcome to dirtidy - directory organization made easy!");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: dirtidy <directory_path> [--undo | --dry-run]");
        return;
    }

    let dir_path = &args[1];
    let base_path = Path::new(dir_path);

    // Parse command-line flags
    let undo_mode = args.len() > 2 && args[2] == "--undo";
    let dry_run_mode = args.len() > 2 && args[2] == "--dry-run";

    let command = if undo_mode {
        OrganizeCommand::Undo
    } else {
        OrganizeCommand::Organize {
            dry_run: dry_run_mode,
        }
    };

    if let Err(e) = run_cli(command, base_path) {
        eprintln!("Error: {}", e);
    }
}
