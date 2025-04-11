# Implement Basic Router Creation

## Task
Create the `create_router` function that sets up an Axum `Router` with a catch-all `any` route (`/*path`) pointing to `proxy_handler`.

## Implementation Approach
The current implementation already has a placeholder that creates a basic router but it doesn't connect it to the existing `proxy_handler` function. I'll update the `create_router` function to:

1. Take the `reqwest::Client` and `Config` as parameters
2. Create and return a new Axum router with a catch-all route (`/*path`)
3. The catch-all route will use the `any` method to handle all HTTP methods
4. Instead of returning a placeholder string, it will call our existing `proxy_handler` function
5. Make sure to clone the client for each request to avoid ownership issues