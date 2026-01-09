# Appendix: Shell Scripting for DevOps Excellence

**Purpose**
Establish production-grade shell scripting standards that align with our functional core principles: treating scripts as **composable, testable, side-effect-aware programs** rather than ad-hoc automation. Complements:

- Effect algebras & interpreters (§ Algebraic Effects)
- Concurrency & time modeling (§ Concurrency & Time)
- FSM patterns for deployment workflows
- Testing & observability standards across stacks

---

## §1. Non-Negotiables

1. **No hidden side effects**
   Every filesystem modification, network call, or process spawn must be:
   - Explicit in function signatures (via naming or effect comments)
   - Logged with structured output
   - Wrapped in validation (pre-conditions, post-conditions)

2. **Pure core, effectful shell**
   Separate validation/computation (pure functions) from execution (side effects).
   Test the core without touching the filesystem or network.

3. **One writer per resource**
   Scripts operating on shared resources (config files, DB, deployment state) must:
   - Use locking mechanisms (flock, advisory locks)
   - Document contention expectations
   - Fail explicitly on conflicts

4. **Idempotency by default**
   Every script must be safely re-runnable:
   - Check state before mutation ("does file exist?", "is service running?")
   - Use atomic operations (temp file → mv, CREATE TABLE IF NOT EXISTS)
   - Document non-idempotent operations with clear warnings

5. **Cross-platform by design**
   Default to POSIX-compliant patterns; document Bash-specific requirements.
   Use portable command abstractions (see §3.2).

6. **Security > convenience**
   - No eval, no unquoted variables, no hardcoded secrets
   - Input validation mandatory (whitelist > blacklist)
   - Least privilege (drop permissions when possible)

7. **Observable by default**
   Emit structured logs (JSON or key=value) at:
   - Script start/end with context (env, version, args)
   - State transitions (FSM-style)
   - Error boundaries with full context

---

## §2. Mental Model: Scripts as Effect Programs

### §2.1 The Effect Algebra Perspective

Treat shell scripts like domain services that **describe** effects before executing them:

```bash
# ❌ BAD: Imperative mess with hidden effects
deploy() {
    docker build -t app .
    docker push app
    ssh prod "docker pull app && systemctl restart app"
}

# ✅ GOOD: Explicit effect boundaries
deploy() {
    local image_tag="$1"

    # Pure validation
    validate_image_tag "$image_tag" || return 1
    validate_environment "production" || return 1

    # Effects logged and wrapped
    log_info "Starting deployment" "image=$image_tag" "env=production"

    build_image "$image_tag" || { log_error "Build failed"; return 1; }
    push_image "$image_tag" || { log_error "Push failed"; return 1; }
    deploy_to_production "$image_tag" || { log_error "Deploy failed"; return 1; }

    validate_deployment "$image_tag" || { log_error "Validation failed"; return 1; }
    log_info "Deployment complete"
}
```

### §2.2 Script Lifecycle as FSM

Model deployment/operation scripts as **finite state machines**:

| State | Valid Events | Next State | Actions | Guards |
|-------|--------------|------------|---------|--------|
| `Init` | `START` | `Validating` | Load config, check deps | - |
| `Validating` | `VALID` | `Building` | - | All deps present |
| `Validating` | `INVALID` | `Failed` | Log errors | Any dep missing |
| `Building` | `BUILT` | `Testing` | Run build cmd | Build exit 0 |
| `Building` | `BUILD_FAILED` | `Failed` | Cleanup, log | Build exit ≠ 0 |
| `Testing` | `PASSED` | `Deploying` | - | Tests pass |
| `Testing` | `FAILED` | `Failed` | Rollback | Tests fail |
| `Deploying` | `DEPLOYED` | `Success` | Health check | Deploy exit 0 |
| `Deploying` | `DEPLOY_FAILED` | `RollingBack` | Trigger rollback | Deploy exit ≠ 0 |

