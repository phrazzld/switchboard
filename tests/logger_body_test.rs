use bytes::Bytes;
use hyper::{header::HeaderMap, http::Method, Uri};
use reqwest::StatusCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use switchboard::proxy_handler::{log_request_details, log_response_details};
use tracing::{info, Level, Subscriber};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, Layer};

// Define a memory capture layer for testing, similar to the one in logger_stdout_test.rs
struct MemoryCapture {
    buffer: Arc<Mutex<Vec<String>>>,
    level: Level,
}

impl MemoryCapture {
    fn new(level: Level) -> Self {
        MemoryCapture {
            buffer: Arc::new(Mutex::new(Vec::new())),
            level,
        }
    }

    fn buffer(&self) -> Arc<Mutex<Vec<String>>> {
        self.buffer.clone()
    }
}

impl<S> Layer<S> for MemoryCapture
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Check if the event's level is high enough for our filter
        if event.metadata().level() > &self.level {
            return;
        }

        // Visit the event to extract its message and fields
        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);
        let formatted_message = visitor.finish();

        // Store the formatted message in the buffer
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push(formatted_message);
        }
    }
}

// Log visitor to extract message and fields
#[derive(Default)]
struct LogVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl LogVisitor {
    fn finish(self) -> String {
        // Format the message and fields for easy testing
        let mut result = self.message.unwrap_or_default();

        if !self.fields.is_empty() {
            result.push_str(" {");
            for (i, (key, value)) in self.fields.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                result.push_str(&format!("{}={}", key, value));
            }
            result.push('}');
        }

        result
    }
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value).trim_matches('"').to_string());
        } else {
            self.fields
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }
}

// Helper function to create a test subscriber
fn create_test_subscriber(
    level: Level,
) -> (impl Subscriber + Send + Sync, Arc<Mutex<Vec<String>>>) {
    let capture = MemoryCapture::new(level);
    let buffer = capture.buffer();
    let subscriber = tracing_subscriber::registry().with(capture);
    (subscriber, buffer)
}

