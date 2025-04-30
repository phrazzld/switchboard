# C001 Resolution: Strategy for handling Result<Config, ConfigError> with static config initialization

## Chosen Strategy: Explicit Initialization in `main` with `OnceLock`

Based on the thinktank analysis and keeping with the project's development philosophy, we will implement the following strategy:

1. Modify `config::load_config()` to return `Result<Config, ConfigError>` instead of panicking on recoverable errors.
2. Handle the `Result` explicitly in `main.rs` at application startup, logging errors and exiting gracefully if loading fails.
3. Store the validated `Config` in a `static OnceLock<Config>` within the `config` module only after successful loading in `main`.
4. Provide a simple getter function (`config::get_config()`) that returns `&'static Config` and panics only if accessed before initialization in `main`.

This approach aligns with Rust's error handling best practices by:
- Separating the fallible loading logic from the static storage
- Ensuring clear error handling at the application boundary
- Providing a clean API for accessing configuration throughout the codebase
- Making configuration loading testable

## Implementation Plan

### 1. Modify `src/config.rs`:

- Change `load_config()` signature to return `Result<Config, ConfigError>`
- Convert panics to appropriate `Err` returns
- Keep the existing `OnceLock<Config>` but make it private
- Add a new function `set_global_config(config: Config) -> Result<(), ConfigError>` to initialize the global config
- Add a function `get_config() -> &'static Config` that returns a reference to the initialized config

### 2. Modify `src/main.rs`:

- Call `config::load_config()` at the start of the application
- Match on the `Result` and handle errors by logging and exiting
- On success, call `config::set_global_config()` to store the config
- Use the stored config for the rest of the application

### 3. Update Tests:

- Modify tests to expect `Result` from `load_config()`
- Add tests for error conditions that previously would have panicked
- Add tests for the new `set_global_config()` and `get_config()` functions

## Advantages of This Approach

1. **Separation of Concerns**: Loading logic is separated from storage/initialization
2. **Error Handling**: Errors are properly propagated and handled at the application boundary
3. **Testability**: `load_config()` can be tested in isolation without affecting global state
4. **API Clarity**: Clean, simple API for accessing configuration throughout the codebase
5. **Thread Safety**: Uses thread-safe `OnceLock` for static initialization

With this resolution in place, tasks T012-T015 can now proceed with a clear strategy for handling configuration errors properly.