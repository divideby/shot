#!/bin/sh
set -e

REPO="divideby/shot"
INSTALL_DIR="${HOME}/.local/bin"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
    linux) PLATFORM="linux-${ARCH}" ;;
    darwin) PLATFORM="macos-${ARCH}" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

BINARY_URL="https://github.com/${REPO}/releases/latest/download/shot-${PLATFORM}"

echo "Downloading shot for ${PLATFORM}..."
mkdir -p "$INSTALL_DIR"

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$BINARY_URL" -o "${INSTALL_DIR}/shot"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$BINARY_URL" -O "${INSTALL_DIR}/shot"
else
    echo "Error: curl or wget required"
    exit 1
fi

chmod +x "${INSTALL_DIR}/shot"

echo "Installed shot to ${INSTALL_DIR}/shot"

# Check if in PATH
case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *) echo "Add ${INSTALL_DIR} to your PATH" ;;
esac
