# Todo

## Configuration (`src/config.rs`)
- [x] **T001 · Feature · P2: define default configuration constants**
    - **Context:** PLAN.md > Detailed Build Steps > 1. Define Configuration Constants
    - **Action:**
        1. Add `pub const` definitions for:
           - `DEFAULT_PORT`
           - `DEFAULT_ANTHROPIC_TARGET_URL`
           - `DEFAULT_LOG_STDOUT_LEVEL`
           - `DEFAULT_LOG_FILE_LEVEL`
           - `DEFAULT_LOG_FORMAT`
           - `DEFAULT_LOG_BODIES`
           - `DEFAULT_LOG_FILE_PATH`
           - `DEFAULT_LOG_MAX_BODY_SIZE`
           - `DEFAULT_LOG_DIRECTORY_MODE`
           - `DEFAULT_LOG_MAX_AGE_DAYS`
           in `src/config.rs`.
        2. Prepend each constant with a `///` doc comment explaining its rationale.
    - **Done-when:**
        1. All specified constants compile as `pub const`.
        2. Each constant has a non-empty doc comment.
    - **Depends-on:** none

- [x] **T002 · Refactor · P2: refactor `Config::default()` to use constants**
    - **Context:** PLAN.md > Detailed Build Steps > 2. Refactor `Config::default()` & `load_config()`
    - **Action:**
        1. In `src/config.rs`, update `impl Default for Config` or `Config::default()` to replace hardcoded literals with the new `DEFAULT_*` constants.
    - **Done-when:**
        1. `Config::default()` uses only the defined constants.
        2. Existing unit tests for defaults pass.
    - **Depends-on:** T001

- [x] **T003 · Refactor · P2: refactor `Config::load()` to use constants with fallback logging**
    - **Context:** PLAN.md > Detailed Build Steps > 2. Refactor `Config::default()` & `load_config()`; PLAN.md > Logging & Observability
    - **Action:**
        1. In `src/config.rs`, update `Config::load()` to call `env::var("X").unwrap_or_else(|_| DEFAULT_X.to_string())` for each variable.
        2. On parse errors (numeric/bool), log a `warn!` with `{ var, error }` and fall back to the constant.
    - **Done-when:**
        1. All environment fallbacks use `DEFAULT_*` constants.
        2. A warning is logged on parse failure before defaulting.
    - **Depends-on:** T001

- [x] **T004 · Chore · P3: add module-level docs for config.rs**
    - **Context:** PLAN.md > Detailed Build Steps > 8. Update Documentation
    - **Action:**
        1. Insert `//!` comments at the top of `src/config.rs` summarizing module purpose and default constants.
    - **Done-when:**
        1. `src/config.rs` has module-level documentation.
        2. `cargo doc` generates without warnings.
    - **Depends-on:** T001

## Filesystem Utilities (`src/fs_utils.rs`)
- [x] **T005 · Feature · P2: create fs_utils module skeleton**
    - **Context:** PLAN.md > Detailed Build Steps > 3. Create `fs_utils` Module
    - **Action:**
        1. Add new file `src/fs_utils.rs`.
        2. Declare the module in `src/lib.rs` or `src/main.rs`.
    - **Done-when:**
        1. Project compiles with empty `fs_utils` module present.
    - **Depends-on:** none

- [x] **T006 · Feature · P2: implement `ensure_directory` in fs_utils**
    - **Context:** PLAN.md > Detailed Build Steps > 3. Create `fs_utils` Module; PLAN.md > Logging & Observability; PLAN.md > Security & Config
    - **Action:**
        1. In `src/fs_utils.rs`, implement:
           ```rust
           pub fn ensure_directory(path: &Path, mode: Option<u32>) -> io::Result<()>
           ```
        2. Use `create_dir_all` and on Unix apply `PermissionsExt::from_mode(mode)` when provided; no-op on Windows.
        3. Emit `INFO { event = "create_dir", path = ?path }` on successful creation.
    - **Done-when:**
        1. Directories are created recursively.
        2. Permissions are set on Unix when `mode` is `Some`.
        3. INFO log is emitted on creation.
    - **Depends-on:** T005

