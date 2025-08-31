#!/bin/bash

set -euo pipefail

usage() {
  echo "Usage: $0 <directory>"
  exit 1
}

if [ "$#" -lt 1 ]; then
  usage
fi

echo ""
echo "########################################"
echo "#            Rename Script             #"
echo "########################################"
echo ""

INPUT_DIR="$1"

if [[ -d "$INPUT_DIR" ]]; then
  cd "$INPUT_DIR" || exit

  for file in *.description; do
    cleaned_name="${file//⧸/_}"
    cleaned_name=$(echo "$cleaned_name" | sed 's/[ /:：]/_/g')

    [[ "$cleaned_name" == "$file" ]] && continue

    if [[ -e "$cleaned_name" ]]; then
      echo "File $cleaned_name already exists, skipping..."
      continue
    fi
    
    # Debug
    # echo "mv \"$file\" \"$secondary_special_char\""

    mv "$file" "$cleaned_name"
    echo "Renamed: $file → $cleaned_name"
  done
fi
