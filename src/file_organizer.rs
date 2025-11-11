/// File organization system for moving files into category directories.
///
/// This module provides functionality to organize files by moving them into
/// category-specific subdirectories within a given base directory.
/// It handles directory creation, file movement, and operation history logging.
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a single file organization operation.
///
/// This struct records the original and new paths of a file that was moved
/// during an organization run, enabling undo functionality.
#[derive(Debug, Clone)]
pub struct Operation {
    /// The original path of the file before organization.
    pub original_path: PathBuf,
    /// The new path of the file after organization.
    pub new_path: PathBuf,
    /// The category the file was moved to.
    pub category: String,
}

/// Represents a complete transaction of file operations.
///
/// This is persisted to disk to enable undo functionality.
#[derive(Debug, Clone)]
pub struct OperationLog {
    /// ISO 8601 timestamp of when the organization occurred.
    pub timestamp: String,
    /// The base directory where organization occurred.
    pub base_path: PathBuf,
    /// All operations performed in this organization run.
    pub operations: Vec<Operation>,
}

impl OperationLog {
    /// Creates a new operation log for a given base path.
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            base_path,
            operations: Vec::new(),
        }
    }

    /// Adds an operation to this log.
    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    /// Returns the path to the history file for this base path.
    fn history_file_path(base_path: &Path) -> PathBuf {
        base_path.join(".dirtidy_history.json")
    }

    /// Saves this log to disk in JSON format.
    pub fn save(&self, base_path: &Path) -> OrganizeResult<()> {
        let json = json!({
            "timestamp": self.timestamp,
            "base_path": self.base_path.to_string_lossy().to_string(),
            "operations": self.operations.iter().map(|op| {
                json!({
                    "original_path": op.original_path.to_string_lossy().to_string(),
                    "new_path": op.new_path.to_string_lossy().to_string(),
                    "category": op.category,
                })
            }).collect::<Vec<_>>(),
        });

        let history_path = Self::history_file_path(base_path);
        let json_string =
            serde_json::to_string_pretty(&json).map_err(|e| OrganizeError::HistoryWriteFailed {
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("JSON serialization failed: {}", e),
                ),
            })?;

        fs::write(&history_path, json_string)
            .map_err(|e| OrganizeError::HistoryWriteFailed { source: e })?;

        Ok(())
    }

    /// Loads the most recent operation log from disk.
    pub fn load(base_path: &Path) -> OrganizeResult<Option<Self>> {
        let history_path = Self::history_file_path(base_path);

        if !history_path.exists() {
            return Ok(None);
        }

        let json_string = fs::read_to_string(&history_path)
            .map_err(|e| OrganizeError::HistoryReadFailed { source: e })?;

        let json: Value = serde_json::from_str(&json_string).map_err(|e| {
            OrganizeError::InvalidHistoryFormat {
                reason: format!("JSON parse error: {}", e),
            }
        })?;

        let timestamp = json["timestamp"]
            .as_str()
            .ok_or_else(|| OrganizeError::InvalidHistoryFormat {
                reason: "Missing or invalid 'timestamp' field".to_string(),
            })?
            .to_string();

        let base_path_str =
            json["base_path"]
                .as_str()
                .ok_or_else(|| OrganizeError::InvalidHistoryFormat {
                    reason: "Missing or invalid 'base_path' field".to_string(),
                })?;

        let ops_array =
            json["operations"]
                .as_array()
                .ok_or_else(|| OrganizeError::InvalidHistoryFormat {
                    reason: "Missing or invalid 'operations' field".to_string(),
                })?;

        let operations: Result<Vec<_>, _> =
            ops_array
                .iter()
                .map(|op| {
                    let original_path = op["original_path"].as_str().ok_or_else(|| {
                        OrganizeError::InvalidHistoryFormat {
                            reason: "Missing 'original_path' in operation".to_string(),
                        }
                    })?;
                    let new_path = op["new_path"].as_str().ok_or_else(|| {
                        OrganizeError::InvalidHistoryFormat {
                            reason: "Missing 'new_path' in operation".to_string(),
                        }
                    })?;
                    let category = op["category"].as_str().ok_or_else(|| {
                        OrganizeError::InvalidHistoryFormat {
                            reason: "Missing 'category' in operation".to_string(),
                        }
                    })?;

                    Ok(Operation {
                        original_path: PathBuf::from(original_path),
                        new_path: PathBuf::from(new_path),
                        category: category.to_string(),
                    })
                })
                .collect();

        Ok(Some(OperationLog {
            timestamp,
            base_path: PathBuf::from(base_path_str),
            operations: operations?,
        }))
    }

    /// Deletes the history file for a given base path.
    pub fn delete(base_path: &Path) -> OrganizeResult<()> {
        let history_path = Self::history_file_path(base_path);
        if history_path.exists() {
            fs::remove_file(&history_path)
                .map_err(|e| OrganizeError::HistoryWriteFailed { source: e })?;
        }
        Ok(())
    }
}

