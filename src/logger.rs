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

use crate::config::Config;
use std::io;
#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
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

    /// Path traversal attempt detected in log path
    #[error("Path traversal attempt detected in log path: {0}")]
    PathTraversalAttempt(String),

    /// Reserved system path not allowed for logging
    #[error("Reserved system path not allowed for logging: {0}")]
    ReservedSystemPath(String),

    /// Permission issue with log directory
    #[error("Permission issue with log directory at {path}: {reason}")]
    PermissionIssue {
        /// Path to the directory with permission issues
        path: String,
        /// Reason for the permission issue
        reason: String,
    },

    /// Path canonicalization error
    #[error("Failed to canonicalize path {path}: {source}")]
    PathCanonicalizationError {
        /// Path that could not be canonicalized
        path: String,
        /// Source IO error
        #[source]
        source: io::Error,
    },
}

/// Validate a log file path for security and usability concerns
///
/// This function performs comprehensive validation on a log file path to ensure it:
/// - Does not contain path traversal attempts
/// - Is not a reserved system path
/// - Has appropriate permissions for logging
/// - Can be canonicalized to an absolute path
///
/// # Arguments
/// * `path_str` - The string path to validate
///
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized absolute path if validation passes
/// * `Err(LogInitError)` - Specific error if validation fails
///
/// # Security Considerations
/// This validation helps prevent common security issues:
/// - Path traversal: Prevents writing to unexpected locations via `../` sequences
/// - System path protection: Prevents overwriting system files in locations like `/bin`, `/etc`, etc.
/// - Permission verification: Ensures logs can actually be written
///
/// # Errors
/// Returns detailed error types for different validation failures:
/// - `LogInitError::InvalidPath` - Path is empty or missing a filename component
/// - `LogInitError::PathTraversalAttempt` - Path contains `../` sequences
/// - `LogInitError::ReservedSystemPath` - Path points to a reserved system directory
/// - `LogInitError::PermissionIssue` - Directory exists but is not writable
/// - `LogInitError::PathCanonicalizationError` - Path cannot be canonicalized
///
/// # Examples
///
/// ```
/// # use switchboard::logger::validate_log_path;
/// // Valid path in a temporary directory
/// let valid_path = std::env::temp_dir().join("app.log").to_string_lossy().to_string();
/// assert!(validate_log_path(&valid_path).is_ok());
///
/// // Invalid path (path traversal attempt)
/// let invalid_path = "../etc/passwd";
/// assert!(validate_log_path(invalid_path).is_err());
/// ```
pub fn validate_log_path(path_str: &str) -> Result<PathBuf, LogInitError> {
    // Check for empty path
    if path_str.is_empty() {
        return Err(LogInitError::InvalidPath(
            "Log file path cannot be empty".to_string(),
        ));
    }

    let path = Path::new(path_str);

    // Check if path contains parent directory components that could lead to traversal
    if path_str.contains("../") || path_str.contains("..\\") {
        return Err(LogInitError::PathTraversalAttempt(format!(
            "Path contains parent directory traversal sequences: {}",
            path_str
        )));
    }

    // Check if filename is present
    // Handle the case of a directory path ending with / or \
    if path_str.ends_with('/') || path_str.ends_with('\\') {
        return Err(LogInitError::InvalidPath(format!(
            "Invalid log file path (ends with directory separator): {}",
            path_str
        )));
    }

    let file_name = match path.file_name() {
        Some(name) => name,
        None => {
            return Err(LogInitError::InvalidPath(format!(
                "Invalid log file path (no filename): {}",
                path_str
            )));
        }
    };

    // Get directory component
    let dir_path = path.parent().unwrap_or_else(|| Path::new("."));

    // Reserved system paths that should not be used for logging
    let reserved_paths = [
        #[cfg(target_family = "unix")]
        "/bin",
        #[cfg(target_family = "unix")]
        "/sbin",
        #[cfg(target_family = "unix")]
        "/usr/bin",
        #[cfg(target_family = "unix")]
        "/usr/sbin",
        #[cfg(target_family = "unix")]
        "/etc",
        #[cfg(target_family = "unix")]
        "/dev",
        #[cfg(target_family = "unix")]
        "/proc",
        #[cfg(target_family = "unix")]
        "/sys",
        #[cfg(target_family = "windows")]
        "C:\\Windows",
        #[cfg(target_family = "windows")]
        "C:\\Program Files",
        #[cfg(target_family = "windows")]
        "C:\\Program Files (x86)",
    ];

    // Try to canonicalize the path to resolve any . or .. components
    let canonical_path = match std::fs::canonicalize(dir_path) {
        Ok(p) => p,
        Err(e) => {
            // If the directory doesn't exist yet, that's ok - we'll create it later
            // Only return an error if it's not a NotFound error
            if e.kind() != io::ErrorKind::NotFound {
                return Err(LogInitError::PathCanonicalizationError {
                    path: dir_path.display().to_string(),
                    source: e,
                });
            }
            // For NotFound, we'll continue with the original path
            dir_path.to_path_buf()
        }
    };

    // Check if the path (without canonicalization) is a reserved system path
    // This is to catch paths that might not be able to be canonicalized
    let path_to_check = if canonical_path == dir_path.to_path_buf() {
        // If canonicalization didn't work (e.g., directory doesn't exist yet),
        // check the original path against reserved paths
        path_str.to_string()
    } else {
        // Otherwise, check the canonical path
        canonical_path.to_string_lossy().to_string()
    };

    for reserved in &reserved_paths {
        if path_to_check.starts_with(reserved) {
            return Err(LogInitError::ReservedSystemPath(format!(
                "Path '{}' is within a reserved system directory: {}",
                path_str, reserved
            )));
        }
    }

    // If directory exists, check permissions
    if dir_path.exists() {
        // Try to check if we can write to this directory
        match std::fs::metadata(dir_path) {
            Ok(metadata) => {
                #[cfg(target_family = "unix")]
                {
                    // On Unix systems, check if directory is writable by the current user
                    let dir_mode = metadata.mode();
                    let uid = unsafe { libc::getuid() };
                    let gid = unsafe { libc::getgid() };

                    let owner_writable = (dir_mode & 0o200) != 0; // Check owner write permission
                    let group_writable = (dir_mode & 0o020) != 0; // Check group write permission
                    let world_writable = (dir_mode & 0o002) != 0; // Check world write permission

                    let is_owner = metadata.uid() == uid;
                    let is_group = metadata.gid() == gid;

                    let is_writable = (is_owner && owner_writable)
                        || (is_group && group_writable)
                        || world_writable;

                    if !is_writable {
                        return Err(LogInitError::PermissionIssue {
                            path: dir_path.display().to_string(),
                            reason: "Directory is not writable by the current user".to_string(),
                        });
                    }
                }

                // For other platforms, we'll try a more basic approach
                #[cfg(not(target_family = "unix"))]
                {
                    // Check if readonly
                    if metadata.permissions().readonly() {
                        return Err(LogInitError::PermissionIssue {
                            path: dir_path.display().to_string(),
                            reason: "Directory is read-only".to_string(),
                        });
                    }
                }
            }
            Err(e) => {
                // If we can't read metadata, that might be a permissions issue
                return Err(LogInitError::PermissionIssue {
                    path: dir_path.display().to_string(),
                    reason: format!("Failed to read directory metadata: {}", e),
                });
            }
        }
    }

    // Construct a canonicalized path for the full file path
    let mut result_path = canonical_path;
    result_path.push(file_name);

    Ok(result_path)
}

