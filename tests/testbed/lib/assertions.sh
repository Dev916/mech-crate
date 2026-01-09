#!/usr/bin/env bash
#
# MechCrate Testbed - Assertion Functions
# Test assertions for recipe validation
#
# Version: 1.0.0
# Author: MechCrate
#

# Prevent double-sourcing
[ -n "${TESTBED_ASSERTIONS_LOADED:-}" ] && return 0
readonly TESTBED_ASSERTIONS_LOADED=1

# Source common if not already loaded
if [[ -z "${TESTBED_COMMON_LOADED:-}" ]]; then
    source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Test Recording
# ─────────────────────────────────────────────────────────────────────────────

_record_pass() {
    local test_name="$1"
    ((TEST_PASSED++))
    echo -e "    ${GREEN}✓${NC} $test_name"
}

_record_fail() {
    local test_name="$1"
    local message="${2:-}"
    ((TEST_FAILED++))
    echo -e "    ${RED}✗${NC} $test_name"
    if [[ -n "$message" ]]; then
        echo -e "      ${DIM}$message${NC}"
    fi
}

_record_skip() {
    local test_name="$1"
    local reason="${2:-}"
    ((TEST_SKIPPED++))
    echo -e "    ${YELLOW}○${NC} $test_name ${DIM}(skipped${reason:+: $reason})${NC}"
}

# ─────────────────────────────────────────────────────────────────────────────
# Basic Assertions
# ─────────────────────────────────────────────────────────────────────────────

# Assert condition is true
assert() {
    local test_name="$1"
    local condition="$2"
    
    if eval "$condition"; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Condition failed: $condition"
        return 1
    fi
}

