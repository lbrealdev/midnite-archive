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
