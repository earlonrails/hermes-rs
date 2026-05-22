#!/usr/bin/env bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=======================================${NC}"
echo -e "${BLUE}      Athena Installation Script    ${NC}"
echo -e "${BLUE}=======================================${NC}"
echo ""

# 1. Check for Rust and Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust and Cargo are not installed on this system.${NC}"
    echo -n "Would you like to install Rust via rustup? (y/n) "
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        echo -e "${GREEN}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Source the cargo env for the remainder of this script
        source "$HOME/.cargo/env"
    else
        echo -e "${RED}Installation aborted. Rust is required to compile Athena.${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}Rust/Cargo is already installed.${NC}"
fi

# 2. Determine installation directory
INSTALL_DIR=${INSTALL_DIR:-"$HOME/.athena"}
REPO_URL="https://github.com/earlonrails/athena.git" # Update with actual URL

if [ -d "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}Athena directory already exists at $INSTALL_DIR.${NC}"
    echo -e "Pulling latest changes..."
    cd "$INSTALL_DIR"
    git pull origin main
else
    echo -e "${GREEN}Cloning Athena repository...${NC}"
    git clone "$REPO_URL" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# 3. Build and Install the CLI
echo -e "${GREEN}Compiling Athena (this may take a few minutes)...${NC}"
cargo install --path athena-cli --locked

# 4. Final instructions
echo ""
echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}    Athena successfully installed!  ${NC}"
echo -e "${GREEN}=======================================${NC}"
echo ""
echo -e "The 'athena' command-line tool has been installed to your cargo bin directory (usually ~/.cargo/bin)."
echo -e "Make sure this directory is in your system's PATH."
echo ""
echo -e "To get started, simply run:"
echo -e "  ${YELLOW}athena --help${NC}"
echo ""
