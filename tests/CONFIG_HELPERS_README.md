# Config Helpers for Tests

The `tests/common/config_helpers.rs` module provides standardized utilities for creating and manipulating `Config` instances in tests. These helpers provide a reliable, consistent way to work with configuration in tests while minimizing side effects.

## Key Features

- **Non-panicking behavior**: Returns proper `Result` types instead of panicking
- **Mirrors production code**: Same behavior as the main configuration system
- **Environment safety**: Tools to safely manage environment variables during tests
- **Comprehensive error handling**: Proper error variants for different failure modes
- **Well-tested**: Comprehensive test coverage for all helpers

## Usage Guide

### 1. Importing the helpers

```rust
// At the top of your test file
use crate::common::config_helpers::{
    load_test_config,  // For Result-returning helper that mirrors production
    create_test_config, // For simple test configs with defaults
    with_env_vars,     // For safely managing environment variables
};
```

### 2. Loading config with environment variables

```rust
#[test]
fn test_with_custom_environment() {
    // Set multiple environment variables
    let env_vars = vec![
        ("ANTHROPIC_API_KEY", "test-key"),
        ("PORT", "9090"),
        ("LOG_FORMAT", "json"),
    ];

    with_env_vars(env_vars, || {
        // Load the config within the closure where env vars are set
        let config = load_test_config().expect("Failed to load test config");

        // Verify the loaded values
        assert_eq!(config.port, "9090");
        assert_eq!(config.log_format, "json");
    });
    
    // Environment is automatically restored after closure exits
}
```

### 3. Testing error handling

```rust
#[test]
fn test_error_handling() {
    // Test with invalid values
    let env_vars = vec![
        ("ANTHROPIC_API_KEY", "test-key"),
        ("PORT", "invalid-port"), // Not a number
    ];

    with_env_vars(env_vars, || {
        let result = load_test_config();
        
        // Result should be an error with correct type
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidNumericValue { var, .. }) => {
                assert_eq!(var, "PORT");
            }
            _ => panic!("Expected InvalidNumericValue error"),
        }
    });
}
```

### 4. Simple testing with default values

```rust
#[test]
fn test_with_default_config() {
    // Create a config with sensible defaults for testing
    let config = create_test_config();
    
    // Verify expected test values
    assert_eq!(config.log_stdout_level, "debug");
    assert_eq!(config.log_file_level, "trace");
    
    // Use the config for testing without worrying about environment
    // This config has test API keys already set
}
```

## Advantages Over Individual Test Helpers

- **Consistent behavior**: All tests use the same config loading logic
- **Proper error handling**: Returns `Result` types instead of panicking
- **Isolation**: Environment variables are properly managed to avoid side effects
- **Maintainability**: Changes to config structure only need to be updated in one place

## Migration Guide

For tests currently using custom config helpers like `create_test_config_with_env`, consider migrating to these standardized helpers to ensure consistent behavior across tests.