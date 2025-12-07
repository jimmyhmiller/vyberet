#!/bin/bash
# Test Pyret compiler output with different Scheme backends
# Usage: ./test_with_backend.sh [chicken|gambit|chez|ribbit] [file.arr]

set -e  # Exit on error

BACKEND=${1:-chicken}
FILE=${2:-tests/pyret-files/checks.arr}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Testing Pyret Compiler with $BACKEND backend ===${NC}"
echo "File: $FILE"
echo ""

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

cd "$PROJECT_DIR"

# Compile Pyret to Scheme
echo -e "${YELLOW}Step 1: Compiling Pyret to Scheme...${NC}"
cargo run --bin compile -- "$FILE" -o /tmp/test_backend.scm --check
echo -e "${GREEN}✓ Compiled successfully${NC}"
echo ""

# Combine with runtime
echo -e "${YELLOW}Step 2: Combining with runtime library...${NC}"
cat runtime/runtime.scm /tmp/test_backend.scm > /tmp/test_backend_complete.scm
echo -e "${GREEN}✓ Runtime added${NC}"
echo ""

if [ "$BACKEND" = "chicken" ]; then
    echo -e "${YELLOW}Step 3: Running with Chicken Scheme...${NC}"
    echo "---"
    csi -s /tmp/test_backend_complete.scm
    echo "---"
    echo -e "${GREEN}✓ Execution completed${NC}"

elif [ "$BACKEND" = "gambit" ]; then
    echo -e "${YELLOW}Step 3: Running with Gambit Scheme...${NC}"
    echo "---"
    gsi /tmp/test_backend_complete.scm
    echo "---"
    echo -e "${GREEN}✓ Execution completed${NC}"

elif [ "$BACKEND" = "chez" ]; then
    echo -e "${YELLOW}Step 3: Running with Chez Scheme...${NC}"
    echo "---"
    chez --script /tmp/test_backend_complete.scm
    echo "---"
    echo -e "${GREEN}✓ Execution completed${NC}"

elif [ "$BACKEND" = "ribbit" ]; then
    RIBBIT_DIR="/Users/jimmyhmiller/Documents/Code/open-source/ribbit/src"

    if [ ! -f "$RIBBIT_DIR/rsc" ]; then
        echo -e "${RED}Error: Ribbit compiler not found at $RIBBIT_DIR/rsc${NC}"
        exit 1
    fi

    echo -e "${YELLOW}Step 3: Compiling with Ribbit (C target)...${NC}"
    cd "$RIBBIT_DIR"
    ./rsc -t c -l r4rs /tmp/test_backend_complete.scm -o /tmp/test_backend.c
    echo -e "${GREEN}✓ Ribbit compilation completed${NC}"
    echo ""

    echo -e "${YELLOW}Step 4: Compiling C code with GCC...${NC}"
    gcc /tmp/test_backend.c -o /tmp/test_backend
    echo -e "${GREEN}✓ C compilation completed${NC}"
    echo ""

    echo -e "${YELLOW}Step 5: Running executable...${NC}"
    echo "---"
    /tmp/test_backend
    echo "---"
    echo -e "${GREEN}✓ Execution completed${NC}"

else
    echo -e "${RED}Error: Unknown backend '$BACKEND'${NC}"
    echo "Usage: $0 [chicken|gambit|chez|ribbit] [file.arr]"
    exit 1
fi

echo ""
echo -e "${GREEN}=== Test completed successfully! ===${NC}"
