# Requirements

To run the midnite-archive scripts, you need to install the following tools. The scripts use yt-dlp with External JavaScript (EJS) support for downloading YouTube content.

- https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#dependencies

## Prerequisites

- **yt-dlp** - YouTube downloader with EJS support
- **Deno** - JavaScript runtime required for yt-dlp's EJS functionality
- **ffmpeg** - Media processing library

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
