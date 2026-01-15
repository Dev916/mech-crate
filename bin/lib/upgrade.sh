#!/bin/bash
#
# MechCrate Upgrade Command
# Update project with latest scaffolding
#
# Architecture: Discovery-based upgrade with declarative rules
# - Automatically discovers template files (no hardcoded lists)
# - Categorizes files by upgrade behavior
# - Follows FSM pattern for state tracking
#

# ─────────────────────────────────────────────────────────────────────────────
# Upgrade Categories (declarative rules)
# ─────────────────────────────────────────────────────────────────────────────
#
# TOOLING: Prompt for updates when file differs from template
#   - make/*.mk
#   - scripts/*.sh, scripts/*.mjs
#   - Makefile
#
# CONFIG: Add if missing, never update (user-customized)
#   - docker/.config/*
#   - docker/compose/*
#   - docker/system/*
#   - docker/dockerfiles/*
#
# CONDITIONAL: Only process if feature is enabled
#   - make/cloudflare.mk → requires infra/cloudflare/
#   - scripts/cf-*.sh → requires infra/cloudflare/
#   - infra/cloudflare/* → requires infra/cloudflare/
#

# ─────────────────────────────────────────────────────────────────────────────
# Pure Functions: File categorization (no side effects)
# ─────────────────────────────────────────────────────────────────────────────

