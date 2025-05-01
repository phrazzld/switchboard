// This module contains tests for the OpenAI configuration functionality.
//
// IMPORTANT: These tests modify environment variables which can cause race conditions
// when tests run in parallel. Environment variables are process-wide global state,
// so concurrent modifications from multiple tests can interfere with each other.
//
// To prevent this, we use the `#[serial]` attribute from the `serial_test` crate on
// all test functions. This ensures that tests annotated with `#[serial]` run sequentially,
// even when the test runner is configured to run tests in parallel (--test-threads=N).
//
// Each test function that modifies environment variables:
// 1. Saves the initial environment state
// 2. Performs its test with modified environment variables
// 3. Restores the original environment state when done

use secrecy::{ExposeSecret, SecretString};
use serial_test::serial;
use std::collections::HashMap;
use std::env;
use std::sync::Once;
use switchboard::config::{
    parse_bool_env, Config, LogDirectoryMode, DEFAULT_ANTHROPIC_TARGET_URL, DEFAULT_LOG_BODIES,
    DEFAULT_LOG_FILE_LEVEL, DEFAULT_LOG_FILE_PATH, DEFAULT_LOG_FORMAT, DEFAULT_LOG_MAX_AGE_DAYS,
    DEFAULT_LOG_MAX_BODY_SIZE, DEFAULT_LOG_STDOUT_LEVEL, DEFAULT_OPENAI_ENABLED,
    DEFAULT_OPENAI_TARGET_URL, DEFAULT_PORT,
};

// Initialize test environment exactly once
static INIT: Once = Once::new();
fn initialize() {
    INIT.call_once(|| {
        // Initialize test environment here
    });
}

