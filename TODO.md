# Todo

## Configuration (API Key Redaction - cr-02)
- [x] **T001 · Chore · P0: add secrecy dependency**
    - **Context:** PLAN.md > cr-02 > Steps > 1
    - **Action:**
        1. Add `secrecy = "1.0"` (or latest compatible) to `[dependencies]` in `Cargo.toml`.
    - **Done‑when:**
        1. Dependency added to `Cargo.toml`.
        2. `cargo build` completes successfully.
    - **Depends‑on:** none
- [ ] **T002 · Refactor · P0: modify config struct fields to use secret types**
    - **Context:** PLAN.md > cr-02 > Steps > 2
    - **Action:**
        1. Import `Secret` from `secrecy` in `src/config.rs`.
        2. Change `Config` field `anthropic_api_key: String` to `anthropic_api_key: Secret<String>`.
        3. Change `Config` field `openai_api_key: Option<String>` to `openai_api_key: Option<Secret<String>>`.
    - **Done‑when:**
        1. `Config` struct definition uses `Secret<String>` for API key fields.
        2. `cargo check` passes (may have downstream errors until dependent tasks complete).
    - **Depends‑on:** [T001]
- [ ] **T003 · Refactor · P0: update load_config to wrap loaded api keys in secret**
    - **Context:** PLAN.md > cr-02 > Steps > 3
    - **Action:**
        1. Modify the config loading function (`load_config` or its replacement post-T007) to wrap retrieved API key strings using `Secret::new()` before assigning to `Config` fields.
    - **Done‑when:**
        1. `load_config` correctly wraps retrieved API keys in `Secret` types.
        2. `cargo check` passes for `load_config`.
    - **Depends‑on:** [T002, T009]
- [ ] **T004 · Refactor · P0: update api key usage sites to call expose_secret**
    - **Context:** PLAN.md > cr-02 > Steps > 4
    - **Action:**
        1. Find all code locations (e.g., HTTP client setup) accessing `anthropic_api_key` or `openai_api_key`.
        2. Update these locations to call `.expose_secret()` on the `Secret` values.
    - **Done‑when:**
        1. All necessary usages of API keys correctly use `.expose_secret()`.
        2. Code compiles (`cargo check`).
    - **Depends‑on:** [T002]
- [ ] **T005 · Refactor · P0: update test/benchmark config instantiations for secrecy**
    - **Context:** PLAN.md > cr-02 > Steps > 5
    - **Action:**
        1. Update `Config` struct instantiations in `tests/common/mod.rs`, `tests/config_test.rs`, and `benches/*.rs`.
        2. Wrap dummy API key strings with `Secret::new()` in these instantiations.
    - **Done‑when:**
        1. Test and benchmark code correctly instantiates `Config` with `Secret` types.
        2. `cargo test --no-run` and `cargo bench --no-run` compile.
    - **Depends‑on:** [T002]
- [ ] **T006 · Test · P0: add unit test verifying config debug redaction**
    - **Context:** PLAN.md > cr-02 > Steps > 6
    - **Action:**
        1. Add a unit test (e.g., in `src/config.rs` or `tests/config_test.rs`).
        2. Create a `Config` with dummy keys wrapped in `Secret::new()`.
        3. Assert `format!("{:?}", config)` output contains `"[REDACTED]"` and *not* the dummy key values.
    - **Done‑when:**
        1. Unit test verifies `Debug` formatting redacts secret fields.
        2. `cargo test` passes for this test.
    - **Verification:**
        1. Run `cargo test -- --show-output` and visually inspect logs for any leaked dummy keys from Config debug prints.
        2. Run `cargo run` (if applicable) with debug logging and API keys set; inspect logs for redaction.
        3. Grep test/debug log output for dummy key values; ensure none are found.
    - **Depends‑on:** [T002, T005]

## Configuration (Boolean Parsing - cr-04)
- [ ] **T007 · Refactor · P2: define parse_bool_env helper function**
    - **Context:** PLAN.md > cr-04 > Steps > 1, 2
    - **Action:**
        1. Define `fn parse_bool_env(var_name: &str, default: bool) -> bool` in `src/config.rs`.
        2. Implement logic: read env var, normalize case, parse "true"/"1" -> true, "false"/"0" -> false, log warning via `tracing::warn!` and return `default` otherwise/unset.
    - **Done‑when:**
        1. Helper function `parse_bool_env` defined and implemented correctly.
        2. `cargo check` passes.
    - **Depends‑on:** none
