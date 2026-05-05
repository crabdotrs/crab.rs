# Features

This document describes the complete feature set of the Crab programming language. Crab is a Dart-syntax language that transpiles to idiomatic Rust, combining ergonomic development with Rust's performance and safety guarantees.

## Core Syntax and Type System

### Variables and Declarations

- Variables with type inference: `var x = 10;`
- Explicit type annotations: `int x = 10;`
- Final variables: `final x = 10;`
- Mutable variables: `var x = 10; x = 20;`
- Multiple assignments: `var a = 1, b = 2;`
- Constants: `const x = 10;` (compile-time evaluation)
- Type aliases: `typedef Point = (double x, double y);`

### Primitive Types

- Integers: `int` (64-bit signed)
- Floating point: `double` (64-bit IEEE 754)
- Boolean: `bool` with values `true` and `false`
- Strings: `String` (UTF-8 encoded)
- Unit type: `void` for functions with no return value
- Type checks at runtime: `is`, `is!` operators
- Type casting: `as` operator for explicit conversions

### Type System Features

- Type inference in variable declarations
- Type inference in function return types
- Type inference in generic instantiations
- Type inference in closure parameters
- Implicit numeric conversions: `int` to `double`
- Explicit type casts: `x as double`
- Type narrowing through flow analysis
- `Object` as the top type for all values
- `Never` type for diverging functions

## Null Safety and Optionality

### Nullable and Non-Nullable Types

- Non-nullable types by default: `String`, `int`, `List<T>`
- Nullable type syntax: `String?`, `int?`, `List<T>?`
- Null checking with smart casts: `if (x != null) { ... }`
- Null assertion operator: `x!` to assert non-null
- Null-coalescing operator: `x ?? defaultValue`
- Null-coalescing assignment: `x ??= value`
- Null-aware access operator: `obj?.property`
- Null-aware method calls: `obj?.method()`
- Safe chaining: `a?.b?.c?.d`
- Conditional expressions with null handling

### Optional Parameters and Fallbacks

- Optional positional parameters with defaults
- Optional named parameters with defaults
- Required named parameters: `{required String name}`
- Fallback values in null-coalescing operations

## Functions and Parameters

### Function Declarations

- Basic functions: `ReturnType name(params) { ... }`
- Void functions: `void name() { ... }`
- Expression-bodied functions: `=> expr`
- Async functions: `async ... {}`
- Generator functions with `yield`
- Async generators: `async* ...`

### Parameter Types

- Positional parameters: `func(a, b, c)`
- Optional positional parameters: `func([a, b])`
- Default values for optional parameters
- Named parameters: `func({required String name})`
- Named parameters with defaults: `func({String name = ""})`
- Mixed positional and named parameters
- Rest parameters: `func(a, ...rest)`

### Function Features

- Function types: `int Function(int, int) add`
- First-class functions as arguments
- Function references (tear-offs): `list.forEach(print)`
- Return type inference from expressions
- Parameter type inference in closures
- Named and optional named parameters

### Closures and Lambdas

- Anonymous functions: `(x) => x * 2`
- Block closures: `(x) { return x * 2; }`
- Capturing outer scope variables
- Nested function definitions
- Function composition patterns

## Control Flow

### Conditional Statements

- `if (condition) { ... }`
- `if-else` chains
- Ternary operator: `condition ? value1 : value2`
- Nested conditional expressions

### Loop Constructs

- For-each loops: `for (var x in iterable) { ... }`
- Traditional for loops: `for (int i = 0; i < 10; i++)`
- While loops: `while (condition) { ... }`
- Do-while loops: `do { ... } while (condition);`
- `break` and `continue` statements
- Labeled breaks and continues for nested loops
- Range-based iteration: `for (int i in 0..10)`

### Switch Expressions

- Exhaustive pattern matching: `switch (expr) { case pattern => value }`
- Default case with `_`
- Guard clauses: `case pattern when condition => value`
- Pattern destructuring in switch cases
- Multiple values per case: `case a || b => value`
- Compiler-enforced exhaustiveness checking

## Classes and Object-Oriented Programming

### Class Declaration

- Basic classes: `class Name { ... }`
- Inheritance: `class Child extends Parent { ... }`
- Abstract classes: `abstract class Base { ... }`
- Sealed classes for exhaustive pattern matching
- Final classes that cannot be extended
- Interface classes for pure contracts
- Base classes with restricted inheritance

### Constructors

- Standard constructors with parameter initialization
- Shorthand field initialization: `Name(this.field)`
- Named constructors: `ClassName.named() { ... }`
- Factory constructors for alternative instantiation
- Redirect constructors delegating to other constructors
- Initializer lists for field setup
- Super constructor calls: `super(args)`
- Const constructors for compile-time instances

### Fields and Properties

