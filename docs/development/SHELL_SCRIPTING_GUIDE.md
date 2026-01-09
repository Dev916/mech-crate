# Shell Scripting Guide: DevOps Excellence

**Version**: 1.0
**Last Updated**: 2026-01-08
**Purpose**: Comprehensive guide to production-grade shell scripting for DevOps, emphasizing security, modularity, cross-platform compatibility, and maintainability

---

## Table of Contents

1. [Core Principles](#1-core-principles)
2. [Cross-Platform Compatibility](#2-cross-platform-compatibility)
3. [Security Best Practices](#3-security-best-practices)
4. [Script Architecture & Modularity](#4-script-architecture--modularity)
5. [Error Handling & Validation](#5-error-handling--validation)
6. [Testing Strategies](#6-testing-strategies)
7. [Documentation Standards](#7-documentation-standards)
8. [DevOps Integration Patterns](#8-devops-integration-patterns)
9. [Quality Gates & Validation](#9-quality-gates--validation)
10. [Common Patterns & Anti-Patterns](#10-common-patterns--anti-patterns)

---

## 1. Core Principles

### 1.1 Framework Alignment

Shell scripts in our ecosystem follow the same rigorous standards as application code:

**Primary Directive**: "Security > reliability > maintainability > convenience"

**Key Principles**:
- **Evidence > Assumptions**: Validate all inputs, environment state, and dependencies
- **Fail Fast, Fail Explicitly**: Detect errors immediately with actionable context
- **Modularity**: Single Responsibility Principle applies to scripts and functions
- **Composability**: Scripts should be pipeline-friendly and reusable
- **Observability**: Comprehensive logging with structured output
- **Idempotency**: Scripts should be safely re-runnable

### 1.2 Script Classification

```yaml
operational_scripts:
  purpose: "Production operations, deployments, automation"
  requirements: "Maximum reliability, comprehensive error handling"
  examples: ["deploy.sh", "backup.sh", "health-check.sh"]

development_scripts:
  purpose: "Local development, testing, tooling"
  requirements: "Developer experience, clear feedback"
  examples: ["dev-setup.sh", "test-runner.sh", "db-seed.sh"]

infrastructure_scripts:
  purpose: "System provisioning, configuration management"
  requirements: "Idempotency, auditability, cross-platform"
  examples: ["setup-node.sh", "configure-firewall.sh"]

utility_scripts:
  purpose: "Helper functions, shared libraries"
  requirements: "Pure functions, composability, zero side effects"
  examples: ["lib/logger.sh", "lib/validators.sh"]
```

### 1.3 Quality Standards

All production scripts must meet:

- **Shellcheck**: Zero warnings at default strictness
- **POSIX Compliance**: Or explicit bash requirement documentation
- **Error Handling**: Comprehensive with exit codes
- **Documentation**: Inline comments + usage help
- **Testing**: Critical paths covered with bats or shunit2
- **Security Review**: Pass security checklist (see §3)

---

## 2. Cross-Platform Compatibility

### 2.1 Platform Detection

**Reliable platform detection pattern**:

```bash
#!/usr/bin/env bash
# detect-platform.sh - Portable platform detection

# ✅ Robust platform detection
detect_platform() {
    local platform
    case "$(uname -s)" in
        Linux*)     platform="linux";;
        Darwin*)    platform="macos";;
        CYGWIN*|MINGW*|MSYS*) platform="windows";;
        FreeBSD*)   platform="freebsd";;
        OpenBSD*)   platform="openbsd";;
        *)          platform="unknown";;
    esac
    echo "${platform}"
}

# ✅ Architecture detection
detect_arch() {
    local arch
    case "$(uname -m)" in
        x86_64|amd64)   arch="amd64";;
        aarch64|arm64)  arch="arm64";;
        armv7l)         arch="armv7";;
        i386|i686)      arch="386";;
        *)              arch="unknown";;
    esac
    echo "${arch}"
}

# Usage
PLATFORM=$(detect_platform)
ARCH=$(detect_arch)

echo "Running on: ${PLATFORM}/${ARCH}"
```

### 2.2 POSIX vs Bash

**Decision Matrix**:

| Use POSIX (`#!/bin/sh`) | Use Bash (`#!/usr/bin/env bash`) |
|-------------------------|-----------------------------------|
| Maximum portability needed | Advanced features required |
| Alpine/BusyBox environments | Arrays, associative arrays needed |
| Minimal dependencies | Extended pattern matching |
| CI/CD containers | Local development scripts |
| System init scripts | Complex data processing |

**POSIX-Compatible Example**:
```bash
#!/bin/sh
# posix-example.sh - Maximum portability

set -eu  # POSIX supports these

# ✅ POSIX-compatible string operations
string_contains() {
    case "$1" in
        *"$2"*) return 0;;
        *) return 1;;
    esac
}

# ✅ POSIX-compatible loops
for item in "$@"; do
    if string_contains "$item" "test"; then
        echo "Found test item: $item"
    fi
done
```

**Bash-Specific Example**:
```bash
#!/usr/bin/env bash
# bash-example.sh - Requires bash 4.0+

set -euo pipefail

# ✅ Bash arrays
declare -a services=("api" "worker" "scheduler")

# ✅ Bash associative arrays
declare -A config=(
    [env]="production"
    [region]="us-east-1"
    [replicas]="3"
)

# ✅ Bash parameter expansion
for service in "${services[@]}"; do
    echo "Deploying ${service} to ${config[region]}"
done

# ✅ Bash extended globbing
shopt -s extglob nullglob
for file in !(*.md|*.txt); do
    echo "Processing: $file"
done
```

### 2.3 Command Portability

**Platform-Specific Command Handling**:

```bash
#!/usr/bin/env bash
# portable-commands.sh

# ✅ Handle GNU vs BSD command differences
portable_sed() {
    if sed --version >/dev/null 2>&1; then
        # GNU sed
        sed -i "$@"
    else
        # BSD sed (macOS)
        sed -i '' "$@"
    fi
}

# ✅ Portable readlink (for getting absolute paths)
portable_realpath() {
    local path="$1"

    if command -v realpath >/dev/null 2>&1; then
        # Linux: use realpath
        realpath "$path"
    elif command -v greadlink >/dev/null 2>&1; then
        # macOS with coreutils: use greadlink
        greadlink -f "$path"
    else
        # Fallback: Python
        python3 -c "import os; print(os.path.realpath('$path'))"
    fi
}

# ✅ Portable date handling
portable_date() {
    if date --version >/dev/null 2>&1; then
        # GNU date
        date -d "$1" "$2"
    else
        # BSD date (macOS)
        date -j -f "%Y-%m-%d" "$1" "$2"
    fi
}

# ✅ Cross-platform clipboard
copy_to_clipboard() {
    local content="$1"

    if command -v pbcopy >/dev/null 2>&1; then
        # macOS
        echo "$content" | pbcopy
    elif command -v xclip >/dev/null 2>&1; then
        # Linux with X11
        echo "$content" | xclip -selection clipboard
    elif command -v wl-copy >/dev/null 2>&1; then
        # Linux with Wayland
        echo "$content" | wl-copy
    else
        echo "Error: No clipboard utility found" >&2
        return 1
    fi
}
```

### 2.4 Dependency Management

**Graceful dependency handling**:

```bash
#!/usr/bin/env bash
# check-dependencies.sh

# ✅ Required dependencies (fail if missing)
declare -a REQUIRED_DEPS=(
    "git"
    "docker"
    "jq"
)

# ✅ Optional dependencies (warn if missing)
declare -A OPTIONAL_DEPS=(
    [gh]="GitHub CLI - required for PR automation"
    [terraform]="Infrastructure provisioning"
    [kubectl]="Kubernetes deployments"
)

check_required_dependencies() {
    local missing_deps=()

    for cmd in "${REQUIRED_DEPS[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing_deps+=("$cmd")
        fi
    done

    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo "Error: Missing required dependencies:" >&2
        printf '  - %s\n' "${missing_deps[@]}" >&2
        return 1
    fi
}

check_optional_dependencies() {
    for cmd in "${!OPTIONAL_DEPS[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            echo "Warning: Optional dependency '$cmd' not found" >&2
            echo "  Purpose: ${OPTIONAL_DEPS[$cmd]}" >&2
        fi
    done
}

# Run checks
check_required_dependencies || exit 1
check_optional_dependencies
```

---

## 3. Security Best Practices

### 3.1 Security Principles

**Zero Trust Model for Scripts**:
- **Never trust input**: Validate everything
- **Never trust environment**: Verify state before operations
- **Principle of Least Privilege**: Request minimum necessary permissions
- **Defense in Depth**: Multiple validation layers
- **Fail Securely**: Default to safe state on errors

### 3.2 Input Validation & Sanitization

```bash
#!/usr/bin/env bash
# input-validation.sh

# ❌ DANGEROUS: Command injection vulnerability
dangerous_example() {
    local user_input="$1"
    eval "echo $user_input"  # NEVER DO THIS
    sh -c "ls $user_input"    # NEVER DO THIS
}

# ✅ SAFE: Proper input validation
validate_alphanumeric() {
    local input="$1"
    if [[ ! "$input" =~ ^[a-zA-Z0-9_-]+$ ]]; then
        echo "Error: Invalid input. Only alphanumeric, dash, underscore allowed." >&2
        return 1
    fi
}

# ✅ SAFE: Whitelist validation
validate_environment() {
    local env="$1"
    case "$env" in
        dev|staging|production) return 0;;
        *)
            echo "Error: Invalid environment. Must be: dev, staging, production" >&2
            return 1
            ;;
    esac
}

# ✅ SAFE: Path sanitization
sanitize_path() {
    local path="$1"

    # Remove leading/trailing whitespace
    path=$(echo "$path" | xargs)

    # Prevent directory traversal
    if [[ "$path" == *".."* ]]; then
        echo "Error: Path traversal detected" >&2
        return 1
    fi

    # Ensure path is within allowed directory
    local base_dir="/var/app"
    local full_path="${base_dir}/${path}"
    local canonical_path
    canonical_path=$(portable_realpath "$full_path")

    if [[ ! "$canonical_path" == "$base_dir"* ]]; then
        echo "Error: Path outside allowed directory" >&2
        return 1
    fi

    echo "$canonical_path"
}

# ✅ SAFE: URL validation
validate_url() {
    local url="$1"

    # Basic URL pattern validation
    if [[ ! "$url" =~ ^https?://[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(/.*)?$ ]]; then
        echo "Error: Invalid URL format" >&2
        return 1
    fi

    # Additional security: block private IP ranges
    if [[ "$url" =~ (localhost|127\.0\.0\.1|10\.|172\.(1[6-9]|2[0-9]|3[01])\.|192\.168\.) ]]; then
        echo "Error: Private IP ranges not allowed" >&2
        return 1
    fi
}
```

### 3.3 Secrets Management

```bash
#!/usr/bin/env bash
# secrets-management.sh

# ❌ NEVER: Hardcode secrets
DANGEROUS_API_KEY="sk-1234567890abcdef"  # NEVER DO THIS

# ❌ NEVER: Log secrets
echo "API Key: $API_KEY"  # NEVER DO THIS

# ✅ SAFE: Load from secure environment
load_secrets() {
    if [ -z "$API_KEY" ]; then
        echo "Error: API_KEY environment variable not set" >&2
        echo "Load from: vault, AWS Secrets Manager, or secure env file" >&2
        return 1
    fi
}

# ✅ SAFE: Use environment files securely
load_env_file() {
    local env_file="${1:-.env}"

    if [ ! -f "$env_file" ]; then
        echo "Error: Environment file not found: $env_file" >&2
        return 1
    fi

    # Check file permissions (should be 600 or 400)
    local perms
    perms=$(stat -c %a "$env_file" 2>/dev/null || stat -f %A "$env_file")
    if [[ ! "$perms" =~ ^[46]00$ ]]; then
        echo "Warning: Insecure permissions on $env_file (found: $perms, expected: 600)" >&2
    fi

    # Load without exposing in process list
    set -a
    # shellcheck source=/dev/null
    source "$env_file"
    set +a
}

# ✅ SAFE: Mask secrets in logs
mask_secret() {
    local secret="$1"
    local length=${#secret}

    if [ "$length" -le 4 ]; then
        echo "****"
    else
        local prefix="${secret:0:2}"
        local suffix="${secret: -2}"
        echo "${prefix}****${suffix}"
    fi
}

# ✅ SAFE: Clean up secrets on exit
cleanup_secrets() {
    unset API_KEY
    unset DATABASE_PASSWORD
    unset JWT_SECRET
}
trap cleanup_secrets EXIT
```

### 3.4 File Operations Security

```bash
#!/usr/bin/env bash
# secure-file-ops.sh

# ✅ Create temporary files securely
create_temp_file() {
    local temp_file
    temp_file=$(mktemp) || {
        echo "Error: Failed to create temporary file" >&2
        return 1
    }

    # Set restrictive permissions immediately
    chmod 600 "$temp_file"

    echo "$temp_file"
}

# ✅ Secure temporary directory
create_temp_dir() {
    local temp_dir
    temp_dir=$(mktemp -d) || {
        echo "Error: Failed to create temporary directory" >&2
        return 1
    }

    chmod 700 "$temp_dir"
    echo "$temp_dir"
}

# ✅ Atomic file operations
atomic_write() {
    local target_file="$1"
    local content="$2"
    local temp_file

    temp_file=$(create_temp_file) || return 1

    # Write to temp file
    echo "$content" > "$temp_file" || {
        rm -f "$temp_file"
        return 1
    }

    # Atomic move
    mv -f "$temp_file" "$target_file" || {
        rm -f "$temp_file"
        return 1
    }
}

# ✅ Safe file deletion
secure_delete() {
    local file="$1"

    if [ ! -f "$file" ]; then
        echo "Warning: File not found: $file" >&2
        return 1
    fi

    # Overwrite before deletion (for sensitive data)
    if [ -w "$file" ]; then
        dd if=/dev/urandom of="$file" bs=1k count=$(du -k "$file" | cut -f1) 2>/dev/null
    fi

    rm -f "$file"
}
```

### 3.5 Process Security

```bash
#!/usr/bin/env bash
# process-security.sh

# ✅ Safe command execution (avoid shell injection)
safe_execute() {
    local -a cmd=("$@")

    # Use array to prevent word splitting and globbing
    "${cmd[@]}" || {
        echo "Error: Command failed with exit code $?" >&2
        return 1
    }
}

# ✅ Execute with timeout
execute_with_timeout() {
    local timeout="$1"
    shift
    local -a cmd=("$@")

    if command -v timeout >/dev/null 2>&1; then
        # GNU timeout
        timeout "$timeout" "${cmd[@]}"
    else
        # Fallback for systems without timeout
        "${cmd[@]}" &
        local pid=$!

        ( sleep "$timeout"; kill -TERM "$pid" 2>/dev/null ) &
        local killer=$!

        wait "$pid"
        local exit_code=$?
        kill "$killer" 2>/dev/null

        return $exit_code
    fi
}

# ✅ Safely handle background processes
run_with_cleanup() {
    local -a cmd=("$@")
    local pid

    # Start background process
    "${cmd[@]}" &
    pid=$!

    # Ensure cleanup on exit
    trap "kill $pid 2>/dev/null; wait $pid 2>/dev/null" EXIT INT TERM

    # Wait for completion
    wait "$pid"
}

# ✅ Drop privileges when possible
drop_privileges() {
    local target_user="$1"

    if [ "$(id -u)" -eq 0 ]; then
        if command -v sudo >/dev/null 2>&1; then
            exec sudo -u "$target_user" "$0" "$@"
        elif command -v su >/dev/null 2>&1; then
            exec su -s /bin/bash "$target_user" -c "$0 $*"
        else
            echo "Error: Cannot drop privileges - no sudo or su available" >&2
            return 1
        fi
    fi
}
```

---

## 4. Script Architecture & Modularity

### 4.1 Single Responsibility Principle

**Script Organization**:

```bash
# ❌ BAD: Monolithic script doing everything
#!/usr/bin/env bash
# deploy-everything.sh - 2000 lines of mixed concerns

# ✅ GOOD: Modular architecture
project-root/
├── scripts/
│   ├── deploy.sh              # Main orchestration script
│   ├── lib/                   # Shared libraries
│   │   ├── logger.sh          # Logging utilities
│   │   ├── validators.sh      # Input validation
│   │   ├── aws.sh             # AWS operations
│   │   └── notifications.sh   # Slack/email alerts
│   ├── deploy/                # Deployment modules
│   │   ├── build.sh           # Build application
│   │   ├── test.sh            # Run tests
│   │   ├── backup.sh          # Backup before deploy
│   │   └── rollback.sh        # Rollback procedures
│   └── utils/                 # Utility scripts
│       ├── health-check.sh    # Service health checks
│       └── cleanup.sh         # Cleanup resources
```

### 4.2 Module Pattern

**Creating reusable library modules**:

```bash
# lib/logger.sh - Structured logging module

#!/usr/bin/env bash
# Prevent double-sourcing
[ -n "${LOGGER_SH_LOADED:-}" ] && return 0
readonly LOGGER_SH_LOADED=1

# Configuration
LOG_LEVEL="${LOG_LEVEL:-INFO}"
LOG_FILE="${LOG_FILE:-}"
LOG_JSON="${LOG_JSON:-false}"

# Log levels
readonly LOG_LEVEL_DEBUG=0
readonly LOG_LEVEL_INFO=1
readonly LOG_LEVEL_WARN=2
readonly LOG_LEVEL_ERROR=3
readonly LOG_LEVEL_FATAL=4

# Convert log level name to number
_log_level_to_number() {
    case "${1^^}" in
        DEBUG) echo $LOG_LEVEL_DEBUG;;
        INFO)  echo $LOG_LEVEL_INFO;;
        WARN)  echo $LOG_LEVEL_WARN;;
        ERROR) echo $LOG_LEVEL_ERROR;;
        FATAL) echo $LOG_LEVEL_FATAL;;
        *)     echo $LOG_LEVEL_INFO;;
    esac
}

# Core logging function
_log() {
    local level="$1"
    local message="$2"
    shift 2
    local context=("$@")

    local level_num
    local config_level_num
    level_num=$(_log_level_to_number "$level")
    config_level_num=$(_log_level_to_number "$LOG_LEVEL")

    # Skip if below configured level
    [ "$level_num" -lt "$config_level_num" ] && return 0

    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    if [ "$LOG_JSON" = "true" ]; then
        # JSON structured logging
        printf '{"timestamp":"%s","level":"%s","message":"%s"' \
            "$timestamp" "$level" "$message"

        # Add context fields
        for ctx in "${context[@]}"; do
            local key="${ctx%%=*}"
            local value="${ctx#*=}"
            printf ',"%s":"%s"' "$key" "$value"
        done

        printf '}\n'
    else
        # Human-readable logging
        local color_reset="\033[0m"
        local color
        case "$level" in
            DEBUG) color="\033[0;36m";;  # Cyan
            INFO)  color="\033[0;32m";;  # Green
            WARN)  color="\033[0;33m";;  # Yellow
            ERROR) color="\033[0;31m";;  # Red
            FATAL) color="\033[1;31m";;  # Bold Red
        esac

        printf "%b[%s] %-5s %s%b" \
            "$color" "$timestamp" "$level" "$message" "$color_reset"

        # Add context
        if [ ${#context[@]} -gt 0 ]; then
            printf " |"
            for ctx in "${context[@]}"; do
                printf " %s" "$ctx"
            done
        fi

        printf "\n"
    fi | if [ -n "$LOG_FILE" ]; then
        tee -a "$LOG_FILE"
    else
        cat
    fi
}

# Public API
log_debug() { _log "DEBUG" "$@"; }
log_info()  { _log "INFO" "$@"; }
log_warn()  { _log "WARN" "$@"; }
log_error() { _log "ERROR" "$@"; }
log_fatal() { _log "FATAL" "$@"; exit 1; }

# Export functions
export -f log_debug log_info log_warn log_error log_fatal
```

**Using the logger module**:

```bash
#!/usr/bin/env bash
# deploy.sh - Main deployment script

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source libraries
# shellcheck source=lib/logger.sh
source "${SCRIPT_DIR}/lib/logger.sh"

# Configure logging
export LOG_LEVEL="INFO"
export LOG_FILE="/var/log/deploy.log"
export LOG_JSON="false"

main() {
    log_info "Starting deployment" "env=production" "version=1.2.3"

    # Deployment logic
    if ! perform_deployment; then
        log_error "Deployment failed" "stage=deploy"
        return 1
    fi

    log_info "Deployment completed successfully"
}

main "$@"
```

### 4.3 Configuration Management

```bash
# lib/config.sh - Configuration module

#!/usr/bin/env bash
[ -n "${CONFIG_SH_LOADED:-}" ] && return 0
readonly CONFIG_SH_LOADED=1

# Configuration hierarchy (later sources override earlier)
declare -a CONFIG_SOURCES=(
    "/etc/app/config.env"           # System-wide defaults
    "${HOME}/.config/app/config.env" # User defaults
    "${PWD}/.env"                    # Project-specific
    "${ENV_FILE:-}"                  # Explicit override
)

# Load configuration with validation
load_config() {
    local -A loaded_configs=()

    for config_file in "${CONFIG_SOURCES[@]}"; do
        if [ -n "$config_file" ] && [ -f "$config_file" ]; then
            log_debug "Loading config from: $config_file"

            # Validate before loading
            validate_config_file "$config_file" || {
                log_error "Invalid config file: $config_file"
                return 1
            }

            # Load safely
            set -a
            # shellcheck source=/dev/null
            source "$config_file"
            set +a

            loaded_configs["$config_file"]=1
        fi
    done

    # Log which configs were loaded
    if [ ${#loaded_configs[@]} -gt 0 ]; then
        log_info "Loaded ${#loaded_configs[@]} config files"
    else
        log_warn "No config files loaded"
    fi
}

# Validate config file format
validate_config_file() {
    local file="$1"

    # Check syntax (no execution)
    bash -n "$file" 2>/dev/null || {
        log_error "Syntax error in config file: $file"
        return 1
    }

    # Check for dangerous patterns
    if grep -qE '(rm -rf|eval |exec |>\s*/dev/)' "$file"; then
        log_error "Dangerous commands detected in config: $file"
        return 1
    fi

    return 0
}

# Get required config value
config_require() {
    local var_name="$1"
    local var_value="${!var_name:-}"

    if [ -z "$var_value" ]; then
        log_error "Required configuration missing: $var_name"
        return 1
    fi

    echo "$var_value"
}

# Get config value with default
config_get() {
    local var_name="$1"
    local default="${2:-}"
    local var_value="${!var_name:-$default}"

    echo "$var_value"
}
```

### 4.4 Function Design

**Pure functions and side effect isolation**:

```bash
#!/usr/bin/env bash
# function-design.sh

# ✅ GOOD: Pure function - no side effects
calculate_sha256() {
    local file="$1"
    sha256sum "$file" | awk '{print $1}'
}

# ✅ GOOD: Clear input/output contract
validate_semver() {
    local version="$1"

    if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
        return 0
    else
        return 1
    fi
}

# ✅ GOOD: Side effects isolated and documented
# Side effects: Creates directory, writes file, modifies filesystem
initialize_project() {
    local project_dir="$1"

    mkdir -p "$project_dir" || return 1
    touch "$project_dir/README.md" || return 1

    log_info "Initialized project" "dir=$project_dir"
}

# ✅ GOOD: Function composition
process_deployment() {
    local version="$1"

    validate_semver "$version" || return 1

    local artifact_path
    artifact_path=$(build_artifact "$version") || return 1

    local checksum
    checksum=$(calculate_sha256 "$artifact_path") || return 1

    upload_artifact "$artifact_path" "$checksum" || return 1
}

# ✅ GOOD: Named parameters pattern
deploy_service() {
    local service=""
    local environment=""
    local version=""
    local dry_run="false"

    # Parse named parameters
    while [ $# -gt 0 ]; do
        case "$1" in
            --service)    service="$2"; shift 2;;
            --env)        environment="$2"; shift 2;;
            --version)    version="$2"; shift 2;;
            --dry-run)    dry_run="true"; shift;;
            *)
                echo "Unknown parameter: $1" >&2
                return 1
                ;;
        esac
    done

    # Validate required parameters
    [ -z "$service" ] && { echo "Error: --service required" >&2; return 1; }
    [ -z "$environment" ] && { echo "Error: --env required" >&2; return 1; }
    [ -z "$version" ] && { echo "Error: --version required" >&2; return 1; }

    # Execute with parameters
    log_info "Deploying service" \
        "service=$service" \
        "env=$environment" \
        "version=$version" \
        "dry_run=$dry_run"
}
```

### 4.5 Script Template

**Production-ready script template**:

```bash
#!/usr/bin/env bash
#
# script-name.sh - Brief description of what this script does
#
# Usage: script-name.sh [options] <arguments>
#
# Options:
#   -h, --help         Show this help message
#   -v, --verbose      Enable verbose output
#   -d, --dry-run      Run without making changes
#   -e, --env ENV      Environment (dev|staging|production)
#
# Examples:
#   script-name.sh --env production
#   script-name.sh --dry-run --verbose
#
# Exit codes:
#   0 - Success
#   1 - General error
#   2 - Invalid arguments
#   3 - Dependency missing
#

set -euo pipefail
IFS=$'\n\t'

# Script metadata
readonly SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}")"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_VERSION="1.0.0"

# Configuration defaults
VERBOSE="${VERBOSE:-false}"
DRY_RUN="${DRY_RUN:-false}"
ENVIRONMENT="${ENVIRONMENT:-dev}"

# Load libraries
# shellcheck source=lib/logger.sh
source "${SCRIPT_DIR}/lib/logger.sh"

#######################################
# Display usage information
# Globals: None
# Arguments: None
# Outputs: Usage text to stdout
#######################################
usage() {
    cat << EOF
Usage: ${SCRIPT_NAME} [options] <arguments>

Options:
    -h, --help         Show this help message
    -v, --verbose      Enable verbose output
    -d, --dry-run      Run without making changes
    -e, --env ENV      Environment (dev|staging|production)

Examples:
    ${SCRIPT_NAME} --env production
    ${SCRIPT_NAME} --dry-run --verbose

Exit codes:
    0 - Success
    1 - General error
    2 - Invalid arguments
    3 - Dependency missing
EOF
}

#######################################
# Check required dependencies
# Globals: None
# Arguments: None
# Returns: 0 if all dependencies present, 3 otherwise
#######################################
check_dependencies() {
    local -a required_commands=("git" "jq" "curl")
    local missing_commands=()

    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing_commands+=("$cmd")
        fi
    done

    if [ ${#missing_commands[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_commands[*]}"
        return 3
    fi
}

#######################################
# Parse command line arguments
# Globals: VERBOSE, DRY_RUN, ENVIRONMENT
# Arguments: All script arguments ($@)
# Returns: 0 on success, 2 on invalid arguments
#######################################
parse_arguments() {
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                usage
                exit 0
                ;;
            -v|--verbose)
                VERBOSE="true"
                export LOG_LEVEL="DEBUG"
                shift
                ;;
            -d|--dry-run)
                DRY_RUN="true"
                shift
                ;;
            -e|--env)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -*)
                log_error "Unknown option: $1"
                usage >&2
                return 2
                ;;
            *)
                break
                ;;
        esac
    done

    # Validate environment
    case "$ENVIRONMENT" in
        dev|staging|production) ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT"
            return 2
            ;;
    esac
}

#######################################
# Cleanup function (called on EXIT)
# Globals: None
# Arguments: None
#######################################
cleanup() {
    local exit_code=$?

    # Cleanup temporary files
    if [ -n "${TEMP_DIR:-}" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi

    # Log exit
    if [ $exit_code -eq 0 ]; then
        log_info "Script completed successfully"
    else
        log_error "Script failed with exit code: $exit_code"
    fi

    exit $exit_code
}

#######################################
# Main script logic
# Globals: All configuration variables
# Arguments: Remaining positional arguments
# Returns: 0 on success, 1 on error
#######################################
main() {
    log_info "Starting ${SCRIPT_NAME}" \
        "version=${SCRIPT_VERSION}" \
        "env=${ENVIRONMENT}"

    # Your script logic here
    if [ "$DRY_RUN" = "true" ]; then
        log_info "DRY RUN: Would execute deployment"
        return 0
    fi

    # Actual operations
    log_info "Executing deployment..."

    return 0
}

# Trap cleanup on exit
trap cleanup EXIT

# Execute
check_dependencies || exit $?
parse_arguments "$@" || exit $?
main "$@"
```

---

## 5. Error Handling & Validation

### 5.1 Robust Error Handling

```bash
#!/usr/bin/env bash
# error-handling.sh

# ✅ Set strict error handling
set -euo pipefail
IFS=$'\n\t'

# Explanation:
# -e: Exit on any command failure
# -u: Exit on undefined variable usage
# -o pipefail: Pipelines fail if any command fails
# IFS: Prevent word splitting issues

# ✅ Better error handling with custom trap
error_handler() {
    local line_no="$1"
    local bash_lineno="$2"
    local command="$3"
    local exit_code="$4"

    log_error "Command failed" \
        "line=$line_no" \
        "command=$command" \
        "exit_code=$exit_code"

    # Optional: Send alert
    send_error_notification "$command" "$exit_code"
}

# Set error trap
trap 'error_handler ${LINENO} ${BASH_LINENO} "$BASH_COMMAND" $?' ERR

# ✅ Defensive programming patterns
safe_function() {
    # Validate preconditions
    [ $# -eq 0 ] && { log_error "No arguments provided"; return 1; }

    local input="$1"
    [ -z "$input" ] && { log_error "Empty input"; return 1; }

    # Main logic with error checking
    if ! some_command "$input"; then
        log_error "Command failed for input: $input"
        return 1
    fi

    # Validate postconditions
    if [ ! -f "expected-output.txt" ]; then
        log_error "Expected output file not created"
        return 1
    fi

    return 0
}

# ✅ Retry logic with exponential backoff
retry_with_backoff() {
    local max_attempts="$1"
    local base_delay="$2"
    shift 2
    local -a command=("$@")

    local attempt=1
    local delay="$base_delay"

    while [ $attempt -le "$max_attempts" ]; do
        log_debug "Attempt $attempt of $max_attempts"

        if "${command[@]}"; then
            return 0
        fi

        if [ $attempt -lt "$max_attempts" ]; then
            log_warn "Command failed, retrying in ${delay}s..." \
                "attempt=$attempt"
            sleep "$delay"

            # Exponential backoff
            delay=$((delay * 2))
        fi

        attempt=$((attempt + 1))
    done

    log_error "Command failed after $max_attempts attempts"
    return 1
}

# ✅ Circuit breaker pattern
declare -A CIRCUIT_BREAKERS=()
readonly CIRCUIT_BREAKER_THRESHOLD=5
readonly CIRCUIT_BREAKER_TIMEOUT=60

circuit_breaker_execute() {
    local circuit_name="$1"
    shift
    local -a command=("$@")

    # Check if circuit is open
    local failure_count="${CIRCUIT_BREAKERS[$circuit_name]:-0}"
    local last_failure="${CIRCUIT_BREAKERS[${circuit_name}_last]:-0}"
    local current_time
    current_time=$(date +%s)

    if [ "$failure_count" -ge "$CIRCUIT_BREAKER_THRESHOLD" ]; then
        local time_since_last=$((current_time - last_failure))

        if [ "$time_since_last" -lt "$CIRCUIT_BREAKER_TIMEOUT" ]; then
            log_error "Circuit breaker open for: $circuit_name" \
                "failures=$failure_count" \
                "retry_in=$((CIRCUIT_BREAKER_TIMEOUT - time_since_last))s"
            return 1
        else
            # Reset circuit breaker
            log_info "Circuit breaker reset: $circuit_name"
            CIRCUIT_BREAKERS[$circuit_name]=0
        fi
    fi

    # Execute command
    if "${command[@]}"; then
        # Success - reset failure count
        CIRCUIT_BREAKERS[$circuit_name]=0
        return 0
    else
        # Failure - increment counter
        CIRCUIT_BREAKERS[$circuit_name]=$((failure_count + 1))
        CIRCUIT_BREAKERS[${circuit_name}_last]=$current_time

        log_warn "Circuit breaker failure recorded" \
            "circuit=$circuit_name" \
            "count=${CIRCUIT_BREAKERS[$circuit_name]}"

        return 1
    fi
}
```

### 5.2 Validation Patterns

```bash
#!/usr/bin/env bash
# validation-patterns.sh

# ✅ Pre-execution validation checklist
validate_environment() {
    log_info "Validating environment..."

    # Check required environment variables
    local -a required_vars=(
        "AWS_REGION"
        "ENVIRONMENT"
        "APP_VERSION"
    )

    for var in "${required_vars[@]}"; do
        if [ -z "${!var:-}" ]; then
            log_error "Required environment variable not set: $var"
            return 1
        fi
    done

    # Check file system
    if [ ! -w "/var/app" ]; then
        log_error "Application directory not writable: /var/app"
        return 1
    fi

    # Check disk space
    local available_space
    available_space=$(df -k /var/app | awk 'NR==2 {print $4}')
    local required_space=$((1024 * 1024))  # 1GB in KB

    if [ "$available_space" -lt "$required_space" ]; then
        log_error "Insufficient disk space" \
            "available=${available_space}KB" \
            "required=${required_space}KB"
        return 1
    fi

    # Check network connectivity
    if ! ping -c 1 -W 2 8.8.8.8 >/dev/null 2>&1; then
        log_error "No network connectivity"
        return 1
    fi

    log_info "Environment validation passed"
}

# ✅ State validation
validate_deployment_state() {
    local service="$1"
    local expected_version="$2"

    log_info "Validating deployment state" \
        "service=$service" \
        "version=$expected_version"

    # Check service is running
    if ! systemctl is-active --quiet "$service"; then
        log_error "Service not running: $service"
        return 1
    fi

    # Check version matches
    local actual_version
    actual_version=$(get_deployed_version "$service")

    if [ "$actual_version" != "$expected_version" ]; then
        log_error "Version mismatch" \
            "expected=$expected_version" \
            "actual=$actual_version"
        return 1
    fi

    # Check health endpoint
    local health_url="http://localhost:8080/health"
    local response
    response=$(curl -sf "$health_url" || echo "FAILED")

    if [ "$response" = "FAILED" ]; then
        log_error "Health check failed: $health_url"
        return 1
    fi

    log_info "Deployment state validation passed"
}

# ✅ Data integrity validation
validate_backup() {
    local backup_file="$1"
    local checksum_file="${backup_file}.sha256"

    # Verify backup exists
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        return 1
    fi

    # Verify checksum file exists
    if [ ! -f "$checksum_file" ]; then
        log_error "Checksum file not found: $checksum_file"
        return 1
    fi

    # Verify checksum matches
    local expected_checksum
    expected_checksum=$(cat "$checksum_file")

    local actual_checksum
    actual_checksum=$(sha256sum "$backup_file" | awk '{print $1}')

    if [ "$expected_checksum" != "$actual_checksum" ]; then
        log_error "Backup checksum mismatch" \
            "file=$backup_file" \
            "expected=$expected_checksum" \
            "actual=$actual_checksum"
        return 1
    fi

    # Verify backup is not empty
    local file_size
    file_size=$(stat -c%s "$backup_file" 2>/dev/null || stat -f%z "$backup_file")

    if [ "$file_size" -eq 0 ]; then
        log_error "Backup file is empty: $backup_file"
        return 1
    fi

    log_info "Backup validation passed" "file=$backup_file" "size=${file_size}B"
}
```

---

## 6. Testing Strategies

### 6.1 Testing Framework Setup

**Using BATS (Bash Automated Testing System)**:

```bash
# Install BATS
# macOS: brew install bats-core
# Linux: git clone https://github.com/bats-core/bats-core.git && cd bats-core && ./install.sh /usr/local

# tests/test-helper.bash - Shared test utilities

#!/usr/bin/env bash

# Setup function (runs before each test)
setup() {
    # Create temporary directory for test
    export TEST_TEMP_DIR="$(mktemp -d)"

    # Load the script under test
    load "../lib/validators.sh"
}

# Teardown function (runs after each test)
teardown() {
    # Cleanup
    if [ -n "${TEST_TEMP_DIR:-}" ] && [ -d "$TEST_TEMP_DIR" ]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
}

# Helper assertions
assert_file_exists() {
    [ -f "$1" ] || {
        echo "Expected file to exist: $1" >&2
        return 1
    }
}

assert_equals() {
    [ "$1" = "$2" ] || {
        echo "Expected: $2" >&2
        echo "Actual: $1" >&2
        return 1
    }
}
```

### 6.2 Unit Tests

```bash
# tests/validators.bats - Unit tests for validators

#!/usr/bin/env bats

load test-helper

@test "validate_email accepts valid email" {
    run validate_email "user@example.com"
    [ "$status" -eq 0 ]
}

@test "validate_email rejects invalid email" {
    run validate_email "invalid-email"
    [ "$status" -eq 1 ]
}

@test "validate_email rejects empty string" {
    run validate_email ""
    [ "$status" -eq 1 ]
}

@test "validate_semver accepts valid versions" {
    run validate_semver "1.2.3"
    [ "$status" -eq 0 ]

    run validate_semver "1.0.0-alpha.1"
    [ "$status" -eq 0 ]

    run validate_semver "2.0.0+build.123"
    [ "$status" -eq 0 ]
}

@test "validate_semver rejects invalid versions" {
    run validate_semver "1.2"
    [ "$status" -eq 1 ]

    run validate_semver "v1.2.3"
    [ "$status" -eq 1 ]

    run validate_semver "1.2.3.4"
    [ "$status" -eq 1 ]
}

@test "sanitize_path prevents directory traversal" {
    run sanitize_path "../../../etc/passwd"
    [ "$status" -eq 1 ]
    [[ "$output" == *"Path traversal detected"* ]]
}

@test "sanitize_path allows valid paths" {
    mkdir -p "$TEST_TEMP_DIR/valid"
    run sanitize_path "$TEST_TEMP_DIR/valid"
    [ "$status" -eq 0 ]
}
```

### 6.3 Integration Tests

```bash
# tests/deploy-integration.bats - Integration tests

#!/usr/bin/env bats

load test-helper

setup() {
    # Setup test environment
    export TEST_ENV="test"
    export APP_DIR="$TEST_TEMP_DIR/app"
    export BACKUP_DIR="$TEST_TEMP_DIR/backup"

    mkdir -p "$APP_DIR" "$BACKUP_DIR"

    # Create mock application
    echo "v1.0.0" > "$APP_DIR/version.txt"
}

@test "deploy performs backup before deployment" {
    # Mock deployment function
    deploy() {
        backup_current_version || return 1
        install_new_version || return 1
    }

    run deploy
    [ "$status" -eq 0 ]

    # Verify backup was created
    backup_count=$(find "$BACKUP_DIR" -type f | wc -l)
    [ "$backup_count" -gt 0 ]
}

@test "deploy rolls back on failure" {
    # Mock that fails
    install_new_version() {
        return 1
    }

    deploy_with_rollback() {
        backup_current_version || return 1

        if ! install_new_version; then
            rollback_to_backup
            return 1
        fi
    }

    run deploy_with_rollback
    [ "$status" -eq 1 ]

    # Verify original version restored
    version=$(cat "$APP_DIR/version.txt")
    [ "$version" = "v1.0.0" ]
}

@test "deploy validates post-deployment state" {
    deploy_and_validate() {
        install_new_version || return 1
        validate_deployment || return 1
    }

    run deploy_and_validate
    [ "$status" -eq 0 ]
}
```

### 6.4 Mock and Stub Patterns

```bash
# tests/mocks.bash - Mock utilities

#!/usr/bin/env bash

# Mock external commands
mock_command() {
    local cmd="$1"
    local return_code="${2:-0}"
    local output="${3:-}"

    # Create mock function
    eval "
    $cmd() {
        echo \"$output\"
        return $return_code
    }
    export -f $cmd
    "
}

# Stub with recorded calls
declare -A STUB_CALLS

stub_command() {
    local cmd="$1"
    shift
    local -a responses=("$@")

    STUB_CALLS[$cmd]=0

    eval "
    $cmd() {
        local call_count=\${STUB_CALLS[$cmd]}
        STUB_CALLS[$cmd]=\$((call_count + 1))

        if [ \$call_count -lt ${#responses[@]} ]; then
            echo \"\${responses[\$call_count]}\"
        fi
    }
    export -f $cmd
    "
}

# Verify stub was called
verify_stub_called() {
    local cmd="$1"
    local expected_calls="${2:-1}"
    local actual_calls="${STUB_CALLS[$cmd]:-0}"

    if [ "$actual_calls" -ne "$expected_calls" ]; then
        echo "Expected $cmd to be called $expected_calls times, was called $actual_calls times" >&2
        return 1
    fi
}

# Example usage in tests
@test "deploy calls backup before install" {
    # Stub the commands
    stub_command backup_current_version "backup-success"
    stub_command install_new_version "install-success"

    # Run deployment
    run deploy

    # Verify order of calls
    verify_stub_called backup_current_version 1
    verify_stub_called install_new_version 1
}
```

---

## 7. Documentation Standards

### 7.1 Inline Documentation

```bash
#!/usr/bin/env bash
#
# deploy-service.sh - Deploy application services to production
#
# This script orchestrates the deployment of application services with
# zero-downtime rolling updates, automatic health checks, and rollback
# capabilities.
#
# USAGE:
#   deploy-service.sh [OPTIONS] <service> <version>
#
# ARGUMENTS:
#   service    Service name (api|worker|frontend)
#   version    Semantic version to deploy (e.g., 1.2.3)
#
# OPTIONS:
#   -e, --env ENV          Target environment (staging|production)
#   -r, --replicas N       Number of replicas (default: 3)
#   -t, --timeout SECONDS  Deployment timeout (default: 300)
#   -d, --dry-run          Preview changes without applying
#   -h, --help             Show this help message
#
# ENVIRONMENT VARIABLES:
#   AWS_REGION             AWS region for deployment
#   SLACK_WEBHOOK_URL      Slack notifications (optional)
#   ROLLBACK_ON_FAILURE    Auto-rollback on failure (default: true)
#
# EXAMPLES:
#   # Deploy API service to staging
#   deploy-service.sh --env staging api 1.2.3
#
#   # Dry-run production deployment
#   deploy-service.sh --env production --dry-run frontend 2.0.0
#
#   # Deploy with custom replicas and timeout
#   deploy-service.sh --replicas 5 --timeout 600 worker 1.5.0
#
# EXIT CODES:
#   0   Success
#   1   Deployment failed
#   2   Invalid arguments
#   3   Pre-deployment validation failed
#   4   Health check failed
#   5   Rollback failed
#
# DEPENDENCIES:
#   - aws-cli (>= 2.0)
#   - kubectl (>= 1.20)
#   - jq (>= 1.6)
#
# AUTHOR:
#   DevOps Team <devops@example.com>
#
# SEE ALSO:
#   rollback-service.sh, validate-deployment.sh
#

#######################################
# Deploy a service with rolling update
#
# Performs zero-downtime deployment with health checks and automatic
# rollback on failure. Creates backup of previous deployment state.
#
# Globals:
#   AWS_REGION
#   ENVIRONMENT
#   ROLLBACK_ON_FAILURE
#
# Arguments:
#   $1 - service: Service name to deploy
#   $2 - version: Semantic version string
#   $3 - replicas: Number of instances (optional, default: 3)
#
# Outputs:
#   Writes deployment progress to stdout
#   Writes errors to stderr
#   Writes deployment logs to /var/log/deploy/service-TIMESTAMP.log
#
# Returns:
#   0 if deployment succeeds
#   1 if deployment fails
#   4 if health check fails
#
# Example:
#   deploy_service "api" "1.2.3" 5
#######################################
deploy_service() {
    local service="$1"
    local version="$2"
    local replicas="${3:-3}"

    # Implementation...
}
```

### 7.2 README Template

```markdown
# Script Name

> One-line description of what the script does

## Overview

Detailed description of the script's purpose, use cases, and benefits.

## Prerequisites

- Bash 4.0 or higher
- Required system packages: `git`, `jq`, `curl`
- AWS CLI configured with appropriate credentials
- Kubernetes cluster access

## Installation

```bash
# Clone repository
git clone https://github.com/org/scripts.git

# Install dependencies
./scripts/setup-dependencies.sh

# Verify installation
./scripts/deploy.sh --version
```

## Usage

### Basic Usage

```bash
./deploy.sh --env production api 1.2.3
```

### Advanced Options

```bash
# Dry-run mode
./deploy.sh --dry-run --env production api 1.2.3

# Custom configuration
./deploy.sh --env production --replicas 5 --timeout 600 api 1.2.3

# With environment file
ENV_FILE=.env.production ./deploy.sh api 1.2.3
```

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `AWS_REGION` | Yes | - | AWS region for deployment |
| `ENVIRONMENT` | Yes | - | Target environment |
| `SLACK_WEBHOOK_URL` | No | - | Slack notifications |

### Configuration Files

Scripts load configuration in this order (later overrides earlier):

1. `/etc/app/config.env` - System-wide defaults
2. `~/.config/app/config.env` - User defaults
3. `.env` - Project-specific
4. `ENV_FILE` - Explicit override

## Architecture

```
deploy.sh (main orchestration)
├── lib/
│   ├── logger.sh (logging utilities)
│   ├── aws.sh (AWS operations)
│   └── validators.sh (input validation)
└── deploy/
    ├── backup.sh (pre-deployment backup)
    ├── health-check.sh (service validation)
    └── rollback.sh (failure recovery)
```

## Testing

```bash
# Run unit tests
bats tests/*.bats

# Run integration tests
bats tests/integration/*.bats

# Run with coverage
bats --tap tests/*.bats | tee test-results.tap
```

## Troubleshooting

### Common Issues

**Problem**: Deployment times out
**Solution**: Increase timeout with `--timeout 600`

**Problem**: Health check fails
**Solution**: Verify service endpoints with `./health-check.sh`

### Debug Mode

```bash
# Enable verbose logging
LOG_LEVEL=DEBUG ./deploy.sh ...

# Enable bash tracing
bash -x ./deploy.sh ...
```

## Security

- Never commit `.env` files with secrets
- Use AWS Secrets Manager or similar for sensitive data
- Verify checksums before deployment
- Audit logs stored in `/var/log/deploy/`

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing`)
3. Add tests for changes
4. Ensure `shellcheck` passes
5. Submit pull request

## License

MIT License - see LICENSE file

## Support

- Issues: https://github.com/org/scripts/issues
- Slack: #devops-support
- Email: devops@example.com
```

---

## 8. DevOps Integration Patterns

### 8.1 CI/CD Integration

**GitLab CI Example**:

```yaml
# .gitlab-ci.yml

variables:
  SHELLCHECK_VERSION: "0.9.0"

stages:
  - validate
  - test
  - deploy

# Shell script validation
shellcheck:
  stage: validate
  image: koalaman/shellcheck:v${SHELLCHECK_VERSION}
  script:
    - shellcheck scripts/**/*.sh
    - shellcheck lib/**/*.sh
  only:
    changes:
      - scripts/**/*.sh
      - lib/**/*.sh

# Unit tests
bats-tests:
  stage: test
  image: ubuntu:22.04
  before_script:
    - apt-get update && apt-get install -y bats git
  script:
    - bats tests/*.bats
  coverage: '/^Coverage: \d+\.\d+%/'
  artifacts:
    reports:
      junit: test-results.xml

# Integration tests
integration-tests:
  stage: test
  image: ubuntu:22.04
  services:
    - docker:dind
  script:
    - ./scripts/integration-test.sh
  only:
    - main
    - merge_requests

# Deploy to staging (automatic)
deploy-staging:
  stage: deploy
  script:
    - ./scripts/deploy.sh --env staging
  environment:
    name: staging
    url: https://staging.example.com
  only:
    - main

# Deploy to production (manual)
deploy-production:
  stage: deploy
  script:
    - ./scripts/deploy.sh --env production
  environment:
    name: production
    url: https://example.com
  when: manual
  only:
    - main
```

**GitHub Actions Example**:

```yaml
# .github/workflows/shell-scripts.yml

name: Shell Scripts CI

on:
  push:
    branches: [main]
  pull_request:
    paths:
      - 'scripts/**/*.sh'
      - 'lib/**/*.sh'

jobs:
  shellcheck:
    name: ShellCheck Validation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run ShellCheck
        uses: ludeeus/action-shellcheck@master
        with:
          scandir: './scripts'
          severity: warning

      - name: Check POSIX compliance
        run: |
          find scripts -name "*.sh" -exec shellcheck -s sh {} +

  bats-tests:
    name: BATS Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup BATS
        uses: mig4/setup-bats@v1
        with:
          bats-version: 1.10.0

      - name: Run tests
        run: |
          bats tests/*.bats

      - name: Upload coverage
        uses: codecov/codecov-action@v3

  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    needs: [shellcheck, bats-tests]
    steps:
      - uses: actions/checkout@v3

      - name: Run integration tests
        run: |
          chmod +x scripts/*.sh
          ./scripts/integration-test.sh

  deploy:
    name: Deploy
    runs-on: ubuntu-latest
    needs: [integration-tests]
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v2
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1

      - name: Deploy to staging
        run: |
          ./scripts/deploy.sh --env staging
```

### 8.2 Container Integration

**Dockerfile with scripts**:

```dockerfile
# Dockerfile for script execution environment

FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    bash \
    curl \
    git \
    jq \
    awscli \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash scriptuser && \
    mkdir -p /app /var/log/scripts && \
    chown -R scriptuser:scriptuser /app /var/log/scripts

# Copy scripts
COPY --chown=scriptuser:scriptuser scripts/ /app/scripts/
COPY --chown=scriptuser:scriptuser lib/ /app/lib/

# Set permissions
RUN find /app/scripts -name "*.sh" -exec chmod +x {} \; && \
    chmod 700 /app/scripts /app/lib

# Switch to non-root user
USER scriptuser
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /app/scripts/health-check.sh || exit 1

# Default command
ENTRYPOINT ["/app/scripts/entrypoint.sh"]
CMD ["--help"]
```

**Docker Compose with scripts**:

```yaml
# docker-compose.yml

version: '3.8'

services:
  deploy-runner:
    build:
      context: .
      dockerfile: Dockerfile
    image: deploy-runner:latest
    environment:
      - AWS_REGION=${AWS_REGION}
      - ENVIRONMENT=${ENVIRONMENT:-dev}
      - LOG_LEVEL=${LOG_LEVEL:-INFO}
    volumes:
      # Mount scripts for development
      - ./scripts:/app/scripts:ro
      - ./lib:/app/lib:ro
      # Persistent logs
      - deploy-logs:/var/log/scripts
    secrets:
      - aws_credentials
    networks:
      - deploy-network

secrets:
  aws_credentials:
    file: ~/.aws/credentials

volumes:
  deploy-logs:

networks:
  deploy-network:
    driver: bridge
```

### 8.3 Kubernetes Jobs

```yaml
# k8s/deploy-job.yaml

apiVersion: batch/v1
kind: Job
metadata:
  name: deploy-job
  namespace: ci-cd
spec:
  ttlSecondsAfterFinished: 3600
  backoffLimit: 3

  template:
    metadata:
      labels:
        app: deploy-runner
    spec:
      restartPolicy: Never

      serviceAccountName: deploy-service-account

      initContainers:
      - name: validate-environment
        image: deploy-runner:latest
        command: ["/app/scripts/validate-env.sh"]
        env:
        - name: ENVIRONMENT
          valueFrom:
            configMapKeyRef:
              name: deploy-config
              key: environment

      containers:
      - name: deploy
        image: deploy-runner:latest
        command: ["/app/scripts/deploy.sh"]
        args:
          - "--env"
          - "$(ENVIRONMENT)"
          - "$(SERVICE_NAME)"
          - "$(VERSION)"
        env:
        - name: ENVIRONMENT
          valueFrom:
            configMapKeyRef:
              name: deploy-config
              key: environment
        - name: SERVICE_NAME
          value: "api"
        - name: VERSION
          value: "1.2.3"
        - name: AWS_REGION
          valueFrom:
            configMapKeyRef:
              name: deploy-config
              key: aws_region
        volumeMounts:
        - name: scripts
          mountPath: /app/scripts
          readOnly: true
        - name: aws-credentials
          mountPath: /root/.aws
          readOnly: true
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"

      volumes:
      - name: scripts
        configMap:
          name: deploy-scripts
          defaultMode: 0755
      - name: aws-credentials
        secret:
          secretName: aws-credentials
```

### 8.4 Infrastructure as Code Integration

**Terraform with provisioning scripts**:

```hcl
# terraform/main.tf

resource "aws_instance" "app_server" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = var.instance_type

  user_data = templatefile("${path.module}/scripts/cloud-init.sh", {
    environment = var.environment
    app_version = var.app_version
    region      = var.aws_region
  })

  # Copy provisioning scripts
  provisioner "file" {
    source      = "${path.module}/../scripts/"
    destination = "/tmp/scripts"

    connection {
      type        = "ssh"
      user        = "ubuntu"
      private_key = file(var.ssh_private_key)
      host        = self.public_ip
    }
  }

  # Execute setup script
  provisioner "remote-exec" {
    inline = [
      "chmod +x /tmp/scripts/*.sh",
      "sudo /tmp/scripts/setup-instance.sh --env ${var.environment}",
      "sudo /tmp/scripts/deploy-app.sh --version ${var.app_version}"
    ]

    connection {
      type        = "ssh"
      user        = "ubuntu"
      private_key = file(var.ssh_private_key)
      host        = self.public_ip
    }
  }

  tags = {
    Name        = "app-server-${var.environment}"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

# Null resource for script updates
resource "null_resource" "update_scripts" {
  triggers = {
    script_hash = filemd5("${path.module}/../scripts/deploy-app.sh")
  }

  provisioner "remote-exec" {
    inline = [
      "sudo /tmp/scripts/deploy-app.sh --version ${var.app_version}"
    ]

    connection {
      type        = "ssh"
      user        = "ubuntu"
      private_key = file(var.ssh_private_key)
      host        = aws_instance.app_server.public_ip
    }
  }

  depends_on = [aws_instance.app_server]
}
```

---

## 9. Quality Gates & Validation

### 9.1 Pre-commit Hooks

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit - Validate scripts before commit

set -euo pipefail

echo "Running pre-commit checks for shell scripts..."

# Find all shell scripts in staging
mapfile -t scripts < <(git diff --cached --name-only --diff-filter=ACM | grep '\.sh$' || true)

if [ ${#scripts[@]} -eq 0 ]; then
    echo "No shell scripts to validate"
    exit 0
fi

# Track if any checks fail
checks_failed=0

# 1. ShellCheck validation
echo "→ Running ShellCheck..."
for script in "${scripts[@]}"; do
    if ! shellcheck "$script"; then
        echo "✗ ShellCheck failed: $script"
        checks_failed=1
    fi
done

# 2. Bash syntax check
echo "→ Validating syntax..."
for script in "${scripts[@]}"; do
    if ! bash -n "$script"; then
        echo "✗ Syntax check failed: $script"
        checks_failed=1
    fi
done

# 3. Check for common security issues
echo "→ Checking for security issues..."
for script in "${scripts[@]}"; do
    # Check for eval usage
    if grep -qE '\beval\s' "$script"; then
        echo "✗ Security: 'eval' usage detected in $script"
        checks_failed=1
    fi

    # Check for hardcoded secrets patterns
    if grep -qE '(password|secret|key)\s*=\s*["\047][^"\047]+["\047]' "$script"; then
        echo "✗ Security: Possible hardcoded secret in $script"
        checks_failed=1
    fi
done

# 4. Check for proper shebang
echo "→ Validating shebangs..."
for script in "${scripts[@]}"; do
    first_line=$(head -n1 "$script")
    if [[ ! "$first_line" =~ ^#!/(usr/bin/env bash|bin/(ba)?sh) ]]; then
        echo "✗ Invalid or missing shebang in $script"
        checks_failed=1
    fi
done

# 5. Check for TODO/FIXME with required context
echo "→ Checking TODOs..."
for script in "${scripts[@]}"; do
    if grep -qE 'TODO(?!\([a-z]+\))' "$script"; then
        echo "✗ TODO without assignee in $script (use: TODO(username))"
        checks_failed=1
    fi
done

if [ $checks_failed -eq 1 ]; then
    echo ""
    echo "✗ Pre-commit checks failed. Please fix the issues above."
    exit 1
fi

echo "✓ All pre-commit checks passed"
exit 0
```

### 9.2 Continuous Validation

```bash
#!/usr/bin/env bash
# scripts/validate-all.sh - Comprehensive validation suite

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Load logger
source "${SCRIPT_DIR}/lib/logger.sh"

# Validation results
declare -A VALIDATION_RESULTS=()
TOTAL_CHECKS=0
PASSED_CHECKS=0

run_check() {
    local check_name="$1"
    shift
    local -a check_command=("$@")

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    log_info "Running: $check_name"

    if "${check_command[@]}"; then
        VALIDATION_RESULTS["$check_name"]="PASS"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        log_info "✓ $check_name passed"
        return 0
    else
        VALIDATION_RESULTS["$check_name"]="FAIL"
        log_error "✗ $check_name failed"
        return 1
    fi
}

# 1. ShellCheck validation
validate_shellcheck() {
    log_info "ShellCheck validation"

    local scripts
    mapfile -t scripts < <(find "$PROJECT_ROOT/scripts" -name "*.sh")

    for script in "${scripts[@]}"; do
        shellcheck "$script" || return 1
    done
}

# 2. Unit tests
validate_unit_tests() {
    log_info "Running unit tests"
    bats "$PROJECT_ROOT/tests"/*.bats
}

# 3. Integration tests
validate_integration_tests() {
    log_info "Running integration tests"
    bats "$PROJECT_ROOT/tests/integration"/*.bats
}

# 4. Security audit
validate_security() {
    log_info "Security audit"

    local issues=0

    # Check for eval usage
    if grep -rE '\beval\s' "$PROJECT_ROOT/scripts" --include="*.sh"; then
        log_error "Found 'eval' usage (security risk)"
        issues=$((issues + 1))
    fi

    # Check for hardcoded secrets
    if grep -rE '(password|secret|key)\s*=\s*["\047][^"\047]+["\047]' \
        "$PROJECT_ROOT/scripts" --include="*.sh"; then
        log_error "Found possible hardcoded secrets"
        issues=$((issues + 1))
    fi

    # Check file permissions
    while IFS= read -r -d '' file; do
        local perms
        perms=$(stat -c %a "$file" 2>/dev/null || stat -f %A "$file")
        if [[ "$perms" =~ [2367]$ ]]; then
            log_error "Insecure permissions on $file: $perms"
            issues=$((issues + 1))
        fi
    done < <(find "$PROJECT_ROOT/scripts" -name "*.sh" -print0)

    return "$issues"
}

# 5. Documentation validation
validate_documentation() {
    log_info "Documentation validation"

    local scripts
    mapfile -t scripts < <(find "$PROJECT_ROOT/scripts" -name "*.sh" ! -path "*/lib/*")

    for script in "${scripts[@]}"; do
        # Check for usage function
        if ! grep -q '^usage()' "$script"; then
            log_warn "Missing usage() function in $script"
        fi

        # Check for header comments
        if ! head -n 10 "$script" | grep -q '^#.*Usage:'; then
            log_warn "Missing usage documentation in $script"
        fi
    done
}

# 6. Code quality metrics
validate_code_quality() {
    log_info "Code quality metrics"

    local scripts
    mapfile -t scripts < <(find "$PROJECT_ROOT/scripts" -name "*.sh")

    for script in "${scripts[@]}"; do
        local lines
        lines=$(wc -l < "$script")

        # Warn on large scripts (>500 lines)
        if [ "$lines" -gt 500 ]; then
            log_warn "Large script detected: $script ($lines lines) - consider refactoring"
        fi

        # Check cyclomatic complexity (approximate)
        local complexity
        complexity=$(grep -cE '\b(if|while|for|case)\b' "$script" || true)
        if [ "$complexity" -gt 20 ]; then
            log_warn "High complexity: $script (complexity: $complexity)"
        fi
    done
}

# Main validation
main() {
    log_info "Starting comprehensive validation"

    run_check "ShellCheck" validate_shellcheck
    run_check "Unit Tests" validate_unit_tests
    run_check "Integration Tests" validate_integration_tests
    run_check "Security Audit" validate_security
    run_check "Documentation" validate_documentation
    run_check "Code Quality" validate_code_quality

    # Summary
    echo ""
    log_info "Validation Summary"
    log_info "=================="
    log_info "Total checks: $TOTAL_CHECKS"
    log_info "Passed: $PASSED_CHECKS"
    log_info "Failed: $((TOTAL_CHECKS - PASSED_CHECKS))"

    # Detailed results
    for check in "${!VALIDATION_RESULTS[@]}"; do
        local result="${VALIDATION_RESULTS[$check]}"
        if [ "$result" = "PASS" ]; then
            log_info "✓ $check"
        else
            log_error "✗ $check"
        fi
    done

    # Exit with failure if any checks failed
    if [ "$PASSED_CHECKS" -ne "$TOTAL_CHECKS" ]; then
        log_error "Validation failed"
        return 1
    fi

    log_info "All validation checks passed"
    return 0
}

main "$@"
```

---

## 10. Common Patterns & Anti-Patterns

### 10.1 Good Patterns

```bash
#!/usr/bin/env bash
# good-patterns.sh - Examples of best practices

# ✅ GOOD: Early return pattern
process_file() {
    local file="$1"

    # Validate early, return early
    [ ! -f "$file" ] && { log_error "File not found: $file"; return 1; }
    [ ! -r "$file" ] && { log_error "File not readable: $file"; return 1; }

    # Main logic only executes if validations pass
    cat "$file" | process_content
}

# ✅ GOOD: Use functions for reusability
is_valid_semver() {
    local version="$1"
    [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
}

deploy_version() {
    local version="$1"

    if ! is_valid_semver "$version"; then
        log_error "Invalid version format: $version"
        return 1
    fi

    # Deployment logic...
}

# ✅ GOOD: Proper array handling
process_items() {
    local -a items=("$@")

    for item in "${items[@]}"; do
        echo "Processing: $item"
    done
}

# ✅ GOOD: Proper quoting
safe_file_operations() {
    local source_file="$1"
    local dest_dir="$2"

    # Always quote variables with spaces
    cp "$source_file" "$dest_dir/"

    # Array expansion with quotes
    local -a files=("file 1.txt" "file 2.txt")
    for file in "${files[@]}"; do
        echo "$file"
    done
}

# ✅ GOOD: Command substitution with error handling
get_git_commit() {
    local commit
    commit=$(git rev-parse HEAD 2>/dev/null) || {
        log_error "Not a git repository"
        return 1
    }
    echo "$commit"
}

# ✅ GOOD: Here-doc for multi-line content
generate_config() {
    cat << EOF
{
    "environment": "$ENVIRONMENT",
    "version": "$VERSION",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
}

# ✅ GOOD: Proper signal handling
cleanup_on_exit() {
    local exit_code=$?

    if [ -n "${TEMP_DIR:-}" ]; then
        rm -rf "$TEMP_DIR"
    fi

    if [ -n "${PID_FILE:-}" ]; then
        rm -f "$PID_FILE"
    fi

    exit $exit_code
}
trap cleanup_on_exit EXIT INT TERM

# ✅ GOOD: Named pipes for complex pipelines
process_with_named_pipe() {
    local pipe
    pipe=$(mktemp -u)
    mkfifo "$pipe"

    # Producer
    producer > "$pipe" &
    local producer_pid=$!

    # Consumer
    consumer < "$pipe"

    # Cleanup
    wait "$producer_pid"
    rm -f "$pipe"
}

# ✅ GOOD: Parallel execution with proper waiting
parallel_process() {
    local -a pids=()

    for item in "${items[@]}"; do
        process_item "$item" &
        pids+=($!)
    done

    # Wait for all background jobs
    local failed=0
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            failed=$((failed + 1))
        fi
    done

    return "$failed"
}
```

### 10.2 Anti-Patterns to Avoid

```bash
#!/usr/bin/env bash
# anti-patterns.sh - Common mistakes to avoid

# ❌ BAD: Useless use of cat
bad_cat() {
    cat file.txt | grep "pattern"
}

# ✅ GOOD: Direct input redirection
good_grep() {
    grep "pattern" file.txt
}

# ❌ BAD: Parsing ls output
bad_loop() {
    for file in $(ls *.txt); do
        echo "$file"
    done
}

# ✅ GOOD: Use glob patterns
good_loop() {
    for file in *.txt; do
        [ -f "$file" ] || continue
        echo "$file"
    done
}

# ❌ BAD: Unquoted variables
bad_quoting() {
    local file=$1
    cat $file  # Breaks with spaces
}

# ✅ GOOD: Always quote variables
good_quoting() {
    local file="$1"
    cat "$file"
}

# ❌ BAD: Using eval
bad_eval() {
    local cmd="ls -la"
    eval $cmd  # Command injection risk
}

# ✅ GOOD: Direct execution
good_execution() {
    ls -la
}

# ❌ BAD: Ignoring errors
bad_error_handling() {
    important_command
    # Continues even if it fails
}

# ✅ GOOD: Check return codes
good_error_handling() {
    if ! important_command; then
        log_error "Command failed"
        return 1
    fi
}

# ❌ BAD: Using backticks
bad_substitution() {
    result=`command`
}

# ✅ GOOD: Use $()
good_substitution() {
    result=$(command)
}

# ❌ BAD: [[ vs [ confusion
bad_test() {
    if [ $var == "test" ]; then  # Wrong operator for [ ]
        echo "match"
    fi
}

# ✅ GOOD: Correct test syntax
good_test() {
    if [ "$var" = "test" ]; then  # POSIX-compatible
        echo "match"
    fi

    # Or use bash [[
    if [[ "$var" == "test" ]]; then  # Bash-specific
        echo "match"
    fi
}

# ❌ BAD: Creating security vulnerabilities
bad_security() {
    local user_input="$1"
    eval "echo $user_input"  # Command injection
    curl "http://api.example.com/$user_input"  # URL injection
}

# ✅ GOOD: Validate and sanitize input
good_security() {
    local user_input="$1"

    # Validate format
    if [[ ! "$user_input" =~ ^[a-zA-Z0-9_-]+$ ]]; then
        log_error "Invalid input format"
        return 1
    fi

    # Use safely
    echo "$user_input"
    curl "http://api.example.com/$(urlencode "$user_input")"
}

# ❌ BAD: Not checking command existence
bad_command_check() {
    jq '.key' file.json  # Fails if jq not installed
}

# ✅ GOOD: Check dependencies
good_command_check() {
    if ! command -v jq >/dev/null 2>&1; then
        log_error "jq is required but not installed"
        return 1
    fi

    jq '.key' file.json
}

# ❌ BAD: Global variables without readonly
bad_globals() {
    CONFIG_FILE="/etc/app.conf"  # Can be modified
}

# ✅ GOOD: Use readonly for constants
good_globals() {
    readonly CONFIG_FILE="/etc/app.conf"
}

# ❌ BAD: Complex pipelines without error handling
bad_pipeline() {
    cat file.txt | grep pattern | sed 's/old/new/' | sort
    # If any step fails, continues anyway
}

# ✅ GOOD: Check pipeline status
good_pipeline() {
    set -o pipefail
    if ! cat file.txt | grep pattern | sed 's/old/new/' | sort; then
        log_error "Pipeline failed"
        return 1
    fi
}
```

### 10.3 Performance Patterns

```bash
#!/usr/bin/env bash
# performance-patterns.sh

# ✅ GOOD: Use built-in parameter expansion
fast_string_ops() {
    local str="hello_world_test"

    # Instead of: echo "$str" | sed 's/_/-/g'
    echo "${str//_/-}"

    # Instead of: echo "$str" | cut -d_ -f1
    echo "${str%%_*}"

    # Instead of: echo "$str" | awk '{print length}'
    echo "${#str}"
}

# ✅ GOOD: Batch operations
batch_process() {
    local -a files=()

    # Collect files first
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(find . -name "*.txt" -print0)

    # Process in batch
    if [ ${#files[@]} -gt 0 ]; then
        tar czf backup.tar.gz "${files[@]}"
    fi
}

# ❌ BAD: Multiple calls in loop
slow_loop() {
    for file in *.txt; do
        grep -l "pattern" "$file"
    done
}

# ✅ GOOD: Single call with all files
fast_grep() {
    grep -l "pattern" *.txt
}

# ✅ GOOD: Use process substitution
fast_comparison() {
    # Instead of creating temp files
    diff <(command1) <(command2)
}

# ✅ GOOD: Parallel processing with GNU parallel
parallel_process() {
    # Install: apt-get install parallel
    parallel -j4 process_file ::: *.txt
}

# ✅ GOOD: Use appropriate tools
fast_json() {
    # Instead of: cat file.json | python -c "import json..."
    jq '.key' file.json  # Much faster
}
```

---

## Appendix A: Quick Reference

### Common Commands Cheat Sheet

```bash
# File operations
cp -a source dest           # Copy preserving attributes
mv -n source dest           # Move, no clobber
rm -rf dir/                 # Remove recursively
mkdir -p path/to/dir        # Create with parents
touch file                  # Create empty file
chmod 755 script.sh         # Make executable

# Text processing
grep -r "pattern" dir/      # Recursive search
sed -i 's/old/new/g' file   # In-place replace
awk '{print $1}' file       # Print first column
cut -d: -f1 file            # Cut by delimiter
sort -u file                # Sort unique
uniq -c file                # Count duplicates

# System operations
ps aux | grep process       # Find process
kill -TERM pid              # Graceful shutdown
kill -KILL pid              # Force kill
df -h                       # Disk usage
du -sh dir/                 # Directory size
free -h                     # Memory usage

# Network operations
curl -sf URL                # Silent fetch
wget -q URL                 # Quiet download
nc -zv host port            # Port check
ping -c1 host               # Single ping

# Compression
tar czf archive.tar.gz dir/ # Create tarball
tar xzf archive.tar.gz      # Extract tarball
gzip file                   # Compress file
gunzip file.gz              # Decompress file
```

### ShellCheck Directives

```bash
# Disable specific warnings
# shellcheck disable=SC2034  # Variable appears unused
UNUSED_VAR="value"

# Disable for entire file
# shellcheck disable=SC1090  # Can't follow sourced file

# Source external files
# shellcheck source=lib/common.sh
source lib/common.sh

# Specify shell
# shellcheck shell=bash

# Exclude patterns
# shellcheck disable=SC2086  # Double quote to prevent globbing
```

---

## Appendix B: Tool Installation

### ShellCheck

```bash
# macOS
brew install shellcheck

# Ubuntu/Debian
apt-get install shellcheck

# From source
git clone https://github.com/koalaman/shellcheck
cd shellcheck && cabal install
```

### BATS

```bash
# macOS
brew install bats-core

# Ubuntu/Debian
npm install -g bats

# From source
git clone https://github.com/bats-core/bats-core.git
cd bats-core && ./install.sh /usr/local
```

### Additional Tools

```bash
# shfmt - Shell formatter
GO111MODULE=on go install mvdan.cc/sh/v3/cmd/shfmt@latest

# shellharden - Shell hardening
cargo install shellharden
```

---

## Appendix C: Resources

### Official Documentation
- [Bash Reference Manual](https://www.gnu.org/software/bash/manual/)
- [POSIX Shell Command Language](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [ShellCheck Wiki](https://github.com/koalaman/shellcheck/wiki)

### Style Guides
- [Google Shell Style Guide](https://google.github.io/styleguide/shellguide.html)
- [Defensive Bash Programming](https://kfirlavi.herokuapp.com/blog/2012/11/14/defensive-bash-programming/)

### Learning Resources
- [Bash Hackers Wiki](https://wiki.bash-hackers.org/)
- [Advanced Bash-Scripting Guide](https://tldp.org/LDP/abs/html/)
- [explainshell.com](https://explainshell.com/) - Command explanation

---

**Document Version**: 1.0
**Last Updated**: 2026-01-08
**Maintained By**: DevOps Team
**License**: MIT
