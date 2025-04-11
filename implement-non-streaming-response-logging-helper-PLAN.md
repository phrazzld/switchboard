# Implement Non-Streaming Response Logging Helper

## Task
Create `log_response_details` function to log status, headers, and body (truncated, formatted).

## Implementation Approach
1. Create a new function called `log_response_details` similar to the existing `log_request_details` function
2. The function will take status code, headers, and body as parameters
3. Create a new info span for the response details
4. Log the status code at info level
5. Mask sensitive headers and log all headers at debug level
6. For the response body:
   - Handle empty bodies
   - For small bodies (within size limit), attempt to parse as JSON for pretty printing
   - For large bodies, log only the size
7. Use the same `MAX_LOG_BODY_LEN` constant already defined to limit body logging size

This function will help maintain consistent and structured logging for API responses, following the same pattern established for request logging.