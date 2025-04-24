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

### Logging Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LOG_LEVEL` | Minimum log level for stdout (trace, debug, info, warn, error) | info |
| `LOG_FILE_LEVEL` | Minimum log level for file output | debug |
| `LOG_FORMAT` | Log output format for stdout (pretty or json) | pretty |
| `LOG_FILE_PATH` | Path to the log file with daily rotation | ./switchboard.log |
| `LOG_BODIES` | Whether to log full request and response bodies | true |
| `LOG_MAX_BODY_SIZE` | Maximum size in bytes for logged bodies before truncation | 20480 |

## Getting Started

### Setup

1. Clone the repository
2. Create a `.env` file in the project root (or set environment variables):

```
# Server configuration
PORT=8080
ANTHROPIC_API_KEY=your-api-key-here
ANTHROPIC_TARGET_URL=https://api.anthropic.com

# Logging configuration
LOG_LEVEL=info                  # Stdout log level
LOG_FILE_LEVEL=debug            # File log level
LOG_FORMAT=pretty               # Stdout format (pretty or json)
LOG_FILE_PATH=./switchboard.log # Log file path with daily rotation
LOG_BODIES=true                 # Log request/response bodies
LOG_MAX_BODY_SIZE=20480         # Max size of logged bodies in bytes
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

Requests will be forwarded to the Anthropic API, and both requests and responses will be logged according to your logging configuration.

## Logging System

Switchboard implements a dual-output logging system that provides comprehensive logging capabilities with minimal performance impact.

### Dual-Output Logging

Logs are sent to two destinations simultaneously:

1. **File Output:**
   - JSON-formatted logs for machine parsing
   - Daily log rotation (files named like `switchboard.log.2023-04-24`)
   - Non-blocking I/O for minimal performance impact
   - Typically more verbose (default level: debug)
   - Ideal for automated analysis and troubleshooting

2. **Stdout Output:**
   - Configurable format (pretty or JSON)
   - Typically less verbose (default level: info)
   - Ideal for immediate feedback during development

### Log Formats

#### Pretty Format (default for stdout)

```
2023-04-24T12:34:56.789012Z  INFO switchboard::proxy_handler: Processing request method=POST path=/v1/messages query=
2023-04-24T12:34:56.842125Z  INFO switchboard::proxy_handler: Received response from Anthropic API with status 200 request_id=d29f1db4-73cb-4e8f-9cd1-b9a971b088ff status=200 headers_count=12
```

#### JSON Format (used for file logs)

```json
{
  "timestamp": "2023-04-24T12:34:56.789012Z",
  "level": "INFO",
  "fields": {
    "message": "Processing request",
    "method": "POST",
    "path": "/v1/messages",
    "query": ""
  },
  "target": "switchboard::proxy_handler"
}
```

### Log Levels

Logs can be filtered by level, from most to least verbose:

1. **trace**: Very detailed debugging information (rarely used)
2. **debug**: Technical details useful for debugging
3. **info**: General information about normal operation
4. **warn**: Warning conditions that don't prevent operation
5. **error**: Error conditions that may impair functionality

### Request and Response Body Logging

- Bodies are logged when `LOG_BODIES=true` (the default)
- Bodies are truncated at `LOG_MAX_BODY_SIZE` (default: 20480 bytes)
- Logged at DEBUG level for both request and response
- JSON bodies are pretty-printed for readability
- Sensitive headers like `Authorization` are automatically redacted

### Common Configuration Scenarios

#### Production Environment

```
LOG_LEVEL=warn                   # Show only warnings and errors on stdout
LOG_FILE_LEVEL=info              # Keep file logs at info level for troubleshooting
LOG_FILE_PATH=/var/log/switchboard/app.log  # Use a system log directory
LOG_BODIES=false                 # Disable body logging for privacy and performance
```

#### Development Environment

```
LOG_LEVEL=debug                  # Show detailed logs on stdout
LOG_FORMAT=pretty                # Use human-readable format
LOG_FILE_PATH=./dev.log          # Local log file
LOG_BODIES=true                  # Log bodies for debugging
```

#### Performance Testing

```
LOG_LEVEL=error                  # Minimize stdout logging
LOG_FILE_LEVEL=error             # Minimize file logging
LOG_BODIES=false                 # Disable body logging
```

### Log Rotation

File logs are automatically rotated daily with date suffixes:
- `switchboard.log.2023-04-23`
- `switchboard.log.2023-04-24`

This prevents log files from growing too large and makes it easier to find logs from a specific date.

### Troubleshooting with Logs

When investigating issues:

1. **Check file logs first**: They contain more detailed information (debug level)
2. **Increase stdout verbosity**: Set `LOG_LEVEL=debug` to see more details on the console
3. **Enable body logging**: Make sure `LOG_BODIES=true` to see full request/response content
4. **Look for correlation IDs**: Each request gets a unique ID that appears in all related logs
5. **Check log file permissions**: Ensure the application has write access to the log directory

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

We use pre-commit hooks to ensure code quality checks run before each commit. You can set up the pre-commit hook using either a file copy or a symbolic link.

#### Option 1: Copy Method (recommended for most users)

```bash
# Copy the hook to your git hooks directory
cp hooks/pre-commit .git/hooks/
# Make it executable (required on Unix-based systems)
chmod +x .git/hooks/pre-commit
```

#### Option 2: Symlink Method (better for development on the hook itself)

```bash
# Create a symbolic link to the hook
ln -sf "$(pwd)/hooks/pre-commit" .git/hooks/pre-commit
# Make it executable (required on Unix-based systems)
chmod +x .git/hooks/pre-commit
```

The symlink method is preferred if you plan to contribute improvements to the hook, as any changes you make to the hook will be immediately active without requiring you to copy the file again.

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
