# Remediation Plan – Sprint 1

## Executive Summary
This plan targets critical stability, security, and correctness issues identified in the OpenAI Integration Foundation code review. We will prioritize eliminating API key leakage (cr-02), standardizing configuration parsing (cr-04), replacing fatal panics with graceful error handling (cr-01), ensuring reliable test execution via isolation (cr-03), and correcting misleading documentation (cr-05). This order addresses the most severe risks first, establishes a stable configuration foundation, ensures test reliability, and finally updates documentation to reflect the actual state.

## Strike List
| Seq | CR-ID | Title                                           | Effort | Owner?  |
|-----|-------|-------------------------------------------------|--------|---------|
| 1   | cr-02 | Redact API Keys in Config Logging & Debug       | s      | backend |
| 2   | cr-04 | Standardize Boolean Environment Var Parsing     | xs     | backend |
| 3   | cr-01 | Replace Config Panic with Graceful Error Handling | s      | backend |
| 4   | cr-03 | Ensure Test Isolation for Env Var Tests         | xs     | backend |
| 5   | cr-05 | Clarify OpenAI Feature Status in README         | xs     | backend |

## Detailed Remedies

### cr-02 Redact API Keys in Config Logging & Debug
- **Problem:** The `Config` struct derives `Debug` without redacting sensitive API keys, risking exposure in test/benchmark logs.
- **Impact:** Critical security vulnerability; sensitive API keys could be compromised via logs, CI output, or developer machines.
- **Chosen Fix:** Use the `secrecy` crate to wrap sensitive fields (`anthropic_api_key`, `openai_api_key`) in `Secret<String>`, leveraging its automatic `Debug` redaction. Add a test verifying redaction.
    *   Provides type-level safety against accidental exposure.
    *   Clearly marks points of secret handling via `.expose_secret()`.
    *   Aligns with security best practices.
- **Steps:**
  1. Add `secrecy = "1.0"` (or latest) to `[dependencies]` in `Cargo.toml`.
  2. Modify `Config` struct: change `anthropic_api_key: String` to `anthropic_api_key: Secret<String>` and `openai_api_key: Option<String>` to `openai_api_key: Option<Secret<String>>`.
  3. Update `load_config` (or its replacement post cr-01) to wrap loaded API keys using `Secret::new()`.
  4. Update all code locations that need the raw key value (e.g., HTTP clients) to call `.expose_secret()` on the `Secret` fields.
  5. Update `Config` struct instantiations in tests (`tests/common/mod.rs`, `tests/config_test.rs`) and benchmarks (`benches/*.rs`) to wrap dummy keys with `Secret::new()`.
  6. Add a unit test: create a `Config` with dummy keys, `format!("{:?}", config)`, and assert the output string contains `"[REDACTED]"` (or similar marker from `secrecy`) and *not* the dummy key values.
- **Done‑When:** `cargo test` passes, `cargo clippy` passes, manual inspection/grep of test/debug logs confirms keys are redacted, `cargo audit` clean.

### cr-04 Standardize Boolean Environment Var Parsing
- **Problem:** Boolean environment variable parsing (`LOG_BODIES`, `OPENAI_ENABLED`) is inconsistent and overly permissive, accepting non-standard values incorrectly; a test asserts this wrong behavior.
- **Impact:** User confusion, potential misconfiguration, incorrect test validation, violates Simplicity and Coding Standards.
- **Chosen Fix:** Implement a single private helper function for standardized boolean parsing, accepting only "true", "false", "1", "0" (case-insensitive), logging a warning and using the default for other values.
    *   Ensures consistent behavior across all boolean flags.
    *   Reduces code duplication (DRY).
    *   Improves clarity and maintainability.
