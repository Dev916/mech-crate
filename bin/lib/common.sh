#!/bin/bash
#
# MechCrate Common Utilities
# Shared colors, helpers, and utility functions
#

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Crate Raccoon ASCII art
raccoon() {
    echo -e "${CYAN}"
    cat << 'EOF'
    ╭──────────────────────────────────╮
    │  🦝 Crate Raccoon                │
    │     is unpacking your stack...   │
    ╰──────────────────────────────────╯
EOF
    echo -e "${NC}"
}

# Print styled messages
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Prompt for yes/no with default
prompt_yn() {
    local prompt="$1"
    local default="${2:-n}"
    local response
    
    if [[ "$default" == "y" ]]; then
        read -r -p "$prompt [Y/n]: " response
        [[ -z "$response" || "$response" =~ ^[Yy] ]]
    else
        read -r -p "$prompt [y/N]: " response
        [[ "$response" =~ ^[Yy] ]]
    fi
}

# Check if a directory is a MechCrate project root
is_mech_crate_project_dir() {
    local dir="${1:-.}"
    [[ -f "$dir/Makefile" && -d "$dir/docker" && -d "$dir/make" && -d "$dir/scripts" ]]
}

# Check if we're in a MechCrate project (current directory)
is_mech_crate_project() {
    is_mech_crate_project_dir "."
}

# Find the project root by walking up the directory tree
# Returns the path to project root, or empty string if not found
find_project_root() {
    local dir="$(pwd)"
    
    while [[ "$dir" != "/" ]]; do
        if is_mech_crate_project_dir "$dir"; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    
    return 1
}

# Change to project root directory
# Dies with error if not in a project
cd_to_project_root() {
    local project_root
    project_root=$(find_project_root)
    
    if [[ -z "$project_root" ]]; then
        error "Not in a MechCrate project. Run 'mx new <name>' first."
    fi
    
    cd "$project_root" || error "Failed to change to project root: $project_root"
}

# Proxy commands to Make when in project
proxy_to_make() {
    cd_to_project_root
    make "$@"
}
