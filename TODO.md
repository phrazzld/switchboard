# Todo

## Log Directory Structure

- [x] **T001 · Feature · P2: add log directory constants**
    - **Context:** PLAN.md § 1.1 Create Log Directory Constants
    - **Action:**
        1. Define `DEFAULT_LOG_DIR`, `APP_LOG_SUBDIR`, `TEST_LOG_SUBDIR` constants in `logger.rs`
    - **Done-when:**
        1. Constants defined and accessible
        2. Code compiles successfully
    - **Depends-on:** none

- [x] **T002 · Refactor · P2: update `validate_log_path` for directory structure**
    - **Context:** PLAN.md § 1.2 Update Path Generation
    - **Action:**
        1. Modify existing `validate_log_path` function to handle subdirectories
        2. Add logic to check for/create the base log directory (e.g., `./logs`)
        3. Incorporate awareness of subdirectories defined in T001
    - **Done-when:**
        1. Function correctly handles base directory presence/creation
        2. Unit tests for path validation pass
        3. Path validation creates app/test subdirectories automatically
    - **Depends-on:** [T001]

## Environment Detection

- [x] **T003 · Feature · P2: implement `detect_environment` function**
    - **Context:** PLAN.md § 2.1 Create Environment Detection Function
    - **Action:**
        1. Define `LogEnvironment` enum (`Development`, `UserInstallation`, `SystemService`)
        2. Implement `detect_environment()` function with platform-specific detection logic:
           - Linux: Check parent PID == 1, inspect `/proc/self/cgroup`, and look for systemd env vars
           - macOS: Check parent PID == 1, check for no controlling TTY, and look for XPC env vars
           - Windows: Use `QueryServiceStatusEx` with Service Control Manager
    - **Done-when:**
        1. Function returns correct enum variant based on runtime conditions
        2. Unit tests verify detection logic for each environment
    - **Depends-on:** none

- [x] **T004 · Chore · P2: add `directories` crate dependency**
    - **Context:** PLAN.md § 2.2 Add XDG Directory Support
    - **Action:**
        1. Add the `directories` crate to `Cargo.toml`
    - **Done-when:**
        1. Dependency added
        2. Project compiles successfully (`cargo check`)
    - **Depends-on:** none

- [x] **T005 · Feature · P2: implement XDG-compliant path retrieval utility**
    - **Context:** PLAN.md § 2.2 Add XDG Directory Support
    - **Action:**
        1. Create a utility function using the `directories` crate
        2. Function retrieves the XDG data home path (e.g., `~/.local/share/switchboard/logs/`)
    - **Done-when:**
        1. Utility function returns the correct XDG base path for logging
        2. Unit tests verify path retrieval
    - **Depends-on:** [T003, T004]

## Path Resolution

- [x] **T006 · Feature · P1: implement `LogPathResolver` struct and `new` method**
    - **Context:** PLAN.md § 3.1 Create LogPathResolver
    - **Action:**
        1. Define the `LogPathResolver` struct (with fields for `base_dir`, `log_type`, `file_name`)
        2. Implement `LogPathResolver::new` method to initialize based on `Config` and `LogType`
        3. Define `LogType` enum if needed (`Application`, `Test`)
    - **Done-when:**
        1. Struct and `new` method are defined and compile
        2. Basic unit tests for initialization pass
    - **Depends-on:** [T003]

- [x] **T007 · Feature · P1: implement `LogPathResolver::resolve` method logic**
    - **Context:** PLAN.md § 3.2 Handle Path Construction
    - **Action:**
        1. Implement the `resolve` method within `LogPathResolver`
        2. Logic determines the final `PathBuf` based on environment, config, and log type
        3. Include directory creation (`std::fs::create_dir_all`) and set appropriate permissions:
           - Unix/Linux/macOS: 0o750 for directories, 0o640 for files
           - Windows: Use default permissions or tighten with ACLs if needed
    - **Done-when:**
        1. `resolve` returns correct paths for all environments and log types
        2. Necessary directories are created upon resolution
        3. Unit tests verify path construction and directory creation
    - **Depends-on:** [T001, T005, T006]

## Configuration

- [x] **T008 · Feature · P2: define `LogDirectoryMode` enum**
    - **Context:** PLAN.md § 4.1 Update Config Structure
    - **Action:**
        1. Define the `LogDirectoryMode` enum (`Default`, `XDG`, `System`) in `config.rs`
    - **Done-when:**
        1. Enum defined and accessible
        2. Code compiles successfully
    - **Depends-on:** none

- [x] **T009 · Feature · P2: add `log_dir_mode` field to `Config` struct**
    - **Context:** PLAN.md § 4.1 Update Config Structure
    - **Action:**
        1. Add `pub log_dir_mode: LogDirectoryMode` field to the `Config` struct in `config.rs`
        2. Update config loading/parsing logic with default values
    - **Done-when:**
        1. Field added to `Config`
        2. Configuration can be loaded/parsed with the new field
        3. Default value is correctly applied
        4. Unit tests for config loading pass
    - **Depends-on:** [T008]
    - **Note:** Implemented as `log_directory_mode` in T008 for better clarity

