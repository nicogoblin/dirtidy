//! Command-line interface module for dirtidy.
//!
//! This module handles all CLI-related functionality including:
//! - Command parsing and validation
//! - File type detection
//! - Organization orchestration
//! - Undo operation handling
//! - File filtering and exclusion

use crate::config::FilterConfig;
use crate::file_category::FileMapper;
use crate::file_organizer::{FileOrganizer, OperationLog};
use crate::undo::UndoManager;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

/// Represents a file with its type information.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The name of the file.
    pub name: String,
    /// The full path to the file.
    pub path: PathBuf,
    /// The detected file type/extension.
    pub file_type: Option<String>,
    /// The detected MIME type.
    pub mime_type: Option<String>,
    /// The categorized file category.
    pub category: crate::file_category::Category,
}

/// Represents a CLI command to execute.
#[derive(Debug, Clone, Copy)]
pub enum OrganizeCommand {
    /// Organize files in a directory.
    Organize {
        /// If true, simulate the operation without making changes.
        dry_run: bool,
    },
    /// Undo the previous organization.
    Undo,
}

/// Runs the CLI application with the given command and directory path.
///
/// This is the main entry point for CLI operations. It handles both
/// organization and undo operations based on the provided command.
///
/// # Arguments
///
/// * `command` - The command to execute (Organize or Undo)
/// * `dir_path` - The directory path to operate on
///
/// # Examples
///
/// ```no_run
/// use dirtidy::cli::{run_cli, OrganizeCommand};
/// use std::path::Path;
///
/// let result = run_cli(OrganizeCommand::Organize { dry_run: false }, Path::new("/path/to/directory"));
/// match result {
///     Ok(()) => println!("Operation completed successfully"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn run_cli(command: OrganizeCommand, dir_path: &Path) -> Result<(), String> {
    run_cli_with_config(command, dir_path, None)
}

/// Runs the CLI application with optional configuration file.
///
/// # Arguments
///
/// * `command` - The command to execute (Organize or Undo)
/// * `dir_path` - The directory path to operate on
/// * `config_path` - Optional path to configuration file
pub fn run_cli_with_config(
    command: OrganizeCommand,
    dir_path: &Path,
    config_path: Option<&Path>,
) -> Result<(), String> {
    match command {
        OrganizeCommand::Organize { dry_run } => {
            if dry_run {
                organize_directory_dry_run_with_config(dir_path, config_path)
            } else {
                organize_directory_with_config(dir_path, config_path)
            }
        }
        OrganizeCommand::Undo => undo_organization(dir_path),
    }
}

