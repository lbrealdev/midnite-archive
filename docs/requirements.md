# Requirements

To run the **midnite-archive** scripts, you need to install the following tools.

## Prerequisites

- **yt-dlp** - YouTube downloader with EJS support
- **Deno** - JavaScript runtime required for yt-dlp's EJS functionality
- **ffmpeg** - Media processing library

## Installation

### Option 1: Using mise (Recommended)

[mise](https://mise.jdx.dev/) is a tool version manager that makes it easy to install and manage multiple tools.

First, clone this repository and install the dependencies using mise.

Install `mise` if you don't have it:
```shell
curl https://mise.run | sh
```

Once `mise` is installed, install tools from `mise.toml`:
```shell
mise install
```

Verify installations:
```shell
mise ls -l
```

### Option 2: Manual Installation

#### yt-dlp

```shell
# Using pip
pip install yt-dlp

# Or using uv
uv tool install yt-dlp
```

#### Deno

```shell
# Using official installer
curl -fsSL https://deno.land/install.sh | sh

# Add deno to your PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.deno/bin:$PATH"
```

#### ffmpeg

```shell
sudo apt update && sudo apt install ffmpeg -y

# Verify installation
ffmpeg -version
```

## EJS Configuration

> [!IMPORTANT]
> Scripts use yt-dlp's External JavaScript (EJS) system with Deno to handle YouTube's JavaScript challenges.

### Configuration Details

The scripts automatically configure `yt-dlp` with:

- `--remote-components ejs:npm` - Enables EJS with npm package support
- `--js-runtimes deno:$(which deno)` - Specifies Deno as the JavaScript runtime

## References

- [yt-dlp Dependencies](https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#dependencies)
- [yt-dlp EJS Documentation](https://github.com/yt-dlp/yt-dlp/wiki/EJS)
- [EJS Repository](https://github.com/yt-dlp/ejs)
