# yt-dlp Rust Integration

Research note for replacing the direct `std::process::Command` calls in
`src/yt_dlp.rs` with a maintained Rust API while keeping yt-dlp as the download
engine.

## Goals

- Preserve yt-dlp's broad site support and frequent upstream fixes.
- Move subprocess orchestration, JSON metadata, and progress handling behind a
  typed asynchronous Rust API.
- Keep one `midnite-archive` binary for the CLI and a future TUI.
- Retain the current features: channel lists, title filters, download archives,
  comments, EJS/Deno arguments, metadata, thumbnails, and custom output paths.
- Prepare for Linux, macOS, and Windows releases.

Neither candidate is a pure-Rust replacement for yt-dlp. Both wrap the yt-dlp
executable. That is desirable here: replacing yt-dlp's extractors would make
midnite-archive responsible for tracking frequent platform changes.

## Candidates

### `yt-dlp` (`boul2gom/yt-dlp`)

- Crate: <https://crates.io/crates/yt-dlp>
- Source: <https://github.com/boul2gom/yt-dlp>
- Reviewed version: `2.7.2`
- License: `GPL-3.0-only`

An asynchronous, feature-rich yt-dlp and ffmpeg wrapper. It can download both
executables automatically, exposes generic and YouTube-specific extractors, and
offers structured metadata, download hooks, caches, retries, and format
selection.

**Strengths**

- Manages yt-dlp and ffmpeg installation.
- Generic extractor supports sites beyond YouTube.
- YouTube-specific channel, handle, playlist, and search APIs.
- Hooks and statistics are useful foundations for a future TUI.
- Active releases and a relatively broad API.

**Concerns**

- GPL-3.0-only is strong copyleft. Distributing a binary that links this crate
  requires midnite-archive to use GPL-compatible terms and provide corresponding
  source.
- The abstraction is large and brings features midnite-archive may not need.
- Current midnite-archive behavior depends on low-level yt-dlp flags
  (`--download-archive`, `--write-comments`, `--match-title`,
  `--remote-components`, and `--js-runtimes`). API parity must be proven before
  migration.
- Adopting its downloader API would couple the application closely to one
  wrapper.

### `ytd-rs`

- Crate: <https://crates.io/crates/ytd-rs>
- Source: <https://github.com/narrrl/ytd-rs>
- Reviewed version: `0.2.1`
- License: repository README states `MIT`; crates.io reports the license field
  as non-standard, so package metadata should be clarified upstream before
  adoption.

A smaller asynchronous wrapper with a builder API, structured metadata, custom
yt-dlp argument passthrough, line-by-line process output, and binary streaming.
yt-dlp must already be available on `PATH`.

**Strengths**

- Custom arguments preserve access to all current yt-dlp features.
- Streaming process output is a direct path to CLI and TUI progress events.
- Small API and dependency footprint.
- MIT terms are compatible with a permissively licensed midnite-archive, once
  the crates.io metadata discrepancy is resolved.
- Keeps dependency installation separate from download orchestration.

**Concerns**

- Does not manage yt-dlp or ffmpeg installation.
- Much smaller project and ecosystem than yt-dlp itself.
- Version `0.2.x` indicates a young API with potential breaking changes.
- Progress is raw yt-dlp output; midnite-archive must parse it into stable,
  typed events.
- Automatic EJS/Deno setup and all current archive/comments workflows need
  integration tests.

## Comparison

| Requirement | `boul2gom/yt-dlp` | `ytd-rs` |
|-------------|-------------------|----------|
| Async API | Yes | Yes |
| Structured metadata | Yes | Yes |
| Generic sites | Yes | Via yt-dlp arguments |
| Raw argument passthrough | Must be verified | Yes |
| Progress foundation | Typed hooks | Streamed output lines |
| Installs yt-dlp/ffmpeg | Yes | No |
| Current archive/comments parity | Must be proven | Likely via custom args; must be proven |
| License impact | Requires GPL-compatible project distribution | MIT stated upstream; metadata needs confirmation |
| Integration size | Larger, opinionated API | Smaller wrapper |

## Recommendation

Start with a time-boxed integration spike for `ytd-rs`, behind a
midnite-archive-owned backend boundary. It best preserves the current yt-dlp
flags, offers progress streaming for a future TUI, and avoids committing the
whole project to GPL before API fit is known.

**PoC status (2026-07-14):** completed in [`poc/`](../poc/README.md).
Decision: **use `ytd-rs`**. Local maintainer run confirmed full download + EJS/Deno
+ arbitrary args; `boul2gom/yt-dlp` reported success with empty output / zero
progress. See [`poc/RESULTS.md`](../poc/RESULTS.md). `boul2gom/yt-dlp` 2.7.x
currently fails crates.io resolution (`lofty` yanked); PoC used `=2.1.0`.

**Pre-migration baseline (2026-07-23):** compile time, binary size, and dependency
weight for current `midnite-archive` vs both PoCs are recorded in
[`poc/BASELINE.md`](../poc/BASELINE.md). Re-run after the production adapter lands.

Do not expose `ytd-rs` types from the CLI or domain modules. Define
midnite-archive request, result, and progress types, then adapt internally:

```text
CLI / future TUI
        |
        v
midnite-archive service API
        |
        v
YtDlpBackend
        |
        +-- ytd-rs (selected)
```

Keep `boul2gom/yt-dlp` as historical comparison only unless crates.io latest
resolves, raw-arg parity is proven, false-success download behavior is fixed,
and the project explicitly accepts GPL-3.0.

## Spike acceptance criteria

The preferred candidate must demonstrate all of the following before replacing
`src/yt_dlp.rs`:

1. Generate a flat channel list and apply `--match-title`.
2. Download a single URL and a batch file with the existing output template.
3. Preserve `--download-archive` behavior.
4. Download comments without media.
5. Pass EJS remote components and the Deno runtime.
6. Surface cancellation-safe, line-by-line progress suitable for both CLI and
   future TUI consumers.
7. Produce actionable errors with subprocess exit status and stderr.
8. Work with externally installed yt-dlp/ffmpeg on Linux, with a documented
   path for macOS and Windows packaging.
9. Confirm the selected crate's license metadata and compatibility before
   merging the dependency.

## Out of scope

- Replacing yt-dlp's extractors with a pure-Rust YouTube implementation.
- Adding the TUI itself.
- Shipping a second binary.
- Implementing automatic dependency installation before the wrapper choice is
  validated.
