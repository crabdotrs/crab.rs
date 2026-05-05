# Contributing to Crab

Thank you for your interest in contributing to Crab. This document outlines the process for contributing to the project.

## Development Setup

1. Clone the repository:

```bash
git clone https://github.com/crabdotrs/crab.rs Crab
cd Crab
```

2. Install Rust toolchain (1.85+ required):

```bash
rustup update
```

3. Build the project:

```bash
cd crab.rs
cargo build --release
```

## Code Style

- Rust code follows standard Rustfmt formatting
- Crab language examples use 2-space indentation
- Maximum line length: 100 characters
- No emojis in code or documentation
- No comments in source files (self-documenting code preferred)

## Testing

Run all tests before submitting PR:

```bash
cd crab.rs
cargo test
```

Test examples compile:

```bash
cd examples
for f in *.crab; do crab build $f; done
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests and ensure they pass
5. Commit with clear, descriptive messages
6. Push to your fork
7. Submit a pull request

## PR Requirements

- All tests must pass
- New features require tests
- Documentation updated if needed
- No breaking changes without discussion

## Issue Reporting

Report bugs via GitHub Issues with:

- Clear reproduction steps
- Expected vs actual behavior
- Environment details (OS, Rust version)
- Minimal code example

## Code Review

All PRs require review from a maintainer. Address feedback promptly and professionally.

## Questions

For questions, open a discussion on GitHub Discussions or join the community chat.

## AI Agent Guidelines

When working with AI agents for Crab development:

- Follow the existing code style and patterns
- Run tests after making changes
- Update documentation if needed
- Keep changes small and focused
- Ask questions if unsure about code patterns
- You are responsible for your changes make sure to review and test it properly this project is on its own idea,code,topic,docs no copyright claims should be present except the contributors and supporters 99% of its code is human written yes we used AI's help but we didn't let it write code no we're not against ai written code but its not worth it to have a piece of code we dont know what this gonna do also we're hoping others would use this so we dont want any licensing issues
