## 1. Overview

**Goal:** Implement an extremely simple, highly observable HTTP reverse proxy in Rust. Its sole purpose is to intercept traffic between an Anthropic client (like Claude Code) and the official Anthropic Messages API (`https://api.anthropic.com`), logging *all* request and response details (headers, bodies) for visibility and debugging purposes.

**Core Functionality:**

* Listen for HTTP POST requests, primarily targeting `/v1/messages`.
* Receive the incoming request from the client (Claude Code).
* **Log** the full incoming request (method, path, headers, body).
* Forward the *exact* request (headers, body) to the configured Anthropic API endpoint.
* Receive the response from the Anthropic API.
* **Log** the full response (status code, headers, body).
* Forward the *exact* response (status, headers, body) back to the original client.
* Handle both standard JSON responses and streaming (Server-Sent Events) responses correctly by passing through the data chunks.
* Handle network errors during forwarding gracefully.

**Non-Goals (for this simplified version):**

* No translation between API formats (Anthropic <-> Gemini or anything else).
* No interaction with any API other than the official Anthropic API.
* No modification of request/response bodies (except potentially adding tracing headers if needed).
* No complex routing or state management.

## 2. Architecture

* **Simple Reverse Proxy:** A single Rust application acting as a pass-through layer.
* **Stateless:** No session or conversation state is maintained.
* **Logging Focus:** The primary value is comprehensive logging of the intercepted traffic.
* **Streaming Pass-Through:** Handles `text/event-stream` responses by streaming chunks, not buffering full responses.

## 3. Technology Stack

* **Language:** Rust (latest stable edition, e.g., 2021)
* **Async Runtime:** `tokio` (The standard for async Rust)
* **Web Framework:** `axum` (Chosen for ergonomics, composability with `tower`, and integration with `tokio`)
    * Alternatively: `actix-web`
* **HTTP Client:** `reqwest` (Feature-rich async HTTP client)
* **Serialization:** `serde`, `serde_json` (For parsing/logging request/response bodies as JSON where applicable)
* **Logging:** `tracing`, `tracing-subscriber` (For structured, asynchronous logging)
* **Configuration:** Standard library `std::env` or optionally `dotenvy` for `.env` file support.
* **Error Handling:** `anyhow` or `thiserror` (Optional, for cleaner error handling)

## 4. Project Setup

1.  **Initialize Cargo Project:**
    ```bash
    cargo new --bin anthropic-visibility-proxy
    cd anthropic-visibility-proxy
    ```
2.  **Add Dependencies (`Cargo.toml`):**
    ```toml
    [package]
    name = "anthropic-visibility-proxy"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    tokio = { version = "1", features = ["full"] }
    axum = { version = "0.7", features = ["http2", "json", "macros"] } # Or latest compatible
    reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false } # Use rustls
    serde = { version = "1", features = ["derive"] }
    serde_json = "1"
    tracing = "0.1"
    tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
    dotenvy = "0.15" # Optional for .env support
    uuid = { version = "1", features = ["v4"] } # For request IDs
    http = "1" # For status codes, headers etc.
    hyper = { version = "1", features = ["server", "http1", "http2"] } # Underlying Axum server
    bytes = "1" # For handling request/response bodies efficiently
    futures-util = "0.3" # For stream manipulation
    ```
3.  **Directory Structure:**
    ```
    /src/
        main.rs         # Application entrypoint, server setup, logging init
        config.rs       # Configuration loading
        proxy_handler.rs # Axum handler logic
        logger.rs       # Logging setup and helpers (optional)
        error.rs        # Custom error types (optional)
    Cargo.toml
    Cargo.lock
    .env               # Optional: For local config (add to .gitignore)
    .gitignore
    Dockerfile          # Optional
    README.md
    PLAN.md             # This file
    ```

## 5. API Definitions (Rust Structs)

Define basic Rust structs using `serde` primarily for logging identification, not for deep processing. Use `serde_json::Value` for bodies if detailed structure isn't needed for logging, or define minimal structs for key fields.

**`src/proxy_handler.rs` (or a dedicated types module):**

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value; // Use Value for flexible body logging

// Minimal representation for logging identification if needed
#[derive(Deserialize, Debug)]
struct AnthropicMessagesRequestMinimal {
    model: Option<String>,
    stream: Option<bool>,
    // Potentially add messages count or other high-level fields for logging context
    // messages: Option<Vec<Value>>, // Too verbose?
}

// No specific response struct needed if just logging raw body,
// unless specific fields (like stop_reason) are desired for structured logs.
```

## 6. Core Components Implementation

**`src/config.rs`:**

```rust
use std::env;
use std::sync::OnceLock;
use tracing::info; // Use tracing for logging

#[derive(Debug, Clone)]
pub struct Config {
    pub port: String,
    pub anthropic_api_key: String, // The key to use when forwarding *to* Anthropic
    pub anthropic_target_url: String,
    pub log_level: String,
    pub log_format: String, // "json" or "pretty"
}

// Use OnceLock for thread-safe global config initialization
pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn load_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        dotenvy::dotenv().ok(); // Load .env if present, ignore errors
        info!("Loading configuration from environment...");

        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY must be set for forwarding");
        let anthropic_target_url = env::var("ANTHROPIC_TARGET_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());


        let loaded_config = Config {
            port,
            anthropic_api_key,
            anthropic_target_url,
            log_level,
            log_format,
        };
        // Don't log the key itself
        info!(port = %loaded_config.port, target_url = %loaded_config.anthropic_target_url, log_level = %loaded_config.log_level, log_format = %loaded_config.log_format, "Configuration loaded");
        loaded_config
    })
}
```

**`src/logger.rs`:**

```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter, registry};
use crate::config::Config;

