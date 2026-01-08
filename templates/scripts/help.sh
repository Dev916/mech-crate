#!/bin/bash
# Display help from Makefile comments
# Usage: ./scripts/help.sh

# Define the directory where makefiles are stored
MAKEFILES_DIR="make"
COMMON_MK="$MAKEFILES_DIR/common.mk"

# Function to display help from a makefile
# Looks for lines with ## comments (standard help format)
display_help() {
    local mkfile=$1
    awk '
    /^\.PHONY/ {next}
    /^[a-zA-Z_-]+:.*##/ {
        # Parse "target: deps ## description" format
        split($0, parts, /##/)
        target = parts[1]
        desc = parts[2]
        # Clean up target (remove : and deps)
        gsub(/:.*/, "", target)
        gsub(/^[ \t]+|[ \t]+$/, "", target)
        gsub(/^[ \t]+|[ \t]+$/, "", desc)
        if (target != "" && desc != "") {
            printf "\033[0;36m%-20s\033[0m : \033[0;35m%s\033[0m\n", target, desc
        }
    }
    ' "$mkfile"
}

echo ""
echo -e "\033[1m🦝 MechCrate Commands\033[0m"
echo ""

# Process the main Makefile
display_help Makefile

# Process each makefile in the directory
for mkfile in $MAKEFILES_DIR/*.mk; do
    if [ "$mkfile" != "$COMMON_MK" ]; then
        display_help "$mkfile"
    fi
done

echo ""
echo -e "\033[0;33mTip:\033[0m Use s=<service> to target a specific service"
echo "     Example: make dev s=app"
echo ""
