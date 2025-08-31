#!/bin/bash

set -euo pipefail

usage() {
  echo "Usage: $0 <directory>"
  exit 1
}

if [ "$#" -lt 1 ]; then
  usage
fi

echo "########################################"
echo "#            Rename Script             #"
echo "########################################"

INPUT_DIR="$1"

if [[ -d "$INPUT_DIR" ]]; then
  cd "$INPUT_DIR" || exit

  for file in *.description; do
    # mv "$file" "${file// /_}"
    new_name=$(echo "$file" | sed 's/[ /:：]/_/g')
    echo "$new_name"
  done
fi
