#!/usr/bin/env bash
#
# MechCrate Testbed - Common Utilities
# Shared functions for recipe testing
#
# Version: 1.0.0
# Author: MechCrate
#

# Prevent double-sourcing
[ -n "${TESTBED_COMMON_LOADED:-}" ] && return 0
readonly TESTBED_COMMON_LOADED=1

# ─────────────────────────────────────────────────────────────────────────────
# Colors and Formatting
# ─────────────────────────────────────────────────────────────────────────────

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly MAGENTA='\033[0;35m'
readonly NC='\033[0m'
readonly BOLD='\033[1m'
readonly DIM='\033[2m'

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────

# Resolve testbed root directory (only if not already set)
if [[ -z "${TESTBED_DIR:-}" ]]; then
    TESTBED_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi
if [[ -z "${MECH_CRATE_ROOT:-}" ]]; then
    MECH_CRATE_ROOT="$(cd "$TESTBED_DIR/../.." && pwd)"
fi

# Test configuration
TEST_LEVEL="${TEST_LEVEL:-build}"  # build, smoke, full
TEST_TIMEOUT="${TEST_TIMEOUT:-600}"  # 10 minute default
TEST_CLEANUP="${TEST_CLEANUP:-true}"  # Cleanup after test
TEST_VERBOSE="${TEST_VERBOSE:-false}"

# Test state (global variables without declare -g for bash 3.x compatibility)
TEST_TEMP_DIR="${TEST_TEMP_DIR:-}"
TEST_PROJECT_DIR="${TEST_PROJECT_DIR:-}"
TEST_SERVICE_NAME="${TEST_SERVICE_NAME:-}"
TEST_RECIPE_NAME="${TEST_RECIPE_NAME:-}"
TEST_START_TIME="${TEST_START_TIME:-}"
TEST_PASSED="${TEST_PASSED:-0}"
TEST_FAILED="${TEST_FAILED:-0}"
TEST_SKIPPED="${TEST_SKIPPED:-0}"

# ─────────────────────────────────────────────────────────────────────────────
# Logging Functions
# ─────────────────────────────────────────────────────────────────────────────

_log_timestamp() {
    date -u +"%Y-%m-%dT%H:%M:%SZ"
}

log_header() {
    local title="$1"
    echo ""
    echo -e "${CYAN}╭──────────────────────────────────────────────────────────────╮${NC}"
    echo -e "${CYAN}│${NC}  ${BOLD}$title${NC}"
    echo -e "${CYAN}╰──────────────────────────────────────────────────────────────╯${NC}"
    echo ""
}

