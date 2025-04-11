# Call Non-Streaming Response Logging

## Task
Call `log_response_details` after successfully reading the non-streaming response body.

## Implementation Approach
1. Locate the non-streaming response handling section in the `proxy_handler` function
2. After successfully reading the response body but before returning the placeholder response, add a call to the `log_response_details` function
3. Pass the response status, headers, and body to the logging function
4. Verify that the code compiles without errors

This is a simple integration of the previously implemented logging helper function to provide detailed logging of non-streaming responses.