# Categorize a file path into upgrade behavior
# Returns: tooling | config | conditional | skip
categorize_file() {
    local rel_path="$1"
    
    case "$rel_path" in
        # Tooling files - prompt for updates
        make/*.mk)
            # Cloudflare module is conditional
            if [[ "$rel_path" == "make/cloudflare.mk" ]]; then
                echo "conditional:cloudflare"
            else
                echo "tooling"
            fi
            ;;
        scripts/*.sh|scripts/*.mjs)
            # Cloudflare scripts are conditional
            if [[ "$rel_path" == scripts/cf-*.sh ]]; then
                echo "conditional:cloudflare"
            else
                echo "tooling"
            fi
            ;;
        Makefile.template)
            echo "tooling:makefile"
            ;;
        
        # Config files - add only, never update
        docker/compose/*)
            echo "config"
            ;;
        docker/config/*)
            echo "config:env"
            ;;
        docker/system/*)
            echo "config"
            ;;
        docker/dockerfiles/*)
            echo "config"
            ;;
        
        # Infrastructure templates - conditional
        infra/cloudflare/*)
            echo "conditional:cloudflare"
            ;;
        
        # Skip recipes and other non-scaffold files
        recipes/*)
            echo "skip"
            ;;
        *)
            echo "skip"
            ;;
    esac
}

# Check if a conditional feature is enabled
# Pure: only checks filesystem state
is_feature_enabled() {
    local feature="$1"
    
    case "$feature" in
        cloudflare)
            [[ -d "infra/cloudflare" ]]
            ;;
        *)
            return 1
            ;;
    esac
}

# Map template path to project path
# Handles special cases like Makefile.template → Makefile
template_to_project_path() {
    local template_rel="$1"
    
    case "$template_rel" in
        Makefile.template)
            echo "Makefile"
            ;;
        docker/config/*)
            # env.app → .env.app
            local filename
            filename=$(basename "$template_rel")
            local target_name="${filename/env./.env.}"
            echo "docker/.config/$target_name"
            ;;
        *)
            echo "$template_rel"
            ;;
    esac
}

# ─────────────────────────────────────────────────────────────────────────────
# Effect Functions: File operations (side effects isolated here)
# ─────────────────────────────────────────────────────────────────────────────

# Check if file differs from template
file_differs() {
    local project_file="$1"
    local template_file="$2"
    
    [[ -f "$project_file" ]] && [[ -f "$template_file" ]] && \
        ! diff -q "$project_file" "$template_file" &>/dev/null
}

# Show diff between files
show_diff() {
    local project_file="$1"
    local template_file="$2"
    
    echo -e "${CYAN}─────────────────────────────────────────────────────${NC}"
    echo -e "${BOLD}Changes in $project_file:${NC}"
    echo -e "${CYAN}─────────────────────────────────────────────────────${NC}"
    
    if command -v colordiff &>/dev/null; then
        diff -u "$project_file" "$template_file" | colordiff | head -50
    else
        diff -u "$project_file" "$template_file" | head -50
    fi
    
    echo ""
}

# Copy file with appropriate permissions
copy_file() {
    local src="$1"
    local dst="$2"
    local dry_run="${3:-false}"
    
    if [[ "$dry_run" == "true" ]]; then
        info "[DRY RUN] Would copy: $src → $dst"
        return 0
    fi
    
    # Ensure parent directory exists
    local parent_dir
    parent_dir=$(dirname "$dst")
    [[ -d "$parent_dir" ]] || mkdir -p "$parent_dir"
    
    cp "$src" "$dst"
    
    # Make shell scripts executable
    if [[ "$dst" == *.sh ]]; then
        chmod +x "$dst"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Discovery: Walk templates and build upgrade plan
# ─────────────────────────────────────────────────────────────────────────────

# Discover all template files and their upgrade actions
# Output format: action|project_path|template_path
discover_upgrades() {
    local templates_dir="$1"
    
    # Walk template directories (excluding recipes)
    while IFS= read -r -d '' template_file; do
        # Get path relative to templates dir
        local rel_path="${template_file#"$templates_dir"/}"
        
        # Skip directories
        [[ -d "$template_file" ]] && continue
        
        # Categorize the file
        local category
        category=$(categorize_file "$rel_path")
        
        # Skip files marked for skipping
        [[ "$category" == "skip" ]] && continue
        
        # Check conditional features
        if [[ "$category" == conditional:* ]]; then
            local feature="${category#conditional:}"
            if ! is_feature_enabled "$feature"; then
                continue
            fi
            # Treat enabled conditional as tooling
            category="tooling"
        fi
        
        # Map to project path
        local project_path
        project_path=$(template_to_project_path "$rel_path")
        
        # Determine action based on category and file state
        local action
        if [[ ! -f "$project_path" ]]; then
            action="add"
        elif [[ "$category" == "tooling" || "$category" == "tooling:makefile" ]]; then
            if file_differs "$project_path" "$template_file"; then
                action="update"
            else
                action="current"
            fi
        else
            # Config files: never update
            action="skip"
        fi
        
        # Output the upgrade entry
        echo "${action}|${project_path}|${template_file}"
        
    done < <(find "$templates_dir" \
        -type f \
        ! -path "*/recipes/*" \
        -print0 2>/dev/null)
}

# ─────────────────────────────────────────────────────────────────────────────
# FSM: Upgrade state machine
# ─────────────────────────────────────────────────────────────────────────────
#
# States: Init → Discovering → Processing → Prompting → Complete
# Events: START, DISCOVERED, PROCESSED, PROMPTED, DONE
#

UPGRADE_STATE="Init"

transition() {
    local event="$1"
    local to="$2"
    local from="$UPGRADE_STATE"
    
    # Structured log for observability (debug level)
    [[ "${DEBUG:-}" == "true" ]] && \
        echo -e "${MAGENTA}[FSM]${NC} $from --($event)--> $to" >&2
    
    UPGRADE_STATE="$to"
}

# ─────────────────────────────────────────────────────────────────────────────
# Resolve templates directory
# ─────────────────────────────────────────────────────────────────────────────

# Get templates directory, resolving from script location if not injected
get_templates_dir() {
    # If TEMPLATES_DIR is set (by parent mx script), use it
    if [[ -n "${TEMPLATES_DIR:-}" ]]; then
        echo "$TEMPLATES_DIR"
        return 0
    fi
    
    # Otherwise, resolve from this script's location
    local source="${BASH_SOURCE[0]}"
    local script_dir
    script_dir="$(cd -P "$(dirname "$source")" && pwd)"
    local mech_crate_root
    mech_crate_root="$(dirname "$(dirname "$script_dir")")"
    
    echo "$mech_crate_root/templates"
}

# ─────────────────────────────────────────────────────────────────────────────
# Main Upgrade Workflow
# ─────────────────────────────────────────────────────────────────────────────

upgrade_project() {
    local show_diff_flag=false
    local auto_yes=false
    local dry_run=false
    local templates_dir
    templates_dir=$(get_templates_dir)
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --diff|-d)
                show_diff_flag=true
                shift
                ;;
            --yes|-y)
                auto_yes=true
                shift
                ;;
            --dry-run|-n)
                dry_run=true
                shift
                ;;
            --debug)
                DEBUG=true
                shift
                ;;
            --help|-h)
                cat << 'EOF'
mx upgrade - Update project with latest MechCrate scaffolding

USAGE:
    mx upgrade [options]

OPTIONS:
    --diff, -d      Show diff for each changed file
    --yes, -y       Auto-accept all updates (non-interactive)
    --dry-run, -n   Show what would be done without making changes
    --debug         Show FSM state transitions
    --help, -h      Show this help

BEHAVIOR:
    Discovery-based upgrade that automatically finds all template files:
    
    • TOOLING (make/*.mk, scripts/*.sh, Makefile)
      → Missing files are added automatically
      → Changed files prompt for update
    
    • CONFIG (docker/compose/*, docker/.config/*)
      → Missing files are added automatically
      → Existing files are never modified (user-customized)
    
    • CONDITIONAL (cloudflare files)
      → Only processed if feature is enabled (infra/cloudflare/ exists)

EXAMPLES:
    mx upgrade              # Interactive upgrade
    mx upgrade --diff       # Show diffs before prompting
    mx upgrade --yes        # Auto-accept all updates
    mx upgrade --dry-run    # Preview changes only
EOF
                return 0
                ;;
            *)
                shift
                ;;
        esac
    done
    
    # ─────────────────────────────────────────────────────────────────────────
    # FSM: Init → Discovering
    # ─────────────────────────────────────────────────────────────────────────
    
    transition "START" "Discovering"
    
    # Find and change to project root
    cd_to_project_root
    
    raccoon
    info "Upgrading MechCrate project..."
    echo ""
    
    # ─────────────────────────────────────────────────────────────────────────
    # FSM: Discovering → Processing
    # ─────────────────────────────────────────────────────────────────────────
    
    transition "DISCOVERED" "Processing"
    
    # Ensure required directories exist
    local required_dirs=("make" "scripts" "docker/.config" "docker/compose" "docker/system" "docker/dockerfiles" "tmp/up")
    for dir in "${required_dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            if [[ "$dry_run" == "true" ]]; then
                info "[DRY RUN] Would create directory: $dir"
            else
                mkdir -p "$dir"
                success "Created directory: $dir"
            fi
        fi
    done
    
    # Discover all upgrades
    local -a update_files=()
    local added_count=0
    local updated_count=0
    local skipped_count=0
    
    echo ""
    echo -e "${BOLD}Scanning templates...${NC}"
    
    while IFS='|' read -r action project_path template_path; do
        [[ -z "$action" ]] && continue
        
        case "$action" in
            add)
                # Add missing file
                if [[ "$dry_run" == "true" ]]; then
                    info "[DRY RUN] Would add: $project_path"
                else
                    copy_file "$template_path" "$project_path" "$dry_run"
                    success "Added: $project_path"
                fi
                ((added_count++))
                ;;
            update)
                # Queue for update prompt
                update_files+=("${project_path}|${template_path}")
                ;;
            current)
                # Already up to date
                ;;
            skip)
                # Explicitly skipped (config file exists)
                ;;
        esac
    done < <(discover_upgrades "$templates_dir")
    
    # ─────────────────────────────────────────────────────────────────────────
    # FSM: Processing → Prompting (if updates pending)
    # ─────────────────────────────────────────────────────────────────────────
    
    if [[ ${#update_files[@]} -gt 0 ]]; then
        transition "PROCESSED" "Prompting"
        
        echo ""
        echo -e "${CYAN}╭────────────────────────────────────────────────────────────╮${NC}"
        echo -e "${CYAN}│${NC}  ${BOLD}📝 Tooling Updates Available${NC}                              ${CYAN}│${NC}"
        echo -e "${CYAN}╰────────────────────────────────────────────────────────────╯${NC}"
        echo ""
        echo -e "The following tooling files have updates available:"
        echo ""
        
        for entry in "${update_files[@]}"; do
            local project_file="${entry%%|*}"
            echo "    • $project_file"
        done
        
        echo ""
        
        if [[ "$dry_run" == "true" ]]; then
            info "[DRY RUN] Would prompt to update ${#update_files[@]} file(s)"
        else
            for entry in "${update_files[@]}"; do
                local project_file="${entry%%|*}"
                local template_file="${entry##*|}"
                
                echo ""
                echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
                echo -e "${BOLD}$project_file${NC}"
                echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
                
                if [[ "$show_diff_flag" == "true" ]]; then
                    show_diff "$project_file" "$template_file"
                fi
                
                local update_file=false
                
                if [[ "$auto_yes" == "true" ]]; then
                    update_file=true
                else
                    echo ""
                    echo -e "Options:"
                    echo "    [y] Yes, update this file"
                    echo "    [n] No, skip this file"
                    echo "    [d] Show diff first"
                    echo "    [a] Accept all remaining updates"
                    echo "    [q] Quit (skip all remaining)"
                    echo ""
                    
                    while true; do
                        read -r -p "Update $project_file? [y/n/d/a/q]: " response
                        case "$response" in
                            y|Y)
                                update_file=true
                                break
                                ;;
                            n|N)
                                break
                                ;;
                            d|D)
                                show_diff "$project_file" "$template_file"
                                ;;
                            a|A)
                                auto_yes=true
                                update_file=true
                                break
                                ;;
                            q|Q)
                                info "Skipping remaining updates."
                                update_files=()
                                break 2
                                ;;
                            *)
                                echo "Invalid option. Please enter y, n, d, a, or q."
                                ;;
                        esac
                    done
                fi
                
                if [[ "$update_file" == "true" ]]; then
                    # Backup original file
                    cp "$project_file" "$project_file.bak"
                    cp "$template_file" "$project_file"
                    
                    # Preserve execute permission for scripts
                    if [[ "$project_file" == *.sh ]]; then
                        chmod +x "$project_file"
                    fi
                    
                    success "Updated: $project_file (backup: $project_file.bak)"
                    ((updated_count++))
                else
                    warn "Skipped: $project_file"
                    ((skipped_count++))
                fi
            done
        fi
    fi
    
    # ─────────────────────────────────────────────────────────────────────────
    # FSM: → Complete
    # ─────────────────────────────────────────────────────────────────────────
    
    transition "DONE" "Complete"
    
    echo ""
    echo -e "${CYAN}╭────────────────────────────────────────────────────────────╮${NC}"
    echo -e "${CYAN}│${NC}  ${BOLD}📊 Upgrade Summary${NC}                                        ${CYAN}│${NC}"
    echo -e "${CYAN}╰────────────────────────────────────────────────────────────╯${NC}"
    echo ""
    
    if [[ "$dry_run" == "true" ]]; then
        echo -e "  ${BLUE}[DRY RUN]${NC} No changes were made"
        echo ""
    fi
    
    if [[ $added_count -gt 0 ]]; then
        echo -e "  ${GREEN}✓${NC} Added:   $added_count file(s)"
    fi
    
    if [[ $updated_count -gt 0 ]]; then
        echo -e "  ${GREEN}✓${NC} Updated: $updated_count file(s)"
    fi
    
    if [[ $skipped_count -gt 0 ]]; then
        echo -e "  ${YELLOW}○${NC} Skipped: $skipped_count file(s)"
    fi
    
    if [[ $added_count -eq 0 && $updated_count -eq 0 && $skipped_count -eq 0 && ${#update_files[@]} -eq 0 ]]; then
        echo -e "  ${GREEN}✓${NC} Project is up to date!"
    fi
    
    echo ""
    echo -e "${CYAN}🦝 Crate Raccoon says: Your tooling is fresh!${NC}"
}
