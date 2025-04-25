# T012 Plan - Implement Test Log Directory Utilities

## Objective

Create helper functions for test setup/teardown that use the recently implemented LogPathResolver to:
1. Configure test logging to use the TEST_LOG_SUBDIR
2. Provide clean and reusable utilities for all tests
3. Ensure test logs are separated from application logs

## Implementation Strategy

### 1. Explore Current Test Setup

First, examine the current test structure to understand how tests are currently configured:
- How test configurations are currently created
- How logging is initialized in tests
- Where test logs are currently being stored

### 2. Design Test Utilities

Design helper functions to be added to `tests/common/mod.rs`:
- `setup_test_logging`: Initializes logging for tests using TEST_LOG_SUBDIR
- `generate_test_log_path`: Creates unique test log paths to prevent collisions
- `clean_test_logs`: Cleans up test logs after tests complete (optional)

### 3. Implement Test Utilities

Implement the helper functions with these considerations:
- Use LogPathResolver to correctly resolve paths for TEST_LOG_SUBDIR
- Generate unique log file names to avoid conflicts between parallel test executions
- Set appropriate log levels for test output

### 4. Add Test Validation Utilities

Create helper functions to verify test behavior:
- `verify_log_file_exists`: Check if log files were created successfully
- `verify_log_directory`: Validate log directory structure for tests

### 5. Testing Strategy

Create unit tests for the new utilities to ensure they work as expected:
- Test log file creation in the correct directory
- Test unique log file name generation
- Test cleanup functionality if implemented

## Implementation Details

1. Add utility functions to `tests/common/mod.rs` 
2. Ensure functions properly initialize LogPathResolver with TEST_LOG_SUBDIR
3. Create appropriate temporary/unique file naming strategy for tests
4. Add validation helpers to verify correct log file creation