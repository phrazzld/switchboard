# Todo

## Configuration Core (`src/config.rs`)
- [x] **T001 · Refactor · P0: refactor `get_config` to return `Result`**
    - **Context:** cr-01 Eliminate config access panic, Step 1
    - **Action:**
        1. Change `get_config` signature to `pub fn get_config() -> Result<&'static Config, ConfigError>`.
        2. Implement the function to return `CONFIG.get().ok_or(ConfigError::NotInitialized)`.
        3. Define or ensure `ConfigError::NotInitialized` exists and is appropriate.
    - **Done‑when:**
        1. `get_config` signature and implementation match the action steps.
        2. `ConfigError` includes a `NotInitialized` variant.
        3. Code compiles (call sites will be broken until T002).
    - **Depends‑on:** none

- [x] **T002 · Refactor · P0: update all `get_config` call sites to handle `Result`**
    - **Context:** cr-01 Eliminate config access panic, Step 2
    - **Action:**
        1. Find all usages of `get_config()` in `src/` and `tests/` (excluding `src/config.rs` itself).
        2. Update each call site to handle the `Result<&'static Config, ConfigError>`, removing any `.expect()` calls (use `?`, `match`, `.unwrap()` in tests where appropriate).
    - **Done‑when:**
        1. All production and test code call sites of `get_config` correctly handle the `Result`.
        2. No `.expect()` calls remain on `get_config` results.
        3. Code compiles and relevant existing tests pass (or are updated).
    - **Verification:**
        1. Grep codebase for `get_config(...).expect(` to ensure none remain.
        2. Run `cargo check --all-targets` and `cargo test`.
    - **Depends‑on:** [T001]

- [ ] **T003 · Test · P2: add tests for `get_config` uninitialized error**
    - **Context:** cr-01 Eliminate config access panic, Step 3
    - **Action:**
        1. Write a new unit test ensuring `get_config()` returns `Err(ConfigError::NotInitialized)` when called *before* config initialization.
        2. Ensure test setup specifically avoids initializing the global config for this test case.
    - **Done‑when:**
        1. A test explicitly asserts `get_config()` returns `Err(ConfigError::NotInitialized)` when uninitialized.
        2. The test passes.
    - **Depends‑on:** [T002]

## Configuration Logging & Security (`src/config.rs`, codebase-wide logging)
- [ ] **T004 · Chore · P0: audit and enforce safe `Config` logging**
    - **Context:** cr-02 Bulletproof API key redaction, Step 1
    - **Action:**
        1. Audit all code locations (including `Debug` derives, `log::*` macros, `println!`, etc.) where the `Config` struct or its fields might be formatted to string.
        2. Ensure only `Debug` formatting (`{:?}`) is used where secrets might be present.
        3. Refactor any non-Debug logging/formatting to explicitly omit secret fields.
    - **Done‑when:**
        1. Audit completed and documented (e.g., in PR description).
        2. Necessary code changes made to prevent non-`Debug` formatting of secrets.
    - **Verification:**
        1. Run the application with trace logging enabled.
        2. Trigger code paths that log config-related information.
        3. Inspect logs (stdout/stderr/files) to confirm no raw secrets appear, only `[REDACTED]` or similar placeholders used by `SecretString`.
    - **Depends‑on:** none

- [ ] **T005 · Test · P0: strengthen `test_config_debug_redaction` unit test**
    - **Context:** cr-02 Bulletproof API key redaction, Step 2
    - **Action:**
        1. Locate or create the `test_config_debug_redaction` unit test in `src/config.rs`.
        2. Construct a `Config` instance with known, non-default secret values (e.g., API keys).
        3. Assert that the output of `format!("{:?}", config)` contains the appropriate redaction placeholder (e.g., `[REDACTED]`) and *never* contains the raw secret values.
    - **Done‑when:**
        1. The test robustly checks `Debug` output against known raw secrets.
        2. The test passes.
    - **Depends‑on:** [T004]

- [ ] **T006 · Chore · P2: add doc comment warning for `Config` formatting**
    - **Context:** cr-02 Bulletproof API key redaction, Step 3
    - **Action:**
        1. Add a prominent doc comment (`///`) to the `Config` struct definition warning developers against using non-`Debug` formatters (like `Display`) due to secret leakage risk.
        2. Briefly investigate feasibility of a custom clippy lint for this; implement if simple, otherwise rely on doc comment.
    - **Done‑when:**
        1. Doc comment added to `Config` struct.
        2. Lint feasibility investigated; decision documented if not implemented.
    - **Depends‑on:** [T004]

