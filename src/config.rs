//! Configuration management for the Switchboard application
//!
//! This module provides a centralized configuration system that supports:
//! - Loading values from environment variables with sensible defaults
//! - Thread-safe global configuration singleton (`CONFIG`)
//! - Typed configuration values with appropriate conversions
//! - Environment-aware log directory structure
//!
//! # Configuration Defaults
//!
//! All default values are defined as constants in this module:
//! - `DEFAULT_PORT` - HTTP listen port (8080)
//! - `DEFAULT_ANTHROPIC_TARGET_URL` - API endpoint (<https://api.anthropic.com>)
//! - `DEFAULT_LOG_STDOUT_LEVEL` - Console logging level (info)
//! - `DEFAULT_LOG_FILE_LEVEL` - File logging level (debug)
//! - `DEFAULT_LOG_FORMAT` - Log format (pretty or json)
//! - `DEFAULT_LOG_BODIES` - Whether to log request/response bodies
//! - `DEFAULT_LOG_FILE_PATH` - Default log file path
//! - `DEFAULT_LOG_MAX_BODY_SIZE` - Maximum log size for bodies
//! - `DEFAULT_LOG_DIRECTORY_MODE` - Permissions for log directories on Unix
//! - `DEFAULT_LOG_MAX_AGE_DAYS` - How long to retain logs (None = indefinite)
//!
//! # Usage
//!
//! The recommended way to access configuration is through the global singleton:
//!
//! ```rust
//! use switchboard::config;
//!
//! // Load configuration (only needed once at startup)
//! let cfg = config::load_config();
//!
//! // Use configuration values
//! println!("Listening on port {}", cfg.port);
//! ```
//!
//! # Environment Variables
//!
//! The following environment variables can be set to override defaults:
//!
//! | Variable | Purpose | Default |
//! |----------|---------|---------|
//! | `PORT` | HTTP server port | 8080 |
//! | `ANTHROPIC_API_KEY` | API key (required) | None |
//! | `ANTHROPIC_TARGET_URL` | API endpoint | <https://api.anthropic.com> |
//! | `OPENAI_API_KEY` | OpenAI API key (required when enabled) | None |
//! | `OPENAI_API_BASE_URL` | OpenAI API endpoint | <https://api.openai.com> |
//! | `OPENAI_ENABLED` | Enable OpenAI integration | false |
//! | `LOG_LEVEL` | Console log level | info |
//! | `LOG_FORMAT` | Log format (pretty/json) | pretty |
//! | `LOG_BODIES` | Log request/response bodies | true |
//! | `LOG_FILE_PATH` | Path to log file | ./switchboard.log |
//! | `LOG_FILE_LEVEL` | File log level | debug |
//! | `LOG_MAX_BODY_SIZE` | Max body size to log | 20480 |
//! | `LOG_DIRECTORY_MODE` | Directory mode | Default |
//! | `LOG_MAX_AGE_DAYS` | Log retention period | None |

use secrecy::SecretString;
use std::env;
use std::sync::OnceLock;
use tracing::{info, warn};

/// Errors that can occur during configuration loading or validation
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // Will be used in subsequent tasks (T012-T015)
pub enum ConfigError {
    /// The required ANTHROPIC_API_KEY environment variable is not set
    #[error("ANTHROPIC_API_KEY environment variable must be set")]
    MissingAnthropicApiKey,

    /// OpenAI integration is enabled but the API key is not set
    #[error("OPENAI_API_KEY must be set when OPENAI_ENABLED is true")]
    MissingOpenAiKey,

