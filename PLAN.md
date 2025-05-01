```
# Remediation Plan – Sprint 5

## Executive Summary
This sprint targets four critical, root-cause issues in configuration and testing: eliminating fatal panics in configuration access, bulletproofing API key redaction in logs, enforcing strict and consistent boolean environment parsing (in both code and tests), and refactoring test helpers to be robust and non-panicking. We prioritize fixes according to security and stability impact, sequencing foundational changes before test correctness to prevent rework and ensure airtight guarantees.

## Strike List
| Seq | CR‑ID   | Title                                      | Effort | Owner   |
|-----|---------|--------------------------------------------|--------|---------|
| 1   | cr‑01   | Eliminate config access panic              | s      | backend |
| 2   | cr‑02   | Bulletproof API key redaction              | s      | backend |
| 3   | cr‑04   | Strict boolean env parsing in tests        | s      | backend |
| 4   | cr‑03   | Non-panicking, flexible test helpers       | s      | backend |

## Detailed Remedies

### cr‑01 Eliminate config access panic
- **Problem:** `get_config()` panics if called before initialization due to `.expect()`.
- **Impact:** Fatal crash risk, undermines application stability and Rust error handling idioms, blocks testability and clean error reporting.
- **Chosen Fix:** Change `get_config()` to return `Result<&'static Config, ConfigError>`, returning a clear error if not initialized; update all usages to handle the error.
- **Steps:**
  1. Refactor `get_config()` to `pub fn get_config() -> Result<&'static Config, ConfigError>`, returning `CONFIG.get().ok_or(ConfigError::NotInitialized)`.
  2. Update all code and test call sites to handle or propagate the error, removing all `.expect()` on config access.
  3. Add/adjust tests verifying correct error handling when config is not initialized.
- **Done‑When:** All panics removed, errors are properly handled, tests pass with the new API.

### cr‑02 Bulletproof API key redaction
- **Problem:** Deriving `Debug` on `Config` relies on `SecretString`'s redaction, but accidental non-Debug formatting or future changes could leak secrets.
- **Impact:** Security: API keys/tokens could leak via logs or panic output, violating hard security requirements.
- **Chosen Fix:** Audit and enforce safe logging of all `Config` output, and add a unit test asserting that no secrets leak in any `Debug` output.
- **Steps:**
  1. Audit all code for any logging or formatting of `Config`; ensure only `Debug` (`{:?}`) formatting is used for secrets, and that non-Debug logs omit keys entirely.
  2. Strengthen the `test_config_debug_redaction` unit test: create a `Config` with known secret values, ensure `format!("{:?}", config)` never contains the raw values, only `[REDACTED]`.
  3. Add a doc comment or static analyzer lint to warn if `Config` is ever formatted via `Display` or otherwise unsafely.
- **Done‑When:** No secrets in logs (manual and test check), redaction test green, all code/docs clearly state and enforce safe logging policy.

### cr‑04 Strict boolean env parsing in tests
- **Problem:** Test helpers and tests do not use `parse_bool_env` for parsing, and existing tests assert the old, incorrect behavior.
- **Impact:** Tests are misleading and do not verify production behavior, risking undetected bugs and regressions.
- **Chosen Fix:** Refactor all test helpers to use `parse_bool_env` for all boolean fields, and rewrite tests to assert only the strict, explicit parsing behavior.
- **Steps:**
  1. Refactor all relevant test helpers (`create_test_config_with_env` in both test modules) to use `parse_bool_env` for `LOG_BODIES` and `OPENAI_ENABLED`, handling errors appropriately (use default or propagate).
  2. Rewrite `test_boolean_parsing` and all similar tests to assert *only* the allowed inputs for true/false, and error/default otherwise.
  3. Remove any broad or legacy logic (e.g., `!= "false"` checks) and replace with only the strict parser.
- **Done‑When:** All boolean parsing in tests matches production; test assertions enforce strict rules; all tests pass.

### cr‑03 Non-panicking, flexible test helpers
- **Problem:** Test helpers (`create_test_config_with_env`) panic on empty/missing env vars, making negative-path tests impossible and introducing fragility.
- **Impact:** Can't test config error cases, fragile and misleading tests, violates testability and error-handling standards.
- **Chosen Fix:** Refactor helpers to return `Result<Config, ConfigError>`, matching production, and update all test call sites to use this result.
- **Steps:**
  1. Refactor both helper functions into one shared test utility returning `Result<Config, ConfigError>`, mirroring production `load_config`.
  2. Update all test call sites to handle the `Result`, using `.unwrap()` where success is expected and matching errors where negative tests are desired.
  3. Add/expand tests for negative paths: missing/empty required keys, invalid values, etc.
- **Done‑When:** No test helper panics on missing/empty keys; error handling is explicit and robust; negative-path tests pass.

## Standards Alignment
- **Simplicity:** Each fix reduces ambiguity, removes hidden side effects, and clarifies API contracts.
- **Modularity:** Config and test helpers are explicit, composable, and isolated; test utility consolidation reduces duplication.
- **Testability:** Test helpers now allow all cases (including error paths); tests assert actual production behavior.
- **Coding Standards:** No panics except for unrecoverable bugs; errors are handled and typed; logging/formatting is explicit and safe.
- **Security:** API keys/tokens cannot leak via logs or panics; error outputs never expose secrets.

## Validation Checklist
- All automated tests (`cargo test`) pass.
- Static analyzers (`cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit`) report no new warnings/errors.
- Manual pen-test: verify no secrets in log output; intentionally trigger config errors and verify graceful, non-panicking, non-leaking error handling.
- `Debug` output of `Config` always redacts secrets.
- No new lint or audit warnings introduced.
```