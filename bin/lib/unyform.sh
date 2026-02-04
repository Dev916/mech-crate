#!/bin/bash
#
# MechCrate Unyform Integration
# Connect to Unyform.ai for organizational recipes and AI governance
#

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────

UNYFORM_CONFIG_DIR="${HOME}/.mech-crate/config/unyform"
UNYFORM_RECIPES_DIR="${HOME}/.mech-crate/recipes"
UNYFORM_DEFAULT_URL="https://api.unyform.ai"

# Credential files
UNYFORM_CREDENTIALS_FILE="${UNYFORM_CONFIG_DIR}/credentials.json"
UNYFORM_SESSION_FILE="${UNYFORM_CONFIG_DIR}/session.json"

# ─────────────────────────────────────────────────────────────────────────────
# Credential Management
# ─────────────────────────────────────────────────────────────────────────────

# Ensure config directories exist with proper permissions
init_unyform_dirs() {
    mkdir -p "$UNYFORM_CONFIG_DIR"
    mkdir -p "$UNYFORM_RECIPES_DIR"
    chmod 700 "$UNYFORM_CONFIG_DIR"
}

# Get the configured Unyform URL
get_unyform_url() {
    if [[ -f "$UNYFORM_CREDENTIALS_FILE" ]]; then
        local url=$(jq -r '.url // empty' "$UNYFORM_CREDENTIALS_FILE" 2>/dev/null)
        if [[ -n "$url" ]]; then
            echo "$url"
            return
        fi
    fi
    echo "$UNYFORM_DEFAULT_URL"
}

# Get the stored access token
get_access_token() {
    if [[ -f "$UNYFORM_SESSION_FILE" ]]; then
        local token=$(jq -r '.access_token // empty' "$UNYFORM_SESSION_FILE" 2>/dev/null)
        local expires_at=$(jq -r '.expires_at // empty' "$UNYFORM_SESSION_FILE" 2>/dev/null)
        
        # Check if token is expired
        if [[ -n "$expires_at" ]]; then
            local now=$(date +%s)
            local exp=$(date -j -f "%Y-%m-%dT%H:%M:%S" "${expires_at%.*}" +%s 2>/dev/null || echo "0")
            if [[ $now -ge $exp ]]; then
                # Token expired, try to refresh
                refresh_token_if_needed
                # Re-read the token
                token=$(jq -r '.access_token // empty' "$UNYFORM_SESSION_FILE" 2>/dev/null)
            fi
        fi
        
        echo "$token"
    fi
}

# Get the stored API key
get_api_key() {
    if [[ -f "$UNYFORM_CREDENTIALS_FILE" ]]; then
        jq -r '.api_key // empty' "$UNYFORM_CREDENTIALS_FILE" 2>/dev/null
    fi
}

# Save credentials
save_credentials() {
    local api_key="$1"
    local url="$2"
    local org_id="$3"
    
    init_unyform_dirs
    
    cat > "$UNYFORM_CREDENTIALS_FILE" << EOF
{
    "api_key": "$api_key",
    "url": "$url",
    "org_id": "$org_id"
}
EOF
    chmod 600 "$UNYFORM_CREDENTIALS_FILE"
}

# Save session from login response
save_session() {
    local access_token="$1"
    local refresh_token="$2"
    local expires_in="$3"
    local user_json="$4"
    
    init_unyform_dirs
    
    local expires_at=$(date -v +"${expires_in}S" -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || \
                       date -d "+${expires_in} seconds" -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null)
    
    cat > "$UNYFORM_SESSION_FILE" << EOF
{
    "access_token": "$access_token",
    "refresh_token": "$refresh_token",
    "expires_at": "$expires_at",
    "user": $user_json
}
EOF
    chmod 600 "$UNYFORM_SESSION_FILE"
}

# Clear all credentials
clear_credentials() {
    rm -f "$UNYFORM_CREDENTIALS_FILE" "$UNYFORM_SESSION_FILE"
}

# Refresh token if needed
refresh_token_if_needed() {
    if [[ ! -f "$UNYFORM_SESSION_FILE" ]]; then
        return 1
    fi
    
    local refresh_token=$(jq -r '.refresh_token // empty' "$UNYFORM_SESSION_FILE" 2>/dev/null)
    if [[ -z "$refresh_token" ]]; then
        return 1
    fi
    
    local url=$(get_unyform_url)
    local response
    
    response=$(curl -s -X POST "${url}/v1/auth/refresh" \
        -H "Content-Type: application/json" \
        -d "{\"refresh_token\": \"$refresh_token\"}")
    
    if echo "$response" | jq -e '.access_token' >/dev/null 2>&1; then
        local new_token=$(echo "$response" | jq -r '.access_token')
        local expires_in=$(echo "$response" | jq -r '.expires_in')
        local user_json=$(jq '.user' "$UNYFORM_SESSION_FILE")
        
        save_session "$new_token" "$refresh_token" "$expires_in" "$user_json"
        return 0
    fi
    
    return 1
}

