# Scripts (Legacy)

Original bash and Python scripts that preceded the Rust CLI.

> **Note:** These scripts are preserved for reference. Use `midnite-archive` CLI instead.

## Available Scripts

| Script                          | Description                                 | Replaced by        |
|---------------------------------|---------------------------------------------|--------------------|
| `yt/channel_list_generate.sh`   | Generate video lists from YouTube channels  | `midnite generate` |
| `yt/download_video.sh`          | Download videos from list or URL            | `midnite download` |
| `yt/download_video_comments.sh` | Download comments from video list           | `midnite comments` |
| `video/rename.py`               | Rename video files (sanitize special chars) | `midnite rename`   |

## Legacy Usage

```bash
# Generate channel list
./scripts/yt/channel_list_generate.sh <youtube-channel>

# Download videos
./scripts/yt/download_video.sh <generated-list>

# Download comments
./scripts/yt/download_video_comments.sh <generated-list>

# Rename videos
python3 scripts/video/rename.py [options] <directory>
```

### rename.py Options

| Option             | Description                                               |
|--------------------|-----------------------------------------------------------|
| `-r, --recursive`  | Process subdirectories recursively                        |
| `-n, --dry-run`    | Preview changes without renaming                          |
| `-v, --verbose`    | Show each rename operation                                |
| `-e, --extensions` | File extensions to process (default: mkv mp4 description) |
