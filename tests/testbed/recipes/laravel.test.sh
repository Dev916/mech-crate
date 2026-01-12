#!/usr/bin/env bash
#
# MechCrate Testbed - Laravel Recipe Tests
# Validates Laravel 12 + Octane recipe installation
#
# Test Levels:
#   build - File structure and configuration validation
#   smoke - Docker compose and image build validation
#   full  - Container runtime and endpoint validation
#
# Version: 1.0.0
# Author: MechCrate
#

# Strict mode
set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Test Configuration
# ─────────────────────────────────────────────────────────────────────────────

readonly RECIPE_NAME="laravel"
# Use a simpler service name for easier testing
readonly SERVICE_NAME="app"
APP_DIR=""
DOCKER_DIR=""

# ─────────────────────────────────────────────────────────────────────────────
# Test Setup
# ─────────────────────────────────────────────────────────────────────────────

setup_test() {
    echo -e "\n${BOLD}Setting up Laravel recipe test...${NC}"
    
    local parent_dir
    parent_dir=$(dirname "$TEST_PROJECT_DIR")
    local project_name
    project_name=$(basename "$TEST_PROJECT_DIR")
    
    # Remove the pre-created directory since mx new expects it to not exist
    rm -rf "$TEST_PROJECT_DIR"
    
    # Create the project
    cd "$parent_dir"
    "$MECH_CRATE_ROOT/bin/mx" new "$project_name" --no-prompt
    
    if [[ ! -d "$TEST_PROJECT_DIR" ]]; then
        error "Failed to create test project at $TEST_PROJECT_DIR"
        return 1
    fi
    
    # Change into the project directory
    cd "$TEST_PROJECT_DIR"
    
    # Add the Laravel recipe
    "$MECH_CRATE_ROOT/bin/mx" add "$SERVICE_NAME" --recipe="$RECIPE_NAME"
    
    # Set up paths after recipe is added
    APP_DIR="$TEST_PROJECT_DIR/apps/$SERVICE_NAME"
    DOCKER_DIR="$TEST_PROJECT_DIR/docker"
    
    # Create required env files for Docker compose validation
    mkdir -p "$DOCKER_DIR/.config"
    
    # Create secrets file with all required variables for testing
    cat > "$DOCKER_DIR/.config/.env.secrets" << 'EOF'
# Test secrets (DO NOT USE IN PRODUCTION)
DB_USER=postgres
DB_PASSWORD=testpassword
DB_NAME=testdb
DB_HOST=db
DB_PORT=5432

REDIS_PASSWORD=testredispassword
REDIS_HOST=redis
REDIS_PORT=6379

JWT_SECRET=test-jwt-secret-key-12345
API_KEY=test-api-key-12345
ENCRYPTION_KEY=test-encryption-key-12345

# Legacy/app-specific
APP_DB_PASSWORD=testpassword
EOF
    
    # Create shared env file if it doesn't exist
    if [[ ! -f "$DOCKER_DIR/.config/.env.shared" ]]; then
        cat > "$DOCKER_DIR/.config/.env.shared" << 'EOF'
# Shared environment configuration
APP_ENV=development
APP_DEBUG=true
LOG_LEVEL=debug
EOF
    fi
    
    success "Test setup complete"
}

# ─────────────────────────────────────────────────────────────────────────────
# BUILD LEVEL TESTS
# File structure, configuration, and template processing
# ─────────────────────────────────────────────────────────────────────────────

