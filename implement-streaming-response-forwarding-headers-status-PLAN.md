# Implement Streaming Response Forwarding (Headers/Status)

## Task Title
Implement Streaming Response Forwarding (Headers/Status): Build the Axum `Response` for streaming, copying status and filtered headers (remove hop-by-hop, *do not* add `Content-Length`). Attach the streaming body.

## Implementation Approach
Replace the basic response builder currently in the streaming branch with a complete implementation that:
1. Creates a Response using the original response status code
2. Copies all relevant headers from the Anthropic API response, filtering out hop-by-hop headers (connection, proxy-*, te, trailer, transfer-encoding, upgrade) as well as content-length (which should not be set for streaming responses)
3. Attaches the streaming body created in the previous task
4. Includes proper error handling and logging