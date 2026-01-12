#!/bin/bash
#
# MechCrate Infrastructure Configuration
# Global and project-level service configuration with hierarchical resolution
#

# ─────────────────────────────────────────────────────────────────────────────
# Configuration Paths
# ─────────────────────────────────────────────────────────────────────────────

MX_CONFIG_HOME="${MX_CONFIG_HOME:-${HOME}/.mech-crate/config}"
MX_INFRA_CONFIG_DIR="${MX_CONFIG_HOME}/infra"

# Supported infrastructure providers
MX_INFRA_PROVIDERS=("cloudflare" "digitalocean" "aws" "hetzner")

# ─────────────────────────────────────────────────────────────────────────────
# Internal Helpers
# ─────────────────────────────────────────────────────────────────────────────

_infra_log() {
    echo -e "${CYAN}[infra]${NC} $*"
}

_infra_die() {
    echo -e "${RED}[infra] error:${NC} $*" >&2
    exit 1
}

_infra_ensure_config_dir() {
    mkdir -p "${MX_INFRA_CONFIG_DIR}"
}

_infra_provider_config_path() {
    local provider="$1"
    echo "${MX_INFRA_CONFIG_DIR}/${provider}.env"
}

_infra_project_config_path() {
    local provider="$1"
    echo "./infra/${provider}/.env.${provider}"
}

_infra_provider_exists() {
    local provider="$1"
    for p in "${MX_INFRA_PROVIDERS[@]}"; do
        [[ "$p" == "$provider" ]] && return 0
    done
    return 1
}

_infra_has_global_config() {
    local provider="$1"
    local config_file
    config_file="$(_infra_provider_config_path "$provider")"
    [[ -f "$config_file" ]]
}

_infra_has_project_config() {
    local provider="$1"
    local config_file
    config_file="$(_infra_project_config_path "$provider")"
    [[ -f "$config_file" ]]
}

# Check if project config is linked to global
_infra_is_linked() {
    local provider="$1"
    local project_config
    project_config="$(_infra_project_config_path "$provider")"
    
    if [[ -f "$project_config" ]]; then
        # Check for MX_INFRA_USE_GLOBAL marker
        grep -q "^MX_INFRA_USE_GLOBAL=true" "$project_config" 2>/dev/null
        return $?
    fi
    return 1
}

# ─────────────────────────────────────────────────────────────────────────────
# Config Resolution (Hierarchical Lookup)
# ─────────────────────────────────────────────────────────────────────────────

# Resolve config file path with hierarchical lookup
# Returns: path to config file to use, or empty if none found
# Priority: project (if not linked) → global
infra_resolve_config() {
    local provider="$1"
    local project_config global_config
    
    project_config="$(_infra_project_config_path "$provider")"
    global_config="$(_infra_provider_config_path "$provider")"
    
    # If project config exists and is NOT linked to global, use it
    if [[ -f "$project_config" ]]; then
        if ! _infra_is_linked "$provider"; then
            echo "$project_config"
            return 0
        fi
    fi
    
    # Fall back to global config
    if [[ -f "$global_config" ]]; then
        echo "$global_config"
        return 0
    fi
    
    # No config found
    return 1
}

# Load config into environment (call from other scripts)
# Usage: infra_load_config cloudflare
infra_load_config() {
    local provider="$1"
    local config_file
    
    config_file="$(infra_resolve_config "$provider")" || {
        return 1
    }
    
    # Source the config file
    # shellcheck source=/dev/null
    source "$config_file"
    return 0
}

# Get a specific config value
# Usage: infra_get_value cloudflare CF_ACCOUNT_ID
infra_get_value() {
    local provider="$1"
    local key="$2"
    local config_file
    
    config_file="$(infra_resolve_config "$provider")" || return 1
    
    grep "^${key}=" "$config_file" 2>/dev/null | cut -d'=' -f2-
}

