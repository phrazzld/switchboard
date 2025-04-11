mod config;
mod proxy_handler;
mod logger;

use tracing::info;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Main application entry point
    println!("Starting Anthropic visibility proxy...");
    
    // Load configuration from environment variables and .env file
    let config = config::load_config();
    
    // Initialize tracing for structured logging
    logger::init_tracing(config);
    
    info!("Anthropic visibility proxy initialized");
    
    // Create HTTP client with appropriate settings
    // Using rustls (instead of native-tls) for TLS implementation
    let client = reqwest::Client::builder()
        .use_rustls_tls() // Use rustls instead of native-tls
        .timeout(Duration::from_secs(600)) // 10-minute timeout for request completion (LLM responses can be lengthy)
        .connect_timeout(Duration::from_secs(10)) // 10-second timeout for connection establishment
        .pool_idle_timeout(Duration::from_secs(90)) // Keep connections in the pool for reuse
        .build()
        .map_err(|e| {
            tracing::error!("Failed to build reqwest client: {}", e);
            e
        })?;
    
    info!("HTTP client created with rustls TLS support");
    
    Ok(())
}
