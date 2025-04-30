use crate::config::Config;
use crate::logger::{LogInitError, LogPathResolver, LogType};
use std::io;
use std::path::Path;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt as tracing_fmt, prelude::*, registry, EnvFilter};

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
/// # use switchboard::config::{Config, LogDirectoryMode};
/// # use switchboard::logger;
/// # use secrecy::SecretString;
/// # // Create a mock config for testing instead of using global config
/// # let mock_config = Config {
///     openai_api_key: Some(SecretString::new("test-openai-api-key".to_string().into())),
///     openai_api_base_url: "https://api.openai.com".to_string(),
///     openai_enabled: false,
/// #     port: "8080".to_string(),
/// #     anthropic_api_key: SecretString::new("test-key".to_string().into()),
/// #     anthropic_target_url: "https://example.com".to_string(),
/// #     log_stdout_level: "info".to_string(),
/// #     log_format: "pretty".to_string(),
/// #     log_bodies: true,
/// #     log_file_path: "./switchboard.log".to_string(),
/// #     log_file_level: "debug".to_string(),
/// #     log_max_body_size: 20480,
/// #     log_directory_mode: LogDirectoryMode::Default,
/// #     log_max_age_days: None,
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
/// # use switchboard::config::{Config, LogDirectoryMode};
/// # use switchboard::logger;
/// let config = Config {
///     # openai_api_key: None,
///     # openai_api_base_url: "https://api.openai.com".to_string(),
///     # openai_enabled: false,
///     // ... other fields ...
///     log_stdout_level: "info".to_string(),
///     log_format: "json".to_string(), // Use JSON for stdout too
///     log_file_path: "./logs/switchboard.log".to_string(),
///     log_file_level: "debug".to_string(),
///     // ... other fields ...
///     # port: "8080".to_string(),
///     # anthropic_api_key: secrecy::SecretString::new("test-key".to_string().into()),
///     # anthropic_target_url: "https://example.com".to_string(),
///     # log_bodies: true,
///     # log_max_body_size: 20480,
///     # log_directory_mode: LogDirectoryMode::Default,
///     # log_max_age_days: None,
/// };
///
/// let _guard = logger::init_tracing(&config).expect("Failed to initialize logging");
/// ```
///
/// Different log levels for file and stdout:
/// ```
/// # use switchboard::config::{Config, LogDirectoryMode};
/// # use switchboard::logger;
/// # use secrecy;
/// let config = Config {
///     # openai_api_key: None,
///     # openai_api_base_url: "https://api.openai.com".to_string(),
///     # openai_enabled: false,
///     // ... other fields ...
///     log_stdout_level: "warn".to_string(), // Only warnings and errors go to stdout
///     log_file_path: "./logs/switchboard.log".to_string(),
///     log_file_level: "trace".to_string(),  // Everything goes to the file
///     // ... other fields ...
///     # port: "8080".to_string(),
///     # anthropic_api_key: secrecy::SecretString::new("test-key".to_string().into()),
///     # anthropic_target_url: "https://example.com".to_string(),
///     # log_format: "pretty".to_string(),
///     # log_bodies: true,
///     # log_max_body_size: 20480,
///     # log_directory_mode: LogDirectoryMode::Default,
///     # log_max_age_days: None,
/// };
///
/// let _guard = logger::init_tracing(&config).expect("Failed to initialize logging");
/// ```
pub fn init_tracing(config: &Config) -> Result<WorkerGuard, LogInitError> {
    // Check for empty path before creating resolver
    if config.log_file_path.is_empty() {
        return Err(LogInitError::InvalidPath(
            "Log file path cannot be empty".to_string(),
        ));
    }

    // Check if the original path is in legacy format
    let is_legacy = LogPathResolver::is_legacy_path(&config.log_file_path);

    // Use LogPathResolver to get the correct path based on environment and config
    let resolver = LogPathResolver::new(config, LogType::Application);
    let resolved_path = resolver.resolve()?;

    // Extract directory and filename from the resolved path
    let log_dir = resolved_path.parent().unwrap_or_else(|| Path::new("."));
    let log_file_name = resolved_path.file_name().unwrap();

    // Create daily rotating file appender
    let file_appender = tracing_appender::rolling::daily(log_dir, log_file_name);

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

    // Enhanced initialization log with information about legacy paths
    if is_legacy {
        info!(
            log_stdout_level = %config.log_stdout_level,
            log_format = %config.log_format,
            original_path = %config.log_file_path,
            resolved_path = %resolved_path.display(),
            log_file_level = %config.log_file_level,
            log_directory_mode = ?config.log_directory_mode,
            "Dual logging initialized with legacy path adaptation"
        );
    } else {
        info!(
            log_stdout_level = %config.log_stdout_level,
            log_format = %config.log_format,
            log_file_path = %resolved_path.display(),
            log_file_level = %config.log_file_level,
            log_directory_mode = ?config.log_directory_mode,
            "Dual logging initialized"
        );
    }

    // Return guard to keep it alive
    Ok(guard)
}
