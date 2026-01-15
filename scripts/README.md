# Scripts Directory

This directory contains various scripts for managing the midnite-archive repository, primarily focused on YouTube video archiving and processing.

## Video Scripts (`video/` subdirectory)

The `video/` subdirectory contains scripts for processing and renaming video files downloaded from YouTube.

### `rename.sh`

Renames `.mkv` and `.description` files in a specified directory (and its subdirectories) by replacing spaces with underscores.

**Usage:**
```bash
./scripts/video/rename.sh /path/to/directory
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

## Notes

- Both scripts use Bash with error handling (`set -euo pipefail` in `rename.sh`).
- Ensure the directory exists and is writable before running.
- These scripts are designed for post-download cleanup of YouTube video files.</content>
<parameter name="filePath">/home/lbgc/Documents/dev/repos/midnite-archive/scripts/README.md
