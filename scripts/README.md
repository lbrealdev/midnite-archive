# Scripts

## Available Scripts

| Script                   | Description                                          |
|--------------------------|------------------------------------------------------|
| `generate_archive.sh`    | Generate .archive files from existing video downloads |

## Usage

```bash
# Generate archive from current directory
./scripts/generate_archive.sh

# Generate archive from specific directory
./scripts/generate_archive.sh /path/to/videos
```

---

## Legacy Scripts

Original bash and Python scripts that preceded the Rust CLI.

> **Note:** These scripts are preserved for reference. Use `midnite-archive` CLI instead.

| Script                                    | Description                                 | Replaced by        |
|-------------------------------------------|---------------------------------------------|--------------------|
| `legacy/yt/channel_list_generate.sh`      | Generate video lists from YouTube channels  | `midnite generate` |
| `legacy/yt/download_video.sh`             | Download videos from list or URL            | `midnite download` |
| `legacy/yt/download_video_comments.sh`    | Download comments from video list           | `midnite comments` |
| `legacy/video/rename.py`                  | Rename video files (sanitize special chars) | `midnite rename`   |

### Legacy Usage

```bash
# Generate channel list
./scripts/legacy/yt/channel_list_generate.sh <youtube-channel>

# Download videos
./scripts/legacy/yt/download_video.sh <generated-list>

# Download comments
./scripts/legacy/yt/download_video_comments.sh <generated-list>

# Rename videos
python3 scripts/legacy/video/rename.py [options] <directory>
```

### rename.py Options

| Option             | Description                                               |
|--------------------|-----------------------------------------------------------|
| `-r, --recursive`  | Process subdirectories recursively                        |
| `-n, --dry-run`    | Preview changes without renaming                          |
| `-v, --verbose`    | Show each rename operation                                |
| `-e, --extensions` | File extensions to process (default: mkv mp4 description) |