- [ ] **T008 · Refactor · P2: integrate parse_bool_env into load_config**
    - **Context:** PLAN.md > cr-04 > Steps > 3
    - **Action:**
        1. Replace existing parsing logic for `LOG_BODIES` and `OPENAI_ENABLED` in `load_config` (or its replacement post-T009) with calls to `parse_bool_env`.
    - **Done‑when:**
        1. `load_config` uses `parse_bool_env` for boolean environment variables.
        2. `cargo check` passes.
    - **Verification:**
        1. Run application with an invalid boolean (e.g., `LOG_BODIES=maybe`).
        2. Verify a warning log appears (e.g., `Invalid value for environment variable LOG_BODIES: 'maybe'. Using default value.`) and the default is used.
    - **Depends‑on:** [T007, T009]
- [ ] **T009 · Test · P2: update test_boolean_parsing to assert strict behavior**
    - **Context:** PLAN.md > cr-04 > Steps > 4
    - **Action:**
        1. Modify `test_boolean_parsing` test in `tests/config_test.rs`.
        2. Assert only "true"/"1" (case-insensitive) parse as `true`.
        3. Assert only "false"/"0" (case-insensitive) parse as `false`.
        4. Assert other values/unset result in the default value.
    - **Done‑when:**
        1. Test `test_boolean_parsing` validates strict parsing logic and default fallback.
        2. `cargo test` passes.
    - **Depends‑on:** [T008]

## Configuration (Error Handling - cr-01)
- [ ] **T010 · Chore · P1: add thiserror dependency**
    - **Context:** PLAN.md > cr-01 > Steps > 1
    - **Action:**
        1. Add `thiserror = "1.0"` (or latest compatible) to `[dependencies]` in `Cargo.toml`.
    - **Done‑when:**
        1. Dependency added to `Cargo.toml`.
        2. `cargo build` completes successfully.
    - **Depends‑on:** none
- [ ] **T011 · Refactor · P1: define configerror enum using thiserror**
    - **Context:** PLAN.md > cr-01 > Steps > 2
    - **Action:**
        1. Define `pub enum ConfigError` in `src/config.rs` using `#[derive(Debug, thiserror::Error)]`.
        2. Add variants like `MissingOpenApiKey`, `MissingRequiredKey(&'static str)`, `InvalidBooleanValue { var: String, value: String }` with `#[error(...)]` attributes.
    - **Done‑when:**
        1. `ConfigError` enum defined with required variants and derives.
        2. Code compiles (`cargo check`).
    - **Depends‑on:** [T010]
- [ ] **T012 · Refactor · P1: change load_config signature to return result<config, configerror>**
    - **Context:** PLAN.md > cr-01 > Steps > 3
    - **Action:**
        1. Modify `load_config` signature in `src/config.rs` to `pub fn load_config() -> Result<Config, ConfigError>`.
        2. Address clarification `C001` regarding static initialization strategy *before* merging this change, or adapt this task based on the resolution.
    - **Done‑when:**
        1. `load_config` signature updated.
        2. Code compiles (`cargo check`), potentially with temporary `unwrap()`/`expect()` at call sites.
    - **Depends‑on:** [T011, C001]
- [ ] **T013 · Refactor · P1: replace panics/expects in load_config with err returns**
    - **Context:** PLAN.md > cr-01 > Steps > 4
    - **Action:**
        1. Identify `panic!`, `expect`, `unwrap` calls related to config loading/validation in `load_config`.
        2. Replace them with appropriate `return Err(ConfigError::...)` variants.
        3. Use `?` operator for fallible operations like `env::var`.
    - **Done‑when:**
        1. `load_config` no longer panics on configuration errors.
        2. Returns `Err(ConfigError::...)` for invalid/missing configuration.
        3. `cargo check` passes.
    - **Depends‑on:** [T012]
- [ ] **T014 · Refactor · P1: update main.rs to handle load_config result gracefully**
    - **Context:** PLAN.md > cr-01 > Steps > 5
    - **Action:**
        1. Modify `src/main.rs` where `load_config` (or its static accessor) is called.
        2. Match on the `Result<Config, ConfigError>`.
        3. On `Ok(config)`, proceed with application startup.
        4. On `Err(e)`, log the specific error via `tracing::error!` and call `std::process::exit(1)`.
    - **Done‑when:**
        1. `main.rs` handles both `Ok` and `Err` from config loading.
        2. Application exits with status 1 on configuration error, logging the specific error.
        3. `cargo run` with invalid config exits gracefully (no panic trace).
    - **Verification:**
        1. Run application with `OPENAI_ENABLED=true` and `OPENAI_API_KEY` unset.
        2. Verify specific error log message (e.g., "OpenAI integration enabled but OPENAI_API_KEY environment variable is not set") and exit code 1.
    - **Depends‑on:** [T013]
