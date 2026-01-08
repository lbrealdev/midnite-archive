# AGENTS.md - Development Guidelines for midnite-archive

## Overview
This document provides guidelines for contributing to the midnite-archive codebase. The project consists of bash scripts for YouTube video archiving using `yt-dlp` and `ffmpeg`.

## Build/Lint/Test Commands

### Build Commands
```bash
# Run scripts
just run <youtube-channel>
just rename <directory>

# Install dependencies
uv tool install yt-dlp
```

### Lint Commands
```bash
# Install shellcheck first: sudo apt install shellcheck

# Lint all scripts
find scripts -name "*.sh" -exec shellcheck {} \;

# Lint specific script
shellcheck scripts/yt/channel_list_generate.sh
```

### Test Commands
```bash
# Manual testing examples
./scripts/yt/channel_list_generate.sh @testchannel
./scripts/yt/download_video.sh test-list.txt
./scripts/video/rename.sh test_videos/

# Single functionality tests
echo "test:with spaces" | sed 's/[ /:：]/_/g'
command -v yt-dlp && echo "available" || echo "missing"
```

## Code Style Guidelines

### Script Organization

#### File Structure
```
scripts/
├── yt/           # YouTube-related scripts
│   ├── channel_list_generate.sh
│   ├── download_video.sh
│   └── download_video_comments.sh
└── video/        # Video processing scripts
    ├── rename.sh
    └── special_rename.sh
```

- Group related scripts in subdirectories
- Use descriptive, snake_case filenames
- Keep scripts focused on single responsibilities

#### Script Template
Start all scripts with this template:

```bash
#!/bin/bash

set -euo pipefail

# Script description and usage
usage() {
  echo "Usage: $0 <required-arg> [optional-arg]"
  echo "Description: Brief description of what this script does"
  exit 1
}

# Check if required commands are available
available() { command -v "$1" >/dev/null; }

# Validate arguments
if [ "$#" -lt 1 ]; then
  usage
fi

# Main logic here
```

### Naming Conventions

#### Variables
- Use `UPPERCASE_WITH_UNDERSCORES` for global variables and constants
- Use `lowercase_with_underscores` for local variables
- Be descriptive and indicate purpose

```bash
# Good
YT_CHANNEL_NAME="$1"
TIMESTAMP=$(date '+%Y%m%d%H%M%S')
OUTPUT_FILE="${YT_CHANNEL_NAME}-list-${TIMESTAMP}.txt"

# Avoid
channel="$1"        # Too generic
ts=$(date +%s)      # Not descriptive
file="output.txt"   # Unclear context
```

#### Functions
- Use `lowercase_with_underscores` for function names
- Start with action verbs (get, process, validate)
- Keep functions small and focused

```bash
# Good
validate_channel_name() { ... }
process_video_list() { ... }
generate_output_filename() { ... }

# Avoid
function() { ... }  # Too generic
do_stuff() { ... }  # Not descriptive
```

#### Files and Directories
- Use `snake_case` for all filenames and directory names
- Include descriptive suffixes where helpful

```bash
# Good
channel_list_generate.sh
video_rename.sh
download_comments.sh

# Avoid
channelListGenerate.sh  # camelCase
video-rename.sh         # hyphens
downloadComments.sh     # inconsistent
```

### Formatting and Style

#### Indentation and Spacing
- Use 2 spaces for indentation (not tabs)
- Add blank lines between logical sections
- Space around operators and keywords

```bash
# Good
if [[ -f "$file" ]]; then
  echo "Processing $file"
  mv "$file" "${file}.backup"
fi
```

#### Line Length
- Limit lines to 100 characters maximum
- Break long commands for readability

```bash
# Good
yt-dlp \
  --flat-playlist \
  --print "%(title)s-%(id)s" \
  "https://www.youtube.com/@$CHANNEL" \
  > "$OUTPUT_FILE"
```

#### Quoting
- Always quote variable expansions: `"$variable"`
- Quote strings containing spaces or special characters

