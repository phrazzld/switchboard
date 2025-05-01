// Test helpers for Config implementation
//
// This module provides utilities to create and manipulate
// Config instances for testing purposes, mirroring production behavior.

use secrecy::SecretString;
use std::env;
use switchboard::config::{Config, ConfigError, LogDirectoryMode};

// Import constants from config module to ensure consistency
use switchboard::config::{
    DEFAULT_ANTHROPIC_TARGET_URL, DEFAULT_LOG_BODIES, 
    DEFAULT_LOG_FILE_LEVEL, DEFAULT_LOG_FILE_PATH, DEFAULT_LOG_FORMAT, DEFAULT_LOG_MAX_AGE_DAYS,
    DEFAULT_LOG_MAX_BODY_SIZE, DEFAULT_LOG_STDOUT_LEVEL, DEFAULT_OPENAI_ENABLED,
    DEFAULT_OPENAI_TARGET_URL, DEFAULT_PORT,
};

/// Parse a boolean environment variable with standardized behavior, mirroring production config
///
/// This is a direct copy of the parse_bool_env function from src/config.rs to ensure
/// consistent behavior in tests. It follows the same rules:
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

/// Load configuration from environment variables for testing purposes
///
/// This function mirrors the production load_config() behavior exactly, but:
/// - Returns Result<Config, ConfigError> instead of panicking on errors
/// - Does not log configuration (avoiding global state changes)
/// - Can be called repeatedly in tests without side effects
///
/// # Returns
/// Result containing the loaded Config or a ConfigError
pub fn load_test_config() -> Result<Config, ConfigError> {
    // Don't call dotenvy::dotenv() here - tests should set env vars explicitly
    // to avoid side effects and maintain reproducibility

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

    // Skip the info! logging call to avoid side effects in tests

    Ok(loaded_config)
}

/// Create a test configuration with default values suitable for testing.
///
/// This function creates a Config instance with sensible defaults for testing,
/// including a test API key. This avoids the need to set environment variables
/// for common test scenarios.
///
/// # Returns
/// A Config instance suitable for use in tests
pub fn create_test_config() -> Config {
    Config {
        port: DEFAULT_PORT.to_string(),
        anthropic_api_key: SecretString::new("test-api-key".to_string().into()),
        anthropic_target_url: DEFAULT_ANTHROPIC_TARGET_URL.to_string(),
        openai_api_key: Some(SecretString::new("test-openai-api-key".to_string().into())),
        openai_api_base_url: DEFAULT_OPENAI_TARGET_URL.to_string(),
        openai_enabled: false,
        log_stdout_level: "debug".to_string(), // More verbose for testing
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: "./test-switchboard.log".to_string(),
        log_file_level: "trace".to_string(), // Most verbose for file logs
        log_max_body_size: DEFAULT_LOG_MAX_BODY_SIZE,
        log_directory_mode: LogDirectoryMode::Default,
        log_max_age_days: None,
    }
}

