# Crab Programming Language

Crab is a systems programming language with Dart-like syntax that transpiles to idiomatic Rust. Write expressive, type-safe code with the performance of native binaries.

## Features

- **Familiar Syntax**: Dart/OOP developers feel at home immediately
- **Zero-Cost Abstractions**: Compiles to Rust with no runtime overhead
- **Memory Safety**: Rust's ownership system prevents bugs at compile time
- **Native Performance**: Startup ~ rust or a little memory footprint
- **Null Safety**: Non-nullable by default, no billion-dollar mistakes
- **C Interoperability**: Direct C header imports and inline C blocks similar to unsafe block but its just pure c
- **Full OOP**: Classes, inheritance, polymorphism, encapsulation, sealed classes
- **Async/Await**: Native async support with Tokio runtime | its a problem and we will fix this soon by introducing macros as annotation @Annotation
- **Pattern Matching**: Exhaustive switch expressions
- **Type Inference**: Write less boilerplate clean code that doesnt hurt eyes
- **Crates.io Compatible**: The goal was never create a new programming language the goal was creating a subset of rust that increases iteration speed, generational oop patterns, easy to maintain, less scary (well rust isnt scary but I find myself annoyed sometimes with little things) etc.

## Quick Start

### Installation

```bash
git clone https://github.com/crabdotrs/crab.rs ~/.crab
cd ~/.crab/crab.rs
cargo install --path crab-cli
```

### Hello World

Create `hello.crab`:

```crab
Future<void> main() async {
  var message = greet("World");
  print(message);
}

String greet(String name) => "Hello, $name!";
```

Build and run:

```bash
crab build
./target/release/hello
```

## Example: Cookie Shop API (Not Commpleted </3 figured i had a bad decision will take 10-15 hours)

A complete REST API demonstrating OOP patterns:

```crab
import "actix-web";

class Cookie {
  String _id;
  String _name;
  double _price;

  Cookie(this._id, this._name, this._price);

  String get id => _id;
  String get name => _name;
  double get price => _price;
}

Future<HttpResponse> getCookies() async {
  var cookies = [Cookie("1", "Chocolate Chip", 2.50)];
  return HttpResponse.ok().json(cookies);
}
```

See `examples/cookie_shop_api/` for the full implementation.

## Language Tour

### Variables and Types

```crab
var x = 10;                    // Type inference
int y = 20;                   // Explicit type
final z = 30;                 // Immutable
String? maybe;                // Nullable
```

### Functions

```crab
int add(int a, int b) => a + b;

Future<String> fetch() async {
  return await http.get("/api");
}
```

### Classes and OOP

```crab
class Animal {
  String name;
  Animal(this.name);
  void speak() => print("Some sound");
}

class Dog extends Animal {
  Dog(super.name);
  @override void speak() => print("Woof!");
}

sealed class Result<T> {}
class Success<T> extends Result<T> {
  T value;
  Success(this.value);
}
```

### Error Handling

```crab
Result<int, String> parse(String input) {
  if (input.isEmpty) {
    return Err("Empty input");
  }
  return Ok(int.parse(input));
}

var result = parse("42");
switch (result) {
  case Ok v => print("Got: $v");
  case Err e => print("Error: $e");
}
```

### C Interop

```crab
CBlock {
  #include <stdio.h>
  void hello() {
    printf("Hello from C\n");
  }
}

void main() {
  hello();
}
```

## CLI Commands

| Command              | Description          |
| -------------------- | -------------------- |
| `crab new <name>`    | Create a new project |
| `crab build`         | Build the project    |
| `crab run`           | Build and run        |
| `crab test`          | Run tests            |
| `crab fmt`           | Format code          |
| `crab lint`          | Lint code            |
| `crab check`         | Type check           |
| `crab add <package>` | Add dependency       |

## Documentation

Not Available yet but hoping some support to make it available soon

## Project Structure

```
Crab/
  crab.rs/           # Transpiler source (Rust)
  docs/              # Documentation site (Next.js)
  examples/          # Example programs
  tools/             # LSP, formatter, linter
  editor/            # Editor integrations
  language/          # Branding assets
  templates/         # Project templates
  scripts/           # Build scripts
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - Copyright (c) 2026 crabdotrs

See [LICENSE](LICENSE) for full text.