Implementation:
```bash
# Explicit state tracking
STATE="Init"

transition() {
    local from="$STATE"
    local event="$1"
    local to="$2"

    log_info "Transition" "from=$from" "event=$event" "to=$to"
    STATE="$to"
}

deploy_fsm() {
    transition "START" "Validating"

    if ! validate_preconditions; then
        transition "INVALID" "Failed"
        return 1
    fi

    transition "VALID" "Building"
    if ! build_artifact; then
        transition "BUILD_FAILED" "Failed"
        return 1
    fi

    # Continue FSM...
}
```

### §2.3 Time as Data

Following concurrency principles, timers/deadlines are **explicit state**:

```bash
# ✅ Deadline propagation
execute_with_deadline() {
    local deadline_ts="$1"  # Unix timestamp
    shift
    local -a command=("$@")

    local now_ts
    now_ts=$(date +%s)
    local remaining=$((deadline_ts - now_ts))

    if [ "$remaining" -le 0 ]; then
        log_error "Deadline exceeded before execution" \
            "deadline=$deadline_ts" "now=$now_ts"
        return 1
    fi

    log_debug "Executing with timeout" "remaining=${remaining}s"
    timeout "${remaining}s" "${command[@]}"
}

# ✅ Retry with exponential backoff as events
retry_with_backoff() {
    local max_attempts="$1"
    local base_delay="$2"
    shift 2
    local -a command=("$@")

    local attempt=1
    local delay="$base_delay"

    while [ $attempt -le "$max_attempts" ]; do
        log_info "Attempt $attempt/$max_attempts" "delay_if_fail=${delay}s"

        if "${command[@]}"; then
            log_info "Success on attempt $attempt"
            return 0
        fi

        if [ $attempt -lt "$max_attempts" ]; then
            log_warn "Retry scheduled" "attempt=$attempt" "delay=${delay}s"
            sleep "$delay"
            delay=$((delay * 2))  # Exponential backoff
        fi

        attempt=$((attempt + 1))
    done

    log_error "All attempts exhausted" "max_attempts=$max_attempts"
    return 1
}
```

---

## §3. Composable Script Architecture

### §3.1 Module System (Effect Ports Pattern)

Organize scripts as **ports and adapters**:

```
scripts/
├── core/                    # Pure logic, no side effects
│   ├── validators.sh        # Input validation functions
│   ├── parsers.sh           # Config parsing, transformation
│   └── state-machine.sh     # FSM transition logic
├── ports/                   # Effect algebras (interfaces)
│   ├── logger.sh            # Logging port (interface only)
│   ├── filesystem.sh        # File operations port
│   ├── network.sh           # Network operations port
│   └── process.sh           # Process management port
├── adapters/                # Port implementations
│   ├── logger-json.sh       # JSON logger adapter
│   ├── logger-syslog.sh     # Syslog adapter
│   ├── filesystem-real.sh   # Real filesystem adapter
│   ├── filesystem-mock.sh   # Test mock adapter
│   └── aws.sh               # AWS SDK adapter
└── workflows/               # Orchestration scripts
    ├── deploy.sh            # Uses core + adapters
    ├── backup.sh
    └── rollback.sh
```

**Core example (pure)**:
```bash
# core/validators.sh - No side effects

validate_semver() {
    local version="$1"
    [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]
}

validate_environment() {
    local env="$1"
    case "$env" in
        dev|staging|production) return 0;;
        *) return 1;;
    esac
}

compute_next_state() {
    local current="$1"
    local event="$2"

    # Pure state machine logic
    case "$current:$event" in
        "Validating:VALID") echo "Building";;
        "Building:BUILT") echo "Testing";;
        "Testing:PASSED") echo "Deploying";;
        "Deploying:DEPLOYED") echo "Success";;
        *:*) echo "Failed";;
    esac
}
```

