// This module contains error handling tests for the configuration functionality.
//
// IMPORTANT: These tests modify environment variables which can cause race conditions
// when tests run in parallel. Environment variables are process-wide global state,
// so concurrent modifications from multiple tests can interfere with each other.
//
// To prevent this, we use the `#[serial]` attribute from the `serial_test` crate on
// all test functions. This ensures that tests annotated with `#[serial]` run sequentially,
// even when the test runner is configured to run tests in parallel (--test-threads=N).

use secrecy::ExposeSecret;
use serial_test::serial;
use std::collections::HashMap;
use std::env;
use std::sync::Once;
use switchboard::config::{load_config, Config, ConfigError};

// Initialize test environment exactly once
static INIT: Once = Once::new();
fn initialize() {
    INIT.call_once(|| {
        // Initialize test environment here
    });
}

/// A function to test load_config with specific environment variables
/// Returns the Result from load_config()
fn test_load_config_with_env(env_vars: HashMap<&str, &str>) -> Result<Config, ConfigError> {
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

    // Call load_config() which now returns Result<Config, ConfigError>
    let result = load_config();

    // Restore old environment
    for (key, value_opt) in old_vars {
        match value_opt {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
    }

    result
}

#[serial]
#[test]
fn test_openai_api_key_required_when_enabled() {
    // Explicitly remove OPENAI_API_KEY if it exists
    env::remove_var("OPENAI_API_KEY");

    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-api-key"),
        ("OPENAI_ENABLED", "true"),
        // Deliberately not setting OPENAI_API_KEY to trigger the validation error
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::MissingOpenAiKey but got Ok"),
        Err(e) => match e {
            ConfigError::MissingOpenAiKey => { /* This is what we want */ }
            other => panic!("Expected ConfigError::MissingOpenAiKey but got {:?}", other),
        },
    }
}

#[serial]
#[test]
fn test_missing_anthropic_api_key() {
    // Create empty environment variables and explicitly unset ANTHROPIC_API_KEY
    let env_vars = HashMap::new();

    // Explicitly remove ANTHROPIC_API_KEY from environment
    env::remove_var("ANTHROPIC_API_KEY");

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::MissingRequiredKey but got Ok"),
        Err(e) => match e {
            ConfigError::MissingRequiredKey(key) => {
                assert_eq!(
                    key, "ANTHROPIC_API_KEY",
                    "Expected error for ANTHROPIC_API_KEY"
                );
            }
            other => panic!(
                "Expected ConfigError::MissingRequiredKey but got {:?}",
                other
            ),
        },
    }
}

#[serial]
#[test]
fn test_empty_anthropic_api_key() {
    // Set ANTHROPIC_API_KEY to an empty string
    // Note: The HashMap entry with empty string will remove the var in test_load_config_with_env
    // So we need to set it directly
    env::set_var("ANTHROPIC_API_KEY", "");

    let env_vars = HashMap::new();

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Restore environment
    env::remove_var("ANTHROPIC_API_KEY");

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::EmptyValue but got Ok"),
        Err(e) => match e {
            ConfigError::EmptyValue(key) => {
                assert_eq!(
                    key, "ANTHROPIC_API_KEY",
                    "Expected error for empty ANTHROPIC_API_KEY"
                );
            }
            other => panic!("Expected ConfigError::EmptyValue but got {:?}", other),
        },
    }
}

#[serial]
#[test]
fn test_invalid_boolean_value_log_bodies() {
    // Set LOG_BODIES to an invalid boolean value
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("LOG_BODIES", "not-a-boolean"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidBooleanValue but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidBooleanValue { var, value } => {
                assert_eq!(var, "LOG_BODIES", "Expected error for LOG_BODIES");
                assert_eq!(
                    value, "not-a-boolean",
                    "Expected error value to match input"
                );
            }
            other => panic!(
                "Expected ConfigError::InvalidBooleanValue but got {:?}",
                other
            ),
        },
    }
}

#[serial]
#[test]
fn test_invalid_boolean_value_openai_enabled() {
    // Set OPENAI_ENABLED to an invalid boolean value
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("OPENAI_ENABLED", "maybe"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidBooleanValue but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidBooleanValue { var, value } => {
                assert_eq!(var, "OPENAI_ENABLED", "Expected error for OPENAI_ENABLED");
                assert_eq!(value, "maybe", "Expected error value to match input");
            }
            other => panic!(
                "Expected ConfigError::InvalidBooleanValue but got {:?}",
                other
            ),
        },
    }
}

#[serial]
#[test]
fn test_invalid_numeric_value_port() {
    // Set PORT to an invalid numeric value
    let env_vars = HashMap::from([("ANTHROPIC_API_KEY", "test-key"), ("PORT", "not-a-number")]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidNumericValue but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidNumericValue { var, value } => {
                assert_eq!(var, "PORT", "Expected error for PORT");
                assert_eq!(value, "not-a-number", "Expected error value to match input");
            }
            other => panic!(
                "Expected ConfigError::InvalidNumericValue but got {:?}",
                other
            ),
        },
    }
}

#[serial]
#[test]
fn test_invalid_numeric_value_max_body_size() {
    // Set LOG_MAX_BODY_SIZE to an invalid numeric value
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("LOG_MAX_BODY_SIZE", "not-a-number"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidNumericValue but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidNumericValue { var, value } => {
                assert_eq!(
                    var, "LOG_MAX_BODY_SIZE",
                    "Expected error for LOG_MAX_BODY_SIZE"
                );
                assert_eq!(value, "not-a-number", "Expected error value to match input");
            }
            other => panic!(
                "Expected ConfigError::InvalidNumericValue but got {:?}",
                other
            ),
        },
    }
}

