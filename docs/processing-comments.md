# Processing YouTube Comments JSON

This document explains how to process YouTube comment data downloaded by yt-dlp. The comments are stored in JSON format and can be processed using jq to extract specific information.

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
jq '.comments[] | {username: .author.name, comment: .text}' video_id.comments.json

# Extract as simple text format
jq -r '.comments[] | "\(.author.name): \(.text)"' video_id.comments.json
```

### Filter Comments by Username
```shell
# Find comments by specific user
jq '.comments[] | select(.author.name == "Username") | {username: .author.name, comment: .text}' video_id.comments.json
```

### Count Total Comments
```shell
jq '.comments | length' video_id.comments.json
```

### Extract Only Usernames
```shell
# Get list of all usernames who commented
jq -r '.comments[].author.name' video_id.comments.json | sort | uniq
```

### Extract Only Comment Text
```shell
# Get all comment text
jq -r '.comments[].text' video_id.comments.json
```

## Advanced Processing

### Comments with Timestamps
```shell
# Include timestamps in output
jq -r '.comments[] | "\(.author.name) [\(.time_text)]: \(.text)"' video_id.comments.json
```

### Comments with Like Counts
```shell
# Include like counts for popular comments
jq '.comments[] | select(.like_count > 10) | {username: .author.name, comment: .text, likes: .like_count}' video_id.comments.json
```

### Export to CSV Format
```shell
# Create CSV output
jq -r '.comments[] | "\(.author.name),\(.text | gsub("\"";"\"\"") | "\"\(.)\"")"' video_id.comments.json > comments.csv
```

## Batch Processing Multiple Files

### Process All Comment Files in Directory
```shell
# Extract all comments from all JSON files
for file in *.comments.json; do
  echo "Processing $file:"
  jq -r '.comments[] | "\(.author.name): \(.text)"' "$file"
  echo "---"
done
```

### Count Comments Across All Videos
```shell
# Total comments across all video files
find . -name "*.comments.json" -exec jq '.comments | length' {} \; | paste -sd+ | bc
```

## Practical Examples

### Daily Comment Summary
```shell
# Generate summary of comments by user
jq -r '.comments[].author.name' video.comments.json | sort | uniq -c | sort -nr | head -10
```

### Search Comments by Keyword
```shell
# Find comments containing specific words
jq -r '.comments[] | select(.text | test("keyword"; "i")) | "\(.author.name): \(.text)"' video.comments.json
```

### Export for Analysis
```shell
# Create analysis-ready format
jq -r '.comments[] | {username: .author.name, comment: .text, timestamp: .timestamp, likes: .like_count}' video.comments.json > analysis.json
```

## Notes

- Comment data is only available for videos where comments are enabled
- Some comments may be replies to other comments (check the `replies` field)
- yt-dlp may not download all comments if there are rate limits or API restrictions
- Always respect YouTube's terms of service when processing comment data
