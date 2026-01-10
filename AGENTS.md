# AGENTS.md - Development Guidelines for midnite-archive

## Rule Priority Hierarchy

All rules follow strict priority hierarchy. Higher priority rules override lower priority rules:

### Priority 1: Core Operational Rules (Highest Priority)
- Behaviors and work rules from AGENTS.md have absolute priority
- Special commands (@refresh, @do, @help commit) override general guidelines
- Mode restrictions (Plan vs Build) cannot be violated

### Priority 2: Authorization & Safety Protocols
- @do authorization requirements for commits
- Git operation restrictions (never commit to main, build mode limitations)
- Security and safety guidelines

### Priority 3: Operational Guidelines
- Special command behaviors and limitations
- Documentation standards and formatting rules
- Quality assurance procedures

### Priority 4: Development Standards
- Code style and naming conventions
- Best practices for maintainability

### Priority 5: General Recommendations (Lowest Priority)
- Compatibility considerations

## Agents Behavior Guidelines

This section outlines behavioral guidelines for AI agents working on the midnite-archive project. These rules ensure safe, responsible, and effective collaboration while maintaining code quality and security.

### What Agents SHOULD Do
- Operate in Plan Mode first for significant changes
- Ask clarifying questions when intent is unclear
- Follow all guidelines in this AGENTS.md document
- Use appropriate tools for each task
- Never generate malicious code or bypass security

### What Agents SHOULD NOT Do
- Never modify files without explicit user permission
- Never commit changes unless authorized with `@do`
- Never bypass security checks or generate malicious code

### What Agents SHOULD NOT Do
- Never modify files without explicit user permission
- Never commit changes unless authorized with `@do`
- Never bypass security checks or generate malicious code

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

**Mode Restrictions**
- Plan Mode: Read-only operations only
- Build Mode: Modifications allowed with @do authorization



### Quality Assurance

**Pre-commit Checks**
- Always run pre-commit hooks before committing
- Fix any identified linting or formatting issues
- Ensure documentation renders correctly
- Test documented commands on clean environment

## Agent Special Commands

- `@refresh` - Update memory regarding latest AGENTS.md changes
- `@help commit` - Provide commit examples with `git commit -am`
- `@do` - Authorize commits (user message only, Build mode only)

## Build/Lint/Test Commands
- Build: `just run <youtube-channel>`
- Lint: `find scripts -name "*.sh" -exec shellcheck {} \;`
- Test: `./scripts/yt/channel_list_generate.sh @testchannel`

## Code Style Guidelines

- Group related scripts in subdirectories (yt/, video/)
- Use snake_case for filenames and UPPERCASE for global variables
- Include `set -euo pipefail` at script start
- Sanitize user inputs to prevent injection

#### Script Template

```shell
#!/bin/bash
set -euo pipefail

usage() { echo "Usage: $0 <args>"; exit 1; }
available() { command -v "$1" >/dev/null; }

if [ "$#" -lt 1 ]; then usage; fi
# Main logic here
```

### Naming Conventions

#### Variables
- Use `UPPERCASE_WITH_UNDERSCORES` for global variables and constants
- Use `lowercase_with_underscores` for local variables
- Be descriptive and indicate purpose

```shell
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

```shell
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

```shell
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

```shell
# Good
if [[ -f "$file" ]]; then
  echo "Processing $file"
  mv "$file" "${file}.backup"
fi
```

#### Line Length
- Limit lines to 100 characters maximum
- Break long commands for readability

```shell
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

```shell
# Good
mv "$file" "$new_name"
echo "Processing file: $filename"
```

### Error Handling

#### Script Options
Always include `set -euo pipefail` at the top of scripts:

```shell
#!/bin/bash
set -euo pipefail  # Exit on error, undefined vars, pipeline failures
```

#### Input Validation
Validate all inputs early in the script:

```shell
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

```shell
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

```shell
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

```shell
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

```shell
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

```shell
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

```shell
# Good
temp_file=$(mktemp)
trap 'rm -f "$temp_file"' EXIT
```

### Best Practices
- Write self-documenting code with meaningful variable names

### Pre-commit Checklist
- Run pre-commit hooks before committing
- Fix any identified linting or formatting issues
- Ensure documentation renders correctly

### Development Workflow
1. Follow Git Operations rules
2. Create feature branch: `git checkout -b feature/new-script`
3. Write script following guidelines
4. Test thoroughly and run linting
5. Commit with `@do` authorization