# Assert two values are equal
assert_equals() {
    local test_name="$1"
    local expected="$2"
    local actual="$3"
    
    if [[ "$expected" == "$actual" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Expected: '$expected', Actual: '$actual'"
        return 1
    fi
}

# Assert value is not empty
assert_not_empty() {
    local test_name="$1"
    local value="$2"
    
    if [[ -n "$value" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Value is empty"
        return 1
    fi
}

# Assert command succeeds (exit code 0)
assert_success() {
    local test_name="$1"
    shift
    local -a cmd=("$@")
    
    if "${cmd[@]}" >/dev/null 2>&1; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Command failed: ${cmd[*]}"
        return 1
    fi
}

# Assert command fails (exit code non-zero)
assert_failure() {
    local test_name="$1"
    shift
    local -a cmd=("$@")
    
    if ! "${cmd[@]}" >/dev/null 2>&1; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Command should have failed: ${cmd[*]}"
        return 1
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# File System Assertions
# ─────────────────────────────────────────────────────────────────────────────

# Assert file exists
assert_file_exists() {
    local test_name="$1"
    local file_path="$2"
    
    if [[ -f "$file_path" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "File not found: $file_path"
        return 1
    fi
}

# Assert directory exists
assert_dir_exists() {
    local test_name="$1"
    local dir_path="$2"
    
    if [[ -d "$dir_path" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Directory not found: $dir_path"
        return 1
    fi
}

# Assert path exists (file or directory)
assert_path_exists() {
    local test_name="$1"
    local path="$2"
    
    if [[ -e "$path" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Path not found: $path"
        return 1
    fi
}

# Assert file contains pattern
assert_file_contains() {
    local test_name="$1"
    local file_path="$2"
    local pattern="$3"
    
    if [[ ! -f "$file_path" ]]; then
        _record_fail "$test_name" "File not found: $file_path"
        return 1
    fi
    
    if grep -q "$pattern" "$file_path"; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Pattern '$pattern' not found in: $file_path"
        return 1
    fi
}

# Assert file does not contain pattern
assert_file_not_contains() {
    local test_name="$1"
    local file_path="$2"
    local pattern="$3"
    
    if [[ ! -f "$file_path" ]]; then
        _record_fail "$test_name" "File not found: $file_path"
        return 1
    fi
    
    if ! grep -q "$pattern" "$file_path"; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Pattern '$pattern' found in: $file_path (should not be present)"
        return 1
    fi
}

# Assert file is executable
assert_executable() {
    local test_name="$1"
    local file_path="$2"
    
    if [[ -x "$file_path" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "File is not executable: $file_path"
        return 1
    fi
}

# Assert directory is not empty
assert_dir_not_empty() {
    local test_name="$1"
    local dir_path="$2"
    
    if [[ ! -d "$dir_path" ]]; then
        _record_fail "$test_name" "Directory not found: $dir_path"
        return 1
    fi
    
    local count
    count=$(find "$dir_path" -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')
    
    if [[ $count -gt 0 ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Directory is empty: $dir_path"
        return 1
    fi
}

# Assert file count in directory
assert_file_count() {
    local test_name="$1"
    local dir_path="$2"
    local expected_count="$3"
    local pattern="${4:-*}"
    
    if [[ ! -d "$dir_path" ]]; then
        _record_fail "$test_name" "Directory not found: $dir_path"
        return 1
    fi
    
    local count
    count=$(find "$dir_path" -name "$pattern" -type f | wc -l | tr -d ' ')
    
    if [[ $count -eq $expected_count ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Expected $expected_count files, found $count in: $dir_path"
        return 1
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Docker Assertions
# ─────────────────────────────────────────────────────────────────────────────

# Assert Docker image exists
assert_image_exists() {
    local test_name="$1"
    local image_name="$2"
    
    if docker image inspect "$image_name" >/dev/null 2>&1; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Docker image not found: $image_name"
        return 1
    fi
}

# Assert container is running
assert_container_running() {
    local test_name="$1"
    local container_name="$2"
    
    local status
    status=$(docker inspect -f '{{.State.Status}}' "$container_name" 2>/dev/null || echo "not_found")
    
    if [[ "$status" == "running" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Container '$container_name' status: $status (expected: running)"
        return 1
    fi
}

# Assert container is healthy
assert_container_healthy() {
    local test_name="$1"
    local container_name="$2"
    
    local health
    health=$(docker inspect -f '{{.State.Health.Status}}' "$container_name" 2>/dev/null || echo "not_found")
    
    if [[ "$health" == "healthy" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Container '$container_name' health: $health (expected: healthy)"
        return 1
    fi
}

# Assert docker compose config is valid
assert_compose_valid() {
    local test_name="$1"
    shift
    local -a compose_files=("$@")
    
    local compose_args=""
    for f in "${compose_files[@]}"; do
        compose_args="$compose_args -f $f"
    done
    
    # shellcheck disable=SC2086
    if docker compose $compose_args config >/dev/null 2>&1; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Invalid docker-compose config"
        return 1
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# HTTP/Network Assertions
# ─────────────────────────────────────────────────────────────────────────────

# Assert HTTP endpoint returns expected status
assert_http_status() {
    local test_name="$1"
    local url="$2"
    local expected_status="${3:-200}"
    
    local actual_status
    actual_status=$(curl -sf -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
    
    if [[ "$actual_status" == "$expected_status" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "URL: $url - Expected status: $expected_status, Actual: $actual_status"
        return 1
    fi
}

# Assert HTTP response contains text
assert_http_response_contains() {
    local test_name="$1"
    local url="$2"
    local expected_text="$3"
    
    local response
    response=$(curl -sf "$url" 2>/dev/null || echo "")
    
    if [[ "$response" == *"$expected_text"* ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Response from $url does not contain: $expected_text"
        return 1
    fi
}

# Assert port is listening
assert_port_listening() {
    local test_name="$1"
    local host="$2"
    local port="$3"
    
    if nc -z "$host" "$port" 2>/dev/null || timeout 1 bash -c "echo > /dev/tcp/$host/$port" 2>/dev/null; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Port not listening: $host:$port"
        return 1
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# JSON Assertions
# ─────────────────────────────────────────────────────────────────────────────

# Assert JSON file has key
assert_json_has_key() {
    local test_name="$1"
    local json_file="$2"
    local key="$3"
    
    if [[ ! -f "$json_file" ]]; then
        _record_fail "$test_name" "JSON file not found: $json_file"
        return 1
    fi
    
    local value
    value=$(jq -r "$key // empty" "$json_file" 2>/dev/null)
    
    if [[ -n "$value" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Key '$key' not found in: $json_file"
        return 1
    fi
}

# Assert JSON value equals
assert_json_value() {
    local test_name="$1"
    local json_file="$2"
    local key="$3"
    local expected="$4"
    
    if [[ ! -f "$json_file" ]]; then
        _record_fail "$test_name" "JSON file not found: $json_file"
        return 1
    fi
    
    local actual
    actual=$(jq -r "$key // empty" "$json_file" 2>/dev/null)
    
    if [[ "$actual" == "$expected" ]]; then
        _record_pass "$test_name"
        return 0
    else
        _record_fail "$test_name" "Key '$key' - Expected: '$expected', Actual: '$actual'"
        return 1
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Test Flow Control
# ─────────────────────────────────────────────────────────────────────────────

# Skip test with reason
skip_test() {
    local test_name="$1"
    local reason="${2:-}"
    _record_skip "$test_name" "$reason"
}

# Run a test group
run_test_group() {
    local group_name="$1"
    echo ""
    echo -e "  ${BOLD}$group_name${NC}"
}

# Conditional test execution based on test level
if_level() {
    local required_level="$1"
    
    case "$required_level" in
        build)
            return 0  # Always run build tests
            ;;
        smoke)
            [[ "$TEST_LEVEL" == "smoke" || "$TEST_LEVEL" == "full" ]]
            ;;
        full)
            [[ "$TEST_LEVEL" == "full" ]]
            ;;
        *)
            return 1
            ;;
    esac
}

export -f assert assert_equals assert_not_empty assert_success assert_failure
export -f assert_file_exists assert_dir_exists assert_path_exists
export -f assert_file_contains assert_file_not_contains assert_executable
export -f assert_dir_not_empty assert_file_count
export -f assert_image_exists assert_container_running assert_container_healthy assert_compose_valid
export -f assert_http_status assert_http_response_contains assert_port_listening
export -f assert_json_has_key assert_json_value
export -f skip_test run_test_group if_level
