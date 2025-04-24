# Dual-Output Logging System Implementation Plan

## Task Summary
Implement a dual-output logging system where:
1. Comprehensive structured logs are written to a file in JSON format
2. High signal-to-noise ratio logs are written to stdout in a configurable format

## Technical Requirements
- Log Claude Code operations via request/response proxying
- Log request/response content with metadata (headers, status codes, etc.)
- Support configuration of log levels for both outputs
- Implement file output with daily rotation
- Maintain performance with non-blocking I/O for file output

## Implementation Plan

### 1. Analyze Current Logging Infrastructure (0.5 day)
- [x] Review existing logger implementation in `src/logger.rs`
- [x] Identify current log format and content
- [x] Assess integration points for capturing operations
- [x] Determine configuration mechanism

### 2. Design Enhanced Logging System (0.5 day)
- [ ] Define log levels and categories
  - Comprehensive (file): DEBUG level and above with full details
  - High signal (stdout): INFO level and above, focused on key operations
- [ ] Design configuration options:
  - `LOG_FILE_PATH`: Path to the log file (default: "./switchboard.log")
  - `LOG_FILE_LEVEL`: Minimum log level for file output (default: "debug")
  - `LOG_LEVEL`: Minimum log level for stdout (default: "info")
  - `LOG_FORMAT`: Format for stdout logs (default: "pretty", other: "json")
  - `LOG_BODIES`: Boolean to toggle request/response body logging (default: true)
  - `LOG_MAX_BODY_SIZE`: Maximum size for logged bodies (default: 20480 bytes)

### 3. Implement Core Logging Infrastructure (1 day)
- [ ] Add `tracing-appender` dependency for file output
- [ ] Update `Config` struct with new logging parameters
- [ ] Modify `logger.rs` to support dual outputs with separate filters
- [ ] Implement non-blocking JSON file logging with daily rotation
- [ ] Implement configurable stdout logging (pretty or JSON)

### 4. Implement Request/Response Logging (0.5 day)
- [ ] Update request logging functions to use new configuration
- [ ] Update response logging functions to use new configuration
- [ ] Ensure sensitive data (API keys, auth headers) is redacted
- [ ] Add configuration-based body logging with size limits

### 5. Testing (1 day)
- [ ] Create unit tests for logging configuration
- [ ] Test file output with different log levels
- [ ] Test stdout output with different log levels and formats
- [ ] Verify file rotation works correctly
- [ ] Test with large and small request/response bodies
- [ ] Test performance impact

### 6. Documentation (0.5 day)
- [ ] Update code documentation with clear explanations
- [ ] Update README with new configuration options
- [ ] Add examples of log output and usage
- [ ] Document log file format and schema

### 7. Code Review and Refinement (0.5 day)
- [ ] Review implementation against project standards
- [ ] Check for potential improvements
- [ ] Address any issues found during testing
- [ ] Finalize PR

## Success Criteria
- Comprehensive logs are written to the configured file in JSON format
- High signal logs are written to stdout in the configured format
- Log levels are independently configurable for file and stdout
- Request and response content is logged appropriately based on configuration
- File logging uses non-blocking I/O and daily rotation
- Performance impact is minimal
- Sensitive data is properly redacted in logs