# Refactor test config setup to avoid `Box::leak`

## Implementation Approach
Replace `Box::leak` in test setup with `Arc<Config>` for sharing the configuration. This avoids the memory leak while providing the necessary 'static lifetime through atomic reference counting. The refactoring will involve:

1. Modifying the `create_router` function in `src/proxy_handler.rs` to accept `Arc<Config>` instead of `&'static Config`
2. Updating `main.rs` to use `Arc<Config>` when calling `create_router`
3. Updating `tests/common/mod.rs` to create an `Arc<Config>` instead of using `Box::leak`

This approach aligns with better ownership practices in Rust while maintaining the needed shared access to configuration.