# ─────────────────────────────────────────────────────────────────────────────
# Setup Commands
# ─────────────────────────────────────────────────────────────────────────────

_infra_setup_cloudflare() {
    local config_file
    config_file="$(_infra_provider_config_path "cloudflare")"
    
    echo -e "${CYAN}"
    cat << 'EOF'
    ╭──────────────────────────────────────────────────────────╮
    │  ☁️  Cloudflare Global Configuration                      │
    ╰──────────────────────────────────────────────────────────╯
EOF
    echo -e "${NC}"
    
    # Check for existing config
    if [[ -f "$config_file" ]]; then
        # shellcheck source=/dev/null
        source "$config_file"
        echo -e "Current global configuration:"
        echo -e "  Account ID: ${BOLD}${CF_ACCOUNT_ID:-not set}${NC}"
        echo ""
        read -r -p "Reconfigure? [y/N]: " reconfigure
        if [[ ! "$reconfigure" =~ ^[Yy] ]]; then
            echo "Keeping existing configuration."
            return 0
        fi
        echo ""
    fi
    
    # Step 1: Authentication
    echo -e "${BOLD}Step 1: Authentication${NC}"
    echo ""
    
    if npx wrangler whoami &>/dev/null 2>&1; then
        CURRENT_USER=$(npx wrangler whoami 2>/dev/null | grep -oE 'email: [^ ]+' | cut -d' ' -f2 || echo "authenticated")
        success "Already logged in as: $CURRENT_USER"
    else
        info "Not logged in to Cloudflare."
        read -r -p "Run 'wrangler login' now? [Y/n]: " do_login
        if [[ ! "$do_login" =~ ^[Nn] ]]; then
            npx wrangler login
            success "Login successful!"
        else
            warn "Skipping login. You'll need to login before deploying."
        fi
    fi
    
    echo ""
    echo -e "${BOLD}Step 2: Account Configuration${NC}"
    echo ""
    
    echo "Your Cloudflare Account ID can be found at:"
    echo "  https://dash.cloudflare.com → Any zone → Overview → Account ID (right sidebar)"
    echo ""
    
    # Try to auto-detect account ID
    local DETECTED_ACCOUNT_ID
    DETECTED_ACCOUNT_ID=$(npx wrangler whoami 2>/dev/null | grep -oE 'Account ID: [a-f0-9]+' | cut -d' ' -f3 || echo "")
    
    local CF_ACCOUNT_ID=""
    if [[ -n "$DETECTED_ACCOUNT_ID" ]]; then
        echo -e "Detected Account ID: ${BOLD}$DETECTED_ACCOUNT_ID${NC}"
        read -r -p "Use this Account ID? [Y/n]: " use_detected
        if [[ ! "$use_detected" =~ ^[Nn] ]]; then
            CF_ACCOUNT_ID="$DETECTED_ACCOUNT_ID"
        fi
    fi
    
    if [[ -z "$CF_ACCOUNT_ID" ]]; then
        read -r -p "Enter your Cloudflare Account ID: " CF_ACCOUNT_ID
    fi
    
    if [[ -z "$CF_ACCOUNT_ID" ]]; then
        _infra_die "Account ID is required."
    fi
    
    echo ""
    echo -e "${BOLD}Step 3: API Token (for CI/CD)${NC}"
    echo ""
    echo "An API token allows automated deployments without interactive login."
    echo "Create one at: https://dash.cloudflare.com/profile/api-tokens"
    echo "Required permissions: Workers Scripts (Edit), Cloudflare Container Registry (Edit)"
    echo ""
    read -r -p "Enter API Token (or press Enter to skip): " CF_API_TOKEN
    
    echo ""
    echo -e "${BOLD}Step 4: Default Docker Platform${NC}"
    echo ""
    echo "Cloudflare Containers support linux/amd64 and linux/arm64."
    read -r -p "Default platform [linux/amd64]: " CF_DOCKER_PLATFORM
    CF_DOCKER_PLATFORM="${CF_DOCKER_PLATFORM:-linux/amd64}"
    
    # Save configuration
    echo ""
    _infra_log "Saving global configuration..."
    
    cat > "$config_file" << EOF
# Cloudflare Global Configuration
# Generated by mx infra setup cloudflare on $(date)
#
# This is the global config stored in ~/.mech-crate/config/infra/
# Projects can link to this or override with project-local credentials.

# Your Cloudflare Account ID
CF_ACCOUNT_ID=$CF_ACCOUNT_ID

# Docker platform for container builds
CF_DOCKER_PLATFORM=$CF_DOCKER_PLATFORM
EOF
    
    if [[ -n "$CF_API_TOKEN" ]]; then
        cat >> "$config_file" << EOF

# API Token for CI/CD (keep secret!)
CLOUDFLARE_API_TOKEN=$CF_API_TOKEN
EOF
    fi
    
    chmod 600 "$config_file"
    
    echo ""
    success "Cloudflare global configuration saved!"
    echo ""
    echo "Projects can now use this config with:"
    echo "  ${BOLD}mx infra link cloudflare${NC}  (from within a project)"
    echo ""
    echo "Or access directly from any project's Cloudflare setup."
}

