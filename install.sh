#!/usr/bin/env bash
set -e

# Configuration
REPO="DuckyScr/jpkg"
BIN_NAME="jpkg"
TMPDIR=$(mktemp -d)
ARCHIVE="${BIN_NAME}-linux-x86_64.tar.xz"

echo "Downloading latest nightly release..."
curl -sL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep browser_download_url \
  | grep "${ARCHIVE}" \
  | cut -d '"' -f 4 \
  | xargs curl -L -o "${TMPDIR}/${ARCHIVE}"

echo "Extracting..."
tar -xJf "${TMPDIR}/${ARCHIVE}" -C "${TMPDIR}"

echo "Installing to /usr/local/bin..."
sudo mv "${TMPDIR}/${BIN_NAME}" /usr/local/bin/
sudo chmod +x /usr/local/bin/${BIN_NAME}

echo "Installed ${BIN_NAME} successfully!"
rm -rf "${TMPDIR}"
