#!/bin/bash
set -e

echo "======================================"
echo "   Vault Secret Scanner (bkad) Installer"
echo "======================================"
echo ""

# Color setup
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

REPO="markush0f/beeswax-key-agents"

# 1. Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        OS_NAME="linux"
        ;;
    Darwin)
        OS_NAME="macos"
        ;;
    *)
        echo -e "${RED}Error: Unsupported operating system ($OS).${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_NAME="x86_64"
        ;;
    aarch64|arm64)
        ARCH_NAME="aarch64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture ($ARCH).${NC}"
        exit 1
        ;;
esac

if [ "$OS_NAME" = "linux" ] && [ "$ARCH_NAME" = "aarch64" ]; then
    echo -e "${RED}Error: Linux ARM64 binaries are not yet compiled statically on Github releases.${NC}"
    echo "Please build directly from source using cargo install."
    exit 1
fi

ASSET_NAME="bkad-${ARCH_NAME}-${OS_NAME}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$ASSET_NAME"

echo -e "1> ${CYAN}Detecting system: $OS $ARCH${NC}"
echo -e "2> ${CYAN}Downloading $ASSET_NAME from GitHub Releases...${NC}"

# Temporary space
TMP_DIR=$(mktemp -d)

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET_NAME"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O "$TMP_DIR/$ASSET_NAME"
else
    echo -e "${RED}Error: curl or wget is required to download the binary.${NC}"
    exit 1
fi

echo -e "3> ${CYAN}Extracting executable...${NC}"
cd "$TMP_DIR"
tar -xzf "$ASSET_NAME"

# Determine install location (local user bin to avoid sudo prompts)
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo -e "4> ${CYAN}Installing 'bkad' to $INSTALL_DIR...${NC}"
mv bkad "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/bkad"

# Cleanup
cd ~
rm -rf "$TMP_DIR"

echo ""
echo -e "${GREEN}SUCCESS! Agent Key Detector has been installed.${NC}"
echo -e "The executable is located at: ${CYAN}$INSTALL_DIR/bkad${NC}"
echo ""

# Check if INSTALL_DIR is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${RED}WARNING: $INSTALL_DIR is not in your PATH.${NC}"
    echo "To use 'bkad' from anywhere, you must add this line to your ~/.bashrc or ~/.zshrc:"
    echo -e "${CYAN}export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
    echo "Then restart your terminal or run: source ~/.bashrc"
    echo ""
fi

echo "You can now run the detector by typing:"
echo -e "${CYAN}bkad${NC} (or $INSTALL_DIR/bkad)"
