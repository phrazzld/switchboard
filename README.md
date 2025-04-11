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

- Python and pip (for pre-commit hooks)
- [pre-commit](https://pre-commit.com/) for automated code quality checks

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | HTTP port to listen on | 8080 |
| `ANTHROPIC_API_KEY` | Your Anthropic API key (required) | - |
| `ANTHROPIC_TARGET_URL` | Anthropic API base URL | https://api.anthropic.com |
| `LOG_LEVEL` | Logging level (info, debug, etc.) | info |
| `LOG_FORMAT` | Log output format (pretty or json) | pretty |

## Getting Started

### Setup

1. Clone the repository
2. Create a `.env` file in the project root (or set environment variables):

```
PORT=8080
ANTHROPIC_API_KEY=your-api-key-here
LOG_LEVEL=info
LOG_FORMAT=pretty
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

We use pre-commit hooks to ensure code quality checks run before each commit:

1. Install pre-commit:
   ```bash
   pip install pre-commit
   # Or use your system's package manager, e.g.:
   # brew install pre-commit
   ```

2. Install the hooks:
   ```bash
   pre-commit install
   ```

This will automatically run formatting and linting checks before each commit. If any checks fail, the commit will be aborted. Fix the issues and try again.

## License

This project is licensed under the MIT License.