- [x] **T007 · Feature · P2: implement `check_writable` in fs_utils**
    - **Context:** PLAN.md > Detailed Build Steps > 3. Create `fs_utils` Module; PLAN.md > Security & Config
    - **Action:**
        1. In `src/fs_utils.rs`, implement:
           ```rust
           pub fn check_writable(path: &Path) -> io::Result<()>
           ```
        2. Verify the path exists and is writable (metadata permissions on Unix, temp file test on Windows).
    - **Done-when:**
        1. Returns `Ok(())` for writable paths.
        2. Returns `Err(io::Error)` when not writable or missing.
    - **Depends-on:** T005

- [x] **T008 · Chore · P3: add module-level docs for fs_utils.rs**
    - **Context:** PLAN.md > Detailed Build Steps > 8. Update Documentation
    - **Action:**
        1. Insert `//!` comments at the top of `src/fs_utils.rs` describing its functions and cross-platform behavior.
    - **Done-when:**
        1. Module-level docs present and accurate.
        2. `cargo doc` generates without warnings.
    - **Depends-on:** T006, T007

## Testing fs_utils & Helper
- [ ] **T009 · Test · P2: add unit tests for `ensure_directory`**
    - **Context:** PLAN.md > Detailed Build Steps > 7. Add Unit Tests
    - **Action:**
        1. Write tests using `tempfile::TempDir` to cover:
           - Creation of non-existent directory (with/without mode).
           - No-op on existing directory.
           - Error on unwritable parent.
        2. On Unix, verify permission bits.
    - **Done-when:**
        1. Tests cover success, existing, and error cases.
        2. CI passes on all platforms.
    - **Depends-on:** T006

- [ ] **T010 · Test · P2: add unit tests for `check_writable`**
    - **Context:** PLAN.md > Detailed Build Steps > 7. Add Unit Tests
    - **Action:**
        1. Write tests using `tempfile::TempDir` to cover:
           - Writable directory.
           - Non-writable directory (simulate if possible).
           - Non-existent path.
    - **Done-when:**
        1. Tests cover all scenarios.
        2. CI passes on all platforms.
    - **Depends-on:** T007

## Logging Integration (`src/logger.rs`)
- [ ] **T011 · Feature · P2: define `LogInitError` enum**
    - **Context:** PLAN.md > Architecture Blueprint > Public Interfaces / Contracts
    - **Action:**
        1. In `src/logger.rs`, add:
           ```rust
           pub enum LogInitError {
               DirectoryCreationFailed(io::Error),
               PermissionDenied(io::Error),
               // …
           }
           ```
        2. Implement `std::error::Error` and `Display`.
    - **Done-when:**
        1. `LogInitError` compiles with variants and trait impls.
    - **Depends-on:** none

- [ ] **T012 · Refactor · P2: use `ensure_directory` in `LogPathResolver::resolve`**
    - **Context:** PLAN.md > Detailed Build Steps > 4. Refactor `src/logger.rs`
    - **Action:**
        1. Remove inline `create_dir_all` and `set_permissions` from `resolve()`.
        2. Call `fs_utils::ensure_directory(&path, Some(cfg.log_directory_mode))`.
    - **Done-when:**
        1. `LogPathResolver::resolve()` calls `ensure_directory`.
        2. Inline filesystem code is removed.
    - **Depends-on:** T006

- [ ] **T013 · Refactor · P2: map `io::Error` to `LogInitError` in `resolve`**
    - **Context:** PLAN.md > Detailed Build Steps > 4. Refactor `src/logger.rs`; PLAN.md > Logging & Observability
    - **Action:**
        1. Change return type of `resolve()` to `Result<PathBuf, LogInitError>`.
        2. Map `io::Error` from `ensure_directory` to appropriate `LogInitError` variant and log `ERROR` on failure.
    - **Done-when:**
        1. Errors from `ensure_directory` map to `LogInitError`.
        2. An `ERROR` log is emitted before return.
    - **Depends-on:** T011, T012

## Test Helper (`tests/common/mod.rs`)
- [ ] **T014 · Feature · P2: implement `verify_directory_permissions` helper**
    - **Context:** PLAN.md > Detailed Build Steps > 5. Implement Test Helper
    - **Action:**
        1. In `tests/common/mod.rs`, add:
           ```rust
           #[cfg(target_family = "unix")]
           pub fn verify_directory_permissions(path: &Path, required_mode: u32) -> Result<(), String> { … }
           #[cfg(not(target_family = "unix"))]
           pub fn verify_directory_permissions(_path: &Path, _required_mode: u32) -> Result<(), String> { Ok(()) }
           ```
        2. Implement Unix logic using `metadata().permissions().mode()` and compare bits.
    - **Done-when:**
        1. Helper compiles and returns `Ok`/`Err(String)` appropriately.
    - **Depends-on:** none