/// Initialize the tracing system for structured logging with dual output
///
/// Sets up the tracing subscriber with two output layers:
/// 1. **File Layer**: JSON-formatted logs with level filtering by `log_file_level`, using
///    non-blocking I/O and daily rotation.
/// 2. **Stdout Layer**: Configurable format (pretty or JSON) with level filtering by `log_stdout_level`.
///
/// This function handles:
/// - Validating log file paths for security (preventing path traversal, etc.)
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
    // Validate log file path with comprehensive security checks
    let validated_path = validate_log_path(&config.log_file_path)?;

    // Extract directory and filename from the validated path
    let log_dir = validated_path.parent().unwrap_or_else(|| Path::new("."));
    let log_file_name = validated_path.file_name().unwrap(); // Safe because validate_log_path guarantees a filename

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
    use std::env;
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
        // Skip this test on platforms where /proc is not prohibited
        // The path validation would now detect this as a reserved path,
        // so we would get a ReservedSystemPath error before reaching directory creation
        #[cfg(target_family = "unix")]
        {
            // Create a configuration with a path to a location that cannot be created
            let config = Config {
                port: "0".to_string(),
                anthropic_api_key: "test-key".to_string(),
                anthropic_target_url: "https://example.com".to_string(),
                log_stdout_level: "info".to_string(),
                log_format: "pretty".to_string(),
                log_bodies: true,
                // Use a directory that exists but we won't have write permissions to
                // (assuming standard permissions, will need a different approach if running as root)
                log_file_path: "/var/lib/non_existent_dir/test.log".to_string(),
                log_file_level: "debug".to_string(),
                log_max_body_size: 1024,
            };

            // Initialize logging - should return an error
            let result = init_tracing(&config);

            // We'll get either a reserved path error, permission issue, or directory creation failed
            // Any of these are acceptable, based on the exact system permissions
            assert!(matches!(
                result,
                Err(LogInitError::ReservedSystemPath(_))
                    | Err(LogInitError::PermissionIssue { .. })
                    | Err(LogInitError::DirectoryCreationFailed { .. })
            ));
        }

        // On Windows, use a different approach
        #[cfg(target_family = "windows")]
        {
            let config = Config {
                port: "0".to_string(),
                anthropic_api_key: "test-key".to_string(),
                anthropic_target_url: "https://example.com".to_string(),
                log_stdout_level: "info".to_string(),
                log_format: "pretty".to_string(),
                log_bodies: true,
                // Windows equivalent
                log_file_path: "C:\\Windows\\System32\\invalid_dir\\test.log".to_string(),
                log_file_level: "debug".to_string(),
                log_max_body_size: 1024,
            };

            let result = init_tracing(&config);

            assert!(matches!(
                result,
                Err(LogInitError::ReservedSystemPath(_))
                    | Err(LogInitError::PermissionIssue { .. })
                    | Err(LogInitError::DirectoryCreationFailed { .. })
            ));
        }
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

    // Tests for the path validation function

    #[test]
    fn test_path_validation_success() {
        // Create a temporary directory for testing
        let temp_dir = env::temp_dir().join("switchboard_path_test");
        fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

        // Typical valid paths that should pass validation
        let valid_paths = [
            temp_dir.join("simple.log").to_string_lossy().to_string(),
            temp_dir
                .join("nested/path/file.log")
                .to_string_lossy()
                .to_string(),
            "relative_path.log".to_string(),
            "./local_dir/file.log".to_string(),
        ];

        for path in valid_paths {
            let result = validate_log_path(&path);
            assert!(result.is_ok(), "Path should be valid: {}", path);
        }

        // Clean up
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_path_validation_failures() {
        // Empty path
        assert!(matches!(
            validate_log_path(""),
            Err(LogInitError::InvalidPath(_))
        ));

        // Path traversal attempts
        let traversal_paths = [
            "../etc/passwd",
            "logs/../../../etc/shadow",
            "normal/looking/../../path/file.log",
            "..\\Windows\\System32\\config.sys",
        ];

        for path in traversal_paths {
            let result = validate_log_path(path);
            assert!(
                matches!(result, Err(LogInitError::PathTraversalAttempt(_))),
                "Path traversal should be detected: {}",
                path
            );
        }

        // No filename (just a directory)
        assert!(matches!(
            validate_log_path("/tmp/"),
            Err(LogInitError::InvalidPath(_))
        ));

        // Reserved system paths
        #[cfg(target_family = "unix")]
        {
            // Check if we can actually run the reserved path tests
            // Some CI environments or container setups might not have standard directories
            if Path::new("/bin").exists() && Path::new("/etc").exists() {
                let system_paths = [
                    "/bin/log.txt",
                    "/etc/switchboard.log",
                    "/usr/bin/app.log",
                    "/sbin/temp.log",
                ];

                for path in system_paths {
                    println!("Testing reserved path: {}", path);
                    let result = validate_log_path(path);
                    println!("Result: {:?}", result);

                    // Check for either ReservedSystemPath or PermissionIssue
                    // Either is acceptable, as environments vary
                    assert!(
                        matches!(result, Err(LogInitError::ReservedSystemPath(_)))
                            || matches!(result, Err(LogInitError::PermissionIssue { .. })),
                        "System path should be detected as reserved or permission issue: {}",
                        path
                    );
                }
            }

            // Additional sensitive paths that should always be detected
            let traversal_paths = ["/dev/log.txt", "/proc/logger.log"];

            for path in traversal_paths {
                let result = validate_log_path(path);
                // We'll accept any error here - specific error might depend on system
                assert!(result.is_err(), "System path should be rejected: {}", path);
            }
        }

        #[cfg(target_family = "windows")]
        {
            // Only run these tests if we're on Windows and these directories exist
            if Path::new("C:\\Windows").exists() {
                let system_paths = ["C:\\Windows\\log.txt", "C:\\Program Files\\app.log"];

                for path in system_paths {
                    println!("Testing reserved path: {}", path);
                    let result = validate_log_path(path);
                    println!("Result: {:?}", result);

                    // Check for either ReservedSystemPath or PermissionIssue
                    // Either is acceptable, as environments vary
                    assert!(
                        matches!(result, Err(LogInitError::ReservedSystemPath(_)))
                            || matches!(result, Err(LogInitError::PermissionIssue { .. })),
                        "System path should be detected as reserved or permission issue: {}",
                        path
                    );
                }
            }
        }
    }

    #[test]
    fn test_path_with_root_permission_requirements() {
        // Skip for non-unix as the permission model is different
        #[cfg(target_family = "unix")]
        {
            // Root-owned directories that should fail for non-root users
            // This test assumes it's not being run as root
            let root_paths = [
                // Standard system directories that require root to write to
                "/var/log/switchboard.log",
                "/var/spool/app.log",
            ];

            // Check if we're running as root - if so, skip these tests
            let uid = unsafe { libc::getuid() };
            if uid != 0 {
                for path in root_paths {
                    let result = validate_log_path(path);

                    // Could be either permission issue or reserved path
                    assert!(
                        matches!(
                            result,
                            Err(LogInitError::PermissionIssue { .. })
                                | Err(LogInitError::ReservedSystemPath(_))
                        ),
                        "Path requiring root should be detected: {}",
                        path
                    );
                }
            }
        }
    }
}
