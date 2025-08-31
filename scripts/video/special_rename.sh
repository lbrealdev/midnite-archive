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
    # mv "$file" "${file// /_}"
    # rm_fullwidth_colon=$(echo "$file" | sed 's/[ /:：]/_/g')
    # rm_special="${rm_fullwidth_colon/\//_}"
    # mv "$file" "$new_name"
    # echo "mv \"$file\" \"$rm_special\""
    first_speacial_caracter="${file/⧸/_}"
    second_special_caracter=$(echo "$first_speacial_caracter" | sed 's/[ /:：]/_/g')
    echo "$second_special_caracter"
  done
fi
