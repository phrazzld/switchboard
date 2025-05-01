//! Logging configuration and setup
//!
//! This module handles the initialization and configuration of the tracing system
//! used for structured logging throughout the application with support for
//! dual outputs (file and stdout).
//!
//! # Dual-Output Logging
//!
//! The logging system is designed to output logs to two destinations simultaneously:
//!
//! 1. **File Output:** JSON-formatted logs written to a file with daily rotation
//!    - Uses non-blocking I/O to prevent application slowdowns
//!    - Configurable minimum log level via `log_file_level`
//!    - JSON format for easy parsing and analysis
//!    - Daily rotation appends the date to filenames (e.g., `switchboard.log.2023-04-24`)
//!
//! 2. **Stdout Output:** Configurable format for console display
//!    - Choose between human-readable "pretty" format or JSON
//!    - Configurable minimum log level via `log_stdout_level`
//!
//! # Security Features
//!
//! The logging system includes several security features:
//!
//! - **Path validation** to prevent path traversal attacks (e.g., via `../` in paths)
//! - **Reserved path protection** to prevent writing logs to system directories
//! - **Permission validation** to ensure the target directory is writable
//! - **Path canonicalization** to resolve and validate absolute paths
//!
//! # Configuration
//!
//! Logging is configured through the following environment variables:
//!
//! - `LOG_FILE_PATH`: Path to the log file (default: "./switchboard.log")
//! - `LOG_FILE_LEVEL`: Minimum level for file logs (default: "debug")
//! - `LOG_LEVEL`: Minimum level for stdout logs (default: "info")
//! - `LOG_FORMAT`: Format for stdout logs ("pretty" or "json", default: "pretty")
//! - `LOG_BODIES`: Whether to log request/response bodies (default: "true")
//! - `LOG_MAX_BODY_SIZE`: Maximum size for logged bodies in bytes (default: "20480")
//!
//! # JSON Log Format
//!
//! When logging to files (or stdout with JSON format), logs follow this schema:
//!
//! ```json
//! {
//!   "timestamp": "2023-04-24T12:34:56.789012Z",  // ISO-8601 UTC timestamp
//!   "level": "INFO",                             // Log level
//!   "fields": {                                  // Structured fields
//!     "message": "Log message here",             // The log message
//!     "field1": "value1",                        // Additional structured fields
//!     "field2": 123                              // Numeric values preserved as numbers
//!   },
//!   "target": "switchboard::module_name",        // Source module
//!   "span": {                                    // Span information (if present)
//!     "name": "span_name"
//!   },
//!   "spans": [                                   // Span hierarchy (if present)
//!     {"name": "span_name"}
//!   ]
//! }
//! ```
//!
//! # Non-Blocking I/O
//!
//! File logging uses non-blocking I/O through the `tracing_appender` crate. This prevents
//! the application from blocking when writing logs to disk, which is important for maintaining
//! performance under high loads. The `WorkerGuard` returned by `init_tracing()` must be kept
//! alive for the duration of the application to ensure logs are properly flushed.

mod environment;
mod error;
mod init;
mod path_resolver;

#[allow(unused_imports)]
// Re-export public items that are used in tests but may show as unused in the binary
pub use environment::{
    detect_environment, get_environment_log_directory, get_xdg_log_directory, LogEnvironment,
};
pub use error::LogInitError;
pub use init::init_tracing;
#[allow(unused_imports)]
pub use path_resolver::validate_log_path;
pub use path_resolver::{LogPathResolver, LogType};

// Constants that need to be accessible from multiple modules
/// Default base directory for logs
pub const DEFAULT_LOG_DIR: &str = "./logs";
/// Subdirectory for application logs
pub const APP_LOG_SUBDIR: &str = "app";
/// Subdirectory for test logs
pub const TEST_LOG_SUBDIR: &str = "test";
/// System log directory for Unix-like platforms
#[cfg(target_family = "unix")]
pub const SYSTEM_LOG_DIR: &str = "/var/log/switchboard";
/// System log directory for Windows
#[cfg(target_family = "windows")]
pub const SYSTEM_LOG_DIR: &str = "C:\\ProgramData\\Switchboard\\Logs";
