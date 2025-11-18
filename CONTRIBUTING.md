# Contributing to WebSocket Messaging Provider

Thank you for your interest in contributing to the WebSocket Messaging Provider!

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo
- Git

### Getting Started

1. Clone the repository:
```bash
git clone https://github.com/64BitAsura/wasm-cloud-websocket-provider.git
cd wasm-cloud-websocket-provider
```

2. Build the project:
```bash
cargo build
```

3. Run tests:
```bash
cargo test
```

## Project Structure

```
├── src/
│   ├── lib.rs           # Main provider implementation
│   ├── connection.rs    # Connection configuration
│   └── main.rs          # Binary entry point
├── tests/
│   └── integration_test.rs  # Integration tests
├── examples/
│   └── basic_usage.rs   # Example usage
├── wit/                 # WebAssembly Interface Types
│   ├── interfaces.wit
│   └── deps/
└── .github/
    └── workflows/
        └── ci.yml       # CI/CD configuration
```

## Making Changes

### Code Style

- Follow Rust standard formatting (use `cargo fmt`)
- Run clippy and fix warnings (use `cargo clippy`)
- Add tests for new features
- Update documentation as needed

### Testing

All changes should include appropriate tests:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Submitting Changes

1. Create a new branch for your feature:
```bash
git checkout -b feature/your-feature-name
```

2. Make your changes and commit:
```bash
git add .
git commit -m "Description of your changes"
```

3. Push to your fork:
```bash
git push origin feature/your-feature-name
```

4. Create a Pull Request on GitHub

## Code Review Process

- All PRs require at least one review
- CI must pass before merging
- Follow semantic versioning for releases

## Reporting Issues

When reporting issues, please include:
- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs

## Questions?

Feel free to open an issue for questions or discussions!
