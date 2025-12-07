# Pyret Parser & Compiler

A hand-written recursive descent parser and compiler for Pyret in Rust, targeting Scheme.

## Quick Start

```bash
# Run all tests
cargo test

# Compile & run example
cargo run --bin compile tests/pyret-files/factorial.arr --run

# Show generated Scheme
cargo run --bin compile tests/pyret-files/factorial.arr --run --show-scheme

# Use different backends
cargo run --bin compile tests/pyret-files/factorial.arr --run --interpreter gsi     # Gambit
cargo run --bin compile tests/pyret-files/factorial.arr --run --interpreter ribbit  # Native
```

## Key Files

```
src/
├── parser.rs            - Parser implementation
├── ast.rs               - AST node types
├── tokenizer.rs         - Tokenizer
├── codegen.rs           - Pyret -> Scheme compiler
├── module_compiler.rs   - Multi-file compilation
└── bin/compile.rs       - Compiler CLI

runtime/
└── runtime.scm          - R4RS Scheme runtime

tests/
├── parser_tests.rs         - Parser unit tests
├── comparison_tests.rs     - Parser integration tests
└── check_output_tests.rs   - Compiler output tests
```

## Key Concepts

**Whitespace Sensitivity:**
- `f(x)` = function call
- `f (x)` = two expressions

**No Operator Precedence:**
- `2 + 3 * 4` = `(2 + 3) * 4` = `20` (NOT 14)
- All binary operators are left-associative

**Pyret Lists:**
- `[list: 1, 2, 3]` (NOT `[1, 2, 3]`)

## Reference

- **Pyret Grammar:** `/Users/jimmyhmiller/Documents/Code/open-source/pyret-lang/src/js/base/pyret-grammar.bnf`
