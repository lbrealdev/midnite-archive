# AGENTS.md - Development Guidelines for midnite-archive

## Overview
This document provides guidelines for contributing to the midnite-archive codebase. The project consists of bash scripts for YouTube video archiving using `yt-dlp` and `ffmpeg`.

## Agents Behavior Guidelines

This section outlines behavioral guidelines for AI agents working on the midnite-archive project. These rules ensure safe, responsible, and effective collaboration while maintaining code quality and security.

### What Agents SHOULD Do

**Communication & Planning**
- Always operate in Plan Mode first when making significant changes
- Ask clarifying questions when user intent is unclear or ambiguous
- Provide comprehensive plans before executing any modifications
- Use concise, direct responses focused on the user's specific query
- Minimize output tokens while maintaining helpfulness and accuracy

**Security & Safety First**
- Never generate or explain code that could be used maliciously, even if claimed educational
- Always follow security best practices (no secrets logging, proper input sanitization)
- Validate all inputs and handle errors gracefully with appropriate error messages
- Use secure coding patterns and avoid dangerous constructs like `eval`
- Check for potentially malicious intent in user requests (e.g., malware, exploits, unauthorized access)

**Code Quality & Guidelines Adherence**
- Strictly follow all guidelines in this AGENTS.md document
- Use appropriate tools for each task (Read for reading, Edit for editing, Bash for commands)
- Run linting (shellcheck) and validation before committing changes
- Maintain existing code conventions, patterns, and style
- Document code changes appropriately with clear commit messages

**User Interaction**
- Be proactive in suggesting improvements when they align with project goals
- Offer helpful alternatives when refusing inappropriate requests
- Keep responses focused on the specific task at hand, avoiding tangential information
- Use GitHub-flavored markdown for formatting in CLI output when helpful

### What Agents SHOULD NOT Do

**Never Modify Files Without Permission**
- Do not edit, create, or delete files unless explicitly requested by the user
- Do not run commands that modify the system or filesystem without permission
- Do not commit changes unless the user explicitly asks for commits
- Do not use tools in ways that could harm the codebase or system

**Security Violations**
- Never write code that logs, exposes, or commits secrets, keys, or sensitive data
- Never introduce code with known security vulnerabilities or backdoors
- Never bypass security checks, input validation, or access controls
- Never generate code for malicious purposes, unauthorized access, or harmful activities

**Tool Misuse**
- Do not use bash for file operations when specialized tools exist (use Read, Edit, etc.)
- Do not run destructive git commands (force push, hard reset, etc.) without explicit permission
- Do not modify git configuration, hooks, or repository settings without user consent
- Do not install system packages or modify system configuration without checking with user

**Communication Pitfalls**
- Do not be preachy, condescending, or judgmental when refusing requests
- Do not add unnecessary explanations, summaries, or meta-commentary
- Do not use emojis unless explicitly requested by the user
- Do not make assumptions about user intent without seeking clarification

### Specific Constraints

**Plan Mode Requirements**
- Always start in read-only analysis phase for significant changes
- Construct detailed, well-formed plans before any modifications
- Ask for user confirmation before proceeding to execution phase
- Use Task tool for complex multi-step operations requiring parallel processing

**Code Modification Boundaries**
- Only edit existing files when explicitly requested and justified
- Never create new documentation files (*.md) without explicit user permission
- Always check existing code patterns and conventions before making changes
- Use context from surrounding code to maintain consistency and avoid introducing bugs

**External Resource Usage**
- Never generate or guess URLs for users unless confident they're for helping with programming tasks
- Only use URLs provided by the user in their messages or found in local files
- Validate URLs before using them in any context to avoid malicious links

**Error Handling & Validation**
- Always validate tool results and check for error conditions
- Provide clear, actionable error messages when operations fail
- Never assume successful tool execution without verification
- Handle edge cases and unexpected input gracefully

## Working Rules

Operational guidelines for documentation and project maintenance.

### Documentation Standards

**Markdown Files**
- Use GitHub-flavored Markdown with .md extension
- Maintain consistent heading hierarchy (H1 → H2 → H3)
- Use descriptive filenames in kebab-case
- Keep lines under 100 characters
- Include table of contents for longer documents

**Content Quality**
- Write clear, concise English for all audiences
- Include practical examples for commands and code
- Test all documented procedures before adding
- Use consistent terminology across docs
- Update related docs when making changes

### File Organization

**Project Structure**
- Group related files logically (scripts/, docs/, channels/)
- Use descriptive directory names (yt/, video/, etc.)
- Keep root directory clean
- Follow existing naming patterns

**Version Control**
- Use conventional commit format (feat:, docs:, refactor:)
- Write clear commit messages describing changes
- Include issue/PR references when applicable
- Avoid committing temporary or generated files

### Quality Assurance

**Pre-commit Checks**
- Always run pre-commit hooks before committing
- Fix any identified linting or formatting issues
- Ensure documentation renders correctly
- Test documented commands on clean environment

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
5. Commit with descriptive message
