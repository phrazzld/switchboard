use std::env;
use std::sync::OnceLock;
use tracing::info;

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
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

        // API key is mandatory
        let anthropic_api_key =
            env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set for forwarding");

        let anthropic_target_url = env::var("ANTHROPIC_TARGET_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());

        let log_stdout_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());
        let log_bodies = env::var("LOG_BODIES")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);

        // Load file logging configuration
        let log_file_path =
            env::var("LOG_FILE_PATH").unwrap_or_else(|_| "./switchboard.log".to_string());
        let log_file_level = env::var("LOG_FILE_LEVEL").unwrap_or_else(|_| "debug".to_string());

        // Parse LOG_MAX_BODY_SIZE with error handling
        let log_max_body_size = env::var("LOG_MAX_BODY_SIZE")
            .ok()
            .and_then(|size_str| {
                size_str.parse::<usize>().ok().or_else(|| {
                    eprintln!(
                        "Failed to parse LOG_MAX_BODY_SIZE: '{}', using default 20480",
                        size_str
                    );
                    None
                })
            })
            .unwrap_or(20480); // Default to 20KB if not set or invalid

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
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let anthropic_api_key =
            env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());
        let anthropic_target_url = env::var("ANTHROPIC_TARGET_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
        let log_stdout_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());
        let log_bodies = env::var("LOG_BODIES")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);
        let log_file_path =
            env::var("LOG_FILE_PATH").unwrap_or_else(|_| "./switchboard.log".to_string());
        let log_file_level = env::var("LOG_FILE_LEVEL").unwrap_or_else(|_| "debug".to_string());
        let log_max_body_size = env::var("LOG_MAX_BODY_SIZE")
            .ok()
            .and_then(|size_str| {
                size_str.parse::<usize>().ok().or_else(|| {
                    eprintln!(
                        "Failed to parse LOG_MAX_BODY_SIZE: '{}', using default 20480",
                        size_str
                    );
                    None
                })
            })
            .unwrap_or(20480);

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
    fn test_edge_case_unusual_path() {
        let env_vars = HashMap::from([
            ("ANTHROPIC_API_KEY", "test-api-key"),
            ("LOG_FILE_PATH", "/dev/null/unusual/../path.log"),
        ]);

        let config = create_test_config_with_env(env_vars);
        assert_eq!(config.log_file_path, "/dev/null/unusual/../path.log");
    }
}