/// Apply environment variable overrides for testing
///
/// This function applies environment variables to override configuration values
/// for the duration of the provided function. It restores the original environment
/// after the function completes, ensuring tests don't affect each other.
///
/// # Arguments
/// * `env_vars` - A vector of (key, value) tuples representing environment variables
/// * `test_fn` - A function to run with the environment variables set
///
/// # Example
/// ```
/// use switchboard::tests::common::config_helpers;
///
/// #[test]
/// fn test_config_with_custom_port() {
///     let env_vars = vec![("PORT", "9000")];
///     
///     config_helpers::with_env_vars(env_vars, || {
///         let config = config_helpers::load_test_config().unwrap();
///         assert_eq!(config.port, "9000");
///     });
/// }
/// ```
pub fn with_env_vars<F>(env_vars: Vec<(&str, &str)>, test_fn: F)
where
    F: FnOnce(),
{
    // Save the original values to restore later
    let mut original_vars = Vec::new();

    // Set all the environment variables
    for (key, value) in &env_vars {
        let original = env::var(key).ok();
        original_vars.push((key, original));
        env::set_var(key, value);
    }

    // Run the test function
    test_fn();

    // Restore the original environment
    for (key, original) in original_vars {
        match original {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config() {
        let config = create_test_config();

        // Verify that the test config has expected values
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.log_stdout_level, "debug");
        assert_eq!(config.log_file_level, "trace");
        assert_eq!(config.log_format, "pretty");
        assert!(config.log_bodies);
        assert_eq!(config.log_directory_mode, LogDirectoryMode::Default);
    }

    #[test]
    fn test_with_env_vars() {
        // Capture original PORT value if it exists
        let original_port = env::var("PORT").ok();
        
        // Set a custom port for this test and include ANTHROPIC_API_KEY
        let env_vars = vec![
            ("PORT", "9999"),
            ("ANTHROPIC_API_KEY", "test-key"),
            ("ANTHROPIC_TARGET_URL", "https://example.com"),
        ];

        with_env_vars(env_vars, || {
            // Verify the environment variable is set inside the closure
            assert_eq!(env::var("PORT").unwrap(), "9999");

            // Test that it affects load_test_config as expected
            let config = load_test_config().expect("Failed to load test config");
            assert_eq!(config.port, "9999");
        });

        // Verify the environment variable is restored to its original value
        match original_port {
            Some(original) => assert_eq!(env::var("PORT").unwrap(), original),
            None => assert!(env::var("PORT").is_err()),
        }
    }

    #[test]
    fn test_load_test_config_with_required_vars() {
        // Save original PORT value
        let original_port_value = env::var("PORT").ok();
        
        // Temporarily remove PORT from environment to avoid interference
        env::remove_var("PORT");
        
        // Set the required ANTHROPIC_API_KEY to pass validation
        let env_vars = vec![
            ("ANTHROPIC_API_KEY", "test-key"),
            ("ANTHROPIC_TARGET_URL", "https://example.com"),
        ];

        with_env_vars(env_vars, || {
            let result = load_test_config();
            assert!(result.is_ok());

            let config = result.unwrap();
            // With PORT removed from environment, should use default
            assert_eq!(config.port, DEFAULT_PORT);
        });
        
        // Restore original PORT value if it existed
        if let Some(port_value) = original_port_value {
            env::set_var("PORT", port_value);
        }
    }

    #[test]
    fn test_load_test_config_missing_required_key() {
        // Ensure ANTHROPIC_API_KEY is not set
        env::remove_var("ANTHROPIC_API_KEY");

        let result = load_test_config();
        assert!(result.is_err());

        match result {
            Err(ConfigError::MissingRequiredKey("ANTHROPIC_API_KEY")) => {
                // Expected error
            }
            _ => panic!("Expected MissingRequiredKey error, got: {:?}", result),
        }
    }

    #[test]
    fn test_parse_bool_env() {
        // Test true values
        let env_vars = vec![("TEST_BOOL", "true")];
        with_env_vars(env_vars, || {
            assert_eq!(parse_bool_env("TEST_BOOL", false).unwrap(), true);
        });

        // Test false values
        let env_vars = vec![("TEST_BOOL", "false")];
        with_env_vars(env_vars, || {
            assert_eq!(parse_bool_env("TEST_BOOL", true).unwrap(), false);
        });

        // Test numeric values
        let env_vars = vec![("TEST_BOOL", "1")];
        with_env_vars(env_vars, || {
            assert_eq!(parse_bool_env("TEST_BOOL", false).unwrap(), true);
        });

        // Test default values
        env::remove_var("TEST_BOOL");
        assert_eq!(parse_bool_env("TEST_BOOL", true).unwrap(), true);
        assert_eq!(parse_bool_env("TEST_BOOL", false).unwrap(), false);

        // Test invalid values
        let env_vars = vec![("TEST_BOOL", "invalid")];
        with_env_vars(env_vars, || {
            assert!(parse_bool_env("TEST_BOOL", false).is_err());
        });
    }
}
