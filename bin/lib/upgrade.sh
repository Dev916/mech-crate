#!/bin/bash
#
# MechCrate Upgrade Command
# Update project with latest scaffolding
#

# ─────────────────────────────────────────────────────────────────────────────
# Upgrade - Update project with latest scaffolding
# ─────────────────────────────────────────────────────────────────────────────

# Known tooling files that should be prompted for updates
TOOLING_FILES=(
    "Makefile:Makefile.template"
    "scripts/.bashrc:scripts/.bashrc"
    "scripts/build.sh:scripts/build.sh"
    "scripts/dev.sh:scripts/dev.sh"
    "scripts/doctor.sh:scripts/doctor.sh"
    "scripts/down.sh:scripts/down.sh"
    "scripts/exec.sh:scripts/exec.sh"
    "scripts/help.sh:scripts/help.sh"
    "scripts/init.sh:scripts/init.sh"
    "scripts/logs.sh:scripts/logs.sh"
    "scripts/ps.sh:scripts/ps.sh"
    "scripts/restart.sh:scripts/restart.sh"
    "scripts/run.sh:scripts/run.sh"
    "scripts/sh.sh:scripts/sh.sh"
    "scripts/start.sh:scripts/start.sh"
    "scripts/stop.sh:scripts/stop.sh"
    "scripts/test.sh:scripts/test.sh"
    "scripts/up.sh:scripts/up.sh"
    "scripts/cf-setup.sh:scripts/cf-setup.sh"
    "scripts/cf-init-app.sh:scripts/cf-init-app.sh"
    "scripts/simple-release.sh:scripts/simple-release.sh"
    "scripts/release-sync-versions.mjs:scripts/release-sync-versions.mjs"
    "scripts/app-version.mjs:scripts/app-version.mjs"
)

# Make modules - always check for updates
MAKE_MODULES=(
    "make/build.mk"
    "make/cloudflare.mk"
    "make/common.mk"
    "make/dev.mk"
    "make/down.mk"
    "make/logs.mk"
    "make/release.mk"
    "make/restart.mk"
    "make/run.mk"
    "make/sh.mk"
    "make/start.mk"
    "make/stop.mk"
    "make/up.mk"
)

# Check if file differs from template
file_differs() {
    local project_file="$1"
    local template_file="$2"
    
    if [[ ! -f "$project_file" ]]; then
        return 1  # File doesn't exist, so doesn't "differ"
    fi
    
    if [[ ! -f "$template_file" ]]; then
        return 1  # Template doesn't exist
    fi
    
    ! diff -q "$project_file" "$template_file" &>/dev/null
}

# Show diff between files
show_diff() {
    local project_file="$1"
    local template_file="$2"
    
    echo -e "${CYAN}─────────────────────────────────────────────────────${NC}"
    echo -e "${BOLD}Changes in $project_file:${NC}"
    echo -e "${CYAN}─────────────────────────────────────────────────────${NC}"
    
    # Use colordiff if available, otherwise regular diff
    if command -v colordiff &>/dev/null; then
        diff -u "$project_file" "$template_file" | colordiff | head -50
    else
        diff -u "$project_file" "$template_file" | head -50
    fi
    
    echo ""
}

