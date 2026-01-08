#!/bin/bash

#awk '/^#/{gsub("# ", ""); comment=$0} /^[^#].*:/ && !/help/ && !/@awk/ {gsub(":", ""); printf "\033[0;36m%-20s\033[0m : \033[0;35m%s\033[0m\n", $0, comment}' Makefile

# Define the directory where makefiles are stored
MAKEFILES_DIR="make"
COMMON_MK="$MAKEFILES_DIR/common.mk"

# Function to display help from a makefile
display_help() {
    local mkfile=$1
    awk '
    /^\.PHONY/ {next}
    /^#/{gsub("# ", ""); comment=$0}
    /^[^#].*:/ && !/help/ && !/@awk/ && !/^_/ && !/\$\(shell/ {
        gsub(":", ""); printf "\033[0;36m%-20s\033[0m : \033[0;35m%s\033[0m\n", $1, comment
    }
    ' "$mkfile"
}

# Process the main Makefile
display_help Makefile

# Process each makefile in the directory
for mkfile in $MAKEFILES_DIR/*.mk; do
    if [ "$mkfile" != "$COMMON_MK" ]; then
        display_help "$mkfile"
    fi
done
