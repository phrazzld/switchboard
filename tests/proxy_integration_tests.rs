// Integration tests for the proxy handler
mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, HeaderValue, header};
use tower::ServiceExt;
use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use serde_json::{json, Value};

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
    
    // Assert that the response status code is 200 OK
    assert_eq!(response.status(), StatusCode::OK, 
        "Response status code should be 200 OK, got {}", response.status());
        
    // Extract the response body bytes
    // First we convert the response to bytes
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        
    // Deserialize the response body as JSON
    let body_json: Value = serde_json::from_slice(&body)
        .expect("Response body should be valid JSON");
        
    // Create the expected JSON response for comparison
    let expected_json = json!({"status": "ok"});
    
    // Assert that the response body matches our expected JSON
    assert_eq!(body_json, expected_json, 
        "Response body should match expected JSON. Got: {}", body_json);
}
