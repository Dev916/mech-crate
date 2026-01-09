#!/bin/bash
#
# MechCrate Recipe System
# Data-driven recipe engine that reads JSON metadata
#
# Each recipe is defined by:
#   templates/recipes/<name>/recipe.json  - Metadata, options, steps
#   templates/recipes/<name>/app/         - App template files
#   templates/recipes/<name>/docker/      - Docker template files
#   templates/recipes/<name>/config/      - Config template files
#

RECIPES_DIR="$TEMPLATES_DIR/recipes"

# ─────────────────────────────────────────────────────────────────────────────
# JSON Parsing Utilities
# ─────────────────────────────────────────────────────────────────────────────

_has_jq() {
    command -v jq &>/dev/null
}

_json_get() {
    local json_file="$1"
    local key="$2"
    
    if _has_jq; then
        jq -r "$key // empty" "$json_file" 2>/dev/null
    else
        grep -o "\"${key#.}\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" "$json_file" | \
            sed 's/.*: *"\([^"]*\)"/\1/' | head -1
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# String Transformation
# ─────────────────────────────────────────────────────────────────────────────

_transform() {
    local value="$1"
    local transform="$2"
    
    case "$transform" in
        slug)
            echo "$value" | tr '[:upper:]' '[:lower:]' | tr -cd '[:alnum:]-'
            ;;
        upper)
            echo "$value" | tr '[:lower:]' '[:upper:]' | sed 's/[^A-Z0-9_]/_/g'
            ;;
        rust_crate)
            echo "$value" | tr '[:upper:]' '[:lower:]' | tr -cd '[:alnum:]-' | tr '-' '_'
            ;;
        ssr_bool)
            [[ "$value" == "spa" ]] && echo "false" || echo "true"
            ;;
        *)
            echo "$value"
            ;;
    esac
}

