# Requirements

To run the **midnite-archive** scripts, you need to install the following tools. The scripts use yt-dlp with External JavaScript (EJS) support for downloading YouTube content.

- https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#dependencies

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

Once mise is installed, install tools from `mise.toml`:
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
sudo apt update && sudo apt install -y ffmpeg

# Verify installation
ffmpeg -version
```

### yt-dlp: External JS Scripts Setup Guide

To download from YouTube, yt-dlp needs to solve JavaScript challenges presented by YouTube using an external JavaScript runtime.

See more details: https://github.com/yt-dlp/yt-dlp/wiki/EJS

```shell
sudo apt install ffmpeg -y

ffmpeg -version
```

## Setup

### Installation

// to do

### Install using **uv**

If you have `uv` installed, run the following command to install `yt-dlp` as a uv tool.
```shell
uv tool install yt-dlp
```


```shell
mise install
```


- https://github.com/yt-dlp/ejs
-
