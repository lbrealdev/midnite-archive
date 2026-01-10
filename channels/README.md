# YouTube Channels Directory

This directory contains curated YouTube channels with content related to Midnite/Akae Beka and Vaughn Benjamin.

## Files

- `youtube-channels.json` - JSON file containing YouTube channel URLs

## Categories

- **Official Channels**: Primary band channels
- **Fan Channels**: Community-created content
- **Related Artists**: Vaughn Benjamin's other projects
- **Live Performances**: Concert recordings and tours
- **Interviews**: Documentary and interview content

## Contributing

To add new channels:

1. Add a new object to the `channels` array in `youtube-channels.json`
2. Include only the `url` field
3. Test that the channel exists and has relevant content

Example:
```json
{
  "url": "https://www.youtube.com/@NewChannel"
}
```
