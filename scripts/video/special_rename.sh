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
    primary_special_char="${file/⧸/_}"
    secondary_special_char=$(echo "$primary_special_char" | sed 's/[ /:：]/_/g')
    
    # Debug
    # echo "mv \"$file\" \"$secondary_special_char\""

    if [[ -e "$secondary_special_char" ]]; then
      echo "File $secondary_special_char already exist, skipping..."
      continue
    fi

    mv "$file" "$secondary_special_char"
  done
fi