**Port example (interface)**:
```bash
# ports/logger.sh - Abstract interface

# Interface declaration (comment-based contract)
# log_info(message, context...)
# log_error(message, context...)
# log_debug(message, context...)

# Validate implementation exists
if ! declare -f log_info >/dev/null; then
    echo "Error: log_info not implemented" >&2
    exit 1
fi
```

**Adapter example (implementation)**:
```bash
# adapters/logger-json.sh - Concrete implementation

source "$(dirname "${BASH_SOURCE[0]}")/../ports/logger.sh"

log_info() {
    local message="$1"
    shift
    _log "INFO" "$message" "$@"
}

log_error() {
    local message="$1"
    shift
    _log "ERROR" "$message" "$@"
}

_log() {
    local level="$1"
    local message="$2"
    shift 2

    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    # JSON structured logging
    printf '{"timestamp":"%s","level":"%s","message":"%s"' \
        "$timestamp" "$level" "$message"

    # Add context fields
    for ctx in "$@"; do
        local key="${ctx%%=*}"
        local value="${ctx#*=}"
        printf ',"%s":"%s"' "$key" "$value"
    done

    printf '}\n'
}
```

### §3.2 Cross-Platform Abstraction Layer

Use adapters for platform-specific operations:

```bash
# adapters/filesystem-portable.sh

portable_realpath() {
    local path="$1"

    if command -v realpath >/dev/null 2>&1; then
        realpath "$path"
    elif command -v greadlink >/dev/null 2>&1; then
        greadlink -f "$path"
    else
        python3 -c "import os; print(os.path.realpath('$path'))"
    fi
}

portable_sed_inplace() {
    if sed --version >/dev/null 2>&1; then
        sed -i "$@"  # GNU sed
    else
        sed -i '' "$@"  # BSD sed
    fi
}

portable_date_rfc3339() {
    if date --version >/dev/null 2>&1; then
        date -u +"%Y-%m-%dT%H:%M:%SZ"  # GNU date
    else
        date -u +"%Y-%m-%dT%H:%M:%SZ"  # BSD date (same format fortunately)
    fi
}
```

---

## §4. Security Patterns (Zero Trust Model)

### §4.1 Input Validation (Whitelist-First)

```bash
# core/validators.sh

# ✅ Whitelist validation
validate_alphanumeric() {
    local input="$1"
    [[ "$input" =~ ^[a-zA-Z0-9_-]+$ ]]
}

validate_aws_region() {
    local region="$1"
    case "$region" in
        us-east-1|us-west-2|eu-west-1|ap-southeast-1) return 0;;
        *) return 1;;
    esac
}

# ✅ Path sanitization with confinement
sanitize_path() {
    local path="$1"
    local base_dir="${2:-/var/app}"

    # Remove leading/trailing whitespace
    path=$(echo "$path" | xargs)

    # Reject directory traversal
    if [[ "$path" == *".."* ]]; then
        log_error "Path traversal detected" "path=$path"
        return 1
    fi

    # Ensure within allowed directory
    local full_path="${base_dir}/${path}"
    local canonical
    canonical=$(portable_realpath "$full_path" 2>/dev/null) || {
        log_error "Invalid path" "path=$path"
        return 1
    }

    if [[ ! "$canonical" == "$base_dir"* ]]; then
        log_error "Path outside allowed directory" \
            "path=$canonical" "allowed=$base_dir"
        return 1
    fi

    echo "$canonical"
}
```

### §4.2 Secrets Management (Effect Boundary)