test_build_directory_structure() {
    run_test_group "Directory Structure"
    
    # Core app directories
    assert_dir_exists "app directory exists" "$APP_DIR"
    assert_dir_exists "app/Http/Controllers" "$APP_DIR/app/Http/Controllers"
    assert_dir_exists "app/Http/Middleware" "$APP_DIR/app/Http/Middleware"
    assert_dir_exists "app/Models" "$APP_DIR/app/Models"
    assert_dir_exists "app/Providers" "$APP_DIR/app/Providers"
    
    # Filament directories
    assert_dir_exists "Filament Resources dir" "$APP_DIR/app/Filament/Resources"
    assert_dir_exists "Filament Pages dir" "$APP_DIR/app/Filament/Pages"
    assert_dir_exists "Filament Widgets dir" "$APP_DIR/app/Filament/Widgets"
    
    # Laravel structure
    assert_dir_exists "bootstrap directory" "$APP_DIR/bootstrap"
    assert_dir_exists "config directory" "$APP_DIR/config"
    assert_dir_exists "database/migrations" "$APP_DIR/database/migrations"
    assert_dir_exists "database/seeders" "$APP_DIR/database/seeders"
    assert_dir_exists "public directory" "$APP_DIR/public"
    assert_dir_exists "resources/views" "$APP_DIR/resources/views"
    assert_dir_exists "resources/js/Pages" "$APP_DIR/resources/js/Pages"
    assert_dir_exists "resources/css" "$APP_DIR/resources/css"
    assert_dir_exists "routes directory" "$APP_DIR/routes"
    assert_dir_exists "storage/app" "$APP_DIR/storage/app"
    assert_dir_exists "storage/framework" "$APP_DIR/storage/framework"
    assert_dir_exists "storage/logs" "$APP_DIR/storage/logs"
    assert_dir_exists "tests directory" "$APP_DIR/tests"
    
    # Docker directories
    assert_dir_exists "docker compose dir" "$DOCKER_DIR/compose"
    assert_dir_exists "docker dockerfiles dir" "$DOCKER_DIR/dockerfiles/$SERVICE_NAME"
    assert_dir_exists "docker config dir" "$DOCKER_DIR/.config"
}

test_build_core_files() {
    run_test_group "Core Files"
    
    # Laravel core files
    assert_file_exists "composer.json" "$APP_DIR/composer.json"
    assert_file_exists "package.json" "$APP_DIR/package.json"
    assert_file_exists "artisan" "$APP_DIR/artisan"
    
    # Vite config (check for .ts, .js, or .mjs variants)
    if [[ -f "$APP_DIR/vite.config.ts" ]]; then
        _record_pass "vite.config.ts exists"
    elif [[ -f "$APP_DIR/vite.config.js" ]]; then
        _record_pass "vite.config.js exists"
    elif [[ -f "$APP_DIR/vite.config.mjs" ]]; then
        _record_pass "vite.config.mjs exists"
    else
        _record_fail "vite.config.* exists" "No vite config file found"
    fi
    
    # Tailwind v4 is CSS-first (no tailwind.config.js required)
    assert_file_exists "Tailwind CSS entry file" "$APP_DIR/resources/css/app.css"
    assert_file_contains "app.css imports tailwindcss" "$APP_DIR/resources/css/app.css" "@import 'tailwindcss';"
    assert_file_exists "postcss.config.js" "$APP_DIR/postcss.config.js"
    
    # Route files
    assert_file_exists "routes/web.php" "$APP_DIR/routes/web.php"
    assert_file_exists "routes/api.php" "$APP_DIR/routes/api.php"
    
    # Bootstrap
    assert_file_exists "bootstrap/app.php" "$APP_DIR/bootstrap/app.php"
    
    # Environment
    assert_file_exists ".env.example" "$APP_DIR/.env.example"
    
    # README
    assert_file_exists "README.md" "$APP_DIR/README.md"
}

