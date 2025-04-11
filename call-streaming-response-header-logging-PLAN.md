# Call Streaming Response Header Logging

## Task Title
Call Streaming Response Header Logging: Call `log_response_headers` when a streaming response is detected.

## Implementation Approach
Add the function call `log_response_headers(resp_status, &resp_headers);` in the `if is_streaming` block inside the `proxy_handler` function to log the response status and headers for streaming responses.