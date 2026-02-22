# Greeting message
@say:
    echo "Keep good relations, mongst InI"

# Run clippy linter
@lint:
    cargo clippy

# Format the code
@fmt:
    cargo fmt

# Clean build artifacts
@clean:
    cargo clean

# Build debug binary
@build:
    cargo build

# Build release binary (optimized)
@release:
    cargo build --release

# Check compilation without building
@check:
    cargo check

# Run tests
@test:
    cargo test

# CI security audit (zizmor + pinact verify)
[working-directory('.github')]
@ci-scan:
    zizmor dependabot.yml ./workflows/*.yml --no-exit-codes
    pinact run --verify ./workflows/*.yml

# Pin GitHub Actions to immutable SHAs
[working-directory('.github')]
@ci-pin:
    pinact run ./workflows/*.yml

# List mise tools installed in current directory
@mise-tools:
    mise ls --json | jq -r --arg pwd "$(pwd)" 'to_entries[] | select(.value[].source.path != null and (.value[].source.path | contains($pwd))) | .key'

# List GitHub Actions workflows
@workflows:
    gh workflow list --json name --jq "to_entries[] | .value.name"
