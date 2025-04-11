# Setup Axum Server

## Implementation Approach
Implement server startup logic in main.rs to create an HTTP server using Axum. This will involve:
1. Parsing the listen address from config.port
2. Binding a TcpListener to the address
3. Creating a basic placeholder router via a minimal implementation of proxy_handler::create_router
4. Running the server using axum::serve with graceful shutdown support (integrating the existing shutdown_signal function)
5. Adding appropriate error handling for address parsing and port binding

The implementation will focus on setting up the basic HTTP server infrastructure without yet implementing the full proxy functionality. The proxy_handler module will be updated with placeholder functions to support the server setup.