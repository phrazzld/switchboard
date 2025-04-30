# T013 Implementation Plan: Replace panics/expects in load_config with error returns

## Task Description
From TODO.md:
```
- [~] **T013 · Refactor · P1: replace panics/expects in load_config with err returns**
    - **Context:** PLAN.md > cr-01 > Steps > 4
    - **Action:**
        1. Identify `panic!`, `expect`, `unwrap` calls related to config loading/validation in `load_config`.
        2. Replace them with appropriate `return Err(ConfigError::...)` variants.
        3. Use `?` operator for fallible operations like `env::var`.
    - **Done‑when:**
        1. `load_config` no longer panics on configuration errors.
        2. Returns `Err(ConfigError::...)` for invalid/missing configuration.
        3. `cargo check` passes.
    - **Depends‑on:** [T012]
```

## Implementation Approach

### 1. Identify all unwrap/expect in load_config

Looking at `src/config.rs`, I've identified the following instances of unwrap/expect in the `load_config` function:

1. Line 333: `let port = env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());`
   - This is using `unwrap_or_else` which is fine as it provides a default
   - No action needed as this is safe fallback behavior

2. Lines 343-344: 
   ```rust
   let anthropic_target_url = env::var("ANTHROPIC_TARGET_URL")
      .unwrap_or_else(|_| DEFAULT_ANTHROPIC_TARGET_URL.to_string());
   ```
   - Using `unwrap_or_else` with default, no action needed

3. Line 348: `let openai_api_key = env::var("OPENAI_API_KEY").ok();`
   - Using `.ok()` which converts to an Option, no action needed

4. Lines 348-349:
   ```rust
   let openai_api_base_url =
       env::var("OPENAI_API_BASE_URL").unwrap_or_else(|_| DEFAULT_OPENAI_TARGET_URL.to_string());
   ```
   - Using `unwrap_or_else` with default, no action needed

5. Lines 354-355:
   ```rust
   let log_stdout_level =
       env::var("LOG_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_STDOUT_LEVEL.to_string());
   ```
   - Using `unwrap_or_else` with default, no action needed

6. Line 356:
   ```rust
   let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| DEFAULT_LOG_FORMAT.to_string());
   ```
   - Using `unwrap_or_else` with default, no action needed

7. Lines 362-363:
   ```rust
   let log_file_path =
       env::var("LOG_FILE_PATH").unwrap_or_else(|_| DEFAULT_LOG_FILE_PATH.to_string());
   ```
   - Using `unwrap_or_else` with default, no action needed

8. Lines 364-365:
   ```rust
   let log_file_level =
       env::var("LOG_FILE_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_FILE_LEVEL.to_string());
   ```
   - Using `unwrap_or_else` with default, no action needed

9. Lines 368-381: Parsing for LOG_MAX_BODY_SIZE
   ```rust
   let log_max_body_size = env::var("LOG_MAX_BODY_SIZE")
       .ok()
       .and_then(|size_str| {
           size_str.parse::<usize>().ok().or_else(|| {
               warn!(
                   var = "LOG_MAX_BODY_SIZE",
                   value = %size_str,
                   default = DEFAULT_LOG_MAX_BODY_SIZE,
                   "Failed to parse numeric environment variable, using default"
               );
               None
           })
       })
       .unwrap_or(DEFAULT_LOG_MAX_BODY_SIZE);
   ```
   - This is already using `unwrap_or` with a default, no action needed

10. Lines 384-390: Parsing for LOG_DIRECTORY_MODE
    - No unwrap/expect here, using `unwrap_or` with default

11. Lines 393-410: Parsing for LOG_MAX_AGE_DAYS
    - No unwrap/expect here, using `and_then` and `or_else` with proper defaults

Most of the `unwrap_or_else` calls are already using sensible defaults, which is the expected pattern for optional configs. What we need to focus on are potential validation failures that currently return errors.

### 2. Improvements Required

1. For all numeric environment variables that currently emit warnings but continue with defaults, we should consider using appropriate error types for invalid values rather than silently continuing with defaults. Specifically:
   - LOG_MAX_BODY_SIZE
   - LOG_MAX_AGE_DAYS

2. For any error cases that aren't yet handled or that use `.expect()` or `.unwrap()` without fallbacks.

3. We need to ensure that we return appropriate ConfigError variants instead of panicking.

### 3. Changes to Implement

1. Update the numeric parsing functions to return appropriate errors when parsing fails instead of just logging warnings.

2. Add additional validation for required values where needed.

3. Ensure we're using the `?` operator for all potential error cases.

4. Check for any remaining panic/expect/unwrap instances that need to be converted to proper error handling.

## Implementation Plan

1. Update LOG_MAX_BODY_SIZE parsing to return an error when the value is invalid
2. Update LOG_MAX_AGE_DAYS parsing to return an error when the value is invalid
3. Add additional validation for other environment variables where needed
4. Run `cargo check` to ensure the code compiles
5. Run `cargo clippy` to catch any additional issues