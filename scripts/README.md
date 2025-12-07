# Scripts

## `compare_parsers.sh`
Compare our parser output with official Pyret parser.

```bash
./scripts/compare_parsers.sh "fun f(x): x + 1 end"
```

## `test_against_pyret.sh`
Test a Pyret file against the official implementation.

```bash
./scripts/test_against_pyret.sh tests/pyret-files/factorial.arr
```

## `test_all_examples.sh`
Test all example files.

```bash
./scripts/test_all_examples.sh
```
