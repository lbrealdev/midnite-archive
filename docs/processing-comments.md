# Processing YouTube Comments JSON

This document explains how to process YouTube comment data downloaded by yt-dlp. The comments are stored in JSON format and can be processed using jq to extract specific information.

## Summary

YouTube comments downloaded by yt-dlp are stored as JSON files with the pattern `video_id.comments.json`. This guide provides practical jq commands to extract usernames, comment text, and other metadata from these files. All commands use the `cat file.json | jq ...` format for consistency and readability.

## Comment JSON Structure

YouTube comment JSON files typically have this structure:

```json
{
  "comments": [
    {
      "id": "comment_id",
      "text": "Comment text here",
      "timestamp": "2024-01-01T12:00:00Z",
      "time_text": "2 days ago",
      "like_count": 42,
      "is_favorited": false,
      "author": {
        "id": "author_id",
        "name": "Username",
        "channel_url": "https://www.youtube.com/channel/author_id",
        "verified": false,
        "badges": []
      },
      "replies": []
    }
  ]
}
```

## Extract Comments with Usernames

### Get All Comments with Usernames
```shell
# Extract username and comment text as JSON
cat video_id.comments.json | jq '.comments[] | {username: .author.name, comment: .text}'

# Extract as simple text format
cat video_id.comments.json | jq -r '.comments[] | "\(.author.name): \(.text)"'
```

### Filter Comments by Username
```shell
# Find comments by specific user
cat video_id.comments.json | jq '.comments[] | select(.author.name == "Username") | {username: .author.name, comment: .text}'
```

### Count Total Comments
```shell
cat video_id.comments.json | jq '.comments | length'
```

### Extract Only Usernames
```shell
# Get list of all usernames who commented
cat video_id.comments.json | jq -r '.comments[].author.name' | sort | uniq
```

### Extract Only Comment Text
```shell
# Get all comment text
cat video_id.comments.json | jq -r '.comments[].text'
```

## Advanced Processing

### Comments with Timestamps
```shell
# Include timestamps in output
cat video_id.comments.json | jq -r '.comments[] | "\(.author.name) [\(.time_text)]: \(.text)"'
```

### Comments with Like Counts
```shell
# Include like counts for popular comments
cat video_id.comments.json | jq '.comments[] | select(.like_count > 10) | {username: .author.name, comment: .text, likes: .like_count}'
```

### Export to CSV Format
```shell
# Create CSV output
cat video_id.comments.json | jq -r '.comments[] | "\(.author.name),\(.text | gsub("\"";"\"\"") | "\"\(.)\"")"' > comments.csv
```

## Batch Processing Multiple Files

### Process All Comment Files in Directory
```shell
# Extract all comments from all JSON files
for file in *.comments.json; do
  echo "Processing $file:"
  cat "$file" | jq -r '.comments[] | "\(.author.name): \(.text)"'
  echo "---"
done
```

### Count Comments Across All Videos
```shell
# Total comments across all video files
find . -name "*.comments.json" -exec sh -c 'cat "$1" | jq ".comments | length"' _ {} \; | paste -sd+ | bc
```

## Practical Examples

### Daily Comment Summary
```shell
# Generate summary of comments by user
cat video.comments.json | jq -r '.comments[].author.name' | sort | uniq -c | sort -nr | head -10
```

### Search Comments by Keyword
```shell
# Find comments containing specific words
cat video.comments.json | jq -r '.comments[] | select(.text | test("keyword"; "i")) | "\(.author.name): \(.text)"'
```

### Export for Analysis
```shell
# Create analysis-ready format
cat video.comments.json | jq -r '.comments[] | {username: .author.name, comment: .text, timestamp: .timestamp, likes: .like_count}' > analysis.json
```

## Notes

- Comment data is only available for videos where comments are enabled
- Some comments may be replies to other comments (check the `replies` field)
- yt-dlp may not download all comments if there are rate limits or API restrictions
- Always respect YouTube's terms of service when processing comment data
