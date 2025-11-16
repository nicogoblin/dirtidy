# File Filtering and Exclusion Configuration

This document explains how to configure dirtidy to exclude or include specific files from organization.

## Overview

By default, dirtidy:
- **Excludes hidden files** (files starting with ".")
- Organizes all other regular files into category directories

You can customize this behavior using a TOML configuration file to specify:
- Which files to exclude (by filename, extension, glob pattern, or regex)
- Which files to include (whitelist rules that override exclusions)
- Whether to include hidden files

## Configuration Files

dirtidy looks for configuration files in this order:

1. **Explicit path**: `--config <path>` command-line argument
2. **Project-level**: `.dirtidyrc.toml` in the current working directory
3. **User-level**: `~/.config/dirtidy/config.toml` in your home directory
4. **Defaults**: If no config file is found, hidden files are excluded

This layered approach allows you to:
- Have a global configuration in your home directory
- Override it with project-specific settings in `.dirtidyrc.toml`
- Specify a custom path with `--config` for one-off operations

## Configuration File Format

Configuration is stored in TOML format. Here's the basic structure:

```toml
[filters]
enable_hidden_files = false  # Default: false

[filters.exclude]
filenames = ["file1", "file2"]
patterns = ["*.tmp", "node_modules/**"]
extensions = ["bak", "tmp", "log"]
regex = ["^test_.*\\.txt$"]

[filters.include]
patterns = [".important", "**/*.pdf"]
```

### Filters Section

#### `enable_hidden_files` (boolean, default: false)

Controls whether files starting with "." are processed:

```toml
[filters]
enable_hidden_files = false  # Hide dotfiles (default)
```

Set to `true` to include hidden files in organization.

### Exclude Section

Specify which files to exclude from organization:

#### `filenames` (array of strings)

Exact filename matches to exclude:

```toml
[filters.exclude]
filenames = [
    ".DS_Store",     # macOS metadata
    "Thumbs.db",     # Windows thumbnail cache
    "desktop.ini",   # Windows settings
]
```

Matching is case-sensitive and matches the filename exactly.

#### `patterns` (array of strings)

Glob patterns to exclude. Supports:
- `*.ext` - Files with a specific extension
- `dir/**` - Directory and all contents
- `path/to/file` - Exact path matching

```toml
[filters.exclude]
patterns = [
    "*.tmp",              # Temporary files
    "*.cache",            # Cache files
    ".env*",              # Environment files (.env, .env.local, etc.)
    "node_modules/**",    # Node dependencies
    "dist/**",            # Build output
]
```

#### `extensions` (array of strings)

File extensions to exclude (shorthand for `*.ext` patterns):

```toml
[filters.exclude]
extensions = [
    "tmp",     # Excludes *.tmp
    "log",     # Excludes *.log
    "bak",     # Excludes *.bak
]
```

Extensions are matched case-insensitively (e.g., `.TMP`, `.tmp`, `.Tmp` all match).

#### `regex` (array of strings)

Regular expression patterns to exclude (for advanced users):

```toml
[filters.exclude]
regex = [
    "^test_.*\\.txt$",           # test_*.txt files
    ".*\\.\\d{8}\\.bak$",        # filename.YYYYMMDD.bak
    "^\\.[^/]*$",                # Hidden files
]
```

Regex patterns match against the filename only, not the full path.

### Include Section

Whitelist patterns that override exclude rules:

#### `patterns` (array of strings)

Files matching these patterns will be included even if they match exclude rules:

```toml
[filters.include]
patterns = [
    ".importantrc",   # Always include this hidden file
    "**/*.pdf",       # Always organize PDFs even if in excluded directories
]
```

## File Inclusion Logic

Files are checked for inclusion in this order. The first match determines whether the file is included or excluded:

1. **Include patterns** - If matched, file is always included ✓
2. **Hidden file filter** - If starts with "." and `enable_hidden_files=false`, exclude ✗
3. **Exact filename** - If in `exclude.filenames`, exclude ✗
4. **Extension** - If in `exclude.extensions`, exclude ✗ (case-insensitive)
5. **Glob patterns** - If matches `exclude.patterns`, exclude ✗
6. **Regex patterns** - If matches `exclude.regex`, exclude ✗
7. **Default** - Include ✓

## Usage Examples

### Basic Usage (No Configuration)

Hides hidden files by default:

```bash
dirtidy /path/to/directory
dirtidy /path/to/directory --dry-run
```

### Using Project Configuration