_infra_setup_digitalocean() {
    local config_file
    config_file="$(_infra_provider_config_path "digitalocean")"
    
    echo -e "${CYAN}"
    cat << 'EOF'
    ╭──────────────────────────────────────────────────────────╮
    │  🌊 DigitalOcean Global Configuration                     │
    ╰──────────────────────────────────────────────────────────╯
EOF
    echo -e "${NC}"
    
    # Check for existing config
    if [[ -f "$config_file" ]]; then
        # shellcheck source=/dev/null
        source "$config_file"
        echo -e "Current global configuration:"
        echo -e "  Token: ${BOLD}${DO_API_TOKEN:+***configured***}${NC}"
        echo -e "  Spaces Key: ${BOLD}${DO_SPACES_ACCESS_KEY:+***configured***}${NC}"
        echo ""
        read -r -p "Reconfigure? [y/N]: " reconfigure
        if [[ ! "$reconfigure" =~ ^[Yy] ]]; then
            echo "Keeping existing configuration."
            return 0
        fi
        echo ""
    fi
    
    echo -e "${BOLD}Step 1: API Token${NC}"
    echo ""
    echo "Create a Personal Access Token at:"
    echo "  https://cloud.digitalocean.com/account/api/tokens"
    echo ""
    echo "Required scopes: Read + Write"
    echo ""
    read -r -p "Enter API Token: " DO_API_TOKEN
    
    if [[ -z "$DO_API_TOKEN" ]]; then
        _infra_die "API Token is required."
    fi
    
    echo ""
    echo -e "${BOLD}Step 2: Spaces Configuration (Optional)${NC}"
    echo ""
    echo "For object storage (Spaces), create access keys at:"
    echo "  https://cloud.digitalocean.com/account/api/spaces"
    echo ""
    read -r -p "Enter Spaces Access Key ID (or press Enter to skip): " DO_SPACES_ACCESS_KEY
    
    local DO_SPACES_SECRET_KEY=""
    if [[ -n "$DO_SPACES_ACCESS_KEY" ]]; then
        read -r -p "Enter Spaces Secret Key: " DO_SPACES_SECRET_KEY
        read -r -p "Default region [nyc3]: " DO_SPACES_REGION
        DO_SPACES_REGION="${DO_SPACES_REGION:-nyc3}"
    fi
    
    echo ""
    echo -e "${BOLD}Step 3: Default Region${NC}"
    echo ""
    echo "Common regions: nyc1, nyc3, sfo2, sfo3, ams3, sgp1, lon1, fra1"
    read -r -p "Default region [nyc3]: " DO_DEFAULT_REGION
    DO_DEFAULT_REGION="${DO_DEFAULT_REGION:-nyc3}"
    
    # Save configuration
    echo ""
    _infra_log "Saving global configuration..."
    
    cat > "$config_file" << EOF
# DigitalOcean Global Configuration
# Generated by mx infra setup digitalocean on $(date)
#
# This is the global config stored in ~/.mech-crate/config/infra/
# Projects can link to this or override with project-local credentials.

# API Token (Personal Access Token with Read+Write)
DO_API_TOKEN=$DO_API_TOKEN

# Default region for droplets/resources
DO_DEFAULT_REGION=$DO_DEFAULT_REGION
EOF
    
    if [[ -n "$DO_SPACES_ACCESS_KEY" ]]; then
        cat >> "$config_file" << EOF

# Spaces (Object Storage) Configuration
DO_SPACES_ACCESS_KEY=$DO_SPACES_ACCESS_KEY
DO_SPACES_SECRET_KEY=$DO_SPACES_SECRET_KEY
DO_SPACES_REGION=${DO_SPACES_REGION:-$DO_DEFAULT_REGION}
EOF
    fi
    
    chmod 600 "$config_file"
    
    echo ""
    success "DigitalOcean global configuration saved!"
}