pub fn init_tracing(config: &Config) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .unwrap_or_else(|e| {
            eprintln!("Failed to parse log level filter: {}, using default 'info'", e);
            EnvFilter::new("info") // Fallback to info
        });

    let subscriber = registry().with(filter);

    match config.log_format.as_str() {
        "json" => {
            let json_layer = fmt::layer().json();
            subscriber.with(json_layer).init();
        }
        _ => { // Default to pretty
            let pretty_layer = fmt::layer().pretty();
            subscriber.with(pretty_layer).init();
        }
    }
}
```

**`src/proxy_handler.rs`:**

```rust
use axum::{
    body::{Body, Incoming},
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use bytes::Bytes;
use futures_util::stream::StreamExt; // Use StreamExt for mapping the body stream
use http::header;
use reqwest::Client;
use std::net::SocketAddr;
use std::time::Instant;
use tracing::{debug, error, info, info_span, instrument, warn, Instrument, field};
use uuid::Uuid;
use serde_json::Value;


use crate::config::Config;


// Function to create the Axum router
pub fn create_router(client: Client, config: &'static Config) -> Router {
    Router::new().route(
        "/*path", // Capture all paths
        any(move |req| proxy_handler(req, client.clone(), config)),
    )
}

#[instrument(skip_all, name="proxy_request", fields(req_id = field::Empty, http.method = field::Empty, url.path = field::Empty, url.query = field::Empty, http.status_code = field::Empty, duration_ms = field::Empty))]
async fn proxy_handler(
    req: Request<Incoming>,
    client: Client,
    config: &'static Config,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let req_id = Uuid::new_v4();
    let span = info_span::Span::current(); // Get current span
    span.record("req_id", &req_id.to_string()); // Record req_id in the span

    let original_uri = req.uri().clone();
    let path_and_query = original_uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let method = req.method().clone();

    // Record basic info in span
    span.record("http.method", &method.to_string());
    span.record("url.path", original_uri.path());
    if let Some(query) = original_uri.query() {
        span.record("url.query", query);
    }

    // Construct target URL
    let target_url_str = format!("{}{}", config.anthropic_target_url, path_and_query);
    let target_url = match target_url_str.parse::<Uri>() {
        Ok(url) => url,
        Err(e) => {
            error!(error = %e, target_url = %target_url_str, "Failed to parse target URL");
            span.record("http.status_code", StatusCode::INTERNAL_SERVER_ERROR.as_u16());
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    info!(target = %target_url, "Incoming request");

    // Read headers and body
    let original_headers = req.headers().clone();
    let body_bytes_result = axum::body::to_bytes(req.into_body(), usize::MAX).await;

    let body_bytes = match body_bytes_result {
         Ok(bytes) => bytes,
         Err(e) => {
             error!(error = %e, "Failed to read request body");
             span.record("http.status_code", StatusCode::BAD_REQUEST.as_u16());
             return Err(StatusCode::BAD_REQUEST);
         }
    };

    // Log Request Details (structured)
    log_request_details(&method, &original_uri, &original_headers, &body_bytes);

    // --- Forward Request ---
    let mut forward_req_builder = client
        .request(method.clone(), target_url.to_string());
        // Don't clone body yet, use it directly unless streaming pass-through fails

    // Copy headers, overwriting Host and setting Anthropic API Key
    let mut forward_headers = HeaderMap::new();
    for (name, value) in original_headers.iter() {
        // Filter out hop-by-hop headers and host
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

    // Set the target host header
    if let Some(host) = target_url.host() {
        if let Ok(host_value) = HeaderValue::from_str(host) {
             forward_headers.insert(header::HOST, host_value);
        }
    }
    // Set the Anthropic API Key
    if let Ok(api_key_value) = HeaderValue::from_str(&config.anthropic_api_key) {
         forward_headers.insert(HeaderName::from_static("x-api-key"), api_key_value);
         // Remove Authorization header if it exists, as x-api-key is preferred by Anthropic
         forward_headers.remove(header::AUTHORIZATION);
    } else {
         error!("Failed to create header value for Anthropic API key");
         span.record("http.status_code", StatusCode::INTERNAL_SERVER_ERROR.as_u16());
         return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Add headers to the builder
    forward_req_builder = forward_req_builder.headers(forward_headers);

    // Add body
    forward_req_builder = forward_req_builder.body(body_bytes); // Use original bytes

    info!("Forwarding request to Anthropic API...");
    let forward_resp_result = forward_req_builder.send().await;

    match forward_resp_result {
        Ok(forward_resp) => {
            let resp_status = forward_resp.status();
            let resp_headers = forward_resp.headers().clone();
            info!(status = %resp_status, "Received response from Anthropic API");
            span.record("http.status_code", resp_status.as_u16()); // Record final status

            // Check if response is streaming
            let is_streaming = resp_headers
                .get(header::CONTENT_TYPE)
                .map_or(false, |v| v.to_str().unwrap_or("").contains("text/event-stream"));

             if is_streaming {
                 info!("Streaming response detected. Passing through...");
                 log_response_headers(resp_status, &resp_headers);

                // Map the reqwest stream to an Axum stream
                let stream = forward_resp.bytes_stream().map(|result| {
                    result.map_err(|e| {
                         // This error happens *during* streaming
                         error!(error=%e, "Error reading chunk from upstream");
                         // Cannot change HTTP status now, chunk just won't arrive
                         // Convert to a BoxError for Axum body
                         axum::BoxError::from(e)
                    })
                });
                 let axum_body = Body::from_stream(stream);

                let mut response_builder = Response::builder()
                    .status(resp_status);
                 // Copy headers from Anthropic response to our client response
                for (name, value) in resp_headers.iter() {
                    if name != header::CONNECTION && name != header::TRANSFER_ENCODING && name != header::CONTENT_LENGTH {
                         response_builder = response_builder.header(name.clone(), value.clone());
                     }
                 }
                 let response = response_builder.body(axum_body)
                    .map_err(|e| {
                        error!(error = %e, "Failed to build streaming response");
                        StatusCode::INTERNAL_SERVER_ERROR // Error occurred before sending body
                    })?;

                let duration = start.elapsed();
                span.record("duration_ms", duration.as_millis() as u64);
                info!(duration = ?duration, "Streaming response finished");
                Ok(response)

             } else {
                 // --- Handle Non-Streaming ---
                 let resp_body_bytes_result = forward_resp.bytes().await;

                 match resp_body_bytes_result {
                     Ok(resp_body_bytes) => {
                        // Log Response Details
                        log_response_details(resp_status, &resp_headers, &resp_body_bytes);

                        // Build client response
                        let mut response_builder = Response::builder().status(resp_status);
                        // Copy headers
                        for (name, value) in resp_headers.iter() {
                             if name != header::CONNECTION && name != header::TRANSFER_ENCODING && name != header::CONTENT_LENGTH {
                                 response_builder = response_builder.header(name.clone(), value.clone());
                             }
                         }
                         // Set content length for non-streaming
                         response_builder = response_builder.header(header::CONTENT_LENGTH, resp_body_bytes.len().to_string());

                         let response = response_builder.body(Body::from(resp_body_bytes))
                            .map_err(|e| {
                                error!(error = %e, "Failed to build non-streaming response");
                                StatusCode::INTERNAL_SERVER_ERROR
                            })?;

                         let duration = start.elapsed();
                         span.record("duration_ms", duration.as_millis() as u64);
                         info!(duration = ?duration, "Non-streaming request finished");
                         Ok(response)
                     },
                      Err(e) => {
                          error!(error = %e, "Failed to read Anthropic response body");
                           span.record("http.status_code", StatusCode::BAD_GATEWAY.as_u16());
                          Err(StatusCode::BAD_GATEWAY)
                      }
                 }
             }
        }
        Err(e) => {
            error!(error = %e, "Failed to forward request to Anthropic");
            span.record("http.status_code", StatusCode::BAD_GATEWAY.as_u16());
            Err(StatusCode::BAD_GATEWAY) // Indicate upstream failure
        }
    }
}

const MAX_LOG_BODY_LEN: usize = 10 * 1024; // Log max 10KB of body

// Helper to log request details (structured)
fn log_request_details(method: &Method, uri: &Uri, headers: &HeaderMap, body: &Bytes) {
    let span = info_span!("request_details");
    let _enter = span.enter();

    // Log basic info
    info!(http.method = %method, url.full = %uri);

    // Log headers, masking sensitive ones
    let mut headers_log: std::collections::HashMap<String, String> = Default::default();
    for (name, value) in headers.iter() {
        let name_str = name.to_string();
        let value_str = if name == header::AUTHORIZATION || name == "x-api-key" {
            "[REDACTED]".to_string()
        } else {
            String::from_utf8_lossy(value.as_bytes()).to_string()
        };
        headers_log.insert(name_str, value_str);
    }
    debug!(http.request.headers = ?headers_log); // Log headers at debug level

    // Log body (truncated)
    let body_len = body.len();
    let log_body = body_len <= MAX_LOG_BODY_LEN;
    if log_body && body_len > 0 {
        match serde_json::from_slice::<Value>(body) {
            Ok(json_val) => debug!(http.request.body.content = %serde_json::to_string_pretty(&json_val).unwrap_or_default(), http.request.body.size = body_len),
            Err(_) => debug!(http.request.body.content = %String::from_utf8_lossy(body), http.request.body.size = body_len),
        }
    } else if body_len > 0 {
         info!(http.request.body.size = body_len, "Request body too large to log fully");
    } else {
         info!("Request body empty");
    }

}

// Helper to log non-streaming response details (structured)
fn log_response_details(status: StatusCode, headers: &HeaderMap, body: &Bytes) {
    let span = info_span!("response_details");
    let _enter = span.enter();

     info!(http.status_code = %status.as_u16());

     let mut headers_log: std::collections::HashMap<String, String> = Default::default();
     for (name, value) in headers.iter() {
         headers_log.insert(
             name.to_string(),
             String::from_utf8_lossy(value.as_bytes()).to_string(),
         );
     }
     debug!(http.response.headers = ?headers_log);

     let body_len = body.len();
     let log_body = body_len <= MAX_LOG_BODY_LEN;
     if log_body && body_len > 0 {
         match serde_json::from_slice::<Value>(body) {
             Ok(json_val) => debug!(http.response.body.content = %serde_json::to_string_pretty(&json_val).unwrap_or_default(), http.response.body.size = body_len),
             Err(_) => debug!(http.response.body.content = %String::from_utf8_lossy(body), http.response.body.size = body_len),
         }
     } else if body_len > 0 {
          info!(http.response.body.size = body_len, "Response body too large to log fully");
     } else {
          info!("Response body empty");
     }
}

// Helper to log streaming response headers (body handled separately)
fn log_response_headers(status: StatusCode, headers: &HeaderMap) {
    let span = info_span!("response_details");
    let _enter = span.enter();

     info!(http.status_code = %status.as_u16());

     let mut headers_log: std::collections::HashMap<String, String> = Default::default();
     for (name, value) in headers.iter() {
         headers_log.insert(
             name.to_string(),
             String::from_utf8_lossy(value.as_bytes()).to_string(),
         );
     }
     debug!(http.response.headers = ?headers_log);
     info!("Streaming response body initiated");
}
```

**`src/main.rs`:**

```rust
mod config;
mod proxy_handler;
mod logger;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};
use reqwest::Client;
use std::time::Duration;


use config::load_config;
use proxy_handler::create_router;
use logger::init_tracing;


#[tokio::main]
async fn main() {
    // Load config must happen before logger init if logger uses config
    let config = load_config();
    init_tracing(config); // Initialize logging

    info!("Starting Anthropic Visibility Proxy...");

    // Build the reqwest client
    // Configure reasonable timeouts
    let http_client = Client::builder()
        .use_rustls_tls()
        .timeout(Duration::from_secs(300)) // Overall request timeout (generous for LLMs)
        .connect_timeout(Duration::from_secs(10))
        // .pool_max_idle_per_host(10) // Example connection pooling
        // .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .expect("Failed to build reqwest client");

    // Build the Axum application router
    let app = create_router(http_client, config);

    // Run the server
    let addr_str = format!("0.0.0.0:{}", config.port);
    let addr: SocketAddr = match addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            error!(error = %e, addr = %addr_str, "Invalid listen address/port");
            std::process::exit(1);
        }
    };


    info!("Listening on {}", addr);
    let listener = match TcpListener::bind(addr).await {
         Ok(listener) => listener,
         Err(e) => {
              error!(error = %e, addr = %addr, "Failed to bind port");
              std::process::exit(1);
         }
    };


    if let Err(e) = axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await {
             error!(error = %e, "Server error");
             std::process::exit(1);
    }

}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>(); // No-op on non-unix

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown...");
    // Add any cleanup logic here if needed
}
```

## 7. Logging Implementation Details

* **Initialization:** Use `logger::init_tracing()` called early in `main.rs`. Configuration drives format (`json`/`pretty`) and level (`RUST_LOG` env var or config default).
* **Spans:** `proxy_handler` is instrumented with `tracing::instrument`. A unique `req_id` (UUID v4) is generated and attached to the span using `span.record()`. Key request/response attributes like `http.method`, `url.path`, `http.status_code`, `duration_ms` are also recorded on the span.
* **Events:** Use `info!`, `warn!`, `error!`, `debug!` macros. Standard fields like timestamp and log level are handled by the subscriber.
* **Structured Data:** Log complex data like headers and bodies within debug/trace events using `field = ?value` (uses `Debug` trait) or `field = %value` (uses `Display` trait). Request/response bodies are logged as strings (JSON if possible, else lossy UTF-8), truncated via `MAX_LOG_BODY_LEN`.
* **Header Masking:** `Authorization` and `x-api-key` headers are explicitly redacted before logging.
* **Streaming:** Only response headers are logged explicitly for streaming responses. The pass-through nature means chunk logging isn't done by default (too verbose), but errors *during* chunk reading are logged.

## 8. Build & Deployment

1.  **Build:**
    ```bash
    # Development build
    cargo build

    # Release build (optimized)
    cargo build --release
    ```
2.  **Run:**
    ```bash
    # Set environment variables
    export RUST_LOG="info,anthropic_visibility_proxy=debug" # Example: info globally, debug for our app
    export LOG_FORMAT="pretty" # or "json"
    export PORT="8080"
    export ANTHROPIC_API_KEY="sk-ant-..." # Your actual key
    # ANTHROPIC_TARGET_URL defaults to https://api.anthropic.com

    ./target/release/anthropic-visibility-proxy
    ```
3.  **Dockerfile (Minimal Rustls):**
    ```dockerfile
    # ---- Builder Stage ----
    FROM rust:1.7-slim AS builder
    # Using slim-bullseye as base

    WORKDIR /app

    # Install build dependencies if any (e.g., for specific TLS backends if not rustls)
    # RUN apt-get update && apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev

    # Cache dependencies
    COPY Cargo.toml Cargo.lock ./
    # Create dummy src/main.rs for dependency caching step
    RUN mkdir src && echo "fn main(){}" > src/main.rs
    # Build dependencies only first to leverage Docker cache
    RUN cargo build --release --target-dir /app/target_dep

    # Build application
    COPY src ./src
    # Link dependencies build to application build cache
    RUN rm -f /app/target_dep/release/deps/anthropic_visibility_proxy* # Remove dummy binary hash
    RUN cargo build --release --target-dir /app/target_app

    # ---- Final Stage ----
    # Use a minimal runtime image. Debian slim is often a good balance.
    FROM debian:12-slim AS final

    # Install runtime dependencies: ca-certificates for TLS verification (reqwest with rustls needs this)
    RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

    WORKDIR /app

    # Copy the compiled binary from the builder stage
    COPY --from=builder /app/target_app/release/anthropic-visibility-proxy /app/anthropic-visibility-proxy

    # Optional: Create a non-root user for security
    # RUN groupadd --system appuser && useradd --system --group appuser appuser
    # USER appuser:appuser

    # Expose the port (ensure this matches the PORT env var)
    EXPOSE 8080

    # Set Env Vars via `docker run -e` or orchestration layer
    # ENV RUST_LOG=info
    # ENV PORT=8080
    # ENV ANTHROPIC_API_KEY=...
    # ENV ANTHROPIC_TARGET_URL=https://api.anthropic.com

    # Set the entrypoint
    ENTRYPOINT ["/app/anthropic-visibility-proxy"]
    ```
4.  **Deployment:** Deploy the container (or binary). Ensure environment variables are correctly set. Configure networking. Point Claude Code (`ANTHROPIC_BASE_URL`) to this proxy.

====PLAN.MD END====
```
tool_code
print(create_note(title='PLAN.md', content='''# PLAN.MD: Anthropic Visibility Proxy (Rust Implementation)

## 1. Overview

**Goal:** Implement an extremely simple, highly observable HTTP reverse proxy in Rust. Its sole purpose is to intercept traffic between an Anthropic client (like Claude Code) and the official Anthropic Messages API (`https://api.anthropic.com`), logging *all* request and response details (headers, bodies) for visibility and debugging purposes.

**Core Functionality:**

* Listen for HTTP POST requests, primarily targeting `/v1/messages`.
* Receive the incoming request from the client (Claude Code).
* **Log** the full incoming request (method, path, headers, body).
* Forward the *exact* request (headers, body) to the configured Anthropic API endpoint.
* Receive the response from the Anthropic API.
* **Log** the full response (status code, headers, body).
* Forward the *exact* response (status, headers, body) back to the original client.
* Handle both standard JSON responses and streaming (Server-Sent Events) responses correctly by passing through the data chunks.
* Handle network errors during forwarding gracefully.

**Non-Goals (for this simplified version):**

* No translation between API formats (Anthropic <-> Gemini or anything else).
* No interaction with any API other than the official Anthropic API.
* No modification of request/response bodies (except potentially adding tracing headers if needed).
* No complex routing or state management.

## 2. Architecture

* **Simple Reverse Proxy:** A single Rust application acting as a pass-through layer.
* **Stateless:** No session or conversation state is maintained.
* **Logging Focus:** The primary value is comprehensive logging of the intercepted traffic.
* **Streaming Pass-Through:** Handles `text/event-stream` responses by streaming chunks, not buffering full responses.

## 3. Technology Stack

* **Language:** Rust (latest stable edition, e.g., 2021)
* **Async Runtime:** `tokio` (The standard for async Rust)
* **Web Framework:** `axum` (Chosen for ergonomics, composability with `tower`, and integration with `tokio`)
    * Alternatively: `actix-web`
* **HTTP Client:** `reqwest` (Feature-rich async HTTP client)
* **Serialization:** `serde`, `serde_json` (For parsing/logging request/response bodies as JSON where applicable)
* **Logging:** `tracing`, `tracing-subscriber` (For structured, asynchronous logging)
* **Configuration:** Standard library `std::env` or optionally `dotenvy` for `.env` file support.
* **Error Handling:** `anyhow` or `thiserror` (Optional, for cleaner error handling)

## 4. Project Setup

1.  **Initialize Cargo Project:**
    ```bash
    cargo new --bin anthropic-visibility-proxy
    cd anthropic-visibility-proxy
    ```
2.  **Add Dependencies (`Cargo.toml`):**
    ```toml
    [package]
    name = "anthropic-visibility-proxy"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    tokio = { version = "1", features = ["full"] }
    axum = { version = "0.7", features = ["http2", "json", "macros"] } # Or latest compatible
    reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false } # Use rustls
    serde = { version = "1", features = ["derive"] }
    serde_json = "1"
    tracing = "0.1"
    tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
    dotenvy = "0.15" # Optional for .env support
    uuid = { version = "1", features = ["v4"] } # For request IDs
    http = "1" # For status codes, headers etc.
    hyper = { version = "1", features = ["server", "http1", "http2"] } # Underlying Axum server
    bytes = "1" # For handling request/response bodies efficiently
    futures-util = "0.3" # For stream manipulation
    ```
3.  **Directory Structure:**
    ```
    /src/
        main.rs         # Application entrypoint, server setup, logging init
        config.rs       # Configuration loading
        proxy_handler.rs # Axum handler logic
        logger.rs       # Logging setup and helpers (optional)
        error.rs        # Custom error types (optional)
    Cargo.toml
    Cargo.lock
    .env               # Optional: For local config (add to .gitignore)
    .gitignore
    Dockerfile          # Optional
    README.md
    PLAN.md             # This file
    ```

## 5. API Definitions (Rust Structs)

Define basic Rust structs using `serde` primarily for logging identification, not for deep processing. Use `serde_json::Value` for bodies if detailed structure isn't needed for logging, or define minimal structs for key fields.

**`src/proxy_handler.rs` (or a dedicated types module):**

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value; // Use Value for flexible body logging

// Minimal representation for logging identification if needed
#[derive(Deserialize, Debug)]
struct AnthropicMessagesRequestMinimal {
    model: Option<String>,
    stream: Option<bool>,
    // Potentially add messages count or other high-level fields for logging context
    // messages: Option<Vec<Value>>, // Too verbose?
}

// No specific response struct needed if just logging raw body,
// unless specific fields (like stop_reason) are desired for structured logs.
```

## 6. Core Components Implementation

**`src/config.rs`:**

```rust
use std::env;
use std::sync::OnceLock;
use tracing::info; // Use tracing for logging

#[derive(Debug, Clone)]
pub struct Config {
    pub port: String,
    pub anthropic_api_key: String, // The key to use when forwarding *to* Anthropic
    pub anthropic_target_url: String,
    pub log_level: String,
    pub log_format: String, // "json" or "pretty"
}

// Use OnceLock for thread-safe global config initialization
pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn load_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        dotenvy::dotenv().ok(); // Load .env if present, ignore errors
        info!("Loading configuration from environment...");

        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY must be set for forwarding");
        let anthropic_target_url = env::var("ANTHROPIC_TARGET_URL")
            .unwrap_or_else(|_| "[https://api.anthropic.com](https://api.anthropic.com)".to_string());
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());


        let loaded_config = Config {
            port,
            anthropic_api_key,
            anthropic_target_url,
            log_level,
            log_format,
        };
        // Don't log the key itself
        info!(port = %loaded_config.port, target_url = %loaded_config.anthropic_target_url, log_level = %loaded_config.log_level, log_format = %loaded_config.log_format, "Configuration loaded");
        loaded_config
    })
}
```

**`src/logger.rs`:**

```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter, registry};
use crate::config::Config;

pub fn init_tracing(config: &Config) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .unwrap_or_else(|e| {
            eprintln!("Failed to parse log level filter: {}, using default 'info'", e);
            EnvFilter::new("info") // Fallback to info
        });

    let subscriber = registry().with(filter);

    match config.log_format.as_str() {
        "json" => {
            let json_layer = fmt::layer().json();
            subscriber.with(json_layer).init();
        }
        _ => { // Default to pretty
            let pretty_layer = fmt::layer().pretty();
            subscriber.with(pretty_layer).init();
        }
    }
}
```

**`src/proxy_handler.rs`:**

```rust
use axum::{
    body::{Body, Incoming},
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use bytes::Bytes;
use futures_util::stream::StreamExt; // Use StreamExt for mapping the body stream
use http::header;
use reqwest::Client;
use std::net::SocketAddr;
use std::time::Instant;
use tracing::{debug, error, info, info_span, instrument, warn, Instrument, field};
use uuid::Uuid;
use serde_json::Value;


use crate::config::Config;


// Function to create the Axum router
pub fn create_router(client: Client, config: &'static Config) -> Router {
    Router::new().route(
        "/*path", // Capture all paths
        any(move |req| proxy_handler(req, client.clone(), config)),
    )
}

#[instrument(skip_all, name="proxy_request", fields(req_id = field::Empty, http.method = field::Empty, url.path = field::Empty, url.query = field::Empty, http.status_code = field::Empty, duration_ms = field::Empty))]
async fn proxy_handler(
    req: Request<Incoming>,
    client: Client,
    config: &'static Config,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let req_id = Uuid::new_v4();
    let span = info_span::Span::current(); // Get current span
    span.record("req_id", &req_id.to_string()); // Record req_id in the span

    let original_uri = req.uri().clone();
    let path_and_query = original_uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let method = req.method().clone();

    // Record basic info in span
    span.record("http.method", &method.to_string());
    span.record("url.path", original_uri.path());
    if let Some(query) = original_uri.query() {
        span.record("url.query", query);
    }

    // Construct target URL
    let target_url_str = format!("{}{}", config.anthropic_target_url, path_and_query);
    let target_url = match target_url_str.parse::<Uri>() {
        Ok(url) => url,
        Err(e) => {
            error!(error = %e, target_url = %target_url_str, "Failed to parse target URL");
            span.record("http.status_code", StatusCode::INTERNAL_SERVER_ERROR.as_u16());
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    info!(target = %target_url, "Incoming request");

    // Read headers and body
    let original_headers = req.headers().clone();
    let body_bytes_result = axum::body::to_bytes(req.into_body(), usize::MAX).await;

    let body_bytes = match body_bytes_result {
         Ok(bytes) => bytes,
         Err(e) => {
             error!(error = %e, "Failed to read request body");
             span.record("http.status_code", StatusCode::BAD_REQUEST.as_u16());
             return Err(StatusCode::BAD_REQUEST);
         }
    };

    // Log Request Details (structured)
    log_request_details(&method, &original_uri, &original_headers, &body_bytes);

    // --- Forward Request ---
    let mut forward_req_builder = client
        .request(method.clone(), target_url.to_string());
        // Don't clone body yet, use it directly unless streaming pass-through fails

    // Copy headers, overwriting Host and setting Anthropic API Key
    let mut forward_headers = HeaderMap::new();
    for (name, value) in original_headers.iter() {
        // Filter out hop-by-hop headers and host
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

    // Set the target host header
    if let Some(host) = target_url.host() {
        if let Ok(host_value) = HeaderValue::from_str(host) {
             forward_headers.insert(header::HOST, host_value);
        }
    }
    // Set the Anthropic API Key
    if let Ok(api_key_value) = HeaderValue::from_str(&config.anthropic_api_key) {
         forward_headers.insert(HeaderName::from_static("x-api-key"), api_key_value);
         // Remove Authorization header if it exists, as x-api-key is preferred by Anthropic
         forward_headers.remove(header::AUTHORIZATION);
    } else {
         error!("Failed to create header value for Anthropic API key");
         span.record("http.status_code", StatusCode::INTERNAL_SERVER_ERROR.as_u16());
         return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Add headers to the builder
    forward_req_builder = forward_req_builder.headers(forward_headers);

    // Add body
    forward_req_builder = forward_req_builder.body(body_bytes); // Use original bytes

    info!("Forwarding request to Anthropic API...");
    let forward_resp_result = forward_req_builder.send().await;

    match forward_resp_result {
        Ok(forward_resp) => {
            let resp_status = forward_resp.status();
            let resp_headers = forward_resp.headers().clone();
            info!(status = %resp_status, "Received response from Anthropic API");
            span.record("http.status_code", resp_status.as_u16()); // Record final status

            // Check if response is streaming
            let is_streaming = resp_headers
                .get(header::CONTENT_TYPE)
                .map_or(false, |v| v.to_str().unwrap_or("").contains("text/event-stream"));

             if is_streaming {
                 info!("Streaming response detected. Passing through...");
                 log_response_headers(resp_status, &resp_headers);

                // Map the reqwest stream to an Axum stream
                let stream = forward_resp.bytes_stream().map(|result| {
                    result.map_err(|e| {
                         // This error happens *during* streaming
                         error!(error=%e, "Error reading chunk from upstream");
                         // Cannot change HTTP status now, chunk just won't arrive
                         // Convert to a BoxError for Axum body
                         axum::BoxError::from(e)
                    })
                });
                 let axum_body = Body::from_stream(stream);

                let mut response_builder = Response::builder()
                    .status(resp_status);
                 // Copy headers from Anthropic response to our client response
                for (name, value) in resp_headers.iter() {
                    if name != header::CONNECTION && name != header::TRANSFER_ENCODING && name != header::CONTENT_LENGTH {
                         response_builder = response_builder.header(name.clone(), value.clone());
                     }
                 }
                 let response = response_builder.body(axum_body)
                    .map_err(|e| {
                        error!(error = %e, "Failed to build streaming response");
                        StatusCode::INTERNAL_SERVER_ERROR // Error occurred before sending body
                    })?;

                let duration = start.elapsed();
                span.record("duration_ms", duration.as_millis() as u64);
                info!(duration = ?duration, "Streaming response finished");
                Ok(response)

             } else {
                 // --- Handle Non-Streaming ---
                 let resp_body_bytes_result = forward_resp.bytes().await;

                 match resp_body_bytes_result {
                     Ok(resp_body_bytes) => {
                        // Log Response Details
                        log_response_details(resp_status, &resp_headers, &resp_body_bytes);

                        // Build client response
                        let mut response_builder = Response::builder().status(resp_status);
                        // Copy headers
                        for (name, value) in resp_headers.iter() {
                             if name != header::CONNECTION && name != header::TRANSFER_ENCODING && name != header::CONTENT_LENGTH {
                                 response_builder = response_builder.header(name.clone(), value.clone());
                             }
                         }
                         // Set content length for non-streaming
                         response_builder = response_builder.header(header::CONTENT_LENGTH, resp_body_bytes.len().to_string());

                         let response = response_builder.body(Body::from(resp_body_bytes))
                            .map_err(|e| {
                                error!(error = %e, "Failed to build non-streaming response");
                                StatusCode::INTERNAL_SERVER_ERROR
                            })?;

                         let duration = start.elapsed();
                         span.record("duration_ms", duration.as_millis() as u64);
                         info!(duration = ?duration, "Non-streaming request finished");
                         Ok(response)
                     },
                      Err(e) => {
                          error!(error = %e, "Failed to read Anthropic response body");
                           span.record("http.status_code", StatusCode::BAD_GATEWAY.as_u16());
                          Err(StatusCode::BAD_GATEWAY)
                      }
                 }
             }
        }
        Err(e) => {
            error!(error = %e, "Failed to forward request to Anthropic");
            span.record("http.status_code", StatusCode::BAD_GATEWAY.as_u16());
            Err(StatusCode::BAD_GATEWAY) // Indicate upstream failure
        }
    }
}

const MAX_LOG_BODY_LEN: usize = 10 * 1024; // Log max 10KB of body

// Helper to log request details (structured)
fn log_request_details(method: &Method, uri: &Uri, headers: &HeaderMap, body: &Bytes) {
    let span = info_span!("request_details");
    let _enter = span.enter();

    // Log basic info
    info!(http.method = %method, url.full = %uri);

    // Log headers, masking sensitive ones
    let mut headers_log: std::collections::HashMap<String, String> = Default::default();
    for (name, value) in headers.iter() {
        let name_str = name.to_string();
        let value_str = if name == header::AUTHORIZATION || name == "x-api-key" {
            "[REDACTED]".to_string()
        } else {
            String::from_utf8_lossy(value.as_bytes()).to_string()
        };
        headers_log.insert(name_str, value_str);
    }
    debug!(http.request.headers = ?headers_log); // Log headers at debug level

    // Log body (truncated)
    let body_len = body.len();
    let log_body = body_len <= MAX_LOG_BODY_LEN;
    if log_body && body_len > 0 {
        match serde_json::from_slice::<Value>(body) {
            Ok(json_val) => debug!(http.request.body.content = %serde_json::to_string_pretty(&json_val).unwrap_or_default(), http.request.body.size = body_len),
            Err(_) => debug!(http.request.body.content = %String::from_utf8_lossy(body), http.request.body.size = body_len),
        }
    } else if body_len > 0 {
         info!(http.request.body.size = body_len, "Request body too large to log fully");
    } else {
         info!("Request body empty");
    }

}

// Helper to log non-streaming response details (structured)
fn log_response_details(status: StatusCode, headers: &HeaderMap, body: &Bytes) {
    let span = info_span!("response_details");
    let _enter = span.enter();

     info!(http.status_code = %status.as_u16());

     let mut headers_log: std::collections::HashMap<String, String> = Default::default();
     for (name, value) in headers.iter() {
         headers_log.insert(
             name.to_string(),
             String::from_utf8_lossy(value.as_bytes()).to_string(),
         );
     }
     debug!(http.response.headers = ?headers_log);

     let body_len = body.len();
     let log_body = body_len <= MAX_LOG_BODY_LEN;
     if log_body && body_len > 0 {
         match serde_json::from_slice::<Value>(body) {
             Ok(json_val) => debug!(http.response.body.content = %serde_json::to_string_pretty(&json_val).unwrap_or_default(), http.response.body.size = body_len),
             Err(_) => debug!(http.response.body.content = %String::from_utf8_lossy(body), http.response.body.size = body_len),
         }
     } else if body_len > 0 {
          info!(http.response.body.size = body_len, "Response body too large to log fully");
     } else {
          info!("Response body empty");
     }
}

// Helper to log streaming response headers (body handled separately)
fn log_response_headers(status: StatusCode, headers: &HeaderMap) {
    let span = info_span!("response_details");
    let _enter = span.enter();

     info!(http.status_code = %status.as_u16());

     let mut headers_log: std::collections::HashMap<String, String> = Default::default();
     for (name, value) in headers.iter() {
         headers_log.insert(
             name.to_string(),
             String::from_utf8_lossy(value.as_bytes()).to_string(),
         );
     }
     debug!(http.response.headers = ?headers_log);
     info!("Streaming response body initiated");
}
```

**`src/main.rs`:**

```rust
mod config;
mod proxy_handler;
mod logger;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};
use reqwest::Client;
use std::time::Duration;


use config::load_config;
use proxy_handler::create_router;
use logger::init_tracing;


#[tokio::main]
async fn main() {
    // Load config must happen before logger init if logger uses config
    let config = load_config();
    init_tracing(config); // Initialize logging

    info!("Starting Anthropic Visibility Proxy...");

    // Build the reqwest client
    // Configure reasonable timeouts
    let http_client = Client::builder()
        .use_rustls_tls()
        .timeout(Duration::from_secs(300)) // Overall request timeout (generous for LLMs)
        .connect_timeout(Duration::from_secs(10))
        // .pool_max_idle_per_host(10) // Example connection pooling
        // .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .expect("Failed to build reqwest client");

    // Build the Axum application router
    let app = create_router(http_client, config);

    // Run the server
    let addr_str = format!("0.0.0.0:{}", config.port);
    let addr: SocketAddr = match addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            error!(error = %e, addr = %addr_str, "Invalid listen address/port");
            std::process::exit(1);
        }
    };


    info!("Listening on {}", addr);
    let listener = match TcpListener::bind(addr).await {
         Ok(listener) => listener,
         Err(e) => {
              error!(error = %e, addr = %addr, "Failed to bind port");
              std::process::exit(1);
         }
    };


    if let Err(e) = axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await {
             error!(error = %e, "Server error");
             std::process::exit(1);
    }

}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>(); // No-op on non-unix

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown...");
    // Add any cleanup logic here if needed
}
```

## 7. Logging Implementation Details

* **Initialization:** Use `logger::init_tracing()` called early in `main.rs`. Configuration drives format (`json`/`pretty`) and level (`RUST_LOG` env var or config default).
* **Spans:** `proxy_handler` is instrumented with `tracing::instrument`. A unique `req_id` (UUID v4) is generated and attached to the span using `span.record()`. Key request/response attributes like `http.method`, `url.path`, `http.status_code`, `duration_ms` are also recorded on the span.
* **Events:** Use `info!`, `warn!`, `error!`, `debug!` macros. Standard fields like timestamp and log level are handled by the subscriber.
* **Structured Data:** Log complex data like headers and bodies within debug/trace events using `field = ?value` (uses `Debug` trait) or `field = %value` (uses `Display` trait). Request/response bodies are logged as strings (JSON if possible, else lossy UTF-8), truncated via `MAX_LOG_BODY_LEN`.
* **Header Masking:** `Authorization` and `x-api-key` headers are explicitly redacted before logging.
* **Streaming:** Only response headers are logged explicitly for streaming responses. The pass-through nature means chunk logging isn't done by default (too verbose), but errors *during* chunk reading are logged.

## 8. Build & Deployment

1.  **Build:**
    ```bash
    # Development build
    cargo build

    # Release build (optimized)
    cargo build --release
    ```
2.  **Run:**
    ```bash
    # Set environment variables
    export RUST_LOG="info,anthropic_visibility_proxy=debug" # Example: info globally, debug for our app
    export LOG_FORMAT="pretty" # or "json"
    export PORT="8080"
    export ANTHROPIC_API_KEY="sk-ant-..." # Your actual key
    # ANTHROPIC_TARGET_URL defaults to [https://api.anthropic.com](https://api.anthropic.com)

    ./target/release/anthropic-visibility-proxy
    ```
3.  **Dockerfile (Minimal Rustls):**
    ```dockerfile
    # ---- Builder Stage ----
    FROM rust:1.7-slim AS builder
    # Using slim-bullseye as base

    WORKDIR /app

    # Install build dependencies if any (e.g., for specific TLS backends if not rustls)
    # RUN apt-get update && apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev

    # Cache dependencies
    COPY Cargo.toml Cargo.lock ./
    # Create dummy src/main.rs for dependency caching step
    RUN mkdir src && echo "fn main(){}" > src/main.rs
    # Build dependencies only first to leverage Docker cache
    RUN cargo build --release --target-dir /app/target_dep

    # Build application
    COPY src ./src
    # Link dependencies build to application build cache
    RUN rm -f /app/target_dep/release/deps/anthropic_visibility_proxy* # Remove dummy binary hash
    RUN cargo build --release --target-dir /app/target_app

    # ---- Final Stage ----
    # Use a minimal runtime image. Debian slim is often a good balance.
    FROM debian:12-slim AS final

    # Install runtime dependencies: ca-certificates for TLS verification (reqwest with rustls needs this)
    RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

    WORKDIR /app

    # Copy the compiled binary from the builder stage
    COPY --from=builder /app/target_app/release/anthropic-visibility-proxy /app/anthropic-visibility-proxy

    # Optional: Create a non-root user for security
    # RUN groupadd --system appuser && useradd --system --group appuser appuser
    # USER appuser:appuser

    # Expose the port (ensure this matches the PORT env var)
    EXPOSE 8080

    # Set Env Vars via `docker run -e` or orchestration layer
    # ENV RUST_LOG=info
    # ENV PORT=8080
    # ENV ANTHROPIC_API_KEY=...
    # ENV ANTHROPIC_TARGET_URL=[https://api.anthropic.com](https://api.anthropic.com)

    # Set the entrypoint
    ENTRYPOINT ["/app/anthropic-visibility-proxy"]
    ```
4.  **Deployment:** Deploy the container (or binary). Ensure environment variables are correctly set. Configure networking. Point Claude Code (`ANTHROPIC_BASE_URL`) to this proxy.
