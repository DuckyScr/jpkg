#!/usr/bin/env bash
set -e

# ---------------------------
# Configuration
# ---------------------------
REPO="DuckyScr/jpkg"
BIN_NAME="jpkg"
TMPDIR=$(mktemp -d)

# Detect OS
OS_TYPE=$(uname -s)
if [[ "$OS_TYPE" == "Linux" ]]; then
    ARCHIVE="${BIN_NAME}-linux-x86_64.tar.xz"
elif [[ "$OS_TYPE" == "Darwin" ]]; then
    ARCHIVE="${BIN_NAME}-macos-x86_64.tar.xz"
else
    echo "Unsupported OS: $OS_TYPE"
    exit 1
fi

# ---------------------------
# Download latest release
# ---------------------------
echo "Downloading latest nightly release for $OS_TYPE..."
DOWNLOAD_URL=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep browser_download_url \
    | grep "$ARCHIVE" \
    | cut -d '"' -f 4)

curl -L -o "${TMPDIR}/${ARCHIVE}" "$DOWNLOAD_URL"

# ---------------------------
# Extract and install
# ---------------------------
echo "Extracting..."
tar -xJf "${TMPDIR}/${ARCHIVE}" -C "${TMPDIR}"

echo "Installing to /usr/local/bin..."
sudo mv "${TMPDIR}/${BIN_NAME}" /usr/local/bin/
sudo chmod +x /usr/local/bin/${BIN_NAME}

echo "Cleaning up..."
rm -rf "${TMPDIR}"

echo "✅ Installed ${BIN_NAME} successfully!"
