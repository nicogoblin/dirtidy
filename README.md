# dirtidy

A command-line tool that organizes files in a directory into category-based subdirectories. It detects file types by content (not just extension), supports dry-run preview, and records every operation so you can undo it.

## Installation

Build from source:

```bash
git clone https://github.com/nicolabonsi/dirtidy.git
cd dirtidy
cargo build --release
```

The binary will be at `target/release/dirtidy`.

## Usage

Organize a directory:

```bash
dirtidy ~/Downloads
```

Preview changes without applying them:

```bash
dirtidy ~/Downloads --dry-run
```

Undo the last organization:

```bash
dirtidy ~/Downloads --undo
```

Use a custom configuration file:

```bash
dirtidy ~/Downloads --config .dirtidyrc.toml
```

## Options

| Option | Short | Description |
|--------|-------|-------------|
| `--dry-run` | `-n` | Show what would be moved without making changes |
| `--undo` | | Revert the last organization in the given directory |
| `--config <path>` | | Use a specific configuration file |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |

`--undo` and `--dry-run` are mutually exclusive.

## File Categories

| Category | Directory | Extensions |
|----------|-----------|------------|
| Images | `images/` | png, jpg, gif, webp, svg, bmp, tiff, heic |
| Audio | `audio/` | mp3, wav, ogg, flac, aac, m4a, wma, webm |
| Video | `videos/` | mp4, mkv, avi, mov, flv, wmv, 3gp |
| Documents | `documents/` | pdf, txt, html, md, docx, doc, rtf, odt |
| Archives | `archives/` | zip, rar, 7z, tar, gz, bz2, xz |
| Code | `code/` | py, js, ts, java, c, cpp, rs, go, json, xml, yaml, toml |
| Spreadsheets | `spreadsheets/` | csv, xls, xlsx, ods |
| Presentations | `presentations/` | ppt, pptx, odp |
| Fonts | `fonts/` | ttf, otf, woff, woff2 |
| Other | `other/` | anything not matched above |

## Configuration

dirtidy looks for a configuration file in this order:

1. Path passed via `--config`
2. `.dirtidyrc.toml` in the current directory
3. `~/.config/dirtidy/config.toml`
4. Built-in defaults (hidden files excluded)

Example configuration:

```toml
[filters]
enable_hidden_files = false

[filters.exclude]
filenames = [".DS_Store", "Thumbs.db"]
extensions = ["bak", "log"]
patterns = ["*.tmp", "node_modules/**"]
regex = []

[filters.include]
patterns = [".importantrc"]
```

Include patterns take priority over exclude rules. Hidden files are excluded by default regardless of other rules unless `enable_hidden_files = true`.

See [FILE_FILTERING.md](FILE_FILTERING.md) for full configuration documentation and [.dirtidyrc.toml.example](.dirtidyrc.toml.example) for an annotated example.

## How it works

When you run dirtidy on a directory, it reads each file, detects its type by inspecting the file contents (first 8 KB), and moves it into the appropriate subdirectory. Extension matching is used as a fallback when content detection is inconclusive.

Every operation is recorded in a `.dirtidy_history.json` file inside the target directory. Running with `--undo` reads that file and reverses each move. If a file already exists at the original location, it is backed up with a timestamp suffix before the restored file is moved into place. The history file is deleted once all operations are successfully reversed.

Files are never deleted — only moved.

## Contributing

Issues and pull requests are welcome.
