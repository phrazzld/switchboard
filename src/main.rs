mod config;
mod logger;
mod proxy_handler;

use axum::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};

use proxy_handler::create_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Main application entry point
    println!("Starting switchboard...");

    // Load configuration from environment variables and .env file
    // This returns a &'static Config
    let config = config::load_config();

    // Initialize tracing for structured logging
    logger::init_tracing(config);

    info!("switchboard initialized");

    // Create HTTP client with appropriate settings
    // Using rustls (instead of native-tls) for TLS implementation
    let client = reqwest::Client::builder()
        .use_rustls_tls() // Use rustls instead of native-tls
        .timeout(Duration::from_secs(600)) // 10-minute timeout for request completion (LLM responses can be lengthy)
        .connect_timeout(Duration::from_secs(10)) // 10-second timeout for connection establishment
        .pool_idle_timeout(Duration::from_secs(90)) // Keep connections in the pool for reuse
        .build()
        .map_err(|e| {
            error!("Failed to build reqwest client: {}", e);
            e
        })?;

    info!("HTTP client created with rustls TLS support");

    // Create a clone of the static config that we can own and wrap in Arc
    let config_owned = config.clone();
    let config_arc = Arc::new(config_owned);

    // Create the router with the HTTP client and config
    // Clone the Arc to preserve ownership for later use
    let app = create_router(client, config_arc.clone());

    // Parse and bind to the configured address
    let addr_str = format!("0.0.0.0:{}", config_arc.port);
    let addr: SocketAddr = match addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            error!(error = %e, addr = %addr_str, "Invalid listen address/port");
            return Err(e.into());
        }
    };

    // Bind to the configured port
    info!("Binding server to {}", addr);
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!(error = %e, addr = %addr, "Failed to bind to address");
            return Err(e.into());
        }
    };

    // Start the server with graceful shutdown
    info!("Starting Axum server, listening for requests");
    let server = Server::from_tcp(listener.into_std()?)?;

    if let Err(e) = server
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!(error = %e, "Server error");
        return Err(e.into());
    }

    info!("Server shutdown complete");
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
