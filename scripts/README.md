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

**Behavior:**
- Recursively finds all `.mkv` and `.description` files in the given directory.
- Replaces spaces with underscores in filenames.
- Outputs the directory path, file count, and progress.
- Exits with error if no directory is provided or if the directory doesn't exist.

**Example:**
```bash
$ ./scripts/video/rename.sh ~/videos/channel_videos
Input directory: /home/user/videos/channel_videos
Files found in the directory: 15
Renaming files...
Done!
```

### `special_rename.sh`

Renames `.mkv` files in a specified directory (non-recursive) by replacing spaces, colons, slashes, and other special characters with underscores.

**Usage:**
```bash
./scripts/video/special_rename.sh /path/to/directory
```

**Behavior:**
- Only processes `.mkv` files in the top-level directory (not subdirectories).
- Uses sed to replace spaces (` `), colons (`:`), and other characters with underscores.
- Outputs a header message before processing.
- Exits with error if no directory is provided.

**Example:**
```bash
$ ./scripts/video/special_rename.sh ~/videos/channel_videos

########################################
#            Rename Script             #
########################################

$ ls *.mkv
Video_Name_With_Spaces.mkv  ->  Video_Name_With_Spaces.mkv (already renamed)
Another:Video/Name.mkv      ->  Another_Video_Name.mkv
```

### `special_rename.py`

Improved Python version for renaming video files (.mkv and .mp4 by default) by replacing special characters with underscores.

**Usage:**
```bash
python3 scripts/video/special_rename.py [options] /path/to/directory
```

**Options:**
- `-r, --recursive`: Process subdirectories recursively
- `-n, --dry-run`: Preview changes without renaming
- `-v, --verbose`: Show each rename operation
- `-e, --extensions`: Specify file extensions (default: mkv mp4)

**Behavior:**
- Replaces spaces, colons, slashes, quotes, parentheses, brackets, ampersands, pipes, asterisks, question marks, and angle brackets with underscores.
- Preserves hyphens for readability.
- Handles filename conflicts by appending a counter (e.g., _1).
- Supports recursive processing and dry-run mode.

**Example:**
```bash
$ python3 scripts/video/special_rename.py -n -v ~/videos/channel_videos

########################################
#            Special Rename            #
########################################
Directory: /home/user/videos/channel_videos
Extensions: mkv, mp4
Recursive: No
Dry run: Yes

Would rename: Midnite - Value Life Lyrics.mp4 -> Midnite_-_Value_Life_Lyrics.mp4
Would rename: Akae Beka [test].mkv -> Akae_Beka__test_.mkv
Dry run complete. Would rename 2 files.
```

## Notes

- Both scripts use Bash with error handling (`set -euo pipefail` in `rename.sh`).
- Ensure the directory exists and is writable before running.
- These scripts are designed for post-download cleanup of YouTube video files.</content>
<parameter name="filePath">/home/lbgc/Documents/dev/repos/midnite-archive/scripts/README.md
