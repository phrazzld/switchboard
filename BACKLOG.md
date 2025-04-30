# BACKLOG

---

## High Priority

- integrate `ward` pre-commit hook
  - published at `phrazzld/ward`

### Core Functionality & Provider Integration

- **[Feature]**: Implement OpenAI API Integration Adapter
  - **Complexity**: Complex
  - **Rationale**: Expands core functionality to support a major alternative LLM provider, offering users flexibility, redundancy, and potential cost/performance benefits. Critical for market competitiveness.
  - **Expected Outcome**: The proxy can be configured to route requests to specified OpenAI models. Includes adapter logic for request/response mapping and handling OpenAI-specific errors.
  - **Dependencies**: Centralized Configuration Management.

- **[Feature]**: Implement Basic Model Selection & Routing Logic
  - **Complexity**: Medium
  - **Rationale**: Enables the core value proposition of the proxy: directing traffic between configured providers based on defined rules.
  - **Expected Outcome**: Configuration allows defining rules (e.g., default provider, potential header-based routing) to select the target LLM provider (Anthropic/OpenAI). Requests are successfully routed.
  - **Dependencies**: OpenAI API Integration (to have multiple providers).

### Logging & Observability

- **[Refactor]**: Centralize and Standardize Logging Configuration
  - **Complexity**: Medium
  - **Rationale**: Removes hardcoded values, ensures consistency, simplifies maintenance, and enables easier tuning of logging behavior across environments. Foundation for other logging improvements.
  - **Expected Outcome**: All logging parameters (level, file path, format, rotation settings) are driven by a single configuration source (e.g., file or environment variables). Defaults are consistent and documented.
  - **Dependencies**: None.

- **[Feature]**: Implement Robust Log Rotation (Size, Count, Age)
  - **Complexity**: Medium
  - **Rationale**: Prevents log files from consuming excessive disk space, a critical operational requirement for stability and manageability. Supports basic retention policies.
  - **Expected Outcome**: Log files automatically rotate based on configurable size limits. A configurable number of rotated files are kept (count-based retention). Older logs are purged based on a configurable age limit.
  - **Dependencies**: Centralized Logging Configuration.

- **[Feature]**: Implement Comprehensive Request/Response Logging Middleware
  - **Complexity**: Complex
  - **Rationale**: Provides essential end-to-end visibility for debugging, auditing, performance analysis, and understanding provider interactions. Crucial for operational support.
  - **Expected Outcome**: Middleware captures and logs complete request and assembled response bodies (handling streaming). Logs include correlation IDs, timestamps, latency metrics, status codes, and provider information in a structured or easily parsable format.
  - **Dependencies**: Centralized Logging Configuration.

### Technical Excellence & Performance

- **[Refactor]**: Optimize Configuration Handling in Request Path
  - **Complexity**: Medium
  - **Rationale**: Addresses performance bottleneck caused by repeated configuration parsing/cloning during request handling. Essential for scalability and responsiveness under load.
  - **Expected Outcome**: Configuration values are parsed once at startup and accessed efficiently (e.g., via shared state like `Arc<Config>`) within the request lifecycle. Benchmarks demonstrate reduced latency/resource use per request.
  - **Dependencies**: None.

### Testing & Quality Assurance

- **[Fix]**: Stabilize Logging Tests (`tests/logger_file_test.rs`)
  - **Complexity**: Medium
  - **Rationale**: Unreliable tests (due to race conditions, shared file access) impede development velocity and reduce confidence in code changes. Fixing this is critical for maintainability.
  - **Expected Outcome**: Logging tests run reliably, utilize unique temporary resources (files/directories) per test instance, implement proper synchronization for shared resources, and can be executed in parallel without interference.
  - **Dependencies**: None.

---

## Medium Priority

### Logging & Observability

- **[Feature]**: Implement Log Directory Health Monitoring
  - **Complexity**: Simple
  - **Rationale**: Improves operational stability by proactively detecting potential logging failures (permissions, disk space) at startup or runtime.
  - **Expected Outcome**: Application performs checks at startup for log directory existence, write permissions, and sufficient disk space. Optionally, periodic checks or status reporting on directory usage (size, file count) are implemented. Errors trigger warnings or prevent startup.
  - **Dependencies**: Centralized Logging Configuration.

- **[Feature]**: Implement Graceful Shutdown Handling for Logging
  - **Complexity**: Simple
  - **Rationale**: Ensures all buffered log messages are written to disk during application shutdown (e.g., on SIGTERM/SIGINT), preventing data loss.
  - **Expected Outcome**: A shutdown hook reliably flushes all logging buffers before the application exits cleanly.
  - **Dependencies**: Logging Infrastructure.

