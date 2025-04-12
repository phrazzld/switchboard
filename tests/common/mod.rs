// Common test utilities for integration tests

use axum::Router;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use switchboard::config::Config;
use switchboard::proxy_handler::create_router;
use wiremock::MockServer;

/// Represents the setup needed for integration tests.
pub struct TestSetup {
    /// HTTP client for the application to use
    pub client: Client,

    /// Application configuration pointing to the mock server
    pub config: Config,

    /// The WireMock server instance
    pub mock_server: MockServer,

    /// The Axum router configured for testing
    pub app: Router,
}

/// Sets up the test environment with all necessary components for integration testing.
///
/// This function:
/// 1. Starts a WireMock server to mock the Anthropic API
/// 2. Creates a test-specific configuration pointing to the mock server
/// 3. Creates a reqwest Client with appropriate timeouts
/// 4. Instantiates the application router using the test client and config
///
/// Returns a TestSetup instance containing all components needed for testing.
pub async fn setup_test_environment() -> TestSetup {
    // Start a WireMock server on a random available port
    // This will be used to mock the Anthropic API during tests
    let mock_server = MockServer::start().await;

    // Create a test-specific configuration pointing to the mock server
    // Use dummy values for fields that are suitable for testing
    let config = Config {
        port: "0".to_string(), // Use 0 to let OS assign a random port if needed
        anthropic_api_key: "test-api-key".to_string(), // Dummy API key for testing
        anthropic_target_url: mock_server.uri(), // Point to the mock server
        log_level: "debug".to_string(), // Use debug level for more verbose test logs
        log_format: "pretty".to_string(), // Use pretty format for readability in tests
    };

    // Create a reqwest client with appropriate timeouts for testing
    // Using shorter timeouts than production to avoid long-running tests
    let client = Client::builder()
        .timeout(Duration::from_secs(5)) // Overall request timeout
        .connect_timeout(Duration::from_secs(2)) // Connection establishment timeout
        .pool_idle_timeout(Duration::from_secs(10)) // Keep idle connections for reuse
        .build()
        .expect("Failed to build test reqwest client");

    // Create a thread-safe reference-counted config
    // This avoids the memory leak from Box::leak while still allowing sharing
    let config_arc = Arc::new(config.clone());

    // Create the application router with our test client and config
    let app = create_router(client.clone(), config_arc);

    // Return the complete TestSetup with all components
    TestSetup {
        client,
        config,
        mock_server,
        app,
    }
}
