# Send Test Request using `tower::ServiceExt::oneshot` in Basic Test

## Implementation Approach
Inside the `test_simple_post_forward_success` function, construct an HTTP request with method POST, URI "/v1/messages", content-type header set to "application/json", and a simple JSON body. Use the tower::ServiceExt::oneshot extension method on the test_setup.app to send the request and receive the response.