_interpolate() {
    local str="$1"
    shift
    
    while [[ $# -gt 0 ]]; do
        local key="${1%%=*}"
        local value="${1#*=}"
        str="${str//\{\{$key\}\}/$value}"
        shift
    done
    
    echo "$str"
}

# ─────────────────────────────────────────────────────────────────────────────
# Recipe Discovery
# ─────────────────────────────────────────────────────────────────────────────

list_recipes() {
    local recipes=()
    
    if [[ -d "$RECIPES_DIR" ]]; then
        for recipe_dir in "$RECIPES_DIR"/*/; do
            if [[ -f "${recipe_dir}recipe.json" ]]; then
                recipes+=("$(basename "$recipe_dir")")
            fi
        done
    fi
    
    echo "${recipes[@]}"
}

recipe_exists() {
    local recipe_name="$1"
    [[ -f "$RECIPES_DIR/$recipe_name/recipe.json" ]]
}

get_recipe_description() {
    local recipe_name="$1"
    local recipe_json="$RECIPES_DIR/$recipe_name/recipe.json"
    
    if [[ -f "$recipe_json" ]]; then
        _json_get "$recipe_json" ".description"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Template Processing
# ─────────────────────────────────────────────────────────────────────────────

_process_template_file() {
    local src="$1"
    local dest="$2"
    shift 2
    
    mkdir -p "$(dirname "$dest")"
    
    local sed_expr=""
    while [[ $# -gt 0 ]]; do
        local key="${1%%=*}"
        local value="${1#*=}"
        value=$(printf '%s\n' "$value" | sed 's/[&/\]/\\&/g')
        sed_expr+="s/{{$key}}/$value/g;"
        shift
    done
    
    sed "$sed_expr" "$src" > "$dest"
}

_process_template_dir() {
    local src_dir="$1"
    local dest_dir="$2"
    shift 2
    local args=("$@")
    
    find "$src_dir" -type f | while read -r src_file; do
        local rel_path="${src_file#$src_dir/}"
        local dest_file="$dest_dir/$rel_path"
        _process_template_file "$src_file" "$dest_file" "${args[@]}"
    done
}

# ─────────────────────────────────────────────────────────────────────────────
# Recipe Installation
# ─────────────────────────────────────────────────────────────────────────────

install_recipe() {
    local recipe_name="$1"
    local service_name="$2"
    shift 2
    local extra_args=("$@")
    
    if ! recipe_exists "$recipe_name"; then
        error "Recipe '$recipe_name' not found. Available recipes: $(list_recipes | tr ' ' ', ')"
    fi
    
    local recipe_dir="$RECIPES_DIR/$recipe_name"
    local recipe_json="$recipe_dir/recipe.json"
    
    # ─────────────────────────────────────────────────────────────────────────
    # Parse Options
    # ─────────────────────────────────────────────────────────────────────────
    
    declare -A options
    
    if _has_jq; then
        while IFS='=' read -r key val; do
            [[ -n "$key" ]] && options["$key"]="$val"
        done < <(jq -r '.options | to_entries[]? | "\(.key)=\(.value.default)"' "$recipe_json" 2>/dev/null)
    fi
    
    for arg in "${extra_args[@]}"; do
        case "$arg" in
            --*=*)
                local opt_name="${arg%%=*}"
                opt_name="${opt_name#--}"
                options["$opt_name"]="${arg#*=}"
                ;;
        esac
    done
    
    # ─────────────────────────────────────────────────────────────────────────
    # Build Placeholders
    # ─────────────────────────────────────────────────────────────────────────
    
    declare -A placeholders
    placeholders["SERVICE_NAME"]="$service_name"
    placeholders["SERVICE_SLUG"]=$(_transform "$service_name" "slug")
    placeholders["SERVICE_UPPER"]=$(_transform "$service_name" "upper")
    
    if _has_jq; then
        while IFS='|' read -r key source transform; do
            [[ -z "$key" ]] && continue
            
            local value=""
            case "$source" in
                name) value="$service_name" ;;
                option:*) value="${options[${source#option:}]:-}" ;;
            esac
            
            [[ -n "$transform" && "$transform" != "null" ]] && value=$(_transform "$value" "$transform")
            placeholders["$key"]="$value"
        done < <(jq -r '.placeholders | to_entries[]? | "\(.key)|\(.value.source)|\(.value.transform)"' "$recipe_json" 2>/dev/null)
    fi
    
    local placeholder_args=()
    for key in "${!placeholders[@]}"; do
        placeholder_args+=("$key=${placeholders[$key]}")
    done
    
    # ─────────────────────────────────────────────────────────────────────────
    # Display Info
    # ─────────────────────────────────────────────────────────────────────────
    
    local title=$(_json_get "$recipe_json" ".title")
    info "Installing recipe: ${BOLD}$title${NC} as ${BOLD}$service_name${NC}"
    info "Domain: ${CYAN}${placeholders[DOMAIN]:-$service_name.localhost}${NC}"
    
    # ─────────────────────────────────────────────────────────────────────────
    # Create Directories
    # ─────────────────────────────────────────────────────────────────────────
    
    if _has_jq; then
        while read -r dir_template; do
            [[ -z "$dir_template" ]] && continue
            local dir=$(_interpolate "$dir_template" "${placeholder_args[@]}")
            mkdir -p "$dir"
        done < <(jq -r '.directories[]?' "$recipe_json" 2>/dev/null)
    fi
    
    # ─────────────────────────────────────────────────────────────────────────
    # Process Templates
    # ─────────────────────────────────────────────────────────────────────────
    
    if _has_jq; then
        while IFS='|' read -r from_path to_path; do
            [[ -z "$from_path" ]] && continue
            
            local src="$recipe_dir/$from_path"
            local dest=$(_interpolate "$to_path" "${placeholder_args[@]}")
            
            if [[ -d "$src" ]]; then
                _process_template_dir "$src" "$dest" "${placeholder_args[@]}"
            elif [[ -f "$src" ]]; then
                _process_template_file "$src" "$dest" "${placeholder_args[@]}"
            fi
        done < <(jq -r '.templates[]? | "\(.from)|\(.to)"' "$recipe_json" 2>/dev/null)
    fi
    
    # ─────────────────────────────────────────────────────────────────────────
    # Post-Install Actions
    # ─────────────────────────────────────────────────────────────────────────
    
    if _has_jq; then
        # Renames
        while IFS='|' read -r from_path to_path; do
            [[ -z "$from_path" ]] && continue
            local src=$(_interpolate "$from_path" "${placeholder_args[@]}")
            local dest=$(_interpolate "$to_path" "${placeholder_args[@]}")
            [[ -f "$src" ]] && mv "$src" "$dest"
        done < <(jq -r '.post_install.renames[]? | "\(.from)|\(.to)"' "$recipe_json" 2>/dev/null)
        
        # Chmod
        while IFS='|' read -r path mode; do
            [[ -z "$path" ]] && continue
            local target=$(_interpolate "$path" "${placeholder_args[@]}")
            [[ -f "$target" ]] && chmod "$mode" "$target" 2>/dev/null || true
        done < <(jq -r '.post_install.chmod[]? | "\(.path)|\(.mode)"' "$recipe_json" 2>/dev/null)
        
        # Gitkeep
        while read -r dir_template; do
            [[ -z "$dir_template" ]] && continue
            local dir=$(_interpolate "$dir_template" "${placeholder_args[@]}")
            [[ -d "$dir" ]] && touch "$dir/.gitkeep"
        done < <(jq -r '.post_install.gitkeep[]?' "$recipe_json" 2>/dev/null)
        
        # Create files
        while IFS='|' read -r path content; do
            [[ -z "$path" ]] && continue
            local target=$(_interpolate "$path" "${placeholder_args[@]}")
            mkdir -p "$(dirname "$target")"
            printf '%b' "$content" > "$target"
        done < <(jq -r '.post_install.create_files[]? | "\(.path)|\(.content)"' "$recipe_json" 2>/dev/null)
    fi
    
    # ─────────────────────────────────────────────────────────────────────────
    # Success Message
    # ─────────────────────────────────────────────────────────────────────────
    
    success "$title recipe installed for '$service_name'!"
    echo ""
    info "Created files:"
    echo "    apps/$service_name/                       # App source"
    echo "    docker/compose/$service_name.yml          # Production compose"
    echo "    docker/compose/$service_name.dev.yml      # Development overrides"
    echo "    docker/dockerfiles/$service_name/         # Dockerfile"
    echo "    docker/config/.env.$service_name          # Environment"
    echo ""
    
    info "Next steps:"
    if _has_jq; then
        while read -r step; do
            [[ -z "$step" ]] && continue
            echo "    $(_interpolate "$step" "${placeholder_args[@]}")"
        done < <(jq -r '.next_steps[]?' "$recipe_json" 2>/dev/null)
    fi
    echo ""
    
    if _has_jq; then
        while read -r note; do
            [[ -z "$note" ]] && continue
            info "$note"
        done < <(jq -r '.notes[]?' "$recipe_json" 2>/dev/null)
    fi
    
    info "Access at: ${CYAN}http://${placeholders[DOMAIN]:-$service_name.localhost}${NC}"
}

# ─────────────────────────────────────────────────────────────────────────────
# Recipe Info Display
# ─────────────────────────────────────────────────────────────────────────────

show_recipe_info() {
    local recipe_name="$1"
    local recipe_json="$RECIPES_DIR/$recipe_name/recipe.json"
    
    if [[ ! -f "$recipe_json" ]]; then
        error "Recipe '$recipe_name' not found."
        return 1
    fi
    
    local title=$(_json_get "$recipe_json" ".title")
    local desc=$(_json_get "$recipe_json" ".description")
    
    echo ""
    echo -e "  ${BOLD}$title Recipe${NC}"
    echo ""
    echo "  $desc"
    echo ""
    echo "  Features:"
    
    if _has_jq; then
        jq -r '.features[]? | "    • \(.)"' "$recipe_json" 2>/dev/null
    fi
    
    echo ""
    echo -e "  ${BOLD}Usage:${NC}"
    echo "    mx add myapp --recipe=$recipe_name"
    echo "    mx add myapp --recipe=$recipe_name --domain=myapp.com"
    echo ""
    echo -e "  ${BOLD}Options:${NC}"
    
    if _has_jq; then
        jq -r '.options | to_entries[]? | "    --\(.key)=<value>  \(.value.description) (default: \(.value.default))"' "$recipe_json" 2>/dev/null
    fi
    
    echo ""
    echo -e "  ${BOLD}Created Services:${NC}"
    
    if _has_jq; then
        jq -r '.services[]? | "    \(.name | gsub("<name>"; "\u001b[36m<name>\u001b[0m"))  \(.description)"' "$recipe_json" 2>/dev/null
    fi
    echo ""
}

show_all_recipes() {
    echo ""
    echo -e "${BOLD}Available Recipes:${NC}"
    echo ""
    
    local recipes=($(list_recipes))
    
    if [[ ${#recipes[@]} -eq 0 ]]; then
        echo "  No recipes found."
        return
    fi
    
    for recipe in "${recipes[@]}"; do
        local desc=$(get_recipe_description "$recipe")
        printf "  ${CYAN}%-15s${NC} %s\n" "$recipe" "$desc"
    done
    
    echo ""
    echo "  Use 'mx recipes info <name>' for details."
    echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Recipe CLI Command
# ─────────────────────────────────────────────────────────────────────────────

recipes_cmd() {
    local subcommand="${1:-list}"
    shift 2>/dev/null || true
    
    case "$subcommand" in
        list|ls)
            show_all_recipes
            ;;
        info)
            if [[ -z "$1" ]]; then
                error "Recipe name required. Usage: mx recipes info <recipe>"
            fi
            show_recipe_info "$1"
            ;;
        *)
            if recipe_exists "$subcommand"; then
                show_recipe_info "$subcommand"
            else
                echo ""
                echo -e "${BOLD}mx recipes${NC} - Manage MechCrate recipes"
                echo ""
                echo -e "${BOLD}USAGE:${NC}"
                echo "    mx recipes [command]"
                echo ""
                echo -e "${BOLD}COMMANDS:${NC}"
                echo "    list              List all available recipes"
                echo "    info <recipe>     Show detailed info about a recipe"
                echo ""
                echo -e "${BOLD}EXAMPLES:${NC}"
                echo "    mx recipes                    # List all recipes"
                echo "    mx recipes info laravel       # Show Laravel recipe details"
                echo "    mx add myapp --recipe=nuxt    # Add service using recipe"
                echo ""
            fi
            ;;
    esac
}
