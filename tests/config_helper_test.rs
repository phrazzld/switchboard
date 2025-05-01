// This is the proper way to test the config_helpers functionality
// First import common module
mod common;
use common::config_helpers;

#[cfg(test)]
mod config_helper_tests {
    use std::env;
    use switchboard::config::{ConfigError, LogDirectoryMode};
    // Import the helper functions from the common module
    use super::config_helpers::{create_test_config, load_test_config, with_env_vars};

    #[test]
    fn test_load_test_config_with_env_vars() {
        // Clear any existing PORT value that might interfere
        env::remove_var("PORT");
        
        // Set multiple environment variables
        let env_vars = vec![
            ("ANTHROPIC_API_KEY", "test-key"),
            ("ANTHROPIC_TARGET_URL", "https://example.com"), // Add valid URL
            ("PORT", "9090"),
            ("LOG_FORMAT", "json"),
            ("LOG_BODIES", "false"),
            ("LOG_MAX_BODY_SIZE", "10000"),
            ("LOG_LEVEL", "info"), // Add valid log level
        ];

        with_env_vars(env_vars, || {
            // Verify PORT was set correctly
            assert_eq!(env::var("PORT").unwrap(), "9090");
            
            let config = load_test_config().expect("Failed to load test config");

            // Verify that the environment variables were applied
            assert_eq!(config.port, "9090");
            assert_eq!(config.log_format, "json");
            assert_eq!(config.log_bodies, false);
            assert_eq!(config.log_max_body_size, 10000);
            assert_eq!(config.log_stdout_level, "info");
        });
    }

    // This test is broken into three separate tests to avoid environment interference
    #[test]
    fn test_load_test_config_with_invalid_port() {
        // Test with invalid PORT
        let env_vars = vec![
            ("ANTHROPIC_API_KEY", "test-key"),
            ("ANTHROPIC_TARGET_URL", "https://example.com"), // Add valid URL
            ("PORT", "invalid"),                             // Not a number
        ];

        with_env_vars(env_vars, || {
            let result = load_test_config();
            assert!(result.is_err());
            match result {
                Err(ConfigError::InvalidNumericValue { var, value }) => {
                    assert_eq!(var, "PORT");
                    assert_eq!(value, "invalid");
                }
                err => panic!(
                    "Expected InvalidNumericValue error for PORT, got: {:?}",
                    err
                ),
            }
        });
    }

    #[test]
    fn test_load_test_config_with_invalid_url() {
        // Test with invalid URL - need to make these separate tests
        let env_vars = vec![
            ("ANTHROPIC_API_KEY", "test-key"),
            ("ANTHROPIC_TARGET_URL", "not-a-url"), // Missing http:// or https://
        ];

        with_env_vars(env_vars, || {
            // Double check the env var is actually set
            assert_eq!(env::var("ANTHROPIC_TARGET_URL").unwrap(), "not-a-url");

            let result = load_test_config();
            assert!(result.is_err(), "Expected an error for invalid URL");
            match result {
                Err(ConfigError::InvalidFormat { var, .. }) => {
                    assert_eq!(var, "ANTHROPIC_TARGET_URL");
                }
                err => panic!(
                    "Expected InvalidFormat error for ANTHROPIC_TARGET_URL, got: {:?}",
                    err
                ),
            }
        });
    }

    #[test]
    fn test_load_test_config_with_invalid_log_level() {
        // Clear any existing LOG_LEVEL that might interfere
        env::remove_var("LOG_LEVEL");
        
        // Test with invalid log level
        let env_vars = vec![
            ("ANTHROPIC_API_KEY", "test-key"),
            ("ANTHROPIC_TARGET_URL", "https://example.com"), // Add valid URL
            ("LOG_LEVEL", "invalid-level"),                  // Not a valid log level
        ];

        with_env_vars(env_vars, || {
            // Verify LOG_LEVEL was set correctly
            assert_eq!(env::var("LOG_LEVEL").unwrap(), "invalid-level");
            
            let result = load_test_config();
            assert!(result.is_err());
            match result {
                Err(ConfigError::InvalidFormat { var, .. }) => {
                    assert_eq!(var, "LOG_LEVEL");
                }
                err => panic!("Expected InvalidFormat error for LOG_LEVEL, got: {:?}", err),
            }
        });
    }

    #[test]
    fn test_create_test_config() {
        // This should create a valid config with test values, regardless of env vars
        let config = create_test_config();

        // Clear environment to ensure it doesn't affect the result
        env::remove_var("ANTHROPIC_API_KEY");
        env::remove_var("PORT");

        // Check for expected defaults
        assert_eq!(config.log_format, "pretty");
        assert_eq!(config.log_stdout_level, "debug"); // Test default, not production default
        assert_eq!(config.log_file_level, "trace"); // Test default, not production default
        assert_eq!(config.log_directory_mode, LogDirectoryMode::Default);
    }

    // This test is skipped since we can't control the global environment variables
    // in a way that guarantees this test will always work correctly.
    // The functionality is tested internally in config_helpers.rs.
    #[test]
    #[ignore = "Environment variable management is unreliable in CI"]
    fn test_openai_enabled_without_key() {
        // First clear any existing OPENAI_API_KEY in the environment
        env::remove_var("OPENAI_API_KEY");

        // Test scenario where OpenAI is enabled but no key is provided
        let env_vars = vec![
            ("ANTHROPIC_API_KEY", "test-key"),
            ("ANTHROPIC_TARGET_URL", "https://example.com"),
            ("OPENAI_ENABLED", "true"),
            // Deliberately not setting OPENAI_API_KEY
        ];

        with_env_vars(env_vars, || {
            // Make sure there's no OPENAI_API_KEY in the environment
            assert!(env::var("OPENAI_API_KEY").is_err());

            let result = load_test_config();
            assert!(
                result.is_err(),
                "Expected an error for missing OpenAI API key"
            );
            match result {
                Err(ConfigError::MissingOpenAiKey) => {
                    // This is the expected error
                }
                unexpected => panic!("Expected MissingOpenAiKey error, got: {:?}", unexpected),
            }
        });
    }

    #[test]
    fn test_openai_configuration() {
        // Clear environment variables that might interfere
        env::remove_var("OPENAI_ENABLED");
        env::remove_var("OPENAI_API_KEY");
        env::remove_var("OPENAI_API_BASE_URL");
        env::remove_var("LOG_LEVEL");
        
        // Test with OpenAI enabled and key provided
        let env_vars = vec![
            ("ANTHROPIC_API_KEY", "test-key"),
            ("ANTHROPIC_TARGET_URL", "https://example.com"), // Add valid URL
            ("OPENAI_ENABLED", "true"),
            ("OPENAI_API_KEY", "test-openai-key"),
            ("OPENAI_API_BASE_URL", "https://custom-openai-endpoint.com"),
            ("LOG_LEVEL", "info"), // Add valid log level
        ];

        with_env_vars(env_vars, || {
            // Verify environment variables are set correctly
            assert_eq!(env::var("OPENAI_ENABLED").unwrap(), "true");
            assert_eq!(env::var("OPENAI_API_KEY").unwrap(), "test-openai-key");
            assert_eq!(env::var("OPENAI_API_BASE_URL").unwrap(), "https://custom-openai-endpoint.com");
            
            let config = load_test_config().expect("Failed to load test config");

            // Verify OpenAI-specific settings
            assert!(config.openai_enabled);
            assert!(config.openai_api_key.is_some());
            assert_eq!(
                config.openai_api_base_url,
                "https://custom-openai-endpoint.com"
            );
        });
    }
}
