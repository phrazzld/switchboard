# Integrate Config and Logging

## Implementation Approach
Add function calls to `config::load_config()` and `logger::init_tracing()` at the beginning of the main function in `src/main.rs`. The `init_tracing` function requires the config object returned by `load_config()` as an argument, so we'll call them in the correct order.