#[serial]
#[test]
fn test_invalid_numeric_value_max_age_days() {
    // Set LOG_MAX_AGE_DAYS to an invalid numeric value
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("LOG_MAX_AGE_DAYS", "not-a-number"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidNumericValue but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidNumericValue { var, value } => {
                assert_eq!(
                    var, "LOG_MAX_AGE_DAYS",
                    "Expected error for LOG_MAX_AGE_DAYS"
                );
                assert_eq!(value, "not-a-number", "Expected error value to match input");
            }
            other => panic!(
                "Expected ConfigError::InvalidNumericValue but got {:?}",
                other
            ),
        },
    }
}

#[serial]
#[test]
fn test_invalid_url_format_anthropic() {
    // Set ANTHROPIC_TARGET_URL to an invalid URL format
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("ANTHROPIC_TARGET_URL", "invalid-url-no-protocol"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidFormat but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidFormat { var, reason } => {
                assert_eq!(
                    var, "ANTHROPIC_TARGET_URL",
                    "Expected error for ANTHROPIC_TARGET_URL"
                );
                assert!(
                    reason.contains("URL must start with 'http://' or 'https://'"),
                    "Expected error reason to mention URL format requirements"
                );
            }
            other => panic!("Expected ConfigError::InvalidFormat but got {:?}", other),
        },
    }
}

#[serial]
#[test]
fn test_invalid_url_format_openai() {
    // Set OPENAI_API_BASE_URL to an invalid URL format
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("OPENAI_API_BASE_URL", "invalid-url-no-protocol"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidFormat but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidFormat { var, reason } => {
                assert_eq!(
                    var, "OPENAI_API_BASE_URL",
                    "Expected error for OPENAI_API_BASE_URL"
                );
                assert!(
                    reason.contains("URL must start with 'http://' or 'https://'"),
                    "Expected error reason to mention URL format requirements"
                );
            }
            other => panic!("Expected ConfigError::InvalidFormat but got {:?}", other),
        },
    }
}

#[serial]
#[test]
fn test_invalid_log_level() {
    // Set LOG_LEVEL to an invalid log level
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("LOG_LEVEL", "invalid-level"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidFormat but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidFormat { var, reason } => {
                assert_eq!(var, "LOG_LEVEL", "Expected error for LOG_LEVEL");
                assert!(
                    reason.contains("Invalid log level"),
                    "Expected error reason to mention log level requirements"
                );
            }
            other => panic!("Expected ConfigError::InvalidFormat but got {:?}", other),
        },
    }
}

#[serial]
#[test]
fn test_invalid_log_format() {
    // Set LOG_FORMAT to an invalid format
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("LOG_FORMAT", "invalid-format"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidFormat but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidFormat { var, reason } => {
                assert_eq!(var, "LOG_FORMAT", "Expected error for LOG_FORMAT");
                assert!(
                    reason.contains("Invalid log format"),
                    "Expected error reason to mention log format requirements"
                );
            }
            other => panic!("Expected ConfigError::InvalidFormat but got {:?}", other),
        },
    }
}

#[serial]
#[test]
fn test_invalid_log_directory_mode() {
    // Set LOG_DIRECTORY_MODE to an invalid mode
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("LOG_DIRECTORY_MODE", "invalid-mode"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get the expected error
    match result {
        Ok(_) => panic!("Expected ConfigError::InvalidFormat but got Ok"),
        Err(e) => match e {
            ConfigError::InvalidFormat { var, reason } => {
                assert_eq!(
                    var, "LOG_DIRECTORY_MODE",
                    "Expected error for LOG_DIRECTORY_MODE"
                );
                assert!(
                    reason.contains("Invalid value"),
                    "Expected error reason to mention valid mode requirements"
                );
            }
            other => panic!("Expected ConfigError::InvalidFormat but got {:?}", other),
        },
    }
}

#[serial]
#[test]
fn test_successful_config_loading() {
    // Set valid environment variables for all required and some optional configs
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-key"),
        ("PORT", "9090"),
        ("ANTHROPIC_TARGET_URL", "https://custom.anthropic.com"),
        ("LOG_LEVEL", "debug"),
        ("LOG_FORMAT", "json"),
        ("LOG_BODIES", "false"),
        ("LOG_FILE_PATH", "/tmp/test.log"),
    ]);

    // Use our new helper function
    let result = test_load_config_with_env(env_vars);

    // Verify we get a successful result with the expected config values
    match result {
        Ok(config) => {
            assert_eq!(config.port, "9090", "PORT not properly loaded");
            assert_eq!(
                config.anthropic_api_key.expose_secret(),
                "test-key",
                "ANTHROPIC_API_KEY not properly loaded"
            );
            assert_eq!(
                config.anthropic_target_url, "https://custom.anthropic.com",
                "ANTHROPIC_TARGET_URL not properly loaded"
            );
            assert_eq!(
                config.log_stdout_level, "debug",
                "LOG_LEVEL not properly loaded"
            );
            assert_eq!(config.log_format, "json", "LOG_FORMAT not properly loaded");
            assert!(!config.log_bodies, "LOG_BODIES not properly loaded");
            assert_eq!(
                config.log_file_path, "/tmp/test.log",
                "LOG_FILE_PATH not properly loaded"
            );
        }
        Err(e) => panic!("Expected Ok but got error: {:?}", e),
    }
}