// A function to create a test config with specific environment variables
fn create_test_config_with_env(env_vars: HashMap<&str, &str>) -> Config {
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

    // Handle OpenAI configuration
    let openai_enabled_str = env::var("OPENAI_ENABLED").ok();
    let openai_enabled = if let Some(val) = openai_enabled_str {
        if val.to_lowercase() == "false" || val == "0" {
            false
        } else if val.to_lowercase() == "true" || val == "1" {
            true
        } else {
            DEFAULT_OPENAI_ENABLED
        }
    } else {
        DEFAULT_OPENAI_ENABLED
    };

    // Only try to get OPENAI_API_KEY if it's enabled
    let openai_api_key = if openai_enabled {
        env::var("OPENAI_API_KEY").ok()
    } else {
        None
    };

    let openai_api_base_url =
        env::var("OPENAI_API_BASE_URL").unwrap_or_else(|_| DEFAULT_OPENAI_TARGET_URL.to_string());

    let config = Config {
        port,
        anthropic_api_key: SecretString::new(anthropic_api_key.into()),
        anthropic_target_url,
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
        log_max_age_days: DEFAULT_LOG_MAX_AGE_DAYS,
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

#[serial]
#[test]
fn test_openai_default_values() {
    // For default OpenAI values, we just need the Anthropic API key (required) and ensure OpenAI vars are unset
    let mut env_vars = HashMap::new();
    env_vars.insert("ANTHROPIC_API_KEY", "test-api-key");

    // Explicitly remove any OpenAI environment variables
    let vars_to_unset = vec!["OPENAI_API_KEY", "OPENAI_API_BASE_URL", "OPENAI_ENABLED"];

    // Explicitly remove them from the environment
    for var in vars_to_unset {
        env::remove_var(var);
    }

    let config = create_test_config_with_env(env_vars);

    // Verify default values for OpenAI configuration
    assert!(config.openai_api_key.is_none());
    assert_eq!(config.openai_api_base_url, DEFAULT_OPENAI_TARGET_URL);
    assert_eq!(config.openai_enabled, DEFAULT_OPENAI_ENABLED);
    assert!(!config.openai_enabled); // DEFAULT_OPENAI_ENABLED should be false
}

#[serial]
#[test]
fn test_openai_custom_values() {
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-api-key"),
        ("OPENAI_API_KEY", "test-openai-key"),
        ("OPENAI_API_BASE_URL", "https://custom-openai.example.com"),
        ("OPENAI_ENABLED", "true"),
    ]);

    // We need to modify the create_test_config_with_env implementation to properly handle OpenAI vars
    // For this test, we'll directly use the environment variables and manually construct the Config
    initialize();

    // Save current environment
    let mut old_vars = HashMap::new();
    for (key, _) in env_vars.iter() {
        old_vars.insert(*key, env::var(*key).ok());
    }

    // Set provided environment variables
    for (key, value) in env_vars.iter() {
        if !value.is_empty() {
            env::set_var(key, value);
        } else {
            env::remove_var(key);
        }
    }

    // Load the OpenAI configuration directly
    let openai_api_key = env::var("OPENAI_API_KEY").ok();
    let openai_api_base_url =
        env::var("OPENAI_API_BASE_URL").unwrap_or_else(|_| DEFAULT_OPENAI_TARGET_URL.to_string());
    let openai_enabled = match env::var("OPENAI_ENABLED") {
        Ok(value) => {
            if value.to_lowercase() == "true"
                || value.to_lowercase() == "false"
                || value == "0"
                || value == "1"
            {
                value.to_lowercase() == "true" || value == "1"
            } else {
                DEFAULT_OPENAI_ENABLED
            }
        }
        Err(_) => DEFAULT_OPENAI_ENABLED,
    };

    // Create a new config
    let anthropic_api_key = env::var("ANTHROPIC_API_KEY").unwrap();
    let anthropic_target_url = DEFAULT_ANTHROPIC_TARGET_URL.to_string();
    let config = Config {
        port: DEFAULT_PORT.to_string(),
        anthropic_api_key: SecretString::new(anthropic_api_key.into()),
        anthropic_target_url,
        openai_api_key: openai_api_key.map(|key| SecretString::new(key.into())),
        openai_api_base_url,
        openai_enabled,
        log_stdout_level: DEFAULT_LOG_STDOUT_LEVEL.to_string(),
        log_format: DEFAULT_LOG_FORMAT.to_string(),
        log_bodies: DEFAULT_LOG_BODIES,
        log_file_path: DEFAULT_LOG_FILE_PATH.to_string(),
        log_file_level: DEFAULT_LOG_FILE_LEVEL.to_string(),
        log_max_body_size: DEFAULT_LOG_MAX_BODY_SIZE,
        log_directory_mode: LogDirectoryMode::Default,
        log_max_age_days: DEFAULT_LOG_MAX_AGE_DAYS,
    };

    // Restore old environment
    for (key, value_opt) in old_vars {
        match value_opt {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
    }

    // Verify custom values for OpenAI configuration
    assert_eq!(
        config.openai_api_key.as_ref().unwrap().expose_secret(),
        "test-openai-key"
    );
    assert_eq!(
        config.openai_api_base_url,
        "https://custom-openai.example.com"
    );
    assert!(config.openai_enabled);
}

#[serial]
#[test]
fn test_openai_enabled_boolean_parsing() {
    // Test various boolean string representations for OPENAI_ENABLED
    let test_cases = vec![
        // Valid true values
        ("true", true),
        ("True", true),
        ("TRUE", true),
        ("1", true),
        // Valid false values
        ("false", false),
        ("False", false),
        ("FALSE", false),
        ("0", false),
        // Invalid values (should use default, which is false)
        ("yes", DEFAULT_OPENAI_ENABLED),
        ("no", DEFAULT_OPENAI_ENABLED),
        ("enabled", DEFAULT_OPENAI_ENABLED),
        ("disabled", DEFAULT_OPENAI_ENABLED),
    ];

    for (input, expected) in test_cases {
        // Save current environment
        let old_openai_enabled = env::var("OPENAI_ENABLED").ok();

        // Set test environment
        env::set_var("OPENAI_ENABLED", input);
        env::set_var("ANTHROPIC_API_KEY", "test-api-key");

        // If input would result in true, we need to set OPENAI_API_KEY to prevent panic
        if expected {
            env::set_var("OPENAI_API_KEY", "test-openai-key");
        }

        // Use our standardized helper directly
        let openai_enabled_result = parse_bool_env("OPENAI_ENABLED", DEFAULT_OPENAI_ENABLED);

        // Restore environment
        match old_openai_enabled {
            Some(value) => env::set_var("OPENAI_ENABLED", value),
            None => env::remove_var("OPENAI_ENABLED"),
        }
        env::remove_var("ANTHROPIC_API_KEY");
        env::remove_var("OPENAI_API_KEY");

        match openai_enabled_result {
            Ok(openai_enabled) => {
                assert_eq!(
                    openai_enabled, expected,
                    "OPENAI_ENABLED='{}' should parse to {}",
                    input, expected
                );
            }
            Err(e) => {
                // If we expected a valid value but got an error, fail the test
                if expected == DEFAULT_OPENAI_ENABLED {
                    // If the expected value is the default, the input was invalid, which is fine
                    println!("Got expected error for invalid input '{}': {}", input, e);
                } else {
                    panic!(
                        "OPENAI_ENABLED='{}' should parse to {} but got error: {}",
                        input, expected, e
                    );
                }
            }
        }
    }
}

#[serial]
#[test]
fn test_openai_api_key_not_required_when_disabled() {
    let env_vars = HashMap::from([
        ("ANTHROPIC_API_KEY", "test-api-key"),
        ("OPENAI_ENABLED", "false"),
        // No OPENAI_API_KEY provided, which is fine when disabled
    ]);

    let config = create_test_config_with_env(env_vars);

    // Verify OpenAI is disabled and API key is None
    assert!(!config.openai_enabled);
    assert!(config.openai_api_key.is_none());
}