## Test Utilities (`tests/`)
- [ ] **T007 · Refactor · P1: consolidate test config helpers into one utility returning `Result`**
    - **Context:** cr-03 Non-panicking, flexible test helpers, Step 1
    - **Action:**
        1. Identify existing test helper functions (`create_test_config_with_env` likely in multiple test modules).
        2. Create a single, shared test utility function (e.g., in `tests/common/config_helpers.rs`) that takes env var overrides.
        3. Refactor this utility to mirror production `load_config` logic, returning `Result<Config, ConfigError>` instead of panicking on missing/invalid env vars. Remove old helpers.
    - **Done‑when:**
        1. A single, shared test config helper exists and returns `Result<Config, ConfigError>`.
        2. The helper does not panic on missing/invalid required env vars based on production logic.
        3. Old helper functions are removed.
        4. Code compiles (call sites will be broken until T008).
    - **Depends‑on:** none

- [ ] **T008 · Refactor · P1: update test call sites to use consolidated helper and handle `Result`**
    - **Context:** cr-03 Non-panicking, flexible test helpers, Step 2
    - **Action:**
        1. Find all test locations that previously called the old config helpers.
        2. Update these call sites to use the new consolidated helper function (from T007).
        3. Handle the returned `Result<Config, ConfigError>`, using `.unwrap()` for tests expecting success, and matching `Err` variants for tests expecting configuration errors.
    - **Done‑when:**
        1. All relevant test call sites use the new helper and correctly handle the `Result`.
        2. All affected tests compile and pass (or are updated to reflect new error handling).
    - **Verification:**
        1. Run `cargo test`.
    - **Depends‑on:** [T007]

- [ ] **T009 · Test · P2: add negative path tests for config loading helper**
    - **Context:** cr-03 Non-panicking, flexible test helpers, Step 3
    - **Action:**
        1. Add new tests using the consolidated helper (from T007) to cover negative paths.
        2. Include test cases for: missing required keys, empty required keys, invalid values (e.g., non-numeric port, invalid boolean string).
        3. Assert that the helper returns the expected `Err(ConfigError::...)` variant in each case.
    - **Done‑when:**
        1. New tests cover common configuration error scenarios (missing/empty/invalid values).
        2. Tests assert the correct `ConfigError` variants are returned.
        3. All new tests pass.
    - **Verification:**
        1. Run `cargo test`.
    - **Depends‑on:** [T008]

- [ ] **T010 · Refactor · P1: update consolidated test helper to use `parse_bool_env`**
    - **Context:** cr-04 Strict boolean env parsing in tests, Step 1
    - **Action:**
        1. Modify the consolidated test helper (from T007) to use the production `parse_bool_env` function for parsing boolean environment variables (`LOG_BODIES`, `OPENAI_ENABLED`).
        2. Ensure errors from `parse_bool_env` are handled appropriately within the helper (propagated in the `ConfigError` result).
    - **Done‑when:**
        1. Test helper uses `parse_bool_env` for all relevant boolean fields.
        2. Parsing errors are handled correctly (returned as `Err`).
    - **Depends‑on:** [T007]

- [ ] **T011 · Test · P1: rewrite boolean parsing tests for strict rules**
    - **Context:** cr-04 Strict boolean env parsing in tests, Step 2 & 3
    - **Action:**
        1. Locate `test_boolean_parsing` and any similar tests asserting config parsing behavior for booleans.
        2. Rewrite these tests to use the consolidated helper (from T007/T010) and assert *only* the strictly allowed inputs for true/false result in success.
        3. Assert that invalid or empty inputs result in an appropriate `Err` or the documented default behavior, removing any legacy logic checks (e.g., `!= "false"`).
    - **Done‑when:**
        1. Boolean parsing tests use the updated helper.
        2. Tests assert only the strict parsing rules defined by `parse_bool_env`.
        3. Legacy boolean logic checks are removed from tests.
        4. All relevant tests pass.
    - **Verification:**
        1. Run `cargo test`.
    - **Depends‑on:** [T008, T010]

---

### Clarifications & Assumptions
- [ ] **Issue:** Assumes a production function `parse_bool_env` exists and correctly implements the desired strict boolean parsing logic (e.g., only "true"/"1" and "false"/"0", case-insensitive).
    - **Context:** cr-04, T010, T011
    - **Blocking?:** no

- [ ] **Issue:** Assumes `ConfigError` enum exists and can be extended with `NotInitialized` variant, or a suitable existing error variant can be used.
    - **Context:** cr-01, T001
    - **Blocking?:** no