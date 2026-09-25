#!/usr/bin/env bash
# Install the midnite-archive release binary. Linux x86_64 only.
set -euo pipefail

REPO="lbrealdev/midnite-archive"
BIN_NAME="midnite-archive"
LATEST_URL="https://github.com/${REPO}/releases/latest"

usage() {
  cat <<EOF
Usage: ma-installer.sh [--help] [--version]

Download the latest ${BIN_NAME} release binary and install it to
\${PREFIX:-\$HOME/.local/bin}.

Linux x86_64 only. This script installs the CLI binary and nothing else.

Options:
  --help       Show this help and exit
  --version    Print the latest release version and exit
EOF
}

die() {
  printf 'Error: %s\n' "$1" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

resolve_version() {
  local effective tag version
  effective="$(curl -fsSL -o /dev/null -w '%{url_effective}' "$LATEST_URL")"
  tag="${effective##*/}"
  tag="${tag%%\?*}"
  version="${tag#v}"
  if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.]+)?$ ]]; then
    die "could not resolve the latest release from ${effective}"
  fi
  printf '%s\n' "$version"
}

require_linux_x86_64() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  if [[ "$os" != "Linux" || "$arch" != "x86_64" ]]; then
    die "ma-installer.sh supports Linux x86_64 only (found ${os} ${arch})"
  fi
}

on_path() {
  local dir="$1"
  case ":${PATH}:" in
    *":${dir}:"*) return 0 ;;
    *) return 1 ;;
  esac
}

install_binary() {
  local version dest_dir url tmp
  version="$(resolve_version)"
  dest_dir="${PREFIX:-${HOME}/.local/bin}"
  url="https://github.com/${REPO}/releases/download/v${version}/${BIN_NAME}-${version}.tar.gz"
  tmp="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf $(printf '%q' "$tmp")" RETURN

  curl -fsSL -o "${tmp}/archive.tar.gz" "$url"
  tar -xzf "${tmp}/archive.tar.gz" -C "$tmp" "$BIN_NAME"
  mkdir -p "$dest_dir"
  install -m 755 "${tmp}/${BIN_NAME}" "${dest_dir}/${BIN_NAME}"

  printf 'Installed %s %s to %s\n' "$BIN_NAME" "$version" "${dest_dir}/${BIN_NAME}"
  if ! on_path "$dest_dir"; then
    # shellcheck disable=SC2016 # $PATH is literal text for the user's shell
    printf 'Add it to PATH:\nexport PATH="%s:$PATH"\n' "$dest_dir"
  fi
}

main() {
  case "${1:-}" in
    --help | -h)
      usage
      ;;
    --version)
      need curl
      resolve_version
      ;;
    "")
      need curl
      need tar
      need install
      require_linux_x86_64
      install_binary
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
}

main "$@"