- Instance fields with type annotations
- Final fields for immutable state
- Field initialization at declaration
- Getters for computed properties: `int get value => _value;`
- Setters for controlled mutation: `set value(int v) { _value = v; }`
- Static fields and methods
- Private fields with underscore prefix: `_field`

### Methods

- Instance methods with full parameter support
- Static methods for utility functions
- Method overriding with `@override` annotation
- Abstract methods in abstract classes
- Operator method overloading
- Getters and setters as method forms

### Visibility and Access

- Public visibility by default
- Library-private visibility with underscore prefix
- Access control through module boundaries

### Inheritance

- Single inheritance with `extends`
- Parent constructor invocation
- Parent method and property access via `super`
- Method overriding with proper signatures

### Special Class Types

- Enum declarations with named values
- Enums with associated data and constructors
- Enums with methods and properties
- Sealed classes for algebraic data types

## Mixins, Extensions, and Interfaces

### Mixins

- Mixin declarations: `mixin Name { ... }`
- Applying mixins: `class X with Mixin1, Mixin2`
- Mixin constraints: `mixin M on Base { ... }`
- Multiple mixin composition
- Method resolution in mixin chains

### Extensions

- Extension methods on existing types
- Extension properties via getters and setters
- Extension static methods
- Chained extension applications
- Multiple extensions on the same type
- Zero-cost extension types as wrappers

### Interfaces

- Implicit interfaces from all classes
- Explicit interface implementation: `implements Interface`
- Multiple interface implementation
- Abstract method contracts without implementation

## Generics and Type Parameters

### Generic Declarations

- Generic classes: `class Box<T> { T value; }`
- Generic functions: `T first<T>(List<T> list)`
- Generic methods within classes
- Multiple type parameters: `class Pair<A, B>`
- Nested generic types: `Map<String, List<int>>`

### Type Constraints

- Bounded type parameters: `<T extends Base>`
- Upper bounds for type safety
- Implicit Object bounds for all types

### Type Inference

- Automatic generic type inference from usage
- Explicit type arguments when needed
- Inference from function return types
- Type widening and narrowing in generic contexts

## Records and Pattern Matching

### Record Types

- Positional records: `(Type1, Type2, Type3)`
- Named records: `({String name, int age})`
- Mixed positional and named records
- Record literals with type inference
- Nested record type definitions

### Record Access and Destructuring

- Positional field access: `record.$1`, `record.$2`
- Named field access: `record.name`, `record.age`
- Tuple unpacking in assignments
- Destructuring in function parameters

### Pattern Matching

- Destructuring patterns in assignments
- Pattern matching in switch expressions
- Nested pattern destructuring
- Guard patterns with conditions
- Wildcard patterns for catch-all cases
- Or patterns for multiple value matching
- Constant patterns for literal matching
- Binding patterns for value capture
- Record patterns with named field matching

### Exhaustiveness Checking

- Compiler verification of complete pattern coverage
- Errors on non-exhaustive match expressions
- Default case handling for remaining patterns

## Enums and Algebraic Data Types

### Basic Enums

- Enum declarations with named values
- Enum value access and comparison
- Enum index properties

### Enhanced Enums

- Enums with associated fields and constructors
- Enum methods and computed properties
- Enum value comparison and iteration

### Sealed Classes as Union Types

- Sealed class declarations for closed hierarchies
- Subclass definitions for variant types
- Exhaustive pattern matching on sealed classes
- Named subclass constructors

## Collections

### Collection Types

- Lists: `List<T>` with array literal syntax
- Maps: `Map<K, V>` with key-value literal syntax
- Sets: `Set<T>` with unique element semantics
- Iterable interface for lazy sequences
- String as character sequence

### Collection Literals

- List literals: `[1, 2, 3]`
- Map literals: `{'key': value}`
- Set literals: `{1, 2, 3}`
- Empty collection literals with type inference
- Type annotation for empty collections

### Collection Operations

- Length property and indexing
- Safe indexing with null-aware operators
- Element addition and removal
- Iteration with `forEach`, `map`, `filter`
- Membership testing with `contains`
- Sorting and reversing operations
- String joining from collections

### Spread Operator

- List spread: `[...list1, ...list2]`
- Map spread: `{...map1, ...map2}`
- Null-aware spread: `[...?nullableList]`
- Set spread operations

### Collection Control Flow

- Conditional elements: `[if (cond) value]`
- For-loop elements: `[for (item in list) item * 2]`
- Combined conditional and iteration
- Nested collection comprehensions

### Iteration Patterns

- For-each iteration over collections
- Indexed iteration patterns
- Iterator interface access
- While loops with explicit iterators

## Async, Concurrency, and Streams

### Futures and Async/Await