- [x] **T010 · Feature · P2: add environment variable support for `log_dir_mode`**
    - **Context:** PLAN.md § 4.2 Environment Variable Support
    - **Action:**
        1. Implement parsing of environment variable (e.g., `LOG_DIR_MODE`) during config loading
        2. Allow environment variable to override the config file setting
    - **Done-when:**
        1. Environment variable correctly affects `Config.log_dir_mode` when set
        2. Unit tests verify environment variable override behavior
    - **Depends-on:** [T009]
    - **Note:** Implemented as `LOG_DIRECTORY_MODE` in T008 with tests in `test_log_directory_mode_parsing`

- [x] **T011 · Chore · P3: document new log configuration options**
    - **Context:** PLAN.md § 4.2 Environment Variable Support
    - **Action:**
        1. Update documentation with new config options and environment variables
        2. Detail the `log_dir_mode` config option and its possible values
    - **Done-when:**
        1. Documentation accurately reflects the new configuration options
    - **Depends-on:** [T010]

## Test Infrastructure

- [x] **T012 · Test · P2: implement test log directory utilities**
    - **Context:** PLAN.md § 5.1 Test Directory Implementation
    - **Action:**
        1. Create helper functions for test setup/teardown
        2. Configure functions to use the `TEST_LOG_SUBDIR` via the `LogPathResolver`
    - **Done-when:**
        1. Test utility functions correctly configure logging for tests
        2. Utilities ensure test logs go to the designated test subdirectory
    - **Depends-on:** [T001, T007]

- [x] **T013 · Test · P2: update existing tests to use new test log utilities**
    - **Context:** PLAN.md § 5.2 Update Test Framework
    - **Action:**
        1. Refactor existing tests in the `tests/` directory
        2. Replace old logging setup with calls to the new helper functions
    - **Done-when:**
        1. All relevant tests utilize the new test logging utilities
        2. All tests pass and log files are created in the test subdirectory
    - **Depends-on:** [T012]

## Logger Implementation

- [x] **T014 · Feature · P1: integrate `LogPathResolver` into `init_tracing`**
    - **Context:** PLAN.md § 6.1 Update init_tracing
    - **Action:**
        1. Modify the `init_tracing` function
        2. Use `LogPathResolver` to determine the correct log file path
        3. Pass resolved path to logging backend setup
    - **Done-when:**
        1. Logging initialization uses the `LogPathResolver`
        2. Logs are created in correct locations based on configuration and environment
    - **Depends-on:** [T007, T009]

- [x] **T015 · Feature · P2: implement backward compatibility for legacy log paths**
    - **Context:** PLAN.md § 6.2 Update Path Validation, Backward Compatibility
    - **Action:**
        1. Enhance path validation/resolution logic
        2. Detect legacy format paths
        3. Log a warning and adapt to new structure if possible
    - **Done-when:**
        1. Legacy paths trigger warning messages
        2. Application continues to function with legacy paths
        3. Tests verify warning mechanism and graceful handling
    - **Depends-on:** [T014]

## Testing

- [ ] **T016 · Test · P2: add unit tests for environment detection**
    - **Context:** PLAN.md § 7.1 Unit Tests
    - **Action:**
        1. Write unit tests for `detect_environment()` function
        2. Test different environment scenarios (mocked if needed)
    - **Done-when:**
        1. Tests cover all three environment variants
        2. All tests pass
    - **Depends-on:** [T003]

- [ ] **T017 · Test · P2: add unit tests for path resolution**
    - **Context:** PLAN.md § 7.1 Unit Tests
    - **Action:**
        1. Write unit tests for `LogPathResolver.resolve()`
        2. Test different environment and configuration combinations
    - **Done-when:**
        1. Tests cover all path resolution scenarios
        2. All tests pass
    - **Depends-on:** [T007]

- [ ] **T018 · Test · P2: add unit tests for directory creation and permissions**
    - **Context:** PLAN.md § 7.1 Unit Tests
    - **Action:**
        1. Write tests for directory creation logic
        2. Test permission setting with expected values:
           - Unix/Linux/macOS: verify 0o750 for directories, 0o640 for files
           - Windows: verify default permissions are sufficient
    - **Done-when:**
        1. Tests verify directories are created with correct permissions
        2. All tests pass
    - **Depends-on:** [T007]

- [ ] **T019 · Test · P1: write integration test for app/test log separation**
    - **Context:** PLAN.md § 7.2 Integration Tests
    - **Action:**
        1. Create integration test scenario with both app and test logs
        2. Verify logs are created in separate subdirectories
    - **Done-when:**
        1. Integration test passes
        2. Log files exist in correct subdirectories
    - **Depends-on:** [T013, T014]

- [ ] **T020 · Test · P1: write integration tests for environment-specific paths**
    - **Context:** PLAN.md § 7.2 Integration Tests
    - **Action:**
        1. Create tests simulating different environments
        2. Verify log files are created in expected locations for each environment
    - **Done-when:**
        1. Tests pass for Development, User, and System path scenarios
        2. Log files are confirmed to be in correct locations
    - **Depends-on:** [T014]