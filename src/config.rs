use std::env;
use std::sync::OnceLock;

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

// Static CONFIG instance will be implemented in a later task

// Configuration loading function will be implemented in a later task
pub fn load_config() -> &'static Config {
    todo!("Config loading will be implemented in a later task")
}