Create `.dirtidyrc.toml` in your project:

```bash
# Copy example to project
cp .dirtidyrc.toml.example .dirtidyrc.toml

# Edit as needed
nano .dirtidyrc.toml

# Run - will automatically use .dirtidyrc.toml
dirtidy /path/to/directory
```

### Using Custom Configuration Path

```bash
dirtidy /path/to/directory --config /path/to/custom-config.toml
dirtidy /path/to/directory --dry-run --config ~/.config/dirtidy/config.toml
```

### Dry-Run Before Organizing

Always test with dry-run first to see what will be excluded:

```bash
dirtidy /path/to/directory --dry-run --config .dirtidyrc.toml
```

Output shows which files would be organized and which are filtered out.

## Common Configuration Examples

### Web Development Project

```toml
[filters]
enable_hidden_files = false

[filters.exclude]
filenames = [".DS_Store", "Thumbs.db"]
patterns = [
    ".env*",
    "node_modules/**",
    "dist/**",
    "build/**",
]
extensions = ["tmp", "log"]
```

### Python Project

```toml
[filters]
enable_hidden_files = false

[filters.exclude]
filenames = [".DS_Store", "Thumbs.db"]
patterns = [
    ".env*",
    ".venv/**",
    "venv/**",
    "__pycache__/**",
    "*.egg-info/**",
]
extensions = ["pyc", "pyo", "pyd", "tmp", "log"]
```

### Rust Project

```toml
[filters]
enable_hidden_files = false

[filters.exclude]
patterns = [
    "target/**",
    ".env*",
]
extensions = ["bak", "tmp", "log"]
```

### Downloads Folder (Include Everything)

```toml
[filters]
enable_hidden_files = true  # Include hidden files

[filters.exclude]
# Empty - organize everything including hidden files
```

### Strict Filtering (Whitelist Approach)

```toml
[filters]
enable_hidden_files = false

[filters.exclude]
patterns = ["**/*"]  # Exclude everything

[filters.include]
# Only include specific file types
patterns = [
    "*.pdf",
    "*.docx",
    "*.xlsx",
]
```

## Tips and Best Practices

### 1. Use Dry-Run First

Always test your configuration before running the actual organization:

```bash
dirtidy /path/to/directory --dry-run --config .dirtidyrc.toml
```

### 2. Start Minimal

Begin with minimal exclusions and add more as needed:

```toml
[filters.exclude]
filenames = [".DS_Store", "Thumbs.db"]
```

### 3. Use Glob Patterns for Directories

Exclude entire directories with `**`:

```toml
[filters.exclude]
patterns = ["node_modules/**", "dist/**"]
```

### 4. Case-Insensitive Extensions

Extensions are matched case-insensitively, so use lowercase:

```toml
[filters.exclude]
extensions = ["tmp"]  # Matches .tmp, .TMP, .Tmp, etc.
```

### 5. Test Regex Before Using

Test regex patterns carefully. They match against filenames only:

```toml
[filters.exclude]
regex = [
    "^test_.*\\.txt$",  # Matches: test_file.txt
                         # Doesn't match: dir/test_file.txt
]
```

### 6. Whitelist Important Files

Use include rules to protect important files in normally-excluded directories:

```toml
[filters.exclude]
patterns = ["**/*.tmp"]

[filters.include]
patterns = [
    "important.tmp",  # This file won't be excluded
]
```

### 7. Location Matters

- **~/.config/dirtidy/config.toml**: Global defaults for all projects
- **.dirtidyrc.toml**: Project-specific overrides
- **--config**: One-off custom configurations

## Implementation Details

### Filter Compilation

Filters are compiled once at startup for efficiency:
- Regex patterns are pre-compiled
- Extension matching uses HashSet (O(1) lookup)
- Glob patterns are simple substring matching

### Performance

Filtering has minimal performance impact:
- Simple extensions are O(1)
- Glob patterns are O(n) where n is number of patterns
- Regex matching is O(n) where n is number of regex patterns

For typical usage with <1000 files and <100 rules, filtering is negligible.

## Configuration Schema

Here's the complete TOML schema:

```toml
[filters]
enable_hidden_files = boolean  # Optional, default: false

[filters.exclude]
filenames = [string]           # Optional, default: []
patterns = [string]            # Optional, default: []
extensions = [string]          # Optional, default: []
regex = [string]               # Optional, default: []

[filters.include]
patterns = [string]            # Optional, default: []
```

All arrays are optional. Omitted sections use defaults.
