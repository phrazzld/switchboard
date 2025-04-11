# Implement Graceful Shutdown Logic

## Implementation Approach
Implement an async `shutdown_signal` function in main.rs that will handle both SIGTERM signals (for container deployments) and Ctrl+C (for development environments). The function will use tokio::signal to wait for either signal and return when one is received. This will allow for graceful shutdown of the HTTP server by awaiting pending requests before shutting down.