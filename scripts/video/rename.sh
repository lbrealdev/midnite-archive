#!/bin/bash

set -euo pipefail

usage() {
  echo "Usage: $0 <directory>"
  exit 1
}

if [ "$#" -lt 1 ]; then
  usage
fi

DIRECTORY="$1"
DIRECTORY_FULL_PATH=$(realpath "$DIRECTORY")

echo "$DIRECTORY_FULL_PATH"

function rename() {
  echo "Renaming files..."

  local renamed_count=0
  local total_files
  total_files=$(find . -type f \( -name "*.mkv" -o -name "*.description" \) | wc -l)

  while IFS= read -r file; do
    newname="${file// /_}"
    if [ "$file" != "$newname" ]; then
      echo "Renaming: $file -> $newname"
      mv "$file" "$newname"
      ((renamed_count++))
    fi
  done < <(find . -type f \( -name "*.mkv" -o -name "*.description" \))

  echo "Processed $total_files files, renamed $renamed_count."
}

if [[ -d "$DIRECTORY_FULL_PATH" ]]; then
  files_to_rename=$(find "$DIRECTORY_FULL_PATH" -type f \( -name "*.mkv" -o -name "*.description" \))

  count_files=$(echo "$files_to_rename" | wc -l)

  echo "Input directory: $DIRECTORY_FULL_PATH"
  echo "Files found in the directory: $count_files"

  cd "$DIRECTORY_FULL_PATH" || exit

  rename

  echo "Done!"
fi
