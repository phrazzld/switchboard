# PLAN-4: Validation - Integration Testing & Documentation

## 1. Overview

This plan details the final phase of integrating OpenAI: adding comprehensive integration tests using API mocking (`wiremock`) to validate the end-to-end request flow, and finalizing all related documentation (`README.md`, code comments). This ensures the feature is robust, works as expected under various conditions, and is clearly documented.

## 2. Context

With the OpenAI adapter implemented (PLAN-2) and integrated into the proxy handler (PLAN-3), it's crucial to verify the complete flow from incoming request to proxy response, including interactions with the (mocked) backend API. Integration tests provide this end-to-end validation, complementing the unit tests from PLAN-2. Finalizing documentation makes the feature ready for use and maintenance.

## 3. Approach

1.  **Integration Testing:** Use `wiremock` to create a mock OpenAI API server. Write integration tests that send requests to the Switchboard proxy (configured to use the mock server) and verify the proxy's responses under various scenarios (success, errors, streaming, configuration flags).
2.  **Documentation:** Review, update, and finalize the `README.md` section for OpenAI configuration and usage. Ensure comprehensive doc comments (`///`) exist in the new/modified code.
3.  **Final Quality Checks:** Run linters, formatters, and the full test suite (`cargo test --all`) to ensure code quality and correctness.

## 4. Detailed Steps

4.1.  **Create Integration Test File:**
    *   Create a new file, e.g., `tests/openai_integration_test.rs`.
    *   Include necessary test harness setup from `tests/common/mod.rs` (e.g., starting the Axum server, creating HTTP clients).

4.2.  **Setup `wiremock`:**
    *   Add `wiremock` as a dev-dependency if not already present.
    *   In the test setup, start a `wiremock::MockServer`.
    *   Configure the test instance of the Switchboard proxy to:
        *   Set `OPENAI_ENABLED=true`.
        *   Set `OPENAI_API_BASE_URL` to the mock server's URL.
        *   Provide a dummy `OPENAI_API_KEY`.

4.3.  **Implement `wiremock` Mocks:**
    *   Define mocks for the OpenAI Chat Completions endpoint (`/v1/chat/completions`) covering:
        *   Successful non-streaming response (HTTP 200, valid JSON body).
        *   Successful streaming response (HTTP 200, `Content-Type: text/event-stream`, body with multiple SSE chunks and `[DONE]`).
        *   API error responses (e.g., HTTP 401, 429, 500 with corresponding OpenAI error JSON bodies).

4.4.  **Implement Integration Test Cases:**
    *   Write tests using an HTTP client (e.g., `reqwest`) to send requests to the running proxy service.
    *   **Test Case: Success Non-Streaming:** Send valid request, assert mock received it, mock success, assert proxy returns 200 OK with correctly mapped body.
    *   **Test Case: Success Streaming:** Send valid streaming request, assert mock received it, mock streaming success, assert proxy returns 200 OK with `text/event-stream` and correctly formatted streaming body.
    *   **Test Case: API Errors:** For each mocked error (401, 429, 500, etc.), send request, mock error, assert proxy returns the correctly mapped HTTP status code (e.g., 401, 429, 502/500) and error body.
    *   **Test Case: Invalid Proxy Request:** Send request with invalid JSON body to the OpenAI path, assert proxy returns 400 Bad Request.
    *   **Test Case: `OPENAI_ENABLED` Flag:**
        *   Run tests with `OPENAI_ENABLED=true` (default setup).
        *   Reconfigure proxy test instance with `OPENAI_ENABLED=false`. Send request to OpenAI path, assert mock server was *not* called, assert proxy returns appropriate response (e.g., 404 Not Found, 501 Not Implemented, or fall-through depending on handler logic).

4.5.  **Final Documentation Review:**
    *   Review and finalize the `README.md` section on OpenAI. Ensure clarity on environment variables, enabling the provider, and any specific routing requirements (path/header used in PLAN-3).
    *   Review and polish all doc comments (`///`) added in `config.rs`, `openai_adapter.rs`, and `proxy_handler.rs` for accuracy and completeness.

4.6.  **Final Code Quality Checks:**
    *   Run `cargo fmt --all --check`.
    *   Run `cargo clippy --all-targets -- -D warnings`.
    *   Run `cargo test --all`.
    *   Ensure all checks pass, including CI pipeline if applicable.

## 5. Deliverables

*   New integration test file `tests/openai_integration_test.rs` with comprehensive scenarios using `wiremock`.
*   Potentially updated test helpers in `tests/common/mod.rs`.
*   Finalized and polished `README.md` documentation for OpenAI usage.
*   Completed and polished code documentation (doc comments) in relevant source files.
*   Clean results from `cargo fmt`, `cargo clippy`, and `cargo test`.

## 6. Dependency Notes

*   **Depends On:** **PLAN-3** (requires the OpenAI adapter to be integrated into the proxy handler for end-to-end testing).
*   **Validates:** The combined functionality delivered across PLAN-1, PLAN-2, and PLAN-3.
*   **Final Step:** This plan marks the completion of the initial OpenAI integration scope covered by these four plans.