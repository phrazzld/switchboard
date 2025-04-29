# PLAN-3: Integration - Connect Adapter to Proxy Handler

## 1. Overview

This plan details the integration of the implemented OpenAI adapter module (`src/openai_adapter.rs`) into the main request handling logic within `src/proxy_handler.rs`. This involves adding conditional routing logic to direct requests to the adapter based on configuration and a simple routing criterion, invoking the adapter, and mapping its results (both success and error) back to appropriate HTTP responses.

## 2. Context

The OpenAI adapter exists as a self-contained, unit-tested component (from PLAN-2). This plan bridges the gap by modifying the core proxy request handler (`proxy_handler`) to conditionally route traffic to this new adapter, enabling end-to-end request flow for OpenAI requests while preserving existing functionality.

## 3. Approach

1.  **Modify Proxy Handler:** Update `src/proxy_handler.rs::proxy_handler`.
2.  **Conditional Routing:** Check the `OPENAI_ENABLED` flag from the configuration. Implement a simple routing mechanism (e.g., based on request path prefix or a specific header) to identify OpenAI requests.
3.  **Adapter Invocation:** If routing criteria are met, instantiate the adapter's configuration (`OpenAIConfig`) and call `openai_adapter::handle_openai_request`.
4.  **Result Mapping:** Handle the `Result` returned by the adapter: return the `Response` on success, or map the `OpenAIAdapterError` to an appropriate HTTP `StatusCode` and response body on failure.
5.  **Preserve Existing Path:** Ensure requests not matching the OpenAI criteria continue to be handled by the existing logic (e.g., Anthropic provider).
6.  **Logging:** Add logging for routing decisions and error mapping.

## 4. Detailed Steps

4.1.  **Modify `src/proxy_handler.rs::proxy_handler`:**
    *   Import necessary types (`handle_openai_request`, `OpenAIConfig`, `OpenAIAdapterError`) from `src/openai_adapter.rs`.
    *   Ensure access to the main application `Config` (likely available via Axum state).

4.2.  **Implement Routing Logic:**
    *   Add conditional logic near the beginning of the handler.
    *   **Condition:** Check if `config.openai_enabled` is true **AND** if the incoming request matches a predefined criterion for OpenAI routing.
        *   **Initial Criterion:** Use a simple, temporary mechanism like a path prefix (e.g., `/openai/v1/chat/completions`) or a specific header (`X-Provider: openai`). Document this choice clearly. *Note: This basic routing may be enhanced later.*
    *   If the condition is met, proceed to invoke the OpenAI adapter.
    *   If the condition is not met, execute the existing handler logic (fall-through to Anthropic/default provider).

4.3.  **Invoke OpenAI Adapter:**
    *   Inside the "route to OpenAI" branch:
        *   Create an instance of `openai_adapter::OpenAIConfig` using values from the main `Config` (API key, base URL, etc.). Wrap it in an `Arc` if required by the adapter function signature.
        *   Call `openai_adapter::handle_openai_request(req, adapter_config).await`.

4.4.  **Map Adapter Result:**
    *   Handle the `Result<Response, OpenAIAdapterError>`:
        *   If `Ok(response)`, return the response directly.
        *   If `Err(adapter_error)`:
            *   Log the `adapter_error` details at `tracing::error!`.
            *   Map `adapter_error` variants to appropriate `axum::http::StatusCode`:
                *   `RequestMappingError` -> `StatusCode::BAD_REQUEST` (400)
                *   `ApiError(OpenAIError)` -> Map based on underlying API error status (e.g., 401 -> `UNAUTHORIZED`, 429 -> `TOO_MANY_REQUESTS`, 500 -> `BAD_GATEWAY` (502), etc.) or `INTERNAL_SERVER_ERROR` (500) / `BAD_GATEWAY` (502) as a fallback.
                *   `ResponseMappingError`, `StreamingError`, `ConfigError` -> `StatusCode::INTERNAL_SERVER_ERROR` (500) or `BAD_GATEWAY` (502).
            *   Construct and return a simple `axum::response::Response` with the mapped status code and a minimal error body (e.g., plain text or JSON).

4.5.  **Logging:**
    *   Add an `tracing::info!` message when a request is routed to the OpenAI adapter, including correlation ID.
    *   Ensure adapter errors mapped in the handler are logged appropriately before returning the client-facing error response.

4.6.  **Code Documentation:**
    *   Add/update doc comments (`///`) in `proxy_handler.rs` explaining the new conditional routing logic, adapter invocation, and error mapping.

4.7.  **Review & Refactor:**
    *   Ensure changes integrate cleanly. Verify the non-OpenAI path is unaffected.
    *   Run `cargo fmt`, `cargo clippy`, `cargo test` (existing tests should pass).

## 5. Deliverables

*   Modified `src/proxy_handler.rs` incorporating:
    *   Conditional OpenAI routing logic (based on `OPENAI_ENABLED` and temporary criterion).
    *   Invocation of `openai_adapter::handle_openai_request`.
    *   Mapping of `OpenAIAdapterError` to HTTP `StatusCode` and response.
    *   Associated logging.
*   Updated doc comments in `proxy_handler.rs`.

## 6. Dependency Notes

*   **Depends On:** **PLAN-2** (requires the completed `openai_adapter::handle_openai_request` function and `OpenAIAdapterError` type).
*   **Depends On:** **PLAN-1** (requires the `OPENAI_ENABLED` flag and access to OpenAI configuration).
*   **Enables:** **PLAN-4** (allows end-to-end integration testing of the OpenAI flow).