# Check if logged in
is_logged_in() {
    local token=$(get_access_token)
    local api_key=$(get_api_key)
    
    [[ -n "$token" || -n "$api_key" ]]
}

# Get auth header for API calls
get_auth_header() {
    local token=$(get_access_token)
    local api_key=$(get_api_key)
    
    if [[ -n "$token" ]]; then
        echo "Authorization: Bearer $token"
    elif [[ -n "$api_key" ]]; then
        echo "X-API-Key: $api_key"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Login Commands
# ─────────────────────────────────────────────────────────────────────────────

# Login with API key
login_with_api_key() {
    local api_key="$1"
    local url="${2:-$UNYFORM_DEFAULT_URL}"
    
    info "Authenticating with Unyform..."
    
    local response
    response=$(curl -s -X POST "${url}/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"api_key\": \"$api_key\"}")
    
    if echo "$response" | jq -e '.error' >/dev/null 2>&1; then
        local error_msg=$(echo "$response" | jq -r '.error.message')
        error "Login failed: $error_msg"
    fi
    
    local access_token=$(echo "$response" | jq -r '.access_token')
    local refresh_token=$(echo "$response" | jq -r '.refresh_token')
    local expires_in=$(echo "$response" | jq -r '.expires_in')
    local user_json=$(echo "$response" | jq '.user')
    local org_id=$(echo "$response" | jq -r '.user.organizations[0].id // empty')
    
    save_credentials "$api_key" "$url" "$org_id"
    save_session "$access_token" "$refresh_token" "$expires_in" "$user_json"
    
    local user_email=$(echo "$response" | jq -r '.user.email')
    local org_name=$(echo "$response" | jq -r '.user.organizations[0].name // "N/A"')
    
    success "Logged in as ${BOLD}$user_email${NC}"
    info "Organization: ${BOLD}$org_name${NC}"
}

# Login via browser OAuth
login_via_browser() {
    local url="${1:-$UNYFORM_DEFAULT_URL}"
    
    info "Opening browser for authentication..."
    
    # Open OAuth URL in browser
    local oauth_url="${url}/v1/auth/oauth/github"
    
    if command -v open &>/dev/null; then
        open "$oauth_url"
    elif command -v xdg-open &>/dev/null; then
        xdg-open "$oauth_url"
    else
        info "Please open this URL in your browser:"
        echo "$oauth_url"
    fi
    
    echo ""
    info "After authorizing, paste your access token below"
    echo -n "Token: "
    read -r access_token
    
    if [[ -z "$access_token" ]]; then
        error "No token provided"
    fi
    
    # Verify the token by calling /me
    local response
    response=$(curl -s -X GET "${url}/v1/auth/me" \
        -H "Authorization: Bearer $access_token")
    
    if echo "$response" | jq -e '.error' >/dev/null 2>&1; then
        local error_msg=$(echo "$response" | jq -r '.error.message')
        error "Token validation failed: $error_msg"
    fi
    
    local user_json="$response"
    local org_id=$(echo "$response" | jq -r '.organizations[0].id // empty')
    
    init_unyform_dirs
    cat > "$UNYFORM_CREDENTIALS_FILE" << EOF
{
    "url": "$url"
}
EOF
    chmod 600 "$UNYFORM_CREDENTIALS_FILE"
    
    # Create session without refresh token for browser login
    save_session "$access_token" "" "3600" "$user_json"
    
    local user_email=$(echo "$response" | jq -r '.email')
    local org_name=$(echo "$response" | jq -r '.organizations[0].name // "N/A"')
    
    success "Logged in as ${BOLD}$user_email${NC}"
    info "Organization: ${BOLD}$org_name${NC}"
}

# ─────────────────────────────────────────────────────────────────────────────
# Main Command Handler
# ─────────────────────────────────────────────────────────────────────────────

unyform_cmd() {
    local subcommand="${1:-help}"
    shift 2>/dev/null || true
    
    case "$subcommand" in
        login)
            unyform_login "$@"
            ;;
        logout)
            unyform_logout "$@"
            ;;
        whoami)
            unyform_whoami "$@"
            ;;
        help|--help|-h)
            unyform_help
            ;;
        *)
            error "Unknown unyform command: $subcommand. Run 'mx unyform help' for usage."
            ;;
    esac
}

