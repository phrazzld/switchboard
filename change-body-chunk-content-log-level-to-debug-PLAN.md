# Change Body/Chunk Content Log Level to DEBUG

## Implementation Approach
Modify the logging levels in `src/proxy_handler.rs` by changing the `info!` macro calls to `debug!` macro calls for all locations where request/response body content and chunk content are logged. This will maintain inclusion of these contents in the logs when `LOG_BODIES` is true, but at a more appropriate detail level of `DEBUG` rather than `INFO`.