/// Errors that can occur during file organization operations.
#[derive(Debug)]
pub enum OrganizeError {
    /// Failed to create a category directory.
    DirectoryCreationFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Failed to move a file to its category directory.
    FileMoveFailure {
        source: PathBuf,
        destination: PathBuf,
        source_error: std::io::Error,
    },
    /// The base directory path is invalid or doesn't exist.
    InvalidBasePath {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Failed to write history file.
    HistoryWriteFailed { source: std::io::Error },
    /// Failed to read history file.
    HistoryReadFailed { source: std::io::Error },
    /// History file has invalid format.
    InvalidHistoryFormat { reason: String },
}

impl std::fmt::Display for OrganizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectoryCreationFailed { path, source } => {
                write!(
                    f,
                    "Failed to create directory {}: {}",
                    path.display(),
                    source
                )
            }
            Self::FileMoveFailure {
                source,
                destination,
                source_error,
            } => {
                write!(
                    f,
                    "Failed to move {} to {}: {}",
                    source.display(),
                    destination.display(),
                    source_error
                )
            }
            Self::InvalidBasePath { path, source } => {
                write!(f, "Invalid base path {}: {}", path.display(), source)
            }
            Self::HistoryWriteFailed { source } => {
                write!(f, "Failed to write history file: {}", source)
            }
            Self::HistoryReadFailed { source } => {
                write!(f, "Failed to read history file: {}", source)
            }
            Self::InvalidHistoryFormat { reason } => {
                write!(f, "Invalid history file format: {}", reason)
            }
        }
    }
}

impl std::error::Error for OrganizeError {}

/// Result type for file organization operations.
pub type OrganizeResult<T> = Result<T, OrganizeError>;

/// Organizes files by moving them into category subdirectories.
///
/// This struct handles the logistics of organizing files within a base directory.
/// It creates category subdirectories as needed and moves files into them.
pub struct FileOrganizer;

impl FileOrganizer {
    /// Moves a file into its category directory within the base path and records the operation.
    ///
    /// If the category directory doesn't exist, it is created automatically.
    /// The function validates that the base path exists before attempting any operations.
    /// Returns the operation that was performed for history recording.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The root directory where category subdirectories will be created
    /// * `file_path` - The full path to the file to be moved
    /// * `category_dir_name` - The name of the subdirectory for this file's category
    ///
    /// # Returns
    ///
    /// Returns `Ok(Operation)` on successful move, or an `OrganizeError` if any operation fails.
    /// The `Operation` struct records the original and new paths for undo functionality.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dirtidy::file_organizer::FileOrganizer;
    /// use std::path::Path;
    ///
    /// let result = FileOrganizer::move_to_category_with_record(
    ///     Path::new("/path/to/base"),
    ///     Path::new("/path/to/base/image.png"),
    ///     "images"
    /// );
    ///
    /// match result {
    ///     Ok(op) => println!("Moved {} to {}", op.original_path.display(), op.new_path.display()),
    ///     Err(e) => eprintln!("Organization failed: {}", e),
    /// }
    /// ```
    pub fn move_to_category_with_record(
        base_path: &Path,
        file_path: &Path,
        category_dir_name: &str,
    ) -> OrganizeResult<Operation> {
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

        // Construct the category directory path
        let category_path = base_path.join(category_dir_name);

        // Create the category directory if it doesn't exist
        if !category_path.exists() {
            fs::create_dir(&category_path).map_err(|e| OrganizeError::DirectoryCreationFailed {
                path: category_path.clone(),
                source: e,
            })?;
        }

        // Construct the destination path for the file
        let file_name = file_path
            .file_name()
            .ok_or_else(|| OrganizeError::FileMoveFailure {
                source: file_path.to_path_buf(),
                destination: category_path.clone(),
                source_error: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "file has no name component",
                ),
            })?;

