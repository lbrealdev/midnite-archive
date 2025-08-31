#!/bin/bash

CHANNEL="$1"
TIMESTAMP=$(date '+%Y%m%d%H%M%S')

usage() {
  echo "Usage: $0 <youtube-channel>"
  exit 1
}

# Check if there enough arguments
if [ "$#" -lt 1 ]; then
  usage
fi

echo "########################################"
echo "     YouTube Channel Generator List     "
echo "########################################"

if [[ "$CHANNEL" =~ ^@ ]]; then
  CHANNEL="${CHANNEL//@}"
fi

YT_URL="https://www.youtube.com"
YT_CHANNEL_NAME="$CHANNEL"
YT_CHANNEL_FILE_NAME="$YT_CHANNEL_NAME-list-$TIMESTAMP"
YT_CHANNEL_FILE_FORMAT="txt"

YT_CHANNEL_TITLE_OUTPUT_FILE="${YT_CHANNEL_FILE_NAME/list/list-title}.$YT_CHANNEL_FILE_FORMAT"
YT_CHANNEL_URL_OUTPUT_FILE="${YT_CHANNEL_FILE_NAME/list/list-url}.$YT_CHANNEL_FILE_FORMAT"

printf "\nYouTube channel name: %s\n" "${YT_CHANNEL_NAME/#/@}"
printf "YouTube channel url: %s\n\n" "${YT_URL/%//@$YT_CHANNEL_NAME}"

echo "Generating output files..."
echo ""
echo "YouTube Channel file (tile): $YT_CHANNEL_TITLE_OUTPUT_FILE"
echo "YouTube Channel file (url): $YT_CHANNEL_URL_OUTPUT_FILE"
echo ""

echo "Fetching channel list..."

yt-dlp --flat-playlist --print "%(title)s-%(id)s" "https://www.youtube.com/${YT_CHANNEL_NAME/#/@}" >> "$YT_CHANNEL_TITLE_OUTPUT_FILE"

# https://www.youtube.com/watch?v=
grep -oE '[A-Za-z0-9_-]{11}$' "$YT_CHANNEL_TITLE_OUTPUT_FILE" | sed "s|^|${YT_URL/%//watch?v=}|" > "$YT_CHANNEL_URL_OUTPUT_FILE"

echo "Done!"