- `Future<T>` type for asynchronous results
- Async function declarations
- Await expressions for result retrieval
- Completed future creation: `Future.value(v)`
- Delayed futures: `Future.delayed(duration)`
- Concurrent future waiting: `Future.wait(list)`
- Future chaining with `then` and error handling
- Timeout support for long-running operations

### Streams

- `Stream<T>` type for lazy sequences
- Stream creation from iterables
- Periodic stream generation
- Async-for loops: `await for (item in stream)`
- Stream transformation operations
- Stream subscription and control
- Pause, resume, and cancellation support

### Generator Functions

- Generator functions with `yield`
- Async generator functions: `async*`
- Iterable-returning generator methods
- Recursive generator patterns
- Yield delegation with `yield*`

### Concurrency Runtime

- Tokio runtime integration for async execution
- Task spawning and scheduling
- Message passing via channels
- Synchronization primitives for shared state

## Metaprogramming and Annotations

### Annotations

- Built-in annotations: `@override`, `@deprecated`
- Custom annotation class definitions
- Annotation application to classes, methods, and properties
- Annotations with constructor arguments
- Multiple annotations on single elements

### Limited Reflection

- Runtime type checking with `is` and `is!`
- Type casting with `as` operator
- Runtime type property access

## Operators and Operator Overloading

### Arithmetic Operators

- Addition, subtraction, multiplication, division
- Integer division and modulo operations
- Exponentiation operator
- Unary negation

### Comparison Operators

- Equality and inequality comparisons
- Relational operators: less than, greater than, and variants
- Identity comparison function

### Logical Operators

- Logical AND, OR, and NOT operations

### Bitwise Operators

- Bitwise AND, OR, XOR, and NOT
- Left and right shift operations
- Unsigned right shift

### Assignment Operators

- Simple and compound assignment operators
- Null-coalescing assignment

### Increment and Decrement

- Pre and post increment/decrement operators

### Conditional and Access Operators

- Ternary conditional operator
- Null coalescing and null-aware access
- Range operators for iteration
- Index access and assignment

### Operator Overloading

- Custom implementations for arithmetic operators
- Comparison operator overloading
- Bitwise operator overloading
- Index and call operator overloading
- Cascade operator for method chaining

## Error Handling

### Result Type

- `Result<T, E>` generic type for success or error
- `Ok(value)` and `Err(error)` constructors
- Pattern matching on Result variants
- Error propagation with `?` operator
- Result transformation methods

### Option Type

- `Option<T>` generic type for optional values
- `Some(value)` and `None` constructors
- Pattern matching on Option variants
- Unwrapping with defaults and mapping operations

### Error Propagation

- `?` operator for early error return
- Result chaining with propagation
- Error type conversion

### Exception Handling

- Try-catch-finally blocks for error recovery
- Exception type filtering in catch clauses
- Finally blocks for cleanup operations

## Modules, Imports, and Visibility

### Import Statements

- Relative module imports
- Package-based imports for external crates
- Import aliases for namespace management
- Selective imports with show and hide clauses
- Deferred imports for lazy loading

### Export Statements

- Module re-export for public API composition
- Selective export with show and hide

### Library Structure

- Library directive for module identification
- Multi-file library composition with part directives
- Part-of relationships for library cohesion

### Visibility Rules

- Public visibility by default
- Library-private visibility with underscore prefix
- Module boundary enforcement

## String Handling

### String Literals

- Double and single quoted string literals
- Raw strings for literal backslash handling
- Multi-line string literals with triple quotes

### String Interpolation

- Simple variable interpolation: `"Hello $name"`
- Expression interpolation: `"Count: ${items.length}"`
- Method call interpolation within strings
- Nested interpolation support

### String Operations

- Concatenation and repetition
- Length property and character indexing
- Substring extraction
- Case conversion methods
- Trimming and whitespace handling
- Splitting and joining operations
- Pattern matching with contains, startsWith, endsWith
- Replacement operations
- Regular expression matching

### Escape Sequences

- Standard escape sequences for special characters
- Unicode escape sequences

## C Interoperability

### Header Imports

- Direct C header imports: `import "stdio.h";`
- Imported header aliasing
- Multiple header support
- Automatic FFI binding generation
- C function and macro access

### Inline C Blocks

- CBlock syntax for inline C code
- Direct C function definitions
- Calling CBlock functions from Crab code
- Automatic ABI glue code generation
- Type conversion between Crab and C types

### C Type Mapping

- Primitive type mappings between Crab and C
- String to C string pointer conversion
- List to C array conversion
- Struct support via FFI

### Memory Management

- Automatic resource cleanup via Rust drop
- Safe wrapper functions around C allocations
- Reference counting for shared resources

### C Function Integration

- Direct C function calls from Crab code
- Argument passing and return value handling
- Side effect management

### Safety Boundaries

- Unsafe block marking for C interop regions
- Safe wrapper generation around unsafe C code

## Type Conversion and Casting

