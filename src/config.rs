//! File filtering and exclusion configuration.
//!
//! This module provides support for loading and applying file filtering rules
//! via TOML configuration files. It supports multiple filtering strategies:
//! - Exact filename matching
//! - Glob pattern matching
//! - File extension matching
//! - Regex pattern matching
//! - Include (whitelist) rules that override exclude rules
//!
//! # Configuration File Format
//!
//! Configuration is stored in TOML format with the following structure:
//!
//! ```toml
//! [filters]
//! enable_hidden_files = false
//!
//! [filters.exclude]
//! filenames = [".DS_Store", "Thumbs.db"]
//! patterns = ["*.tmp", "node_modules/**"]
//! extensions = ["bak", "tmp"]
//! regex = []
//!
//! [filters.include]
//! patterns = []
//! ```

use glob::Pattern;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Errors that can occur during configuration loading and filtering.
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Configuration file not found at the specified path.
    ConfigNotFound(PathBuf),
    /// Invalid TOML syntax or structure.
    ConfigInvalid(String),
    /// Invalid glob pattern provided.
    InvalidGlobPattern(String),
    /// Invalid regex pattern provided with the actual error reason.
    InvalidRegexPattern {
        /// The regex pattern that failed to compile.
        pattern: String,
        /// The reason why the pattern is invalid.
        reason: String,
    },
    /// IO error while reading configuration.
    IoError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ConfigNotFound(path) => {
                write!(f, "Configuration file not found: {}", path.display())
            }
            ConfigError::ConfigInvalid(msg) => write!(f, "Invalid configuration: {}", msg),
            ConfigError::InvalidGlobPattern(pattern) => {
                write!(
                    f,
                    "Invalid glob pattern '{}': expected *.ext or dir/**",
                    pattern
                )
            }
            ConfigError::InvalidRegexPattern { pattern, reason } => {
                write!(f, "Invalid regex pattern '{}': {}", pattern, reason)
            }
            ConfigError::IoError(msg) => write!(f, "IO error reading configuration: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration for file filtering and exclusion rules.
///
/// This struct is deserialized from TOML configuration files and contains
/// all rules for which files should be filtered (excluded) from organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub filters: FilterRules,
}

/// Root-level filter rules configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRules {
    /// Whether to include hidden files (starting with "."). Defaults to false.
    #[serde(default = "default_enable_hidden_files")]
    pub enable_hidden_files: bool,

    /// Rules for excluding files.
    #[serde(default)]
    pub exclude: ExcludeRules,

    /// Rules for including files (whitelist, overrides exclude rules).
    #[serde(default)]
    pub include: IncludeRules,
}

/// Helper function for default value of `enable_hidden_files`.
fn default_enable_hidden_files() -> bool {
    false
}

/// Rules for excluding files from organization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExcludeRules {
    /// Exact filenames to exclude (e.g., ".DS_Store", "Thumbs.db").
    #[serde(default)]
    pub filenames: Vec<String>,

    /// Glob patterns to exclude (e.g., "*.tmp", "node_modules/**").
    #[serde(default)]
    pub patterns: Vec<String>,

    /// File extensions to exclude (e.g., "bak", "tmp", "log").
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Regex patterns to exclude (for advanced users).
    #[serde(default)]
    pub regex: Vec<String>,
}

