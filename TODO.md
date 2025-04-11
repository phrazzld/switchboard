# TODO

## Setup Testing Infrastructure

- [x] **Task Title:** Create Test Directory Structure
    - **Action:** Create the `tests/` directory at the project root (`switchboard/tests/`) and a `common/` subdirectory within it (`switchboard/tests/common/`). Create empty files `switchboard/tests/common/mod.rs` and `switchboard/tests/proxy_integration_tests.rs`.
    - **Depends On:** None
    - **AC Ref:** AC1, AC2, AC3 (file existence), AC4 (file existence)

- [x] **Task Title:** Add Dev Dependencies to `Cargo.toml`
    - **Action:** Add a `[dev-dependencies]` section to `switchboard/Cargo.toml` (if not present) and include the specified testing crates: `tokio`, `wiremock`, `serde_json`, `axum` (with `test-helpers` feature if needed), `tower` (with `util` feature), `http-body-util`, and `uuid`. Ensure `tokio` features align with main dependencies and include `rt-multi-thread`.
    - **Depends On:** None
    - **AC Ref:** AC5

- [ ] **Task Title:** Define `TestSetup` Struct in `tests/common/mod.rs`
    - **Action:** Define the `TestSetup` struct within `switchboard/tests/common/mod.rs` containing fields for `client: Client`, `config: Config`, `mock_server: MockServer`, and `app: Router`. Include necessary imports (`wiremock::MockServer`, `switchboard::config::Config`, `reqwest::Client`, `axum::Router`).
    - **Depends On:** Create Test Directory Structure
    - **AC Ref:** AC6

- [ ] **Task Title:** Implement `setup_test_environment` Function Skeleton in `tests/common/mod.rs`
    - **Action:** Define an `async fn setup_test_environment() -> TestSetup` function within `switchboard/tests/common/mod.rs`. Include necessary imports (`std::time::Duration`).
    - **Depends On:** Create Test Directory Structure, Define `TestSetup` Struct in `tests/common/mod.rs`
    - **AC Ref:** AC7 (partially - function signature)

- [ ] **Task Title:** Integrate `wiremock::MockServer` Startup into `setup_test_environment`
    - **Action:** Within the `setup_test_environment` function, add code to start a `wiremock::MockServer` instance using `MockServer::start().await` and store it.
    - **Depends On:** Add Dev Dependencies to `Cargo.toml`, Implement `setup_test_environment` Function Skeleton in `tests/common/mod.rs`
    - **AC Ref:** AC8

- [ ] **Task Title:** Create Test-Specific `Config` in `setup_test_environment`
    - **Action:** Within the `setup_test_environment` function, create an instance of `switchboard::config::Config`, setting `anthropic_target_url` to the `mock_server.uri()` and using appropriate dummy values for other fields (e.g., `port`, `anthropic_api_key`, `log_level`).
    - **Depends On:** Integrate `wiremock::MockServer` Startup into `setup_test_environment`
    - **AC Ref:** AC9

- [ ] **Task Title:** Build Test `reqwest::Client` in `setup_test_environment`
    - **Action:** Within the `setup_test_environment` function, create a `reqwest::Client` instance using `Client::builder()` with suitable timeouts for testing.
    - **Depends On:** Add Dev Dependencies to `Cargo.toml`, Implement `setup_test_environment` Function Skeleton in `tests/common/mod.rs`
    - **AC Ref:** AC10

- [ ] **Task Title:** Instantiate Test `axum::Router` in `setup_test_environment`
    - **Action:** Within the `setup_test_environment` function, call `switchboard::proxy_handler::create_router`, passing the test `reqwest::Client` and the test `Config`. Use `Box::leak` for the config as a temporary measure as shown in the plan, acknowledging the need for future refinement. Populate the `TestSetup` struct with the client, config, mock server, and router, and return it.
    - **Depends On:** Add Dev Dependencies to `Cargo.toml`, Create Test-Specific `Config` in `setup_test_environment`, Build Test `reqwest::Client` in `setup_test_environment`
    - **AC Ref:** AC11, AC7 (fully)

