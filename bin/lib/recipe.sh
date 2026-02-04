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
        slug|kebab)
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
# Bash 3.x Compatible Key-Value Store
# Uses newline-separated KEY=VALUE strings instead of associative arrays
# ─────────────────────────────────────────────────────────────────────────────

# Set a key in a kv store (pass store by name)
_kv_set() {
    local store_name="$1"
    local key="$2"
    local value="$3"
    
    # Remove existing key if present, then add new
    local current
    eval "current=\"\$$store_name\""
    local new_store=""
    
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        local existing_key="${line%%=*}"
        if [[ "$existing_key" != "$key" ]]; then
            new_store+="${line}"$'\n'
        fi
    done <<< "$current"
    
    new_store+="${key}=${value}"$'\n'
    eval "$store_name=\"\$new_store\""
}

# Get a key from a kv store
_kv_get() {
    local store="$1"
    local key="$2"
    
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        local k="${line%%=*}"
        if [[ "$k" == "$key" ]]; then
            echo "${line#*=}"
            return 0
        fi
    done <<< "$store"
    return 1
}

# Get all keys from a kv store
_kv_keys() {
    local store="$1"
    
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        echo "${line%%=*}"
    done <<< "$store"
}

# Convert kv store to array of KEY=VALUE arguments
_kv_to_args() {
    local store="$1"
    local -a args=()
    
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        args+=("$line")
    done <<< "$store"
    
    echo "${args[@]}"
}

# Returns success if directory exists and contains at least one entry
_dir_nonempty() {
    local dir="$1"
    [[ -d "$dir" ]] || return 1
    [[ -n "$(ls -A "$dir" 2>/dev/null)" ]]
}

# Normalize common truthy values to "true"/"false"
_is_true() {
    local v="${1:-}"
    case "$v" in
        true|TRUE|1|yes|YES|y|Y|on|ON) return 0 ;;
        *) return 1 ;;
    esac
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