        let destination_path = category_path.join(file_name);

        // Move the file to the category directory
        fs::rename(file_path, &destination_path).map_err(|e| OrganizeError::FileMoveFailure {
            source: file_path.to_path_buf(),
            destination: destination_path.clone(),
            source_error: e,
        })?;

        // Record the operation
        Ok(Operation {
            original_path: file_path.to_path_buf(),
            new_path: destination_path,
            category: category_dir_name.to_string(),
        })
    }

    /// Moves a file into its category directory within the base path.
    ///
    /// If the category directory doesn't exist, it is created automatically.
    /// The function validates that the base path exists before attempting any operations.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The root directory where category subdirectories will be created
    /// * `file_path` - The full path to the file to be moved
    /// * `category_dir_name` - The name of the subdirectory for this file's category
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful move, or an `OrganizeError` if any operation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dirtidy::file_organizer::FileOrganizer;
    /// use std::path::Path;
    ///
    /// let result = FileOrganizer::move_to_category(
    ///     Path::new("/path/to/base"),
    ///     Path::new("/path/to/base/image.png"),
    ///     "images"
    /// );
    ///
    /// match result {
    ///     Ok(()) => println!("File organized successfully"),
    ///     Err(e) => eprintln!("Organization failed: {}", e),
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn move_to_category(
        base_path: &Path,
        file_path: &Path,
        category_dir_name: &str,
    ) -> OrganizeResult<()> {
        Self::move_to_category_with_record(base_path, file_path, category_dir_name).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_move_to_category_creates_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create a test file
        let file_path = base_path.join("test.txt");
        fs::write(&file_path, "test content").expect("Failed to write test file");

        // Move the file
        FileOrganizer::move_to_category(base_path, &file_path, "documents")
            .expect("Failed to move file");

        // Verify the category directory was created
        let category_dir = base_path.join("documents");
        assert!(category_dir.exists());
        assert!(category_dir.is_dir());

        // Verify the file was moved
        assert!(!file_path.exists());
        let moved_file = category_dir.join("test.txt");
        assert!(moved_file.exists());
    }

    #[test]
    fn test_move_to_category_uses_existing_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create a category directory
        let category_dir = base_path.join("images");
        fs::create_dir(&category_dir).expect("Failed to create category directory");

        // Create a test file
        let file_path = base_path.join("test.png");
        fs::write(&file_path, "test content").expect("Failed to write test file");

        // Move the file
        FileOrganizer::move_to_category(base_path, &file_path, "images")
            .expect("Failed to move file");

        // Verify the file was moved to the existing directory
        assert!(!file_path.exists());
        let moved_file = category_dir.join("test.png");
        assert!(moved_file.exists());
    }

    #[test]
    fn test_move_to_category_invalid_base_path() {
        let non_existent = Path::new("/non/existent/path");
        let file_path = Path::new("/some/file.txt");

        let result = FileOrganizer::move_to_category(non_existent, file_path, "documents");
        assert!(result.is_err());
    }
}
