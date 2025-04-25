# LC006: Implement automatic log cleanup for development

## Task Overview
- Add a configuration option for max log age in development
- Implement log cleanup during application startup
- Add a CLI flag for manual log cleanup (`--clean-logs`)

## Implementation Approach

### 1. Analyze Current Codebase

First, I need to understand:
- How configuration is currently structured
- Where/when application startup occurs
- How CLI flags are processed
- Current log structure and file handling

### 2. Implementation Plan

#### Configuration Changes
1. Add a new configuration option to `Config` struct in `src/config.rs`:
   - `log_max_age_days`: Optional u32 for maximum age of log files in days
   - Default to `None` (no automatic cleanup)

#### CLI Flag Addition
1. Update CLI argument parsing in `main.rs` to support a new `--clean-logs` flag
2. When detected, run the cleanup function before normal execution

#### Cleanup Logic Implementation
1. Create a new module `src/log_cleanup.rs` with:
   - Function to scan log directories and identify files older than max_age
   - Utility to parse log filenames and extract timestamps
   - Function to safely delete old log files
   - Entry point function that can be called both at startup and via CLI flag

#### Integration with Startup
1. Modify application startup to call cleanup when configured
2. Ensure proper error handling if cleanup fails (warn, don't fail startup)
3. Add appropriate logging for cleanup operations

### 3. Testing Approach
1. Create unit tests for the cleanup logic:
   - Test identification of old files
   - Test deletion logic
   - Test CLI flag handling

2. Integration tests:
   - Create test files of various ages
   - Run cleanup with different configurations
   - Verify correct files are removed while others remain

### 4. Expected Impact
- Low impact on normal operation
- Development environments will have cleaner log directories
- Manual cleanup option gives developers control

### 5. Considerations
- Need to handle both app and test log directories
- Must be careful not to delete logs from active sessions
- Should work cross-platform (Windows/Linux/MacOS)
- Needs proper error handling to prevent application failures