# Crab Examples

This directory contains example programs demonstrating Crab language features.

## Basic Examples (01-13)

| File | Description | Key Features |
|------|-------------|--------------|
| 01_hello_world.crab | Minimal program | Functions, print |
| 02_variables.crab | Variable declarations | var, final, const, type inference |
| 03_null_safety.crab | Null safety features | ?, ??, ?., null assertion |
| 04_functions.crab | Functions and parameters | Positional, named, optional, arrow syntax |
| 05_control_flow.crab | Control structures | if/else, for, while, switch |
| 06_classes.crab | Object-oriented programming | Classes, inheritance, abstract, sealed |
| 07_generics.crab | Generic types | Generic classes and functions |
| 08_collections.crab | Collections | List, Map, Set, spread operator |
| 09_async.crab | Async programming | Future, async/await, Stream, async* |
| 10_error_handling.crab | Error handling | Result, Option, try-catch |
| 11_operators.crab | Operators | Arithmetic, comparison, logical, bitwise |
| 12_extensions.crab | Extension methods | Extending existing types |
| 13_strings.crab | String handling | Interpolation, multi-line, raw strings |

## Full Applications

### cookie_shop_api/

A complete REST API demonstrating production-ready patterns:

- **OOP**: Encapsulation, inheritance, polymorphism, abstraction
- **Design Patterns**: Factory, Builder, Repository, Strategy
- **Async**: Actix-web integration with async/await
- **Error Handling**: Result types throughout
- **Testing**: Unit tests for all components

See [cookie_shop_api/README.md](cookie_shop_api/README.md) for details.

### todo_list/

A command-line todo application:

- **CLI**: Command parsing and argument handling
- **File I/O**: JSON persistence
- **OOP**: Command pattern, encapsulation
- **Error Handling**: Result types for all operations

See [todo_list/README.md](todo_list/README.md) for details.

## Running Examples

### Single File Examples

```bash
cd examples
crab build 01_hello_world.crab
crab run 01_hello_world.crab
```

### Project Examples

```bash
cd examples/cookie_shop_api
crab build
crab run
```

### All Examples

```bash
cd examples
for f in *.crab; do
  echo "Building $f..."
  crab build "$f"
done
```

## Adding New Examples

1. Create a `.crab` file in this directory
2. Include a brief header comment describing the example
3. Add to the table above in this README
4. Test with `crab build` before submitting

## Verification

All examples should:

- Compile without errors: `crab build`
- Run without crashes: `crab run`
- Demonstrate specific language features
- Follow the Crab style guide (2-space indent, no comments)
