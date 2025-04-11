mod config;
mod proxy_handler;
mod logger;

use tokio::signal;
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

/// Handles graceful shutdown signals by waiting for either Ctrl+C or SIGTERM
/// This allows the application to properly close resources and finish ongoing requests
/// before shutting down.
async fn shutdown_signal() {
    // Set up Ctrl+C handler for interactive use
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    
    // Set up SIGTERM handler for container environments
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    
    // On non-unix platforms, create a future that never resolves
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    // Wait for either signal
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    info!("Shutdown signal received, starting graceful shutdown...");
    // No explicit cleanup needed here, as the server uses this signal
    // to start its graceful shutdown process automatically
}
