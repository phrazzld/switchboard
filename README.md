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
| `LOG_DIRECTORY_MODE` | Controls how the log directory is determined (default, xdg, system) | default |
| `LOG_MAX_AGE_DAYS` | Maximum age for log files in days before automatic cleanup | None (disabled) |

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
LOG_DIRECTORY_MODE=default      # Log directory selection mode (default, xdg, system)
LOG_MAX_AGE_DAYS=30             # Cleanup logs older than 30 days (comment out to disable)
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

# View command-line options
./target/release/switchboard --help

# Run log cleanup and exit
./target/release/switchboard --clean-logs
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
LOG_DIRECTORY_MODE=system        # Use system logs directory (/var/log/switchboard)
LOG_FILE_PATH=app.log            # Log file name (will be placed in the system directory)
LOG_BODIES=false                 # Disable body logging for privacy and performance
LOG_MAX_AGE_DAYS=90              # Keep logs for 90 days
```

#### Development Environment

```
LOG_LEVEL=debug                  # Show detailed logs on stdout
LOG_FORMAT=pretty                # Use human-readable format
LOG_DIRECTORY_MODE=default       # Auto-detect (will use ./logs/ in development)
LOG_FILE_PATH=dev.log            # Log file name (will be placed in ./logs/)
LOG_BODIES=true                  # Log bodies for debugging
LOG_MAX_AGE_DAYS=14              # Clean up logs older than 14 days
```

#### User Installation

```
LOG_LEVEL=info                   # Standard logging level for general usage
LOG_FILE_LEVEL=debug             # More verbose file logs for troubleshooting
LOG_DIRECTORY_MODE=xdg           # Use XDG directory (e.g., ~/.local/share/switchboard/logs on Linux)
LOG_FILE_PATH=switchboard.log    # Log file name (will be placed in the XDG directory)
LOG_MAX_AGE_DAYS=30              # Clean up logs older than 30 days
```

#### Performance Testing

```
LOG_LEVEL=error                  # Minimize stdout logging
LOG_FILE_LEVEL=error             # Minimize file logging
LOG_DIRECTORY_MODE=default       # Use default directory
LOG_BODIES=false                 # Disable body logging
# LOG_MAX_AGE_DAYS               # Omit to disable log cleanup
```

### Log Rotation

File logs are automatically rotated daily with date suffixes:
- `switchboard.log.2023-04-23`
- `switchboard.log.2023-04-24`

This prevents log files from growing too large and makes it easier to find logs from a specific date.

### Automatic Log Cleanup

Switchboard includes automatic log cleanup functionality to prevent logs from accumulating indefinitely:

- **Configuration**: Set `LOG_MAX_AGE_DAYS` to specify the maximum age for log files (in days)
- **Startup Cleanup**: Logs older than the specified age are automatically cleaned up at application startup
- **Manual Cleanup**: Run the application with the `--clean-logs` flag to perform cleanup and exit
  ```bash
  # Clean up logs and exit (using configured LOG_MAX_AGE_DAYS)
  ./target/release/switchboard --clean-logs
  ```
- **Cleanup Scope**: Both application and test logs are cleaned up
- **Safety**: Non-log files are never removed, even if they're in the log directories
- **Detailed Reporting**: Cleanup results are logged with file counts and total bytes removed

When cleanup is enabled, the application will scan both the app/ and test/ log subdirectories and remove any log files with modification times older than the specified cutoff date.

```
┌────────────────────┐     ┌────────────────────┐     ┌────────────────────┐
│ Configuration      │     │ Cleanup Trigger    │     │ Log Directories    │
│ LOG_MAX_AGE_DAYS=30│────▶│ - Application Start│────▶│ - logs/app/        │
│                    │     │ - --clean-logs Flag│     │ - logs/test/       │
└────────────────────┘     └────────────────────┘     └────────────────────┘
           │                                                    │
           │                                                    │
           ▼                                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           Log Cleanup Process                           │
│                                                                         │
│  1. Calculate cutoff date (current date - LOG_MAX_AGE_DAYS)            │
│  2. Scan both app/ and test/ directories                               │
│  3. Identify log files (*.log and *.log.YYYY-MM-DD format)             │
│  4. Check modification times against cutoff date                        │
│  5. Remove log files older than cutoff date                             │
│  6. Report cleanup results (files and bytes removed)                    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Log Directory Structure

