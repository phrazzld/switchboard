# Define API Structs

## Implementation Approach
Create a minimal `AnthropicMessagesRequestMinimal` struct in the proxy_handler.rs file using serde::Deserialize for parsing the Anthropic API request JSON. The struct will contain only the essential fields needed for logging context (model and stream flag) following the structure defined in PLAN.md Section 5. This struct will be used for logging purposes to provide context about the requests going through the proxy.