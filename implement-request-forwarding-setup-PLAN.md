# Implement Request Forwarding Setup

## Task
Build the `reqwest::Request` builder, copy method, target URL, filter/copy headers (remove hop-by-hop, set `Host`, set `x-api-key`, remove `Authorization`), and add the request body. Handle API key header errors.

## Implementation Approach
1. Remove the underscores from variable names (`_client`, `_config`, `_target_url`) to use them in the implementation
2. Create a reqwest::Request builder with the method and target URL
3. Create a new HeaderMap for the forwarded headers
4. Filter the original headers, skipping hop-by-hop headers (HOST, CONNECTION, PROXY_*, etc.)
5. Set the Host header based on the target URL
6. Set the x-api-key header from the config and remove Authorization header if present
7. Handle potential header value creation errors by returning INTERNAL_SERVER_ERROR
8. Add the headers and request body to the builder
9. Store the builder in a variable to be used in the next implementation step

This implementation prepares the request for forwarding but doesn't actually send it yet (that will be done in the next task).