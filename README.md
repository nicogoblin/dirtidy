# 📁 dirtidy

**A fast, safe, and reversible directory organization utility**

dirtidy automatically organizes files in a directory into category-based subdirectories (images, documents, audio, video, etc.) based on their file type. It's perfect for cleaning up Downloads folders, organizing file collections, or managing cluttered directories.

## ✨ Features

- **Automatic File Categorization**: Detects file types and organizes them into 10 categories
- **Fully Reversible**: Undo any organization with a single command
- **Dry-Run Mode**: Preview changes before making them
- **Customizable Filtering**: Exclude specific files via configuration
- **Hidden Files Protected**: Hidden files (dotfiles) are excluded by default
- **Fast**: Written in Rust for optimal performance

## 🚀 Quick Start

### Installation

Build from source:

```bash
git clone https://github.com/nicolabonsi/dirtidy.git
cd dirtidy
cargo build --release
```

The compiled binary will be at `target/release/dirtidy`.

### Basic Usage

Organize a directory:

```bash
dirtidy ~/Downloads
```

Preview changes before organizing (dry-run):

```bash
dirtidy ~/Downloads --dry-run
```

Undo the last organization:

```bash
dirtidy ~/Downloads --undo
```

## 📖 Complete CLI Reference

### Command Syntax

```
dirtidy <directory_path> [OPTIONS]
```

### Options

| Option | Short | Description | Example |
|--------|-------|-------------|---------|
| `--dry-run` | `-n` | Preview what would be organized without making changes | `dirtidy ~/Downloads --dry-run` |
| `--undo` | | Revert the last organization operation | `dirtidy ~/Downloads --undo` |
| `--config <path>` | | Use a specific configuration file | `dirtidy ~/Downloads --config .dirtidyrc.toml` |
| `--help` | `-h` | Show help message and available options | `dirtidy --help` |
| `--version` | `-V` | Show version information | `dirtidy --version` |

## 📂 File Categories

dirtidy organizes files into these 10 categories:

| Category | Directory | File Types |
|----------|-----------|-----------|
| **Images** | `images/` | PNG, JPG, GIF, WebP, SVG, BMP, TIFF, HEIC |
| **Audio** | `audio/` | MP3, WAV, OGG, FLAC, AAC, M4A, WMA, WebM |
| **Video** | `videos/` | MP4, MKV, AVI, MOV, FLV, WMV, 3GP |
| **Documents** | `documents/` | PDF, TXT, HTML, MD, DOCX, DOC, RTF, ODT |
| **Archives** | `archives/` | ZIP, RAR, 7Z, TAR, GZIP, BZIP2, XZ |
| **Code** | `code/` | PY, JS, TS, JAVA, C, C++, RS, GO, JSON, XML, YAML, TOML |
| **Spreadsheets** | `spreadsheets/` | CSV, XLS, XLSX, ODS |
| **Presentations** | `presentations/` | PPT, PPTX, ODP |
| **Fonts** | `fonts/` | TTF, OTF, WOFF, WOFF2 |
| **Other** | `other/` | Files that don't match any category |

## ⚙️ Configuration

dirtidy can be customized using a TOML configuration file to exclude or include specific files from organization.

### Configuration File Locations

dirtidy looks for configuration in this order (first found wins):

1. **CLI argument**: `--config /path/to/config.toml`
2. **Current directory**: `.dirtidyrc.toml`
3. **Home directory**: `~/.config/dirtidy/config.toml`
4. **Defaults**: If no config found, hidden files are excluded

### Basic Configuration Example

Create `.dirtidyrc.toml` in your project directory:

```toml
[filters]
# Whether to organize hidden files (those starting with ".")
enable_hidden_files = false

[filters.exclude]
# Exclude specific filenames
filenames = [".DS_Store", "Thumbs.db"]

# Exclude files matching glob patterns
patterns = ["*.tmp", "node_modules/**"]

# Exclude files by extension (case-insensitive)
extensions = ["bak", "log"]

# Exclude files matching regex patterns (advanced)
regex = []

[filters.include]
# Include (whitelist) specific files even if they match exclude rules
patterns = [".importantrc"]
```

For comprehensive configuration documentation, advanced examples, and detailed option descriptions, see [file filtering](FILE_FILTERING.md).

## 🔄 How It Works

### Organization Process

