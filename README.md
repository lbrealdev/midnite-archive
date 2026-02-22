# midnite-archive

Rust CLI for archiving YouTube content of the reggae band **Midnite/Akae Beka** from our beloved teacher and guide **Vaughn Benjamin**.

Uses yt-dlp with External JavaScript (EJS) support for reliable downloads despite YouTube's frequent changes.

## Installation

### Requirements

- [Rust](https://rustup.rs/) (latest stable)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [deno](https://deno.land/)

### Build

```shell
cargo build --release
```

The binary will be available at `./target/release/midnite-archive`.

## Usage

```shell
midnite-archive <COMMAND>
```

### Commands

| Command                | Description                                 |
|------------------------|---------------------------------------------|
| `generate <channel>`   | Generate video list from YouTube channel    |
| `download <input>`     | Download videos from list file or URL       |
| `comments <list-file>` | Download comments from video list           |
| `rename <directory>`   | Rename video files (sanitize special chars) |

### Examples

```shell
# Generate channel list
midnite-archive generate @severo12

# Download from list file
midnite-archive download severo12/lists/severo12-list-url-*.txt

# Download single video
midnite-archive download "https://www.youtube.com/watch?v=VIDEO_ID"

# Download comments
midnite-archive comments severo12/lists/severo12-list-url-*.txt

# Preview renames (dry-run)
midnite-archive rename -d severo12/videos

# Apply renames
midnite-archive rename severo12/videos
```

### Rename Options

| Option             | Description                                               |
|--------------------|-----------------------------------------------------------|
| `-d, --dry-run`    | Preview changes without renaming                          |
| `-r, --recursive`  | Process subdirectories recursively                        |
| `-v, --verbose`    | Show each rename operation                                |
| `-e, --extensions` | File extensions to process (default: mkv mp4 description) |

## Development

```shell
just build     # Build debug binary
just release   # Build optimized binary
just check     # Check compilation
just test      # Run tests
just lint      # Run clippy
just fmt       # Format code
just ci-scan   # Security audit workflows
just ci-pin    # Pin GitHub Actions to SHAs
```

## Documentation

- [Requirements](docs/requirements.md) - Setup and installation guide
- [Troubleshooting](docs/troubleshooting.md) - Common issues and solutions
- [Processing Comments](docs/processing-comments.md) - YouTube comment data extraction
- [tmux](docs/tmux.md) - Running long downloads in background
- [Channels](channels/README.md) - YouTube channel curation guide

## Legacy Scripts

Original bash and Python scripts are preserved in [`scripts/`](scripts/README.md).