/// Rules for including files, overriding exclude rules (whitelist).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncludeRules {
    /// Glob patterns that override exclude rules.
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl FilterConfig {
    /// Load configuration from a file, with fallback to defaults.
    ///
    /// Attempts to load configuration in the following order:
    /// 1. If `config_path` is provided, load from that file
    /// 2. Look for `.dirtidyrc.toml` in the current directory
    /// 3. Look for `~/.config/dirtidy/config.toml` in home directory
    /// 4. Fall back to default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if a configuration file is explicitly provided but cannot be read.
    pub fn load(config_path: Option<&Path>) -> Result<Self, ConfigError> {
        // If explicitly specified, load from that path
        if let Some(path) = config_path {
            return Self::load_from_file(path);
        }

        // Try current directory
        let local_config = PathBuf::from(".dirtidyrc.toml");
        if local_config.exists() {
            return Self::load_from_file(&local_config);
        }

        // Try home directory
        if let Ok(home) = std::env::var("HOME") {
            let home_config = PathBuf::from(home)
                .join(".config")
                .join("dirtidy")
                .join("config.toml");
            if home_config.exists() {
                return Self::load_from_file(&home_config);
            }
        }

        // Fall back to defaults
        Ok(Self::default())
    }

    /// Load configuration from a specific file.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ConfigNotFound` if file does not exist.
    /// Returns `ConfigError::ConfigInvalid` if TOML parsing fails.
    /// Returns `ConfigError::IoError` if file cannot be read.
    fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::ConfigNotFound(path.to_path_buf()));
        }

        let content = fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        toml::from_str(&content).map_err(|e| ConfigError::ConfigInvalid(e.to_string()))
    }

    /// Compile configuration into optimized filter structures for matching.
    ///
    /// # Errors
    ///
    /// Returns an error if any regex or glob patterns are invalid.
    pub fn compile(self) -> Result<CompiledFilters, ConfigError> {
        CompiledFilters::new(self.filters)
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            filters: FilterRules {
                enable_hidden_files: false,
                exclude: ExcludeRules::default(),
                include: IncludeRules::default(),
            },
        }
    }
}

/// Compiled, optimized filter structures for efficient file matching.
///
/// This struct pre-processes all filter rules (glob patterns, regex patterns, etc.)
/// into efficient data structures so that matching is O(1) or O(n) where n is the
/// number of rules, rather than reparsing patterns on each file.
pub struct CompiledFilters {
    enable_hidden_files: bool,
    exclude_filenames: HashSet<String>,
    exclude_extensions: HashSet<String>,
    exclude_patterns: Vec<Pattern>,
    exclude_regexes: Vec<Regex>,
    include_patterns: Vec<Pattern>,
}