    /// A generic required environment variable is missing
    #[error("Required environment variable {0} is not set")]
    MissingRequiredKey(&'static str),

    /// A boolean environment variable has an invalid value
    #[error("Invalid value for boolean environment variable {var}: '{value}'. Expected 'true', 'false', '1', or '0'")]
    InvalidBooleanValue { var: String, value: String },

    /// A numeric environment variable has an invalid value
    #[error("Failed to parse numeric environment variable {var}: '{value}'")]
    InvalidNumericValue { var: String, value: String },

    /// An environment variable has an invalid format or value
    #[error("Invalid format for {var}: {reason}")]
    InvalidFormat { var: String, reason: String },
}

// Configuration Default Constants

/// Default HTTP port to listen on (8080)
///
/// A standard non-privileged port commonly used for development web servers
pub const DEFAULT_PORT: &str = "8080";

/// Default URL for Anthropic API (<https://api.anthropic.com>)
///
/// The official endpoint for Anthropic's API services
pub const DEFAULT_ANTHROPIC_TARGET_URL: &str = "https://api.anthropic.com";

/// Default URL for OpenAI API (<https://api.openai.com>)
///
/// The official endpoint for OpenAI's API services
pub const DEFAULT_OPENAI_TARGET_URL: &str = "https://api.openai.com";

/// Default log level for stdout (info)
///
/// INFO provides sufficient operational detail without overwhelming output in normal operation
pub const DEFAULT_LOG_STDOUT_LEVEL: &str = "info";

/// Default log level for file output (debug)
///
/// DEBUG provides more verbose logs to file for detailed troubleshooting when needed
pub const DEFAULT_LOG_FILE_LEVEL: &str = "debug";

/// Default log format (pretty)
///
/// Human-readable format for development; can be switched to 'json' for production
pub const DEFAULT_LOG_FORMAT: &str = "pretty";

/// Whether to log full request/response bodies by default (true)
///
/// Enables comprehensive logging of request/response bodies for debugging
pub const DEFAULT_LOG_BODIES: bool = true;

/// Default log file path (./switchboard.log)
///
/// Relative path that works in development environments
pub const DEFAULT_LOG_FILE_PATH: &str = "./switchboard.log";

/// Default maximum body size to log in bytes (20KB)
///
/// Prevents excessive log file growth while retaining meaningful content
pub const DEFAULT_LOG_MAX_BODY_SIZE: usize = 20480;

/// Default directory permission mode on Unix-like systems (0o750)
///
/// Provides owner read/write/execute, group read/execute, and no permissions for others
/// This balances security with necessary access for the application
pub const DEFAULT_LOG_DIRECTORY_MODE: u32 = 0o750;

/// Default maximum age for log files in days (None = no cleanup)
///
/// By default, no automatic log cleanup is performed
pub const DEFAULT_LOG_MAX_AGE_DAYS: Option<u32> = None;

/// Default value for OpenAI integration enablement (false)
///
/// OpenAI integration is disabled by default, requiring explicit opt-in
pub const DEFAULT_OPENAI_ENABLED: bool = false;

/// Parse a boolean environment variable with standardized behavior
///
/// Reads an environment variable and normalizes its value:
/// - Treats "true" and "1" (case-insensitive) as `true`
/// - Treats "false" and "0" (case-insensitive) as `false`
/// - Logs a warning and returns the default for any other value or if unset
///
/// # Arguments
/// * `var_name` - The name of the environment variable to read
/// * `default` - The default value to use if the variable is unset or invalid
///
/// # Returns
/// The parsed boolean value
pub fn parse_bool_env(var_name: &str, default: bool) -> bool {
    match env::var(var_name) {
        Ok(value) => {
            let lowercase_value = value.to_lowercase();
            if lowercase_value == "true" || value == "1" {
                true
            } else if lowercase_value == "false" || value == "0" {
                false
            } else {
                warn!(
                    var = var_name,
                    value = %value,
                    default = default,
                    "Invalid value for environment variable {}: '{}'. Using default value.",
                    var_name, value
                );
                default
            }
        }
        Err(_) => default, // Variable not set, use default
    }
}

/// Specifies how log directory should be determined
///
/// This enum controls how the application selects the base directory for logs,
/// allowing for different deployment scenarios (development, user installation,
/// system service).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogDirectoryMode {
    /// Automatically determine the log directory based on environment detection
    ///
    /// - Development: Uses "./logs/" (DEFAULT_LOG_DIR)
    /// - User Installation: Uses XDG-compliant directory
    /// - System Service: Uses system log path (/var/log/switchboard on Unix)
    #[default]
    Default,

    /// Forces use of XDG Base Directory specification
    ///
    /// Uses platform-specific user data directories:
    /// - Linux: ~/.local/share/switchboard/logs
    /// - macOS: ~/Library/Application Support/switchboard/logs
    /// - Windows: C:\Users\<user>\AppData\Roaming\switchboard\logs
    Xdg,

