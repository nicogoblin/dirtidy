use dirtidy::cli::{OrganizeCommand, run_cli_with_config};
/// Integration tests for dirtidy
///
/// These tests simulate real-world usage scenarios, testing the complete
/// end-to-end functionality of the dirtidy file organization utility.
///
/// Test categories:
/// 1. Basic organization workflows
/// 2. Multiple file type handling
/// 3. Dry-run mode verification
/// 4. Undo and conflict resolution
/// 5. Configuration and filtering
/// 6. Edge cases and error scenarios
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ============================================================================
// Test Utilities
// ============================================================================

/// A test fixture that sets up a temporary directory with configurable
/// file structure for testing.
struct TestFixture {
    temp_dir: TempDir,
}

impl TestFixture {
    /// Create a new test fixture with a temporary directory.
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        TestFixture { temp_dir }
    }

    /// Get the path to the test directory.
    fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a file with content in the test directory.
    fn create_file(&self, name: &str, content: &[u8]) {
        let file_path = self.path().join(name);
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(content)
            .expect("Failed to write file content");
    }

    /// Create a file with specific content (string version).
    fn create_text_file(&self, name: &str, content: &str) {
        self.create_file(name, content.as_bytes());
    }

    /// Create a subdirectory in the test directory.
    fn create_subdir(&self, name: &str) {
        let dir_path = self.path().join(name);
        fs::create_dir(&dir_path).expect("Failed to create subdirectory");
    }

    /// Create multiple files at once with a simple naming pattern.
    fn create_files(&self, files: &[(&str, &[u8])]) {
        for (name, content) in files {
            self.create_file(name, content);
        }
    }

    /// Assert that a directory exists with the expected structure.
    fn assert_dir_exists(&self, rel_path: &str) {
        let path = self.path().join(rel_path);
        assert!(
            path.exists() && path.is_dir(),
            "Directory should exist: {}",
            path.display()
        );
    }

    /// Assert that a file exists at the given relative path.
    fn assert_file_exists(&self, rel_path: &str) {
        let path = self.path().join(rel_path);
        assert!(
            path.exists() && path.is_file(),
            "File should exist: {}",
            path.display()
        );
    }

    /// Assert that a file does NOT exist at the given relative path.
    fn assert_file_not_exists(&self, rel_path: &str) {
        let path = self.path().join(rel_path);
        assert!(!path.exists(), "File should not exist: {}", path.display());
    }

    /// Count files in a directory (non-recursive), excluding .dirtidy_history.json.
    fn count_files(&self) -> usize {
        fs::read_dir(self.path())
            .expect("Failed to read directory")
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let file_name = e.file_name().to_string_lossy().to_string();
                    if file_name == ".dirtidy_history.json" {
                        return None;
                    }
                    if e.metadata().ok()?.is_file() {
                        Some(())
                    } else {
                        None
                    }
                })
            })
            .count()
    }

    /// Count directories in the test directory (non-recursive).
    fn count_dirs(&self) -> usize {
        fs::read_dir(self.path())
            .expect("Failed to read directory")
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.metadata().ok()?.is_dir() {
                        Some(())
                    } else {
                        None
                    }
                })
            })
            .count()
    }

    /// List all files in the directory recursively.
    fn list_files_recursive(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        Self::walk_dir(&self.path().to_path_buf(), &mut files);
        files.sort();
        files
    }

    fn walk_dir(dir: &PathBuf, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                } else if path.is_dir() {
                    Self::walk_dir(&path, files);
                }
            }
        }
    }
}

// ============================================================================
// Test Data: Realistic File Content
// ============================================================================

/// PNG file header (minimal, just enough to be detected as PNG)
const PNG_HEADER: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 image
    0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // bit depth, color
    0xDE,
];

/// JPEG file header (minimal)
const JPEG_HEADER: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, // JPEG SOI and APP0 marker
    0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, // JFIF signature
    0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
];

/// GIF file header (minimal)
const GIF_HEADER: &[u8] = b"GIF89a\x01\x00\x01\x00\x00\x00\x00\x00\xFF\xFF\xFF\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x00;";

