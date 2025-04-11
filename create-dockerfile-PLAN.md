# Create Dockerfile

## Task Description
Create a multi-stage Dockerfile that efficiently builds and packages the Switchboard proxy application, optimizing for build time, container size, and security.

## Implementation Approach
1. Create a Dockerfile based on the template provided in PLAN.md section 8.3
2. Use a multi-stage build with:
   - A builder stage using rust:slim for compiling the application
   - A runtime stage using debian:12-slim with minimal dependencies
3. Implement dependency caching to optimize build time
4. Include only necessary runtime dependencies (ca-certificates)
5. Set appropriate ENTRYPOINT for the application
6. Include comments explaining key steps in the Dockerfile
7. Configure the Dockerfile to expose the default port 8080