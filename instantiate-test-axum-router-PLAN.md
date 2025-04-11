# Instantiate Test `axum::Router` in `setup_test_environment`

## Implementation Approach
Complete the `setup_test_environment` function by calling `switchboard::proxy_handler::create_router` with the test client and config. Use `Box::leak` for the config as a temporary measure to satisfy the `&'static Config` requirement. Populate and return the `TestSetup` struct with all components.