    /// Forces use of system log directories
    ///
    /// Uses system-level log directories:
    /// - Unix-like: /var/log/switchboard
    /// - Windows: C:\ProgramData\Switchboard\Logs
    System,
}

/// Configuration for the application
///
/// Holds all the configuration values needed by the application,
/// loaded from environment variables with sensible defaults.
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP port to listen on
    pub port: String,

    // Anthropic API configuration
    /// API key for authenticating with Anthropic API
    pub anthropic_api_key: SecretString,
    /// Target URL for the Anthropic API
    pub anthropic_target_url: String,

    // OpenAI API configuration
    /// API key for authenticating with OpenAI API (None if disabled)
    #[allow(dead_code)] // Will be used when OpenAI proxy handler is implemented
    pub openai_api_key: Option<SecretString>,
    /// Target URL for the OpenAI API
    pub openai_api_base_url: String,
    /// Whether the OpenAI integration is enabled
    pub openai_enabled: bool,

    // Logging configuration
    /// Minimum log level for stdout output (info, debug, etc.)
    pub log_stdout_level: String,
    /// Format for stdout log output (json or pretty)
    pub log_format: String,
    /// Whether to log full request and response bodies
    pub log_bodies: bool,
    /// Path to the comprehensive log file
    pub log_file_path: String,
    /// Minimum log level for file output (debug, trace, etc.)
    pub log_file_level: String,
    /// Maximum size for logged bodies before truncation (bytes)
    pub log_max_body_size: usize,
    /// How to determine the log directory (Default|XDG|System)
    pub log_directory_mode: LogDirectoryMode,
    /// Maximum age for log files before cleanup (days)
    /// When set to Some(days), logs older than this will be deleted automatically in development
    /// When set to None (default), no automatic cleanup occurs
    pub log_max_age_days: Option<u32>,
}

/// Default implementation for Config
///
/// Provides sensible defaults for a Config instance.
/// Note: anthropic_api_key will be an empty string in the default implementation
/// and needs to be set explicitly for API requests to work.
impl Default for Config {
    fn default() -> Self {
        Config {
            port: DEFAULT_PORT.to_string(),

            // Anthropic API defaults
            anthropic_api_key: SecretString::new("".to_string().into()),
            anthropic_target_url: DEFAULT_ANTHROPIC_TARGET_URL.to_string(),

            // OpenAI API defaults
            openai_api_key: None,
            openai_api_base_url: DEFAULT_OPENAI_TARGET_URL.to_string(),
            openai_enabled: DEFAULT_OPENAI_ENABLED,

            // Logging defaults
            log_stdout_level: DEFAULT_LOG_STDOUT_LEVEL.to_string(),
            log_format: DEFAULT_LOG_FORMAT.to_string(),
            log_bodies: DEFAULT_LOG_BODIES,
            log_file_path: DEFAULT_LOG_FILE_PATH.to_string(),
            log_file_level: DEFAULT_LOG_FILE_LEVEL.to_string(),
            log_max_body_size: DEFAULT_LOG_MAX_BODY_SIZE,
            log_directory_mode: LogDirectoryMode::Default,
            log_max_age_days: DEFAULT_LOG_MAX_AGE_DAYS,
        }
    }
}

/// Global static configuration instance, initialized once on first access
///
/// Uses OnceLock for thread-safe lazy initialization
pub static CONFIG: OnceLock<Config> = OnceLock::new();

