use std::io;
use thiserror::Error;

/// Error type for logging initialization failures
///
/// This enum represents all possible error conditions that can occur during log initialization,
/// including file system errors, permission issues, path validation problems, and configuration
/// parsing errors.
///
/// Each variant includes detailed information about the specific error condition, allowing
/// for proper error handling and reporting throughout the application.
#[derive(Debug, Error)]
#[allow(dead_code)]
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

    /// Permission denied when accessing a path
    #[error("Permission denied when accessing {path}: {source}")]
    PermissionDenied {
        /// Path to which permission was denied
        path: String,
        /// Source IO error
        #[source]
        source: io::Error,
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

    /// Generic I/O error during log initialization
    #[error("I/O error during log initialization: {0}")]
    IoError(#[from] io::Error),
}
