#!/bin/bash

INPUT="$1"

if [[ ! -d "$INPUT" ]]; then
  echo "The $INPUT directory has been created."
  mkdir "$INPUT"
fi