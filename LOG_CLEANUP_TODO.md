# Log Cleanup TODO

## Overview
This document outlines tasks for cleaning up existing log files and ensuring all parts of the application correctly use the new log directory structure. The goal is to migrate from scattered log files at the root directory to the organized structure in `/logs/app/` and `/logs/test/`.

## Tasks

### Phase 1: Migration Utility

- [x] **LC001 · Utility · P1: Create log file migration utility**
    - **Context:** Move existing log files to proper subdirectories
    - **Action:**
        1. Create a utility script `tools/migrate_logs.rs` or `tools/migrate_logs.sh`
        2. Identify log files in root directory
        3. Determine appropriate destination based on filename patterns:
           - `*_test.log*` -> `logs/test/`
           - `test_*.log*` -> `logs/test/`
           - `benchmark.log*` -> `logs/test/`
           - Other `.log*` files -> `logs/app/`
        4. Move files preserving timestamps
    - **Done-when:**
        1. Utility successfully moves existing log files to correct subdirectories
        2. No log files remain in root directory (except for ignored patterns)
    - **Depends-on:** none

- [x] **LC002 · Documentation · P2: Document log migration process**
    - **Context:** Ensure developers know how to handle existing log files
    - **Action:**
        1. Add documentation to README.md about log directory structure
        2. Document the migration utility and how to use it
        3. Explain the convention for log file locations
    - **Done-when:**
        1. Documentation updated with clear instructions
        2. Log file structure clearly explained
    - **Depends-on:** [LC001]

### Phase 2: Fix Remaining Code

- [x] **LC003 · Fix · P1: Update remaining hardcoded log paths in tests**
    - **Context:** Some test files may still use hardcoded log paths
    - **Action:**
        1. Identify any remaining tests using direct log paths
        2. Update to use `LogPathResolver` consistently
        3. Fix test setup and teardown code to use proper paths
    - **Done-when:**
        1. All tests use the new `LogPathResolver` structure
        2. No hardcoded log paths remain in test code
        3. Tests continue to pass
    - **Depends-on:** none

- [x] **LC004 · Fix · P1: Clean up orphaned logs in logger_directory_test.rs and logger_file_test.rs**
    - **Context:** These test files create log files that may not be properly cleaned up
    - **Action:**
        1. Review test cleanup code in these files
        2. Ensure all created log files are properly deleted after tests
        3. Update tests to use `LogPathResolver` if they don't already
    - **Done-when:**
        1. Tests clean up after themselves
        2. No orphaned files remain after test execution
    - **Depends-on:** [LC003]

### Phase 3: Automated Maintenance

- [x] **LC005 · Feature · P2: Add log cleanup as part of CI process**
    - **Context:** Log files should not be committed to repository
    - **Action:**
        1. Add `.gitignore` patterns for log files in all directories
        2. Create a CI step to verify no log files are being committed
        3. Enforce cleanup of test log files after test execution
    - **Done-when:**
        1. Log files are properly excluded from git
        2. CI checks fail if log files would be committed
    - **Depends-on:** [LC001, LC004]

- [ ] **LC006 · Feature · P2: Implement automatic log cleanup for development**
    - **Context:** Development environments should have option to clean old logs
    - **Action:**
        1. Add a configuration option for max log age in development
        2. Implement log cleanup during application startup
        3. Add a CLI flag for manual log cleanup (`--clean-logs`)
    - **Done-when:**
        1. Feature successfully removes old log files in development
        2. Configuration option works as expected
    - **Depends-on:** [LC005]

### Phase 4: Verification and Documentation

- [x] **LC007 · Test · P1: Create verification test for log directory structure**
    - **Context:** Ensure log structure is maintained
    - **Action:**
        1. Create a new test that verifies logs are created in correct locations
        2. Test all log types (app, test, benchmark)
        3. Verify directory permissions and structure
    - **Done-when:**
        1. Test passes and verifies correct log structure
    - **Depends-on:** [LC003]

- [ ] **LC008 · Documentation · P3: Update project documentation with log structure details**
    - **Context:** Ensure complete documentation of log system
    - **Action:**
        1. Update all relevant documentation with log directory structure
        2. Add diagrams or examples showing the structure
        3. Document environment-specific paths
    - **Done-when:**
        1. Documentation is complete and accurate
        2. Examples show correct usage
    - **Depends-on:** [LC002, LC007]