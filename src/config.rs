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
//! ```rust,no_run
//! use switchboard::config;
//!
//! // Load configuration (only needed once at startup)
//! let cfg = match config::load_config() {
//!     Ok(config) => {
//!         // Store the config globally (usually done in main.rs)
//!         config::set_global_config(config.clone()).expect("Failed to set global config");
//!         config
//!     },
//!     Err(e) => panic!("Configuration error: {}", e),
//! };
//!
//! // Use configuration values directly
//! println!("Listening on port {}", cfg.port);
//!
//! // Or access via the global getter after initialization
//! match config::get_config() {
//!     Ok(config) => println!("Listening on port {}", config.port),
//!     Err(e) => eprintln!("Configuration error: {}", e),
//! }
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
use tracing::info;

/// Errors that can occur during configuration loading or validation
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // Some variants will be used in future tasks (T013-T015)
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

    /// An environment variable has an empty string value
    #[error("Environment variable {0} cannot be empty")]
    EmptyValue(&'static str),

    /// A numeric environment variable has an invalid value
    #[error("Failed to parse numeric environment variable {var}: '{value}'")]
    InvalidNumericValue { var: String, value: String },

    /// An environment variable has an invalid format or value
    #[error("Invalid format for {var}: {reason}")]
    InvalidFormat { var: String, reason: String },

    /// The configuration has already been initialized
    #[error("Configuration has already been initialized")]
    AlreadyInitialized,

    /// The configuration has not been initialized
    #[error(
        "Configuration has not been initialized. Call load_config and set_global_config in main."
    )]
    NotInitialized,
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
/// - Returns an error for any other value
/// - Returns the default if the variable is not set
///
/// # Arguments
/// * `var_name` - The name of the environment variable to read
/// * `default` - The default value to use if the variable is unset
///
/// # Returns
/// Result containing the parsed boolean or an error if the value is invalid
pub fn parse_bool_env(var_name: &str, default: bool) -> Result<bool, ConfigError> {
    match env::var(var_name) {
        Ok(value) => {
            let lowercase_value = value.to_lowercase();
            if lowercase_value == "true" || value == "1" {
                Ok(true)
            } else if lowercase_value == "false" || value == "0" {
                Ok(false)
            } else {
                Err(ConfigError::InvalidBooleanValue {
                    var: var_name.to_string(),
                    value,
                })
            }
        }
        Err(_) => Ok(default), // Variable not set, use default
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
/// 4. Validate the configuration (required keys, value formats, etc.)
///
/// Returns a Result containing the loaded Config or a ConfigError
pub fn load_config() -> Result<Config, ConfigError> {
    // Load .env file if present (ignore errors if not found)
    dotenvy::dotenv().ok();
    info!("Loading configuration from environment...");

    // Load configuration values with sensible defaults
    let port = match env::var("PORT") {
        Ok(port_str) => {
            // Validate that it's a valid port number
            if port_str.parse::<u16>().is_err() {
                return Err(ConfigError::InvalidNumericValue {
                    var: "PORT".to_string(),
                    value: port_str,
                });
            }
            port_str
        }
        Err(_) => DEFAULT_PORT.to_string(),
    };

    // API key is mandatory
    let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| ConfigError::MissingRequiredKey("ANTHROPIC_API_KEY"))?;

    if anthropic_api_key.is_empty() {
        return Err(ConfigError::EmptyValue("ANTHROPIC_API_KEY"));
    }

    let anthropic_target_url = match env::var("ANTHROPIC_TARGET_URL") {
        Ok(url) => {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(ConfigError::InvalidFormat {
                    var: "ANTHROPIC_TARGET_URL".to_string(),
                    reason: "URL must start with 'http://' or 'https://'".to_string(),
                });
            }
            url
        }
        Err(_) => DEFAULT_ANTHROPIC_TARGET_URL.to_string(),
    };

    // Load OpenAI configuration
    let openai_api_key = env::var("OPENAI_API_KEY").ok();
    let openai_api_base_url = match env::var("OPENAI_API_BASE_URL") {
        Ok(url) => {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(ConfigError::InvalidFormat {
                    var: "OPENAI_API_BASE_URL".to_string(),
                    reason: "URL must start with 'http://' or 'https://'".to_string(),
                });
            }
            url
        }
        Err(_) => DEFAULT_OPENAI_TARGET_URL.to_string(),
    };

    // Parse OPENAI_ENABLED using standardized helper
    let openai_enabled = parse_bool_env("OPENAI_ENABLED", DEFAULT_OPENAI_ENABLED)?;

    let log_stdout_level = match env::var("LOG_LEVEL") {
        Ok(level) => match level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => level,
            _ => {
                return Err(ConfigError::InvalidFormat {
                        var: "LOG_LEVEL".to_string(),
                        reason: format!("Invalid log level '{}'. Expected 'trace', 'debug', 'info', 'warn', or 'error'", level),
                    });
            }
        },
        Err(_) => DEFAULT_LOG_STDOUT_LEVEL.to_string(),
    };
    let log_format = match env::var("LOG_FORMAT") {
        Ok(format) => match format.to_lowercase().as_str() {
            "pretty" | "json" => format,
            _ => {
                return Err(ConfigError::InvalidFormat {
                    var: "LOG_FORMAT".to_string(),
                    reason: format!(
                        "Invalid log format '{}'. Expected 'pretty' or 'json'",
                        format
                    ),
                });
            }
        },
        Err(_) => DEFAULT_LOG_FORMAT.to_string(),
    };

    // Parse LOG_BODIES using standardized helper
    let log_bodies = parse_bool_env("LOG_BODIES", DEFAULT_LOG_BODIES)?;

    // Load file logging configuration
    let log_file_path =
        env::var("LOG_FILE_PATH").unwrap_or_else(|_| DEFAULT_LOG_FILE_PATH.to_string());
    let log_file_level = match env::var("LOG_FILE_LEVEL") {
        Ok(level) => match level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => level,
            _ => {
                return Err(ConfigError::InvalidFormat {
                        var: "LOG_FILE_LEVEL".to_string(),
                        reason: format!("Invalid log level '{}'. Expected 'trace', 'debug', 'info', 'warn', or 'error'", level),
                    });
            }
        },
        Err(_) => DEFAULT_LOG_FILE_LEVEL.to_string(),
    };

    // Parse LOG_MAX_BODY_SIZE with error handling
    let log_max_body_size = match env::var("LOG_MAX_BODY_SIZE") {
        Ok(size_str) => match size_str.parse::<usize>() {
            Ok(size) => size,
            Err(_) => {
                return Err(ConfigError::InvalidNumericValue {
                    var: "LOG_MAX_BODY_SIZE".to_string(),
                    value: size_str,
                });
            }
        },
        Err(_) => DEFAULT_LOG_MAX_BODY_SIZE, // Default if not set
    };

    // Parse LOG_DIRECTORY_MODE environment variable
    let log_directory_mode = match env::var("LOG_DIRECTORY_MODE") {
        Ok(mode) => match mode.to_lowercase().as_str() {
            "xdg" => LogDirectoryMode::Xdg,
            "system" => LogDirectoryMode::System,
            "default" => LogDirectoryMode::Default,
            _ => {
                return Err(ConfigError::InvalidFormat {
                    var: "LOG_DIRECTORY_MODE".to_string(),
                    reason: format!(
                        "Invalid value '{}'. Expected 'xdg', 'system', or 'default'",
                        mode
                    ),
                });
            }
        },
        Err(_) => LogDirectoryMode::Default, // Default if not set
    };

    // Parse LOG_MAX_AGE_DAYS with error handling
    let log_max_age_days = match env::var("LOG_MAX_AGE_DAYS") {
        Ok(days_str) => match days_str.parse::<u32>() {
            Ok(days) => Some(days),
            Err(_) => {
                return Err(ConfigError::InvalidNumericValue {
                    var: "LOG_MAX_AGE_DAYS".to_string(),
                    value: days_str,
                });
            }
        },
        Err(_) => DEFAULT_LOG_MAX_AGE_DAYS, // Default if not set
    };

    // Validate OpenAI configuration - if enabled, API key must be provided
    if openai_enabled && openai_api_key.is_none() {
        return Err(ConfigError::MissingOpenAiKey);
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

    Ok(loaded_config)
}

