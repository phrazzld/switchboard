// This module will contain the proxy handler implementation
// Core proxy functionality for intercepting and forwarding API requests

use axum::{
    body::{boxed, Body, Full},
    http::StatusCode,
    response::Response,
    routing::any,
    Router,
};
use bytes::Bytes;
use futures_util::StreamExt;
use hyper::{header, HeaderMap, Request, Uri};
use reqwest::{header::HeaderValue as ReqHeaderValue, Client};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, field, info, info_span, instrument, Span};
use uuid::Uuid;

use crate::config::Config;

/// Minimal representation of an Anthropic Messages API request
///
/// This struct is used only for logging context, not for processing.
/// It extracts only the essential fields needed for logging identification.
///
/// # Note
/// Currently not actively used but prepared for future logging enhancements.
/// The fields will be used to provide better contextual information in logs.
#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Explicitly suppressed as this struct is prepared for future use
struct AnthropicMessagesRequestMinimal {
    /// The model being requested (claude-3-opus, claude-3-sonnet, etc.)
    model: Option<String>,

    /// Whether the request is for a streaming response
    stream: Option<bool>,
    // Other fields could be added if needed for better logging context
    // For example, the number of messages in the conversation
    // messages: Option<Vec<Value>>, // Not included by default as it would be verbose
}

/// Creates the Axum router with routes for the application
///
/// Sets up an Axum router with a catch-all route that forwards all
/// incoming requests to the proxy_handler function regardless of
/// HTTP method (GET, POST, etc.)
///
/// # Arguments
///
/// * `client` - The HTTP client used to make requests to the upstream API
/// * `config` - Configuration wrapped in an Arc for thread-safe sharing
pub fn create_router(client: Client, config: Arc<Config>) -> Router {
    info!("Creating Axum router with catch-all route to proxy_handler");

    Router::new().route(
        "/*path", // Catch-all route
        any(move |req: Request<Body>| {
            let config = Arc::clone(&config);
            proxy_handler(req, client.clone(), config)
        }),
    )
}