```bash
# ports/secrets.sh - Secret loading port

# Interface:
# load_secret(key_name) -> value or error

# adapters/secrets-aws-ssm.sh - AWS Parameter Store implementation
load_secret() {
    local key_name="$1"

    log_debug "Loading secret from AWS SSM" "key=$key_name"

    aws ssm get-parameter \
        --name "$key_name" \
        --with-decryption \
        --query 'Parameter.Value' \
        --output text 2>/dev/null || {
            log_error "Failed to load secret" "key=$key_name"
            return 1
        }
}

# adapters/secrets-env.sh - Environment variable implementation (dev only)
load_secret() {
    local key_name="$1"
    local value="${!key_name:-}"

    if [ -z "$value" ]; then
        log_error "Secret not found in environment" "key=$key_name"
        return 1
    fi

    echo "$value"
}

# Usage in workflow
deploy() {
    # Load secret via port (implementation injected)
    local db_password
    db_password=$(load_secret "DB_PASSWORD") || return 1

    # Use secret (never logged)
    connect_database "user=app password=$db_password"

    # Cleanup
    unset db_password
}

# Cleanup on exit
cleanup_secrets() {
    unset DB_PASSWORD API_KEY JWT_SECRET
}
trap cleanup_secrets EXIT
```

### §4.3 Command Injection Prevention

```bash
# ❌ DANGEROUS: Command injection via eval
bad_execute() {
    local user_input="$1"
    eval "$user_input"  # NEVER DO THIS
}

# ❌ DANGEROUS: Command injection via sh -c
bad_shell() {
    local file="$1"
    sh -c "cat $file"  # Unquoted variable
}

# ✅ SAFE: Array execution (no word splitting)
safe_execute() {
    local -a command=("$@")
    "${command[@]}"
}

# ✅ SAFE: Proper quoting
safe_cat() {
    local file="$1"
    cat "$file"  # Quoted variable
}

# ✅ SAFE: Whitelist validation before execution
safe_docker_command() {
    local action="$1"

    # Whitelist validation
    case "$action" in
        start|stop|restart|status) ;;
        *)
            log_error "Invalid docker action" "action=$action"
            return 1
            ;;
    esac

    # Safe execution with validated input
    docker "$action" myapp
}
```

---

## §5. Testing Strategies

### §5.1 Test Pyramid for Scripts

```
      ╱╲
     ╱  ╲      E2E Tests (few)
    ╱────╲     - Full workflow on test infra
   ╱      ╲    - Real deps (docker, DB)
  ╱────────╲   Integration Tests (some)
 ╱          ╲  - Workflows with mock adapters
╱────────────╲ Unit Tests (many)
               - Pure functions
               - Validators, parsers, FSM logic
```

### §5.2 BATS Unit Tests (Pure Functions)

```bash
# tests/validators.bats

#!/usr/bin/env bats

load test-helper
load ../core/validators

@test "validate_semver: accepts valid versions" {
    run validate_semver "1.2.3"
    [ "$status" -eq 0 ]

    run validate_semver "1.0.0-alpha.1"
    [ "$status" -eq 0 ]
}

@test "validate_semver: rejects invalid versions" {
    run validate_semver "1.2"
    [ "$status" -eq 1 ]

    run validate_semver "v1.2.3"
    [ "$status" -eq 1 ]
}

@test "sanitize_path: prevents directory traversal" {
    run sanitize_path "../../../etc/passwd" "/var/app"
    [ "$status" -eq 1 ]
    [[ "$output" == *"Path traversal detected"* ]]
}

@test "compute_next_state: implements FSM correctly" {
    run compute_next_state "Validating" "VALID"
    [ "$output" = "Building" ]

    run compute_next_state "Building" "BUILT"
    [ "$output" = "Testing" ]

    run compute_next_state "Invalid" "UNKNOWN"
    [ "$output" = "Failed" ]
}
```

### §5.3 Integration Tests (With Mock Adapters)

