// Common test utilities for integration tests

use wiremock::MockServer;
use switchboard::config::Config;
use reqwest::Client;
use axum::Router;

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