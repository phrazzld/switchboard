# Refactor Logging Helpers to Accept `log_bodies` Flag Only

## Implementation Approach
Modify the signatures and implementations of `log_request_details`, `log_response_details`, and `log_response_headers` functions in `src/proxy_handler.rs` to accept only the necessary `log_bodies: bool` flag instead of the full `&Config` reference. Update all call sites within the `proxy_handler` function to extract and pass only the `log_bodies` value. This improves modularity by reducing unnecessary dependencies.