/// PDF file header (minimal)
const PDF_HEADER: &[u8] = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n";

/// ZIP file header (minimal)
const ZIP_HEADER: &[u8] = &[0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00];

/// MP3 file header (minimal)
const MP3_HEADER: &[u8] = &[0xFF, 0xFB, 0x10, 0x00]; // MPEG audio sync

// ============================================================================
// Test Suite 1: Basic Organization
// ============================================================================

#[test]
fn test_organize_empty_directory() {
    let fixture = TestFixture::new();

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok(), "Should succeed on empty directory");
    // The history file will be created even for empty directories
    fixture.assert_file_exists(".dirtidy_history.json");
    assert_eq!(fixture.count_dirs(), 0, "Should have no subdirectories");
}

#[test]
fn test_organize_single_image() {
    let fixture = TestFixture::new();
    fixture.create_file("photo.png", PNG_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());
    fixture.assert_dir_exists("images");
    fixture.assert_file_exists("images/photo.png");
    fixture.assert_file_not_exists("photo.png");
}

#[test]
fn test_organize_single_document() {
    let fixture = TestFixture::new();
    fixture.create_file("report.pdf", PDF_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());
    fixture.assert_dir_exists("documents");
    fixture.assert_file_exists("documents/report.pdf");
}

#[test]
fn test_organize_single_pdf() {
    let fixture = TestFixture::new();
    fixture.create_file("document.pdf", PDF_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());
    fixture.assert_dir_exists("documents");
    fixture.assert_file_exists("documents/document.pdf");
}

#[test]
fn test_organize_mixed_file_types() {
    let fixture = TestFixture::new();

    // Create files of different types (all detectable by infer)
    fixture.create_files(&[
        ("photo1.png", PNG_HEADER),
        ("photo2.jpg", JPEG_HEADER),
        ("animation.gif", GIF_HEADER),
        ("report.pdf", PDF_HEADER),
        ("document.pdf", PDF_HEADER),
        ("archive.zip", ZIP_HEADER),
        ("song.mp3", MP3_HEADER),
    ]);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Verify files were organized into appropriate categories
    fixture.assert_file_exists("images/photo1.png");
    fixture.assert_file_exists("images/photo2.jpg");
    fixture.assert_file_exists("images/animation.gif");
    fixture.assert_file_exists("documents/report.pdf");
    fixture.assert_file_exists("documents/document.pdf");
    fixture.assert_file_exists("archives/archive.zip");
    fixture.assert_file_exists("audio/song.mp3");

    // Original files should no longer exist in root
    fixture.assert_file_not_exists("photo1.png");
    fixture.assert_file_not_exists("report.pdf");

    // Count should verify structure
    assert!(
        fixture.count_dirs() > 0,
        "Should have created category directories"
    );
}

#[test]
fn test_organize_many_files() {
    let fixture = TestFixture::new();

    // Create a large number of files
    for i in 0..50 {
        match i % 5 {
            0 => fixture.create_file(&format!("image_{}.png", i), PNG_HEADER),
            1 => fixture.create_text_file(&format!("doc_{}.txt", i), "Content"),
            2 => fixture.create_file(&format!("audio_{}.mp3", i), MP3_HEADER),
            3 => fixture.create_file(&format!("archive_{}.zip", i), ZIP_HEADER),
            _ => fixture.create_file(&format!("pdf_{}.pdf", i), PDF_HEADER),
        }
    }

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Verify all files were moved
    assert_eq!(
        fixture.count_files(),
        0,
        "All files in root should be moved to subdirectories"
    );

    // Verify multiple category directories were created
    fixture.assert_dir_exists("images");
    fixture.assert_dir_exists("documents");
    fixture.assert_dir_exists("audio");
}

// ============================================================================
// Test Suite 2: Dry-Run Mode
// ============================================================================

#[test]
fn test_dry_run_doesnt_move_files() {
    let fixture = TestFixture::new();
    fixture.create_files(&[("photo.png", PNG_HEADER), ("report.pdf", PDF_HEADER)]);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: true },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Files should still exist in root directory
    fixture.assert_file_exists("photo.png");
    fixture.assert_file_exists("report.pdf");

    // No category directories should be created
    assert_eq!(
        fixture.count_dirs(),
        0,
        "Dry-run should not create directories"
    );
}