_infra_setup_aws() {
    local config_file
    config_file="$(_infra_provider_config_path "aws")"
    
    echo -e "${CYAN}"
    cat << 'EOF'
    ╭──────────────────────────────────────────────────────────╮
    │  🔶 AWS Global Configuration                              │
    ╰──────────────────────────────────────────────────────────╯
EOF
    echo -e "${NC}"
    
    # Check for existing config
    if [[ -f "$config_file" ]]; then
        # shellcheck source=/dev/null
        source "$config_file"
        echo -e "Current global configuration:"
        echo -e "  Access Key: ${BOLD}${AWS_ACCESS_KEY_ID:+***configured***}${NC}"
        echo -e "  Region: ${BOLD}${AWS_DEFAULT_REGION:-not set}${NC}"
        echo ""
        read -r -p "Reconfigure? [y/N]: " reconfigure
        if [[ ! "$reconfigure" =~ ^[Yy] ]]; then
            echo "Keeping existing configuration."
            return 0
        fi
        echo ""
    fi
    
    echo -e "${BOLD}Step 1: AWS Credentials${NC}"
    echo ""
    echo "Create access keys at:"
    echo "  https://console.aws.amazon.com/iam/home#/security_credentials"
    echo ""
    echo "Or use IAM user credentials for better security."
    echo ""
    read -r -p "Enter AWS Access Key ID: " AWS_ACCESS_KEY_ID
    
    if [[ -z "$AWS_ACCESS_KEY_ID" ]]; then
        _infra_die "Access Key ID is required."
    fi
    
    read -r -p "Enter AWS Secret Access Key: " AWS_SECRET_ACCESS_KEY
    
    if [[ -z "$AWS_SECRET_ACCESS_KEY" ]]; then
        _infra_die "Secret Access Key is required."
    fi
    
    echo ""
    echo -e "${BOLD}Step 2: Default Region${NC}"
    echo ""
    echo "Common regions: us-east-1, us-west-2, eu-west-1, ap-southeast-1"
    read -r -p "Default region [us-east-1]: " AWS_DEFAULT_REGION
    AWS_DEFAULT_REGION="${AWS_DEFAULT_REGION:-us-east-1}"
    
    # Save configuration
    echo ""
    _infra_log "Saving global configuration..."
    
    cat > "$config_file" << EOF
# AWS Global Configuration
# Generated by mx infra setup aws on $(date)
#
# This is the global config stored in ~/.mech-crate/config/infra/
# Projects can link to this or override with project-local credentials.

# AWS Credentials
AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY

# Default region
AWS_DEFAULT_REGION=$AWS_DEFAULT_REGION
EOF
    
    chmod 600 "$config_file"
    
    echo ""
    success "AWS global configuration saved!"
}

