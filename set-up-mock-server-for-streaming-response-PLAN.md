# Set Up Mock Server for Streaming Response

## Implementation Approach
Configure the WireMock server in the streaming test to respond with a proper streaming response by setting the Content-Type header to "text/event-stream" and configuring a chunked response body with multiple data chunks that simulate an Anthropic API streaming response. Use WireMock's response builder to define this behavior.