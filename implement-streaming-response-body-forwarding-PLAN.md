# Implement Streaming Response Body Forwarding

## Task Title
Implement Streaming Response Body Forwarding: Map the `reqwest` `bytes_stream` to an Axum `Body` stream, handling potential errors during chunk reading.

## Implementation Approach
Convert the reqwest Response's bytes_stream into an Axum compatible Body stream by:
1. Obtaining the stream using `forward_resp.bytes_stream()`
2. Mapping each Result to handle errors appropriately:
   - On success, pass the bytes through
   - On error, log the error with request context and convert to axum::BoxError
3. Creating an Axum Body from the mapped stream using `Body::from_stream()`