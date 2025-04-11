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
    /// Log level (info, debug, etc.)
    pub log_level: String,
    /// Log format (json or pretty)
    pub log_format: String,
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

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

        let loaded_config = Config {
            port,
            anthropic_api_key,
            anthropic_target_url,
            log_level,
            log_format,
        };

        // Log configuration values, but omit the API key for security
        info!(
            port = %loaded_config.port,
            target_url = %loaded_config.anthropic_target_url,
            log_level = %loaded_config.log_level,
            log_format = %loaded_config.log_format,
            "Configuration loaded"
        );

        loaded_config
    })
}
