#!/usr/bin/env bash
#
# MechCrate Testbed - Main Test Runner
# Scalable test framework for validating recipe installations
#
# Usage:
#   ./testbed.sh [options] [recipe...]
#
# Options:
#   -l, --level LEVEL    Test level: build, smoke, full (default: smoke)
#   -k, --keep           Keep test projects after completion
#   -v, --verbose        Enable verbose output
#   -h, --help           Show this help message
#
# Version: 1.0.0
# Author: MechCrate
#

set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Constants & Paths
# ─────────────────────────────────────────────────────────────────────────────

readonly TESTBED_VERSION="1.0.0"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
readonly TESTBED_ROOT="$SCRIPT_DIR"
readonly MECH_CRATE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
readonly RECIPES_DIR="$MECH_CRATE_ROOT/templates/recipes"
readonly TEST_RECIPES_DIR="$TESTBED_ROOT/recipes"
readonly TMP_DIR="${TESTBED_TMP:-${TMPDIR:-/tmp}/mechcrate-testbed}"

# Source libraries
source "$TESTBED_ROOT/lib/common.sh"
source "$TESTBED_ROOT/lib/assertions.sh"

# ─────────────────────────────────────────────────────────────────────────────
# Default Configuration
# ─────────────────────────────────────────────────────────────────────────────

TEST_LEVEL="${TEST_LEVEL:-smoke}"
KEEP_PROJECTS="${KEEP_PROJECTS:-false}"
VERBOSE="${VERBOSE:-false}"
RECIPES_TO_TEST=()

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────

show_help() {
    cat << EOF
${BOLD}MechCrate Testbed - Recipe Validation Framework${NC}
Version: $TESTBED_VERSION

${BOLD}USAGE:${NC}
    ./testbed.sh [options] [recipe...]

${BOLD}OPTIONS:${NC}
    -l, --level LEVEL    Test level (default: smoke)
                         - build: Only test file generation and structure
                         - smoke: Build + basic Docker validation
                         - full:  Complete integration tests with running services

    -k, --keep           Keep test projects after completion (for debugging)
    -v, --verbose        Enable verbose output
    -h, --help           Show this help message

${BOLD}ARGUMENTS:${NC}
    recipe...            Specific recipes to test (default: all available)

${BOLD}EXAMPLES:${NC}
    # Run smoke tests on all recipes
    ./testbed.sh

    # Run full tests on Laravel recipe only
    ./testbed.sh --level full laravel

    # Run build tests, keep projects for inspection
    ./testbed.sh --level build --keep nuxt astro

    # Test multiple specific recipes
    ./testbed.sh laravel rust-api nuxt

${BOLD}TEST LEVELS:${NC}
    ${GREEN}build${NC}   Tests recipe installation, file structure, and configuration
            validation. No Docker required. Fastest execution.

    ${YELLOW}smoke${NC}   Includes build tests plus Docker compose validation and
            image building. Verifies the stack can be built.

    ${RED}full${NC}    Complete integration testing with running containers,
            health checks, and HTTP endpoint validation. Slowest but
            most thorough.

${BOLD}ENVIRONMENT VARIABLES:${NC}
    TEST_LEVEL          Default test level (build|smoke|full)
    TESTBED_TMP         Override temporary directory location
    KEEP_PROJECTS       Set to 'true' to keep test projects

EOF
}

# ─────────────────────────────────────────────────────────────────────────────
# Argument Parsing
# ─────────────────────────────────────────────────────────────────────────────

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -l|--level)
                TEST_LEVEL="$2"
                shift 2
                ;;
            -k|--keep)
                KEEP_PROJECTS=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            -*)
                error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
            *)
                RECIPES_TO_TEST+=("$1")
                shift
                ;;
        esac
    done
    
    # Validate test level
    case "$TEST_LEVEL" in
        build|smoke|full) ;;
        *)
            error "Invalid test level: $TEST_LEVEL"
            echo "Valid levels: build, smoke, full"
            exit 1
            ;;
    esac
}

# ─────────────────────────────────────────────────────────────────────────────
# Recipe Discovery
# ─────────────────────────────────────────────────────────────────────────────

