# PLAN-2: Core Logic - OpenAI Adapter Module & Unit Tests

## 1. Overview

This plan details the implementation of the core OpenAI adapter logic within the `src/openai_adapter.rs` module. This includes defining internal structures, handling request/response mapping (for both streaming and non-streaming modes), interacting with the OpenAI API via the `async-openai` crate, implementing specific error handling, adding relevant logging, and validating the logic with comprehensive unit tests.

## 2. Context

With the foundational configuration and dependencies established (PLAN-1), this phase focuses on creating a self-contained, testable component responsible for all direct interaction with the OpenAI API. This modular approach isolates OpenAI-specific concerns from the main proxy handler.

## 3. Approach

1.  **Module Implementation:** Define necessary structs (`OpenAIConfig`, `OpenAIAdapterError`) within `openai_adapter.rs`. Implement the main handler function (`handle_openai_request`).
2.  **Mapping:** Implement logic to map incoming proxy requests to `async-openai` request types and map `async-openai` responses (or streams) back to `axum::Response`.
3.  **API Interaction:** Use the `async-openai` client to make calls to the `/v1/chat/completions` endpoint, handling both standard and streaming modes based on the request.
4.  **Error Handling:** Define and use a specific `OpenAIAdapterError` enum (using `thiserror`) to wrap errors from API calls, mapping, serialization, etc.
5.  **Logging:** Add structured logging for visibility into the adapter's operations.
6.  **Unit Testing:** Write thorough unit tests covering all mapping, error handling, and core logic within the adapter module in isolation.

## 4. Detailed Steps

4.1.  **Define Adapter Structures (`src/openai_adapter.rs`):**
    *   Define `struct OpenAIConfig` to hold the necessary configuration subset (e.g., API Key, Base URL) passed from the main `Config`.
    *   Define `enum OpenAIAdapterError` using `thiserror::Error` to represent potential failures (e.g., `ConfigError`, `RequestMappingError`, `ApiError(#[from] async_openai::error::OpenAIError)`, `ResponseMappingError`, `StreamingError`).

4.2.  **Implement Adapter Function:**
    *   Define the main entry point: `async fn handle_openai_request(req: Request<Body>, config: Arc<OpenAIConfig>) -> Result<Response, OpenAIAdapterError>`.
    *   Inside the function, initialize the `async_openai::Client` using the provided `OpenAIConfig` (including base URL if specified).

4.3.  **Request Mapping:**
    *   Deserialize the incoming request body (`req.into_body()`). Handle deserialization errors, mapping them to `OpenAIAdapterError::RequestMappingError`.
    *   Map the deserialized request data to `async_openai::types::CreateChatCompletionRequest`. Handle potential mapping errors.

4.4.  **API Call Logic:**
    *   Based on the `stream` parameter in the mapped request:
        *   **Non-streaming:** Call `client.chat().create(request).await`.
        *   **Streaming:** Call `client.chat().create_stream(request).await`.
    *   Handle potential `async_openai::error::OpenAIError` results, allowing them to be converted via `?` or explicit mapping into `OpenAIAdapterError::ApiError`.

4.5.  **Response Mapping:**
    *   **Non-Streaming:** Map the `async_openai::types::CreateChatCompletionResponse` to the proxy's expected response format. Serialize the body and construct an `axum::response::Response` with appropriate status (200 OK) and headers (`Content-Type: application/json`). Handle serialization errors (`OpenAIAdapterError::ResponseMappingError`).
    *   **Streaming:** Map the `Stream<Result<CreateChatCompletionStreamResponse, OpenAIError>>` into an Axum-compatible stream using `Body::wrap_stream`. Each `CreateChatCompletionStreamResponse` needs mapping/formatting (e.g., into JSON lines or Server-Sent Events). Construct an `axum::response::Response` with the stream body, 200 OK status, and correct headers (`Content-Type: text/event-stream` or similar). Handle potential streaming or mapping errors (`OpenAIAdapterError::StreamingError` or `ResponseMappingError`).

4.6.  **Error Handling Implementation:**
    *   Ensure all potential failure points within the adapter (config issues, deserialization, mapping, API calls, serialization, streaming) correctly return an appropriate `OpenAIAdapterError` variant.

4.7.  **Logging:**
    *   Add `tracing::debug!` calls for detailed request/response mapping steps and API interactions.
    *   Add `tracing::info!` calls for significant events like initiating an adapter request.
    *   Log all errors returned by the adapter at `tracing::error!` level, including wrapped error details.
    *   Respect global logging configuration (e.g., `log_bodies`, `log_max_body_size`) if passed via `OpenAIConfig`.

4.8.  **Unit Tests:**
    *   Create a test module within `openai_adapter.rs` or a sub-directory.
    *   Write unit tests covering:
        *   Request mapping logic (various valid/invalid inputs).
        *   Non-streaming response mapping.
        *   Streaming response mapping/chunk processing logic.
        *   Error wrapping (e.g., `async_openai` errors mapped to `ApiError`).
        *   Mapping of different error conditions to specific `OpenAIAdapterError` variants.

4.9.  **Code Documentation:**
    *   Add doc comments (`///`) to public functions, structs (`OpenAIConfig`), and the error enum (`OpenAIAdapterError`) within `src/openai_adapter.rs`.

4.10. **Review & Refactor:**
    *   Run `cargo fmt`, `cargo clippy`, `cargo test`.

## 5. Deliverables

*   Completed `src/openai_adapter.rs` containing:
    *   `OpenAIConfig` struct.
    *   `OpenAIAdapterError` enum.
    *   `handle_openai_request` function implementation.
    *   Request/response mapping logic (streaming/non-streaming).
    *   API call logic using `async-openai`.
    *   Structured logging.
*   Comprehensive unit tests for the `openai_adapter` module.
*   Doc comments within `src/openai_adapter.rs`.

## 6. Dependency Notes

*   **Depends On:** **PLAN-1** (requires `Config` structure additions, `async-openai` dependency, and the `openai_adapter.rs` file placeholder).
*   **Independent Implementation:** Can be implemented and unit-tested in isolation after PLAN-1 is merged. Does not require changes to `proxy_handler.rs`.
*   **Enables:** **PLAN-3** (provides the `handle_openai_request` function and `OpenAIAdapterError` type needed for integration).

## 7. OpenAI API Reference (Relevant for this Plan)

*   **Endpoint:** `/v1/chat/completions`
*   **Request Body:** `async_openai::types::CreateChatCompletionRequest` (fields: `model`, `messages`, `stream`, `temperature`, `max_tokens`, etc.)
*   **Response Body (Non-streaming):** `async_openai::types::CreateChatCompletionResponse`
*   **Response Body (Streaming):** `Stream<Item = Result<CreateChatCompletionStreamResponse, OpenAIError>>` (Server-Sent Events format)
*   **Error Type:** `async_openai::error::OpenAIError` (wraps API errors like 4xx, 5xx status codes)