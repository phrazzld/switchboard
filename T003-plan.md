# T003 Task Plan: Refactor `Config::load()` to use constants with fallback logging

## Task Description
From the TODO.md:
> **T003 · Refactor · P2: refactor `Config::load()` to use constants with fallback logging**
> - **Context:** PLAN.md > Detailed Build Steps > 2. Refactor `Config::default()` & `load_config()`; PLAN.md > Logging & Observability
> - **Action:**
>     1. In `src/config.rs`, update `Config::load()` to call `env::var("X").unwrap_or_else(|_| DEFAULT_X.to_string())` for each variable.
>     2. On parse errors (numeric/bool), log a `warn!` with `{ var, error }` and fall back to the constant.
> - **Done-when:**
>     1. All environment fallbacks use `DEFAULT_*` constants.
>     2. A warning is logged on parse failure before defaulting.
> - **Depends-on:** T001

## Analysis of Current State

Looking at the current `Config::load()` implementation in `src/config.rs`, I observe:

1. Most environment variables are already being loaded with fallbacks to the `DEFAULT_*` constants, which is good.
2. For numeric and boolean variables (`log_max_body_size` and `log_bodies`), the code handles parse errors but doesn't log warnings - it only prints to stderr.
3. The `log_max_age_days` parsing has similar behavior.

## Implementation Approach

1. Keep the existing environment variable loading patterns.
2. Replace the `eprintln!` statements with proper `warn!` log statements for parse errors.
3. Ensure consistent error handling and logging across all environment variable parsing.

Specifically, I will:

1. Update the error handling for `LOG_MAX_BODY_SIZE` to use `tracing::warn!` instead of `eprintln!`.
2. Update the error handling for `LOG_MAX_AGE_DAYS` to use `tracing::warn!` instead of `eprintln!`.
3. Add parse error warning logging for `LOG_BODIES` (currently doesn't warn on error).

The implementation should use the `tracing::warn!` macro with structured fields to show the variable name and the error.

## Testing Plan

The existing tests in `src/config.rs` already cover basic functionality but don't verify log output. Since this change primarily affects logging behavior rather than functionality, I'll:

1. Verify that the code compiles and that all existing tests pass.
2. Manually test by setting invalid environment variables and checking that appropriate warnings are emitted.

## Potential Challenges

It's worth noting that verifying log output in tests can be challenging. Since the task doesn't explicitly require test coverage for logging, I'll focus on implementing the correct logging behavior rather than trying to test it.