### Implicit Conversions

- Numeric widening: `int` to `double`
- Subclass to superclass conversions
- Nullable to non-nullable after null checks

### Explicit Conversions

- `as` operator for type casting
- Type-specific conversion methods: `toInt()`, `toDouble()`, `toString()`
- Parsing methods for string to numeric conversion
- Safe casting with type checks

### Type Checking

- `is` and `is!` operators for runtime type tests
- Smart casts after type checks
- Runtime type property access

## Standard Library Facades

### Collections

- List, Map, and Set operations
- Iterable and Iterator trait implementations

### Input and Output

- Print and println functions
- File operations via Rust standard library
- Console input handling

### Mathematics

- Basic mathematical functions
- Random number generation
- Trigonometric and other math operations

### Time and Duration

- Duration construction and manipulation
- DateTime operations for current time and parsing
- Stopwatch functionality for timing

### Async and Futures

- Future and Stream type support
- Async/await execution model
- Channel-based async messaging

### Result and Option

- Result and Option type handling utilities
- Error mapping and unwrapping operations

## Intentionally Unsupported Features

The following features are intentionally not supported to maintain language safety and simplicity:

- Dynamic type system that bypasses compile-time checks
- Full reflection API with runtime code inspection
- Late-initialized variables without null safety
- Dart-style isolates for concurrency
- Compile-time macros for code generation
- Complex associated type systems
- Generic associated types
- Unsafe blocks for general user code
- Variadic function parameters
- Method overloading by signature
- Implicit type conversions beyond numeric widening
- Custom operator precedence rules
- Goto statements and arbitrary label jumps

## Tooling and Development Experience

### Command Line Interface

- Project scaffolding with `crab new`
- Build and transpilation with `crab build`
- Direct execution with `crab run`
- Testing with `crab test`
- Formatting with `crab fmt`
- Linting with `crab lint`
- Project checking with `crab check`

### Language Server Protocol

- Hover information for symbols and types
- Go-to-definition navigation
- Diagnostic reporting for errors and warnings
- Completion suggestions for identifiers and members

### Code Formatting

- Consistent Dart-style formatting output
- Idempotent formatting operations
- Configurable formatting options

### Static Analysis

- Null safety enforcement
- Exhaustiveness checking for pattern matches
- Style rule enforcement
- Unused import and variable detection

### Editor Integration

- VSCode extension with syntax highlighting and LSP support
- IntelliJ/IDEA plugin for JetBrains IDEs
- Neovim configuration for treesitter and LSP integration

### Syntax Highlighting

- Treesitter grammar for structural parsing
- Syntax highlighting queries for editors
- Structural navigation support

## Documentation and Learning Resources

### Getting Started Guide

- Quick setup and installation instructions
- Hello world example with compilation and execution
- Basic language concepts introduction

### Cookbook

- Practical recipes for common patterns
- Null safety usage examples
- Async/await patterns and best practices
- C interop integration examples
- Result and Option handling patterns

### Tutorial Project

- Step-by-step API development tutorial
- Cookie Shop API example with Actix-Web
- Copy-paste runnable code examples
- Progressive complexity introduction

### Language Reference

- Complete syntax specification
- Type system documentation
- Standard library facade reference
- Tooling CLI reference

## Performance and Runtime Characteristics

### Compilation

- Transpilation from `.crab` to idiomatic `.rs`
- Delegation to system `rustc` and `cargo` for native compilation
- Zero-cost abstraction preservation

### Runtime

- Binary startup time optimized for minimal overhead
- Runtime performance equivalent to hand-written Rust
- Tokio runtime for async execution with efficient scheduling (will be rewritten in v0.1.1 added macros we'll call it annotations just like rusts macro #[tokio::main] -> @tokio.main)
- Concurrency scaling with available CPU cores

### Memory Safety

- Rust's ownership and borrowing guarantees preserved
- No runtime garbage collection overhead
- Compile-time memory safety enforcement

## Project Structure and Organization

### Source Code Organization

- Logical module separation for compiler components
- Clear separation between lexer, parser, codegen, and CLI
- Workspace-based Cargo configuration for multi-crate management

### Example Projects

- Progressive complexity examples from hello world to full API
- Each example demonstrates specific language features
- All examples verified to compile and execute correctly

### Testing Strategy

- Unit tests for compiler components
- Integration tests for end-to-end transpilation
- Example project verification tests
- Performance benchmarking suite

## Release and Distribution

### Build Artifacts

- Native binary output via cargo delegation
- Cross-platform compilation support
- Release optimization profiles

### Package Management

- Crab project manifest (`crab.toml`) for dependencies
- Rust crate dependency specification and resolution
- Local and remote package support

### Versioning and Compatibility

- Semantic versioning for language and tooling
- Rust edition compatibility tracking
- Backward compatibility considerations for language evolution
