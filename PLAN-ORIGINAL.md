# PLAN: Implement OpenAI API Integration Adapter

## 1. Overview

This plan details the steps required to integrate OpenAI as an LLM provider into the Switchboard proxy service. The goal is to create a modular, testable, and maintainable adapter that handles request/response mapping and OpenAI-specific concerns, adhering to the project's development philosophy.

## 2. Configuration

2.1. **Add New Variables:** Introduce the following environment variables (and corresponding fields in `src/config.rs::Config`):
   * `OPENAI_API_KEY`: The API key for OpenAI. (Mandatory if OpenAI provider is enabled).
   * `OPENAI_API_BASE_URL`: The base URL for the OpenAI API (Default: `https://api.openai.com`).
   * `OPENAI_ENABLED`: Boolean flag to enable/disable the OpenAI provider (Default: `false`). Allows deploying the code without requiring an API key initially.
   * (Future Consideration for Routing Task): `DEFAULT_PROVIDER`: Could be extended to include `openai` (e.g., `anthropic` or `openai`). This is *not* part of this task but the config structure should allow it.

2.2. **Update Loading:** Modify `src/config.rs::load_config()` to load these new variables. Add validation to ensure `OPENAI_API_KEY` is present if `OPENAI_ENABLED` is true. Log the loaded OpenAI configuration values (except the API key).

2.3. **Update Documentation:** Update `README.md` to document the new environment variables.

## 3. Dependencies