/// Load application configuration from environment variables
///
/// This function will:
/// 1. Load variables from .env file if present
/// 2. Read configuration values from environment variables
/// 3. Use sensible defaults for missing optional values
/// 4. Require ANTHROPIC_API_KEY to be present (panics if missing)
///
/// Returns a reference to the global static Config instance
pub fn load_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        // Load .env file if present (ignore errors if not found)
        dotenvy::dotenv().ok();
        info!("Loading configuration from environment...");

        // Load configuration values with sensible defaults
        let port = env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());

        // API key is mandatory
        let anthropic_api_key =
            env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set for forwarding");

        let anthropic_target_url = env::var("ANTHROPIC_TARGET_URL")
            .unwrap_or_else(|_| DEFAULT_ANTHROPIC_TARGET_URL.to_string());

        // Load OpenAI configuration
        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let openai_api_base_url = env::var("OPENAI_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_OPENAI_TARGET_URL.to_string());

        // Parse OPENAI_ENABLED using standardized helper
        let openai_enabled = parse_bool_env("OPENAI_ENABLED", DEFAULT_OPENAI_ENABLED);

        let log_stdout_level =
            env::var("LOG_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_STDOUT_LEVEL.to_string());
        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| DEFAULT_LOG_FORMAT.to_string());

        // Parse LOG_BODIES using standardized helper
        let log_bodies = parse_bool_env("LOG_BODIES", DEFAULT_LOG_BODIES);

        // Load file logging configuration
        let log_file_path =
            env::var("LOG_FILE_PATH").unwrap_or_else(|_| DEFAULT_LOG_FILE_PATH.to_string());
        let log_file_level =
            env::var("LOG_FILE_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_FILE_LEVEL.to_string());

        // Parse LOG_MAX_BODY_SIZE with error handling
        let log_max_body_size = env::var("LOG_MAX_BODY_SIZE")
            .ok()
            .and_then(|size_str| {
                size_str.parse::<usize>().ok().or_else(|| {
                    warn!(
                        var = "LOG_MAX_BODY_SIZE",
                        value = %size_str,
                        default = DEFAULT_LOG_MAX_BODY_SIZE,
                        "Failed to parse numeric environment variable, using default"
                    );
                    None
                })
            })
            .unwrap_or(DEFAULT_LOG_MAX_BODY_SIZE); // Default if not set or invalid

        // Parse LOG_DIRECTORY_MODE environment variable
        let log_directory_mode = env::var("LOG_DIRECTORY_MODE")
            .map(|mode| match mode.to_lowercase().as_str() {
                "xdg" => LogDirectoryMode::Xdg,
                "system" => LogDirectoryMode::System,
                _ => LogDirectoryMode::Default,
            })
            .unwrap_or(LogDirectoryMode::Default);

        // Parse LOG_MAX_AGE_DAYS with error handling
        let log_max_age_days = env::var("LOG_MAX_AGE_DAYS").ok().and_then(|days_str| {
            days_str.parse::<u32>().ok().or_else(|| {
                // Format default value for human-readable log message
                let default_display = match DEFAULT_LOG_MAX_AGE_DAYS {
                    Some(days) => days.to_string(),
                    None => "no cleanup".to_string(),
                };

                warn!(
                    var = "LOG_MAX_AGE_DAYS",
                    value = %days_str,
                    default = ?DEFAULT_LOG_MAX_AGE_DAYS,
                    default_display = %default_display,
                    "Failed to parse numeric environment variable, using default"
                );
                None
            })
        });

        // Validate OpenAI configuration - if enabled, API key must be provided
        if openai_enabled && openai_api_key.is_none() {
            panic!("OPENAI_API_KEY must be set when OPENAI_ENABLED is true");
        }

        let loaded_config = Config {
            port,
            anthropic_api_key: SecretString::new(anthropic_api_key.into()),
            anthropic_target_url,

            // Use the loaded OpenAI configuration values
            openai_api_key: openai_api_key.map(|key| SecretString::new(key.into())),
            openai_api_base_url,
            openai_enabled,

            log_stdout_level,
            log_format,
            log_bodies,
            log_file_path,
            log_file_level,
            log_max_body_size,
            log_directory_mode,
            log_max_age_days,
        };

        // Log configuration values, but omit the API keys for security
        info!(
            port = %loaded_config.port,
            anthropic_target_url = %loaded_config.anthropic_target_url,
            openai_target_url = %loaded_config.openai_api_base_url,
            openai_enabled = loaded_config.openai_enabled,
            log_stdout_level = %loaded_config.log_stdout_level,
            log_format = %loaded_config.log_format,
            log_bodies = loaded_config.log_bodies,
            log_file_path = %loaded_config.log_file_path,
            log_file_level = %loaded_config.log_file_level,
            log_max_body_size = loaded_config.log_max_body_size,
            log_directory_mode = ?loaded_config.log_directory_mode,
            log_max_age_days = ?loaded_config.log_max_age_days,
            "Configuration loaded"
        );

        loaded_config
    })
}

#[cfg(test)]
mod tests {
    // Tests have been moved to tests/config_test.rs
}