Logs are organized in a structured directory hierarchy with separate subdirectories for application and test logs. The base directory depends on the deployment environment (configurable with `LOG_DIRECTORY_MODE`):

```
<base_directory>/
├── app/                    # Application logs 
│   ├── switchboard.log
│   ├── switchboard.log.2023-04-23
│   └── switchboard.log.2023-04-24
└── test/                   # Test logs
    ├── test_switchboard.log
    ├── test_switchboard.log.2023-04-23
    └── test_switchboard.log.2023-04-24
```

This separation ensures:
1. Application logs don't get mixed with test logs
2. Log files are organized by purpose
3. Cleanup and retention policies can be applied separately

#### Environment-Specific Base Directories

The base directory for logs is determined based on the deployment environment and the `LOG_DIRECTORY_MODE` setting:

| Environment | LOG_DIRECTORY_MODE | Base Directory |
|-------------|-------------------|---------------|
| Development | default | `./logs/` (current directory) |
| User Installation | default or xdg | Linux: `~/.local/share/switchboard/logs`<br>macOS: `~/Library/Application Support/switchboard/logs`<br>Windows: `C:\Users\<user>\AppData\Roaming\switchboard\logs` |
| System Service | default or system | Unix: `/var/log/switchboard`<br>Windows: `C:\ProgramData\Switchboard\Logs` |

Environment detection is automatic:
- Development is detected by debug builds or when `SWITCHBOARD_DEV` environment variable is set
- User Installation is detected when running from a user's home directory
- System Service is detected when running as a service (systemd, launchd, Windows Service)

#### Path Resolution Process

When you provide a log file path (e.g., `./switchboard.log`), the logger:

1. Extracts just the filename from the provided path
2. Determines the appropriate base directory based on environment and configuration
3. Appends the appropriate subdirectory (`app/` or `test/`) based on the log type
4. Creates the directory structure if it doesn't exist
5. Sets appropriate permissions on the directories
6. Returns the fully resolved path (e.g., `./logs/app/switchboard.log`)

```
┌────────────────────┐     ┌────────────────────┐     ┌────────────────────┐
│ User Configuration │     │ Environment        │     │ Log Type           │
│ LOG_FILE_PATH      │     │ - Development      │     │ - Application      │
│ LOG_DIRECTORY_MODE │────▶│ - User Installation│────▶│ - Test             │
│                    │     │ - System Service   │     │                    │
└────────────────────┘     └────────────────────┘     └────────────────────┘
           │                        │                          │
           │                        │                          │
           ▼                        ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Path Resolution Process                          │
│                                                                         │
│  1. Extract filename from LOG_FILE_PATH                                 │
│  2. Determine base directory from environment & LOG_DIRECTORY_MODE      │
│  3. Append appropriate subdirectory (app/ or test/)                     │
│  4. Create directories if needed                                        │
│  5. Set appropriate permissions                                         │
└─────────────────────────────────────────────────────────────────────────┘
           │
           │
           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Final Resolved Path                             │
│                                                                         │
│  <base_directory>/<subdirectory>/<filename>                             │
│  Example: ./logs/app/switchboard.log                                    │
└─────────────────────────────────────────────────────────────────────────┘
```

#### Log Migration Utility

If you're upgrading from a previous version, you may have log files in the root directory. Use the provided migration utility to move them to the correct locations:

```bash
# Run the log migration utility
./tools/migrate_logs.sh
```

This script will automatically:
- Identify log files in the root directory
- Move them to the appropriate subdirectory based on naming patterns
- Preserve timestamps and handle duplicates

### Log Directory Mode

The `LOG_DIRECTORY_MODE` environment variable controls how the application selects the base directory for logs, allowing for different deployment scenarios:

- **default**: Automatically determines the log directory based on environment detection
  - **Development**: Uses `./logs/` in the current directory
  - **User Installation**: Uses XDG-compliant directory for the current user
  - **System Service**: Uses system log path (`/var/log/switchboard` on Unix-like systems)

- **xdg**: Forces use of XDG Base Directory specification
  - **Linux**: `~/.local/share/switchboard/logs`
  - **macOS**: `~/Library/Application Support/switchboard/logs`
  - **Windows**: `C:\Users\<user>\AppData\Roaming\switchboard\logs`

- **system**: Forces use of system log directories
  - **Unix-like**: `/var/log/switchboard`
  - **Windows**: `C:\ProgramData\Switchboard\Logs`

This allows for seamless operation in different environments without requiring manual configuration changes.

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
