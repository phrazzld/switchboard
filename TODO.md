# Todo

## OpenAI Foundation

- [x] **T001 · chore · P2: add async-openai crate dependency**
    - **Context:** PLAN-1.md, 4.3. Add Dependency
    - **Action:**
        1. Add `async-openai = "<latest_compatible_version>"` to `[dependencies]` in `Cargo.toml`. (Confirm version constraint - see Clarifications)
        2. Run `cargo build` or `cargo update` to fetch the dependency and update `Cargo.lock`.
        3. Verify build completes successfully.
    - **Done‑when:**
        1. `async-openai` is listed as a dependency in `Cargo.toml`.
        2. `Cargo.lock` is updated to include `async-openai`.
        3. Project builds successfully using `cargo build`.
    - **Verification:** none
    - **Depends‑on:** none

- [x] **T002 · chore · P2: create empty openai adapter module file**
    - **Context:** PLAN-1.md, 4.4. Create Placeholder Module
    - **Action:**
        1. Create the file `src/openai_adapter.rs`.
        2. Add `mod openai_adapter;` declaration in `src/lib.rs` or `src/main.rs` as appropriate.
    - **Done‑when:**
        1. The file `src/openai_adapter.rs` exists.
        2. The project compiles successfully (`cargo build`).
    - **Verification:** none
    - **Depends‑on:** none

- [x] **T003 · feature · P2: add openai config fields to config struct**
    - **Context:** PLAN-1.md, 4.1. Add Config Fields
    - **Action:**
        1. Add `openai_api_key: Option<String>`, `openai_api_base_url: String`, and `openai_enabled: bool` fields to the `Config` struct in `src/config.rs`.
        2. Define constants or similar for the environment variable names (`OPENAI_API_KEY`, `OPENAI_API_BASE_URL`, `OPENAI_ENABLED`).
    - **Done‑when:**
        1. `Config` struct in `src/config.rs` includes the new fields.
        2. Environment variable names are clearly associated (e.g., via constants or comments).
        3. Code compiles successfully.
    - **Verification:** none
    - **Depends‑on:** none

- [x] **T004 · feature · P2: implement loading openai config from env vars with defaults**
    - **Context:** PLAN-1.md, 4.2. Update Config Loading (loading & defaults)
    - **Action:**
        1. Modify the config loading function (`load_config` or similar) in `src/config.rs` to read `OPENAI_API_KEY`, `OPENAI_API_BASE_URL`, and `OPENAI_ENABLED` environment variables.
        2. Apply default values: `openai_api_base_url` defaults to `https://api.openai.com`, `openai_enabled` defaults to `false` if the corresponding env vars are not set.
    - **Done‑when:**
        1. Config loading logic correctly reads the new environment variables.
        2. Default values are applied correctly when environment variables are absent.
        3. Unit tests (T008) pass.
    - **Verification:** none
    - **Depends‑on:** [T003]

- [x] **T005 · feature · P2: add validation for openai api key if enabled**
    - **Context:** PLAN-1.md, 4.2. Update Config Loading (validation)
    - **Action:**
        1. Add validation logic within the config loading function in `src/config.rs`.
        2. If `openai_enabled` resolves to `true`, check if `openai_api_key` has a value (`is_some()`).
        3. Return a distinct error if `openai_enabled` is `true` but `openai_api_key` is `None`.
    - **Done‑when:**
        1. Config loading returns an error specifically indicating the missing key when `OPENAI_ENABLED=true` is set but `OPENAI_API_KEY` is missing.
        2. Config loading succeeds when `OPENAI_ENABLED=false` or when both `OPENAI_ENABLED=true` and `OPENAI_API_KEY` are present.
        3. Unit tests (T009) pass.
    - **Verification:** none
    - **Depends‑on:** [T004]