/// Sets the global configuration instance for the application.
/// This should be called once at application startup after successfully loading configuration.
///
/// # Arguments
/// * `config` - The validated configuration instance to set globally
///
/// # Returns
/// * `Ok(())` if successfully set, or
/// * `Err(ConfigError::AlreadyInitialized)` if the global config has already been set
pub fn set_global_config(config: Config) -> Result<(), ConfigError> {
    CONFIG
        .set(config)
        .map_err(|_| ConfigError::AlreadyInitialized)
}

/// Returns a reference to the global configuration instance.
///
/// # Returns
/// * `Ok(&'static Config)` if the configuration has been initialized
/// * `Err(ConfigError::NotInitialized)` if the configuration has not been initialized
///
/// This allows for proper error handling rather than panicking when configuration
/// is not available. Call sites should handle this error appropriately.
#[allow(dead_code)] // Will be used in future tasks
pub fn get_config() -> Result<&'static Config, ConfigError> {
    CONFIG.get().ok_or(ConfigError::NotInitialized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_uninitialized() {
        // This test should be run in isolation to ensure CONFIG is not initialized
        // by other tests. In a real test environment, this would be ensured by
        // using #[serial] attribute, but since this is a unit test within the same
        // module as the static CONFIG, we can test it directly.

        // The test simply verifies that get_config returns NotInitialized error
        // when the CONFIG has not been set.
        let result = get_config();

        // Verify the result is an Err variant with the correct error type
        assert!(result.is_err());
        match result {
            Err(ConfigError::NotInitialized) => {
                // This is the expected error variant
            }
            Err(e) => panic!("Wrong error variant returned: {:?}", e),
            Ok(_) => panic!("Expected Err but got Ok"),
        }
    }
}
