# Add Core Dependencies

## Goal
Add the necessary core Rust dependencies to the `Cargo.toml` file to support the Switchboard application's functionality.

## Implementation Approach
I'll add the required dependencies to the `Cargo.toml` file by directly editing it according to the specifications in `PLAN.md` Section 4.2. The core dependencies include:

- `tokio`: Async runtime
- `axum`: Web framework
- `reqwest`: HTTP client
- `serde` and `serde_json`: Serialization/deserialization
- `tracing` and `tracing-subscriber`: Logging and instrumentation
- `http` and `hyper`: HTTP types and server implementation
- `bytes`: Efficient byte handling
- `futures-util`: Utilities for async programming
- `uuid`: For request ID generation

I'll use the specified versions and features, ensuring that:
1. `reqwest` uses `rustls-tls` instead of OpenSSL (by setting `default-features = false`)
2. Feature flags are properly set (e.g., tokio with "full", axum with "http2", "json", and "macros")
3. Dependencies are organized properly in the `Cargo.toml` file

## Reasoning
This straightforward approach is the most direct and reliable way to add the dependencies. Alternative approaches might include:

1. Using a script to modify the Cargo.toml file - this would be an unnecessary abstraction for a one-time task.
2. Using `cargo add` commands - while this would work, it would require multiple commands and might not correctly set all feature flags.
3. Creating a template Cargo.toml and replacing the existing one - this would risk overwriting other content in the file.

The direct editing approach is preferred because:
- It's simple and transparent
- It ensures precise control over versions and features
- It matches the standard Rust workflow
- It allows for clear commenting of dependencies for future maintenance