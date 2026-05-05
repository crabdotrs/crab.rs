# Compiler Benchmarks

This directory contains benchmarks for the Crab compiler performance.

## Running Benchmarks

```bash
cargo bench
```

## Benchmark Categories

### Lexer Performance
Measures tokenization speed for various source file sizes.

### Parser Performance
Measures AST generation speed.

### Codegen Performance
Measures Rust code generation speed.

### End-to-End Compilation
Measures full compilation pipeline from .crab to binary.
