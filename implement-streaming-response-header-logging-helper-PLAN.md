# Implement Streaming Response Header Logging Helper

## Task Title
Implement Streaming Response Header Logging Helper: Create log_response_headers function to log status and headers only.

## Implementation Approach
Create a new function `log_response_headers` similar to the existing `log_response_details` function, but focused only on logging status code and headers (without body logging). Additionally, include an info message indicating streaming has started to provide clarity in the logs.