// Helper function to generate a test body of the specified size
fn generate_test_body(size: usize) -> Bytes {
    let mut body = String::with_capacity(size);
    // Create valid JSON content for testing
    body.push_str(r#"{"message":"#);
    while body.len() < size - 2 {
        body.push('a');
    }
    body.push('}');
    Bytes::from(body)
}

// Helper functions to check if logs contain specific content
fn logs_contain(logs: &[String], text: &str) -> bool {
    logs.iter().any(|log| log.contains(text))
}

fn logs_contain_body_content(logs: &[String]) -> bool {
    logs.iter().any(|log| log.contains("body.content"))
}

fn logs_contain_body_size(logs: &[String]) -> bool {
    logs.iter().any(|log| log.contains("body.size"))
}

// Tests for request body logging behavior

#[test]
fn test_request_body_logging_enabled() {
    // Set up the test subscriber with debug level
    let (subscriber, buffer) = create_test_subscriber(Level::DEBUG);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Create a test request
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();
    let body = generate_test_body(100); // Small body
    let log_bodies = true;
    let log_max_body_size = 1000;

    // Log the request details
    log_request_details(
        &method,
        &uri,
        &headers,
        &body,
        log_bodies,
        log_max_body_size,
    );

    // Allow time for logs to be processed
    std::thread::sleep(Duration::from_millis(50));

    // Get the captured logs
    let captured = buffer.lock().unwrap();
    let logs: Vec<String> = captured.clone();

    // Print logs for debugging
    println!("Captured {} logs:", logs.len());
    for log in &logs {
        println!(" -> {}", log);
    }

    // Verify the body content is included in the logs
    assert!(
        logs_contain_body_content(&logs),
        "Body content should be logged when log_bodies is true"
    );

    // Verify the body size is logged
    assert!(logs_contain_body_size(&logs), "Body size should be logged");
}

#[test]
fn test_request_body_logging_disabled() {
    // Set up the test subscriber with debug level
    let (subscriber, buffer) = create_test_subscriber(Level::DEBUG);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Create a test request
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();
    let body = generate_test_body(100); // Small body
    let log_bodies = false;
    let log_max_body_size = 1000;

    // Log the request details
    log_request_details(
        &method,
        &uri,
        &headers,
        &body,
        log_bodies,
        log_max_body_size,
    );

    // Allow time for logs to be processed
    std::thread::sleep(Duration::from_millis(50));

    // Get the captured logs
    let captured = buffer.lock().unwrap();
    let logs: Vec<String> = captured.clone();

    // Print logs for debugging
    println!("Captured {} logs:", logs.len());
    for log in &logs {
        println!(" -> {}", log);
    }

    // Verify the body content is NOT included in the logs
    assert!(
        !logs_contain_body_content(&logs),
        "Body content should not be logged when log_bodies is false"
    );

    // Verify a message indicating body logging is disabled
    assert!(
        logs_contain(&logs, "Request body not logged"),
        "Should indicate that body logging is disabled"
    );

    // Verify the body size is still logged
    assert!(
        logs_contain_body_size(&logs),
        "Body size should still be logged even when content logging is disabled"
    );
}

#[test]
fn test_request_body_size_limit() {
    // Set up the test subscriber with debug level
    let (subscriber, buffer) = create_test_subscriber(Level::DEBUG);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Create a test request with body larger than the limit
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();
    let body = generate_test_body(1000); // 1000 bytes
    let log_bodies = true;
    let log_max_body_size = 500; // Limit to 500 bytes

    // Log the request details
    log_request_details(
        &method,
        &uri,
        &headers,
        &body,
        log_bodies,
        log_max_body_size,
    );

    // Allow time for logs to be processed
    std::thread::sleep(Duration::from_millis(50));

    // Get the captured logs
    let captured = buffer.lock().unwrap();
    let logs: Vec<String> = captured.clone();

    // Print logs for debugging
    println!("Captured {} logs:", logs.len());
    for log in &logs {
        println!(" -> {}", log);
    }

    // Verify body content is NOT included since it exceeds the limit
    assert!(
        !logs_contain_body_content(&logs),
        "Body content should not be logged when it exceeds the size limit"
    );

    // Verify there's a message indicating the body is too large
    assert!(
        logs_contain(&logs, "too large to log fully"),
        "Should indicate that the body is too large to log"
    );

    // Verify the body size is logged
    assert!(logs_contain_body_size(&logs), "Body size should be logged");
}

// Tests for response body logging behavior

#[test]
fn test_response_body_logging_enabled() {
    // Set up the test subscriber with debug level
    let (subscriber, buffer) = create_test_subscriber(Level::DEBUG);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Create a test response
    let status = StatusCode::OK;
    let headers = HeaderMap::new();
    let body = generate_test_body(100); // Small body
    let log_bodies = true;
    let log_max_body_size = 1000;

    // Log the response details
    log_response_details(
        &status,
        &headers,
        &body,
        log_bodies,
        log_max_body_size,
        None,
    );

    // Allow time for logs to be processed
    std::thread::sleep(Duration::from_millis(50));

    // Get the captured logs
    let captured = buffer.lock().unwrap();
    let logs: Vec<String> = captured.clone();

    // Print logs for debugging
    println!("Captured {} logs:", logs.len());
    for log in &logs {
        println!(" -> {}", log);
    }

    // Verify the body content is included in the logs
    assert!(
        logs_contain_body_content(&logs),
        "Body content should be logged when log_bodies is true"
    );

    // Verify the body size is logged
    assert!(logs_contain_body_size(&logs), "Body size should be logged");
}

#[test]
fn test_response_body_logging_disabled() {
    // Set up the test subscriber with debug level
    let (subscriber, buffer) = create_test_subscriber(Level::DEBUG);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Create a test response
    let status = StatusCode::OK;
    let headers = HeaderMap::new();
    let body = generate_test_body(100); // Small body
    let log_bodies = false;
    let log_max_body_size = 1000;

    // Log the response details
    log_response_details(
        &status,
        &headers,
        &body,
        log_bodies,
        log_max_body_size,
        None,
    );

    // Allow time for logs to be processed
    std::thread::sleep(Duration::from_millis(50));

    // Get the captured logs
    let captured = buffer.lock().unwrap();
    let logs: Vec<String> = captured.clone();

    // Print logs for debugging
    println!("Captured {} logs:", logs.len());
    for log in &logs {
        println!(" -> {}", log);
    }

    // Verify the body content is NOT included in the logs
    assert!(
        !logs_contain_body_content(&logs),
        "Body content should not be logged when log_bodies is false"
    );

    // Verify a message indicating body logging is disabled
    assert!(
        logs_contain(&logs, "Response body not logged"),
        "Should indicate that body logging is disabled"
    );

    // Verify the body size is still logged
    assert!(
        logs_contain_body_size(&logs),
        "Body size should still be logged even when content logging is disabled"
    );
}

#[test]
fn test_response_body_size_limit() {
    // Set up the test subscriber with debug level
    let (subscriber, buffer) = create_test_subscriber(Level::DEBUG);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Create a test response with body larger than the limit
    let status = StatusCode::OK;
    let headers = HeaderMap::new();
    let body = generate_test_body(1000); // 1000 bytes
    let log_bodies = true;
    let log_max_body_size = 500; // Limit to 500 bytes

    // Log the response details
    log_response_details(
        &status,
        &headers,
        &body,
        log_bodies,
        log_max_body_size,
        None,
    );

    // Allow time for logs to be processed
    std::thread::sleep(Duration::from_millis(50));

    // Get the captured logs
    let captured = buffer.lock().unwrap();
    let logs: Vec<String> = captured.clone();

    // Print logs for debugging
    println!("Captured {} logs:", logs.len());
    for log in &logs {
        println!(" -> {}", log);
    }

    // Verify body content is NOT included since it exceeds the limit
    assert!(
        !logs_contain_body_content(&logs),
        "Body content should not be logged when it exceeds the size limit"
    );

    // Verify there's a message indicating the body is too large
    assert!(
        logs_contain(&logs, "too large to log fully"),
        "Should indicate that the body is too large to log"
    );

    // Verify the body size is logged
    assert!(logs_contain_body_size(&logs), "Body size should be logged");
}

#[test]
fn test_body_exactly_at_size_limit() {
    // Set up the test subscriber with debug level
    let (subscriber, buffer) = create_test_subscriber(Level::DEBUG);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Size limit for this test
    let size_limit = 100;

    // Create a test body exactly at the limit
    let body = generate_test_body(size_limit);
    let log_bodies = true;

    // Test for both request and response at exactly the limit

    // 1. Request at limit
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();

    info!("Testing request body exactly at size limit");
    log_request_details(&method, &uri, &headers, &body, log_bodies, size_limit);

    // 2. Response at limit
    let status = StatusCode::OK;

    info!("Testing response body exactly at size limit");
    log_response_details(&status, &headers, &body, log_bodies, size_limit, None);

    // Allow time for logs to be processed
    std::thread::sleep(Duration::from_millis(50));

    // Get the captured logs
    let captured = buffer.lock().unwrap();
    let logs: Vec<String> = captured.clone();

    // Print logs for debugging
    println!("Captured {} logs:", logs.len());
    for log in &logs {
        println!(" -> {}", log);
    }

    // Bodies exactly at the limit should be logged
    assert!(
        logs_contain_body_content(&logs),
        "Body content should be logged when exactly at the size limit"
    );

    // Verify the correct size is logged
    assert!(
        logs_contain_body_size(&logs),
        "Body size should be logged correctly"
    );

    // Verify there's no message about the body being too large
    assert!(
        !logs_contain(&logs, "too large to log fully"),
        "Should not indicate the body is too large when exactly at the limit"
    );
}

// Test for empty bodies

#[test]
fn test_empty_body_logging() {
    // Set up the test subscriber with info level
    let (subscriber, buffer) = create_test_subscriber(Level::INFO);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Create empty body
    let body = Bytes::new();
    let log_bodies = true;
    let log_max_body_size = 1000;

    // Test for both request and response with empty bodies

    // 1. Empty request body
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();

    log_request_details(
        &method,
        &uri,
        &headers,
        &body,
        log_bodies,
        log_max_body_size,
    );

    // 2. Empty response body
    let status = StatusCode::OK;

    log_response_details(
        &status,
        &headers,
        &body,
        log_bodies,
        log_max_body_size,
        None,
    );

    // Allow time for logs to be processed
    std::thread::sleep(Duration::from_millis(50));

    // Get the captured logs
    let captured = buffer.lock().unwrap();
    let logs: Vec<String> = captured.clone();

    // Print logs for debugging
    println!("Captured {} logs:", logs.len());
    for log in &logs {
        println!(" -> {}", log);
    }

    // Verify empty body is logged correctly
    assert!(
        logs_contain(&logs, "Request body empty"),
        "Should indicate that request body is empty"
    );

    assert!(
        logs_contain(&logs, "Response body empty"),
        "Should indicate that response body is empty"
    );

    // Verify no body content is included
    assert!(
        !logs_contain_body_content(&logs),
        "No body content should be logged for empty bodies"
    );
}
