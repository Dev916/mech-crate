#!/bin/bash
#
# test.sh - MechCrate Recipe Testbed Integration
# Run recipe validation tests
#

# ─────────────────────────────────────────────────────────────────────────────
# Test Command Implementation
# ─────────────────────────────────────────────────────────────────────────────

test_cmd() {
    local level="smoke"
    local keep=false
    local verbose=false
    local recipes=()
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -l|--level)
                level="$2"
                shift 2
                ;;
            -k|--keep)
                keep=true
                shift
                ;;
            -v|--verbose)
                verbose=true
                shift
                ;;
            -h|--help)
                show_test_help
                return 0
                ;;
            -*)
                error "Unknown option: $1"
                show_test_help
                return 1
                ;;
            *)
                recipes+=("$1")
                shift
                ;;
        esac
    done
    
    # Validate test level
    case "$level" in
        build|smoke|full) ;;
        *)
            error "Invalid test level: $level"
            echo "Valid levels: build, smoke, full"
            return 1
            ;;
    esac
    
    local testbed="$MECH_CRATE_ROOT/tests/testbed/testbed.sh"
    
    # Check testbed exists
    if [[ ! -f "$testbed" ]]; then
        error "Testbed not found at: $testbed"
        echo "Run from the mech-crate repository root"
        return 1
    fi
    
    # Make testbed executable
    chmod +x "$testbed"
    
    # Build testbed arguments
    local args=("--level" "$level")
    [[ "$keep" == "true" ]] && args+=("--keep")
    [[ "$verbose" == "true" ]] && args+=("--verbose")
    [[ ${#recipes[@]} -gt 0 ]] && args+=("${recipes[@]}")
    
    # Run testbed
    "$testbed" "${args[@]}"
}

show_test_help() {
    cat << 'EOF'
Usage: mx test [options] [recipe...]

Run recipe validation tests to ensure recipes work correctly.

Options:
    -l, --level LEVEL    Test level (default: smoke)
                         - build: File structure and config validation only
                         - smoke: Build + Docker compose/image validation
                         - full:  Complete integration with running containers

    -k, --keep           Keep test projects after completion (for debugging)
    -v, --verbose        Enable verbose output
    -h, --help           Show this help message

Arguments:
    recipe...            Specific recipes to test (default: all available)

Examples:
    mx test                          # Run smoke tests on all recipes
    mx test laravel                  # Test only Laravel recipe
    mx test --level full laravel     # Run full integration tests
    mx test --level build --keep     # Build tests, keep project for inspection
    mx test laravel nuxt rust-api    # Test multiple recipes

Test Levels:
    build   Quick validation of recipe installation, file structure,
            and configuration. No Docker required.

    smoke   Includes build tests plus Docker compose validation.
            Ensures the stack configuration is valid.

    full    Complete integration testing with running containers,
            health checks, and HTTP endpoint validation.
            Takes longest but is most thorough.

EOF
}
