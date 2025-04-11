# Implement Target URL Construction

## Task
Construct the target Anthropic API URL based on the configuration and the incoming request path/query. Handle parsing errors.

## Implementation Approach
1. In the `proxy_handler` function, use the `_path_and_query` variable that's already extracted but rename it without the underscore.
2. Create a new string by combining the `config.anthropic_target_url` with the extracted `path_and_query` value.
3. Attempt to parse this combined string into a `Uri` using `.parse()`.
4. Use a `match` statement to handle the result of the parsing operation:
   - If parsing succeeds, we'll have a valid target URL to use in upcoming tasks.
   - If parsing fails, log the error with appropriate context and return a `StatusCode::INTERNAL_SERVER_ERROR`.
5. Log the constructed target URL for debugging purposes.