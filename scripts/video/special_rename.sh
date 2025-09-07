#!/bin/bash

set -x

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
  
  for file in *.mkv; do

    [ ! -f "$file" ] && continue

    newname=$(echo "$file" | sed 's/[ /:：⧸]/_/g')

    mv "$file" "$newname"
  done
fi