3.1. **Add `async-openai` Crate:** Utilize the `async-openai` crate ([https://crates.io/crates/async-openai](https://crates.io/crates/async-openai)).
   * **Justification:** Reimplementing the OpenAI API, including request/response types, streaming logic, and error handling, would introduce significant complexity and maintenance burden. `async-openai` is a well-maintained, community-standard crate that abstracts these details, aligning with the "Simplicity First" and "Disciplined Dependency Management" principles (avoiding NIH syndrome for complex external APIs). It provides typed structs and handles underlying HTTP calls via `reqwest`, fitting our stack.
   * **Action:** Add `async-openai = "<latest_compatible_version>"` to `Cargo.toml` dependencies. Ensure compatibility with the existing `tokio` and `reqwest` versions.

## 4. Code Structure & Implementation

4.1. **Provider Abstraction (Conceptual):** While a full provider trait refactor is a future task, design the OpenAI integration with this in mind. The core proxy logic should ideally interact with different providers via a common interface in the future.

4.2. **New Module:** Create `src/openai_adapter.rs`.

4.3. **Configuration Struct:** Define a struct within `openai_adapter.rs` to hold OpenAI-specific configuration extracted from the main `Config` (API Key, Base URL).

4.4. **Adapter Function:** Create an async function, e.g., `handle_openai_request`, within `openai_adapter.rs`.
   * **Signature:** `async fn handle_openai_request(req: Request<Body>, config: Arc<OpenAIConfig>) -> Result<Response, OpenAIAdapterError>` (Error type TBD).
   * **Initialization:** Instantiate the `async_openai::Client` using the provided configuration.

4.5. **Request Mapping:**
   * Inside `handle_openai_request`, deserialize the incoming request body (`req.into_body()`) into a generic internal request format (or directly map if the proxy passes through Anthropic-like requests initially).
   * Map this internal/incoming format to `async_openai::types::CreateChatCompletionRequest`. This involves mapping messages, model name (initially hardcoded or passed via header/config), stream parameter, etc.
   * Handle potential deserialization errors.

4.6. **API Call:** Use the `async_openai::Client` to make the chat completion request (`client.chat().create(request)`). Handle both streaming (`client.chat().create_stream(request)`) and non-streaming cases based on the mapped request.

4.7. **Response Mapping:**
   * **Non-Streaming:** Map the `async_openai::types::CreateChatCompletionResponse` back to the format expected by the proxy client. Serialize the response body and construct an `axum::response::Response`.
   * **Streaming:** Map the `Stream<Result<CreateChatCompletionStreamResponse, OpenAIError>>` from `async-openai` to an Axum-compatible stream (`Body::wrap_stream`). Each chunk needs to be mapped to the expected output format (likely JSON lines or SSE). Ensure correct headers (e.g., `Content-Type: text/event-stream`) are set on the Axum response.

4.8. **Integration Point:** Modify `src/proxy_handler.rs::proxy_handler`.
   * Add logic (controlled by `OPENAI_ENABLED` config flag and potentially a header or path prefix for initial testing) to determine if the request should be routed to OpenAI.
   * If routing to OpenAI:
     * Instantiate the `OpenAIConfig` from the main `Config`.
     * Call `openai_adapter::handle_openai_request`.
     * Map the `OpenAIAdapterError` to an appropriate HTTP status code (`StatusCode`).
   * If not routing to OpenAI, proceed with the existing Anthropic logic.
   * **Note:** This initial routing logic will be basic, pending the "Basic Model Selection & Routing Logic" task.

## 5. Error Handling

5.1. **Adapter Error Type:** Define a specific error enum `OpenAIAdapterError` in `openai_adapter.rs` using `thiserror`.
   * Variants should cover:
     * Configuration errors (e.g., missing API key).
     * Request deserialization/mapping errors.
     * API call errors (wrapping `async_openai::error::OpenAIError`).
     * Response mapping/serialization errors.
     * Streaming errors.

5.2. **Mapping:** In `proxy_handler.rs`, map `OpenAIAdapterError` variants to appropriate HTTP `StatusCode`s (e.g., 400 for bad requests, 401/403 for auth errors, 429 for rate limits, 500/502 for server errors). Log the original error details.

## 6. Logging

6.1. **Request Forwarding:** Log at INFO level when a request is identified for forwarding to OpenAI, including the correlation ID and target model (if available).

6.2. **Adapter Logic:** Use DEBUG level logging within `openai_adapter.rs` for detailed mapping steps.

6.3. **API Interaction:** Log at DEBUG level before sending the request to OpenAI and upon receiving the response (headers only for streaming). Include status codes.

6.4. **Error Logging:** Log all errors at ERROR level, including the mapped error and the original error details (especially the wrapped `async_openai::error::OpenAIError`).

6.5. **Body Logging:** Respect the `log_bodies` and `log_max_body_size` configuration within the adapter, similar to the existing Anthropic logging.

## 7. Testing Strategy

7.1. **Unit Tests:**
   * Test request/response mapping functions within `openai_adapter.rs` with sample data.
   * Test error mapping logic.

7.2. **Integration Tests (`tests/openai_integration_test.rs`):**
   * Create a new test file.
   * Use `wiremock` to mock the OpenAI API endpoints (`/v1/chat/completions`).
   * **Test Cases:**
     * Successful non-streaming request/response cycle.
     * Successful streaming request/response cycle (mock multiple SSE events).
     * API error responses (e.g., 401 Unauthorized, 429 Rate Limit, 500 Server Error) and verify correct mapping to proxy response status codes.
     * Invalid incoming request body (verify 400 response).
     * Test with `OPENAI_ENABLED=true` and `OPENAI_ENABLED=false`.
   * Use the test setup utilities from `tests/common/mod.rs`, potentially adapting them to configure the mock server for OpenAI endpoints.

## 8. Documentation

8.1. **README.md:** Update the Environment Variables section with the new OpenAI variables (`OPENAI_API_KEY`, `OPENAI_API_BASE_URL`, `OPENAI_ENABLED`). Explain how to enable and configure the OpenAI provider.

8.2. **Code Documentation:** Add doc comments (`///`) to public functions, structs, and modules in `openai_adapter.rs` explaining their purpose and usage.

## 9. Dependencies & Sequencing

9.1. **Configuration:** This task implements the necessary configuration additions for OpenAI. It assumes basic config loading infrastructure exists.

9.2. **Routing:** This task focuses *only* on adding the adapter. The integration point in `proxy_handler.rs` will use a simple mechanism (e.g., config flag + maybe a specific path prefix like `/openai/v1/messages` or a header) to direct traffic to the new adapter. Full dynamic routing based on model names, headers, etc., is deferred to the "Basic Model Selection & Routing Logic" task. The adapter implementation should be agnostic to the specific routing mechanism used later.

## 10. Implementation Steps

10.1. **Setup:** Create branch `feat/openai-adapter`.

10.2. **Dependencies:** Add `async-openai` to `Cargo.toml`. Run `cargo build` to fetch.

10.3. **Configuration:** Implement changes in `src/config.rs` and update `README.md`.

10.4. **Adapter Module:** Create `src/openai_adapter.rs` and define initial structs/functions (`OpenAIConfig`, `OpenAIAdapterError`, `handle_openai_request` skeleton).

10.5. **Request/Response Mapping:** Implement the core mapping logic within `handle_openai_request` using `async-openai` types. Handle both streaming and non-streaming.

10.6. **Error Handling:** Implement `OpenAIAdapterError` and mapping logic.

10.7. **Logging:** Add structured logging calls within the adapter.

10.8. **Integration Point:** Modify `proxy_handler.rs` to conditionally call the OpenAI adapter based on `OPENAI_ENABLED` and a temporary routing mechanism (e.g., path prefix).

10.9. **Unit Tests:** Write unit tests for mapping and error handling.

10.10. **Integration Tests:** Write integration tests using `wiremock` in `tests/openai_integration_test.rs`.

10.11. **Review & Refactor:** Ensure code adheres to project standards and philosophy. Run `cargo fmt`, `cargo clippy`, `cargo test`.

10.12. **Final Documentation:** Update `README.md` thoroughly.

## 11. Deliverables

* Updated `src/config.rs` with OpenAI configuration.
* New `src/openai_adapter.rs` module implementing the adapter logic.
* Modifications to `src/proxy_handler.rs` to integrate the adapter.
* New `tests/openai_integration_test.rs` file with integration tests.
* Updated `Cargo.toml` and `Cargo.lock` with new dependencies.
* Updated `README.md` documentation.

## 12. OpenAI API Reference

For the implementation, here are key API details:

### OpenAI API Endpoints
- Base URL: `https://api.openai.com`
- Chat Completions: `/v1/chat/completions`

### Request Structure
The Chat Completions API expects a JSON body with:
- `model`: String (e.g., "gpt-4", "gpt-3.5-turbo")
- `messages`: Array of message objects with `role` ("system", "user", "assistant") and `content`
- `stream`: Boolean (default: false)
- Optional parameters: `temperature`, `max_tokens`, `top_p`, etc.

### Response Structure
- Non-streaming: JSON object with `id`, `choices` (array with `message`, `finish_reason`), `usage`, etc.
- Streaming: Server-Sent Events (SSE) with delta updates (partial content chunks)

### Error Responses
- Authentication errors: 401 Unauthorized
- Rate limits: 429 Too Many Requests
- Validation errors: 400 Bad Request
- Server errors: 500 Internal Server Error, 503 Service Unavailable

This reference will help ensure accurate implementation of the request/response mapping logic.