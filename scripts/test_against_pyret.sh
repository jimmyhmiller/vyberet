#!/bin/bash

# Test compiled Scheme against official Pyret implementation
# Usage: ./scripts/test_against_pyret.sh <pyret-file> [options]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PYRET_LANG_DIR="/Users/jimmyhmiller/Documents/Code/open-source/pyret-lang"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
INTERPRETER="chicken"
SHOW_SCHEME=0
SHOW_DIFF=1
VERBOSE=0

usage() {
    cat << EOF
Usage: $0 <pyret-file> [options]

Test your compiled Scheme against the official Pyret implementation.

Options:
    --interpreter <name>    Scheme interpreter to use (chicken/gsi/ribbit, default: chicken)
    --show-scheme          Show generated Scheme code
    --no-diff              Don't show diff when outputs differ
    --verbose              Show detailed output
    -h, --help             Show this help message

Examples:
    $0 tests/pyret-files/factorial.arr
    $0 tests/pyret-files/power.arr --show-scheme
    $0 tests/pyret-files/fibonacci.arr --interpreter gsi

Requirements:
    - pyret CLI installed (npm install -g pyret-npm or see pyret.org/getting-started)
    - Scheme interpreter installed (csi, gsi, or ribbit)
    - cargo and rust toolchain

Note: Checks are disabled by default. Use --check flag to enable check blocks.

EOF
    exit 1
}

# Parse arguments
if [ $# -lt 1 ]; then
    usage
fi

PYRET_FILE="$1"
shift

while [ $# -gt 0 ]; do
    case "$1" in
        --interpreter)
            INTERPRETER="$2"
            shift 2
            ;;
        --show-scheme)
            SHOW_SCHEME=1
            shift
            ;;
        --no-diff)
            SHOW_DIFF=0
            shift
            ;;
        --verbose)
            VERBOSE=1
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

# Check if file exists
if [ ! -f "$PYRET_FILE" ]; then
    echo -e "${RED}Error: File not found: $PYRET_FILE${NC}"
    exit 1
fi

# Create temp directory for outputs
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

PYRET_OUTPUT="$TEMP_DIR/pyret_output.txt"
SCHEME_OUTPUT="$TEMP_DIR/scheme_output.txt"
SCHEME_FILE="$TEMP_DIR/compiled.scm"

echo -e "${BLUE}Testing: $PYRET_FILE${NC}"
echo ""

# Step 1: Check if pyret is available
if ! command -v pyret &> /dev/null; then
    echo -e "${YELLOW}Warning: pyret not found${NC}"
    echo "Install with: npm install -g pyret-npm"
    echo "Or follow instructions at: https://www.pyret.org/getting-started/"
    echo ""
    echo "Continuing with only Scheme compilation test..."
    PYRET_AVAILABLE=0
else
    PYRET_AVAILABLE=1
fi

# Step 2: Run official Pyret if available
if [ $PYRET_AVAILABLE -eq 1 ]; then
    echo -e "${BLUE}[1/3] Running official Pyret...${NC}"

    # Run Pyret and capture output
    if pyret "$PYRET_FILE" > "$PYRET_OUTPUT" 2>&1; then
        if [ $VERBOSE -eq 1 ]; then
            echo -e "${GREEN}Pyret execution succeeded${NC}"
            cat "$PYRET_OUTPUT"
        else
            # Extract just the final value (after all the Pyret runtime messages)
            PYRET_RESULT=$(tail -1 "$PYRET_OUTPUT")
            echo -e "${GREEN}Pyret output: $PYRET_RESULT${NC}"
        fi
    else
        echo -e "${RED}Pyret execution failed:${NC}"
        cat "$PYRET_OUTPUT"
        exit 1
    fi
    echo ""
else
    echo -e "${YELLOW}[1/3] Skipping Pyret execution (not installed)${NC}"
    echo ""
fi

# Step 3: Compile to Scheme
echo -e "${BLUE}[2/3] Compiling to Scheme...${NC}"

COMPILE_CMD="cargo run --quiet --bin compile $PYRET_FILE --output $SCHEME_FILE --check"
if [ $VERBOSE -eq 1 ]; then
    echo "Running: $COMPILE_CMD"
fi

if $COMPILE_CMD 2>&1 | grep -v "^Compiled successfully"; then
    echo -e "${RED}Compilation failed${NC}"
    exit 1
fi

if [ $SHOW_SCHEME -eq 1 ]; then
    echo ""
    echo -e "${YELLOW}Generated Scheme code:${NC}"
    cat "$SCHEME_FILE"
    echo ""
fi

echo -e "${GREEN}Compilation succeeded${NC}"
echo ""

# Step 4: Run compiled Scheme
echo -e "${BLUE}[3/3] Running compiled Scheme ($INTERPRETER)...${NC}"

RUN_CMD="cargo run --quiet --bin compile $PYRET_FILE --run --interpreter $INTERPRETER --check"
if [ $VERBOSE -eq 1 ]; then
    echo "Running: $RUN_CMD"
fi

if $RUN_CMD > "$SCHEME_OUTPUT" 2>&1; then
    SCHEME_RESULT=$(cat "$SCHEME_OUTPUT")
    echo -e "${GREEN}Scheme output: $SCHEME_RESULT${NC}"
else
    echo -e "${RED}Scheme execution failed:${NC}"
    cat "$SCHEME_OUTPUT"
    exit 1
fi
echo ""

# Step 5: Compare outputs
if [ $PYRET_AVAILABLE -eq 1 ]; then
    echo -e "${BLUE}Comparing outputs...${NC}"

    # Compare raw outputs byte-for-byte
    if diff -q "$PYRET_OUTPUT" "$SCHEME_OUTPUT" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS: Outputs are identical!${NC}"
        exit 0
    else
        echo -e "${RED}✗ FAIL: Outputs differ!${NC}"
        echo ""
        if [ $SHOW_DIFF -eq 1 ]; then
            echo -e "${YELLOW}Diff:${NC}"
            diff -u "$PYRET_OUTPUT" "$SCHEME_OUTPUT" || true
        fi
        exit 1
    fi
else
    echo -e "${GREEN}✓ Scheme execution succeeded${NC}"
    echo -e "${YELLOW}(Could not compare with Pyret - not installed)${NC}"
    exit 0
fi
