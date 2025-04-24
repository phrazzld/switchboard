# TODO

- [x] **T001: Add `tracing-appender` Dependency**
  - Add the `tracing-appender` crate to the `Cargo.toml` file
  - Ensure compatibility with existing dependencies
  - Run `cargo update` to update the lockfile

  **Acceptance Criteria:**
  - `tracing-appender = "0.2"` (or latest version) added to `Cargo.toml`
  - `cargo build` completes successfully after adding the dependency
  - No dependency conflicts arise

  **Dependencies:** None

- [x] **T002: Extend Config Struct with New Logging Fields**
  - Add the following fields to the `Config` struct:
    - `log_file_path: String` - Path to the log file (default: "./switchboard.log")
    - `log_file_level: String` - Minimum log level for file output (default: "debug")
    - `log_stdout_level: String` - Minimum log level for stdout (default: "info") (replaces existing `log_level`)
    - `log_format: String` - Format for stdout logs (default: "pretty", other: "json")
    - `log_bodies: bool` - Toggle for request/response body logging (default: true)
    - `log_max_body_size: usize` - Maximum size for logged bodies (default: 20480 bytes)
  - Add Doc comments explaining each field and default value

  **Acceptance Criteria:**
  - All new fields added to `Config` struct with appropriate types
  - Default values match the specification
  - Doc comments present for each new field
  - Existing code updated to use `log_stdout_level` where appropriate

  **Dependencies:** None

- [x] **T003: Update Configuration Loading from Environment**
  - Update `load_config()` to read the new environment variables:
    - `LOG_FILE_PATH`
    - `LOG_FILE_LEVEL`
    - `LOG_LEVEL` (map to `log_stdout_level`)
    - `LOG_FORMAT`
    - `LOG_BODIES`
    - `LOG_MAX_BODY_SIZE`
  - Implement proper parsing and validation for each variable
  - Apply defaults when environment variables are not set or invalid

  **Acceptance Criteria:**
  - All environment variables correctly parsed
  - Invalid log levels/formats fall back to defaults with appropriate warning logs
  - All new config fields properly initialized
  - Config initialization log updated to include new fields

  **Dependencies:** T002

- [x] **T004: Implement Dual-Output Logging Infrastructure**
  - Modify `src/logger.rs` to support two separate logging outputs:
    - File output with JSON formatting and filtering by `log_file_level`
    - Stdout output with configurable formatting and filtering by `log_stdout_level`
  - Implement non-blocking file logging with daily rotation using `tracing_appender`
  - Configure JSON formatting for file output
  - Implement configurable formatting for stdout (pretty or JSON)

  **Acceptance Criteria:**
  - File logging uses non-blocking I/O via `tracing_appender`
  - File logs are formatted as JSON
  - File logs are filtered by `log_file_level`
  - Daily log rotation works correctly
  - Stdout logs use the format specified by `log_format`
  - Stdout logs are filtered by `log_stdout_level`
  - Both outputs include the same contextual fields

  **Dependencies:** T001, T003

- [x] **T005: Update Request Logging with New Configuration**
  - Update `log_request_details` function to:
    - Include the `log_max_body_size` parameter
    - Respect the `log_bodies` configuration
    - Use appropriate log levels for file vs. stdout
    - Continue redacting sensitive headers
  - Update callers to pass the new parameter

  **Acceptance Criteria:**
  - Request bodies are logged only when `log_bodies` is true
  - Request bodies are truncated at `log_max_body_size` bytes
  - Appropriate structured context is included in all logs
  - Sensitive headers continue to be redacted

  **Dependencies:** T004

- [x] **T006: Update Response Logging with New Configuration**
  - Update `log_response_details` function to:
    - Include the `log_max_body_size` parameter
    - Respect the `log_bodies` configuration
    - Use appropriate log levels for file vs. stdout
    - Include response timing metrics
  - Update callers to pass the new parameter

  **Acceptance Criteria:**
  - Response bodies are logged only when `log_bodies` is true
  - Response bodies are truncated at `log_max_body_size` bytes
  - Appropriate structured context is included in all logs
  - Response timing information is logged

  **Dependencies:** T004

- [x] **T007: Create Unit Tests for Configuration**
  - Add tests to verify:
    - Default values when environment variables are not set
    - Proper parsing when environment variables are set
    - Fallback to defaults for invalid values
    - Edge cases (extremely large/small values, unusual paths)

  **Acceptance Criteria:**
  - Tests cover all new configuration parameters
  - Tests verify correct default application
  - Tests verify environment variable precedence
  - Tests verify validation and error handling

  **Dependencies:** T003

- [x] **T008: Create Tests for File Logging**
  - Add tests to verify:
    - Logs are written to the specified file
    - JSON format is correctly used
    - Filtering by `log_file_level` works correctly
    - File rotation creates new files at appropriate times

  **Acceptance Criteria:**
  - Tests verify file creation and writing
  - Tests verify JSON formatting
  - Tests verify level filtering
  - Tests verify rotation behavior (possibly with mocked time)

  **Dependencies:** T004

- [x] **T009: Create Tests for Stdout Logging**
  - Add tests to verify:
    - Logs are written to stdout
    - Format switches between pretty and JSON correctly
    - Filtering by `log_stdout_level` works correctly

  **Acceptance Criteria:**
  - Tests verify stdout output
  - Tests verify format switching
  - Tests verify level filtering

  **Dependencies:** T004

- [x] **T010: Test Body Logging and Size Limits**
  - Add tests to verify:
    - Bodies are logged when `log_bodies` is true
    - Bodies are omitted when `log_bodies` is false
    - Bodies are truncated at `log_max_body_size` when too large
    - Truncation is clearly indicated

  **Acceptance Criteria:**
  - Tests verify body logging toggle
  - Tests verify size-based truncation
  - Tests verify truncation indication

  **Dependencies:** T005, T006

- [x] **T011: Conduct Performance Benchmarking**
  - Compare performance metrics:
    - Before implementing dual-output logging
    - After implementing dual-output logging
  - Focus on:
    - Request throughput
    - Latency
    - CPU usage during high-volume logging

  **Acceptance Criteria:**
  - Benchmark data collected for before/after
  - Performance impact is minimal (<5% degradation)
  - Non-blocking I/O behavior is verified
  - Results documented for future reference

  **Dependencies:** T004, T005, T006

- [x] **T012: Update Documentation for Logging**
  - Update module-level documentation in `src/logger.rs`
  - Add detailed documentation for all public logging functions
  - Document the JSON schema used for file logs
  - Provide examples of common logging patterns

  **Acceptance Criteria:**
  - All public APIs are documented
  - JSON schema is documented
  - Examples are provided
  - Documentation builds without warnings

  **Dependencies:** T004

- [x] **T013: Update README with Logging Configuration**
  - Add a section to the README detailing:
    - New environment variables with descriptions and defaults
    - Example log output for both stdout and file
    - Information about log rotation
    - Common configuration scenarios

  **Acceptance Criteria:**
  - README includes all new environment variables
  - Examples demonstrate both formats
  - Log rotation is explained
  - Common scenarios are illustrated

  **Dependencies:** T004

- [x] **T014: Final Code Review and Refinement**
  - Conduct a thorough code review
  - Address any issues with:
    - Code style and consistency
    - Error handling
    - Edge cases
    - Documentation clarity

  **Acceptance Criteria:**
  - Code passes all linters and formatters
  - All tests pass
  - Error handling is comprehensive
  - Documentation is complete and accurate

  **Dependencies:** T001-T013