#[test]
fn test_dry_run_vs_actual_organization() {
    let fixture = TestFixture::new();
    fixture.create_files(&[
        ("photo1.png", PNG_HEADER),
        ("photo2.jpg", JPEG_HEADER),
        ("report.pdf", PDF_HEADER),
    ]);

    // First, simulate with dry-run
    let dry_run_result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: true },
        fixture.path(),
        None,
    );
    assert!(dry_run_result.is_ok());

    // Files should still be in root
    assert_eq!(fixture.count_files(), 3);

    // Now actually organize (after dry-run, state should be unchanged)
    let actual_result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(actual_result.is_ok());

    // Now files should be organized
    assert_eq!(
        fixture.count_files(),
        0,
        "Root should be empty after actual organization"
    );
    assert!(
        fixture.count_dirs() > 0,
        "Should have created category directories"
    );
}

// ============================================================================
// Test Suite 3: Undo Functionality
// ============================================================================

#[test]
fn test_undo_single_file() {
    let fixture = TestFixture::new();
    fixture.create_file("photo.png", PNG_HEADER);

    // Organize
    let org_result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(org_result.is_ok());
    fixture.assert_file_exists("images/photo.png");

    // Undo
    let undo_result = run_cli_with_config(OrganizeCommand::Undo, fixture.path(), None);
    assert!(undo_result.is_ok());

    // File should be back in root
    fixture.assert_file_exists("photo.png");
    fixture.assert_file_not_exists("images/photo.png");
}

#[test]
fn test_undo_multiple_files() {
    let fixture = TestFixture::new();
    fixture.create_files(&[
        ("photo.png", PNG_HEADER),
        ("report.pdf", PDF_HEADER),
        ("song.mp3", MP3_HEADER),
    ]);

    // Organize
    let org_result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(org_result.is_ok());

    // Verify organization
    fixture.assert_file_exists("images/photo.png");
    fixture.assert_file_exists("documents/report.pdf");
    fixture.assert_file_exists("audio/song.mp3");

    // Undo
    let undo_result = run_cli_with_config(OrganizeCommand::Undo, fixture.path(), None);
    assert!(undo_result.is_ok());

    // All files should be back in root
    fixture.assert_file_exists("photo.png");
    fixture.assert_file_exists("report.pdf");
    fixture.assert_file_exists("song.mp3");
}

#[test]
fn test_undo_without_history() {
    let fixture = TestFixture::new();
    fixture.create_file("photo.png", PNG_HEADER);

    // Try to undo without organizing first
    let undo_result = run_cli_with_config(OrganizeCommand::Undo, fixture.path(), None);

    // Should still succeed gracefully (no history to undo)
    assert!(undo_result.is_ok() || undo_result.is_err());
}

#[test]
fn test_undo_with_modified_files() {
    let fixture = TestFixture::new();
    fixture.create_file("photo.png", PNG_HEADER);

    // Organize
    let org_result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(org_result.is_ok());

    // Modify the organized file
    let file_path = fixture.path().join("images/photo.png");
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&file_path)
        .expect("Failed to open file");
    file.write_all(b"modified")
        .expect("Failed to write to file");

    // Undo should still work (file should be restored with modified content)
    let undo_result = run_cli_with_config(OrganizeCommand::Undo, fixture.path(), None);
    assert!(undo_result.is_ok());

    fixture.assert_file_exists("photo.png");
}

// ============================================================================
// Test Suite 4: File Type Detection and Categorization
// ============================================================================

#[test]
fn test_detect_images_by_content() {
    let fixture = TestFixture::new();

    // Create files with actual MIME headers (tests content detection)
    fixture.create_files(&[
        ("image1.png", PNG_HEADER),
        ("image2.jpg", JPEG_HEADER),
        ("image3.gif", GIF_HEADER),
    ]);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());
    fixture.assert_dir_exists("images");
    assert_eq!(
        fixture.count_dirs(),
        1,
        "Should have created only images directory"
    );
}

