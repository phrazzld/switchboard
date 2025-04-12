# Update Doc Comments for Logging Helper Functions

## Implementation Approach
Update the documentation comments for the `log_request_details`, `log_response_details`, and `log_response_headers` functions in `src/proxy_handler.rs` to accurately reflect their new parameter lists. In particular, ensure the `# Arguments` sections correctly document that they now take a `log_bodies: bool` parameter instead of a `config: &Config` parameter, with clear descriptions that match the function's behavior.