- **Steps:**
  1. Define a private helper function `fn parse_bool_env(var_name: &str, default: bool) -> bool` within `src/config.rs`.
  2. Implement the function to read the env var, normalize to lowercase, return `true` for "true"/"1", `false` for "false"/"0", and log a warning (e.g., using `tracing::warn!`) returning `default` for all other values or if unset.
  3. Replace the existing parsing logic for `LOG_BODIES` and `OPENAI_ENABLED` in `load_config` (or its replacement post cr-01) with calls to this helper function.
  4. Update the `test_boolean_parsing` test in `tests/config_test.rs` to assert the *correct*, strict behavior (only "true"/"1", "false"/"0" parse definitively, others use default). Include cases for casing, valid inputs, invalid inputs, and unset.
- **Done‑When:** `cargo test` passes (including the updated `test_boolean_parsing`), manual testing confirms consistent parsing and warnings for invalid boolean env vars.

### cr-01 Replace Config Panic with Graceful Error Handling
- **Problem:** Application panics fatally if `OPENAI_ENABLED` is true but `OPENAI_API_KEY` is missing, treating a recoverable user config error as an unrecoverable program bug.
- **Impact:** Service crash (DoS vector), prevents graceful error reporting/logging, hinders testing of error paths, violates Rust error handling guidelines.
- **Chosen Fix:** Modify `load_config` to return `Result<Config, ConfigError>`, define a specific `ConfigError` enum using `thiserror`, and handle the `Result` gracefully in `main.rs`.
    *   Follows idiomatic Rust error handling practices.
    *   Makes configuration errors explicit, recoverable, and testable.
    *   Provides clear error messages to the user/operator.
- **Steps:**
  1. Add `thiserror = "1.0"` (or latest) to `[dependencies]` in `Cargo.toml`.
  2. Define a public `enum ConfigError` in `src/config.rs` using `#[derive(Debug, thiserror::Error)]`. Include variants like `#[error("OpenAI integration enabled but OPENAI_API_KEY environment variable is not set")] MissingOpenApiKey`, `#[error("Required environment variable {0} not set or empty")] MissingRequiredKey(&'static str)`, `#[error("Invalid value for environment variable {var}: '{value}'")] InvalidBooleanValue { var: String, value: String }`, etc.
  3. Change the signature of `load_config` to `pub fn load_config() -> Result<Config, ConfigError>`. (Note: This might require adjusting how/when a static `Config` is initialized, potentially loading in `main` first).
  4. Replace `panic!` and `expect` calls related to configuration loading/validation within `load_config` with `return Err(ConfigError::...)`. Use `?` for fallible operations like `env::var`.
  5. In `src/main.rs`, call `config::load_config()`. Match on the `Result`:
     - `Ok(config)`: Proceed with application startup (potentially initializing a `static OnceLock<Config>` or passing the config down).
     - `Err(e)`: Log the specific error using `tracing::error!` and exit the process gracefully with a non-zero status code (`std::process::exit(1)`).
  6. Update tests that previously expected panics or relied on infallible config loading. Test the `Err` variants returned by `load_config`.
- **Done‑When:** `cargo test` passes, running the application with invalid configuration (e.g., missing required keys) results in a clear error message and exit code 1 (no panic trace), error paths are covered by tests.

### cr-03 Ensure Test Isolation for Env Var Tests
- **Problem:** Tests modifying global environment variables (`tests/config_test.rs`) rely solely on a `Mutex`, which is insufficient to prevent race conditions and state leakage when tests run in parallel.
- **Impact:** Flaky tests, unreliable CI results, difficult debugging, hinders confidence in test suite. Violates Test Isolation.
- **Chosen Fix:** Use the `serial_test` crate to annotate tests that modify environment variables with `#[serial]`, forcing them to run sequentially while allowing other tests to run in parallel.
    *   Directly addresses the isolation need for the specific problematic tests.
    *   Minimal code change, avoids slowing down unrelated tests.
    *   Standard practice for managing shared state in Rust tests.