```bash
# tests/deploy-integration.bats

setup() {
    export TEST_TEMP_DIR="$(mktemp -d)"
    export LOG_JSON="true"

    # Inject mock adapters
    source "$(pwd)/adapters/filesystem-mock.sh"
    source "$(pwd)/adapters/logger-test.sh"

    # Load workflow under test
    source "$(pwd)/workflows/deploy.sh"
}

teardown() {
    rm -rf "$TEST_TEMP_DIR"
}

@test "deploy: creates backup before deployment" {
    # Arrange
    setup_mock_app_state "v1.0.0"

    # Act
    run deploy "v1.1.0"

    # Assert
    [ "$status" -eq 0 ]
    [ -f "$TEST_TEMP_DIR/backups/v1.0.0.tar.gz" ]
}

@test "deploy: rolls back on validation failure" {
    # Arrange
    setup_mock_app_state "v1.0.0"
    MOCK_VALIDATION_SHOULD_FAIL="true"

    # Act
    run deploy "v1.1.0"

    # Assert
    [ "$status" -eq 1 ]
    current_version=$(get_deployed_version)
    [ "$current_version" = "v1.0.0" ]
}

@test "deploy: follows FSM transitions" {
    run deploy "v1.1.0"

    # Check logged transitions
    assert_log_contains "from=Init" "event=START"
    assert_log_contains "from=Validating" "event=VALID"
    assert_log_contains "from=Building" "event=BUILT"
    assert_log_contains "from=Testing" "event=PASSED"
    assert_log_contains "from=Deploying" "event=DEPLOYED"
}
```

### §5.4 Property-Based Testing (Optional)

For critical validators, use property-based testing:

```bash
# tests/properties.bats

@test "property: sanitize_path always returns canonical paths" {
    for i in {1..100}; do
        # Generate random path with various tricks
        path=$(generate_random_path)

        if sanitized=$(sanitize_path "$path" "/var/app"); then
            # Property: result must be absolute and within base
            [[ "$sanitized" == /var/app/* ]]
            [[ ! "$sanitized" == *".."* ]]
        fi
    done
}
```

---

## §6. Observability & Instrumentation

### §6.1 Structured Logging (Effect at Edge)

```bash
# adapters/logger-json.sh

_log() {
    local level="$1"
    local message="$2"
    shift 2
    local -a context=("$@")

    local timestamp
    timestamp=$(portable_date_rfc3339)

    # Base log entry
    local log_entry
    log_entry=$(jq -n \
        --arg ts "$timestamp" \
        --arg lvl "$level" \
        --arg msg "$message" \
        --arg script "$(basename "$0")" \
        --arg pid "$$" \
        '{
            timestamp: $ts,
            level: $lvl,
            message: $msg,
            script: $script,
            pid: $pid
        }')

    # Add context fields
    for ctx in "${context[@]}"; do
        local key="${ctx%%=*}"
        local value="${ctx#*=}"
        log_entry=$(echo "$log_entry" | jq --arg k "$key" --arg v "$value" '. + {($k): $v}')
    done

    echo "$log_entry" | tee -a "$LOG_FILE"
}
```

### §6.2 FSM State Transitions as Events

```bash
transition() {
    local from="$STATE"
    local event="$1"
    local to="$2"
    shift 2
    local -a context=("$@")

    # Log transition as structured event
    log_info "FSM transition" \
        "from=$from" \
        "event=$event" \
        "to=$to" \
        "aggregate_id=${DEPLOYMENT_ID:-unknown}" \
        "${context[@]}"

    # Emit metric
    emit_metric "fsm.transitions" 1 \
        "state=$to" \
        "event=$event"

    # Update state
    STATE="$to"
}
```

### §6.3 Metrics Export

```bash
# adapters/metrics-statsd.sh

emit_metric() {
    local metric_name="$1"
    local value="$2"
    shift 2
    local -a tags=("$@")

    local tags_str
    tags_str=$(IFS=,; echo "${tags[*]}")

    # StatsD format
    echo "${metric_name}:${value}|c|#${tags_str}" | \
        nc -u -w0 localhost 8125
}

# Usage in workflow
deploy() {
    local start_time
    start_time=$(date +%s)

    # ... deployment logic ...

    local duration=$(( $(date +%s) - start_time ))
    emit_metric "deploy.duration" "$duration" \
        "env=production" \
        "service=$SERVICE_NAME"
}
```

