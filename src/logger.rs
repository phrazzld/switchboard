//! Logging configuration and setup
//!
//! This module handles the initialization and configuration of the tracing system
//! used for structured logging throughout the application with support for
//! dual outputs (file and stdout).

use crate::config::Config;
use std::io;
use std::path::Path;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};

/// Initialize the tracing system for structured logging with dual output
///
/// Sets up the tracing subscriber with two output layers:
/// 1. File: JSON-formatted logs with level filtering by `log_file_level`, using
///    non-blocking I/O and daily rotation.
/// 2. Stdout: Configurable format (pretty or JSON) with level filtering by `log_stdout_level`.
///
/// Returns a `WorkerGuard` which must be kept alive for the non-blocking file appender
/// to flush logs on shutdown.
///
/// # Arguments
/// * `config` - The application configuration containing logging settings
///
/// # Examples
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
/// let _guard = logger::init_tracing(&mock_config);
/// // The guard must be kept alive for the duration of the application
/// // to ensure logs are flushed properly on shutdown
/// ```
pub fn init_tracing(config: &Config) -> WorkerGuard {
    // Parse log file path to get directory and filename
    let log_file_path = Path::new(&config.log_file_path);
    let log_dir = log_file_path.parent().unwrap_or_else(|| Path::new("."));
    let log_file_name = log_file_path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("switchboard.log"));

    // Create directory if it doesn't exist
    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            eprintln!(
                "Failed to create log directory {}: {}",
                log_dir.display(),
                e
            );
        }
    }

    // Create daily rotating file appender
    let file_appender = rolling::daily(log_dir, log_file_name);

    // Create non-blocking writer and get the guard
    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Create file filter based on config.log_file_level
    let file_filter = EnvFilter::try_new(&config.log_file_level).unwrap_or_else(|e| {
        eprintln!(
            "Failed to parse file log level filter: {}, using default 'debug'",
            e
        );
        EnvFilter::new("debug") // Fallback to debug
    });

    // Create file layer with JSON formatting
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking_writer)
        .with_filter(file_filter);

    // Create stdout filter based on RUST_LOG or config.log_stdout_level
    let stdout_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_stdout_level))
        .unwrap_or_else(|e| {
            eprintln!(
                "Failed to parse stdout log level filter: {}, using default 'info'",
                e
            );
            EnvFilter::new("info") // Fallback to info
        });

    // We'll apply the stdout layer based on format later

    // Create registry and add file layer
    let subscriber = registry().with(file_layer);

    // Add the appropriate stdout layer based on format
    if config.log_format == "json" {
        let json_layer = fmt::layer()
            .json()
            .with_writer(io::stdout)
            .with_filter(stdout_filter);
        subscriber.with(json_layer).init();
    } else {
        let pretty_layer = fmt::layer()
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
    guard
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
        let _guard = init_tracing(&config);

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
}
