# Scripts Directory

This directory contains various scripts for managing the midnite-archive repository, primarily focused on YouTube video archiving and processing.

## Video Scripts (`video/` subdirectory)

The `video/` subdirectory contains scripts for processing and renaming video files downloaded from YouTube.

### `rename.py`

Unified Python script for renaming video files with simple or advanced modes.

**Usage:**
```bash
python3 scripts/video/rename.py [options] /path/to/directory
```

**Options:**
- `-r, --recursive`: Process subdirectories recursively
- `-n, --dry-run`: Preview changes without renaming
- `-v, --verbose`: Show each rename operation
- `-e, --extensions`: File extensions to process (default: mkv mp4 description)

**Behavior:**
- Replaces spaces, colons, slashes, quotes, parentheses, brackets, ampersands, pipes, asterisks, question marks, angle brackets, commas, and hyphens with underscores.
- Handles filename conflicts by appending a counter (e.g., _1).
- Supports dry-run and verbose output.

**Examples:**
```bash
# Basic usage (dry-run)
$ python3 scripts/video/rename.py -n ~/videos/channel_videos
########################################
#              Rename Tool             #
########################################
Directory: /home/user/videos/channel_videos
Extensions: mkv, mp4, description
Recursive: No
Dry run: Yes

Would rename: Midnite - Value Life Lyrics.mp4 -> Midnite_-_Value_Life_Lyrics.mp4
Would rename: Akae Beka [test].mkv -> Akae_Beka__test_.mkv
Dry run complete. Would rename 2 files.

# Recursive with verbose
$ python3 scripts/video/rename.py -r -v ~/videos/channel_videos
########################################
#              Rename Tool             #
########################################
Directory: /home/user/videos/channel_videos
Extensions: mkv, mp4, description
Recursive: Yes
Dry run: No

Renamed: Midnite - Value Life Lyrics.mp4 -> Midnite_-_Value_Life_Lyrics.mp4
Renamed: subdir/Akae Beka [test].mkv -> subdir/Akae_Beka__test_.mkv
Renamed 2 files.
```

## Notes

- The script uses Python for robust handling of special characters and file operations.
- Ensure the directory exists and is writable before running.
- Designed for post-download cleanup of YouTube video files.</content>
<parameter name="filePath">/home/lbgc/Documents/dev/repos/midnite-archive/scripts/README.md
