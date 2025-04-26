# T002 Task Plan: Refactor `Config::default()` to use constants

## Task Description
From the TODO.md:
> **T002 · Refactor · P2: refactor `Config::default()` to use constants**
> - **Context:** PLAN.md > Detailed Build Steps > 2. Refactor `Config::default()` & `load_config()`
> - **Action:**
>     1. In `src/config.rs`, update `impl Default for Config` or `Config::default()` to replace hardcoded literals with the new `DEFAULT_*` constants.
> - **Done-when:**
>     1. `Config::default()` uses only the defined constants.
>     2. Existing unit tests for defaults pass.
> - **Depends-on:** T001

## Analysis of Current State

After reviewing the code in `src/config.rs`, I can see that T001 has already been completed and the constants have been defined. Furthermore, `Config::default()` already uses the constants that were defined in T001. The implementation in lines 126-142 clearly shows that all values in the default implementation already reference the appropriate constants:

```rust
impl Default for Config {
    fn default() -> Self {
        Config {
            port: DEFAULT_PORT.to_string(),
            anthropic_api_key: "".to_string(),
            anthropic_target_url: DEFAULT_ANTHROPIC_TARGET_URL.to_string(),
            log_stdout_level: DEFAULT_LOG_STDOUT_LEVEL.to_string(),
            log_format: DEFAULT_LOG_FORMAT.to_string(),
            log_bodies: DEFAULT_LOG_BODIES,
            log_file_path: DEFAULT_LOG_FILE_PATH.to_string(),
            log_file_level: DEFAULT_LOG_FILE_LEVEL.to_string(),
            log_max_body_size: DEFAULT_LOG_MAX_BODY_SIZE,
            log_directory_mode: LogDirectoryMode::Default,
            log_max_age_days: DEFAULT_LOG_MAX_AGE_DAYS,
        }
    }
}
```

The only value that doesn't use a constant is `anthropic_api_key`, which is set to an empty string. This is intentional since API keys should never be set to default values (they should always be explicitly provided), which is also noted in the documentation comment.

## Verification

The tests in `src/config.rs` are already validating that the default values match the constants. The `test_default_values` test confirms that the defaults are being used correctly.

## Conclusion

T002 is already completed as part of the earlier work on T001. All default values in `Config::default()` correctly use the constants defined in T001. The tests already verify that these defaults are being used correctly. No changes are needed for this task.

The next task to address would be T003: refactoring `Config::load()` with better logging for parse errors.