_infra_setup_hetzner() {
    local config_file
    config_file="$(_infra_provider_config_path "hetzner")"
    
    echo -e "${CYAN}"
    cat << 'EOF'
    ╭──────────────────────────────────────────────────────────╮
    │  🔷 Hetzner Global Configuration                          │
    ╰──────────────────────────────────────────────────────────╯
EOF
    echo -e "${NC}"
    
    # Check for existing config
    if [[ -f "$config_file" ]]; then
        # shellcheck source=/dev/null
        source "$config_file"
        echo -e "Current global configuration:"
        echo -e "  API Token: ${BOLD}${HETZNER_API_TOKEN:+***configured***}${NC}"
        echo ""
        read -r -p "Reconfigure? [y/N]: " reconfigure
        if [[ ! "$reconfigure" =~ ^[Yy] ]]; then
            echo "Keeping existing configuration."
            return 0
        fi
        echo ""
    fi
    
    echo -e "${BOLD}Step 1: API Token${NC}"
    echo ""
    echo "Create an API token at:"
    echo "  https://console.hetzner.cloud/projects → Select project → Security → API Tokens"
    echo ""
    echo "Select: Read & Write permissions"
    echo ""
    read -r -p "Enter API Token: " HETZNER_API_TOKEN
    
    if [[ -z "$HETZNER_API_TOKEN" ]]; then
        _infra_die "API Token is required."
    fi
    
    echo ""
    echo -e "${BOLD}Step 2: Default Location${NC}"
    echo ""
    echo "Common locations: fsn1 (Falkenstein), nbg1 (Nuremberg), hel1 (Helsinki)"
    echo "                  ash (Ashburn), hil (Hillsboro)"
    read -r -p "Default location [fsn1]: " HETZNER_DEFAULT_LOCATION
    HETZNER_DEFAULT_LOCATION="${HETZNER_DEFAULT_LOCATION:-fsn1}"
    
    # Save configuration
    echo ""
    _infra_log "Saving global configuration..."
    
    cat > "$config_file" << EOF
# Hetzner Global Configuration
# Generated by mx infra setup hetzner on $(date)
#
# This is the global config stored in ~/.mech-crate/config/infra/
# Projects can link to this or override with project-local credentials.

# API Token
HETZNER_API_TOKEN=$HETZNER_API_TOKEN

# Default location
HETZNER_DEFAULT_LOCATION=$HETZNER_DEFAULT_LOCATION
EOF
    
    chmod 600 "$config_file"
    
    echo ""
    success "Hetzner global configuration saved!"
}

_infra_setup() {
    local provider="${1:-}"
    
    _infra_ensure_config_dir
    
    if [[ -z "$provider" ]]; then
        # Interactive provider selection
        echo -e "${BOLD}Select an infrastructure provider to configure:${NC}"
        echo ""
        echo "  1) cloudflare    - Cloudflare Workers & Containers"
        echo "  2) digitalocean  - DigitalOcean Droplets & App Platform"
        echo "  3) aws           - Amazon Web Services"
        echo "  4) hetzner       - Hetzner Cloud"
        echo ""
        read -r -p "Enter choice [1-4]: " choice
        
        case "$choice" in
            1) provider="cloudflare" ;;
            2) provider="digitalocean" ;;
            3) provider="aws" ;;
            4) provider="hetzner" ;;
            *) _infra_die "Invalid choice." ;;
        esac
        echo ""
    fi
    
    if ! _infra_provider_exists "$provider"; then
        _infra_die "Unknown provider: $provider. Available: ${MX_INFRA_PROVIDERS[*]}"
    fi
    
    case "$provider" in
        cloudflare)
            _infra_setup_cloudflare
            ;;
        digitalocean)
            _infra_setup_digitalocean
            ;;
        aws)
            _infra_setup_aws
            ;;
        hetzner)
            _infra_setup_hetzner
            ;;
    esac
}

# ─────────────────────────────────────────────────────────────────────────────
# List/Inspect Commands
# ─────────────────────────────────────────────────────────────────────────────

