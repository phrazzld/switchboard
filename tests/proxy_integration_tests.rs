// Integration tests for the proxy handler
mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, HeaderValue, header};
use tower::ServiceExt;
use http_body_util::BodyExt;
use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use serde_json::json;

/// Tests that a simple POST request to /v1/messages is correctly forwarded
/// to the Anthropic API and the response is returned to the client.
#[tokio::test]
async fn test_simple_post_forward_success() {
    // Set up the test environment with mock server, client, config, and router
    let test_setup = common::setup_test_environment().await;
    
    // Define a mock expectation for the Anthropic API endpoint
    // This mock will match POST requests to the /v1/messages path
    // and respond with a 200 OK status and a simple JSON response
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"status": "ok"})))
        .mount(&test_setup.mock_server)
        .await;
    
    // Create a sample JSON request body for the Anthropic Messages API
    let request_body = json!({
        "model": "claude-3-opus-20240229",
        "messages": [
            {
                "role": "user",
                "content": "Hello, Claude!"
            }
        ]
    });
    
    // Construct the HTTP request
    let request = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    
    // Send the request to our test router instance using oneshot
    // oneshot consumes the request and sends it through the service
    let response = test_setup.app.oneshot(request).await.unwrap();
    
    // The assertions for the response will be added in the next task
}
