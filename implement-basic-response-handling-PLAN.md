# Implement Basic Response Handling

## Task
Handle the `Result` from `send()`. On error, log and return `StatusCode::BAD_GATEWAY`. On success, extract status and headers. Log basic response info and record status code in span.

## Implementation Approach
1. Update the existing code in `src/proxy_handler.rs` to properly extract and log response status code
2. Store the status and headers from the successful response
3. Record the status code in the tracing span
4. Log response details at an appropriate level
5. Return a placeholder response until the next task (non-streaming response handling) is implemented

We've already implemented some of the error handling in the previous task, so now we need to enhance the successful response path to extract and log status code and headers.