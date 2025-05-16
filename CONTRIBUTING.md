# Contributing to AuthGate

Thank you for considering contributing to AuthGate! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project. We aim to foster an inclusive and welcoming community.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue with the following information:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Any relevant logs or error messages
- Your environment (OS, Rust version, etc.)

### Suggesting Features

Feature suggestions are welcome! Please create an issue with:

- A clear, descriptive title
- A detailed description of the proposed feature
- Any relevant examples or use cases
- If possible, an implementation approach

### Pull Requests

1. Fork the repository
2. Create a new branch for your changes
3. Make your changes
4. Add or update tests as needed
5. Ensure all tests pass with `cargo test`
6. Submit a pull request

## Development Setup

1. Install Rust and Cargo (https://rustup.rs/)
2. Clone the repository
3. Run `cargo build` to build the project
4. Run `cargo test` to run the tests

## Project Structure

- `src/` - Source code
  - `auth.rs` - Authentication and authorization logic
  - `config.rs` - Configuration handling
  - `matcher.rs` - Route matching logic
  - `proxy.rs` - Forward auth proxy implementation
  - `types.rs` - Data structures and types
  - `main.rs` - Application entry point
  - `lib.rs` - Library exports
- `tests/` - Integration tests
- `examples/` - Example configurations and usage

## Coding Style

- Follow the Rust style guide
- Use meaningful variable and function names
- Write clear comments for complex logic
- Include documentation for public APIs

## Testing

- Write unit tests for new functionality
- Ensure existing tests pass
- Consider adding integration tests for complex features

## License

By contributing to this project, you agree that your contributions will be licensed under the project's MIT License.