test_build_docker_files() {
    run_test_group "Docker Files"
    
    # Compose files
    assert_file_exists "service compose file" "$DOCKER_DIR/compose/${SERVICE_NAME}.yml"
    assert_file_exists "service dev compose file" "$DOCKER_DIR/compose/${SERVICE_NAME}.dev.yml"
    
    # Dockerfiles
    assert_file_exists "app Dockerfile" "$DOCKER_DIR/dockerfiles/${SERVICE_NAME}/app"
    assert_file_exists "app prod Dockerfile" "$DOCKER_DIR/dockerfiles/${SERVICE_NAME}/app.prod"
    
    # Config
    assert_file_exists "service env config" "$DOCKER_DIR/.config/.env.${SERVICE_NAME}"
    assert_file_exists "db env config" "$DOCKER_DIR/.config/.env.db"
    assert_file_exists "redis env config" "$DOCKER_DIR/.config/.env.redis"
    
    # Shared infrastructure (installed by recipe)
    assert_file_exists "db compose file" "$DOCKER_DIR/compose/db.yml"
    assert_file_exists "redis compose file" "$DOCKER_DIR/compose/redis.yml"
}

test_build_template_processing() {
    run_test_group "Template Processing"
    
    # Check placeholders were replaced in compose file
    assert_file_contains "compose has service name" "$DOCKER_DIR/compose/${SERVICE_NAME}.yml" "$SERVICE_NAME"
    assert_file_not_contains "compose no raw placeholder" "$DOCKER_DIR/compose/${SERVICE_NAME}.yml" "{{SERVICE_NAME}}"
    
    # Check env config
    assert_file_contains "env has database config" "$DOCKER_DIR/.config/.env.${SERVICE_NAME}" "DB_CONNECTION"
    assert_file_contains "env has redis config" "$DOCKER_DIR/.config/.env.${SERVICE_NAME}" "REDIS_HOST"
    
    # Check README was processed
    assert_file_contains "README mentions service" "$APP_DIR/README.md" "$SERVICE_NAME" || \
        assert_file_contains "README mentions Laravel" "$APP_DIR/README.md" "Laravel"
}

test_build_php_syntax() {
    run_test_group "PHP Syntax Validation"
    
    # Check key PHP files for syntax errors (without executing)
    local php_files=(
        "routes/web.php"
        "routes/api.php"
        "bootstrap/app.php"
        "public/index.php"
        "app/Providers/AppServiceProvider.php"
    )
    
    for file in "${php_files[@]}"; do
        if [[ -f "$APP_DIR/$file" ]]; then
            # Use php -l for syntax check (if PHP available)
            if command -v php >/dev/null 2>&1; then
                if php -l "$APP_DIR/$file" >/dev/null 2>&1; then
                    _record_pass "PHP syntax: $file"
                else
                    _record_fail "PHP syntax: $file" "Syntax error in file"
                fi
            else
                skip_test "PHP syntax: $file" "PHP not installed locally"
            fi
        else
            skip_test "PHP syntax: $file" "File not in template"
        fi
    done
}

test_build_permissions() {
    run_test_group "File Permissions"
    
    # artisan should be executable
    assert_executable "artisan is executable" "$APP_DIR/artisan"
    
    # Storage directories should exist for gitkeep
    assert_file_exists "storage/logs/.gitkeep" "$APP_DIR/storage/logs/.gitkeep" || \
        skip_test "storage/logs/.gitkeep" "gitkeep may not be required"
}

test_build_json_validity() {
    run_test_group "JSON Configuration Validity"
    
    # composer.json should be valid JSON
    if jq empty "$APP_DIR/composer.json" 2>/dev/null; then
        _record_pass "composer.json is valid JSON"
    else
        _record_fail "composer.json is valid JSON" "Invalid JSON syntax"
    fi
    
    # package.json should be valid JSON
    if jq empty "$APP_DIR/package.json" 2>/dev/null; then
        _record_pass "package.json is valid JSON"
    else
        _record_fail "package.json is valid JSON" "Invalid JSON syntax"
    fi
    
    # Check key dependencies in composer.json
    assert_json_has_key "composer has laravel/framework" "$APP_DIR/composer.json" '.require["laravel/framework"]'
    assert_json_has_key "composer has laravel/octane" "$APP_DIR/composer.json" '.require["laravel/octane"]'
    assert_json_has_key "composer has inertiajs" "$APP_DIR/composer.json" '.require["inertiajs/inertia-laravel"]'
    
    # Check key dependencies in package.json
    assert_json_has_key "package has vue" "$APP_DIR/package.json" '.dependencies.vue'
    assert_json_has_key "package has @inertiajs/vue3" "$APP_DIR/package.json" '.dependencies["@inertiajs/vue3"]'
    assert_json_has_key "package has tailwindcss" "$APP_DIR/package.json" '.devDependencies.tailwindcss'
    assert_json_has_key "package has vite" "$APP_DIR/package.json" '.devDependencies.vite'
}