---

## §7. Stack-Specific Guidelines

### §7.1 CI/CD Integration

**Principle**: Scripts are infrastructure-agnostic; CI platforms inject adapters.

#### GitLab CI
```yaml
# .gitlab-ci.yml

variables:
  LOGGER_ADAPTER: "json"
  SECRETS_ADAPTER: "gitlab-ci"

before_script:
  - export SCRIPT_DIR="$(pwd)/scripts"
  - source "$SCRIPT_DIR/adapters/logger-${LOGGER_ADAPTER}.sh"
  - source "$SCRIPT_DIR/adapters/secrets-${SECRETS_ADAPTER}.sh"

validate:
  script:
    - shellcheck scripts/**/*.sh
    - bats tests/*.bats

deploy:
  script:
    - ./scripts/workflows/deploy.sh --env production "$CI_COMMIT_TAG"
  environment:
    name: production
  only:
    - tags
```

#### GitHub Actions
```yaml
# .github/workflows/deploy.yml

jobs:
  deploy:
    runs-on: ubuntu-latest
    env:
      LOGGER_ADAPTER: json
      SECRETS_ADAPTER: github-actions
    steps:
      - uses: actions/checkout@v3

      - name: Run deployment
        env:
          AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
        run: |
          source scripts/adapters/logger-${LOGGER_ADAPTER}.sh
          source scripts/adapters/secrets-${SECRETS_ADAPTER}.sh
          ./scripts/workflows/deploy.sh --env production "${{ github.ref_name }}"
```

### §7.2 Docker/Container Integration

```dockerfile
# Dockerfile for script execution environment

FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    bash \
    jq \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash scriptuser

# Copy scripts
COPY --chown=scriptuser:scriptuser scripts/ /app/scripts/

# Set permissions
RUN find /app/scripts -type f -name "*.sh" -exec chmod +x {} \;

USER scriptuser
WORKDIR /app

# Inject production adapters
ENV LOGGER_ADAPTER=json
ENV SECRETS_ADAPTER=aws-ssm
ENV FILESYSTEM_ADAPTER=real

ENTRYPOINT ["/app/scripts/workflows/deploy.sh"]
```

### §7.3 Terraform Provisioning

```hcl
# terraform/main.tf

resource "null_resource" "provision_instance" {
  provisioner "file" {
    source      = "${path.module}/../scripts"
    destination = "/tmp/scripts"

    connection {
      type        = "ssh"
      user        = "ubuntu"
      private_key = file(var.ssh_key_path)
      host        = aws_instance.app.public_ip
    }
  }

  provisioner "remote-exec" {
    inline = [
      "chmod +x /tmp/scripts/**/*.sh",

      # Inject production adapters
      "export LOGGER_ADAPTER=syslog",
      "export SECRETS_ADAPTER=aws-ssm",

      # Run provisioning with FSM tracking
      "/tmp/scripts/workflows/provision.sh --env ${var.environment}"
    ]

    connection {
      type        = "ssh"
      user        = "ubuntu"
      private_key = file(var.ssh_key_path)
      host        = aws_instance.app.public_ip
    }
  }
}
```

---

## §8. Anti-Patterns & Common Mistakes

### §8.1 Effect Leakage

```bash
# ❌ BAD: Effects mixed with logic
validate_and_deploy() {
    if [ -f "$CONFIG_FILE" ]; then  # Side effect: filesystem read
        local config=$(cat "$CONFIG_FILE")  # Side effect
        if [ "$config" = "valid" ]; then
            docker deploy app  # Side effect
        fi
    fi
}

# ✅ GOOD: Separated pure and effectful
validate_config_content() {
    # Pure: takes content as input
    local content="$1"
    [ "$content" = "valid" ]
}

deploy_workflow() {
    # Effects at edges
    local config
    config=$(cat "$CONFIG_FILE") || return 1

    # Pure validation
    if ! validate_config_content "$config"; then
        log_error "Invalid config"
        return 1
    fi

    # Explicit effect
    docker deploy app
}
```