- **Steps:**
  1. Add `serial_test = "2.0"` (or latest) to `[dev-dependencies]` in `Cargo.toml`.
  2. Add `use serial_test::serial;` to `tests/config_test.rs`.
  3. Identify all `#[test]` functions in `tests/config_test.rs` that modify environment variables (e.g., using `std::env::set_var`, `std::env::remove_var`, or helpers that do so).
  4. Add the `#[serial]` attribute directly above the `#[test]` attribute for each identified function (e.g., `#[serial] #[test] fn test_config_with_env_var() { ... }`).
  5. Remove the `ENV_MUTEX` static variable and its usage (`_lock = ...`) as it's now redundant.
  6. Add a comment at the top of `tests/config_test.rs` explaining the use of `#[serial]` for tests modifying the environment.
- **Done‑When:** `cargo test -- --test-threads=N` (N > 1) passes consistently without failures related to environment variable conflicts in `config_test.rs`. `ENV_MUTEX` is removed.

### cr-05 Clarify OpenAI Feature Status in README
- **Problem:** The `README.md` implies OpenAI integration is functional and provides usage examples, failing to clarify that only foundational code exists and core logic is missing.
- **Impact:** Misleads users and contributors about feature readiness, sets incorrect expectations. Violates Documentation Approach (accuracy).
- **Chosen Fix:** Add a prominent, explicit note in relevant README sections stating the non-functional status and remove misleading usage examples.
    *   Provides clear and unambiguous information.
    *   Quickest way to correct the documentation.
- **Steps:**
  1. Edit `README.md`.
  2. In the main "Features" or "Overview" section mentioning OpenAI, add a highly visible note:
     ```markdown
     > **⚠️ Note:** OpenAI integration is currently **foundational only** and **not yet functional**. The necessary adapter logic and request routing are planned for future implementation. Configuration variables exist, but the proxy will not route requests to OpenAI at this time.
     ```
  3. Repeat or reference this note in the "Configuration" or "Environment Variables" section where `OPENAI_API_KEY` / `OPENAI_ENABLED` are described.
  4. Remove or comment out any specific command-line or API examples that demonstrate sending requests *through* the service *to* OpenAI (e.g., `curl ... /v1/chat/completions`). Clarify that examples involving OpenAI config variables are for setting up the foundation only.
- **Done‑When:** `README.md` accurately reflects that the OpenAI integration is not functional, clearly states its foundational status, and contains no misleading usage examples.

## Standards Alignment
- **Security:** CR-02 directly hardens against secret leakage, CR-01 removes a potential DoS vector.
- **Modularity:** CR-01 introduces a dedicated `ConfigError` type, CR-04 encapsulates parsing logic, CR-02 uses `secrecy` for modular secret handling.
- **Testability:** CR-01 makes config errors testable, CR-03 ensures reliable test execution by guaranteeing isolation for specific tests.
- **Coding Standards & Simplicity:** CR-01 adopts idiomatic `Result`-based error handling, CR-04 standardizes boolean parsing logic, CR-03 uses a standard testing utility.
- **Documentation:** CR-05 ensures documentation accurately reflects the system's state.

## Validation Checklist
- [ ] All automated tests pass (`cargo test`).
- [ ] Test suite passes reliably when run in parallel (`cargo test -- --test-threads=4` or similar).
- [ ] Static analysis (`cargo clippy -- -D warnings`) passes with no errors/warnings.
- [ ] Code formatting (`cargo fmt -- --check`) passes.
- [ ] Security audit (`cargo audit`) passes with no new vulnerabilities.
- [ ] Manual Test: Run application with `OPENAI_ENABLED=true` and missing `OPENAI_API_KEY`; verify graceful exit with specific error log message (no panic).
- [ ] Manual Test: Run application with invalid boolean values (e.g., `OPENAI_ENABLED=maybe`); verify warning log and use of default value.
- [ ] Manual Inspection/grep: Confirm no raw API keys appear in `cargo test` or `cargo run` (with debug logging) output.
- [ ] Review `README.md` changes for clarity and accuracy regarding OpenAI feature status.
