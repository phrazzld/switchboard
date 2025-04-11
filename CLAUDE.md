# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands
- Build: `cargo build` (development) or `cargo build --release` (optimized)
- Run: `./target/release/anthropic-visibility-proxy`
- Test: `cargo test` (all tests) or `cargo test <test_name>` (single test)
- Format: `cargo fmt`
- Lint: `cargo clippy`

## Code Style
- **Formatting**: Enforced by `cargo fmt`
- **Linting**: Use `cargo clippy` to catch issues
- **Naming**: `camelCase` for unexported, `PascalCase` for exported
- **Imports**: Group standard library, external crates, then internal imports
- **Error Handling**: Return errors explicitly, avoid panic for recoverable errors
- **Comments**: Focus on explaining *why*, not *what* or *how*
- **Types**: Use specific types, avoid `any` equivalents, define clear interfaces

## Architecture
- Rust implementation with tokio async runtime
- Hexagonal/ports-and-adapters style with core logic isolated from infrastructure
- Simplicity is prioritized over premature optimization
- Pure functions preferred for core logic with side effects at boundaries
- Strong error handling and logging throughout
- Configuration via environment variables