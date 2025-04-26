# plan.md

# Centralize Logging Configuration and Directory Permission Refactor

## Chosen Approach (One-liner)
Extract every hardcoded logging default into documented constants in `src/config.rs`, refactor `src/logger.rs` to consume those constants, and centralize directory‐creation and permission‐checking logic into a new `fs_utils` module (used by both production and tests) to eliminate duplication and ensure consistency.

## Architecture Blueprint

- Modules / Packages  
  - `src/config.rs`  
    • Defines all default values as `pub const` with doc comments explaining the rationale.  
    • Houses `Config` struct, `Config::default()`, and `load_config()` using those constants.  
  - `src/logger.rs`  
    • Initializes the tracing subscriber.  
    • Uses a `LogPathResolver` that receives a `Config` and resolves file paths.  
    • Delegates directory creation/permission checks to `fs_utils`.  
  - `src/fs_utils.rs` (new)  
    • `ensure_directory(path: &Path, mode: Option<u32>) -> Result<(), io::Error>`  
    • `check_writable(path: &Path) -> Result<(), io::Error>`  
    • Centralizes all filesystem operations and permission logic.  
  - `tests/common/mod.rs`  
    • Test helpers for permission assertions:  
      - `verify_directory_permissions(path: &Path, required_mode: u32) -> Result<(), String>`  
    • Shared config/test-config creation utilities.  

- Public Interfaces / Contracts  
  ```rust
  // src/config.rs
  pub const DEFAULT_PORT: &str;
  pub const DEFAULT_ANTHROPIC_TARGET_URL: &str;
  pub const DEFAULT_LOG_STDOUT_LEVEL: &str;
  pub const DEFAULT_LOG_FILE_LEVEL: &str;
  pub const DEFAULT_LOG_FORMAT: &str;
  pub const DEFAULT_LOG_BODIES: bool;
  pub const DEFAULT_LOG_FILE_PATH: &str;
  pub const DEFAULT_LOG_MAX_BODY_SIZE: usize;
  pub const DEFAULT_LOG_DIRECTORY_MODE: u32;        // e.g., 0o750
  pub const DEFAULT_LOG_MAX_AGE_DAYS: Option<u32>;

  pub struct Config { /* fields populated from env or defaults */ }
  impl Config {
      pub fn default() -> Self;
      pub fn load() -> Self;
  }

  // src/logger.rs
  pub struct LogPathResolver { /* holds base_dir, subdir, filename */ }
  impl LogPathResolver {
      pub fn new(cfg: &Config, log_type: LogType) -> Self;
      pub fn resolve(&self) -> Result<PathBuf, LogInitError>;
  }
  pub enum LogInitError { DirectoryCreationFailed(io::Error), PermissionDenied(io::Error), /*…*/ }

  // src/fs_utils.rs
  pub fn ensure_directory(path: &Path, mode: Option<u32>) -> Result<(), io::Error>;
  pub fn check_writable(path: &Path) -> Result<(), io::Error>;

  // tests/common/mod.rs
  #[cfg(target_family = "unix")]
  pub fn verify_directory_permissions(path: &Path, required_mode: u32) -> Result<(), String>;
  #[cfg(not(target_family = "unix"))]
  pub fn verify_directory_permissions(_path: &Path, _required_mode: u32) -> Result<(), String> { Ok(()) }
  ```

- Data Flow Diagram (mermaid)
  ```mermaid
  graph TD
    A[Application Start] --> B(load_config)
    B --> C(Config)
    C --> D(init_tracing)
    D --> E(LogPathResolver)
    E --> F(fs_utils.ensure_directory)
    F --> G[Filesystem]
    G --> F
    F --> E
    E --> D
    D --> H[Tracing Subscriber Active]

    subgraph Tests
      T1[Test Setup] --> T2(common::TestConfig)
      T2 --> T3(LogPathResolver with test Config)
      T3 --> T4(fs_utils.ensure_directory)
      T4 --> G
      T1 --> T5(common::verify_directory_permissions)
      T5 --> G
    end
  ```

- Error & Edge-Case Strategy  
  • All path creation and permission‐setting funnels through `fs_utils`, returning `io::Error` on failure.  
  • `LogInitError` maps `fs_utils` errors to named variants.  
  • Config parsing falls back to defaults on missing or unparsable env vars, logging a warning.  
  • Filenames missing in `log_file_path` default to `switchboard.log`.  
  • Edge cases—non‐UTF8 paths, symlinks, permission boundaries—are caught in `fs_utils`.  

## Detailed Build Steps

1. **Define Configuration Constants**  
   - In `src/config.rs`, add `pub const` for each default (port, URLs, levels, format, bodies, file path, max body size, dir mode, max age).  
   - Above each, write `///` doc comments citing reasons (e.g., “INFO stdout for minimal noise in CI”; “20MB max body to avoid memory blowup”).

