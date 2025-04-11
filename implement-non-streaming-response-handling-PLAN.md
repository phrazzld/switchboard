# Implement Non-Streaming Response Handling

## Task
Inside the `Ok` arm, if the response is *not* streaming, read the full response body (`Bytes`). Handle body reading errors.

## Implementation Approach
1. Add logic to check if the response is streaming by examining the `Content-Type` header for `text/event-stream`
2. If not streaming, read the full response body using `forward_resp.bytes().await`
3. Handle potential errors from reading the body
   - Log detailed error information
   - Return appropriate error status code
4. Store the response body for future use
5. Return a placeholder response until the next task (response logging) is implemented

The implementation will distinguish between streaming and non-streaming responses and handle the full reading of the response body for non-streaming responses.