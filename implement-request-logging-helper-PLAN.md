# Implement Request Logging Helper

## Task
Create the `log_request_details` function to log method, URI, headers (masked), and body (truncated, formatted).

## Implementation Approach
1. Add the necessary imports: `bytes::Bytes`, `std::collections::HashMap`, `http::header`, and `serde_json::Value`.
2. Define a constant `MAX_LOG_BODY_LEN` (set to 10KB) to limit the size of request bodies that are fully logged.
3. Create the `log_request_details` function that takes method, URI, headers, and body as parameters.
4. Create a new span for the logging context and enter it.
5. Log basic request information at the info level.
6. Mask sensitive headers like "Authorization" and "x-api-key" by replacing their values with "[REDACTED]".
7. Log the headers (including masked ones) at the debug level.
8. Check the body size and truncate if necessary:
   - If small enough (less than MAX_LOG_BODY_LEN), attempt to parse as JSON
   - If JSON parsing succeeds, log the pretty-printed JSON
   - If JSON parsing fails, log the raw body as UTF-8
   - If body is too large, just log its size
   - If body is empty, log that it's empty