# Upgrade project
upgrade_project() {
    local show_diff_flag=false
    local auto_yes=false
    local dry_run=false
    
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
            --help|-h)
                echo -e "${BOLD}mx upgrade${NC} - Update project with latest MechCrate scaffolding"
                echo ""
                echo -e "${BOLD}USAGE:${NC}"
                echo "    mx upgrade [options]"
                echo ""
                echo -e "${BOLD}OPTIONS:${NC}"
                echo "    --diff, -d      Show diff for each changed file"
                echo "    --yes, -y       Auto-accept all updates (non-interactive)"
                echo "    --dry-run, -n   Show what would be done without making changes"
                echo "    --help, -h      Show this help"
                echo ""
                echo -e "${BOLD}BEHAVIOR:${NC}"
                echo "    • Missing files are added automatically"
                echo "    • Tooling files (make/*.mk, scripts/*.sh) prompt for update"
                echo "    • User config files are never touched"
                echo ""
                return 0
                ;;
            *)
                shift
                ;;
        esac
    done
    
    if ! is_mech_crate_project; then
        error "Not in a MechCrate project. Run 'mx new <name>' first."
    fi
    
    raccoon
    info "Upgrading MechCrate project..."
    echo ""
    
    local added_count=0
    local updated_count=0
    local skipped_count=0
    local pending_updates=()
    
    # ─────────────────────────────────────────────────────────────────────────
    # Phase 1: Check for missing directories
    # ─────────────────────────────────────────────────────────────────────────
    
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
    
    # ─────────────────────────────────────────────────────────────────────────
    # Phase 2: Add missing make modules
    # ─────────────────────────────────────────────────────────────────────────
    
    echo ""
    echo -e "${BOLD}Checking make modules...${NC}"
    
    for mk_file in "${MAKE_MODULES[@]}"; do
        local template_path="$TEMPLATES_DIR/$mk_file"
        local project_path="$mk_file"
        
        if [[ ! -f "$template_path" ]]; then
            continue
        fi
        
        # Special handling for cloudflare.mk - only add if infra/cloudflare exists
        if [[ "$mk_file" == "make/cloudflare.mk" && ! -d "infra/cloudflare" ]]; then
            continue
        fi
        
        if [[ ! -f "$project_path" ]]; then
            # File is missing - add it
            if [[ "$dry_run" == "true" ]]; then
                info "[DRY RUN] Would add: $project_path"
            else
                cp "$template_path" "$project_path"
                success "Added: $project_path"
            fi
            ((added_count++))
        elif file_differs "$project_path" "$template_path"; then
            # File exists but differs - queue for update prompt
            pending_updates+=("$project_path:$template_path")
        fi
    done
    
    # ─────────────────────────────────────────────────────────────────────────
    # Phase 3: Add missing scripts
    # ─────────────────────────────────────────────────────────────────────────
    
    echo ""
    echo -e "${BOLD}Checking scripts...${NC}"
    
    # Enable dotglob to include hidden files like .bashrc
    shopt -s dotglob
    for template_script in "$TEMPLATES_DIR/scripts/"*; do
        if [[ ! -f "$template_script" ]]; then
            continue
        fi
        
        local script_name=$(basename "$template_script")
        local project_path="scripts/$script_name"
        local template_path="$template_script"
        
        if [[ ! -f "$project_path" ]]; then
            # File is missing - add it
            if [[ "$dry_run" == "true" ]]; then
                info "[DRY RUN] Would add: $project_path"
            else
                cp "$template_path" "$project_path"
                [[ "$script_name" == *.sh ]] && chmod +x "$project_path"
                success "Added: $project_path"
            fi
            ((added_count++))
        elif file_differs "$project_path" "$template_path"; then
            # File exists but differs - queue for update prompt
            pending_updates+=("$project_path:$template_path")
        fi
    done
    shopt -u dotglob
    
    # ─────────────────────────────────────────────────────────────────────────
    # Phase 4: Check Makefile
    # ─────────────────────────────────────────────────────────────────────────
    
    echo ""
    echo -e "${BOLD}Checking Makefile...${NC}"
    
    local makefile_template="$TEMPLATES_DIR/Makefile.template"
    if [[ -f "$makefile_template" ]]; then
        if [[ ! -f "Makefile" ]]; then
            if [[ "$dry_run" == "true" ]]; then
                info "[DRY RUN] Would add: Makefile"
            else
                cp "$makefile_template" "Makefile"
                success "Added: Makefile"
            fi
            ((added_count++))
        elif file_differs "Makefile" "$makefile_template"; then
            pending_updates+=("Makefile:$makefile_template")
        fi
    fi
    
    # ─────────────────────────────────────────────────────────────────────────
    # Phase 5: Add missing Docker compose templates (only if not present)
    # ─────────────────────────────────────────────────────────────────────────
    
    echo ""
    echo -e "${BOLD}Checking Docker compose templates...${NC}"
    
    for template_compose in "$TEMPLATES_DIR/docker/compose/"*; do
        if [[ ! -f "$template_compose" ]]; then
            continue
        fi
        
        local compose_name=$(basename "$template_compose")
        local project_path="docker/compose/$compose_name"
        
        if [[ ! -f "$project_path" ]]; then
            if [[ "$dry_run" == "true" ]]; then
                info "[DRY RUN] Would add: $project_path"
            else
                cp "$template_compose" "$project_path"
                success "Added: $project_path"
            fi
            ((added_count++))
        fi
        # Note: We don't prompt to update compose files - those are user-customized
    done
    
    # ─────────────────────────────────────────────────────────────────────────
    # Phase 6: Add missing Docker config templates
    # ─────────────────────────────────────────────────────────────────────────
    
    echo ""
    echo -e "${BOLD}Checking Docker config templates...${NC}"
    
    for config in "$TEMPLATES_DIR/docker/config/"*; do
        if [[ ! -f "$config" ]]; then
            continue
        fi
        
        local filename=$(basename "$config")
        local target_name="${filename/env./.env.}"
        local project_path="docker/.config/$target_name"
        
        # Only add template files, never overwrite user config
        if [[ ! -f "$project_path" ]]; then
            if [[ "$dry_run" == "true" ]]; then
                info "[DRY RUN] Would add: $project_path"
            else
                cp "$config" "$project_path"
                success "Added: $project_path"
            fi
            ((added_count++))
        fi
    done
    
    # ─────────────────────────────────────────────────────────────────────────
    # Phase 7: Add missing Cloudflare infra templates (if cloudflare is enabled)
    # ─────────────────────────────────────────────────────────────────────────
    
    if [[ -d "infra/cloudflare" ]]; then
        echo ""
        echo -e "${BOLD}Checking Cloudflare templates...${NC}"
        
        # Check for README
        if [[ ! -f "infra/cloudflare/README.md" && -f "$TEMPLATES_DIR/infra/cloudflare/README.md" ]]; then
            if [[ "$dry_run" == "true" ]]; then
                info "[DRY RUN] Would add: infra/cloudflare/README.md"
            else
                cp "$TEMPLATES_DIR/infra/cloudflare/README.md" "infra/cloudflare/README.md"
                success "Added: infra/cloudflare/README.md"
            fi
            ((added_count++))
        fi
        
        # Ensure apps directory exists
        if [[ ! -d "infra/cloudflare/apps" ]]; then
            if [[ "$dry_run" == "true" ]]; then
                info "[DRY RUN] Would create: infra/cloudflare/apps/"
            else
                mkdir -p "infra/cloudflare/apps"
                success "Created: infra/cloudflare/apps/"
            fi
        fi
    fi
    
    # ─────────────────────────────────────────────────────────────────────────
    # Phase 8: Prompt for tooling file updates
    # ─────────────────────────────────────────────────────────────────────────
    
    if [[ ${#pending_updates[@]} -gt 0 ]]; then
        echo ""
        echo -e "${CYAN}╭────────────────────────────────────────────────────────────╮${NC}"
        echo -e "${CYAN}│${NC}  ${BOLD}📝 Tooling Updates Available${NC}                              ${CYAN}│${NC}"
        echo -e "${CYAN}╰────────────────────────────────────────────────────────────╯${NC}"
        echo ""
        echo -e "The following tooling files have updates available:"
        echo ""
        
        for entry in "${pending_updates[@]}"; do
            local project_file="${entry%%:*}"
            echo "    • $project_file"
        done
        
        echo ""
        
        if [[ "$dry_run" == "true" ]]; then
            info "[DRY RUN] Would prompt to update ${#pending_updates[@]} file(s)"
        else
            for entry in "${pending_updates[@]}"; do
                local project_file="${entry%%:*}"
                local template_file="${entry##*:}"
                
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
                                pending_updates=()
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
    # Summary
    # ─────────────────────────────────────────────────────────────────────────
    
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
    
    if [[ $added_count -eq 0 && $updated_count -eq 0 && $skipped_count -eq 0 && ${#pending_updates[@]} -eq 0 ]]; then
        echo -e "  ${GREEN}✓${NC} Project is up to date!"
    fi
    
    echo ""
    echo -e "${CYAN}🦝 Crate Raccoon says: Your tooling is fresh!${NC}"
}
