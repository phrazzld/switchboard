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

use crate::config::Config;
use std::io;
use std::path::Path;
use thiserror::Error;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{fmt as tracing_fmt, prelude::*, registry, EnvFilter};

/// Error type for logging initialization failures
#[derive(Debug, Error)]
pub enum LogInitError {
    /// Failed to create the log directory
    #[error("Failed to create log directory at {path}: {source}")]
    DirectoryCreationFailed {
        /// Path to the directory that could not be created
        path: String,
        /// Source IO error
        #[source]
        source: io::Error,
    },

    /// Failed to parse log level
    #[error("Failed to parse log level filter: {0}")]
    FilterParseError(String),

    /// Base path issue for logging
    #[error("Invalid log file path: {0}")]
    InvalidPath(String),
}

/// Initialize the tracing system for structured logging with dual output
///
/// Sets up the tracing subscriber with two output layers:
/// 1. **File Layer**: JSON-formatted logs with level filtering by `log_file_level`, using
///    non-blocking I/O and daily rotation.
/// 2. **Stdout Layer**: Configurable format (pretty or JSON) with level filtering by `log_stdout_level`.
///
/// This function handles:
/// - Creating log directories if they don't exist
/// - Setting up daily log rotation
/// - Configuring non-blocking file I/O
/// - Applying the appropriate filters based on log levels
/// - Setting up the correct output format for each destination
///
/// The function logs its own initialization with configuration details, which serves as
/// verification that logging is properly set up.
///
/// # Arguments
/// * `config` - The application configuration containing logging settings
///
/// # Returns
/// Returns a `Result` containing a `WorkerGuard` which **must be kept alive** for the duration
/// of the application, or a `LogInitError` if initialization fails.
/// If this guard is dropped, pending logs might not be flushed to disk. In a typical
/// application, store this guard in your main application struct or in the `main` function.
///
/// # Errors
/// This function will return an error if:
/// - The log directory cannot be created
/// - The log file path is invalid
/// - The log level filters cannot be parsed
///
/// # Examples
/// Basic initialization with default settings:
/// ```
/// # use switchboard::config::Config;
/// # use switchboard::logger;
/// # // Create a mock config for testing instead of using global config
/// # let mock_config = Config {
/// #     port: "8080".to_string(),
/// #     anthropic_api_key: "test-key".to_string(),
/// #     anthropic_target_url: "https://example.com".to_string(),
/// #     log_stdout_level: "info".to_string(),
/// #     log_format: "pretty".to_string(),
/// #     log_bodies: true,
/// #     log_file_path: "./switchboard.log".to_string(),
/// #     log_file_level: "debug".to_string(),
/// #     log_max_body_size: 20480,
/// # };
/// // Initialize logging and keep the guard alive
/// let _guard = logger::init_tracing(&mock_config).expect("Failed to initialize logging");
///
/// // Your application code here...
///
/// // Guard automatically dropped at end of scope, flushing remaining logs
/// ```
///
/// Using JSON format for both outputs:
/// ```
/// # use switchboard::config::Config;
/// # use switchboard::logger;
/// let config = Config {
///     // ... other fields ...
///     log_stdout_level: "info".to_string(),
///     log_format: "json".to_string(), // Use JSON for stdout too
///     log_file_path: "./logs/switchboard.log".to_string(),
///     log_file_level: "debug".to_string(),
///     // ... other fields ...
///     # port: "8080".to_string(),
///     # anthropic_api_key: "test-key".to_string(),
///     # anthropic_target_url: "https://example.com".to_string(),
///     # log_bodies: true,
///     # log_max_body_size: 20480,
/// };
///
/// let _guard = logger::init_tracing(&config).expect("Failed to initialize logging");
/// ```
///
/// Different log levels for file and stdout:
/// ```
/// # use switchboard::config::Config;
/// # use switchboard::logger;
/// let config = Config {
///     // ... other fields ...
///     log_stdout_level: "warn".to_string(), // Only warnings and errors go to stdout
///     log_file_path: "./logs/switchboard.log".to_string(),
///     log_file_level: "trace".to_string(),  // Everything goes to the file
///     // ... other fields ...
///     # port: "8080".to_string(),
///     # anthropic_api_key: "test-key".to_string(),
///     # anthropic_target_url: "https://example.com".to_string(),
///     # log_format: "pretty".to_string(),
///     # log_bodies: true,
///     # log_max_body_size: 20480,
/// };
///
/// let _guard = logger::init_tracing(&config).expect("Failed to initialize logging");
/// ```
pub fn init_tracing(config: &Config) -> Result<WorkerGuard, LogInitError> {
    // Validate log file path
    if config.log_file_path.is_empty() {
        return Err(LogInitError::InvalidPath(
            "Log file path cannot be empty".to_string(),
        ));
    }

    // Parse log file path to get directory and filename
    let log_file_path = Path::new(&config.log_file_path);

    // Extract directory and filename
    let log_dir = log_file_path.parent().unwrap_or_else(|| Path::new("."));
    let log_file_name = match log_file_path.file_name() {
        Some(name) => name,
        None => {
            return Err(LogInitError::InvalidPath(format!(
                "Invalid log file path (no filename): {}",
                config.log_file_path
            )));
        }
    };

    // Create directory if it doesn't exist
    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            return Err(LogInitError::DirectoryCreationFailed {
                path: log_dir.display().to_string(),
                source: e,
            });
        }
    }

    // Create daily rotating file appender
    let file_appender = rolling::daily(log_dir, log_file_name);

    // Create non-blocking writer and get the guard
    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Create file filter based on config.log_file_level
    let file_filter = match EnvFilter::try_new(&config.log_file_level) {
        Ok(filter) => filter,
        Err(e) => {
            // Return a FilterParseError to make it clear what happened
            return Err(LogInitError::FilterParseError(format!(
                "Failed to parse file log level filter '{}': {}",
                config.log_file_level, e
            )));
        }
    };

    // Create file layer with JSON formatting
    let file_layer = tracing_fmt::layer()
        .json()
        .with_writer(non_blocking_writer)
        .with_filter(file_filter);

    // Create stdout filter based on RUST_LOG or config.log_stdout_level
    let stdout_filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => match EnvFilter::try_new(&config.log_stdout_level) {
            Ok(filter) => filter,
            Err(e) => {
                // Return a FilterParseError to make it clear what happened
                return Err(LogInitError::FilterParseError(format!(
                    "Failed to parse stdout log level filter '{}': {}",
                    config.log_stdout_level, e
                )));
            }
        },
    };

    // Create registry and add file layer
    let subscriber = registry().with(file_layer);

    // Add the appropriate stdout layer based on format
    if config.log_format == "json" {
        let json_layer = tracing_fmt::layer()
            .json()
            .with_writer(io::stdout)
            .with_filter(stdout_filter);
        subscriber.with(json_layer).init();
    } else {
        let pretty_layer = tracing_fmt::layer()
            .pretty()
            .with_writer(io::stdout)
            .with_filter(stdout_filter);
        subscriber.with(pretty_layer).init();
    }

    // Log initialization
    info!(
        log_stdout_level = %config.log_stdout_level,
        log_format = %config.log_format,
        log_file_path = %config.log_file_path,
        log_file_level = %config.log_file_level,
        "Dual logging initialized"
    );

    // Return guard to keep it alive
    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tracing::{debug, error, info, warn};

    #[test]
    fn test_dual_output_logging() {
        // Create a test file in the current directory
        let log_file_path = "test_switchboard.log";

        // Create a test configuration
        let config = Config {
            port: "0".to_string(),
            anthropic_api_key: "test-key".to_string(),
            anthropic_target_url: "https://example.com".to_string(),
            log_stdout_level: "info".to_string(),
            log_format: "pretty".to_string(),
            log_bodies: true,
            log_file_path: log_file_path.to_string(),
            log_file_level: "debug".to_string(),
            log_max_body_size: 1024,
        };

        // Initialize logging
        let _guard = init_tracing(&config).expect("Failed to initialize logging");

        // Emit logs at different levels
        debug!("This is a debug message");
        info!("This is an info message");
        warn!("This is a warning message");
        error!("This is an error message");

        // Explicitly drop the guard to flush logs
        drop(_guard);

        // Clean up file after test
        let _ = fs::remove_file(log_file_path);

        // This test simply verifies the dual-output logging initializes properly
        // We don't attempt to verify the content of the logs due to the complexity
        // of capturing and parsing both stdout and file output in a unit test.
        // The real test is that the code doesn't panic or crash.
    }

    #[test]
    fn test_directory_creation_error() {
        // Create a configuration with a path to a location that cannot be created
        // On most systems, writing to /proc/ should fail
        let config = Config {
            port: "0".to_string(),
            anthropic_api_key: "test-key".to_string(),
            anthropic_target_url: "https://example.com".to_string(),
            log_stdout_level: "info".to_string(),
            log_format: "pretty".to_string(),
            log_bodies: true,

            // This should cause a directory creation error
            // (trying to create /proc/invalid is generally not allowed)
            log_file_path: "/proc/invalid/test.log".to_string(),

            log_file_level: "debug".to_string(),
            log_max_body_size: 1024,
        };

        // Initialize logging - should return an error
        let result = init_tracing(&config);

        // Assert that we got the expected error type
        assert!(matches!(
            result,
            Err(LogInitError::DirectoryCreationFailed { .. })
        ));
    }

    #[test]
    fn test_invalid_path_error() {
        // Create a configuration with an invalid path (empty filename)
        let config = Config {
            port: "0".to_string(),
            anthropic_api_key: "test-key".to_string(),
            anthropic_target_url: "https://example.com".to_string(),
            log_stdout_level: "info".to_string(),
            log_format: "pretty".to_string(),
            log_bodies: true,

            // Empty string - should fail validation
            log_file_path: "".to_string(),

            log_file_level: "debug".to_string(),
            log_max_body_size: 1024,
        };

        // Initialize logging - should return an error
        let result = init_tracing(&config);

        // Assert that we got the expected error type
        assert!(matches!(result, Err(LogInitError::InvalidPath(_))));
    }

    #[test]
    fn test_filter_parse_error() {
        // Test that the FilterParseError variant can be constructed and matched against
        let error = LogInitError::FilterParseError("Test error message".to_string());

        assert!(matches!(error, LogInitError::FilterParseError(_)));

        // Test that the error message is properly formatted
        if let LogInitError::FilterParseError(msg) = error {
            assert_eq!(msg, "Test error message");
        }

        // Test the Display implementation
        assert_eq!(
            format!(
                "{}",
                LogInitError::FilterParseError("Test error".to_string())
            ),
            "Failed to parse log level filter: Test error"
        );
    }
}