_infra_list() {
    _infra_ensure_config_dir
    
    echo -e "${BOLD}Global Infrastructure Configurations${NC}"
    echo -e "${CYAN}Location: ${MX_INFRA_CONFIG_DIR}${NC}"
    echo ""
    
    local found=false
    for provider in "${MX_INFRA_PROVIDERS[@]}"; do
        local config_file
        config_file="$(_infra_provider_config_path "$provider")"
        
        if [[ -f "$config_file" ]]; then
            found=true
            echo -e "  ${GREEN}●${NC} ${BOLD}$provider${NC}"
            
            # Show key info without exposing secrets
            case "$provider" in
                cloudflare)
                    local account_id
                    account_id=$(grep "^CF_ACCOUNT_ID=" "$config_file" 2>/dev/null | cut -d'=' -f2-)
                    [[ -n "$account_id" ]] && echo "      Account ID: $account_id"
                    ;;
                digitalocean)
                    local region
                    region=$(grep "^DO_DEFAULT_REGION=" "$config_file" 2>/dev/null | cut -d'=' -f2-)
                    [[ -n "$region" ]] && echo "      Region: $region"
                    ;;
                aws)
                    local region
                    region=$(grep "^AWS_DEFAULT_REGION=" "$config_file" 2>/dev/null | cut -d'=' -f2-)
                    [[ -n "$region" ]] && echo "      Region: $region"
                    ;;
                hetzner)
                    local location
                    location=$(grep "^HETZNER_DEFAULT_LOCATION=" "$config_file" 2>/dev/null | cut -d'=' -f2-)
                    [[ -n "$location" ]] && echo "      Location: $location"
                    ;;
            esac
        else
            echo -e "  ${RED}○${NC} $provider (not configured)"
        fi
    done
    
    echo ""
    
    # Show project-level configs if in a project
    if is_mech_crate_project; then
        echo -e "${BOLD}Project-Level Configurations${NC}"
        local project_found=false
        
        for provider in "${MX_INFRA_PROVIDERS[@]}"; do
            local project_config
            project_config="$(_infra_project_config_path "$provider")"
            
            if [[ -f "$project_config" ]]; then
                project_found=true
                if _infra_is_linked "$provider"; then
                    echo -e "  ${BLUE}↗${NC} ${BOLD}$provider${NC} (linked to global)"
                else
                    echo -e "  ${GREEN}●${NC} ${BOLD}$provider${NC} (project-local)"
                fi
            fi
        done
        
        if [[ "$project_found" == "false" ]]; then
            echo "  (no project-level configs)"
        fi
        echo ""
    fi
    
    echo "Run 'mx infra setup <provider>' to configure a provider."
}

