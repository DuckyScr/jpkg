#!/usr/bin/env bash

# install.sh – one‑liner installer for jpkg (macOS / Linux)
# -------------------------------------------------------
# This script downloads the latest release binary from GitHub,
# installs it to /usr/local/bin/jpkg and creates the cache directory.

set -euo pipefail

# Determine OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map Rust target triples
case "$ARCH" in
  x86_64) TARGET="x86_64-unknown-linux-gnu";;
  aarch64|arm64) TARGET="aarch64-unknown-linux-gnu";;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1;;
esac

# Build download URL (latest release)
REPO="DuckyScr/jpkg"
URL="https://api.github.com/repos/$REPO/releases/latest"

# Get asset download URL for the appropriate binary
ASSET_URL=$(curl -sL "$URL" | grep "browser_download_url" | grep "$TARGET" | cut -d '"' -f 4)
if [ -z "$ASSET_URL" ]; then
  echo "Could not find a binary for $OS/$TARGET in the latest release" >&2
  exit 1
fi

# Download and extract
TMPDIR=$(mktemp -d)
cd "$TMPDIR"
curl -sL "$ASSET_URL" -o jpkg.tar.gz
tar -xzf jpkg.tar.gz
# Assume the binary is named 'jpkg' inside the archive
chmod +x jpkg
sudo mv jpkg /usr/local/bin/jpkg

# Create cache directory
mkdir -p "$HOME/.jpkg/cache"

echo "✅ jpkg installed to /usr/local/bin/jpkg"