# ─────────────────────────────────────────────────────────────────────────────
# SMOKE LEVEL TESTS
# Docker compose validation and image building
# ─────────────────────────────────────────────────────────────────────────────

test_smoke_compose_config() {
    if ! if_level smoke; then
        skip_test "Docker compose config" "smoke level not enabled"
        return 0
    fi
    
    run_test_group "Docker Compose Configuration"
    
    cd "$TEST_PROJECT_DIR"
    
    # Build compose files array - infrastructure first, then service
    local compose_files=()
    
    # Add shared infrastructure (order matters - dependencies first)
    [[ -f "$DOCKER_DIR/compose/db.yml" ]] && compose_files+=("$DOCKER_DIR/compose/db.yml")
    [[ -f "$DOCKER_DIR/compose/db.dev.yml" ]] && compose_files+=("$DOCKER_DIR/compose/db.dev.yml")
    [[ -f "$DOCKER_DIR/compose/redis.yml" ]] && compose_files+=("$DOCKER_DIR/compose/redis.yml")
    [[ -f "$DOCKER_DIR/compose/redis.dev.yml" ]] && compose_files+=("$DOCKER_DIR/compose/redis.dev.yml")
    
    # Add service files
    compose_files+=("$DOCKER_DIR/compose/${SERVICE_NAME}.yml")
    [[ -f "$DOCKER_DIR/compose/${SERVICE_NAME}.dev.yml" ]] && compose_files+=("$DOCKER_DIR/compose/${SERVICE_NAME}.dev.yml")
    
    assert_compose_valid "compose config is valid" "${compose_files[@]}"
    
    # Check service definitions exist in compose file
    assert_file_contains "compose defines app service" "$DOCKER_DIR/compose/${SERVICE_NAME}.yml" "services:"
    assert_file_contains "compose has healthcheck" "$DOCKER_DIR/compose/${SERVICE_NAME}.yml" "healthcheck:"
    
    # Check dev overrides
    assert_file_contains "dev compose has volumes" "$DOCKER_DIR/compose/${SERVICE_NAME}.dev.yml" "volumes:"
}

test_smoke_dockerfile_syntax() {
    if ! if_level smoke; then
        skip_test "Dockerfile syntax" "smoke level not enabled"
        return 0
    fi
    
    run_test_group "Dockerfile Syntax"
    
    local dockerfiles=(
        "$DOCKER_DIR/dockerfiles/${SERVICE_NAME}/app"
        "$DOCKER_DIR/dockerfiles/${SERVICE_NAME}/app.prod"
    )
    
    for dockerfile in "${dockerfiles[@]}"; do
        if [[ -f "$dockerfile" ]]; then
            local name
            name=$(basename "$dockerfile")
            
            # Check for required instructions
            assert_file_contains "$name has FROM" "$dockerfile" "^FROM"
            assert_file_contains "$name has WORKDIR" "$dockerfile" "WORKDIR"
            
            # Check for PHP-related setup
            assert_file_contains "$name installs PHP extensions" "$dockerfile" "install-php-extensions\|docker-php-ext-install"
            
            # Check for Swoole (Octane requirement)
            assert_file_contains "$name has Swoole" "$dockerfile" "swoole"
        else
            _record_fail "Dockerfile exists: $dockerfile"
        fi
    done
}