# Resolve template source path, supporting namespace references (e.g., common://path/to/file)
_resolve_template_source() {
    local from_path="$1"
    local current_recipe_dir="$2"
    
    # Check for namespace syntax: namespace://path
    if [[ "$from_path" =~ ^([a-zA-Z0-9_-]+)://(.+)$ ]]; then
        local namespace="${BASH_REMATCH[1]}"
        local rel_path="${BASH_REMATCH[2]}"
        local resolved="$RECIPES_DIR/$namespace/$rel_path"
        
        # Validate namespace exists
        if [[ ! -d "$RECIPES_DIR/$namespace" ]]; then
            error "Namespace '$namespace' not found. No folder at: $RECIPES_DIR/$namespace"
        fi
        
        # Validate file/directory exists
        if [[ ! -e "$resolved" ]]; then
            error "Namespaced path not found: $from_path
  Resolved to: $resolved
  Available in '$namespace':
$(find "$RECIPES_DIR/$namespace" -type f | sed 's|'"$RECIPES_DIR/$namespace/"'|    |')"
        fi
        
        echo "$resolved"
    else
        # Standard path: relative to current recipe
        echo "$current_recipe_dir/$from_path"
    fi
}

_process_template_file() {
    local src="$1"
    local dest="$2"
    shift 2
    
    mkdir -p "$(dirname "$dest")"
    
    # Check if file is binary (skip sed processing for binary files)
    local file_type
    file_type=$(file -b "$src" 2>/dev/null || echo "text")
    
    if [[ "$file_type" =~ (image|binary|data|SQLite|executable) ]] || \
       [[ "$src" =~ \.(png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot|sqlite|db)$ ]]; then
        # Binary file: copy directly without processing
        cp "$src" "$dest"
        return 0
    fi
    
    local sed_expr=""
    while [[ $# -gt 0 ]]; do
        local key="${1%%=*}"
        local value="${1#*=}"
        value=$(printf '%s\n' "$value" | sed 's/[&/\]/\\&/g')
        sed_expr+="s/{{$key}}/$value/g;"
        shift
    done
    
    # Process text file with sed (ensure UTF-8 locale for macOS compatibility)
    LC_ALL=en_US.UTF-8 sed "$sed_expr" "$src" > "$dest" 2>/dev/null || \
        LC_ALL=C sed "$sed_expr" "$src" > "$dest"
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
    # Parse Options (bash 3.x compatible using kv store)
    # ─────────────────────────────────────────────────────────────────────────
    
    local OPTIONS_STORE=""
    local FLAGMAP_STORE=""
    
    if _has_jq; then
        while IFS='=' read -r key val; do
            [[ -n "$key" ]] && _kv_set OPTIONS_STORE "$key" "$val"
        done < <(jq -r '.options | to_entries[]? | "\(.key)=\(.value.default)"' "$recipe_json" 2>/dev/null)

        # Map declared flags (e.g. "--zola-version") to option keys (e.g. "zola_version")
        while IFS='|' read -r key flag; do
            [[ -z "$key" ]] && continue
            [[ -z "$flag" || "$flag" == "null" ]] && continue
            flag="${flag#--}"
            [[ -n "$flag" ]] && _kv_set FLAGMAP_STORE "$flag" "$key"
        done < <(jq -r '.options | to_entries[]? | "\(.key)|\(.value.flag // empty)"' "$recipe_json" 2>/dev/null)
    fi
    
    for arg in "${extra_args[@]}"; do
        case "$arg" in
            --*=*)
                local raw_name="${arg%%=*}"
                raw_name="${raw_name#--}"
                local opt_value="${arg#*=}"

                local opt_key="$raw_name"
                if ! _kv_get "$OPTIONS_STORE" "$raw_name" >/dev/null 2>&1; then
                    local mapped_key=""
                    mapped_key=$(_kv_get "$FLAGMAP_STORE" "$raw_name" 2>/dev/null || echo "")
                    [[ -n "$mapped_key" ]] && opt_key="$mapped_key"
                fi

                _kv_set OPTIONS_STORE "$opt_key" "$opt_value"
                ;;
            --*)
                # Support boolean flags (e.g. --force-init) as true
                local raw_name="${arg#--}"
                local opt_key="$raw_name"
                if ! _kv_get "$OPTIONS_STORE" "$raw_name" >/dev/null 2>&1; then
                    local mapped_key=""
                    mapped_key=$(_kv_get "$FLAGMAP_STORE" "$raw_name" 2>/dev/null || echo "")
                    [[ -n "$mapped_key" ]] && opt_key="$mapped_key"
                fi
                _kv_set OPTIONS_STORE "$opt_key" "true"
                ;;
        esac
    done
    
    # ─────────────────────────────────────────────────────────────────────────
    # Build Placeholders (bash 3.x compatible using kv store)
    # ─────────────────────────────────────────────────────────────────────────
    
    local PLACEHOLDERS_STORE=""
    _kv_set PLACEHOLDERS_STORE "SERVICE_NAME" "$service_name"
    _kv_set PLACEHOLDERS_STORE "SERVICE_SLUG" "$(_transform "$service_name" "slug")"
    _kv_set PLACEHOLDERS_STORE "SERVICE_UPPER" "$(_transform "$service_name" "upper")"
    
    if _has_jq; then
        while IFS='|' read -r key source transform; do
            [[ -z "$key" ]] && continue
            
            local value=""
            case "$source" in
                name) value="$service_name" ;;
                option:*) 
                    local opt_key="${source#option:}"
                    value=$(_kv_get "$OPTIONS_STORE" "$opt_key" || echo "")
                    ;;
            esac
            
            [[ -n "$transform" && "$transform" != "null" ]] && value=$(_transform "$value" "$transform")
            _kv_set PLACEHOLDERS_STORE "$key" "$value"
        done < <(jq -r '.placeholders | to_entries[]? | "\(.key)|\(.value.source)|\(.value.transform)"' "$recipe_json" 2>/dev/null)
    fi
    
    # Build initial placeholder args for interpolation
    local initial_args=()
    while IFS= read -r key; do
        [[ -z "$key" ]] && continue
        local val
        val=$(_kv_get "$PLACEHOLDERS_STORE" "$key")
        initial_args+=("$key=$val")
    done < <(_kv_keys "$PLACEHOLDERS_STORE")
    
    # Second pass: interpolate any placeholder references within values
    # (e.g., DOMAIN default "{{SERVICE_NAME}}.localhost" -> "app.localhost")
    local placeholder_args=()
    while IFS= read -r key; do
        [[ -z "$key" ]] && continue
        local val
        val=$(_kv_get "$PLACEHOLDERS_STORE" "$key")
        # Interpolate the value itself with all known placeholders
        val=$(_interpolate "$val" "${initial_args[@]}")
        placeholder_args+=("$key=$val")
    done < <(_kv_keys "$PLACEHOLDERS_STORE")
    
    # ─────────────────────────────────────────────────────────────────────────
    # Display Info
    # ─────────────────────────────────────────────────────────────────────────
    
    local title=$(_json_get "$recipe_json" ".title")
    local domain_val
    domain_val=$(_kv_get "$PLACEHOLDERS_STORE" "DOMAIN" || echo "$service_name.localhost")
    # Interpolate any placeholders in domain value
    domain_val=$(_interpolate "$domain_val" "${initial_args[@]}")
    info "Installing recipe: ${BOLD}$title${NC} as ${BOLD}$service_name${NC}"
    info "Domain: ${CYAN}${domain_val}${NC}"
    
    # ─────────────────────────────────────────────────────────────────────────
    # init_app (CLI scaffolding step)
    # ─────────────────────────────────────────────────────────────────────────
    
    if _has_jq; then
        local init_cmd
        init_cmd=$(jq -r '.init_app.command // empty' "$recipe_json" 2>/dev/null)

        if [[ -n "$init_cmd" ]]; then
            local init_cwd init_target init_skip
            init_cwd=$(jq -r '.init_app.cwd // "."' "$recipe_json" 2>/dev/null)
            init_target=$(jq -r '.init_app.target_dir // empty' "$recipe_json" 2>/dev/null)
            init_skip=$(jq -r '.init_app.skip_if_exists // "true"' "$recipe_json" 2>/dev/null)

            init_cmd=$(_interpolate "$init_cmd" "${placeholder_args[@]}")
            init_cwd=$(_interpolate "$init_cwd" "${placeholder_args[@]}")
            init_target=$(_interpolate "$init_target" "${placeholder_args[@]}")

            # Optional force init (recipe should expose this as an option)
            local force_init_val
            force_init_val=$(_kv_get "$OPTIONS_STORE" "force_init" 2>/dev/null || echo "false")

            if _dir_nonempty "$init_target"; then
                if _is_true "$force_init_val"; then
                    # Safety: only allow deleting the standard target path for this recipe run
                    if [[ "$init_target" != "apps/$service_name" ]]; then
                        error "Refusing to delete non-standard init_app.target_dir: $init_target (expected apps/$service_name)"
                    fi
                    warn "force_init=true: removing existing $init_target before scaffolding"
                    rm -rf "$init_target"
                elif _is_true "$init_skip"; then
                    info "Skipping init_app (target_dir exists and is non-empty): $init_target"
                    init_cmd=""
                else
                    error "init_app target_dir already exists and is non-empty: $init_target (set --force-init or enable skip_if_exists)"
                fi
            fi

            if [[ -n "$init_cmd" ]]; then
                info "Scaffolding application via init_app..."
                mkdir -p "$init_cwd"
                (
                    cd "$init_cwd" && bash -lc "$init_cmd"
                ) || error "init_app failed: $init_cmd"
            fi
        fi
    fi

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
            
            # Resolve source path (supports namespace references like common://path)
            local src
            src=$(_resolve_template_source "$from_path" "$recipe_dir")
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
        
        # Run scripts
        while IFS='|' read -r cmd desc; do
            [[ -z "$cmd" ]] && continue
            local interpolated_cmd=$(_interpolate "$cmd" "${placeholder_args[@]}")
            [[ -n "$desc" ]] && info "$desc..."
            if ! bash -c "$interpolated_cmd" 2>/dev/null; then
                warn "Script failed: $interpolated_cmd"
            fi
        done < <(jq -r '.post_install.run[]? | "\(.command)|\(.description)"' "$recipe_json" 2>/dev/null)
        
        # Add to .gitignore
        local gitignore_file=".gitignore"
        while read -r pattern; do
            [[ -z "$pattern" ]] && continue
            local interpolated_pattern=$(_interpolate "$pattern" "${placeholder_args[@]}")
            if [[ -f "$gitignore_file" ]]; then
                grep -qxF "$interpolated_pattern" "$gitignore_file" 2>/dev/null || echo "$interpolated_pattern" >> "$gitignore_file"
            else
                echo "$interpolated_pattern" >> "$gitignore_file"
            fi
        done < <(jq -r '.post_install.gitignore[]?' "$recipe_json" 2>/dev/null)
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
    echo "    docker/.config/.env.$service_name         # Environment"
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
            # Interpolate placeholders in notes
            note=$(_interpolate "$note" "${placeholder_args[@]}")
            info "$note"
        done < <(jq -r '.notes[]?' "$recipe_json" 2>/dev/null)
    fi
    
    local final_domain
    final_domain=$(_kv_get "$PLACEHOLDERS_STORE" "DOMAIN" || echo "$service_name.localhost")
    # Interpolate any placeholders in domain value
    final_domain=$(_interpolate "$final_domain" "${placeholder_args[@]}")
    info "Access at: ${CYAN}http://${final_domain}${NC}"
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
        jq -r '.options | to_entries[]? | "    \(.value.flag // ("--" + .key))=<value>  \(.value.description) (default: \(.value.default))"' "$recipe_json" 2>/dev/null
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
# Remote Recipe Management (Unyform Integration)
# ─────────────────────────────────────────────────────────────────────────────

# Parse recipe name and version from NAME[@VERSION] format
_parse_recipe_ref() {
    local ref="$1"
    local name version
    
    if [[ "$ref" == *@* ]]; then
        name="${ref%%@*}"
        version="${ref#*@}"
    else
        name="$ref"
        version=""
    fi
    
    echo "$name|$version"
}

# List remote recipes from Unyform
recipes_list_remote() {
    if ! is_logged_in; then
        error "Not logged in. Run 'mx login' first."
    fi
    
    local url=$(get_unyform_url)
    local auth_header=$(get_auth_header)
    
    # Get default org from credentials
    local org_slug=""
    if [[ -f "$UNYFORM_SESSION_FILE" ]]; then
        org_slug=$(jq -r '.user.organizations[0].slug // empty' "$UNYFORM_SESSION_FILE" 2>/dev/null)
    fi
    
    if [[ -z "$org_slug" ]]; then
        error "No organization found. Please ensure your account has an organization."
    fi
    
    info "Fetching recipes from $org_slug..."
    
    local response
    response=$(curl -s -X GET "${url}/v1/orgs/${org_slug}/recipes" \
        -H "$auth_header" 2>/dev/null)
    
    if echo "$response" | jq -e '.error' >/dev/null 2>&1; then
        local error_msg=$(echo "$response" | jq -r '.error.message')
        error "Failed to list recipes: $error_msg"
    fi
    
    echo ""
    echo -e "${BOLD}Organization Recipes (${org_slug})${NC}"
    echo "────────────────────────────────────"
    
    local recipe_count=$(echo "$response" | jq -r '.recipes | length')
    
    if [[ "$recipe_count" == "0" ]]; then
        echo "  No recipes found."
        echo ""
        info "Recipes are generated from your connected repositories."
        info "Connect a repo and run analysis from the Unyform dashboard."
    else
        echo "$response" | jq -r '.recipes[] | "  \(.name) @ v\(.version) - \(.description // "No description")"'
    fi
    echo ""
}

# Pull a recipe from Unyform to local cache
recipes_pull() {
    local ref="$1"
    
    if [[ -z "$ref" ]]; then
        error "Recipe name required. Usage: mx recipes pull NAME[@VERSION]"
    fi
    
    if ! is_logged_in; then
        error "Not logged in. Run 'mx login' first."
    fi
    
    local parsed=$(_parse_recipe_ref "$ref")
    local recipe_name="${parsed%%|*}"
    local version="${parsed#*|}"
    
    local url=$(get_unyform_url)
    local auth_header=$(get_auth_header)
    
    # Get default org
    local org_slug=""
    if [[ -f "$UNYFORM_SESSION_FILE" ]]; then
        org_slug=$(jq -r '.user.organizations[0].slug // empty' "$UNYFORM_SESSION_FILE" 2>/dev/null)
    fi
    
    if [[ -z "$org_slug" ]]; then
        error "No organization found."
    fi
    
    # Build endpoint
    local endpoint="${url}/v1/orgs/${org_slug}/recipes/${recipe_name}"
    if [[ -n "$version" ]]; then
        endpoint="${endpoint}/versions/${version}"
    else
        # Get latest version first
        local versions_response
        versions_response=$(curl -s -X GET "${endpoint}/versions" \
            -H "$auth_header" 2>/dev/null)
        
        if echo "$versions_response" | jq -e '.error' >/dev/null 2>&1; then
            local error_msg=$(echo "$versions_response" | jq -r '.error.message')
            error "Recipe not found: $error_msg"
        fi
        
        version=$(echo "$versions_response" | jq -r '.versions[] | select(.is_latest) | .version')
        if [[ -z "$version" ]]; then
            version=$(echo "$versions_response" | jq -r '.versions[0].version')
        fi
        endpoint="${endpoint}/versions/${version}"
    fi
    
    info "Pulling $recipe_name@$version from $org_slug..."
    
    local response
    response=$(curl -s -X GET "$endpoint" \
        -H "$auth_header" 2>/dev/null)
    
    if echo "$response" | jq -e '.error' >/dev/null 2>&1; then
        local error_msg=$(echo "$response" | jq -r '.error.message')
        error "Failed to pull recipe: $error_msg"
    fi
    
    # Create cache directory
    local cache_dir="${UNYFORM_RECIPES_DIR}/${org_slug}/${recipe_name}/${version}"
    mkdir -p "$cache_dir"
    
    # Save recipe
    echo "$response" > "${cache_dir}/recipe.json"
    
    # Save manifest
    cat > "${cache_dir}/manifest.json" << EOF
{
    "pulled_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "sha256": "$(echo "$response" | shasum -a 256 | cut -d' ' -f1)",
    "org": "$org_slug",
    "name": "$recipe_name",
    "version": "$version"
}
EOF
    
    # Update latest symlink
    local latest_link="${UNYFORM_RECIPES_DIR}/${org_slug}/${recipe_name}/latest"
    rm -f "$latest_link"
    ln -s "$version" "$latest_link"
    
    success "Recipe cached: $cache_dir"
    info "Run 'mx recipes apply $recipe_name' to apply to current project"
}

# List available versions for a recipe
recipes_versions() {
    local recipe_name="$1"
    
    if [[ -z "$recipe_name" ]]; then
        error "Recipe name required. Usage: mx recipes versions NAME"
    fi
    
    if ! is_logged_in; then
        error "Not logged in. Run 'mx login' first."
    fi
    
    local url=$(get_unyform_url)
    local auth_header=$(get_auth_header)
    
    local org_slug=""
    if [[ -f "$UNYFORM_SESSION_FILE" ]]; then
        org_slug=$(jq -r '.user.organizations[0].slug // empty' "$UNYFORM_SESSION_FILE" 2>/dev/null)
    fi
    
    if [[ -z "$org_slug" ]]; then
        error "No organization found."
    fi
    
    local response
    response=$(curl -s -X GET "${url}/v1/orgs/${org_slug}/recipes/${recipe_name}/versions" \
        -H "$auth_header" 2>/dev/null)
    
    if echo "$response" | jq -e '.error' >/dev/null 2>&1; then
        local error_msg=$(echo "$response" | jq -r '.error.message')
        error "Failed to get versions: $error_msg"
    fi
    
    echo ""
    echo -e "${BOLD}Versions for ${recipe_name}${NC}"
    echo "────────────────────────────────────"
    echo "$response" | jq -r '.versions[] | "  v\(.version) \(if .is_latest then "(latest)" else "" end) - \(.generated_at)"'
    echo ""
}

# Show cached recipes
recipes_cache() {
    local subcommand="${1:-list}"
    
    case "$subcommand" in
        list|"")
            echo ""
            echo -e "${BOLD}Cached Recipes${NC}"
            echo "────────────────────────────────────"
            
            if [[ ! -d "$UNYFORM_RECIPES_DIR" ]] || [[ -z "$(ls -A "$UNYFORM_RECIPES_DIR" 2>/dev/null)" ]]; then
                echo "  No cached recipes."
                echo ""
                info "Pull recipes with 'mx recipes pull NAME'"
            else
                for org_dir in "$UNYFORM_RECIPES_DIR"/*/; do
                    [[ -d "$org_dir" ]] || continue
                    local org=$(basename "$org_dir")
                    
                    for recipe_dir in "$org_dir"*/; do
                        [[ -d "$recipe_dir" ]] || continue
                        local recipe=$(basename "$recipe_dir")
                        
                        # List versions
                        local versions=""
                        for ver_dir in "$recipe_dir"*/; do
                            [[ -d "$ver_dir" ]] || continue
                            local ver=$(basename "$ver_dir")
                            [[ "$ver" == "latest" ]] && continue
                            [[ -n "$versions" ]] && versions+=", "
                            versions+="$ver"
                        done
                        
                        echo "  ${org}/${recipe}: $versions"
                    done
                done
            fi
            echo ""
            ;;
        clear)
            if [[ -d "$UNYFORM_RECIPES_DIR" ]]; then
                rm -rf "$UNYFORM_RECIPES_DIR"/*
                success "Recipe cache cleared"
            else
                info "Cache already empty"
            fi
            ;;
        *)
            error "Unknown cache command: $subcommand"
            ;;
    esac
}

# Apply a recipe to the current project
recipes_apply() {
    local ref="$1"
    shift 2>/dev/null || true
    local extra_args=("$@")
    
    if [[ -z "$ref" ]]; then
        error "Recipe name required. Usage: mx recipes apply NAME[@VERSION]"
    fi
    
    local parsed=$(_parse_recipe_ref "$ref")
    local recipe_name="${parsed%%|*}"
    local version="${parsed#*|}"
    
    # Get org from credentials
    local org_slug=""
    if [[ -f "$UNYFORM_SESSION_FILE" ]]; then
        org_slug=$(jq -r '.user.organizations[0].slug // empty' "$UNYFORM_SESSION_FILE" 2>/dev/null)
    fi
    
    # Find cached recipe
    local recipe_path=""
    
    if [[ -n "$version" ]]; then
        recipe_path="${UNYFORM_RECIPES_DIR}/${org_slug}/${recipe_name}/${version}/recipe.json"
    else
        # Try latest symlink
        local latest_dir="${UNYFORM_RECIPES_DIR}/${org_slug}/${recipe_name}/latest"
        if [[ -L "$latest_dir" ]]; then
            recipe_path="${UNYFORM_RECIPES_DIR}/${org_slug}/${recipe_name}/$(readlink "$latest_dir")/recipe.json"
        fi
    fi
    
    # If not cached, pull first
    if [[ ! -f "$recipe_path" ]]; then
        warn "Recipe not in cache, pulling..."
        recipes_pull "$ref"
        
        # Re-resolve path
        if [[ -n "$version" ]]; then
            recipe_path="${UNYFORM_RECIPES_DIR}/${org_slug}/${recipe_name}/${version}/recipe.json"
        else
            local latest_dir="${UNYFORM_RECIPES_DIR}/${org_slug}/${recipe_name}/latest"
            recipe_path="${UNYFORM_RECIPES_DIR}/${org_slug}/${recipe_name}/$(readlink "$latest_dir")/recipe.json"
        fi
    fi
    
    if [[ ! -f "$recipe_path" ]]; then
        error "Recipe not found in cache: $recipe_path"
    fi
    
    # Read recipe
    local recipe_json
    recipe_json=$(cat "$recipe_path")
    
    local recipe_version=$(echo "$recipe_json" | jq -r '.version')
    local recipe_desc=$(echo "$recipe_json" | jq -r '.description // "No description"')
    
    echo ""
    echo -e "Applying recipe: ${BOLD}${recipe_name}${NC} v${recipe_version}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Apply patterns
    local patterns_count=$(echo "$recipe_json" | jq -r '.patterns | length')
    if [[ "$patterns_count" -gt 0 ]]; then
        _apply_patterns "$recipe_json" "${extra_args[@]}"
    fi
    
    # Check dependencies
    _check_dependencies "$recipe_json"
    
    # Check infrastructure
    _check_infrastructure "$recipe_json"
    
    echo ""
    success "Recipe applied!"
    echo ""
    info "Run 'mx recipes apply --fix' to auto-update dependencies."
}

# Apply patterns from recipe to project
_apply_patterns() {
    local recipe_json="$1"
    shift
    local extra_args=("$@")
    
    local rules_dir=".cursor/rules"
    mkdir -p "$rules_dir"
    
    # Get org name from recipe
    local org_name=$(echo "$recipe_json" | jq -r '.name' | sed 's/ Engineering Standards$//' | tr '[:upper:]' '[:lower:]' | tr ' ' '-')
    local rules_file="${rules_dir}/${org_name}-patterns.md"
    
    # Generate rules file from patterns
    {
        echo "# ${org_name} Coding Patterns"
        echo ""
        echo "Generated from organizational recipe."
        echo ""
        
        echo "$recipe_json" | jq -r '.patterns[] | "## \(.name)\n\n\(.description)\n\n### Rules\n\n\(.rules[] | "- \(.)")\n"'
    } > "$rules_file"
    
    local patterns_count=$(echo "$recipe_json" | jq -r '.patterns | length')
    success "Created ${rules_file} (${patterns_count} coding rules)"
}

# Check project dependencies against recipe
_check_dependencies() {
    local recipe_json="$1"
    
    echo -e "${BOLD}Dependencies:${NC}"
    
    # Check Rust dependencies
    local rust_deps=$(echo "$recipe_json" | jq -r '.dependencies.rust[]?')
    if [[ -n "$rust_deps" ]] && [[ -f "Cargo.toml" ]]; then
        echo "$recipe_json" | jq -r '.dependencies.rust[]? | "  \(.name): \(.version // "any")"' | while read -r line; do
            local dep_name=$(echo "$line" | cut -d: -f1 | tr -d ' ')
            if grep -q "^$dep_name" Cargo.toml 2>/dev/null; then
                echo -e "  ${GREEN}✓${NC} $line"
            else
                echo -e "  ${YELLOW}⚠${NC} Missing: $dep_name (recommended by recipe)"
            fi
        done
    fi
    
    # Check Node dependencies
    local node_deps=$(echo "$recipe_json" | jq -r '.dependencies.node[]?')
    if [[ -n "$node_deps" ]] && [[ -f "package.json" ]]; then
        echo "$recipe_json" | jq -r '.dependencies.node[]? | "  \(.name): \(.version // "any")"' | while read -r line; do
            local dep_name=$(echo "$line" | cut -d: -f1 | tr -d ' ')
            if grep -q "\"$dep_name\"" package.json 2>/dev/null; then
                echo -e "  ${GREEN}✓${NC} $line"
            else
                echo -e "  ${YELLOW}⚠${NC} Missing: $dep_name (recommended by recipe)"
            fi
        done
    fi
    
    echo ""
}

# Check project infrastructure against recipe
_check_infrastructure() {
    local recipe_json="$1"
    
    echo -e "${BOLD}Infrastructure:${NC}"
    
    # Check Docker
    local has_docker=$(echo "$recipe_json" | jq -r '.infrastructure.docker.enabled // false')
    if [[ "$has_docker" == "true" ]]; then
        if [[ -f "Dockerfile" ]] || [[ -f "docker/Dockerfile" ]]; then
            echo -e "  ${GREEN}✓${NC} Dockerfile matches recipe patterns"
        else
            echo -e "  ${YELLOW}⚠${NC} Missing: Dockerfile (run 'mx add' to create)"
        fi
    fi
    
    # Check CI/CD
    local has_cicd=$(echo "$recipe_json" | jq -r '.infrastructure.ci_cd.enabled // false')
    if [[ "$has_cicd" == "true" ]]; then
        if [[ -f ".github/workflows/ci.yml" ]] || [[ -d ".github/workflows" ]]; then
            echo -e "  ${GREEN}✓${NC} CI/CD workflow present"
        else
            echo -e "  ${YELLOW}⚠${NC} Missing: .github/workflows/ci.yml (run 'mx recipes apply --with-ci')"
        fi
    fi
    
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
            # Check if we want remote recipes
            if [[ "$1" == "--remote" ]] || is_logged_in 2>/dev/null; then
                if is_logged_in 2>/dev/null; then
                    recipes_list_remote
                fi
            fi
            show_all_recipes
            ;;
        info)
            if [[ -z "$1" ]]; then
                error "Recipe name required. Usage: mx recipes info <recipe>"
            fi
            show_recipe_info "$1"
            ;;
        pull)
            recipes_pull "$@"
            ;;
        apply)
            recipes_apply "$@"
            ;;
        versions)
            recipes_versions "$@"
            ;;
        cache)
            recipes_cache "$@"
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
                echo -e "${BOLD}LOCAL RECIPES:${NC}"
                echo "    list              List all available local recipes"
                echo "    info <recipe>     Show detailed info about a recipe"
                echo ""
                echo -e "${BOLD}REMOTE RECIPES (requires login):${NC}"
                echo "    pull NAME[@VER]   Pull recipe from Unyform to cache"
                echo "    apply NAME[@VER]  Apply recipe to current project"
                echo "    versions NAME     List available versions"
                echo "    cache [clear]     Show or clear cached recipes"
                echo ""
                echo -e "${BOLD}EXAMPLES:${NC}"
                echo "    mx recipes                         # List all recipes"
                echo "    mx recipes info laravel            # Show Laravel recipe details"
                echo "    mx add myapp --recipe=nuxt         # Add service using recipe"
                echo ""
                echo "    mx login                           # Authenticate with Unyform"
                echo "    mx recipes pull engineering-standards      # Pull org recipe"
                echo "    mx recipes pull engineering-standards@1.0  # Pull specific version"
                echo "    mx recipes apply engineering-standards     # Apply to project"
                echo "    mx recipes cache                           # Show cached recipes"
                echo ""
            fi
            ;;
    esac
}