```bash
# Good
mv "$file" "$new_name"
echo "Processing file: $filename"
```

### Error Handling

#### Script Options
Always include `set -euo pipefail` at the top of scripts:

```bash
#!/bin/bash
set -euo pipefail  # Exit on error, undefined vars, pipeline failures
```

#### Input Validation
Validate all inputs early in the script:

```bash
# Check argument count
if [ "$#" -lt 1 ]; then
  echo "Error: Missing required argument"
  usage
fi

# Check file existence
if [[ ! -f "$input_file" ]]; then
  echo "Error: Input file '$input_file' not found"
  exit 1
fi

# Check command availability
if ! available yt-dlp; then
  echo "Error: yt-dlp is not installed. Please install it first."
  exit 1
fi
```

#### Error Messages
- Prefix errors with "Error:" for clarity
- Include helpful context and suggestions
- Exit with appropriate codes (1 for general errors)

```bash
# Good
if [[ ! -d "$output_dir" ]]; then
  echo "Error: Output directory '$output_dir' does not exist"
  echo "Create it with: mkdir -p '$output_dir'"
  exit 1
fi
```

### Functions

#### Definition Style
- Use consistent function definition syntax
- Add brief documentation comments
- Keep functions small (under 20 lines)

```bash
# Generate timestamp-based filename
# Arguments: prefix, extension
# Returns: filename string
generate_filename() {
  local prefix="$1"
  local extension="$2"
  local timestamp
  timestamp=$(date '+%Y%m%d%H%M%S')
  echo "${prefix}-${timestamp}.${extension}"
}
```

#### Parameter Handling
- Use local variables for parameters
- Validate parameter types and values
- Document expected parameters

### Comments and Documentation

#### Comment Style
- Use `#` for all comments
- Add space after `#`
- Keep comments concise but descriptive

```bash
# Good
# Generate channel video list using yt-dlp
yt-dlp --flat-playlist "https://youtube.com/@$channel" > "$output_file"

# Check if output directory exists and create if needed
if [[ ! -d "$output_dir" ]]; then
  mkdir -p "$output_dir"
fi
```

#### Function Documentation
- Add brief description above each function
- Document parameters and return values

```bash
# Extract video IDs from channel list file
# Arguments:
#   $1: Input file containing video titles and IDs
#   $2: Output file for video URLs
# Returns: 0 on success, 1 on error
extract_video_urls() { ... }
```

### Security Considerations

#### Input Sanitization
- Sanitize user inputs to prevent command injection
- Use arrays for complex command arguments
- Avoid eval and dangerous constructs

```bash
# Good - safe command construction
yt_dlp_args=(
  --flat-playlist
  --print "%(title)s-%(id)s"
  "https://www.youtube.com/@$channel"
)
yt-dlp "${yt_dlp_args[@]}" > "$output_file"
```

#### File Operations
- Use absolute paths when possible
- Check permissions before file operations
- Clean up temporary files

```bash
# Good
temp_file=$(mktemp)
trap 'rm -f "$temp_file"' EXIT
```

### Best Practices

#### Performance
- Use efficient commands (e.g., `grep` instead of `sed` for simple matches)
- Process files in streams when possible
- Avoid unnecessary forks

#### Maintainability
- Write self-documenting code
- Use meaningful variable names
- Keep scripts modular and reusable

#### Compatibility
- Use POSIX-compliant syntax where possible
- Test on multiple environments
- Document system requirements

### Pre-commit Checklist

Before committing changes:

1. Run shellcheck on all modified scripts
2. Test scripts manually with edge cases
3. Ensure all functions have documentation
4. Verify error handling works correctly
5. Check that naming conventions are followed

### Development Workflow

1. Create feature branch: `git checkout -b feature/new-script`
2. Write script following these guidelines
3. Test thoroughly with various inputs
4. Run linting: `shellcheck your-script.sh`
5. Commit with descriptive message</content>
<parameter name="filePath">AGENTS.md