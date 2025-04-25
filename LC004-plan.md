# LC004: Clean up orphaned logs in test files

## Objective
Ensure that test files properly clean up log files created during test execution, focusing specifically on `logger_directory_test.rs` and `logger_file_test.rs`.

## Current State Analysis
From the previous task, we know that these test files have been updated to use `LogPathResolver` correctly. However, there may still be issues with:
1. Incomplete cleanup after tests
2. Error handling in cleanup code
3. Test failures leaving orphaned files

## Implementation Plan

1. **Review Current Cleanup Approach**
   - Examine current cleanup code in the target files
   - Check if cleanup happens in all test code paths (including failure paths)
   - Verify that cleanup uses the correct paths returned by `LogPathResolver`

2. **Identify Improvement Areas**
   - Look for tests without proper cleanup
   - Check for cleanup in success paths only (missing cleanup on test failures)
   - Identify any hardcoded paths still being used in cleanup logic

3. **Implement Robust Cleanup**
   - Add/improve cleanup code using the same path resolution as the test setup
   - Ensure cleanup occurs even if tests fail (using test fixture teardown patterns)
   - Verify correct cleanup in all scenarios

4. **Testing**
   - Run tests and check for residual log files
   - Add intentional test failures to verify cleanup still occurs
   - Verify that the proper paths are being cleaned up

## Success Criteria
- All tests in both files properly clean up their log files after execution
- No orphaned log files remain in the test or app log directories
- Tests handle cleanup consistently, using the same path resolution as file creation
- All tests pass after these changes