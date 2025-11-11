/// Undo functionality for reverting file organization operations.
///
/// This module provides the ability to undo file organization by moving files
/// back to their original locations based on a recorded operation history.
use crate::file_organizer::{Operation, OperationLog, OrganizeError, OrganizeResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents the result of an undo operation.
#[derive(Debug)]
pub struct UndoReport {
    /// Number of files successfully restored.
    pub restored_files: usize,
    /// Number of files that failed to restore.
    pub failed_restores: Vec<(PathBuf, String)>,
    /// Number of files that were skipped (e.g., file not found).
    pub skipped_files: Vec<(PathBuf, String)>,
}

impl UndoReport {
    /// Creates a new empty undo report.
    fn new() -> Self {
        Self {
            restored_files: 0,
            failed_restores: Vec::new(),
            skipped_files: Vec::new(),
        }
    }

    /// Returns the total number of operations processed.
    #[allow(dead_code)]
    pub fn total_processed(&self) -> usize {
        self.restored_files + self.failed_restores.len() + self.skipped_files.len()
    }

    /// Returns true if the undo was completely successful.
    pub fn is_complete_success(&self) -> bool {
        self.failed_restores.is_empty() && self.skipped_files.is_empty()
    }
}

/// Manages undo operations for file organization.
pub struct UndoManager;

impl UndoManager {
    /// Undoes the most recent file organization operation.
    ///
    /// This function loads the operation history from the specified base path,
    /// validates it, and then reverses all recorded file movements.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The directory where the organization was performed
    ///
    /// # Returns
    ///
    /// Returns an `UndoReport` describing what was restored, what failed,
    /// and what was skipped. Returns an error if the history file is missing,
    /// corrupted, or if the base path doesn't exist.
    ///
    /// # Edge Cases Handled
    ///
    /// * **File not found**: Skipped with a note that the file couldn't be found
    /// * **File name conflict**: The conflicting file is backed up with a timestamp suffix
    /// * **Permission denied**: Recorded as a failure with the error reason
    /// * **Missing history**: Returns an error indicating no undo is available
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dirtidy::undo::UndoManager;
    /// use std::path::Path;
    ///
    /// let result = UndoManager::undo(Path::new("/path/to/directory"));
    /// match result {
    ///     Ok(report) => println!("Restored {} files", report.restored_files),
    ///     Err(e) => eprintln!("Undo failed: {}", e),
    /// }
    /// ```
    pub fn undo(base_path: &Path) -> OrganizeResult<UndoReport> {
        // Validate that the base path exists
        if !base_path.exists() {
            return Err(OrganizeError::InvalidBasePath {
                path: base_path.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "base path does not exist",
                ),
            });
        }

        // Load the operation log
        let log = OperationLog::load(base_path)?;
        let log = log.ok_or_else(|| OrganizeError::InvalidHistoryFormat {
            reason: "No previous organization found to undo".to_string(),
        })?;

        // Process operations in reverse order (undo is LIFO)
        let mut report = UndoReport::new();
        for operation in log.operations.iter().rev() {
            match Self::restore_file(operation) {
                Ok(()) => {
                    report.restored_files += 1;
                }
                Err((path, reason)) => {
                    if reason.contains("not found") {
                        report.skipped_files.push((path, reason));
                    } else {
                        report.failed_restores.push((path, reason));
                    }
                }
            }
        }

        // Only delete history if undo was successful
        if report.is_complete_success()
            && let Err(e) = OperationLog::delete(base_path)
        {
            eprintln!("Warning: Could not delete history file: {}", e);
        }

        Ok(report)
    }

    /// Restores a single file to its original location.
    ///
    /// Handles file name conflicts by backing up the existing file with a timestamp.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `Err((path, reason))` on failure.
    fn restore_file(operation: &Operation) -> Result<(), (PathBuf, String)> {
        // Check if the current location exists
        if !operation.new_path.exists() {
            return Err((
                operation.new_path.clone(),
                "File not found at expected location".to_string(),
            ));
        }

        // Check if a file already exists at the original location
        if operation.original_path.exists() {
            // Try to back up the conflicting file
            let backup_path = Self::generate_backup_path(&operation.original_path);
            fs::rename(&operation.original_path, &backup_path).map_err(|e| {
                (
                    operation.original_path.clone(),
                    format!("Could not backup conflicting file: {}", e),
                )
            })?;
        }

        // Move the file back to its original location
        fs::rename(&operation.new_path, &operation.original_path).map_err(|e| {
            (
                operation.new_path.clone(),
                format!("Failed to restore file: {}", e),
            )
        })?;

        Ok(())
    }

    /// Generates a backup path for a file by appending a timestamp.
    ///
    /// Example: `file.txt` becomes `file.txt.bak.20251109-143052`
    fn generate_backup_path(original_path: &Path) -> PathBuf {
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let filename = original_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        let backup_name = format!("{}.bak.{}", filename, timestamp);

        if let Some(parent) = original_path.parent() {
            parent.join(backup_name)
        } else {
            PathBuf::from(backup_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_organizer::FileOrganizer;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_undo_no_history() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        let result = UndoManager::undo(base_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_undo_single_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create and move a file
        let file_path = base_path.join("test.txt");
        fs::write(&file_path, "test content").expect("Failed to write test file");

        let operation =
            FileOrganizer::move_to_category_with_record(base_path, &file_path, "documents")
                .expect("Failed to move file");

        // Record the operation
        let mut log = OperationLog::new(base_path.to_path_buf());
        log.add_operation(operation);
        log.save(base_path).expect("Failed to save history");

        // Verify file was moved
        assert!(!file_path.exists());
        let moved_file = base_path.join("documents").join("test.txt");
        assert!(moved_file.exists());

        // Undo the operation
        let report = UndoManager::undo(base_path).expect("Undo failed");

        // Verify the file was restored
        assert_eq!(report.restored_files, 1);
        assert!(report.is_complete_success());
        assert!(file_path.exists());
        assert!(!moved_file.exists());
    }

    #[test]
    fn test_undo_multiple_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create and move multiple files
        let file1 = base_path.join("image.png");
        let file2 = base_path.join("document.pdf");

        fs::write(&file1, "image data").expect("Failed to write file1");
        fs::write(&file2, "pdf data").expect("Failed to write file2");

        let op1 = FileOrganizer::move_to_category_with_record(base_path, &file1, "images")
            .expect("Failed to move file1");
        let op2 = FileOrganizer::move_to_category_with_record(base_path, &file2, "documents")
            .expect("Failed to move file2");

        // Record operations
        let mut log = OperationLog::new(base_path.to_path_buf());
        log.add_operation(op1);
        log.add_operation(op2);
        log.save(base_path).expect("Failed to save history");

        // Undo
        let report = UndoManager::undo(base_path).expect("Undo failed");

        // Verify both files were restored
        assert_eq!(report.restored_files, 2);
        assert!(report.is_complete_success());
        assert!(file1.exists());
        assert!(file2.exists());
    }

    #[test]
    fn test_undo_with_file_name_conflict() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create initial file and move it
        let file_path = base_path.join("test.txt");
        fs::write(&file_path, "original content").expect("Failed to write file");

        let operation =
            FileOrganizer::move_to_category_with_record(base_path, &file_path, "documents")
                .expect("Failed to move file");

        // Record and save
        let mut log = OperationLog::new(base_path.to_path_buf());
        log.add_operation(operation);
        log.save(base_path).expect("Failed to save history");

        // Create a new file at the original location (simulates manual restoration)
        fs::write(&file_path, "new content").expect("Failed to create conflict");

        // Undo
        let report = UndoManager::undo(base_path).expect("Undo failed");

        // Verify the operation succeeded with backup created
        assert_eq!(report.restored_files, 1);
        assert_eq!(report.failed_restores.len(), 0);

        // Original file should have the moved content
        let moved_content = fs::read_to_string(&file_path).expect("Failed to read file");
        assert_eq!(moved_content, "original content");

        // New content should be backed up
        let backup_files: Vec<_> = fs::read_dir(base_path)
            .expect("Failed to read dir")
            .filter_map(|e| {
                e.ok().and_then(|entry| {
                    let path = entry.path();
                    if path.file_name()?.to_string_lossy().contains("bak") {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();

        assert_eq!(backup_files.len(), 1);
    }

    #[test]
    fn test_undo_with_missing_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create an operation log with a file that doesn't exist
        let operation = Operation {
            original_path: base_path.join("nonexistent.txt"),
            new_path: base_path.join("documents").join("nonexistent.txt"),
            category: "documents".to_string(),
        };

        let mut log = OperationLog::new(base_path.to_path_buf());
        log.add_operation(operation);
        log.save(base_path).expect("Failed to save history");

        // Attempt undo
        let report = UndoManager::undo(base_path).expect("Undo failed");

        // Should have skipped the file
        assert_eq!(report.restored_files, 0);
        assert_eq!(report.skipped_files.len(), 1);
    }

    #[test]
    fn test_undo_invalid_base_path() {
        let non_existent = Path::new("/non/existent/path");
        let result = UndoManager::undo(non_existent);
        assert!(result.is_err());
    }
}
