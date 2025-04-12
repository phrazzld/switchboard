# TODO

## Code Review Feedback: Conditional Body Logging

- [ ] **Task Title:** Re-evaluate `MAX_LOG_BODY_LEN` Increase
    - **Action:** Review the increase of `MAX_LOG_BODY_LEN` from 10KB to 20KB in `src/proxy_handler.rs`. Justify the need for 20KB based on expected data sizes versus potential performance impact (CPU/IO). Consider making this limit configurable via `Config` or reverting to 10KB if the increase is not strictly necessary. Monitor performance after implementation if the increase is kept.
    - **Depends On:** None
    - **AC Ref:** Increased Max Log Body Size

- [x] **Task Title:** Change Body/Chunk Content Log Level to DEBUG
    - **Action:** Modify the logging logic in `src/proxy_handler.rs` (lines ~587-622, ~658-693, ~335-356). Change the `info!` macro calls that log `http.request.body.content`, `http.response.body.content`, and `chunk_content` to use the `debug!` macro, even when `config.log_bodies` is true. The `LOG_BODIES` flag should enable *inclusion* of the body in logs, but the *level* should reflect the detail (`DEBUG` is more appropriate than `INFO` for full body content).
    - **Depends On:** None
    - **AC Ref:** Body/Chunk Logging Level Too High

- [ ] **Task Title:** Refactor Logging Helpers to Accept `log_bodies` Flag Only
    - **Action:** Modify the function signatures and implementations of `log_request_details`, `log_response_details`, and `log_response_headers` in `src/proxy_handler.rs`. Instead of accepting the full `config: &Config`, pass only the necessary `log_bodies: bool` flag. Update the call sites within `proxy_handler` accordingly.
    - **Depends On:** None
    - **AC Ref:** Logging Helpers Take Full Config

- [x] **Task Title:** Remove Redundant Debug Logs for Body Logging
    - **Action:** Remove the `debug!` log messages added in `src/proxy_handler.rs` (lines ~200-204, ~471-475) that explicitly state "Full request/response details logged (verbose mode enabled)". These are redundant as the actual body logs (now at `DEBUG` level after implementing the related task) already indicate that logging occurred.
    - **Depends On:** Change Body/Chunk Content Log Level to DEBUG
    - **AC Ref:** Redundant Debug Logs for Body Logging

- [ ] **Task Title:** Update Doc Comments for Logging Helper Functions
    - **Action:** Update the Rustdoc comments (`///`) for the `log_request_details`, `log_response_details`, and `log_response_headers` functions in `src/proxy_handler.rs`. Specifically, update the `# Arguments` section to accurately reflect the parameters passed to these functions after refactoring (should now be `log_bodies: bool` instead of `config: &Config`). Ensure the description accurately reflects the function's behavior.
    - **Depends On:** Refactor Logging Helpers to Accept `log_bodies` Flag Only
    - **AC Ref:** Doc Comment Updates Needed

## [!] CLARIFICATIONS NEEDED / ASSUMPTIONS

- [ ] **Issue/Assumption:** Input document is a Code Review, not a Feature Plan.
    - **Context:** The provided input (`PLAN.md`) is a detailed code review document for the "Conditional Body Logging" feature, not a high-level plan. This `TODO.md` is generated based on the actionable feedback and suggestions outlined in the "Summary Table" and detailed feedback sections of that code review. It assumes the goal is to create tasks to address this feedback.