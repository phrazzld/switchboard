# BACKLOG

This document outlines future work for the Switchboard project, organized into focused, manageable epics.

## Logging Improvements

### Log Organization & Infrastructure
*Goal: Create a robust, configurable logging file structure with proper organization standards*

- [~] Restructure log file organization with proper directory structure
  - Use dedicated directories (`./logs/` by default)
  - Separate application logs from test logs into different subdirectories
  - Respect XDG Base Directory spec for user-level logs
  - Use system standards (/var/log) for service deployment
- [ ] Add log directory health monitoring
  - Implement startup checks for permissions and disk space
  - Add log file status reporting (disk usage, file counts)
- [ ] Centralize logging configuration
  - Extract hardcoded values from `src/logger.rs` and `src/config.rs` to constants
  - Ensure consistent default values across the codebase
  - Add documentation explaining the reasoning behind default values

### Log Rotation & Lifecycle Management
*Goal: Implement comprehensive log rotation with flexible retention policies*

- [ ] Implement robust log rotation with configurable parameters
  - Add size-based rotation (e.g., rotate at 10MB)
  - Implement count-based retention (keep last N files)
  - Support age-based cleanup (remove logs older than X days)
- [ ] Add advanced log file features
  - Implement log compression for rotated files
  - Add proper shutdown handler to ensure final logs are flushed
  - Implement graceful recovery from log file access failures

### Request/Response Logging Enhancements
*Goal: Improve observability by capturing complete API interactions in a searchable format*

- [ ] Add comprehensive request/response logging
  - Implement middleware to collect streamed response chunks
  - Log complete, assembled response bodies in a single entry
  - Include request/response correlation and performance metrics
  - Add clear markers for completed transactions
- [ ] Enhance log content searchability
  - Create a dedicated "transaction log" format for API calls
  - Develop tools to extract and reconstruct full conversations
  - Provide utilities to convert chunked responses into readable format
  - Expose configuration to adjust detail level of streaming logs

## API Features & Integrations

### OpenAI Adapter Implementation
*Goal: Add support for OpenAI models as alternatives to Anthropic models*

- [ ] Implement OpenAI API integration
  - Add configuration options to route requests to OpenAI instead of Anthropic
  - Create adapter layer to map between API formats
  - Add response transformation for Claude Code compatibility
  - Implement proper error handling for OpenAI-specific responses
- [ ] Add model selection features
  - Support configuration-based routing rules
  - Implement fallback mechanisms between providers
  - Add monitoring for usage across different providers

## Developer Experience

### Development Workflow Improvements
*Goal: Enhance developer experience with modern tooling and automation*

- [ ] Refactor pre-commit hooks
  - Migrate to proper pre-commit framework
  - Add automatic formatting checks
  - Implement commit message validation
  - Support skipping specific hooks when needed
- [ ] Improve test infrastructure
  - Fix race conditions in `tests/logger_file_test.rs` related to file access
  - Implement unique temporary file generation for each test
  - Add proper synchronization mechanisms for shared test resources
  - Ensure tests can run in parallel without interference

## Performance Optimizations

### Config & Performance Refinements
*Goal: Improve performance in core request handling paths*

- [ ] Optimize configuration handling
  - Analyze request handling flow in `src/proxy_handler.rs` to identify repeated config parsing
  - Refactor to parse configuration values once during service initialization
  - Cache parsed values to avoid repeated conversions
  - Add benchmarks to quantify performance improvements
- [ ] Reduce resource usage
  - Review config usage in `src/main.rs` lines 46-47
  - Replace cloning with references where ownership isn't required
  - Identify and optimize other performance-critical paths
  - Document performance-sensitive areas where cloning should be avoided
