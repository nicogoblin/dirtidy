mod file_category;
mod file_organizer;
mod undo;

use file_category::{Category, FileMapper};
use file_organizer::{FileOrganizer, OperationLog};
use std::env;
use std::fs::{self, DirEntry};
use std::path::Path;
use undo::UndoManager;

/// Represents a file with its type information
#[derive(Debug, Clone)]
struct FileInfo {
    name: String,
    path: std::path::PathBuf,
    file_type: Option<String>,
    mime_type: Option<String>,
    category: Category,
}

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

    if undo_mode {
        handle_undo(base_path);
    } else {
        handle_organize(base_path);
    }
}

/// Handles the undo operation.
fn handle_undo(base_path: &Path) {
    println!("Undoing previous organization...");

    match UndoManager::undo(base_path) {
        Ok(report) => {
            println!("Undo complete!");
            println!("  Restored: {}", report.restored_files);

            if !report.skipped_files.is_empty() {
                println!("  Skipped: {}", report.skipped_files.len());
                for (path, reason) in &report.skipped_files {
                    println!("    - {}: {}", path.display(), reason);
                }
            }

            if !report.failed_restores.is_empty() {
                println!("  Failed: {}", report.failed_restores.len());
                for (path, reason) in &report.failed_restores {
                    eprintln!("    - {}: {}", path.display(), reason);
                }
                eprintln!("\nWarning: History file was NOT deleted due to failures.");
                eprintln!("Please fix the issues and try again.");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

/// Handles the organize operation.
fn handle_organize(base_path: &Path) {
    println!("Organizing contents of: {}", base_path.display());

    match fs::read_dir(base_path) {
        Ok(entries) => {
            let mut file_infos: Vec<FileInfo> = Vec::new();
            let mapper = FileMapper::default();
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let file_info = detect_file_type(&entry, &mapper);
                        file_infos.push(file_info);
                    }
                }
            }

            println!("Files found and organizing:");
            let mut operation_log = OperationLog::new(base_path.to_path_buf());
            let mut organize_failed = false;

            for info in &file_infos {
                let type_info = if let Some(ref ftype) = info.file_type {
                    format!(" [{}]", ftype)
                } else {
                    String::new()
                };
                let mime_info = if let Some(ref mime) = info.mime_type {
                    format!(" ({})", mime)
                } else {
                    String::new()
                };
                let category_dir = info.category.dir_name();
                println!(" - {}{}{}", info.name, type_info, mime_info);

                // Organize the file into its category directory and record the operation
                match FileOrganizer::move_to_category_with_record(
                    base_path,
                    &info.path,
                    category_dir,
                ) {
                    Ok(operation) => {
                        println!("   ✓ Moved to {}/", category_dir);
                        operation_log.add_operation(operation);
                    }
                    Err(e) => {
                        eprintln!("   ✗ Error: {}", e);
                        organize_failed = true;
                    }
                }
            }

            // Save the operation log
            match operation_log.save(base_path) {
                Ok(()) => {
                    println!("\nOrganization complete!");
                    println!(
                        "History saved. Use 'dirtidy {} --undo' to revert changes.",
                        base_path.display()
                    );
                }
                Err(e) => {
                    eprintln!("Warning: Could not save history: {}", e);
                    if organize_failed {
                        eprintln!(
                            "Undo may not be available. Please verify files were organized correctly."
                        );
                    }
                }
            }

            if organize_failed {
                eprintln!("\nSome files could not be organized. Please review errors above.");
            }
        }
        Err(e) => {
            eprintln!("Error reading directory {}: {}", base_path.display(), e);
        }
    }
}

/// Detects the file type, MIME type, and category of a given directory entry.
fn detect_file_type(entry: &DirEntry, mapper: &FileMapper) -> FileInfo {
    let name = entry.file_name().to_string_lossy().to_string();
    let path = entry.path();

    let (file_type, mime_type) = if let Ok(data) = std::fs::read(&path) {
        if let Some(kind) = infer::get(&data) {
            let mime = kind.mime_type().to_string();
            let extension = kind.extension().to_string();
            (Some(extension), Some(mime))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Determine the category using both MIME type and extension
    let category = mapper.categorize(mime_type.as_deref(), file_type.as_deref());

    FileInfo {
        name,
        path,
        file_type,
        mime_type,
        category,
    }
}