discover_recipes() {
    local -a discovered=()
    
    for recipe_dir in "$RECIPES_DIR"/*/; do
        if [[ -f "${recipe_dir}recipe.json" ]]; then
            local recipe_name
            recipe_name=$(basename "$recipe_dir")
            discovered+=("$recipe_name")
        fi
    done
    
    # Print one recipe per line (friendly for read loops; bash 3.x compatible)
    printf "%s\n" "${discovered[@]}"
}

validate_recipe_exists() {
    local recipe="$1"
    
    if [[ ! -d "$RECIPES_DIR/$recipe" ]]; then
        error "Recipe not found: $recipe"
        local available
        available=$(discover_recipes | tr '\n' ' ' | xargs)
        echo "Available recipes: ${available// /, }"
        return 1
    fi
    
    if [[ ! -f "$RECIPES_DIR/$recipe/recipe.json" ]]; then
        error "Recipe missing recipe.json: $recipe"
        return 1
    fi
    
    return 0
}

# ─────────────────────────────────────────────────────────────────────────────
# Test Execution
# ─────────────────────────────────────────────────────────────────────────────

run_recipe_test() {
    local recipe="$1"
    local test_file="$TEST_RECIPES_DIR/${recipe}.test.sh"
    
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}Testing Recipe: ${CYAN}$recipe${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    
    # Check if test file exists
    if [[ ! -f "$test_file" ]]; then
        warn "No test file found: $test_file"
        echo -e "  ${YELLOW}○${NC} Recipe '$recipe' has no test implementation (skipped)"
        ((TOTAL_SKIPPED++))
        return 0
    fi
    
    # Create test project directory
    local project_name="test-${recipe}-$(date +%s)"
    local project_dir="$TMP_DIR/$project_name"
    
    info "Creating test project: $project_dir"
    mkdir -p "$project_dir"
    
    # Export test context
    export TEST_RECIPE="$recipe"
    export TEST_PROJECT_DIR="$project_dir"
    export TEST_PROJECT_NAME="$project_name"
    export TEST_LEVEL
    export MECH_CRATE_ROOT
    export RECIPES_DIR
    export VERBOSE
    
    # Reset test counters for this recipe
    TEST_PASSED=0
    TEST_FAILED=0
    TEST_SKIPPED=0
    
    # Run the test file
    local test_result=0
    if ! source "$test_file"; then
        test_result=1
    fi
    
    # Aggregate results
    ((TOTAL_PASSED += TEST_PASSED))
    ((TOTAL_FAILED += TEST_FAILED))
    ((TOTAL_SKIPPED += TEST_SKIPPED))
    
    # Print recipe summary
    echo ""
    echo -e "  ${BOLD}Recipe Summary:${NC} ${GREEN}$TEST_PASSED passed${NC}, ${RED}$TEST_FAILED failed${NC}, ${YELLOW}$TEST_SKIPPED skipped${NC}"
    
    # Cleanup unless --keep flag
    if [[ "$KEEP_PROJECTS" != "true" ]]; then
        cleanup_test_project "$project_dir"
    else
        info "Keeping test project: $project_dir"
    fi
    
    return $test_result
}

cleanup_test_project() {
    local project_dir="$1"
    
    # Stop any running containers
    if [[ -f "$project_dir/docker/compose/app.yml" ]]; then
        debug "Stopping containers for $(basename "$project_dir")"
        (cd "$project_dir" && docker compose -f docker/compose/*.yml down -v 2>/dev/null || true)
    fi
    
    # Remove project directory
    if [[ -d "$project_dir" ]]; then
        debug "Removing test project: $project_dir"
        rm -rf "$project_dir"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Results Summary
# ─────────────────────────────────────────────────────────────────────────────

print_summary() {
    local total=$((TOTAL_PASSED + TOTAL_FAILED + TOTAL_SKIPPED))
    
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}                    TESTBED RESULTS SUMMARY${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  ${BOLD}Test Level:${NC}    $TEST_LEVEL"
    echo -e "  ${BOLD}Total Tests:${NC}   $total"
    echo ""
    echo -e "  ${GREEN}✓ Passed:${NC}      $TOTAL_PASSED"
    echo -e "  ${RED}✗ Failed:${NC}      $TOTAL_FAILED"
    echo -e "  ${YELLOW}○ Skipped:${NC}     $TOTAL_SKIPPED"
    echo ""
    
    if [[ $TOTAL_FAILED -gt 0 ]]; then
        echo -e "  ${RED}${BOLD}TESTBED FAILED${NC}"
        echo ""
        return 1
    else
        echo -e "  ${GREEN}${BOLD}TESTBED PASSED${NC}"
        echo ""
        return 0
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Main Entry Point
# ─────────────────────────────────────────────────────────────────────────────

main() {
    parse_args "$@"
    
    echo ""
    echo -e "${BOLD}╔═══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║            MechCrate Testbed v${TESTBED_VERSION}                             ║${NC}"
    echo -e "${BOLD}║            Recipe Validation Framework                            ║${NC}"
    echo -e "${BOLD}╚═══════════════════════════════════════════════════════════════════╝${NC}"
    
    # Determine which recipes to test
    if [[ ${#RECIPES_TO_TEST[@]} -eq 0 ]]; then
        # Test all available recipes
        # Bash 3.x compatible (macOS default) replacement for mapfile/readarray
        while IFS= read -r recipe; do
            [[ -z "$recipe" ]] && continue
            RECIPES_TO_TEST+=("$recipe")
        done < <(discover_recipes)
        info "Discovered ${#RECIPES_TO_TEST[@]} recipes to test"
    else
        # Validate specified recipes exist
        for recipe in "${RECIPES_TO_TEST[@]}"; do
            validate_recipe_exists "$recipe" || exit 1
        done
        info "Testing ${#RECIPES_TO_TEST[@]} specified recipes"
    fi
    
    echo -e "  ${DIM}Test Level: $TEST_LEVEL${NC}"
    echo -e "  ${DIM}Recipes: ${RECIPES_TO_TEST[*]}${NC}"
    
    # Initialize global counters
    TOTAL_PASSED=0
    TOTAL_FAILED=0
    TOTAL_SKIPPED=0
    
    # Ensure temp directory exists
    mkdir -p "$TMP_DIR"
    
    # Check prerequisites
    check_prerequisites
    
    # Run tests for each recipe
    local exit_code=0
    for recipe in "${RECIPES_TO_TEST[@]}"; do
        if ! run_recipe_test "$recipe"; then
            exit_code=1
        fi
    done
    
    # Print summary and exit
    print_summary || exit_code=1
    
    exit $exit_code
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
