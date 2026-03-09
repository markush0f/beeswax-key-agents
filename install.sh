#!/bin/bash
set -e

echo "======================================"
echo "   Vault Secret Scanner (vss-can) Installer"
echo "======================================"
echo ""

# Color setup
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

REPO="markush0f/vault-secret-scanner"

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
    MINGW*|MSYS*|CYGWIN*)
        echo -e "${RED}Windows detected via Git Bash / MSYS2 / Cygwin.${NC}"
        echo ""
        echo "Please use the PowerShell installer instead:"
        echo ""
        echo -e "  ${CYAN}irm https://raw.githubusercontent.com/markush0f/vault-secret-scanner/main/install.ps1 | iex${NC}"
        echo ""
        echo "Open PowerShell (Win + X -> Terminal) and run the command above."
        exit 1
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

ASSET_NAME="vss-can-${ARCH_NAME}-${OS_NAME}.tar.gz"
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

echo -e "4> ${CYAN}Installing 'vss-can' to $INSTALL_DIR...${NC}"
mv vss-can "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/vss-can"

# Cleanup
cd ~
rm -rf "$TMP_DIR"

echo ""
echo -e "${GREEN}SUCCESS! Agent Key Detector has been installed.${NC}"
echo -e "The executable is located at: ${CYAN}$INSTALL_DIR/vss-can${NC}"
echo ""

# Check if INSTALL_DIR is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${CYAN}Adding $INSTALL_DIR to your PATH automatically...${NC}"
    
    # Try to detect which shell config to use
    SHELL_RC=""
    if [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    else
        SHELL_RC="$HOME/.profile"
    fi

    if [ -n "$SHELL_RC" ]; then
        echo -e "\nexport PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"
        echo -e "${GREEN}Successfully added PATH to $SHELL_RC${NC}"
        echo "Please restart your terminal or run: source $SHELL_RC"
    else
        echo -e "${RED}WARNING: Could not detect your shell configuration file.${NC}"
        echo "Please add this line manually:"
        echo -e "${CYAN}export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
    fi
    echo ""
fi

echo "You can now run the detector by typing:"
echo -e "${CYAN}vss-can${NC} (or $INSTALL_DIR/vss-can)"