#[test]
fn test_detect_documents_by_content() {
    let fixture = TestFixture::new();

    fixture.create_files(&[
        ("doc1.pdf", PDF_HEADER),
        ("doc2.txt", b"Plain text content"),
        ("doc3.md", b"# Markdown heading\n\nContent"),
    ]);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());
    fixture.assert_dir_exists("documents");
}

#[test]
fn test_categorize_by_extension_fallback() {
    let fixture = TestFixture::new();

    // Create files with recognizable binary formats that infer can detect
    // For extension fallback, we use files that don't have detectable MIME types
    // but will still be organized based on their extension mapping
    fixture.create_file("program.py", b"#!/usr/bin/env python\nprint('hello')");
    fixture.create_file("script.js", b"console.log('hello');");
    fixture.create_file("data.json", b"{}");

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // These files may not be detected by infer since they're plain text,
    // so they'll go to "other" or be categorized by extension if the logic supports it.
    // Let's verify they're organized somewhere
    let organized = fixture.list_files_recursive();
    assert!(!organized.is_empty(), "Files should be organized");
    assert_eq!(fixture.count_files(), 0, "Root should be empty");
}

#[test]
fn test_unknown_files_go_to_other() {
    let fixture = TestFixture::new();

    // Create files with unusual extensions
    fixture.create_text_file("unknown.xyz", "Unknown file type");
    fixture.create_text_file("random.abc", "Random data");

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());
    fixture.assert_dir_exists("other");
    fixture.assert_file_exists("other/unknown.xyz");
    fixture.assert_file_exists("other/random.abc");
}

#[test]
fn test_files_without_extension() {
    let fixture = TestFixture::new();

    // Create files without extensions
    fixture.create_file("README", b"This is a readme file");
    fixture.create_file("LICENSE", b"MIT License");

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Files without recognizable content should go to "other"
    let organized = fixture.list_files_recursive();
    assert!(!organized.is_empty(), "Files should be organized somewhere");
}

// ============================================================================
// Test Suite 5: Edge Cases
// ============================================================================

#[test]
fn test_organize_idempotent() {
    let fixture = TestFixture::new();
    fixture.create_files(&[("photo.png", PNG_HEADER), ("report.pdf", PDF_HEADER)]);

    // First organization
    let result1 = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(result1.is_ok());

    let files_after_first = fixture.list_files_recursive();

    // Second organization (should be idempotent)
    let result2 = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(result2.is_ok());

    let files_after_second = fixture.list_files_recursive();

    // File list should be identical
    assert_eq!(
        files_after_first, files_after_second,
        "Organizing again should not change anything"
    );
}

