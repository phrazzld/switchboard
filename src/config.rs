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
//! | `LOG_LEVEL` | Console log level | info |
//! | `LOG_FORMAT` | Log format (pretty/json) | pretty |
//! | `LOG_BODIES` | Log request/response bodies | true |
//! | `LOG_FILE_PATH` | Path to log file | ./switchboard.log |
//! | `LOG_FILE_LEVEL` | File log level | debug |
//! | `LOG_MAX_BODY_SIZE` | Max body size to log | 20480 |
//! | `LOG_DIRECTORY_MODE` | Directory mode | Default |
//! | `LOG_MAX_AGE_DAYS` | Log retention period | None |

use std::env;
use std::sync::OnceLock;
use tracing::{info, warn};

// Configuration Default Constants

/// Default HTTP port to listen on (8080)
///
/// A standard non-privileged port commonly used for development web servers
pub const DEFAULT_PORT: &str = "8080";

/// Default URL for Anthropic API (<https://api.anthropic.com>)
///
/// The official endpoint for Anthropic's API services
pub const DEFAULT_ANTHROPIC_TARGET_URL: &str = "https://api.anthropic.com";

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
    /// API key for authenticating with Anthropic API
    pub anthropic_api_key: String,
    /// Target URL for the Anthropic API
    pub anthropic_target_url: String,
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
            anthropic_api_key: "".to_string(),
            anthropic_target_url: DEFAULT_ANTHROPIC_TARGET_URL.to_string(),
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

        let log_stdout_level =
            env::var("LOG_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_STDOUT_LEVEL.to_string());
        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| DEFAULT_LOG_FORMAT.to_string());

        // Parse LOG_BODIES with error handling for non-boolean values
        let log_bodies = match env::var("LOG_BODIES") {
            Ok(value) => {
                // Check if it's a valid boolean representation
                if value.to_lowercase() == "true"
                    || value.to_lowercase() == "false"
                    || value == "0"
                    || value == "1"
                {
                    // Only consider "false" and "0" as false values (maintain existing behavior)
                    value.to_lowercase() != "false" && value != "0"
                } else {
                    // Non-standard boolean value, log a warning
                    warn!(
                        var = "LOG_BODIES",
                        value = %value,
                        default = DEFAULT_LOG_BODIES,
                        "Ambiguous boolean value in environment variable, using default"
                    );
                    DEFAULT_LOG_BODIES
                }
            }
            Err(_) => DEFAULT_LOG_BODIES, // Use default if not set
        };

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

        let loaded_config = Config {
            port,
            anthropic_api_key,
            anthropic_target_url,
            log_stdout_level,
            log_format,
            log_bodies,
            log_file_path,
            log_file_level,
            log_max_body_size,
            log_directory_mode,
            log_max_age_days,
        };

        // Log configuration values, but omit the API key for security
        info!(
            port = %loaded_config.port,
            target_url = %loaded_config.anthropic_target_url,
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
    use super::*;
    use std::collections::HashMap;
    use std::env;
    use std::sync::Mutex;
    use std::sync::Once;

    // Use a mutex to ensure environment variable tests don't interfere with each other
    static ENV_MUTEX: Mutex<()> = Mutex::new(());
    static INIT: Once = Once::new();

    // Initialize test environment exactly once
    fn initialize() {
        INIT.call_once(|| {
            // Initialize test environment here
        });
    }

    // A function to create a test config with specific environment variables
    fn create_test_config_with_env(env_vars: HashMap<&str, &str>) -> Config {
        // Ensure synchronization across tests
        let _lock = ENV_MUTEX.lock().unwrap();
        initialize();

        // Save current environment
        let mut old_vars = HashMap::new();
        for (key, _) in env_vars.iter() {
            old_vars.insert(*key, env::var(*key).ok());
        }

        // Set provided environment variables
        for (key, value) in env_vars.iter() {
            // Only set non-empty environment variables
            if !value.is_empty() {
                env::set_var(key, value);
            } else {
                env::remove_var(key);
            }
        }

        // Create the config (similar to create_test_config but cleaner)
        let port = env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
        let anthropic_api_key =
            env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());
        let anthropic_target_url = env::var("ANTHROPIC_TARGET_URL")
            .unwrap_or_else(|_| DEFAULT_ANTHROPIC_TARGET_URL.to_string());
        let log_stdout_level =
            env::var("LOG_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_STDOUT_LEVEL.to_string());
        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| DEFAULT_LOG_FORMAT.to_string());
        let log_bodies = env::var("LOG_BODIES")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(DEFAULT_LOG_BODIES);
        let log_file_path =
            env::var("LOG_FILE_PATH").unwrap_or_else(|_| DEFAULT_LOG_FILE_PATH.to_string());
        let log_file_level =
            env::var("LOG_FILE_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_FILE_LEVEL.to_string());
        let log_max_body_size = env::var("LOG_MAX_BODY_SIZE")
            .ok()
            .and_then(|size_str| {
                size_str.parse::<usize>().ok().or_else(|| {
                    eprintln!(
                        "Failed to parse LOG_MAX_BODY_SIZE: '{}', using default {}",
                        size_str, DEFAULT_LOG_MAX_BODY_SIZE
                    );
                    None
                })
            })
            .unwrap_or(DEFAULT_LOG_MAX_BODY_SIZE);

        // Parse LOG_DIRECTORY_MODE
        let log_directory_mode = env::var("LOG_DIRECTORY_MODE")
            .map(|mode| match mode.to_lowercase().as_str() {
                "xdg" => LogDirectoryMode::Xdg,
                "system" => LogDirectoryMode::System,
                _ => LogDirectoryMode::Default,
            })
            .unwrap_or(LogDirectoryMode::Default);

        let config = Config {
            port,
            anthropic_api_key,
            anthropic_target_url,
            log_stdout_level,
            log_format,
            log_bodies,
            log_file_path,
            log_file_level,
            log_max_body_size,
            log_directory_mode,
            log_max_age_days: None,
        };

        // Restore old environment
        for (key, value_opt) in old_vars {
            match value_opt {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }

        config
    }

    #[test]
    fn test_default_values() {
        // For default values, we just need the API key (required) and all others empty/unset
        let mut env_vars = HashMap::new();
        env_vars.insert("ANTHROPIC_API_KEY", "test-api-key");

        // These variables should be unset for default tests, not empty strings
        let vars_to_unset = vec![
            "PORT",
            "ANTHROPIC_TARGET_URL",
            "LOG_LEVEL",
            "LOG_FORMAT",
            "LOG_BODIES",
            "LOG_FILE_PATH",
            "LOG_FILE_LEVEL",
            "LOG_MAX_BODY_SIZE",
        ];

        // Explicitly remove them from the environment
        for var in vars_to_unset {
            env::remove_var(var);
        }

        let config = create_test_config_with_env(env_vars);

        // Verify default values
        assert_eq!(config.port, "8080");
        assert_eq!(config.anthropic_api_key, "test-api-key");
        assert_eq!(config.anthropic_target_url, "https://api.anthropic.com");
        assert_eq!(config.log_stdout_level, "info");
        assert_eq!(config.log_format, "pretty");
        assert!(config.log_bodies);
        assert_eq!(config.log_file_path, "./switchboard.log");
        assert_eq!(config.log_file_level, "debug");
        assert_eq!(config.log_max_body_size, 20480);
        assert_eq!(config.log_directory_mode, LogDirectoryMode::Default);
    }

    #[test]
    fn test_environment_variable_parsing() {
        let env_vars = HashMap::from([
            ("PORT", "9090"),
            ("ANTHROPIC_API_KEY", "custom-api-key"),
            ("ANTHROPIC_TARGET_URL", "https://custom.example.com"),
            ("LOG_LEVEL", "debug"),
            ("LOG_FORMAT", "json"),
            ("LOG_BODIES", "false"),
            ("LOG_FILE_PATH", "/tmp/custom.log"),
            ("LOG_FILE_LEVEL", "trace"),
            ("LOG_MAX_BODY_SIZE", "10240"),
            ("LOG_DIRECTORY_MODE", "xdg"),
        ]);

        let config = create_test_config_with_env(env_vars);

        // Verify custom values were used
        assert_eq!(config.port, "9090");
        assert_eq!(config.anthropic_api_key, "custom-api-key");
        assert_eq!(config.anthropic_target_url, "https://custom.example.com");
        assert_eq!(config.log_stdout_level, "debug");
        assert_eq!(config.log_format, "json");
        assert_eq!(config.log_bodies, false);
        assert_eq!(config.log_file_path, "/tmp/custom.log");
        assert_eq!(config.log_file_level, "trace");
        assert_eq!(config.log_max_body_size, 10240);
        assert_eq!(config.log_directory_mode, LogDirectoryMode::Xdg);
    }

    #[test]
    fn test_boolean_parsing() {
        // Test various boolean string representations
        let test_cases = vec![
            ("true", true),
            ("True", true),
            ("TRUE", true),
            ("1", true),
            ("yes", true),
            ("Y", true),
            ("false", false),
            ("False", false),
            ("FALSE", false),
            ("0", false),
            ("no", true), // This should be true since we only check for "false" and "0"
            ("n", true),  // Same here
        ];

        for (input, expected) in test_cases {
            let mut env_vars = HashMap::new();
            env_vars.insert("ANTHROPIC_API_KEY", "test-api-key");
            env_vars.insert("LOG_BODIES", input);

            let config = create_test_config_with_env(env_vars);
            assert_eq!(config.log_bodies, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_numeric_parsing_valid() {
        let env_vars = HashMap::from([
            ("ANTHROPIC_API_KEY", "test-api-key"),
            ("LOG_MAX_BODY_SIZE", "12345"),
        ]);

        let config = create_test_config_with_env(env_vars);
        assert_eq!(config.log_max_body_size, 12345);
    }

    #[test]
    fn test_numeric_parsing_invalid() {
        let env_vars = HashMap::from([
            ("ANTHROPIC_API_KEY", "test-api-key"),
            ("LOG_MAX_BODY_SIZE", "not-a-number"),
        ]);

        let config = create_test_config_with_env(env_vars);
        assert_eq!(config.log_max_body_size, 20480);
    }

    #[test]
    fn test_edge_case_large_value() {
        let max_size_str = usize::MAX.to_string();
        let env_vars = HashMap::from([
            ("ANTHROPIC_API_KEY", "test-api-key"),
            ("LOG_MAX_BODY_SIZE", max_size_str.as_str()),
        ]);

        let config = create_test_config_with_env(env_vars);
        assert_eq!(config.log_max_body_size, usize::MAX);
    }

    #[test]
    fn test_empty_string_environment_variable() {
        // In Rust, setting an environment variable to an empty string with env::set_var
        // is equivalent to removing it for env::var (returns Err)
        // Our test utility now removes empty string vars to match this behavior
        let mut env_vars = HashMap::new();
        env_vars.insert("ANTHROPIC_API_KEY", "test-api-key");

        // First test with the variable unset
        let config = create_test_config_with_env(env_vars.clone());
        assert_eq!(
            config.log_stdout_level, "info",
            "Default should be used when LOG_LEVEL is unset"
        );

        // Then test with an empty string (same behavior as unset)
        env_vars.insert("LOG_LEVEL", "");
        let config = create_test_config_with_env(env_vars);
        assert_eq!(
            config.log_stdout_level, "info",
            "Default should be used when LOG_LEVEL is empty"
        );
    }

    #[test]
    fn test_log_directory_mode_parsing() {
        // Test the default value
        let mut env_vars = HashMap::new();
        env_vars.insert("ANTHROPIC_API_KEY", "test-api-key");

        let config = create_test_config_with_env(env_vars.clone());
        assert_eq!(
            config.log_directory_mode,
            LogDirectoryMode::Default,
            "Default mode should be used when LOG_DIRECTORY_MODE is unset"
        );

        // Test explicit "default" value
        env_vars.insert("LOG_DIRECTORY_MODE", "default");
        let config = create_test_config_with_env(env_vars.clone());
        assert_eq!(
            config.log_directory_mode,
            LogDirectoryMode::Default,
            "Default mode should be used when LOG_DIRECTORY_MODE is 'default'"
        );

        // Test XDG mode
        env_vars.insert("LOG_DIRECTORY_MODE", "xdg");
        let config = create_test_config_with_env(env_vars.clone());
        assert_eq!(
            config.log_directory_mode,
            LogDirectoryMode::Xdg,
            "XDG mode should be used when LOG_DIRECTORY_MODE is 'xdg'"
        );

        // Test System mode
        env_vars.insert("LOG_DIRECTORY_MODE", "system");
        let config = create_test_config_with_env(env_vars.clone());
        assert_eq!(
            config.log_directory_mode,
            LogDirectoryMode::System,
            "System mode should be used when LOG_DIRECTORY_MODE is 'system'"
        );

        // Test case insensitivity
        env_vars.insert("LOG_DIRECTORY_MODE", "XDG");
        let config = create_test_config_with_env(env_vars.clone());
        assert_eq!(
            config.log_directory_mode,
            LogDirectoryMode::Xdg,
            "XDG mode should be used when LOG_DIRECTORY_MODE is 'XDG' (uppercase)"
        );

        // Test invalid value (should default)
        env_vars.insert("LOG_DIRECTORY_MODE", "invalid");
        let config = create_test_config_with_env(env_vars);
        assert_eq!(
            config.log_directory_mode,
            LogDirectoryMode::Default,
            "Default mode should be used when LOG_DIRECTORY_MODE is invalid"
        );
    }

    #[test]
    fn test_edge_case_unusual_path() {
        let env_vars = HashMap::from([
            ("ANTHROPIC_API_KEY", "test-api-key"),
            ("LOG_FILE_PATH", "/dev/null/unusual/../path.log"),
        ]);

        let config = create_test_config_with_env(env_vars);
        assert_eq!(config.log_file_path, "/dev/null/unusual/../path.log");
    }
}
