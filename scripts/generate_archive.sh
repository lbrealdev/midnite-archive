#!/bin/bash

set -euo pipefail

INPUT_DIR="${1:-.}"

# Validate directory
if [[ ! -d "$INPUT_DIR" ]]; then
  echo "Error: '$INPUT_DIR' is not a directory"
  exit 1
fi

INPUT_DIR=$(realpath "$INPUT_DIR")

echo "Scanning directory: $INPUT_DIR"

# Video extensions to search
VIDEO_EXTENSIONS="mp4 mkv webm mp3 flac"

# Build find command
FIND_ARGS=()
for ext in $VIDEO_EXTENSIONS; do
  FIND_ARGS+=(-name "*.$ext" -o)
done
# Remove trailing -o
unset 'FIND_ARGS[${#FIND_ARGS[@]}-1]'

# Find video files and extract IDs
ARCHIVE_DIR="$INPUT_DIR/.archive"
ARCHIVE_FILE="$ARCHIVE_DIR/downloads.archive"

mkdir -p "$ARCHIVE_DIR"

count=0
skipped=0

# Process files
while IFS= read -r -d '' file; do
  filename=$(basename "$file")

  # Extract 11-char YouTube ID before extension
  if [[ "$filename" =~ ([A-Za-z0-9_-]{11})\.[^.]+$ ]]; then
    video_id="${BASH_REMATCH[1]}"
    echo "youtube $video_id" >> "$ARCHIVE_FILE"
    count=$((count + 1))
  else
    echo "Skipped (no ID): $filename"
    skipped=$((skipped + 1))
  fi
done < <(find "$INPUT_DIR" -maxdepth 1 -type f \( "${FIND_ARGS[@]}" \) -print0 | sort -z)

echo ""
echo "Done: $count video IDs written to $ARCHIVE_FILE"
if [[ $skipped -gt 0 ]]; then
  echo "Skipped: $skipped files (no YouTube ID found)"
fi
