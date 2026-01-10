# YouTube Channels Directory

This directory contains curated YouTube channels with content related to Midnite/Akae Beka and Vaughn Benjamin.

## Files

- `youtube-channels.txt` - List of YouTube channel URLs (one per line)
- Comments starting with `#` group channels by category

## Usage

### Manual Usage
```bash
# Download from a specific channel
./scripts/yt/download_video.sh https://www.youtube.com/@MidniteOfficial

# Generate channel list first, then download
./scripts/yt/channel_list_generate.sh @MidniteOfficial
./scripts/yt/download_video.sh MidniteOfficial/lists/*.txt
```

### Batch Processing
```bash
# Process all official channels
for url in $(grep -v '^#' youtube-channels.txt | grep -E "(MidniteOfficial|AkaeBeka)$"); do
  echo "Processing: $url"
  ./scripts/yt/download_video.sh "$url"
done
```

## Categories

- **Official Channels**: Primary band channels
- **Fan Channels**: Community-created content
- **Related Artists**: Vaughn Benjamin's other projects
- **Live Performances**: Concert recordings and tours
- **Interviews**: Documentary and interview content

## Contributing

To add new channels:
1. Add the channel URL to `youtube-channels.txt`
2. Group with similar channels using comments
3. Test that the channel exists and has relevant content
4. Update this README if adding new categories
