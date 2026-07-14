# yt-dlp wrapper PoCs (issue #50)

video_url := env("POC_VIDEO_URL", "https://www.youtube.com/watch?v=jNQXAC9IVRw")

# Build both independent PoC crates
@build:
    cargo build --release --manifest-path ytd-rs/Cargo.toml
    cargo build --release --manifest-path boul2gom/Cargo.toml

# Run ytd-rs PoC (needs yt-dlp + deno on PATH)
@ytdrs *args:
    POC_OUT=ytd-rs/out cargo run --release --manifest-path ytd-rs/Cargo.toml -- {{ if args == "" { video_url } else { args } }}

# Run boul2gom PoC (manages libs under boul2gom/libs)
@boul2gom *args:
    POC_OUT=boul2gom/out POC_LIBS=boul2gom/libs cargo run --release --manifest-path boul2gom/Cargo.toml -- {{ if args == "" { video_url } else { args } }}

# Remove PoC build artifacts and download dirs
@clean:
    cargo clean --manifest-path ytd-rs/Cargo.toml
    cargo clean --manifest-path boul2gom/Cargo.toml
    rm -rf ytd-rs/out boul2gom/out boul2gom/libs