- [ ] **T015 · Test · P2: add unit tests for `verify_directory_permissions`**
    - **Context:** PLAN.md > Detailed Build Steps > 7. Add Unit Tests
    - **Action:**
        1. Write tests using `tempfile::TempDir` and `fs::set_permissions` to:
           - Exercise matching mode (expect `Ok`).
           - Mismatched mode (expect `Err(String)` with descriptive message).
    - **Done-when:**
        1. Tests cover both pass and fail scenarios on Unix.
    - **Depends-on:** T014

- [ ] **T016 · Refactor · P2: update existing tests to use `verify_directory_permissions`**
    - **Context:** PLAN.md > Detailed Build Steps > 6. Refactor Tests
    - **Action:**
        1. Search all tests for manual permission checks or `set_permissions` calls.
        2. Replace with `common::verify_directory_permissions(&path, DEFAULT_LOG_DIRECTORY_MODE).expect("…")`.
        3. Remove duplicated filesystem setup.
    - **Done-when:**
        1. No manual permission logic remains in tests.
        2. All tests pass.
    - **Depends-on:** T001, T014

## Documentation & CI
- [ ] **T017 · Chore · P3: update README logging table with constants**
    - **Context:** PLAN.md > Detailed Build Steps > 8. Update Documentation
    - **Action:**
        1. In `README.md`, replace hardcoded default values in the logging section with references to `src/config.rs` constants.
        2. Note that defaults are centralized.
    - **Done-when:**
        1. README values match constants.
    - **Depends-on:** T001

- [ ] **T018 · Chore · P2: enforce formatting, linting, and cross-platform tests in CI**
    - **Context:** PLAN.md > Detailed Build Steps > 9. Quality Gates
    - **Action:**
        1. Update CI config to run:
           - `cargo fmt -- --check`
           - `cargo clippy -- -D warnings`
           - `cargo test -- --all-targets`
           on Linux, macOS, and Windows.
    - **Done-when:**
        1. CI fails on format or lint warnings.
        2. Tests run on all platforms.
    - **Depends-on:** none

- [ ] **T019 · Chore · P3: add CI lint to detect hardcoded default literals**
    - **Context:** PLAN.md > Risk Matrix: Incomplete sweep of hardcoded literals
    - **Action:**
        1. Introduce a CI step (script or `grep`) that fails if default literal patterns (e.g., `"INFO"`, numeric defaults, `"0o750"`) appear outside `src/config.rs`.
    - **Done-when:**
        1. CI step flags new hardcoded literals outside config.
    - **Depends-on:** T001

- [ ] **T020 · Test · P1: verify all tests pass across CI platforms**
    - **Context:** PLAN.md > Detailed Build Steps > 9. Quality Gates; PLAN.md > Testing Strategy
    - **Action:**
        1. Run CI pipeline on PR.
        2. Confirm `cargo test` passes on Linux, macOS, and Windows.
        3. Ensure coverage thresholds are met.
    - **Done-when:**
        1. All CI jobs succeed.
    - **Depends-on:** T018, T019, T016

### Clarifications & Assumptions
- [ ] **Issue:** should `fs_utils::ensure_directory` reject symlink targets outside the intended base directory?
    - **Context:** PLAN.md > Open Questions
    - **Blocking?:** no
- [ ] **Issue:** should fs_utils functions be public (`pub`) or crate-internal (`pub(crate)`)?
    - **Context:** PLAN.md > Open Questions
    - **Blocking?:** no
- [ ] **Issue:** what additional path validation should `fs_utils` perform beyond non-empty/non-UTF8 checks (e.g., `..` components)?
    - **Context:** PLAN.md > Security & Config > Input Validation
    - **Blocking?:** no
- [ ] **Issue:** should CI lint for hardcoded default literals be part of this work or deferred?
    - **Context:** PLAN.md > Open Questions
    - **Blocking?:** no