### §8.2 Hidden Mutation

```bash
# ❌ BAD: Global state mutation
STATE="idle"

process_event() {
    STATE="processing"  # Hidden side effect
    do_work
    STATE="done"
}

# ✅ GOOD: Explicit state transitions
process_event_pure() {
    local current_state="$1"
    local event="$2"

    # Pure computation
    case "$current_state:$event" in
        "idle:START") echo "processing";;
        "processing:COMPLETE") echo "done";;
        *) echo "error";;
    esac
}

# Wrapper handles state
STATE="idle"
process_event() {
    local event="$1"
    local next_state

    next_state=$(process_event_pure "$STATE" "$event")
    transition "$event" "$next_state"
}
```

### §8.3 Non-Idempotent Operations

```bash
# ❌ BAD: Not idempotent
setup_database() {
    createdb myapp  # Fails on re-run
    psql -c "CREATE TABLE users (...)"  # Fails on re-run
}

# ✅ GOOD: Idempotent checks
setup_database() {
    if ! psql -lqt | cut -d \| -f 1 | grep -qw myapp; then
        log_info "Creating database"
        createdb myapp
    else
        log_info "Database exists, skipping"
    fi

    psql myapp -c "
        CREATE TABLE IF NOT EXISTS users (...);
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
    "
}
```

### §8.4 Useless Use of Cat (Performance)

```bash
# ❌ BAD: Unnecessary process spawn
cat file.txt | grep pattern

# ✅ GOOD: Direct input redirection
grep pattern < file.txt
# Or better:
grep pattern file.txt
```

---

## §9. LLM Guidance Hooks

When integrating shell scripts with AI tooling (MCP, RAG):

### §9.1 Ground Rules Document

Pin a "Shell Scripting Ground Rules" doc:

```markdown
# Shell Scripting Ground Rules

## Non-Negotiables
- Pure core, effectful shell: separate validation from execution
- No eval, no unquoted variables, no hardcoded secrets
- Input validation mandatory (whitelist > blacklist)
- Idempotency by default: check before mutate
- Structured logging: JSON or key=value at all effect boundaries

## Effect Boundaries
- Filesystem: adapters/filesystem-*.sh
- Network: adapters/network-*.sh
- Secrets: adapters/secrets-*.sh (never hardcode)
- Logs: adapters/logger-*.sh

## Testing
- Unit tests for pure functions (core/)
- Integration tests with mock adapters
- Property tests for critical validators

## Observability
- Log all FSM transitions: from, to, event, context
- Emit metrics: duration, error rates, state occupancy
```

### §9.2 Code Review Prompts

Add to AI code review prompts:

```markdown
## Shell Script Review Checklist

❓ Are side effects explicit and isolated to adapters?
❓ Is input validation using whitelist patterns?
❓ Are secrets loaded via ports, never hardcoded?
❓ Is the script idempotent (safe to re-run)?
❓ Are all state transitions logged?
❓ Are variables quoted to prevent word splitting?
❓ Does the script work on POSIX sh, or is Bash requirement documented?
❓ Are there tests for core logic (validators, parsers)?
```

### §9.3 Prompt Engineering

When asking LLMs to generate shell scripts:

```
Generate a deployment script following these rules:
- Separate pure validation functions in core/ from effects in adapters/
- Model deployment as FSM with states: Init, Validating, Building, Testing, Deploying, Success, Failed
- Use structured logging (JSON) for all transitions
- Implement idempotency checks before mutations
- Use whitelist validation for all inputs
- No secrets in code - use load_secret() port
```

---

## §10. Decision Matrix: When to Use Shell vs Application Code