1. **Scan**: Reads all files in the directory
2. **Filter**: Applies exclusion rules from configuration
3. **Detect**: Identifies file types using content analysis
4. **Categorize**: Maps file types to categories
5. **Move**: Creates category directories and moves files
6. **Record**: Saves operation history for undo

### File Type Detection

dirtidy uses intelligent file type detection:
- **Primary method**: Content-based MIME type detection (looks at file contents, not just extension)
- **Fallback**: File extension matching
- **Default**: Places unrecognized files in the `other/` category

This means files are correctly categorized even if renamed or missing extensions.

### Operation History

When you organize files, dirtidy creates a `.dirtidy_history.json` file in the target directory. This file records all operations and enables full undo capability.

To undo an organization:

```bash
dirtidy ~/Downloads --undo
```

## 📋 Workflow Examples

### Scenario 1: Clean Up Downloads Folder

```bash
# Step 1: Preview what will happen
cd ~/Downloads
dirtidy . --dry-run

# Step 2: Organize for real
dirtidy .

# Step 3: If you want to revert
dirtidy . --undo
```

### Scenario 2: Organize with Custom Rules

```bash
# Create a configuration file
cat > .dirtidyrc.toml << 'EOF'
[filters]
enable_hidden_files = false

[filters.exclude]
extensions = ["bak", "tmp"]
patterns = ["node_modules/**"]
EOF

# Preview with custom config
dirtidy . --dry-run --config .dirtidyrc.toml

# Organize with custom config
dirtidy . --config .dirtidyrc.toml
```

### Scenario 3: Global Configuration

```bash
# Create a global config in your home directory
mkdir -p ~/.config/dirtidy
cp .dirtidyrc.toml.example ~/.config/dirtidy/config.toml

# Edit to suit your preferences
nano ~/.config/dirtidy/config.toml

# Now dirtidy will use this config in all directories
dirtidy ~/Downloads
dirtidy ~/Documents
dirtidy ~/Desktop
```

## 🆘 Common Questions & Troubleshooting

### Q: Will dirtidy delete my files?

**A:** No. dirtidy only moves files into subdirectories. All files are preserved, and you can undo any operation.

### Q: How do I undo an organization?

**A:** Use the `--undo` flag:

```bash
dirtidy ~/Downloads --undo
```

dirtidy keeps a history file (`.dirtidy_history.json`) to enable complete undo functionality.

### Q: Can I preview changes before organizing?

**A:** Yes! Always use `--dry-run` first:

```bash
dirtidy ~/Downloads --dry-run
```

This shows exactly what would be organized without making any changes.

### Q: How do I exclude certain files?

**A:** Create a `.dirtidyrc.toml` configuration file:

```toml
[filters.exclude]
filenames = [".DS_Store"]
extensions = ["tmp", "bak"]
patterns = ["*.cache"]
```

See the [Configuration](#-configuration) section above for full details.

### Q: Can I use dirtidy on my entire home directory?

**A:** Yes, but use `--dry-run` first to see what would be organized:

```bash
dirtidy ~/ --dry-run
```

Consider creating a configuration file to exclude directories you don't want to organize:

```toml
[filters.exclude]
patterns = ["Library/**", ".cache/**", ".local/**"]
```

### Q: What if the history file is missing?

**A:** If you've deleted `.dirtidy_history.json`, undo won't work for that directory. However, you can manually move files back from the category directories to the original location.

### Q: Are hidden files (dotfiles) organized?

**A:** By default, no. Hidden files are excluded. To include them, set `enable_hidden_files = true` in your configuration:

```toml
[filters]
enable_hidden_files = true
```

## 🛡️ Safety Features

dirtidy is designed with safety in mind:

- **No Deletion**: Files are only moved, never deleted
- **Fully Reversible**: Complete undo capability via `--undo`
- **Dry-Run Preview**: Always test with `--dry-run` first
- **Safe Defaults**: Hidden files excluded, explicit configuration required
- **Conflict Handling**: If a file exists in the destination, a timestamp backup is created
- **Operation Logging**: All operations are recorded for transparency

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

---

## 📚 Additional Documentation

- **[File filtering doc](FILE_FILTERING.md)** - Complete configuration file reference with advanced examples
- **[.dirtidyrc.toml.example](.dirtidyrc.toml.example)** - Well-commented configuration example
- **Source Code** - See `src/` directory for implementation details

---
