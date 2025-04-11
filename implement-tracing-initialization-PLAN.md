# Implement Tracing Initialization

## Implementation Approach
Implement the `init_tracing` function in `src/logger.rs` using `tracing-subscriber` to set up structured logging based on the `Config` struct's settings. Configure the log filter level dynamically using the `EnvFilter` from either environment variables or the config, and set up either JSON or pretty formatting based on the `config.log_format` value.