/// Organizes files in a directory into category subdirectories.
///
/// This function:
/// 1. Loads filter configuration (if available)
/// 2. Reads all files from the directory
/// 3. Applies filtering rules to exclude files
/// 4. Detects types using MIME type detection
/// 5. Categorizes them using the FileMapper
/// 6. Moves them to appropriate category directories
/// 7. Records the operations for potential undo
///
/// # Arguments
///
/// * `base_path` - The directory to organize
pub fn organize_directory_with_config(
    base_path: &Path,
    config_path: Option<&Path>,
) -> Result<(), String> {
    println!("Organizing contents of: {}", base_path.display());

    // Load and compile filter configuration
    let config = FilterConfig::load(config_path)
        .map_err(|e| format!("Error loading configuration: {}", e))?;
    let compiled_filters = config
        .compile()
        .map_err(|e| format!("Error compiling filters: {}", e))?;

    let entries = fs::read_dir(base_path)
        .map_err(|e| format!("Error reading directory {}: {}", base_path.display(), e))?;

    let mut file_infos: Vec<FileInfo> = Vec::new();
    let mapper = FileMapper::default();

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type()
            && file_type.is_file()
        {
            let file_path = entry.path();
            // Apply filter rules
            if compiled_filters.should_include(&file_path) {
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

        match FileOrganizer::move_to_category_with_record(base_path, &info.path, category_dir) {
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

    Ok(())
}

/// Simulates file organization without making any actual changes.
///
/// This function performs the same analysis as `organize_directory` but:
/// 1. Loads filter configuration (if available)
/// 2. Reads all files from the directory
/// 3. Applies filtering rules to exclude files
/// 4. Detects their types using MIME type detection
/// 5. Categorizes them using the FileMapper
/// 6. Displays what would be organized WITHOUT moving any files
/// 7. Shows a summary of files by category
///
/// # Arguments
///
/// * `base_path` - The directory to analyze
/// * `config_path` - Optional path to configuration file
pub fn organize_directory_dry_run_with_config(
    base_path: &Path,
    config_path: Option<&Path>,
) -> Result<(), String> {
    println!("DRY RUN: Analyzing contents of: {}", base_path.display());

    // Load and compile filter configuration
    let config = FilterConfig::load(config_path)
        .map_err(|e| format!("Error loading configuration: {}", e))?;
    let compiled_filters = config
        .compile()
        .map_err(|e| format!("Error compiling filters: {}", e))?;

    let entries = fs::read_dir(base_path)
        .map_err(|e| format!("Error reading directory {}: {}", base_path.display(), e))?;

    let mut file_infos: Vec<FileInfo> = Vec::new();
    let mapper = FileMapper::default();

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type()
            && file_type.is_file()
        {
            let file_path = entry.path();
            // Apply filter rules
            if compiled_filters.should_include(&file_path) {
                let file_info = detect_file_type(&entry, &mapper);
                file_infos.push(file_info);
            }
        }
    }

    if file_infos.is_empty() {
        println!("No files found to organize.");
        return Ok(());
    }

    println!("\nDRY RUN: Files would be organized as follows:");
    let mut category_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

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
        println!("   → Would move to {}/", category_dir);

        *category_counts.entry(category_dir.to_string()).or_insert(0) += 1;
    }

    println!("\nDRY RUN SUMMARY:");
    println!("Total files: {}", file_infos.len());

    // Sort category names for consistent output
    let mut categories: Vec<_> = category_counts.iter().collect();
    categories.sort_by_key(|&(name, _)| name);

    for (category, count) in categories {
        println!(
            "  {} {}: {}",
            category,
            if *count == 1 { "file" } else { "files" },
            count
        );
    }

    println!("\n✓ Dry run complete. No files were modified.");
    println!(
        "Run 'dirtidy {}' (without --dry-run) to execute the organization.",
        base_path.display()
    );

    Ok(())
}

/// Undoes the previous file organization operation.
///
/// This function:
/// 1. Loads the operation history from disk
/// 2. Reverses all recorded file movements
/// 3. Reports on any skipped or failed restorations
/// 4. Deletes the history file if undo was successful
///
/// # Arguments
///
/// * `base_path` - The directory where organization was performed
fn undo_organization(base_path: &Path) -> Result<(), String> {
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

            Ok(())
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

/// Detects the file type, MIME type, and category of a given directory entry.
///
/// Uses the `infer` crate to detect MIME type from file content,
/// then maps it to a category using the provided FileMapper.
///
/// # Arguments
///
/// * `entry` - The directory entry to analyze
/// * `mapper` - The FileMapper to use for categorization
///
/// # Returns
///
/// Returns a FileInfo struct with detected type information and category
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        use crate::file_category::Category;
        let file_info = FileInfo {
            name: "test.txt".to_string(),
            path: PathBuf::from("/path/to/test.txt"),
            file_type: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            category: Category::Document,
        };

        assert_eq!(file_info.name, "test.txt");
        assert_eq!(file_info.file_type, Some("txt".to_string()));
    }

    #[test]
    fn test_organize_command_enum() {
        let organize = OrganizeCommand::Organize { dry_run: false };
        let organize_dry_run = OrganizeCommand::Organize { dry_run: true };
        let undo = OrganizeCommand::Undo;

        // Just verify enum variants can be created
        matches!(organize, OrganizeCommand::Organize { dry_run: false });
        matches!(
            organize_dry_run,
            OrganizeCommand::Organize { dry_run: true }
        );
        matches!(undo, OrganizeCommand::Undo);
    }
}
