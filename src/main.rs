use dirtidy::cli::{OrganizeCommand, run_cli_with_config};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("Welcome to dirtidy - directory organization made easy!");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: dirtidy <directory_path> [--undo | --dry-run] [--config <path>]");
        return;
    }

    let dir_path = &args[1];
    let base_path = Path::new(dir_path);

    // Parse command-line flags and options
    let mut undo_mode = false;
    let mut dry_run_mode = false;
    let mut config_path: Option<PathBuf> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--undo" => undo_mode = true,
            "--dry-run" => dry_run_mode = true,
            "--config" => {
                if i + 1 < args.len() {
                    config_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1; // Skip next arg since it's the config path
                } else {
                    eprintln!("Error: --config requires a path argument");
                    return;
                }
            }
            arg => {
                eprintln!("Unknown argument: {}", arg);
                return;
            }
        }
        i += 1;
    }

    let command = if undo_mode {
        OrganizeCommand::Undo
    } else {
        OrganizeCommand::Organize {
            dry_run: dry_run_mode,
        }
    };

    let config_path_ref = config_path.as_deref();

    if let Err(e) = run_cli_with_config(command, base_path, config_path_ref) {
        eprintln!("Error: {}", e);
    }
}
