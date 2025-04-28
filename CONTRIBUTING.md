# Contributing to Switchboard

This document outlines the development practices and tooling setup for contributing to the Switchboard project.

## Development Setup

### Prerequisites

- Rust toolchain (via [rustup](https://rustup.rs/))
- [pre-commit](https://pre-commit.com/) for Git hooks

### Installation

1. Clone the repository
   ```bash
   git clone https://github.com/yourusername/switchboard.git
   cd switchboard
   ```

2. Install pre-commit hooks
   ```bash
   pre-commit install --install-hooks
   ```

3. Build the project
   ```bash
   cargo build
   ```

## Pre-commit Hooks

Switchboard uses pre-commit hooks to ensure code quality and consistency. The following hooks are configured:

### Rust Formatter (rustfmt)

- Automatically formats Rust code according to the project's style
- Configuration is in `rustfmt.toml`
- Runs on all Rust files

### Rust Linter (clippy)

- Checks Rust code for common mistakes and style issues
- Uses the `--all-targets` and `-D warnings` flags to check all targets and treat warnings as errors
- Runs on all Rust files

### File Size Check

- Warns when files exceed 500 lines
- Fails when files exceed 1000 lines
- Purpose: Encourages smaller, more maintainable modules
- Excluded files:
  - `Cargo.lock` (generated file)
  - Files in `tests/linecounts/` (test files for the hook itself)
  - Files in `benches/` (benchmark files)

### Cargo Tests

- Runs all tests with `cargo test --no-fail-fast`
- Ensures all tests pass before allowing commits
- Runs at the pre-commit stage
- Applies to all Rust files, regardless of which files are being committed

### Commit Message Linting

- Enforces [Conventional Commits](https://www.conventionalcommits.org/) format
- Runs at the commit-msg stage
- Uses `@commitlint/config-conventional` rules

## Coding Guidelines

### Rust Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use meaningful variable and function names
- Write clear comments for complex sections
- Add documentation comments (`///`) for public API
- Keep functions small and focused
- Write tests for all new functionality

### Error Handling

- Use Result types for error handling
- Avoid panic!() in library code
- Use descriptive error messages
- Propagate errors up when appropriate

## Pull Request Process

1. Create a branch with a descriptive name
2. Make your changes, following the coding guidelines
3. Write or update tests as necessary
4. Run all checks locally (`cargo check`, `cargo test`, `cargo clippy`, etc.)
5. Submit a pull request with a clear description of the changes
6. Address any feedback from code reviews

## License

By contributing to Switchboard, you agree that your contributions will be licensed under the project's license.