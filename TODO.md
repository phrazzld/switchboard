```markdown
# TODO

## CI Workflow (.github/workflows/ci.yml)
- [x] **Remove `dead_code` suppression from CI lint job**
  - **Action:** Modify the `lint` job in `.github/workflows/ci.yml` to remove the `-A dead_code` flag from the `cargo clippy` command. If specific dead code is temporarily acceptable, use `#[allow(dead_code)]` attributes in the source code with justification comments, or configure exceptions via `clippy.toml`.
  - **Depends On:** None
  - **AC Ref:** `CODING_STANDARDS.md` (Std 8)

- [x] **Enforce clippy warnings as errors in CI lint job**
  - **Action:** Modify the `lint` job in `.github/workflows/ci.yml` to add the `-D warnings` flag (or deny specific lints as configured) to the `cargo clippy` command to ensure stricter checks are enforced in the CI pipeline.
  - **Depends On:** None
  - **AC Ref:** `CODING_STANDARDS.md` (Std 1, 7)

- [ ] **Investigate and fix underlying doctest failures**
  - **Action:** Identify the root cause preventing documentation tests (`doctests`) from passing when running `cargo test` (without `--tests`). Implement the necessary fixes in the code or documentation examples.
  - **Depends On:** None
  - **AC Ref:** `CODING_STANDARDS.md` (Std 12), `DOCUMENTATION_APPROACH.md` (Sec 4)

- [ ] **Enable doctest execution in CI test job**
  - **Action:** Modify the `test` job in `.github/workflows/ci.yml` by removing the `--tests` flag from the `cargo test` command, ensuring doctests are included in the standard test suite run by CI.
  - **Depends On:** Investigate and fix underlying doctest failures
  - **AC Ref:** `CODING_STANDARDS.md` (Std 12), `DOCUMENTATION_APPROACH.md` (Sec 4)

## Developer Experience (README.md, hooks/pre-commit)
- [ ] **Replace custom pre-commit hook with standard `pre-commit` framework**
  - **Action:** Implement the standard `pre-commit` framework:
    1. Add a `.pre-commit-config.yaml` file defining hooks for `cargo fmt --check` and `cargo clippy` (with strict settings, e.g., `-D warnings`).
    2. Update `README.md` prerequisites to include `pre-commit` (`pip install pre-commit`).
    3. Update `README.md` setup instructions to use `pre-commit install`.
    4. Remove the custom `hooks/pre-commit` shell script and its setup instructions from `README.md`.
  - **Depends On:** None
  - **AC Ref:** `CORE_PRINCIPLES.md` (Principle 6), `DOCUMENTATION_APPROACH.md` (Sec 2), `CODING_STANDARDS.md` (Std 8)

## Integration Tests (tests/proxy_integration_tests.rs)
- [x] **Resolve `unused_imports` suppression in integration tests**
  - **Action:** Investigate the `#[allow(unused_imports)]` for `futures_util::StreamExt` in `tests/proxy_integration_tests.rs`. Either remove the import and the `allow` attribute if it's truly unused, or add a `// TODO:` comment explaining its intended future use if it's planned.
  - **Depends On:** None
  - **AC Ref:** `CODING_STANDARDS.md` (Std 8)

- [ ] **Refactor streaming test assertions for robustness**
  - **Action:** Modify the `test_streaming_response_forward_success` test in `tests/proxy_integration_tests.rs`. Replace the brittle `body_str.contains(...)` assertions with logic that parses the Server-Sent Events (SSE) stream properly (e.g., line by line, checking `data: ` prefix, parsing JSON) and asserts on the sequence and content of the parsed events.
  - **Depends On:** None
  - **AC Ref:** `TESTING_STRATEGY.md` (Principle 1b)

## Test Utilities (tests/common/mod.rs)
- [ ] **Refactor test config setup to avoid `Box::leak`**
  - **Action:** Evaluate and potentially refactor the test setup in `tests/common/mod.rs` to avoid using `Box::leak` for the static config reference. Consider using `Arc<Config>` passed into the router/handler setup if the complexity increase is acceptable.
  - **Depends On:** None
  - **AC Ref:** `ARCHITECTURE_GUIDELINES.md` (Implied: Safety/Ownership)

## Code Formatting
- [x] **Ensure all files end with a final newline**
  - **Action:** Add a final newline character to the end of the following files: `README.md`, `hooks/pre-commit` (if not removed by framework switch), `tests/common/mod.rs`. Ensure `rustfmt` or other formatters enforce this going forward.
  - **Depends On:** None
  - **AC Ref:** General Best Practice

## [!] CLARIFICATIONS NEEDED / ASSUMPTIONS
- [ ] **Issue/Assumption:** Need details on the "separate issue" preventing doctests from passing.
  - **Context:** The comment in `.github/workflows/ci.yml:85` mentions skipping doctests (`--tests`) due to a separate issue. Understanding this issue is required for the "Investigate and fix underlying doctest failures" task.

- [ ] **Issue/Assumption:** Confirm the decision to switch from the custom pre-commit script to the standard `pre-commit` framework.
  - **Context:** The feedback strongly suggests switching (`README.md:106-112`, `hooks/pre-commit`). This task assumes the switch will be implemented. If not, the task needs modification, and a separate task to fix linting in the *existing* script would be needed.

- [ ] **Issue/Assumption:** Evaluate complexity vs. benefit of replacing `Box::leak` with `Arc<Config>` in test setup.
  - **Context:** The suggestion in `tests/common/mod.rs:61` is conditional ("consider using... if the complexity isn't significantly higher"). An evaluation is needed before proceeding with the "Refactor test config setup to avoid `Box::leak`" task.
```