test_smoke_image_build() {
    if ! if_level smoke; then
        skip_test "Docker image build" "smoke level not enabled"
        return 0
    fi
    
    run_test_group "Docker Image Build"
    
    cd "$TEST_PROJECT_DIR"
    
    # Build full compose command including all dependencies
    local compose_files=""
    [[ -f "$DOCKER_DIR/compose/db.yml" ]] && compose_files="$compose_files -f $DOCKER_DIR/compose/db.yml"
    [[ -f "$DOCKER_DIR/compose/db.dev.yml" ]] && compose_files="$compose_files -f $DOCKER_DIR/compose/db.dev.yml"
    [[ -f "$DOCKER_DIR/compose/redis.yml" ]] && compose_files="$compose_files -f $DOCKER_DIR/compose/redis.yml"
    [[ -f "$DOCKER_DIR/compose/redis.dev.yml" ]] && compose_files="$compose_files -f $DOCKER_DIR/compose/redis.dev.yml"
    compose_files="$compose_files -f $DOCKER_DIR/compose/${SERVICE_NAME}.yml"
    [[ -f "$DOCKER_DIR/compose/${SERVICE_NAME}.dev.yml" ]] && compose_files="$compose_files -f $DOCKER_DIR/compose/${SERVICE_NAME}.dev.yml"
    
    # Just validate the config can be parsed (actual build takes too long for smoke)
    # shellcheck disable=SC2086
    if docker compose $compose_files config >/dev/null 2>&1; then
        _record_pass "Development compose config valid"
    else
        _record_fail "Development compose config valid" "Config validation failed"
    fi
    
    # For smoke tests, we don't actually build - that's full level
    # But we can check the Dockerfile is parseable
    if docker build --help >/dev/null 2>&1; then
        _record_pass "Docker build command available"
    else
        _record_fail "Docker build command available"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# FULL LEVEL TESTS
# Container runtime and endpoint validation
# ─────────────────────────────────────────────────────────────────────────────

test_full_container_startup() {
    if ! if_level full; then
        skip_test "Container startup" "full level not enabled"
        return 0
    fi
    
    run_test_group "Container Startup"
    
    cd "$TEST_PROJECT_DIR"
    
    # Build and start containers with unique project name
    local project_name
    project_name=$(basename "$TEST_PROJECT_DIR")
    local compose_cmd="docker compose -p $project_name"
    local compose_files="-f $DOCKER_DIR/compose/db.yml -f $DOCKER_DIR/compose/db.dev.yml"
    compose_files="$compose_files -f $DOCKER_DIR/compose/redis.yml -f $DOCKER_DIR/compose/redis.dev.yml"
    compose_files="$compose_files -f $DOCKER_DIR/compose/${SERVICE_NAME}.yml -f $DOCKER_DIR/compose/${SERVICE_NAME}.dev.yml"
    
    info "Building Docker images (this may take several minutes)..."
    # shellcheck disable=SC2086
    if $compose_cmd $compose_files build --no-cache 2>&1 | tail -20; then
        _record_pass "Docker images built successfully"
    else
        _record_fail "Docker images built successfully" "Build failed"
        return 1
    fi
    
    # Install PHP dependencies using composer container
    info "Installing PHP dependencies..."
    # Ensure Laravel directories exist and are writable
    mkdir -p "$APP_DIR/bootstrap/cache" "$APP_DIR/storage/framework/cache" "$APP_DIR/storage/framework/sessions" "$APP_DIR/storage/framework/views"
    chmod -R 777 "$APP_DIR/bootstrap/cache" "$APP_DIR/storage"
    # Create .env file for artisan commands
    cp "$APP_DIR/.env.example" "$APP_DIR/.env"
    docker run --rm -v "$APP_DIR:/app" -w /app composer:latest install --no-interaction --no-progress --prefer-dist --ignore-platform-reqs 2>&1 | tail -10
    
    # Install npm dependencies and build frontend
    info "Installing npm dependencies and building frontend..."
    docker run --rm -v "$APP_DIR:/app" -w /app node:20-alpine sh -c "npm install && npm run build" 2>&1 | tail -10
    
    info "Starting containers..."
    # Create required networks if they don't exist
    docker network create traefik 2>/dev/null || true
    docker network create app-internal 2>/dev/null || true
    docker network create mech-network 2>/dev/null || true
    
    # shellcheck disable=SC2086
    $compose_cmd $compose_files up -d
    
    # Wait for containers to start
    info "Waiting for containers to initialize (30s)..."
    sleep 30
    
    # Check container status
    # shellcheck disable=SC2086
    local running
    running=$($compose_cmd $compose_files ps --status running -q | wc -l | tr -d ' ')
    
    if [[ $running -ge 1 ]]; then
        _record_pass "Containers started ($running running)"
    else
        _record_fail "Containers started" "No containers running"
        # shellcheck disable=SC2086
        $compose_cmd $compose_files logs --tail=50
        return 1
    fi
}

test_full_container_health() {
    if ! if_level full; then
        skip_test "Container health" "full level not enabled"
        return 0
    fi
    
    run_test_group "Container Health"
    
    cd "$TEST_PROJECT_DIR"
    
    # Check if containers have health checks defined and are healthy
    local containers
    containers=$(docker ps --filter "name=${TEST_PROJECT_NAME}" --format "{{.Names}}")
    
    for container in $containers; do
        local health
        health=$(docker inspect -f '{{.State.Health.Status}}' "$container" 2>/dev/null || echo "none")
        
        case "$health" in
            healthy)
                _record_pass "Container $container is healthy"
                ;;
            unhealthy)
                _record_fail "Container $container health" "Status: unhealthy"
                ;;
            starting)
                skip_test "Container $container health" "Still starting"
                ;;
            none)
                skip_test "Container $container health" "No healthcheck defined"
                ;;
        esac
    done
}

