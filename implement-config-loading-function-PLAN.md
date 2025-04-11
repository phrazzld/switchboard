# Implement Config Loading Function

## Implementation Approach
Implement the `load_config` function in `src/config.rs` using `std::env::var` to load configuration values from environment variables, with sensible defaults for optional values, and making `ANTHROPIC_API_KEY` mandatory. Use `dotenvy` to load variables from a `.env` file if present. Utilize `OnceLock` for thread-safe initialization and memoization of the config instance.