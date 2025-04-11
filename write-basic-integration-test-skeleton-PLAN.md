# Write Basic Integration Test Skeleton in `proxy_integration_tests.rs`

## Implementation Approach
Create an async test function `test_simple_post_forward_success()` with the `#[tokio::test]` attribute in `proxy_integration_tests.rs`. Add the necessary imports for Axum, HTTP types, and Tower utilities. Call `common::setup_test_environment().await` to setup the test environment.