- [ ] **T015 · Test · P1: update tests for config loading result/error variants**
    - **Context:** PLAN.md > cr-01 > Steps > 6
    - **Action:**
        1. Review tests in `tests/config_test.rs` and others relying on config loading.
        2. Update tests expecting panics to assert `Err` results with specific `ConfigError` variants.
        3. Add tests covering different `ConfigError` scenarios.
    - **Done‑when:**
        1. Tests accurately reflect `Result`-returning behavior of `load_config`.
        2. Tests cover specific `ConfigError` variants.
        3. `cargo test` passes.
    - **Depends‑on:** [T013]

## Testing (Isolation - cr-03)
- [ ] **T016 · Chore · P2: add serial_test dev-dependency**
    - **Context:** PLAN.md > cr-03 > Steps > 1
    - **Action:**
        1. Add `serial_test = "2.0"` (or latest compatible) to `[dev-dependencies]` in `Cargo.toml`.
    - **Done‑when:**
        1. Dependency added to `Cargo.toml`.
        2. `cargo build --tests` completes successfully.
    - **Depends‑on:** none
- [ ] **T017 · Test · P2: apply #[serial] attribute to env-modifying config tests**
    - **Context:** PLAN.md > cr-03 > Steps > 2, 3, 4
    - **Action:**
        1. Add `use serial_test::serial;` to `tests/config_test.rs`.
        2. Identify `#[test]` functions in `tests/config_test.rs` modifying environment variables.
        3. Add `#[serial]` attribute above `#[test]` for each identified function.
    - **Done‑when:**
        1. `#[serial]` attribute applied to relevant tests in `config_test.rs`.
        2. `use` statement added.
        3. `cargo test -- --test-threads=N` (N>1) runs tests without env var conflicts.
    - **Depends‑on:** [T016]
- [ ] **T018 · Refactor · P2: remove env_mutex and usage from config_test.rs**
    - **Context:** PLAN.md > cr-03 > Steps > 5
    - **Action:**
        1. Remove `static ENV_MUTEX: Mutex<()>` definition from `tests/config_test.rs`.
        2. Remove all associated `.lock()` calls.
    - **Done‑when:**
        1. `ENV_MUTEX` and its usage are completely removed.
        2. `cargo test` passes.
    - **Depends‑on:** [T017]
- [ ] **T019 · Chore · P3: add comment explaining #[serial] usage in config_test.rs**
    - **Context:** PLAN.md > cr-03 > Steps > 6
    - **Action:**
        1. Add a comment at the top of `tests/config_test.rs` explaining why `#[serial]` is used for env-modifying tests.
    - **Done‑when:**
        1. Explanatory comment is present.
    - **Depends‑on:** [T017]

## Documentation (README - cr-05)
- [ ] **T020 · Chore · P2: add warning note about openai status to readme features/overview**
    - **Context:** PLAN.md > cr-05 > Steps > 1, 2
    - **Action:**
        1. Edit `README.md`.
        2. Add the specified warning note about OpenAI foundational status prominently in the "Features" or "Overview" section.
    - **Done‑when:**
        1. `README.md` contains the warning note in the main feature description section.
    - **Verification:**
        1. Review the rendered `README.md`; confirm note is clear and visible.
    - **Depends‑on:** none
- [ ] **T021 · Chore · P2: add/reference warning note in readme configuration section**
    - **Context:** PLAN.md > cr-05 > Steps > 3
    - **Action:**
        1. Edit `README.md`.
        2. Add or reference the warning note from T020 within the "Configuration" or "Environment Variables" section near `OPENAI_API_KEY` / `OPENAI_ENABLED`.
    - **Done‑when:**
        1. `README.md` contains the warning note (or reference) in the configuration section.
    - **Verification:**
        1. Review the rendered `README.md`; confirm note is clear in config context.
    - **Depends‑on:** [T020]
- [ ] **T022 · Chore · P2: remove/comment misleading openai usage examples from readme**
    - **Context:** PLAN.md > cr-05 > Steps > 4
    - **Action:**
        1. Edit `README.md`.
        2. Remove/comment out examples (e.g., `curl`) demonstrating sending requests *through* the service *to* OpenAI.
        3. Clarify examples involving OpenAI config vars are for setup only.
    - **Done‑when:**
        1. `README.md` no longer contains misleading OpenAI usage examples.
    - **Verification:**
        1. Review the rendered `README.md`; confirm no misleading usage examples exist.
    - **Depends‑on:** [T020]

### Clarifications & Assumptions
- [ ] **C001 · Issue:** Determine strategy for handling `Result<Config, ConfigError>` with potential static config initialization.
    - **Context:** PLAN.md > cr-01 > Steps > 3 (note about adjusting static init)
    - **Blocking?:** yes (Blocks T012 implementation detail)