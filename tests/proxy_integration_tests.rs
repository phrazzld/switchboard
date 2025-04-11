// Integration tests for the proxy handler
mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, HeaderValue, header};
use tower::ServiceExt;
use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use serde_json::{json, Value};
use futures_util::StreamExt;

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

/// Tests that streaming responses from the Anthropic API are correctly
/// processed and forwarded to the client with proper streaming semantics.
#[tokio::test]
async fn test_streaming_response_forward_success() {
    // Set up the test environment with mock server, client, config, and router
    let test_setup = common::setup_test_environment().await;
    
    // Define a streaming response template
    // This simulates how the Anthropic API would send a streaming response
    // with multiple event chunks using Server-Sent Events (SSE) format
    let streaming_response = ResponseTemplate::new(200)
        // Set content type to text/event-stream which is standard for SSE
        .insert_header("content-type", "text/event-stream")
        // Disable content-length header to enable chunked transfer encoding
        .insert_header("transfer-encoding", "chunked")
        // Each chunk is formatted as SSE: "data: {...}\n\n"
        .set_body_bytes(concat!(
            "data: {\"type\": \"message_start\", \"message\": {\"id\": \"msg_123\", \"type\": \"message\"}}\n\n",
            "data: {\"type\": \"content_block_start\", \"index\": 0, \"content_block\": {\"type\": \"text\"}}\n\n",
            "data: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"text_delta\", \"text\": \"Hello\"}}\n\n",
            "data: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"text_delta\", \"text\": \" world\"}}\n\n",
            "data: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"text_delta\", \"text\": \"!\"}}\n\n",
            "data: {\"type\": \"content_block_stop\", \"index\": 0}\n\n",
            "data: {\"type\": \"message_stop\"}\n\n"
        ));
    
    // Set up the mock to respond with our streaming response
    // when receiving a POST request to /v1/messages
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(streaming_response)
        .mount(&test_setup.mock_server)
        .await;
    
    // Create a sample JSON request body for the Anthropic Messages API with streaming enabled
    let request_body = json!({
        "model": "claude-3-opus-20240229",
        "messages": [
            {
                "role": "user",
                "content": "Hello, Claude!"
            }
        ],
        "stream": true  // Enable streaming in the request
    });
    
    // Construct the HTTP request
    let request = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();
    
    // Send the request to our test router instance using oneshot
    let response = test_setup.app.oneshot(request).await.unwrap();
    
    // Assert that the response status code is 200 OK
    assert_eq!(response.status(), StatusCode::OK, 
        "Response status code should be 200 OK, got {}", response.status());
    
    // Verify the response has the correct content type for streaming responses
    let content_type = response.headers().get(header::CONTENT_TYPE)
        .expect("Response should have a Content-Type header");
    assert_eq!(content_type, "text/event-stream", 
        "Content-Type should be text/event-stream, got {}", content_type.to_str().unwrap());
    
    // The streaming response verification will be implemented in the next task
    // This will involve processing chunks as they arrive and validating the streaming behavior
}
