mod config;
mod proxy_handler;
mod logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Main application entry point
    println!("Starting Anthropic visibility proxy...");
    
    // Load configuration from environment variables and .env file
    let config = config::load_config();
    
    // Initialize tracing for structured logging
    logger::init_tracing(config);
    
    tracing::info!("Anthropic visibility proxy initialized");
    
    Ok(())
}