- [x] **T006 · feature · P2: add logging for loaded openai config (excluding key)**
    - **Context:** PLAN-1.md, 4.2. Update Config Loading (logging)
    - **Action:**
        1. In the config loading logic (after successful load and validation), add logging statements (INFO or DEBUG level).
        2. Log the resolved values of `openai_api_base_url` and `openai_enabled` if `openai_enabled` is true.
        3. Ensure the `openai_api_key` value is never logged.
    - **Done‑when:**
        1. Startup logs include the OpenAI base URL and enabled status when `OPENAI_ENABLED=true`.
        2. Startup logs do not show OpenAI specific config if `OPENAI_ENABLED=false`.
        3. The OpenAI API key is confirmed absent from all logs.
    - **Verification:**
        1. Run the application with `OPENAI_ENABLED=true` and `OPENAI_API_KEY=testkey`; check logs for base URL and `enabled=true`.
        2. Run the application with `OPENAI_ENABLED=false`; check logs confirm no OpenAI config logged or indicate disabled status.
        3. Search application logs for `testkey` (or the actual key used); verify it's not found.
    - **Depends‑on:** [T004]

- [x] **T007 · chore · P2: document openai env vars in readme**
    - **Context:** PLAN-1.md, 4.5. Update Documentation
    - **Action:**
        1. Add a subsection for OpenAI under the "Environment Variables" section in `README.md`.
        2. Document `OPENAI_API_KEY`, `OPENAI_API_BASE_URL`, and `OPENAI_ENABLED`.
        3. Include their purpose, default values (`openai_api_base_url`, `openai_enabled`), and the requirement for `OPENAI_API_KEY` if `OPENAI_ENABLED` is true.
    - **Done‑when:**
        1. `README.md` contains accurate documentation for the three new environment variables.
    - **Verification:**
        1. Review the rendered `README.md` for clarity and correctness.
    - **Depends‑on:** none

- [ ] **T008 · test · P2: add unit tests for openai config loading and defaults**
    - **Context:** PLAN-1.md, 4.6. Add Unit Tests (loading & defaults)
    - **Action:**
        1. Add unit tests in the config test module (e.g., `src/config.rs` tests or `tests/config_tests.rs`).
        2. Test correct loading of `OPENAI_API_KEY`, `OPENAI_API_BASE_URL`, `OPENAI_ENABLED` from mocked environment variables.
        3. Test correct application of default values for `openai_api_base_url` and `openai_enabled` when env vars are unset.
    - **Done‑when:**
        1. New unit tests covering OpenAI config loading and defaults exist.
        2. All tests pass via `cargo test`.
    - **Verification:** Run `cargo test`.
    - **Depends‑on:** [T004]

- [ ] **T009 · test · P2: add unit tests for openai config validation**
    - **Context:** PLAN-1.md, 4.6. Add Unit Tests (validation)
    - **Action:**
        1. Add unit tests in the config test module.
        2. Test that config loading returns an error when `OPENAI_ENABLED` is mocked to true but `OPENAI_API_KEY` is unset.
        3. Test that config loading succeeds when `OPENAI_ENABLED` is false (key unset) and when `OPENAI_ENABLED` is true and key is set.
    - **Done‑when:**
        1. New unit tests covering OpenAI API key validation logic exist.
        2. All tests pass via `cargo test`.
    - **Verification:** Run `cargo test`.
    - **Depends‑on:** [T005]

- [ ] **T010 · chore · P2: run final checks (fmt, clippy, test)**
    - **Context:** PLAN-1.md, 4.7. Review & Refactor
    - **Action:**
        1. Run `cargo fmt --check` to verify formatting.
        2. Run `cargo clippy -- -D warnings` to check for lints.
        3. Run `cargo test` to ensure all tests pass.
        4. Address any issues reported by the above commands.
    - **Done‑when:**
        1. `cargo fmt --check` passes.
        2. `cargo clippy -- -D warnings` passes.
        3. `cargo test` passes.
    - **Verification:** none
    - **Depends‑on:** [T001, T002, T005, T006, T007, T008, T009]

### Clarifications & Assumptions
- [ ] **Issue:** Confirm specific version or compatibility range for `async-openai` crate.
    - **Context:** PLAN-1.md § 4.3 (T001)
    - **Blocking?:** yes
- [ ] **Issue:** Confirm preferred location for config unit tests (`src/config.rs` inline or `tests/config_tests.rs`).
    - **Context:** PLAN-1.md § 4.6 (T008, T009)
    - **Blocking?:** no