_infra_inspect() {
    local provider="${1:-}"
    
    if [[ -z "$provider" ]]; then
        _infra_die "Usage: mx infra inspect <provider>"
    fi
    
    if ! _infra_provider_exists "$provider"; then
        _infra_die "Unknown provider: $provider. Available: ${MX_INFRA_PROVIDERS[*]}"
    fi
    
    local global_config project_config resolved_config
    global_config="$(_infra_provider_config_path "$provider")"
    project_config="$(_infra_project_config_path "$provider")"
    resolved_config="$(infra_resolve_config "$provider" || echo "")"
    
    echo -e "${BOLD}$provider Configuration${NC}"
    echo ""
    
    echo -e "${CYAN}Locations:${NC}"
    echo "  Global:  $global_config"
    echo "  Project: $project_config"
    echo ""
    
    echo -e "${CYAN}Status:${NC}"
    
    if [[ -f "$global_config" ]]; then
        echo -e "  Global:  ${GREEN}configured${NC}"
    else
        echo -e "  Global:  ${RED}not configured${NC}"
    fi
    
    if is_mech_crate_project; then
        if [[ -f "$project_config" ]]; then
            if _infra_is_linked "$provider"; then
                echo -e "  Project: ${BLUE}linked to global${NC}"
            else
                echo -e "  Project: ${GREEN}configured (local override)${NC}"
            fi
        else
            echo -e "  Project: ${YELLOW}not configured${NC}"
        fi
    fi
    
    echo ""
    echo -e "${CYAN}Active Config:${NC}"
    if [[ -n "$resolved_config" ]]; then
        echo "  $resolved_config"
        echo ""
        echo -e "${CYAN}Contents (secrets masked):${NC}"
        while IFS= read -r line; do
            # Skip comments and empty lines
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            [[ -z "$line" ]] && continue
            
            # Mask secret values
            if [[ "$line" =~ (TOKEN|SECRET|PASSWORD|KEY)= ]]; then
                local key="${line%%=*}"
                echo "  $key=***masked***"
            else
                echo "  $line"
            fi
        done < "$resolved_config"
    else
        echo "  (none - run 'mx infra setup $provider' to configure)"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Link/Unlink Commands (Project-level)
# ─────────────────────────────────────────────────────────────────────────────

_infra_link() {
    local provider="${1:-}"
    
    if ! is_mech_crate_project; then
        _infra_die "Not in a MechCrate project. Run 'mx new <name>' first."
    fi
    
    if [[ -z "$provider" ]]; then
        _infra_die "Usage: mx infra link <provider>"
    fi
    
    if ! _infra_provider_exists "$provider"; then
        _infra_die "Unknown provider: $provider. Available: ${MX_INFRA_PROVIDERS[*]}"
    fi
    
    local global_config
    global_config="$(_infra_provider_config_path "$provider")"
    
    if [[ ! -f "$global_config" ]]; then
        _infra_die "No global config for $provider. Run 'mx infra setup $provider' first."
    fi
    
    local project_config project_dir
    project_config="$(_infra_project_config_path "$provider")"
    project_dir="$(dirname "$project_config")"
    
    mkdir -p "$project_dir"
    
    # Create a link marker file
    cat > "$project_config" << EOF
# $provider configuration for this project
# Generated by mx infra link on $(date)
#
# This project is linked to the global config at:
#   $global_config
#
# To use project-specific credentials instead, remove this file
# and run the provider's setup wizard (e.g., make cf-setup).

MX_INFRA_USE_GLOBAL=true
MX_INFRA_PROVIDER=$provider
MX_INFRA_LINKED_AT=$(date -Iseconds)
EOF
    
    success "Linked $provider to global configuration"
    echo ""
    info "This project will now use credentials from:"
    echo "    $global_config"
    echo ""
    info "To use project-specific credentials instead:"
    echo "    rm $project_config"
    echo "    mx cf setup  # (or provider-specific setup)"
}

_infra_unlink() {
    local provider="${1:-}"
    
    if ! is_mech_crate_project; then
        _infra_die "Not in a MechCrate project. Run 'mx new <name>' first."
    fi
    
    if [[ -z "$provider" ]]; then
        _infra_die "Usage: mx infra unlink <provider>"
    fi
    
    local project_config
    project_config="$(_infra_project_config_path "$provider")"
    
    if [[ ! -f "$project_config" ]]; then
        warn "No project config for $provider"
        return 0
    fi
    
    if ! _infra_is_linked "$provider"; then
        warn "Project config for $provider is not linked to global (it's a local override)"
        read -r -p "Remove the project config anyway? [y/N]: " confirm
        if [[ ! "$confirm" =~ ^[Yy] ]]; then
            return 0
        fi
    fi
    
    rm "$project_config"
    success "Unlinked $provider from global configuration"
    echo ""
    info "To configure project-specific credentials:"
    echo "    mx cf setup  # (or provider-specific setup)"
}

_infra_remove() {
    local provider="${1:-}"
    
    if [[ -z "$provider" ]]; then
        _infra_die "Usage: mx infra remove <provider>"
    fi
    
    if ! _infra_provider_exists "$provider"; then
        _infra_die "Unknown provider: $provider. Available: ${MX_INFRA_PROVIDERS[*]}"
    fi
    
    local global_config
    global_config="$(_infra_provider_config_path "$provider")"
    
    if [[ ! -f "$global_config" ]]; then
        warn "No global config for $provider"
        return 0
    fi
    
    echo -e "${YELLOW}This will remove the global $provider configuration.${NC}"
    echo "Projects linked to this config will stop working."
    echo ""
    read -r -p "Are you sure? [y/N]: " confirm
    
    if [[ "$confirm" =~ ^[Yy] ]]; then
        rm "$global_config"
        success "Removed global $provider configuration"
    else
        echo "Cancelled."
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────

_infra_help() {
    echo -e "${BOLD}mx infra${NC} - Global Infrastructure Configuration"
    echo ""
    echo -e "${BOLD}USAGE:${NC}"
    echo "    mx infra <command> [provider]"
    echo ""
    echo -e "${BOLD}COMMANDS:${NC}"
    echo "    setup [provider]      Configure credentials for a provider"
    echo "    list                  List all configured providers"
    echo "    inspect <provider>    Show detailed config for a provider"
    echo "    link <provider>       Link project to global config (from project)"
    echo "    unlink <provider>     Remove project link to global config"
    echo "    remove <provider>     Remove global config for a provider"
    echo ""
    echo -e "${BOLD}PROVIDERS:${NC}"
    echo "    cloudflare            Cloudflare Workers & Containers"
    echo "    digitalocean          DigitalOcean Droplets & App Platform"
    echo "    aws                   Amazon Web Services"
    echo "    hetzner               Hetzner Cloud"
    echo ""
    echo -e "${BOLD}HIERARCHICAL CONFIG:${NC}"
    echo "    When a project needs credentials, MechCrate looks in this order:"
    echo ""
    echo "    1. Project-local config (./infra/<provider>/.env.<provider>)"
    echo "       → Used if exists AND is not linked to global"
    echo ""
    echo "    2. Global config (~/.mech-crate/config/infra/<provider>.env)"
    echo "       → Used if project has no local config, or is linked"
    echo ""
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    mx infra setup                    # Interactive provider selection"
    echo "    mx infra setup cloudflare         # Configure Cloudflare globally"
    echo "    mx infra setup digitalocean       # Configure DigitalOcean globally"
    echo "    mx infra list                     # Show all configs"
    echo "    mx infra inspect cloudflare       # Show Cloudflare config details"
    echo ""
    echo -e "${BOLD}PROJECT LINKING:${NC}"
    echo "    # From within a MechCrate project:"
    echo "    mx infra link cloudflare          # Use global Cloudflare config"
    echo "    mx infra unlink cloudflare        # Stop using global config"
    echo ""
    echo -e "${BOLD}WORKFLOW:${NC}"
    echo "    1. Set up global config once:     mx infra setup cloudflare"
    echo "    2. Create projects:               mx new myproject --infra cloudflare"
    echo "    3. Link to global (optional):     mx infra link cloudflare"
    echo "       Or use project-local creds:    mx cf setup"
    echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Main Command Handler
# ─────────────────────────────────────────────────────────────────────────────

infra_cmd() {
    local subcommand="${1:-help}"
    shift 2>/dev/null || true
    
    case "$subcommand" in
        setup|configure)
            _infra_setup "$@"
            ;;
        list|ls)
            _infra_list "$@"
            ;;
        inspect|show|info)
            _infra_inspect "$@"
            ;;
        link)
            _infra_link "$@"
            ;;
        unlink)
            _infra_unlink "$@"
            ;;
        remove|rm|delete)
            _infra_remove "$@"
            ;;
        help|--help|-h)
            _infra_help
            ;;
        *)
            error "Unknown infra command: $subcommand. Run 'mx infra help' for usage."
            ;;
    esac
}
