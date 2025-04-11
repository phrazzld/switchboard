# Define TestSetup Struct in tests/common/mod.rs - Plan

## Task Title
Define `TestSetup` Struct in `tests/common/mod.rs`

## Implementation Approach
Define the `TestSetup` struct in the `tests/common/mod.rs` file with fields for `client: Client`, `config: Config`, `mock_server: MockServer`, and `app: Router`. Include the required imports for these types from the appropriate modules: `wiremock::MockServer`, `switchboard::config::Config`, `reqwest::Client`, and `axum::Router`.