- **[Enhancement]**: Implement Graceful Recovery from Log File Access Failures
  - **Complexity**: Medium
  - **Rationale**: Increases application resilience by allowing the proxy to continue operating (potentially with degraded logging) if log file access fails temporarily (e.g., disk full, permissions change).
  - **Expected Outcome**: The application detects log write errors, attempts recovery (e.g., reopening file), logs errors to stderr or an alternative sink, and avoids crashing due to logging failures.
  - **Dependencies**: Logging Infrastructure.

- **[Feature]**: Add Basic Service Metrics Exposure (Prometheus)
  - **Complexity**: Medium
  - **Rationale**: Enables standardized monitoring and alerting on application performance, resource usage, and business metrics using common observability tools.
  - **Expected Outcome**: An HTTP endpoint (e.g., `/metrics`) exposes key application metrics (request counts, latency percentiles, error rates per provider, active connections) in Prometheus format.
  - **Dependencies**: None.

- **[Feature]**: Implement Health Check Endpoint
  - **Complexity**: Simple
  - **Rationale**: Provides a standard mechanism for load balancers, orchestration systems (Kubernetes), and monitoring tools to verify service availability.
  - **Expected Outcome**: An HTTP endpoint (e.g., `/healthz` or `/livez`) returns a 200 OK status if the core service components are operational.
  - **Dependencies**: None.

### Technical Excellence & Performance

- **[Enhancement]**: Reduce Resource Usage via Optimized Cloning/Ownership
  - **Complexity**: Medium
  - **Rationale**: Improves efficiency, scalability, and reduces operational costs by minimizing unnecessary memory allocations and CPU cycles in performance-sensitive code paths.
  - **Expected Outcome**: Code analysis (e.g., profiling, review of `main.rs`, `proxy_handler.rs`) identifies and refactors areas using excessive cloning, favoring references (`&T`, `Arc<T>`) where appropriate. Measurable reduction in memory footprint or CPU usage under load.
  - **Dependencies**: Optimize Configuration Handling (as a primary candidate).

### Developer Experience & Tooling

- **[Refactor]**: Migrate Pre-commit Hooks to Standard Framework
  - **Complexity**: Medium
  - **Rationale**: Improves code quality, consistency across developer environments, simplifies hook management, and leverages community best practices using tools like `pre-commit`.
  - **Expected Outcome**: Existing and new pre-commit checks (formatting, linting, commit messages) are managed via a `pre-commit-config.yaml`. Setup is documented. Hooks run automatically on commit.
  - **Dependencies**: None.

- **[Enhancement]**: Add Automatic Formatting & Linting Checks (Pre-commit)
  - **Complexity**: Simple
  - **Rationale**: Enforces consistent code style (`cargo fmt`) and identifies potential issues (`cargo clippy`) automatically, improving readability and reducing review friction.
  - **Expected Outcome**: `cargo fmt --check` and `cargo clippy` (with relevant options) run as part of the pre-commit hooks, failing commits that don't meet standards.
  - **Dependencies**: Migrate Pre-commit Hooks.

- **[Enhancement]**: Add Commit Message Validation (Pre-commit)
  - **Complexity**: Simple
  - **Rationale**: Enforces a consistent commit message format (e.g., Conventional Commits), improving changelog generation, history readability, and traceability.
  - **Expected Outcome**: A pre-commit hook validates commit messages against a defined standard before allowing the commit.
  - **Dependencies**: Migrate Pre-commit Hooks.

---

## Low Priority

### Logging & Observability

- **[Enhancement]**: Enhance Log Content Searchability (Structured Logging)
  - **Complexity**: Medium
  - **Rationale**: Facilitates easier querying, filtering, and analysis of logs, especially when integrated with log aggregation platforms (e.g., ELK, Splunk). Improves debugging efficiency.
  - **Expected Outcome**: Introduce a structured logging format (e.g., JSON) for key events, particularly request/response transaction logs. Provide configuration to toggle structured logging.
  - **Dependencies**: Comprehensive Request/Response Logging.

- **[Enhancement]**: Add Log Compression for Rotated Files
  - **Complexity**: Simple
  - **Rationale**: Reduces disk space consumption for archived logs, lowering storage costs and potentially speeding up transfers.
  - **Expected Outcome**: Rotated log files are automatically compressed (e.g., using gzip) based on a configuration flag.
  - **Dependencies**: Robust Log Rotation.

