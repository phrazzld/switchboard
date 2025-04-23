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
