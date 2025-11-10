use dirtidy::cli::{run_cli, OrganizeCommand};
use std::env;
use std::path::Path;

fn main() {
    println!("Welcome to dirtidy - directory organization made easy!");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: dirtidy <directory_path> [--undo]");
        return;
    }

    let dir_path = &args[1];
    let base_path = Path::new(dir_path);
    let undo_mode = args.len() > 2 && args[2] == "--undo";

    let command = if undo_mode {
        OrganizeCommand::Undo
    } else {
        OrganizeCommand::Organize
    };

    if let Err(e) = run_cli(command, base_path) {
        eprintln!("Error: {}", e);
    }
}
