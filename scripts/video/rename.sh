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

  for file in *.{mkv,description}; do
    mv "$file" "${file// /_}"
  done
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