| Criteria | Shell Script | Application Code (Rust/PHP/TS) |
|----------|--------------|--------------------------------|
| Task nature | Orchestration, glue, system ops | Complex business logic, data processing |
| Dependencies | System commands, basic tools | Libraries, frameworks, external APIs |
| Complexity | Linear workflows, simple FSMs | Multi-step sagas, complex state machines |
| Performance | Not critical (< 1 req/sec) | High throughput (> 100 req/sec) |
| Type safety | Low (runtime validation only) | High (compile-time guarantees) |
| Testing needs | Integration tests sufficient | Unit + integration + property tests |
| Deployment | Copy files, chmod +x | Build, package, deploy artifact |
| Portability | POSIX sh → Bash → OS-specific | Containerized, platform-abstracted |

**Rule of thumb**: Use shell for **infrastructure automation** where system command composition is natural. Use application code for **domain logic** where type safety and testability are critical.

---

## §11. Adoption Checklist

To integrate these patterns into existing projects:

- [ ] **Audit existing scripts**: Identify pure vs effectful sections
- [ ] **Establish directory structure**: core/, ports/, adapters/, workflows/
- [ ] **Extract validators**: Move input validation to core/validators.sh
- [ ] **Create port interfaces**: Define logger, filesystem, secrets, network ports
- [ ] **Implement adapters**: Real adapters for production, mock for tests
- [ ] **Add structured logging**: JSON or key=value at all effect boundaries
- [ ] **Write unit tests**: BATS tests for pure functions
- [ ] **Document FSMs**: For deployment/backup/restore workflows
- [ ] **Add pre-commit hooks**: ShellCheck + syntax validation
- [ ] **Integrate with CI/CD**: Inject adapters via environment variables
- [ ] **Pin LLM ground rules**: Add shell scripting standards to MCP/RAG context

---

## Appendix A: Quick Reference Card

### Error Handling
```bash
set -euo pipefail        # Fail fast
trap cleanup EXIT        # Always cleanup
```

### Quoting Rules
```bash
"$var"                   # Always quote variables
"${array[@]}"            # Quote array expansions
$(command)               # Use $() not backticks
```

### Portable Commands
```bash
portable_realpath        # Use adapter, not direct realpath
portable_sed_inplace     # Handle GNU vs BSD sed
portable_date_rfc3339    # Cross-platform timestamps
```

### Validation Patterns
```bash
[[ "$input" =~ ^[a-z]+$ ]]  # Whitelist regex
case "$input" in            # Whitelist case
    valid1|valid2) ;;
    *) return 1;;
esac
```

### Effect Boundaries
```bash
source adapters/logger-json.sh      # Inject logger
source adapters/secrets-aws-ssm.sh  # Inject secrets
source adapters/filesystem-real.sh  # Inject filesystem
```

---

## Appendix B: Tool Setup

### ShellCheck
```bash
# macOS
brew install shellcheck

# Ubuntu/Debian
apt-get install shellcheck

# Pre-commit hook
echo '#!/bin/bash
shellcheck scripts/**/*.sh' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### BATS (Bash Automated Testing System)
```bash
# macOS
brew install bats-core

# Ubuntu/Debian
npm install -g bats

# Run tests
bats tests/*.bats
```

### jq (JSON processing)
```bash
# macOS
brew install jq

# Ubuntu/Debian
apt-get install jq
```

---

## Appendix C: Further Reading

- [POSIX Shell Specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [Google Shell Style Guide](https://google.github.io/styleguide/shellguide.html)
- [ShellCheck Wiki](https://github.com/koalaman/shellcheck/wiki)
- [Bash Pitfalls](https://mywiki.wooledge.org/BashPitfalls)
- Algebraic Effects & Handlers (see § Algebraic Effects appendix)
- FSM Modeling (see § FSM appendix)
- Concurrency & Time (see § Concurrency & Time appendix)

---

**Version**: 1.0
**Last Updated**: 2026-01-08
**Maintained By**: DevOps & Platform Engineering
**Related Appendices**: FSM, Algebraic Effects, Concurrency & Time, Pattern Playbook
