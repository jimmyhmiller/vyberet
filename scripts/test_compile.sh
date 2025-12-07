#!/bin/bash
# Test script for Pyret->Scheme compilation
# Usage: ./scripts/test_compile.sh <pyret-file> [scheme-interpreter]

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <pyret-file> [interpreter]"
    echo "  interpreter: chicken, gsi, or ribbit (default: chicken)"
    exit 1
fi

PYRET_FILE="$1"
INTERPRETER="${2:-chicken}"
BASE_NAME=$(basename "$PYRET_FILE" .arr)
SCM_FILE="test_output/${BASE_NAME}.scm"
RUNTIME="runtime/runtime.scm"

# Ensure output directory exists
mkdir -p test_output

echo "==== Compiling Pyret to Scheme ===="
cargo run --quiet --bin compile "$PYRET_FILE" "$SCM_FILE"

echo ""
echo "==== Generated Scheme Code ===="
cat "$SCM_FILE"

echo ""
echo "==== Running with $INTERPRETER ===="

# Create a wrapper that loads runtime and runs the code
WRAPPER_FILE="test_output/${BASE_NAME}_wrapper.scm"

# Just concatenate runtime and compiled code
cat "$RUNTIME" > "$WRAPPER_FILE"
echo "" >> "$WRAPPER_FILE"
echo "; ==== Compiled Pyret Code ====" >> "$WRAPPER_FILE"
cat "$SCM_FILE" >> "$WRAPPER_FILE"

case "$INTERPRETER" in
    chicken|csi)
        # Chicken Scheme
        csi -q -b "$WRAPPER_FILE"
        ;;
    gsi)
        # Gambit Scheme
        gsi -:d- "$WRAPPER_FILE"
        ;;
    ribbit)
        # Ribbit Scheme - compile to C and run
        RIBBIT_DIR="/Users/jimmyhmiller/Documents/Code/open-source/ribbit"
        RIBBIT_RSC="$RIBBIT_DIR/src/rsc.exe"
        RIBBIT_OUTPUT="test_output/${BASE_NAME}.c"

        # Check if rsc.exe exists, build if not
        if [ ! -f "$RIBBIT_RSC" ]; then
            echo "Building Ribbit compiler..."
            (cd "$RIBBIT_DIR/src" && make rsc.exe)
        fi

        # Compile Scheme to C using Ribbit
        "$RIBBIT_RSC" -t c -l min "$WRAPPER_FILE" -o "$RIBBIT_OUTPUT"

        # Compile C to executable
        gcc "$RIBBIT_OUTPUT" -o "test_output/${BASE_NAME}.out"

        # Run the executable
        "test_output/${BASE_NAME}.out"
        ;;
    *)
        echo "Unknown interpreter: $INTERPRETER"
        echo "Supported: chicken, gsi, ribbit"
        exit 1
        ;;
esac