2. **Refactor `Config::default()` & `load_config()`**  
   - Replace all hardcoded literals with the new constants.  
   - In `load_config()`, use `env::var("X").unwrap_or_else(|_| DEFAULT_*.to_string())`. On parse errors (numeric/bool), log `warn!` and use default.

3. **Create `fs_utils` Module**  
   - New file `src/fs_utils.rs`: implement  
     ```rust
     pub fn ensure_directory(path: &Path, mode: Option<u32>) -> io::Result<()> { … }
     pub fn check_writable(path: &Path) -> io::Result<()> { … }
     ```  
   - Set `mode` on Unix via `PermissionsExt::from_mode`. On Windows, no-op or use ACL defaults.

4. **Refactor `src/logger.rs`**  
   - Remove inline `create_dir_all`, `set_permissions` calls.  
   - Inject `fs_utils::ensure_directory` in `LogPathResolver::resolve()`.  
   - Map any `io::Error` into `LogInitError` variants.

5. **Implement Test Helper in `tests/common/mod.rs`**  
   - Write `verify_directory_permissions(path, required_mode)` with Unix‐only logic using `metadata().permissions().mode()`.  
   - Return `Err(String)` with `"expected 0o750, got 0o755"` style messages.

6. **Refactor Tests**  
   - Search all tests for custom permission‐checking code.  
   - Replace with `common::verify_directory_permissions(&path, DEFAULT_LOG_DIRECTORY_MODE).expect("…")`.  
   - Remove duplicated fs calls.

7. **Add Unit Tests for `fs_utils` and Helper**  
   - Use `tempfile::TempDir` to create dirs with custom modes.  
   - Test `ensure_directory` handles existing, missing, and permission errors.  
   - Test `verify_directory_permissions` passes and fails correctly.

8. **Update Documentation**  
   - Add module‐level docs in `config.rs` and `fs_utils.rs` summarizing the design.  
   - Update `README.md`’s logging table to reference new constants.

9. **Quality Gates**  
   - Run `cargo fmt`.  
   - Run `cargo clippy -- -D warnings`.  
   - Execute `cargo test` on Linux/macOS/Windows in CI.

## Testing Strategy

- Unit Tests  
  • `config` constants correctness via `Config::default()`.  
  • `fs_utils::ensure_directory` and `check_writable` errors & success.  
  • `common::verify_directory_permissions` on Unix modes.

- Integration Tests  
  • Logger initialization end-to-end: directory creation, log file writing, subscriber guard returned.  
  • Tests unchanged except for using the helper—verify no regressions.

- Mocking  
  • None for internal functions.  
  • Simulate permission errors by creating a read-only parent directory.

- Coverage  
  • 100% on new utility and helper modules.  
  • No regression on existing coverage thresholds.

## Logging & Observability

- Emit `INFO` when a log directory is created:  
  `{ event = "create_dir", path = ?path }`.  
- Emit `WARN` if parsing an env var fails, with `{ var, error }`.  
- Emit `ERROR` on directory or permission failure before application exit.  
- Continue standard structured logging: timestamp, level, module, message, error details.

## Security & Config

- Input Validation  
  • Paths validated in `fs_utils` against traversal (reject `".."` components outside base).  
  • Non-UTF8 paths converted via `OsString` safely.

- Secrets Handling  
  • API keys not logged.  
  • No config values printed at DEBUG level.

- Least Privilege  
  • Directory mode default `0o750` limits “others”.  
  • Tests only escalate to `0o755` when explicitly required.

## Documentation

- Code Self-Doc  
  • Each constant has a rationale.  
  • `fs_utils` functions documented with parameters, behavior, and cross-platform notes.

- README Updates  
  • Sync default values table with constants.  
  • Note centralization of defaults and new `fs_utils` responsibilities.

## Risk Matrix

| Risk                                                          | Severity | Mitigation                                                          |
|---------------------------------------------------------------|----------|---------------------------------------------------------------------|
| Changing observable defaults (port/levels/paths) accidentally | medium   | Code review against `README.md`; tests assert constants in code; CI re-runs integration tests. |
| Incomplete sweep of hardcoded literals                        | medium   | `grep` audit for string patterns; clippy lint for forbidden literals. |
| Platform‐specific fs behavior (Windows ACLs)                  | medium   | CI matrix on Win/Linux/macOS; conditional code with cfgs; unit tests per platform. |
| Permission helper false negatives due to bitmask errors       | low      | Unit tests with all relevant modes; review expected masks.          |
| Conflicts with ongoing logging-cleanup work                   | low      | Coordinate via PR tags; isolate module-level changes; small commits. |

## Open Questions

- Should `fs_utils::ensure_directory` reject symlink targets outside the intended base?  
- Expose `fs_utils` publicly for downstream crates or keep `pub(crate)`?  
- Enforce a CI lint that flags any new hardcoded default literal?