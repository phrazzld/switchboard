# Implementation Plan: Restructure Log File Organization

## Overview
This plan outlines the implementation for restructuring the log file organization with proper directory structure, the top item from the Log Organization & Infrastructure section of the backlog.

## Goals
- Create a robust directory structure for log files
- Use dedicated directories (`./logs/` by default)
- Separate application logs from test logs into different subdirectories
- Respect XDG Base Directory spec for user-level logs
- Use system standards (/var/log) for service deployment

## Implementation Steps

### 1. Baseline Directory Structure

#### 1.1 Create Log Directory Constants
- Add constants for standard log directories in logger.rs:
  ```rust
  // Standard log directory paths
  pub const DEFAULT_LOG_DIR: &str = "./logs";
  pub const APP_LOG_SUBDIR: &str = "app";
  pub const TEST_LOG_SUBDIR: &str = "test";
  ```

#### 1.2 Update Path Generation
- Modify `validate_log_path` to handle directory creation and structure
- Update path validation to manage subdirectories properly

### 2. Environment Detection

#### 2.1 Create Environment Detection Function
- Implement function to detect the running environment:
  ```rust
  pub enum LogEnvironment {
      Development,
      UserInstallation,
      SystemService,
  }
  
  fn detect_environment() -> LogEnvironment { ... }
  ```

#### 2.2 Add XDG Directory Support
- Add dependency on `directories` crate
- Implement XDG-compliant path resolution for user installations
- Create utility functions to retrieve appropriate paths based on environment

### 3. Path Resolution

#### 3.1 Create LogPathResolver
- Implement a resolver that determines correct log path based on:
  - Explicitly configured path
  - Environment (development, user, service)
  - Log type (application, test)

#### 3.2 Handle Path Construction
- Build proper paths with standardized directory structure
- Create necessary directories if they don't exist
- Set appropriate permissions

### 4. Config Integration

#### 4.1 Update Config Structure
- Add new configuration options in config.rs:
  ```rust
  pub log_dir_mode: LogDirectoryMode, // enum with values like Default, XDG, System
  ```

#### 4.2 Environment Variable Support
- Add environment variables for the new settings
- Document the new configuration options

### 5. Test Log Separation

#### 5.1 Test Directory Implementation
- Create separate test log directory
- Add utility functions for test setup
- Ensure test logs don't interfere with application logs

#### 5.2 Update Test Framework
- Modify test files to use the test log directory
- Update test utilities to handle proper directory structure

### 6. Implementation in Logger

#### 6.1 Update init_tracing
- Modify to use the new path resolution logic
- Handle creation of proper directory structure
- Support different environments

#### 6.2 Update Path Validation
- Enhance validation for the new directory structure
- Maintain backward compatibility with existing paths

### 7. Testing

#### 7.1 Unit Tests
- Test path resolution in different environments
- Test directory creation and management
- Test environment detection

#### 7.2 Integration Tests
- Test log file creation in correct locations
- Test proper separation of app and test logs
- Test environment-specific behavior

## Implementation Details

### Directory Structure
```
logs/
├── app/                    # Application logs 
│   ├── switchboard-YYYY-MM-DD.log
│   └── switchboard-YYYY-MM-DD.log.1
└── test/                   # Test logs
    ├── test-YYYY-MM-DD.log
    └── test-YYYY-MM-DD.log.1
```

### Environment-Specific Paths
- Development: `./logs/` (relative to working directory)
- User Installation: `~/.local/share/switchboard/logs/` (XDG_DATA_HOME)
- System Service: `/var/log/switchboard/`

### Log Path Resolver Implementation
```rust
struct LogPathResolver {
    base_dir: PathBuf,
    log_type: LogType,
    file_name: String,
}

impl LogPathResolver {
    fn new(config: &Config, log_type: LogType) -> Self { ... }
    
    fn resolve(&self) -> Result<PathBuf, LogInitError> {
        // Logic to build the appropriate path with:
        // - Base directory based on environment
        // - Subdirectory based on log type
        // - Proper file name
    }
}
```

### Configuration Changes
```rust
// In Config struct (config.rs)
pub struct Config {
    // Existing fields...
    
    /// Log directory mode (Default, XDG, System)
    pub log_dir_mode: LogDirectoryMode,
}

pub enum LogDirectoryMode {
    Default,  // Use ./logs/
    XDG,      // Use XDG directories
    System,   // Use system directories (/var/log)
}
```

## Testing Strategy

1. Unit tests:
   - Test path resolution in different environments
   - Test directory creation and permissions
   - Test environment detection logic

2. Integration tests:
   - Test correct log file creation in app vs test directories
   - Test proper file naming and directory structure
   - Test environment-specific behavior

## Backward Compatibility

1. Maintain backward compatibility with existing log paths
2. If a legacy path is specified, convert it to the new structure when possible
3. Provide warning logs when using a deprecated path format