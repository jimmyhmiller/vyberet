#!/bin/bash

# Test all examples against Pyret
# Usage: ./scripts/test_all_examples.sh [options]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Default options
INTERPRETER="chicken"
VERBOSE=0
STOP_ON_FAIL=0

usage() {
    cat << EOF
Usage: $0 [options]

Test all example files against the official Pyret implementation.

Options:
    --interpreter <name>    Scheme interpreter to use (chicken/gsi/ribbit, default: chicken)
    --verbose              Show detailed output for each test
    --stop-on-fail         Stop at first failure
    -h, --help             Show this help message

Examples:
    $0
    $0 --interpreter gsi
    $0 --verbose --stop-on-fail

EOF
    exit 1
}

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --interpreter)
            INTERPRETER="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=1
            shift
            ;;
        --stop-on-fail)
            STOP_ON_FAIL=1
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# Find all .arr files in tests/pyret-files/
EXAMPLES=$(find "$PROJECT_DIR/examples" -name "*.arr" -type f | sort)

if [ -z "$EXAMPLES" ]; then
    echo -e "${RED}No .arr files found in tests/pyret-files/${NC}"
    exit 1
fi

# Count examples
TOTAL=$(echo "$EXAMPLES" | wc -l | tr -d ' ')
PASSED=0
FAILED=0
SKIPPED=0

echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║         Testing All Examples Against Pyret                   ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Interpreter: ${BLUE}$INTERPRETER${NC}"
echo -e "Total examples: ${BLUE}$TOTAL${NC}"
echo ""

# Track failed tests
FAILED_TESTS=()

# Test each example
for example in $EXAMPLES; do
    BASENAME=$(basename "$example")

    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Testing: $BASENAME${NC}"
    echo ""

    # Build test command
    TEST_CMD="$SCRIPT_DIR/test_against_pyret.sh $example --interpreter $INTERPRETER --no-diff"
    if [ $VERBOSE -eq 1 ]; then
        TEST_CMD="$TEST_CMD --verbose"
    fi

    # Run test
    if $TEST_CMD 2>&1; then
        ((PASSED++))
        echo ""
    else
        ((FAILED++))
        FAILED_TESTS+=("$BASENAME")
        echo ""

        if [ $STOP_ON_FAIL -eq 1 ]; then
            echo -e "${RED}Stopping due to failure (--stop-on-fail)${NC}"
            break
        fi
    fi
done

# Print summary
echo ""
echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                      Test Summary                             ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Total:   ${BLUE}$TOTAL${NC}"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"

if [ $FAILED -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "  ${RED}✗${NC} $test"
    done
    echo ""
    exit 1
else
    echo ""
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    exit 0
fi
