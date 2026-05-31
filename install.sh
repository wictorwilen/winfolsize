#!/usr/bin/env bash
# WinFolSize installer for Linux and macOS.
#
# Quick install:
#   curl -fsSL https://raw.githubusercontent.com/wictorwilen/winfolsize/main/install.sh | bash
#
# Environment overrides:
#   WINFOLSIZE_VERSION  Specific version tag (e.g. v0.1.0). Defaults to latest release.
#   WINFOLSIZE_PREFIX   Install prefix. Defaults to $HOME/.local (binary in $PREFIX/bin).
#   WINFOLSIZE_REPO     GitHub repo (owner/name). Defaults to wictorwilen/winfolsize.

set -euo pipefail

REPO="${WINFOLSIZE_REPO:-wictorwilen/winfolsize}"
PREFIX="${WINFOLSIZE_PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"

info()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33mwarn:\033[0m %s\n' "$*" >&2; }
die()   { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"; }
need curl
need tar

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$os" in
  linux)  target_os="linux" ;;
  darwin) target_os="macos" ;;
  *) die "unsupported OS: $os (use install.ps1 on Windows)" ;;
esac
case "$arch" in
  x86_64|amd64) target_arch="x86_64" ;;
  aarch64|arm64) target_arch="aarch64" ;;
  *) die "unsupported architecture: $arch" ;;
esac
target="${target_os}-${target_arch}"

if [[ -n "${WINFOLSIZE_VERSION:-}" ]]; then
  tag="$WINFOLSIZE_VERSION"
else
  info "Looking up latest release of $REPO…"
  tag="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep -oE '"tag_name":\s*"[^"]+"' | head -n1 | cut -d'"' -f4)"
  [[ -n "$tag" ]] || die "could not determine latest release tag"
fi
version="${tag#v}"

asset="winfolsize-${version}-${target}.tar.gz"
url="https://github.com/$REPO/releases/download/$tag/$asset"
info "Downloading $asset"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
if ! curl -fsSL "$url" -o "$tmp/$asset"; then
  die "failed to download $url

This installer expects a release asset named '$asset'.
If the project does not yet publish a $target build, install from source:

    cargo install --git https://github.com/$REPO --locked"
fi

info "Extracting…"
tar -xzf "$tmp/$asset" -C "$tmp"

src_bin="$tmp/winfolsize"
[[ -f "$src_bin" ]] || die "expected 'winfolsize' binary in archive"

mkdir -p "$BIN_DIR"
install -m 0755 "$src_bin" "$BIN_DIR/winfolsize"
info "Installed to $BIN_DIR/winfolsize"

if ! command -v winfolsize >/dev/null 2>&1; then
  case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) warn "$BIN_DIR is not on your PATH. Add this to your shell rc file:"
       printf '\n    export PATH="%s:$PATH"\n\n' "$BIN_DIR" ;;
  esac
fi

"$BIN_DIR/winfolsize" --version || true
info "Done. Try: winfolsize --help"
