# Switchboard

A Rust-based HTTP proxy service that intercepts and logs traffic between clients and LLM APIs. Switchboard transparently forwards all requests and responses while providing comprehensive logging of both non-streaming and streaming API interactions.

## Features

- Transparently proxies requests to Anthropic API endpoints
- Optional support for OpenAI API integration (disabled by default)
- Detailed request and response logging with sensitive data masking
- Support for both streaming and non-streaming API responses
- Graceful shutdown handling for reliable operation
- Configurable via environment variables or .env file

## Prerequisites

- Rust 1.68.1 or later
- An Anthropic API key
- An OpenAI API key (only required if OpenAI integration is enabled)

### Development Prerequisites

For contributing to the project, you'll need the following additional tools:

- Git
- Python 3.6+ (for pre-commit hooks)
- pip (for installing pre-commit)
- glance (optional, used by post-commit hook)

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | HTTP port to listen on | `DEFAULT_PORT` (8080) |
| `ANTHROPIC_API_KEY` | Your Anthropic API key (required) | - |
| `ANTHROPIC_TARGET_URL` | Anthropic API base URL | `DEFAULT_ANTHROPIC_TARGET_URL` (https://api.anthropic.com) |

### OpenAI Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | Your OpenAI API key (required when OpenAI is enabled) | - |
| `OPENAI_API_BASE_URL` | OpenAI API base URL | `DEFAULT_OPENAI_TARGET_URL` (https://api.openai.com) |
| `OPENAI_ENABLED` | Enable OpenAI integration | `DEFAULT_OPENAI_ENABLED` (false) |

### Logging Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LOG_LEVEL` | Minimum log level for stdout (trace, debug, info, warn, error) | `DEFAULT_LOG_STDOUT_LEVEL` (info) |
| `LOG_FILE_LEVEL` | Minimum log level for file output | `DEFAULT_LOG_FILE_LEVEL` (debug) |
| `LOG_FORMAT` | Log output format for stdout (pretty or json) | `DEFAULT_LOG_FORMAT` (pretty) |
| `LOG_FILE_PATH` | Path to the log file with daily rotation | `DEFAULT_LOG_FILE_PATH` (./switchboard.log) |
| `LOG_BODIES` | Whether to log full request and response bodies | `DEFAULT_LOG_BODIES` (true) |
| `LOG_MAX_BODY_SIZE` | Maximum size in bytes for logged bodies before truncation | `DEFAULT_LOG_MAX_BODY_SIZE` (20480) |
| `LOG_DIRECTORY_MODE` | Controls how the log directory is determined (default, xdg, system) | `LogDirectoryMode::Default` (default) |
| `LOG_MAX_AGE_DAYS` | Maximum age for log files in days before automatic cleanup | `DEFAULT_LOG_MAX_AGE_DAYS` (None - disabled) |

> Note: All default values are centralized in `src/config.rs` as constants to ensure consistency throughout the application.

## Getting Started

### Setup

1. Clone the repository
2. Create a `.env` file in the project root (or set environment variables):

```
# Server configuration
PORT=8080                       # From DEFAULT_PORT
ANTHROPIC_API_KEY=your-api-key-here
ANTHROPIC_TARGET_URL=https://api.anthropic.com  # From DEFAULT_ANTHROPIC_TARGET_URL

# OpenAI configuration (optional)
# OPENAI_API_KEY=your-openai-api-key-here  # Required if OPENAI_ENABLED=true
# OPENAI_API_BASE_URL=https://api.openai.com  # From DEFAULT_OPENAI_TARGET_URL
# OPENAI_ENABLED=false         # From DEFAULT_OPENAI_ENABLED (set to true to enable)

# Logging configuration
LOG_LEVEL=info                  # From DEFAULT_LOG_STDOUT_LEVEL
LOG_FILE_LEVEL=debug            # From DEFAULT_LOG_FILE_LEVEL
LOG_FORMAT=pretty               # From DEFAULT_LOG_FORMAT
LOG_FILE_PATH=./switchboard.log # From DEFAULT_LOG_FILE_PATH
LOG_BODIES=true                 # From DEFAULT_LOG_BODIES
LOG_MAX_BODY_SIZE=20480         # From DEFAULT_LOG_MAX_BODY_SIZE
LOG_DIRECTORY_MODE=default      # Maps to LogDirectoryMode::Default
LOG_MAX_AGE_DAYS=30             # Cleanup logs older than 30 days (defaults to None when not set)
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

Once running, the proxy service listens on the configured port (default: 8080). Configure your LLM API client to direct requests to this proxy instead of the API directly:

### Anthropic API

```
# Original API endpoint
https://api.anthropic.com/v1/messages

# Proxied endpoint (if running locally on port 8080)
http://localhost:8080/v1/messages
```

### OpenAI API (when enabled)

```
# Original API endpoint
https://api.openai.com/v1/chat/completions

# Proxied endpoint (if running locally on port 8080)
http://localhost:8080/v1/chat/completions
```

Requests will be forwarded to the appropriate API, and both requests and responses will be logged according to your logging configuration. 

Note that OpenAI integration is disabled by default and must be explicitly enabled by setting `OPENAI_ENABLED=true` and providing a valid `OPENAI_API_KEY`.

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
LOG_LEVEL=warn                   # Higher than DEFAULT_LOG_STDOUT_LEVEL for less verbose output
LOG_FILE_LEVEL=info              # Less verbose than DEFAULT_LOG_FILE_LEVEL
LOG_DIRECTORY_MODE=system        # Use LogDirectoryMode::System
LOG_FILE_PATH=app.log            # Different from DEFAULT_LOG_FILE_PATH
LOG_BODIES=false                 # Opposite of DEFAULT_LOG_BODIES
LOG_MAX_AGE_DAYS=90              # Longer retention than default
```

#### Development Environment

```
LOG_LEVEL=debug                  # More verbose than DEFAULT_LOG_STDOUT_LEVEL
LOG_FORMAT=pretty                # Same as DEFAULT_LOG_FORMAT
LOG_DIRECTORY_MODE=default       # Same as LogDirectoryMode::Default
LOG_FILE_PATH=dev.log            # Different from DEFAULT_LOG_FILE_PATH
LOG_BODIES=true                  # Same as DEFAULT_LOG_BODIES
LOG_MAX_AGE_DAYS=14              # Shorter retention than typical production
```

#### User Installation

```
LOG_LEVEL=info                   # Same as DEFAULT_LOG_STDOUT_LEVEL
LOG_FILE_LEVEL=debug             # Same as DEFAULT_LOG_FILE_LEVEL
LOG_DIRECTORY_MODE=xdg           # Use LogDirectoryMode::Xdg
LOG_FILE_PATH=switchboard.log    # Similar to DEFAULT_LOG_FILE_PATH
LOG_MAX_AGE_DAYS=30              # Custom retention period
```

#### Performance Testing

```
LOG_LEVEL=error                  # Minimal logging compared to DEFAULT_LOG_STDOUT_LEVEL
LOG_FILE_LEVEL=error             # Minimal logging compared to DEFAULT_LOG_FILE_LEVEL
LOG_DIRECTORY_MODE=default       # Same as LogDirectoryMode::Default
LOG_BODIES=false                 # Opposite of DEFAULT_LOG_BODIES for performance
# LOG_MAX_AGE_DAYS               # Omitted to use DEFAULT_LOG_MAX_AGE_DAYS (None)
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

We use the Python-based `pre-commit` framework to manage Git hooks for ensuring code quality and commit message standards. This tool automatically runs checks before commits are created to catch issues early.

#### Prerequisites

Before setting up the hooks, ensure you have the following tools installed:

- **Python 3.6+**: Required to run the pre-commit framework
- **pip**: Python package manager to install pre-commit
- **glance** (optional): Used by post-commit hook to automatically scan the repository after commits

#### Installation and Setup

1. **Install pre-commit:**

```bash
# Install pre-commit using pip
pip install pre-commit
```

2. **Set up the hooks:**

```bash
# Install the pre-commit and commit-msg hooks
pre-commit install --hook-type pre-commit --hook-type commit-msg
```

3. **Set up the post-commit hook (optional):**

```bash
# Copy the post-commit hook template to your Git hooks directory
cp templates/post-commit.template .git/hooks/post-commit
# Make it executable (required on Unix-based systems)
chmod +x .git/hooks/post-commit
```

#### Hook Descriptions

The pre-commit framework manages the following hooks:

1. **rustfmt (pre-commit)**: 
   - Ensures all Rust code follows the project's formatting standards
   - Uses `cargo fmt -- --check` to verify without modifying files
   - Fails if any file doesn't meet the formatting standards

2. **clippy (pre-commit)**: 
   - Performs static analysis on Rust code to detect common issues
   - Uses `cargo clippy --all-targets -- -D warnings` to treat warnings as errors
   - Helps catch potential bugs and improve code quality

3. **commitlint (commit-msg)**: 
   - Validates commit messages follow the Conventional Commits format
   - Ensures messages have the proper structure (type, scope, description)
   - Helps maintain a clean, readable commit history

4. **post-commit (optional)**: 
   - Runs the `glance` tool asynchronously after successful commits
   - Provides quick feedback on the repository state
   - Non-blocking, so it won't delay your workflow

#### Usage Guidelines

The hooks run automatically when you commit changes. If a hook fails, the commit will be aborted with an error message. Fix the reported issues and try again.

If you need to bypass hooks in exceptional circumstances:

```bash
# Skip one or more specific hooks
SKIP=rustfmt,clippy git commit -m "your commit message"

# Skip all pre-commit hooks (not recommended for regular use)
SKIP=pre-commit git commit -m "your commit message"
```

**Note:** Using `git commit --no-verify` to bypass hooks is strongly discouraged as it circumvents all quality checks. Use the `SKIP` mechanism instead, which clearly documents which specific checks are being skipped.

#### Troubleshooting

If you encounter issues with the hooks:

1. Ensure pre-commit is installed: `pre-commit --version`
2. Update pre-commit and hooks: `pre-commit autoupdate`
3. Check hook configurations in `.pre-commit-config.yaml`
4. For commitlint issues, verify your commit message format against the Conventional Commits specification

## License

This project is licensed under the MIT License.