# ─────────────────────────────────────────────────────────────────────────────
# Login command
# ─────────────────────────────────────────────────────────────────────────────

unyform_login() {
    local api_key=""
    local url="$UNYFORM_DEFAULT_URL"
    local use_browser=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --api-key)
                api_key="$2"
                shift 2
                ;;
            --url)
                url="$2"
                shift 2
                ;;
            --browser)
                use_browser=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    # If already logged in, warn
    if is_logged_in; then
        warn "Already logged in. Use 'mx logout' first to switch accounts."
        unyform_whoami
        return 0
    fi
    
    if [[ -n "$api_key" ]]; then
        login_with_api_key "$api_key" "$url"
    elif [[ "$use_browser" == true ]]; then
        login_via_browser "$url"
    else
        # Interactive mode - prompt for method
        echo "How would you like to authenticate?"
        echo ""
        echo "  1) API Key (for CI/automation)"
        echo "  2) Browser (GitHub OAuth)"
        echo ""
        echo -n "Choice [1/2]: "
        read -r choice
        
        case "$choice" in
            1)
                echo -n "API Key: "
                read -r api_key
                login_with_api_key "$api_key" "$url"
                ;;
            2)
                login_via_browser "$url"
                ;;
            *)
                error "Invalid choice"
                ;;
        esac
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Logout command
# ─────────────────────────────────────────────────────────────────────────────

unyform_logout() {
    if ! is_logged_in; then
        info "Not logged in"
        return 0
    fi
    
    # Call logout endpoint
    local url=$(get_unyform_url)
    local auth_header=$(get_auth_header)
    
    if [[ -n "$auth_header" ]]; then
        curl -s -X POST "${url}/v1/auth/logout" \
            -H "$auth_header" >/dev/null 2>&1 || true
    fi
    
    clear_credentials
    success "Logged out"
}

# ─────────────────────────────────────────────────────────────────────────────
# Whoami command
# ─────────────────────────────────────────────────────────────────────────────

unyform_whoami() {
    if ! is_logged_in; then
        info "Not logged in. Run 'mx login' to authenticate."
        return 1
    fi
    
    local url=$(get_unyform_url)
    local auth_header=$(get_auth_header)
    
    # Try to get fresh user info
    local response
    response=$(curl -s -X GET "${url}/v1/auth/me" \
        -H "$auth_header" 2>/dev/null)
    
    if echo "$response" | jq -e '.error' >/dev/null 2>&1; then
        # Fallback to cached user info
        if [[ -f "$UNYFORM_SESSION_FILE" ]]; then
            response=$(jq '.user' "$UNYFORM_SESSION_FILE")
        else
            error "Session expired. Please login again."
        fi
    fi
    
    local email=$(echo "$response" | jq -r '.email // "Unknown"')
    local name=$(echo "$response" | jq -r '.name // "Unknown"')
    local org_count=$(echo "$response" | jq -r '.organizations | length')
    
    echo ""
    echo -e "${BOLD}Unyform Account${NC}"
    echo "────────────────────────────────────"
    echo -e "  Email: ${CYAN}$email${NC}"
    echo -e "  Name:  $name"
    echo -e "  URL:   $(get_unyform_url)"
    echo ""
    
    if [[ $org_count -gt 0 ]]; then
        echo -e "${BOLD}Organizations${NC}"
        echo "────────────────────────────────────"
        echo "$response" | jq -r '.organizations[] | "  \(.slug) (\(.role))"'
    fi
    echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────

unyform_help() {
    cat << 'EOF'
Unyform Integration - Connect MechCrate to organizational recipes

USAGE:
    mx login [OPTIONS]         Authenticate with Unyform
    mx logout                  Clear stored credentials
    mx whoami                  Show current user and organizations

LOGIN OPTIONS:
    --api-key KEY             Login with API key (for CI)
    --url URL                 Custom Unyform instance URL
    --browser                 Login via GitHub OAuth in browser

EXAMPLES:
    mx login                  Interactive login
    mx login --api-key uny_xxxxx   Login with API key
    mx login --browser        Login via browser OAuth
    mx login --url https://unyform.mycompany.com   Use self-hosted instance
    
    mx whoami                 Show current authentication status
    mx logout                 Clear credentials

CREDENTIAL STORAGE:
    Credentials are stored in ~/.mech-crate/config/unyform/
    Files are created with 600 permissions (owner read/write only)

EOF
}

# Alias commands at top level
login_cmd() {
    unyform_login "$@"
}

logout_cmd() {
    unyform_logout "$@"
}

whoami_cmd() {
    unyform_whoami "$@"
}