/// The main proxy handler function that processes incoming requests
///
/// This function:
/// 1. Receives an incoming request
/// 2. Assigns a unique request ID for tracing
/// 3. Records basic request information in the tracing span
/// 4. Forwards the request to the Anthropic API and returns the response
///
/// # Arguments
///
/// * `req` - The incoming HTTP request to be proxied
/// * `client` - The HTTP client used to make requests to the upstream API
/// * `config` - Configuration wrapped in an Arc for thread-safe sharing
///
/// The #[instrument] macro automatically creates a tracing span for this function,
/// with empty fields that will be filled in during processing.
#[instrument(
    skip_all,                                  // Don't include the function arguments in the span
    name = "proxy_request",                    // Name the span 'proxy_request'
    fields(
        req_id = field::Empty,                 // Unique ID for this request
        http.method = field::Empty,            // HTTP method (GET, POST, etc.)
        url.path = field::Empty,               // Request path
        url.query = field::Empty,              // Query parameters
        http.status_code = field::Empty,       // Response status code
        duration_ms = field::Empty             // Total request duration
    )
)]
pub async fn proxy_handler(
    req: Request<Body>,
    client: Client,
    config: Arc<Config>,
) -> Result<Response, StatusCode> {
    // Start timing the request processing
    let start = Instant::now();

    // Generate a unique ID for this request
    let req_id = Uuid::new_v4();

    // Get the current span created by the #[instrument] macro
    let span = Span::current();

    // Record the request ID in the span
    span.record("req_id", req_id.to_string());

    info!(request_id = %req_id, "Starting request processing");

    // Extract and clone the essential request information
    let original_uri = req.uri().clone();
    let method = req.method().clone();
    let original_headers = req.headers().clone();

    // Record basic request information in the tracing span
    span.record("http.method", method.to_string());
    span.record("url.path", original_uri.path());

    // If there's a query string, record it in the span
    if let Some(query) = original_uri.query() {
        span.record("url.query", query);
    }

    // Extract the path and query, defaulting to "/" if none
    let path_and_query = original_uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    info!(
        method = %method,
        path = %original_uri.path(),
        query = %original_uri.query().unwrap_or(""),
        "Processing request"
    );

    // Construct the target Anthropic API URL
    let target_url_str = format!("{}{}", config.anthropic_target_url, path_and_query);

    // Parse the constructed URL into a Uri
    let target_url = match target_url_str.parse::<Uri>() {
        Ok(uri) => {
            info!(target_url = %uri, "Target URL constructed successfully");
            uri
        }
        Err(e) => {
            // Log the error with context and return an error status
            error!(
                error = %e,
                attempted_url = %target_url_str,
                "Failed to parse target URL"
            );

            // Record the error status in the span
            span.record(
                "http.status_code",
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            );

            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Convert the request body to bytes for processing
    // The usize::MAX parameter means we'll read the entire body, no matter how large
    let body_bytes_result = hyper::body::to_bytes(req.into_body()).await;

    // Handle any errors that might occur during body extraction
    // The extracted body bytes will be used in future implementations
    let body_bytes = match body_bytes_result {
        Ok(bytes) => {
            info!(body_size = bytes.len(), "Request body read successfully");
            bytes
        }
        Err(e) => {
            // Log the error and return a BAD_REQUEST status
            error!(error = %e, "Failed to read request body");

            // Record the error status in the span
            span.record("http.status_code", StatusCode::BAD_REQUEST.as_u16());

            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Log detailed request information including headers and body
    log_request_details(&method, &original_uri, &original_headers, &body_bytes);

    // Create the request builder for forwarding to Anthropic API
    info!("Setting up request forwarding to Anthropic API");
    let mut forward_req_builder = client.request(method.clone(), target_url.to_string());

    // Create new headers for the forwarded request
    let mut forward_headers = HeaderMap::new();

    // Copy original headers, filtering out hop-by-hop headers
    for (name, value) in original_headers.iter() {
        // Filter out hop-by-hop headers that shouldn't be forwarded
        if name != header::HOST
            && name != header::CONNECTION
            && name != header::PROXY_AUTHENTICATE
            && name != header::PROXY_AUTHORIZATION
            && name != header::TE
            && name != header::TRAILER
            && name != header::TRANSFER_ENCODING
            && name != header::UPGRADE
        {
            forward_headers.insert(name.clone(), value.clone());
        }
    }

    // Set the Host header based on the target URL
    if let Some(host) = target_url.host() {
        match ReqHeaderValue::from_str(host) {
            Ok(host_value) => {
                forward_headers.insert(header::HOST, host_value);
            }
            Err(e) => {
                error!(error = %e, host = %host, "Failed to create Host header value");
                // Continue without setting Host header - reqwest will handle it
            }
        }
    }

    // Set the Anthropic API key as x-api-key header
    match ReqHeaderValue::from_str(&config.anthropic_api_key) {
        Ok(api_key_value) => {
            // Add the API key header
            forward_headers.insert(header::HeaderName::from_static("x-api-key"), api_key_value);

            // Remove Authorization header if it exists (x-api-key is preferred by Anthropic)
            forward_headers.remove(header::AUTHORIZATION);
        }
        Err(e) => {
            error!(error = %e, "Failed to create header value for Anthropic API key");
            span.record(
                "http.status_code",
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Add the headers to the request builder
    forward_req_builder = forward_req_builder.headers(forward_headers);

    // Add the request body to the builder
    forward_req_builder = forward_req_builder.body(body_bytes);

    // Store the builder for the next step (actually sending the request)
    info!("Request forwarding setup complete");

    // Send the request to the Anthropic API
    info!("Sending request to Anthropic API");
    let forward_resp_result = forward_req_builder.send().await;

    // Check if the request was successful
    let forward_resp = match forward_resp_result {
        Ok(resp) => {
            info!(
                status = %resp.status(),
                "Received response from Anthropic API"
            );
            resp
        }
        Err(e) => {
            // Log the error with context
            error!(
                error = %e,
                "Failed to send request to Anthropic API"
            );

            // Record the error status in the span
            span.record("http.status_code", StatusCode::BAD_GATEWAY.as_u16());

            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    // Extract the status code and headers from the response
    let resp_status = forward_resp.status();
    let resp_headers = forward_resp.headers().clone();

    // Record the response status code in the span for observability
    span.record("http.status_code", resp_status.as_u16());

    // Log detailed information about the response
    info!(
        request_id = %req_id,
        status = %resp_status,
        headers_count = resp_headers.len(),
        "Received response from Anthropic API with status {}", resp_status
    );

    // Log headers at debug level (won't show in normal operation)
    debug!(
        request_id = %req_id,
        status = %resp_status,
        headers = ?resp_headers,
        "Response headers from Anthropic API"
    );

    // Check if this is a streaming response by examining Content-Type header
    let is_streaming = resp_headers
        .get(header::CONTENT_TYPE)
        .map(|ct| ct.to_str().unwrap_or("").contains("text/event-stream"))
        .unwrap_or(false);

    if is_streaming {
        // Log headers for streaming response
        info!(
            request_id = %req_id,
            "Detected streaming response from Anthropic API"
        );

        // Call the header logging helper to log status and headers
        log_response_headers(&resp_status, &resp_headers);

        // Create a stream from the reqwest response
        info!(
            request_id = %req_id,
            "Creating stream from Anthropic API response"
        );

        // Get the bytes stream from the reqwest response
        let reqwest_stream = forward_resp.bytes_stream();

        // Convert reqwest stream to axum stream by mapping each chunk
        // and handling errors appropriately
        let axum_stream = reqwest_stream.map(move |result| match result {
            Ok(bytes) => {
                // On success, log the chunk size and return the bytes
                debug!(
                    request_id = %req_id,
                    chunk_size = bytes.len(),
                    "Received stream chunk from Anthropic API"
                );
                Ok::<_, axum::BoxError>(bytes)
            }
            Err(e) => {
                // On error, log it and convert to axum::BoxError
                error!(
                    request_id = %req_id,
                    error = %e,
                    "Error reading streaming response chunk from Anthropic API"
                );
                Err(axum::BoxError::from(format!("Stream error: {}", e)))
            }
        });

        // Create the Axum body from the stream
        let stream_body = Body::wrap_stream(axum_stream);

        info!(
            request_id = %req_id,
            "Successfully created streaming body for client response"
        );

        // Start building the streaming response with the original status code
        info!(
            request_id = %req_id,
            status = %resp_status,
            "Forwarding streaming response to client"
        );

        // Start building the response with the same status code
        let mut response_builder = Response::builder().status(resp_status);

        // Copy the headers from the Anthropic API response, excluding hop-by-hop headers
        // For streaming responses, we also exclude Content-Length as it's not applicable
        for (name, value) in resp_headers.iter() {
            // Filter out hop-by-hop headers that shouldn't be forwarded back
            // and Content-Length which doesn't apply to streaming responses
            if name != header::CONNECTION
                && name != header::PROXY_AUTHENTICATE
                && name != header::PROXY_AUTHORIZATION
                && name != header::TE
                && name != header::TRAILER
                && name != header::TRANSFER_ENCODING
                && name != header::UPGRADE
                && name != header::HOST
                && name != header::CONTENT_LENGTH
            {
                // Add the header to our response
                response_builder = response_builder.header(name.clone(), value.clone());
            }
        }

        // For streaming responses, we ensure the correct Content-Type is set
        // This is critical for the client to recognize it as a stream
        if !resp_headers.contains_key(header::CONTENT_TYPE) {
            response_builder = response_builder.header(header::CONTENT_TYPE, "text/event-stream");
        }

        // Build the final streaming response with the body
        match response_builder.body(boxed(stream_body)) {
            Ok(response) => {
                // Calculate the elapsed time since the request started
                let duration = start.elapsed();

                // Record the duration in milliseconds in the span for observability
                span.record("duration_ms", duration.as_millis());

                info!(
                    request_id = %req_id,
                    duration_ms = %duration.as_millis(),
                    "Successfully built streaming client response"
                );
                Ok(response)
            }
            Err(e) => {
                // This is unlikely to happen but we should handle it
                error!(
                    request_id = %req_id,
                    error = %e,
                    "Failed to build streaming response"
                );

                // Record the error status in the span
                span.record(
                    "http.status_code",
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                );

                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        // Handle non-streaming response by reading the full body
        info!(
            request_id = %req_id,
            "Handling non-streaming response from Anthropic API"
        );

        // Read the full response body
        let resp_body_bytes_result = forward_resp.bytes().await;

        // Handle any errors that might occur during body extraction
        let resp_body_bytes = match resp_body_bytes_result {
            Ok(bytes) => {
                info!(
                    request_id = %req_id,
                    body_size = bytes.len(),
                    "Response body read successfully"
                );
                bytes
            }
            Err(e) => {
                // Log the error with context
                error!(
                    request_id = %req_id,
                    error = %e,
                    "Failed to read response body from Anthropic API"
                );

                // Record the error status in the span
                span.record("http.status_code", StatusCode::BAD_GATEWAY.as_u16());

                return Err(StatusCode::BAD_GATEWAY);
            }
        };

        // Log detailed response information including headers and body
        log_response_details(&resp_status, &resp_headers, &resp_body_bytes);

        // Build the response to return to the client
        info!(
            request_id = %req_id,
            status = %resp_status,
            body_size = resp_body_bytes.len(),
            "Forwarding non-streaming response to client"
        );

        // Start building the response with the same status code
        let mut response_builder = Response::builder().status(resp_status);

        // Copy the headers from the Anthropic API response, excluding hop-by-hop headers
        for (name, value) in resp_headers.iter() {
            // Filter out hop-by-hop headers that shouldn't be forwarded back
            if name != header::CONNECTION
                && name != header::PROXY_AUTHENTICATE
                && name != header::PROXY_AUTHORIZATION
                && name != header::TE
                && name != header::TRAILER
                && name != header::TRANSFER_ENCODING
                && name != header::UPGRADE
                // Also don't forward the host header in the response
                && name != header::HOST
            {
                // Add the header to our response
                response_builder = response_builder.header(name.clone(), value.clone());
            }
        }

        // Explicitly set the Content-Length header based on the response body size
        response_builder =
            response_builder.header(header::CONTENT_LENGTH, resp_body_bytes.len().to_string());

        // Build the final response with the body
        // Converting the body to a boxed body to make it compatible with axum's expectations
        match response_builder.body(boxed(Full::from(resp_body_bytes))) {
            Ok(response) => {
                // Calculate the elapsed time since the request started
                let duration = start.elapsed();

                // Record the duration in milliseconds in the span for observability
                span.record("duration_ms", duration.as_millis());

                info!(
                    request_id = %req_id,
                    duration_ms = %duration.as_millis(),
                    "Successfully built client response"
                );
                Ok(response)
            }
            Err(e) => {
                // This is unlikely to happen but we should handle it
                error!(
                    request_id = %req_id,
                    error = %e,
                    "Failed to build response"
                );

                // Record the error status in the span
                span.record(
                    "http.status_code",
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                );

                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Maximum length of request/response bodies that will be logged in full
/// Bodies larger than this will only have their size logged to avoid excessive logging
pub const MAX_LOG_BODY_LEN: usize = 10 * 1024; // 10KB

/// Logs details of an incoming request in a structured format
///
/// This function creates a new logging span and records comprehensive information about
/// the request, including method, URI, headers (with sensitive values masked), and the
/// request body (with size limits and JSON formatting).
///
/// # Arguments
/// * `method` - The HTTP method (GET, POST, etc.)
/// * `uri` - The request URI including path and query
/// * `headers` - The request headers map
/// * `body` - The request body as bytes
pub fn log_request_details(method: &hyper::Method, uri: &Uri, headers: &HeaderMap, body: &Bytes) {
    // Create a new span for the request details to keep them separate from the main request span
    let span = info_span!("request_details");
    let _enter = span.enter();

    // Log basic request information at the info level
    info!(http.method = %method, url.full = %uri);

    // Build a map of header names to values, masking sensitive headers
    let mut headers_log: HashMap<String, String> = HashMap::new();
    for (name, value) in headers.iter() {
        let name_str = name.to_string();
        // Mask sensitive authentication headers
        let value_str = if name == header::AUTHORIZATION || name == "x-api-key" {
            "[REDACTED]".to_string()
        } else {
            // Convert header value to string (lossy UTF-8 conversion if needed)
            String::from_utf8_lossy(value.as_bytes()).to_string()
        };
        headers_log.insert(name_str, value_str);
    }

    // Log all headers at debug level (won't show in normal operation)
    debug!(http.request.headers = ?headers_log);

    // Log the request body with appropriate handling based on size
    let body_len = body.len();

    if body_len == 0 {
        // Empty body
        info!("Request body empty");
    } else if body_len <= MAX_LOG_BODY_LEN {
        // Body is small enough to log fully
        // Try to parse as JSON first for pretty formatting
        match serde_json::from_slice::<Value>(body) {
            Ok(json_val) => {
                // Successfully parsed as JSON, pretty print it
                let pretty_json = serde_json::to_string_pretty(&json_val)
                    .unwrap_or_else(|_| String::from_utf8_lossy(body).to_string());
                debug!(
                    http.request.body.content = %pretty_json,
                    http.request.body.size = body_len
                );
            }
            Err(_) => {
                // Not valid JSON, log as regular string
                debug!(
                    http.request.body.content = %String::from_utf8_lossy(body),
                    http.request.body.size = body_len
                );
            }
        }
    } else {
        // Body too large to log fully
        info!(
            http.request.body.size = body_len,
            "Request body too large to log fully"
        );
    }
}

/// Logs details of an API response in a structured format
///
/// This function creates a new logging span and records comprehensive information about
/// the response, including status code, headers, and the response body (with size limits
/// and JSON formatting).
///
/// # Arguments
/// * `status` - The HTTP status code
/// * `headers` - The response headers map
/// * `body` - The response body as bytes
pub fn log_response_details(status: &reqwest::StatusCode, headers: &HeaderMap, body: &Bytes) {
    // Create a new span for the response details to keep them separate from the main request span
    let span = info_span!("response_details");
    let _enter = span.enter();

    // Log basic response information at the info level
    info!(http.status_code = %status.as_u16(), status_text = %status.canonical_reason().unwrap_or("Unknown"));

    // Build a map of header names to values, masking sensitive headers
    let mut headers_log: HashMap<String, String> = HashMap::new();
    for (name, value) in headers.iter() {
        let name_str = name.to_string();
        // Mask sensitive headers if any (similar to request handling)
        let value_str = if name == header::AUTHORIZATION || name == "x-api-key" {
            "[REDACTED]".to_string()
        } else {
            // Convert header value to string (lossy UTF-8 conversion if needed)
            String::from_utf8_lossy(value.as_bytes()).to_string()
        };
        headers_log.insert(name_str, value_str);
    }

    // Log all headers at debug level (won't show in normal operation)
    debug!(http.response.headers = ?headers_log);

    // Log the response body with appropriate handling based on size
    let body_len = body.len();

    if body_len == 0 {
        // Empty body
        info!("Response body empty");
    } else if body_len <= MAX_LOG_BODY_LEN {
        // Body is small enough to log fully
        // Try to parse as JSON first for pretty formatting
        match serde_json::from_slice::<Value>(body) {
            Ok(json_val) => {
                // Successfully parsed as JSON, pretty print it
                let pretty_json = serde_json::to_string_pretty(&json_val)
                    .unwrap_or_else(|_| String::from_utf8_lossy(body).to_string());
                debug!(
                    http.response.body.content = %pretty_json,
                    http.response.body.size = body_len
                );
            }
            Err(_) => {
                // Not valid JSON, log as regular string
                debug!(
                    http.response.body.content = %String::from_utf8_lossy(body),
                    http.response.body.size = body_len
                );
            }
        }
    } else {
        // Body too large to log fully
        info!(
            http.response.body.size = body_len,
            "Response body too large to log fully"
        );
    }
}

/// Logs details of response headers for streaming responses
///
/// This function creates a new logging span and records the response status and headers,
/// without attempting to log the body (since the body will be streamed). This is specifically
/// designed for streaming responses where we want to log headers immediately before
/// starting to stream the response body.
///
/// # Arguments
/// * `status` - The HTTP status code of the response
/// * `headers` - The response headers map
pub fn log_response_headers(status: &reqwest::StatusCode, headers: &HeaderMap) {
    // Create a new span for the streaming response details
    let span = info_span!("streaming_response_details");
    let _enter = span.enter();

    // Log that streaming is starting
    info!(
        http.status_code = %status.as_u16(),
        status_text = %status.canonical_reason().unwrap_or("Unknown"),
        "Starting streaming response"
    );

    // Build a map of header names to values, masking sensitive headers
    let mut headers_log: HashMap<String, String> = HashMap::new();
    for (name, value) in headers.iter() {
        let name_str = name.to_string();
        // Mask sensitive authentication headers
        let value_str = if name == header::AUTHORIZATION || name == "x-api-key" {
            "[REDACTED]".to_string()
        } else {
            // Convert header value to string (lossy UTF-8 conversion if needed)
            String::from_utf8_lossy(value.as_bytes()).to_string()
        };
        headers_log.insert(name_str, value_str);
    }

    // Log all headers at debug level (won't show in normal operation)
    debug!(http.response.headers = ?headers_log);

    // Log a message indicating that we're about to start streaming
    info!("Headers logged, beginning to stream response body");
}