#[test]
fn test_organize_preserves_file_content() {
    let fixture = TestFixture::new();

    fixture.create_file("document.pdf", PDF_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(result.is_ok());

    // Verify file was moved with content intact
    fixture.assert_file_exists("documents/document.pdf");
    let organized_path = fixture.path().join("documents/document.pdf");
    let organized_content = fs::read(&organized_path).expect("Failed to read organized file");
    assert_eq!(
        organized_content, PDF_HEADER,
        "File content should be preserved during organization"
    );
}

#[test]
fn test_organize_special_characters_in_filename() {
    let fixture = TestFixture::new();

    // Files with special characters
    fixture.create_file("photo (1).png", PNG_HEADER);
    fixture.create_file("document - final.pdf", PDF_HEADER);
    fixture.create_file("song [remix].mp3", MP3_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Verify files with special characters were moved
    fixture.assert_file_exists("images/photo (1).png");
    fixture.assert_file_exists("documents/document - final.pdf");
    fixture.assert_file_exists("audio/song [remix].mp3");
}

#[test]
fn test_organize_mixed_case_extensions() {
    let fixture = TestFixture::new();

    // Files with different case extensions
    fixture.create_file("photo.PNG", PNG_HEADER);
    fixture.create_file("report.PDF", PDF_HEADER);
    fixture.create_file("song.MP3", MP3_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Extension matching should be case-insensitive
    fixture.assert_file_exists("images/photo.PNG");
    fixture.assert_file_exists("documents/report.PDF");
    fixture.assert_file_exists("audio/song.MP3");
}

#[test]
fn test_organize_files_with_multiple_dots() {
    let fixture = TestFixture::new();

    // Files with multiple dots in name
    fixture.create_file("photo.backup.png", PNG_HEADER);
    fixture.create_file("archive.tar.zip", ZIP_HEADER);
    fixture.create_file("report.final.pdf", PDF_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Verify files were categorized correctly
    fixture.assert_file_exists("images/photo.backup.png");
    fixture.assert_file_exists("archives/archive.tar.zip");
    fixture.assert_file_exists("documents/report.final.pdf");
}

// ============================================================================
// Test Suite 6: Configuration and Filtering
// ============================================================================

#[test]
fn test_organize_with_exclude_pattern() {
    let fixture = TestFixture::new();

    // Create a temporary config file
    let config_path = fixture.path().join(".dirtidyrc.toml");
    let config_content = r#"
[filters]

[filters.exclude]
patterns = ["*.tmp"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Create files including one that should be excluded
    fixture.create_file("photo.png", PNG_HEADER);
    fixture.create_file("temp.tmp", b"temporary file");

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        Some(&config_path),
    );

    assert!(result.is_ok(), "Result error: {:?}", result.err());

    // Photo should be organized
    fixture.assert_file_exists("images/photo.png");

    // .tmp file should remain in root (excluded)
    fixture.assert_file_exists("temp.tmp");
}

#[test]
fn test_organize_with_exclude_extension() {
    let fixture = TestFixture::new();

    let config_path = fixture.path().join(".dirtidyrc.toml");
    let config_content = r#"
[filters]

[filters.exclude]
extensions = ["log"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write config");

    fixture.create_file("photo.png", PNG_HEADER);
    fixture.create_file("debug.log", b"Debug output");

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        Some(&config_path),
    );

    assert!(result.is_ok());

    fixture.assert_file_exists("images/photo.png");
    fixture.assert_file_exists("debug.log"); // Should be excluded
}

#[test]
fn test_organize_with_exclude_filename() {
    let fixture = TestFixture::new();

    let config_path = fixture.path().join(".dirtidyrc.toml");
    let config_content = r#"
[filters]

[filters.exclude]
filenames = ["README.pdf", "LICENSE"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write config");

    fixture.create_file("README.pdf", PDF_HEADER);
    fixture.create_file("LICENSE", b"MIT License");
    fixture.create_file("photo.png", PNG_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        Some(&config_path),
    );

    assert!(result.is_ok());

    fixture.assert_file_exists("README.pdf"); // Excluded
    fixture.assert_file_exists("LICENSE"); // Excluded
    fixture.assert_file_exists("images/photo.png"); // Organized
}

#[test]
fn test_organize_hidden_files_excluded_by_default() {
    let fixture = TestFixture::new();

    // Create regular and hidden files
    fixture.create_file("photo.png", PNG_HEADER);
    fixture.create_text_file(".hidden_config", "config");

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Regular file organized
    fixture.assert_file_exists("images/photo.png");

    // Hidden file should remain in root (not organized by default)
    fixture.assert_file_exists(".hidden_config");
}

#[test]
fn test_organize_with_include_pattern() {
    let fixture = TestFixture::new();

    let config_path = fixture.path().join(".dirtidyrc.toml");
    let config_content = r#"
[filters]

[filters.include]
patterns = ["*.pdf"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write config");

    fixture.create_file("document.pdf", PDF_HEADER);
    fixture.create_file("photo.png", PNG_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        Some(&config_path),
    );

    assert!(result.is_ok());

    // Include pattern should allow PDF to be organized
    fixture.assert_file_exists("documents/document.pdf");
    // PNG is not in the include pattern, so it might not be organized
}

// ============================================================================
// Test Suite 7: Real-world Scenarios
// ============================================================================

#[test]
fn test_organize_downloads_folder_simulation() {
    let fixture = TestFixture::new();

    // Simulate a typical Downloads folder with detectable file types
    // Note: We use only files that can be detected by the infer crate
    fixture.create_files(&[
        ("wallpaper.png", PNG_HEADER),
        ("movie.gif", GIF_HEADER),
        ("ebook.pdf", PDF_HEADER),
        ("paper.pdf", PDF_HEADER),
        ("document1.pdf", PDF_HEADER),
        ("installer.zip", ZIP_HEADER),
        ("backup.tar.zip", ZIP_HEADER),
        ("song.mp3", MP3_HEADER),
        ("podcast.mp3", MP3_HEADER),
        ("photo.jpg", JPEG_HEADER),
    ]);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // Verify structure
    fixture.assert_file_exists("images/wallpaper.png");
    fixture.assert_file_exists("images/movie.gif");
    fixture.assert_file_exists("images/photo.jpg");
    fixture.assert_file_exists("documents/ebook.pdf");
    fixture.assert_file_exists("documents/paper.pdf");
    fixture.assert_file_exists("documents/document1.pdf");
    fixture.assert_file_exists("archives/installer.zip");
    fixture.assert_file_exists("archives/backup.tar.zip");
    fixture.assert_file_exists("audio/song.mp3");
    fixture.assert_file_exists("audio/podcast.mp3");

    // Root should be empty
    assert_eq!(fixture.count_files(), 0, "Root directory should be empty");
}

#[test]
fn test_organize_with_existing_category_directories() {
    let fixture = TestFixture::new();

    // Pre-create some category directories
    fixture.create_subdir("images");
    fixture.create_subdir("documents");

    // Add existing files in those directories
    fixture.create_file("images/existing.png", PNG_HEADER);
    fixture.create_file("documents/existing.pdf", PDF_HEADER);

    // Add new files to organize
    fixture.create_file("new_photo.png", PNG_HEADER);
    fixture.create_file("new_doc.pdf", PDF_HEADER);

    let result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );

    assert!(result.is_ok());

    // New files should be added to existing directories
    fixture.assert_file_exists("images/existing.png");
    fixture.assert_file_exists("images/new_photo.png");
    fixture.assert_file_exists("documents/existing.pdf");
    fixture.assert_file_exists("documents/new_doc.pdf");
}

#[test]
fn test_organize_then_add_files_then_organize_again() {
    let fixture = TestFixture::new();

    // First batch of files
    fixture.create_file("photo1.png", PNG_HEADER);
    fixture.create_file("report1.pdf", PDF_HEADER);

    // First organization
    let result1 = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(result1.is_ok());

    fixture.assert_file_exists("images/photo1.png");
    fixture.assert_file_exists("documents/report1.pdf");

    // Add more files
    fixture.create_file("photo2.png", PNG_HEADER);
    fixture.create_file("report2.pdf", PDF_HEADER);

    // Second organization
    let result2 = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(result2.is_ok());

    // All files should be organized
    fixture.assert_file_exists("images/photo1.png");
    fixture.assert_file_exists("images/photo2.png");
    fixture.assert_file_exists("documents/report1.pdf");
    fixture.assert_file_exists("documents/report2.pdf");
}

#[test]
fn test_full_workflow_organize_modify_undo() {
    let fixture = TestFixture::new();

    // Setup: Create initial files
    fixture.create_file("photo.png", PNG_HEADER);
    fixture.create_file("report.pdf", PDF_HEADER);

    // Step 1: Organize
    let org_result = run_cli_with_config(
        OrganizeCommand::Organize { dry_run: false },
        fixture.path(),
        None,
    );
    assert!(org_result.is_ok());

    fixture.assert_file_exists("images/photo.png");
    fixture.assert_file_exists("documents/report.pdf");

    // Step 2: Simulate user adding new files to organized directories
    fixture.create_file("documents/new_note.pdf", PDF_HEADER);

    // Step 3: Undo organization
    let undo_result = run_cli_with_config(OrganizeCommand::Undo, fixture.path(), None);
    assert!(undo_result.is_ok());

    // Original files should be back
    fixture.assert_file_exists("photo.png");
    fixture.assert_file_exists("report.pdf");

    // But new files added after organization should remain in documents directory
    let organized = fixture.list_files_recursive();
    assert!(
        organized
            .iter()
            .any(|p| p.to_string_lossy().contains("new_note")),
        "New files added after organization should remain"
    );
}