- **[Tooling]**: Develop Utility for Reconstructing Logged Conversations
  - **Complexity**: Medium
  - **Rationale**: Provides a developer tool to easily piece together streamed request/response chunks from logs into a readable format for debugging complex interactions.
  - **Expected Outcome**: A simple script or command-line tool can parse log files and output reconstructed conversations based on correlation IDs.
  - **Dependencies**: Comprehensive Request/Response Logging.

### Core Functionality & Provider Integration

- **[Feature]**: Implement Provider Fallback Mechanism
  - **Complexity**: Medium
  - **Rationale**: Increases service reliability by automatically retrying requests with a secondary provider if the primary one fails or times out.
  - **Expected Outcome**: Configuration allows defining a fallback provider sequence. If a request to the primary provider fails based on configurable conditions (e.g., specific errors, timeout), the proxy automatically retries with the next provider in the sequence.
  - **Dependencies**: Basic Model Routing Rules.

- **[Feature]**: Add Monitoring/Tracking for LLM Provider Usage
  - **Complexity**: Medium
  - **Rationale**: Provides visibility into costs, token consumption, and usage patterns across different LLM providers, enabling informed decision-making and cost optimization.
  - **Expected Outcome**: Metrics (e.g., token counts, request counts per provider/model) are tracked and exposed via the metrics endpoint or detailed logs.
  - **Dependencies**: Basic Service Metrics Exposure OR Enhanced Log Searchability.

### Developer Experience & Tooling

- **[Enhancement]**: Add Support for Skipping Specific Pre-commit Hooks
  - **Complexity**: Simple
  - **Rationale**: Provides necessary flexibility for developers to bypass specific checks in exceptional circumstances (e.g., work-in-progress, specific known issues). Standard frameworks usually support this.
  - **Expected Outcome**: Documentation clarifies how to skip specific hooks using the chosen framework's standard mechanism (e.g., `SKIP=hook-id git commit ...`).
  - **Dependencies**: Migrate Pre-commit Hooks.

- **[Research]**: Investigate Async `glance ./` Post-commit Hook Integration
  - **Complexity**: Simple (Research)
  - **Rationale**: Explores automatically updating codebase summaries (`glance.md` files) after commits to keep architectural documentation current, potentially improving developer onboarding and understanding.
  - **Expected Outcome**: A brief investigation determines the feasibility, performance impact, and value of running `glance` automatically post-commit. Decision documented; implementation if deemed low-impact and high-value.
  - **Dependencies**: Migrate Pre-commit Hooks.

---

## Future Considerations

- **[Feature]**: Support Additional LLM Providers (e.g., Google Gemini, Cohere, Mistral)
  - **Rationale**: Further enhance flexibility and value by broadening the ecosystem of supported models.
- **[Refactor]**: Formalize Provider Adapter Pattern/Trait
  - **Rationale**: Create a more robust, extensible, and testable interface for adding future LLM providers with less boilerplate code.
- **[Feature]**: Implement Request/Response Caching Layer
  - **Rationale**: Reduce latency and provider costs for repeated identical requests via configurable in-memory or external caching.
- **[Feature]**: Implement Request Rate Limiting & Quota Management
  - **Rationale**: Protect backend services, manage costs, and enforce usage policies per client/API key.
- **[Feature]**: Implement Client Authentication/Authorization (API Keys)
  - **Rationale**: Secure the proxy endpoint, restricting access to authorized consumers.
- **[Operational]**: Containerization Support (Optimized Dockerfile)
  - **Rationale**: Simplify deployment, ensure consistent runtime environments, and follow cloud-native best practices.
- **[Technical Excellence]**: Introduce Stricter Static Analysis (Clippy Lints)
  - **Rationale**: Proactively catch potential bugs, performance issues, and enforce idiomatic Rust code beyond default levels.
- **[Innovation]**: Research Advanced Routing Strategies (Cost-Aware, Latency-Aware, Load-Based)
  - **Rationale**: Enable smarter, dynamic routing decisions based on real-time conditions or predefined optimization goals.
- **[Innovation]**: Research Vector Databases for Enhanced Context Management or Log Analysis
  - **Rationale**: Explore advanced techniques for managing conversation history, providing context to LLMs, or enabling semantic search over logs.
- **[Security]**: Conduct Security Audit & Implement PII Redaction in Logs
  - **Rationale**: Address potential security vulnerabilities and enhance data privacy/compliance, especially when logging request/response bodies.
