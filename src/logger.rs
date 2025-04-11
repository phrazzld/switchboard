//! Logging configuration and setup
//!
//! This module handles the initialization and configuration of the tracing system
//! used for structured logging throughout the application.

use tracing_subscriber::{fmt, prelude::*, EnvFilter, registry};
use crate::config::Config;

/// Initialize the tracing system for structured logging
///
/// Sets up the tracing subscriber with appropriate filtering and formatting
/// based on the provided configuration.
///
/// # Arguments
/// * `config` - The application configuration containing log_level and log_format settings
///
/// # Example
/// ```
/// let config = config::load_config();
/// logger::init_tracing(config);
/// ```
pub fn init_tracing(config: &Config) {
    // Try to get filter directive from environment first (RUST_LOG),
    // fall back to config.log_level, or use 'info' as last resort
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .unwrap_or_else(|e| {
            eprintln!("Failed to parse log level filter: {}, using default 'info'", e);
            EnvFilter::new("info") // Fallback to info
        });

    // Start building the subscriber with the filter
    let subscriber = registry().with(filter);

    // Configure formatting based on config setting
    match config.log_format.as_str() {
        "json" => {
            // JSON formatting for structured logging (good for production/cloud environments)
            let json_layer = fmt::layer().json();
            subscriber.with(json_layer).init();
        }
        _ => {
            // Default to pretty formatting (good for development)
            let pretty_layer = fmt::layer().pretty();
            subscriber.with(pretty_layer).init();
        }
    }
}