- [ ] **Task Title:** Declare `common` Module in `proxy_integration_tests.rs`
    - **Action:** Add `mod common;` at the top of `switchboard/tests/proxy_integration_tests.rs` to make the shared utilities accessible.
    - **Depends On:** Create Test Directory Structure
    - **AC Ref:** Implicit requirement for AC12

- [ ] **Task Title:** Write Basic Integration Test Skeleton in `proxy_integration_tests.rs`
    - **Action:** Create an `async fn test_simple_post_forward_success()` test function annotated with `#[tokio::test]` in `switchboard/tests/proxy_integration_tests.rs`. Import and call `common::setup_test_environment().await` to get the `TestSetup` instance. Import necessary types (`axum::body::Body`, `axum::http::{Request, StatusCode}`, `tower::ServiceExt`, `http_body_util::BodyExt`).
    - **Depends On:** Create Test Directory Structure, Add Dev Dependencies to `Cargo.toml`, Instantiate Test `axum::Router` in `setup_test_environment`, Declare `common` Module in `proxy_integration_tests.rs`
    - **AC Ref:** AC12, AC13

- [ ] **Task Title:** Define `wiremock::Mock` Expectation in Basic Test
    - **Action:** Inside the `test_simple_post_forward_success` function, use `wiremock::Mock::given(...)` with appropriate matchers (`method`, `path`) to define an expectation on the mock server for a POST request to `/v1/messages`. Use `respond_with` to return a sample JSON success response (e.g., `200 OK` with `{"status": "ok"}`). Mount the mock onto the server instance from `TestSetup`. Import `wiremock::{Mock, ResponseTemplate, matchers::{method, path}}`.
    - **Depends On:** Add Dev Dependencies to `Cargo.toml`, Write Basic Integration Test Skeleton in `proxy_integration_tests.rs`
    - **AC Ref:** AC14

- [ ] **Task Title:** Send Test Request using `tower::ServiceExt::oneshot` in Basic Test
    - **Action:** Inside the `test_simple_post_forward_success` function, construct an `http::Request` with method POST, URI `/v1/messages`, appropriate headers (e.g., `content-type: application/json`), and a sample JSON body. Use `app.oneshot(request).await` to send the request to the test router instance obtained from `TestSetup`.
    - **Depends On:** Add Dev Dependencies to `Cargo.toml`, Write Basic Integration Test Skeleton in `proxy_integration_tests.rs`
    - **AC Ref:** AC15

- [ ] **Task Title:** Assert Response Status and Body in Basic Test
    - **Action:** Inside the `test_simple_post_forward_success` function, after receiving the response from `oneshot`, assert that the response status code is `StatusCode::OK`. Extract the response body bytes, deserialize it as `serde_json::Value`, and assert that it matches the expected JSON (`{"status": "ok"}`). Import `serde_json`.
    - **Depends On:** Add Dev Dependencies to `Cargo.toml`, Send Test Request using `tower::ServiceExt::oneshot` in Basic Test
    - **AC Ref:** AC16, AC17, AC18 (implicitly tested when running `cargo test`)

## [!] CLARIFICATIONS NEEDED / ASSUMPTIONS

- [ ] **Issue/Assumption:** Assumed `switchboard::proxy_handler::create_router` function exists and accepts `reqwest::Client` and `&'static Config` as arguments.
    - **Context:** PLAN.md Step 3 (`Implement Shared Test Utilities`) and Step 4 (`Write Basic Integration Test`) rely on this function signature. Existing code snippets (`src/main.rs`, `src/proxy_handler.rs`) confirm this signature.

- [ ] **Issue/Assumption:** Acknowledged use of `Box::leak` for passing `Config` to `create_router` in tests is a temporary simplification as noted in PLAN.md Step 3 Self-Correction.
    - **Context:** PLAN.md Step 3 notes this is not ideal and suggests future refinement (e.g., using `Arc<Config>`). This initial setup proceeds with the leak for simplicity as planned.

- [ ] **Issue/Assumption:** Assumed project root directory is named `switchboard`.
    - **Context:** PLAN.md Step 1 specifies paths relative to `switchboard/`.

- [ ] **Issue/Assumption:** Assumed the specified dev-dependency versions in PLAN.md Step 2 are compatible with the project's current Rust toolchain version.
    - **Context:** PLAN.md Step 2 lists specific versions. Compatibility might require adjustments or toolchain updates.