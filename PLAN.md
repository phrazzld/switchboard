# PLAN-1: Foundation - OpenAI Configuration & Dependencies

## 1. Overview

This plan details the first phase of integrating OpenAI as an LLM provider into the Switchboard proxy service. The goal is to establish the necessary configuration settings, add the external dependency (`async-openai`), update documentation, and prepare the basic project structure for the OpenAI adapter. This foundational work enables subsequent implementation phases.

## 2. Context

To support OpenAI alongside existing providers, the application requires:
*   New configuration parameters (API key, base URL, enable flag) managed via environment variables.
*   A dedicated library (`async-openai`) to interact with the OpenAI API, avoiding reimplementation of HTTP calls and data structures.
*   A placeholder module for the future adapter code.
*   Documentation reflecting the new configuration options.

This groundwork allows subsequent development phases (adapter logic, integration) to proceed smoothly on a stable base.

## 3. Approach

1.  **Configuration:** Define new OpenAI-related environment variables and corresponding fields in the `Config` struct. Update the configuration loading mechanism to handle these variables, including validation (e.g., API key required if enabled) and logging.
2.  **Dependency:** Add the `async-openai` crate as a project dependency using `Cargo.toml`.
3.  **Structure:** Create an empty module file (`src/openai_adapter.rs`) as a placeholder for the adapter logic.
4.  **Documentation:** Update `README.md` to document the new configuration variables.
5.  **Testing:** Add unit tests specifically for the new configuration loading and validation logic.

## 4. Detailed Steps

4.1.  **Add Config Fields (`src/config.rs`):**
    *   Extend the `Config` struct with:
        *   `openai_api_key: Option<String>`
        *   `openai_api_base_url: String`
        *   `openai_enabled: bool`
    *   Define corresponding environment variable names (e.g., `OPENAI_API_KEY`, `OPENAI_API_BASE_URL`, `OPENAI_ENABLED`).

4.2.  **Update Config Loading (`src/config.rs::load_config()`):**
    *   Modify the function to load the new environment variables.
    *   Apply default values (`openai_api_base_url` defaults to `https://api.openai.com`, `openai_enabled` defaults to `false`).
    *   Implement validation: If `openai_enabled` is `true`, ensure `openai_api_key` is present; otherwise, return an error.
    *   Log the loaded OpenAI configuration values (base URL, enabled status) at INFO or DEBUG level upon startup if enabled. Avoid logging the API key.

4.3.  **Add Dependency (`Cargo.toml`):**
    *   Add `async-openai = "<latest_compatible_version>"` to the `[dependencies]` section.
    *   Run `cargo build` or `cargo update` to fetch the dependency and update `Cargo.lock`. Verify compatibility with existing project dependencies (e.g., `tokio`, `reqwest`).

4.4.  **Create Placeholder Module:**
    *   Create an empty file: `src/openai_adapter.rs`.

4.5.  **Update Documentation (`README.md`):**
    *   Add a section under "Environment Variables" documenting `OPENAI_API_KEY`, `OPENAI_API_BASE_URL`, and `OPENAI_ENABLED`. Briefly explain their purpose, defaults, and requirements (e.g., key required if enabled).

4.6.  **Add Unit Tests:**
    *   In the relevant test module (e.g., `src/config.rs` tests or `tests/config_tests.rs`), add unit tests to verify:
        *   Correct loading of OpenAI config variables and defaults.
        *   Validation logic (error if enabled without API key).

4.7.  **Review & Refactor:**
    *   Run `cargo fmt`, `cargo clippy`, `cargo test` to ensure code quality and confirm tests pass.

## 5. Deliverables

*   Updated `src/config.rs` with new OpenAI fields, loading, validation, and logging logic.
*   Unit tests covering the new configuration logic.
*   Updated `Cargo.toml` and `Cargo.lock` including the `async-openai` dependency.
*   New empty file: `src/openai_adapter.rs`.
*   Updated `README.md` with documentation for the new environment variables.

## 6. Dependency Notes

*   **Independent:** This plan is the foundational step and has no dependencies on other plans in this scope.
*   **Enables:** This plan must be completed before **PLAN-2**, **PLAN-3**, and **PLAN-4**, as they rely on the configuration structure, the `async-openai` dependency, and the existence of `src/openai_adapter.rs`.