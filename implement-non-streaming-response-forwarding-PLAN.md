# Implement Non-Streaming Response Forwarding

## Task
Build the Axum `Response`, copying status, filtered headers (remove hop-by-hop, add `Content-Length`), and the response body.

## Implementation Approach
1. Replace the placeholder response with an actual Axum `Response` construction
2. Use `Response::builder()` to create a response with the same status code as the Anthropic API response
3. Copy headers from the Anthropic API response, filtering out hop-by-hop headers similar to the request forwarding
4. Set the `Content-Length` header based on the body size
5. Set the body using `Body::from(resp_body_bytes)`
6. Handle any potential builder errors with proper logging and fallback error status codes
7. Return `Ok(response)` instead of the placeholder `Err(StatusCode::NOT_IMPLEMENTED)`

This change will allow the proxy to properly forward non-streaming responses from the Anthropic API back to the client.