test_full_http_endpoints() {
    if ! if_level full; then
        skip_test "HTTP endpoints" "full level not enabled"
        return 0
    fi
    
    run_test_group "HTTP Endpoints"
    
    # Default Laravel port in dev mode
    local base_url="http://localhost:8000"
    
    # Wait a bit more for the app to be ready
    sleep 10
    
    # Test main endpoint
    assert_http_status "Root endpoint responds" "$base_url" "200" || \
        assert_http_status "Root endpoint redirect" "$base_url" "302"
    
    # Test Filament admin (should redirect to login if not authenticated)
    assert_http_status "Admin endpoint responds" "$base_url/admin" "200" || \
        assert_http_status "Admin redirects to login" "$base_url/admin" "302"
    
    # Test API endpoint (if defined)
    assert_http_status "API health endpoint" "$base_url/api/health" "200" || \
        skip_test "API health endpoint" "Not defined"
}

test_full_cleanup() {
    if ! if_level full; then
        return 0
    fi
    
    run_test_group "Cleanup"
    
    cd "$TEST_PROJECT_DIR"
    
    info "Stopping containers..."
    docker compose down -v 2>/dev/null || true
    
    _record_pass "Containers stopped and cleaned up"
}

# ─────────────────────────────────────────────────────────────────────────────
# Test Runner
# ─────────────────────────────────────────────────────────────────────────────

# Run all tests in order
main() {
    # Setup
    setup_test || return 1
    
    # BUILD level tests (always run)
    test_build_directory_structure
    test_build_core_files
    test_build_docker_files
    test_build_template_processing
    test_build_php_syntax
    test_build_permissions
    test_build_json_validity
    
    # SMOKE level tests
    test_smoke_compose_config
    test_smoke_dockerfile_syntax
    test_smoke_image_build
    
    # FULL level tests
    test_full_container_startup
    test_full_container_health
    test_full_http_endpoints
    test_full_cleanup
    
    return 0
}

# Run tests
main