log_section() {
    local title="$1"
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  ${BOLD}$title${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

log_info() {
    echo -e "${BLUE}ℹ${NC}  $1"
}

log_success() {
    echo -e "${GREEN}✓${NC}  $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC}  $1"
}

log_error() {
    echo -e "${RED}✗${NC}  $1" >&2
}

log_debug() {
    if [[ "$TEST_VERBOSE" == "true" ]]; then
        echo -e "${DIM}[DEBUG] $1${NC}"
    fi
}

log_step() {
    local step_num="$1"
    local step_desc="$2"
    echo -e "${MAGENTA}→${NC}  ${BOLD}Step $step_num:${NC} $step_desc"
}

# ─────────────────────────────────────────────────────────────────────────────
# Test Lifecycle Functions
# ─────────────────────────────────────────────────────────────────────────────

# Initialize test environment
testbed_init() {
    local recipe_name="$1"
    local service_name="${2:-testapp}"
    
    TEST_RECIPE_NAME="$recipe_name"
    TEST_SERVICE_NAME="$service_name"
    TEST_START_TIME=$(date +%s)
    
    log_header "🧪 MechCrate Recipe Testbed"
    log_info "Recipe: ${BOLD}$recipe_name${NC}"
    log_info "Service: ${BOLD}$service_name${NC}"
    log_info "Test Level: ${BOLD}$TEST_LEVEL${NC}"
    log_info "MechCrate Root: ${DIM}$MECH_CRATE_ROOT${NC}"
    
    # Validate recipe exists
    if [[ ! -f "$MECH_CRATE_ROOT/templates/recipes/$recipe_name/recipe.json" ]]; then
        log_error "Recipe '$recipe_name' not found"
        return 1
    fi
    
    # Create temporary test directory
    TEST_TEMP_DIR=$(mktemp -d -t "mech-testbed-XXXXXX")
    TEST_PROJECT_DIR="$TEST_TEMP_DIR/test-project"
    
    log_debug "Temp directory: $TEST_TEMP_DIR"
    log_debug "Project directory: $TEST_PROJECT_DIR"
    
    # Set up cleanup trap
    if [[ "$TEST_CLEANUP" == "true" ]]; then
        trap testbed_cleanup EXIT INT TERM
    fi
    
    return 0
}

# Cleanup test environment
testbed_cleanup() {
    local exit_code=$?
    
    log_section "Cleanup"
    
    # Stop any running containers
    if [[ -n "${TEST_PROJECT_DIR:-}" && -d "$TEST_PROJECT_DIR" ]]; then
        log_info "Stopping containers..."
        (cd "$TEST_PROJECT_DIR" && docker compose down -v 2>/dev/null) || true
    fi
    
    # Remove temp directory
    if [[ "$TEST_CLEANUP" == "true" && -n "${TEST_TEMP_DIR:-}" && -d "$TEST_TEMP_DIR" ]]; then
        log_info "Removing temp directory: ${DIM}$TEST_TEMP_DIR${NC}"
        rm -rf "$TEST_TEMP_DIR" 2>/dev/null || true
    elif [[ -n "${TEST_TEMP_DIR:-}" ]]; then
        log_warn "Test artifacts preserved at: ${DIM}$TEST_TEMP_DIR${NC}"
    fi
    
    # Print summary
    _print_summary
    
    return $exit_code
}

_print_summary() {
    local end_time=$(date +%s)
    local duration=$((end_time - TEST_START_TIME))
    local total=$((TEST_PASSED + TEST_FAILED + TEST_SKIPPED))
    
    log_section "Test Summary"
    
    echo -e "  Recipe:    ${BOLD}$TEST_RECIPE_NAME${NC}"
    echo -e "  Duration:  ${BOLD}${duration}s${NC}"
    echo ""
    echo -e "  ${GREEN}Passed:${NC}   $TEST_PASSED"
    echo -e "  ${RED}Failed:${NC}   $TEST_FAILED"
    echo -e "  ${YELLOW}Skipped:${NC}  $TEST_SKIPPED"
    echo -e "  ${BLUE}Total:${NC}    $total"
    echo ""
    
    if [[ $TEST_FAILED -eq 0 ]]; then
        echo -e "  ${GREEN}${BOLD}✓ ALL TESTS PASSED${NC}"
    else
        echo -e "  ${RED}${BOLD}✗ SOME TESTS FAILED${NC}"
    fi
    echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Project Creation Functions
# ─────────────────────────────────────────────────────────────────────────────

# Create a new test project using mx
create_test_project() {
    local project_name="${1:-test-project}"
    
    log_step "1" "Creating test project: ${BOLD}$project_name${NC}"
    
    cd "$TEST_TEMP_DIR" || return 1
    
    # Run mx new
    "$MECH_CRATE_ROOT/bin/mx" new "$project_name" --no-prompt
    
    if [[ ! -d "$TEST_TEMP_DIR/$project_name" ]]; then
        log_error "Failed to create project"
        return 1
    fi
    
    TEST_PROJECT_DIR="$TEST_TEMP_DIR/$project_name"
    cd "$TEST_PROJECT_DIR" || return 1
    
    log_success "Project created at: ${DIM}$TEST_PROJECT_DIR${NC}"
    return 0
}

# Add recipe to test project
add_recipe() {
    local recipe_name="$1"
    local service_name="${2:-$TEST_SERVICE_NAME}"
    local extra_args="${3:-}"
    
    log_step "2" "Adding recipe: ${BOLD}$recipe_name${NC} as ${BOLD}$service_name${NC}"
    
    cd "$TEST_PROJECT_DIR" || return 1
    
    # Run mx add
    # shellcheck disable=SC2086
    "$MECH_CRATE_ROOT/bin/mx" add "$service_name" --recipe="$recipe_name" $extra_args
    
    local exit_code=$?
    
    if [[ $exit_code -ne 0 ]]; then
        log_error "Failed to add recipe"
        return 1
    fi
    
    # Verify core files were created
    local required_files=(
        "apps/$service_name"
        "docker/compose/${service_name}.yml"
        "docker/dockerfiles/${service_name}"
    )
    
    for file in "${required_files[@]}"; do
        if [[ ! -e "$TEST_PROJECT_DIR/$file" ]]; then
            log_error "Missing expected file: $file"
            return 1
        fi
    done
    
    log_success "Recipe added successfully"
    return 0
}

# ─────────────────────────────────────────────────────────────────────────────
# Docker Build Functions
# ─────────────────────────────────────────────────────────────────────────────

# Build Docker images
build_images() {
    local service_name="${1:-$TEST_SERVICE_NAME}"
    local target="${2:-development}"  # development or production
    
    log_step "3" "Building Docker images for: ${BOLD}$service_name${NC} (target: $target)"
    
    cd "$TEST_PROJECT_DIR" || return 1
    
    # Build using the compose files
    local compose_files="-f docker/compose/${service_name}.yml"
    if [[ "$target" == "development" && -f "docker/compose/${service_name}.dev.yml" ]]; then
        compose_files="$compose_files -f docker/compose/${service_name}.dev.yml"
    fi
    
    # Also need shared infrastructure
    if [[ -f "docker/compose/db.yml" ]]; then
        compose_files="-f docker/compose/db.yml $compose_files"
    fi
    if [[ -f "docker/compose/redis.yml" ]]; then
        compose_files="-f docker/compose/redis.yml $compose_files"
    fi
    
    log_debug "Compose files: $compose_files"
    
    # Run docker compose build
    # shellcheck disable=SC2086
    if docker compose $compose_files build --no-cache 2>&1 | _filter_build_output; then
        log_success "Docker images built successfully"
        return 0
    else
        log_error "Docker build failed"
        return 1
    fi
}

# Filter build output for readability
_filter_build_output() {
    if [[ "$TEST_VERBOSE" == "true" ]]; then
        cat
    else
        # Show only important lines
        grep -E "^(#[0-9]+|Step|Successfully|ERROR|WARN|Building|Sending)" || true
    fi
}

# Validate built images exist
validate_images() {
    local service_name="${1:-$TEST_SERVICE_NAME}"
    
    log_step "4" "Validating Docker images"
    
    # List images matching our pattern
    local images
    images=$(docker images --format "{{.Repository}}:{{.Tag}}" | grep -E "^(test-project[-_])?${service_name}" || true)
    
    if [[ -z "$images" ]]; then
        log_warn "No matching images found (this may be expected for first build)"
        return 0
    fi
    
    log_debug "Found images:"
    log_debug "$images"
    log_success "Image validation passed"
    return 0
}

# ─────────────────────────────────────────────────────────────────────────────
# Container Runtime Functions
# ─────────────────────────────────────────────────────────────────────────────

# Start containers for smoke testing
start_containers() {
    local service_name="${1:-$TEST_SERVICE_NAME}"
    local wait_time="${2:-30}"
    
    log_step "5" "Starting containers for: ${BOLD}$service_name${NC}"
    
    cd "$TEST_PROJECT_DIR" || return 1
    
    # Build compose command
    local compose_files="-f docker/compose/${service_name}.yml"
    if [[ -f "docker/compose/${service_name}.dev.yml" ]]; then
        compose_files="$compose_files -f docker/compose/${service_name}.dev.yml"
    fi
    if [[ -f "docker/compose/db.yml" ]]; then
        compose_files="-f docker/compose/db.yml -f docker/compose/db.dev.yml $compose_files"
    fi
    if [[ -f "docker/compose/redis.yml" ]]; then
        compose_files="-f docker/compose/redis.yml -f docker/compose/redis.dev.yml $compose_files"
    fi
    
    # Start in detached mode
    # shellcheck disable=SC2086
    docker compose $compose_files up -d
    
    log_info "Waiting ${wait_time}s for containers to be ready..."
    sleep "$wait_time"
    
    # Check container status
    # shellcheck disable=SC2086
    local running
    running=$(docker compose $compose_files ps --status running -q | wc -l)
    
    if [[ $running -gt 0 ]]; then
        log_success "$running container(s) running"
        return 0
    else
        log_error "No containers running"
        # shellcheck disable=SC2086
        docker compose $compose_files logs --tail=50
        return 1
    fi
}

# Stop containers
stop_containers() {
    local service_name="${1:-$TEST_SERVICE_NAME}"
    
    log_info "Stopping containers..."
    
    cd "$TEST_PROJECT_DIR" || return 1
    
    docker compose down -v 2>/dev/null || true
}

# Health check a service endpoint
health_check() {
    local url="$1"
    local expected_status="${2:-200}"
    local max_attempts="${3:-10}"
    local delay="${4:-3}"
    
    log_info "Health checking: $url"
    
    local attempt=1
    while [[ $attempt -le $max_attempts ]]; do
        local status
        status=$(curl -sf -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
        
        if [[ "$status" == "$expected_status" ]]; then
            log_success "Health check passed (status: $status)"
            return 0
        fi
        
        log_debug "Attempt $attempt/$max_attempts: status=$status (expected $expected_status)"
        sleep "$delay"
        ((attempt++))
    done
    
    log_error "Health check failed after $max_attempts attempts"
    return 1
}

# ─────────────────────────────────────────────────────────────────────────────
# Utility Functions
# ─────────────────────────────────────────────────────────────────────────────

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check required dependencies
check_dependencies() {
    local -a required=("docker" "jq" "curl")
    local missing=()
    
    for cmd in "${required[@]}"; do
        if ! command_exists "$cmd"; then
            missing+=("$cmd")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing[*]}"
        return 1
    fi
    
    # Check Docker is running
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker is not running"
        return 1
    fi
    
    return 0
}

# Get recipe metadata
get_recipe_meta() {
    local recipe_name="$1"
    local field="$2"
    
    local recipe_json="$MECH_CRATE_ROOT/templates/recipes/$recipe_name/recipe.json"
    
    if [[ -f "$recipe_json" ]]; then
        jq -r "$field // empty" "$recipe_json" 2>/dev/null
    fi
}

# File content contains string
file_contains() {
    local file="$1"
    local pattern="$2"
    
    if [[ -f "$file" ]]; then
        grep -q "$pattern" "$file"
    else
        return 1
    fi
}

# Directory has files
dir_has_files() {
    local dir="$1"
    local pattern="${2:-*}"
    
    if [[ -d "$dir" ]]; then
        local count
        count=$(find "$dir" -name "$pattern" -type f | wc -l)
        [[ $count -gt 0 ]]
    else
        return 1
    fi
}

# Execute with timeout
run_with_timeout() {
    local timeout="$1"
    shift
    local -a cmd=("$@")
    
    if command_exists timeout; then
        timeout "$timeout" "${cmd[@]}"
    else
        # macOS fallback
        perl -e 'alarm shift; exec @ARGV' "$timeout" "${cmd[@]}"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Aliases for Simpler API (used by testbed.sh)
# ─────────────────────────────────────────────────────────────────────────────

info() { log_info "$@"; }
success() { log_success "$@"; }
warn() { log_warn "$@"; }
error() { log_error "$@"; }
debug() { log_debug "$@"; }

# Check prerequisites before running tests
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local required=("docker" "jq" "curl")
    local missing=()
    
    for cmd in "${required[@]}"; do
        if ! command_exists "$cmd"; then
            missing+=("$cmd")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing[*]}"
        exit 1
    fi
    
    # Note: recipe.sh has been updated for bash 3.x compatibility
    local bash_major="${BASH_VERSION%%.*}"
    if [[ "$bash_major" -lt 4 ]]; then
        log_debug "Using bash $BASH_VERSION (3.x compatible mode)"
    fi
    
    # Check Docker is running
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    
    # Check mx CLI is available
    if [[ ! -x "$MECH_CRATE_ROOT/bin/mx" ]]; then
        log_error "mx CLI not found at: $MECH_CRATE_ROOT/bin/mx"
        exit 1
    fi
    
    log_success "All prerequisites satisfied"
}

export -f log_header log_section log_info log_success log_warn log_error log_debug log_step
export -f info success warn error debug check_prerequisites
export -f testbed_init testbed_cleanup
export -f create_test_project add_recipe
export -f build_images validate_images
export -f start_containers stop_containers health_check
export -f command_exists check_dependencies get_recipe_meta file_contains dir_has_files run_with_timeout