impl CompiledFilters {
    /// Create compiled filters from filter rules.
    ///
    /// # Errors
    ///
    /// Returns an error if any glob or regex patterns are invalid.
    fn new(rules: FilterRules) -> Result<Self, ConfigError> {
        // Pre-compile all glob patterns and validate them
        let exclude_patterns = rules
            .exclude
            .patterns
            .iter()
            .map(|pattern| {
                Pattern::new(pattern).map_err(|_| ConfigError::InvalidGlobPattern(pattern.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let include_patterns = rules
            .include
            .patterns
            .iter()
            .map(|pattern| {
                Pattern::new(pattern).map_err(|_| ConfigError::InvalidGlobPattern(pattern.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Pre-compile all regex patterns and validate them
        let exclude_regexes = rules
            .exclude
            .regex
            .iter()
            .map(|pattern| {
                Regex::new(pattern).map_err(|e| ConfigError::InvalidRegexPattern {
                    pattern: pattern.clone(),
                    reason: e.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            enable_hidden_files: rules.enable_hidden_files,
            exclude_filenames: rules.exclude.filenames.into_iter().collect(),
            exclude_extensions: rules
                .exclude
                .extensions
                .iter()
                .map(|ext| ext.to_lowercase())
                .collect(),
            exclude_patterns,
            exclude_regexes,
            include_patterns,
        })
    }

    /// Check if a file should be included in organization (not excluded).
    ///
    /// Checks are performed in this order, with early termination:
    /// 1. Include patterns (whitelist) - if matched, always include
    /// 2. Hidden file filter - if hidden and disabled, exclude
    /// 3. Exact filename match - if matched, exclude
    /// 4. File extension match - if matched, exclude
    /// 5. Glob pattern match - if matched, exclude
    /// 6. Regex pattern match - if matched, exclude
    /// 7. Default: include
    pub fn should_include(&self, file_path: &Path) -> bool {
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        // 1. Include rules have priority (whitelist override)
        if self.matches_include_patterns(file_path) {
            return true;
        }

        // 2. Check hidden file filter
        if !self.enable_hidden_files && file_name.starts_with('.') {
            return false;
        }

        // 3. Check exact filename match
        if self.exclude_filenames.contains(file_name.as_ref()) {
            return false;
        }

        // 4. Check extension match
        if let Some(ext) = file_path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if self.exclude_extensions.contains(&ext_lower) {
                return false;
            }
        }

        // 5. Check glob patterns
        if self.matches_exclude_patterns(file_path) {
            return false;
        }

        // 6. Check regex patterns
        if self.matches_exclude_regex(&file_name) {
            return false;
        }

        // 7. Include by default
        true
    }

    /// Check if file matches any include (whitelist) patterns.
    fn matches_include_patterns(&self, file_path: &Path) -> bool {
        self.include_patterns
            .iter()
            .any(|pattern| pattern.matches_path(file_path))
    }

    /// Check if file matches any exclude glob patterns.
    fn matches_exclude_patterns(&self, file_path: &Path) -> bool {
        self.exclude_patterns
            .iter()
            .any(|pattern| pattern.matches_path(file_path))
    }

    /// Check if file matches any exclude regex patterns.
    fn matches_exclude_regex(&self, file_name: &str) -> bool {
        self.exclude_regexes
            .iter()
            .any(|regex| regex.is_match(file_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_hides_hidden_files() {
        let config = FilterConfig::default();
        assert!(!config.filters.enable_hidden_files);
    }

    #[test]
    fn test_compile_valid_config() {
        let config = FilterConfig::default();
        let compiled = config.compile();
        assert!(compiled.is_ok());
    }

    #[test]
    fn test_hidden_file_excluded_by_default() {
        let config = FilterConfig::default();
        let compiled = config.compile().unwrap();

        assert!(!compiled.should_include(Path::new(".DS_Store")));
        assert!(!compiled.should_include(Path::new(".gitignore")));
    }

    #[test]
    fn test_hidden_file_included_when_enabled() {
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules::default(),
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        assert!(compiled.should_include(Path::new(".DS_Store")));
    }

    #[test]
    fn test_exclude_exact_filename() {
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    filenames: vec!["Thumbs.db".to_string(), ".DS_Store".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        assert!(!compiled.should_include(Path::new("Thumbs.db")));
        assert!(compiled.should_include(Path::new("image.jpg")));
    }

    #[test]
    fn test_exclude_extensions() {
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    extensions: vec!["bak".to_string(), "tmp".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        assert!(!compiled.should_include(Path::new("file.bak")));
        assert!(!compiled.should_include(Path::new("file.tmp")));
        assert!(!compiled.should_include(Path::new("file.BAK"))); // Case-insensitive
        assert!(compiled.should_include(Path::new("file.txt")));
    }

    #[test]
    fn test_exclude_glob_patterns() {
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    patterns: vec!["*.cache".to_string(), "node_modules/**".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        assert!(!compiled.should_include(Path::new("file.cache")));
        assert!(!compiled.should_include(Path::new("node_modules/package.json")));
        assert!(compiled.should_include(Path::new("file.txt")));
    }

    #[test]
    fn test_include_overrides_exclude() {
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: false,
                exclude: ExcludeRules {
                    ..Default::default()
                },
                include: IncludeRules {
                    patterns: vec![".important".to_string()],
                },
            },
        };
        let compiled = config.compile().unwrap();

        // Normally hidden files are excluded, but .important is in include list
        assert!(compiled.should_include(Path::new(".important")));
        assert!(!compiled.should_include(Path::new(".other")));
    }

    #[test]
    fn test_exclude_regex() {
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    regex: vec![r"^test_.*\.txt$".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        assert!(!compiled.should_include(Path::new("test_file.txt")));
        assert!(!compiled.should_include(Path::new("test_another.txt")));
        assert!(compiled.should_include(Path::new("file.txt")));
    }

    #[test]
    fn test_invalid_regex_returns_error() {
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    regex: vec!["[invalid(".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };

        let result = config.compile();
        assert!(result.is_err());
    }

    #[test]
    fn test_glob_pattern_directory_boundary_semantics() {
        // This test verifies that glob patterns respect directory boundaries
        // Previously, substring matching would incorrectly match "my_logs/file.txt"
        // when pattern was "**/logs/**"
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    patterns: vec!["**/logs/**".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        // Correct matches - logs at any level
        assert!(!compiled.should_include(Path::new("logs/file.txt")));
        assert!(!compiled.should_include(Path::new("app/logs/file.txt")));

        // These should NOT match with proper glob semantics
        assert!(compiled.should_include(Path::new("my_logs/file.txt")));
        assert!(compiled.should_include(Path::new("app/my_logs/file.txt")));
    }

    #[test]
    fn test_glob_pattern_complex_nested_directories() {
        // Test that glob handles multiple ** patterns correctly
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    patterns: vec!["**/src/**/test_*.rs".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        // Should match test files at any depth under any src/
        assert!(!compiled.should_include(Path::new("src/test_main.rs")));
        assert!(!compiled.should_include(Path::new("src/utils/test_helpers.rs")));
        assert!(!compiled.should_include(Path::new("project/src/deep/nested/path/test_utils.rs")));

        // Should NOT match files that don't match the pattern
        assert!(compiled.should_include(Path::new("src/main.rs")));
        assert!(compiled.should_include(Path::new("src/utils/helpers.rs")));
        assert!(compiled.should_include(Path::new("test_main.rs"))); // Not under src/
    }

    #[test]
    fn test_glob_pattern_multiple_patterns() {
        // Test that multiple glob patterns can be specified and work together
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    patterns: vec![
                        "**/logs/**".to_string(),
                        "**/cache/**".to_string(),
                        "**/tmp/**".to_string(),
                    ],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        // Files in these directories should be excluded
        assert!(!compiled.should_include(Path::new("logs/app.log")));
        assert!(!compiled.should_include(Path::new("app/logs/debug.log")));
        assert!(!compiled.should_include(Path::new("cache/data.bin")));
        assert!(!compiled.should_include(Path::new("tmp/tempfile.txt")));

        // Other files should be included
        assert!(compiled.should_include(Path::new("src/main.rs")));
        assert!(compiled.should_include(Path::new("data/app.log")));
    }

    #[test]
    fn test_glob_pattern_character_class() {
        // Test that glob handles character classes
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    patterns: vec!["[0-9]*.tmp".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        // Files starting with digits should be excluded
        assert!(!compiled.should_include(Path::new("1cache.tmp")));
        assert!(!compiled.should_include(Path::new("99data.tmp")));

        // Files not starting with digits should be included
        assert!(compiled.should_include(Path::new("cache.tmp")));
        assert!(compiled.should_include(Path::new("a1cache.tmp")));
    }

    #[test]
    fn test_glob_pattern_recursive_directory_matching() {
        // Test that ** properly matches across multiple levels
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    patterns: vec!["**/node_modules/**".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        // Should match node_modules at any depth
        assert!(!compiled.should_include(Path::new("node_modules/pkg/index.js")));
        assert!(!compiled.should_include(Path::new("src/node_modules/pkg/index.js")));
        assert!(!compiled.should_include(Path::new("a/b/c/node_modules/pkg/index.js")));

        // Should not match files without node_modules in path
        assert!(compiled.should_include(Path::new("src/pkg/index.js")));
        assert!(compiled.should_include(Path::new("my_node_modules/pkg/index.js")));
    }

    #[test]
    fn test_glob_pattern_single_char_wildcard() {
        // Test that glob handles ? (single character wildcard)
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    patterns: vec!["file?.txt".to_string()],
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };
        let compiled = config.compile().unwrap();

        // Single character in that position should be excluded
        assert!(!compiled.should_include(Path::new("file1.txt")));
        assert!(!compiled.should_include(Path::new("filea.txt")));

        // Different pattern should be included
        assert!(compiled.should_include(Path::new("file.txt")));
        assert!(compiled.should_include(Path::new("file12.txt")));
    }

    #[test]
    fn test_invalid_glob_pattern_returns_error() {
        // Test that invalid glob patterns are caught during compilation
        let config = FilterConfig {
            filters: FilterRules {
                enable_hidden_files: true,
                exclude: ExcludeRules {
                    patterns: vec!["[invalid".to_string()], // Unclosed bracket
                    ..Default::default()
                },
                include: IncludeRules::default(),
            },
        };

        let result = config.compile();
        assert!(result.is_err());
    }
}
