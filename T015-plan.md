# T015 Implementation Plan: Update Tests for Config Loading Result/Error Variants

## Task Description
From TODO.md:
```
- [~] **T015 · Test · P1: update tests for config loading result/error variants**
    - **Context:** PLAN.md > cr-01 > Steps > 6
    - **Action:**
        1. Review tests in `tests/config_test.rs` and others relying on config loading.
        2. Update tests expecting panics to assert `Err` results with specific `ConfigError` variants.
        3. Add tests covering different `ConfigError` scenarios.
    - **Done‑when:**
        1. Tests accurately reflect `Result`-returning behavior of `load_config`.
        2. Tests cover specific `ConfigError` variants.
        3. `cargo test` passes.
    - **Depends‑on:** [T013]
```

## Task Classification
This is a **Simple** task that involves updating tests to properly test the refactored `load_config` function. The changes will be focused in a single file (`tests/config_test.rs`) and follow a clear pattern of testing different error variants.

## Analysis of Current Tests
After reviewing `tests/config_test.rs`, I can see that the tests need to be updated because:

1. Most tests use the `create_test_config_with_env` helper function, which directly constructs a `Config` instead of using `load_config`
2. The few tests that do test error cases (like `test_openai_api_key_required_when_enabled`) use hardcoded panic messages instead of testing for specific `ConfigError` variants
3. Not all `ConfigError` variants have dedicated tests

## Implementation Plan

### 1. Create a new helper function for testing load_config

Create a new helper function named `test_load_config_with_env` that:
- Takes a `HashMap<&str, &str>` of environment variables
- Sets those environment variables
- Calls `load_config()`
- Returns the `Result<Config, ConfigError>`
- Restores the original environment

### 2. Update or Create Tests for Each ConfigError Variant

Create or update tests for the following error variants:

1. `MissingAnthropicApiKey`
   - Test with no ANTHROPIC_API_KEY set

2. `EmptyValue`
   - Test with empty ANTHROPIC_API_KEY

3. `MissingOpenAiKey`
   - Update the existing test to verify the error type

4. `InvalidBooleanValue`
   - Create a test with invalid values for LOG_BODIES or OPENAI_ENABLED

5. `InvalidNumericValue`
   - Create tests for invalid PORT and LOG_MAX_BODY_SIZE values

6. `InvalidFormat`
   - Create tests for invalid URL format
   - Create tests for invalid log level format
   - Create tests for invalid log directory mode

7. `AlreadyInitialized`
   - This is used by `set_global_config`, so it might need a separate test

### 3. Keep Working Tests Intact

Ensure that all existing functionality tests continue to pass, only modifying those that should be testing errors.

## Implementation Steps

1. Create the `test_load_config_with_env` helper function
2. Implement all error variant tests
3. Update any tests that use `#[should_panic]` to instead check for specific error variants
4. Run the tests to verify they pass
5. Check test coverage for all ConfigError variants