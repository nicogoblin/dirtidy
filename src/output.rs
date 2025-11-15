//! Output formatting and styling module.
//!
//! Provides a centralized interface for all CLI output, including colored output,
//! progress tracking, and formatted tables. This module abstracts away output details,
//! making it easy to change formatting globally.

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;

/// Manages all CLI output with consistent styling and formatting.
///
/// This struct provides methods for:
/// - Success messages (green with ✓)
/// - Error messages (red with ✗)
/// - Warning messages (yellow with ⚠)
/// - Info messages (cyan)
/// - Progress bars for operations
/// - Summary tables with statistics
pub struct OutputFormatter;

impl OutputFormatter {
    /// Prints a success message in green with a checkmark.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dirtidy::output::OutputFormatter;
    /// OutputFormatter::success("File organized successfully!");
    /// ```
    pub fn success(message: &str) {
        println!("{} {}", "✓".green(), message);
    }

    /// Prints an error message in red with an X mark.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dirtidy::output::OutputFormatter;
    /// OutputFormatter::error("Failed to organize file");
    /// ```
    pub fn error(message: &str) {
        eprintln!("{} {}", "✗".red(), message);
    }

    /// Prints a warning message in yellow with a warning symbol.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dirtidy::output::OutputFormatter;
    /// OutputFormatter::warning("Some files could not be organized");
    /// ```
    pub fn warning(message: &str) {
        println!("{} {}", "⚠".yellow(), message);
    }

    /// Prints an info message in cyan.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dirtidy::output::OutputFormatter;
    /// OutputFormatter::info("Organizing directory: /home/user/Downloads");
    /// ```
    pub fn info(message: &str) {
        println!("{}", message.cyan());
    }

    /// Prints a regular message without styling.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    pub fn plain(message: &str) {
        println!("{}", message);
    }

    /// Prints a section header.
    ///
    /// # Arguments
    ///
    /// * `header` - The header text
    pub fn header(header: &str) {
        println!("\n{}", header.bold());
    }

    /// Creates and returns a progress bar for file operations.
    ///
    /// # Arguments
    ///
    /// * `total` - Total number of items to process
    ///
    /// # Returns
    ///
    /// A configured `ProgressBar` ready for use.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dirtidy::output::OutputFormatter;
    /// let pb = OutputFormatter::create_progress_bar(100);
    /// pb.inc(1); // Increment by 1
    /// pb.finish_with_message("Completed!");
    /// ```
    pub fn create_progress_bar(total: u64) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("█▓░"),
        );
        pb
    }

    /// Prints a summary table with file statistics by category.
    ///
    /// # Arguments
    ///
    /// * `category_counts` - HashMap of category names to file counts
    /// * `total_files` - Total number of files organized
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dirtidy::output::OutputFormatter;
    /// use std::collections::HashMap;
    ///
    /// let mut counts = HashMap::new();
    /// counts.insert("Documents".to_string(), 15);
    /// counts.insert("Images".to_string(), 8);
    /// OutputFormatter::summary_table(&counts, 23);
    /// ```
    pub fn summary_table(category_counts: &HashMap<String, usize>, total_files: usize) {
        Self::header("SUMMARY");

        // Sort categories for consistent output
        let mut categories: Vec<_> = category_counts.iter().collect();
        categories.sort_by_key(|&(name, _)| name);

        // Calculate column widths
        let max_category_len = categories
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0)
            .max(8); // At least "Category" width

        // Print header
        println!(
            "{:<width$} | {}",
            "Category".bold(),
            "Files".bold(),
            width = max_category_len
        );
        println!("{}", "-".repeat(max_category_len + 10));

        // Print rows
        for (category, count) in &categories {
            let file_word = if **count == 1 { "file" } else { "files" };
            println!(
                "{:<width$} | {} {}",
                category,
                count.to_string().green(),
                file_word,
                width = max_category_len
            );
        }

        // Print footer
        println!("{}", "-".repeat(max_category_len + 10));
        println!(
            "{:<width$} | {} {}",
            "Total".bold(),
            total_files.to_string().green().bold(),
            if total_files == 1 { "file" } else { "files" },
            width = max_category_len
        );
    }

    /// Prints a dry-run notice message.
    ///
    /// # Arguments
    ///
    /// * `message` - The dry-run message
    pub fn dry_run_notice(message: &str) {
        println!("{}", format!("[DRY RUN] {}", message).yellow());
    }
}
