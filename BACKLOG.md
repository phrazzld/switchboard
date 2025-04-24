# BACKLOG

## High Priority Features

- write comprehensive structured logs to an output file, but write high signal-to-noise ratio logs to stdout
   - i want to see what claude code is doing
   - i want to see the request and response content and useful metadata
   - if possible, i want to see other details about what claude code is doing, what it expects back in responses, etc
- add adapter for openai
   - i want to be able to configure switchboard to use openai models to fulfill requests that would have otherwise been sent to anthropic
   - note: when the responses come back, they might need to be reformatted so as to "trick" claude code into thinking they came from anthropic -- this is because claude code generally works better when dealing with anthropic responses

## Infrastructure Improvements

- refactor pre-commit hooks to use proper pre-commit framework

## Code Quality & Optimizations

- centralize hardcoded logging defaults
   - extract common default values from `src/logger.rs` and `src/config.rs` to constants or config struct static methods
   - ensure consistent values are used throughout the codebase
   - add comments explaining the reasoning behind each default value

- optimize config parsing for performance
   - analyze the request handling flow in `src/proxy_handler.rs` to identify repeated config parsing
   - refactor to parse configuration values once during service initialization
   - cache parsed values to avoid repeated conversions
   - add benchmarks to quantify performance improvements

- improve test file handling
   - identify potential race conditions in `tests/logger_file_test.rs` related to file access
   - implement unique temporary file generation for each test
   - or add proper synchronization mechanisms (mutex) for shared file access
   - ensure tests can run in parallel without interference

- reduce unnecessary cloning
   - review config usage in `src/main.rs` lines 46-47
   - use references instead of cloning when ownership isn't required
   - evaluate other locations where cloning might be optimized
   - document performance-critical paths where cloning should be avoided
