# Switchboard

A Rust-based HTTP proxy service that intercepts and logs traffic between clients and the Anthropic API. Switchboard transparently forwards all requests and responses while providing comprehensive logging of both non-streaming and streaming API interactions.

## Features

- Transparently proxies requests to Anthropic API endpoints
- Detailed request and response logging with sensitive data masking
- Support for both streaming and non-streaming API responses
- Graceful shutdown handling for reliable operation
- Configurable via environment variables or .env file

## Prerequisites

- Rust 1.68.1 or later
- An Anthropic API key

### Development Prerequisites

For contributing to the project, you'll need the following additional tools:

- Git

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | HTTP port to listen on | 8080 |
| `ANTHROPIC_API_KEY` | Your Anthropic API key (required) | - |
| `ANTHROPIC_TARGET_URL` | Anthropic API base URL | https://api.anthropic.com |
| `LOG_LEVEL` | Logging level (info, debug, etc.) | info |
| `LOG_FORMAT` | Log output format (pretty or json) | pretty |
| `LOG_BODIES` | Whether to log full request and response bodies | true |

## Getting Started

### Setup

1. Clone the repository
2. Create a `.env` file in the project root (or set environment variables):

```
PORT=8080
ANTHROPIC_API_KEY=your-api-key-here
LOG_LEVEL=info
LOG_FORMAT=pretty
LOG_BODIES=true  # Enabled by default, set to false to disable full logging
```

### Building

```bash
# Development build
cargo build

# Production build
cargo build --release
```

### Running

```bash
# Run in development mode
cargo run

# Run with the compiled binary
./target/release/switchboard
```

### Testing

```bash
# Run all tests
cargo test
```

## Usage

Once running, the proxy service listens on the configured port (default: 8080). Configure your Anthropic API client to direct requests to this proxy instead of the Anthropic API directly:

```
# Original API endpoint
https://api.anthropic.com/v1/messages

# Proxied endpoint (if running locally on port 8080)
http://localhost:8080/v1/messages
```

Requests will be forwarded to the Anthropic API, and both requests and responses will be logged according to your LOG_LEVEL setting.

## Common Commands

```bash
# Check for errors without building
cargo check

# Run formatter
cargo fmt

# Run linter
cargo clippy

# Generate and open documentation
cargo doc --open
```

## Contributing

### Setting Up Pre-commit Hooks

We use pre-commit hooks to ensure code quality checks run before each commit. To enable the pre-commit hook:

```bash
# Copy the hook to your git hooks directory
cp hooks/pre-commit .git/hooks/
# Make it executable (if needed)
chmod +x .git/hooks/pre-commit
```

The pre-commit hook performs the following checks:

1. **File Length Check**: Ensures Rust files maintain reasonable size
   - Warning at 500+ lines: Suggests refactoring but allows commit
   - Error at 1000+ lines: Blocks commit until file is refactored into smaller modules

2. **Code Quality Checks**:
   - `cargo fmt --check`: Verifies code adheres to formatting standards
   - `cargo clippy -- -D warnings`: Ensures code passes linting checks

3. **Test Execution**:
   - `cargo test`: Runs all tests to ensure functionality is maintained

If any checks fail, the commit will be aborted with a descriptive error message. Fix the issues and try again.

## License

This project is licensed under the MIT License.
