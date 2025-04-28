```markdown
# Remediation Plan – Sprint <n>

## Executive Summary
This plan addresses critical and significant findings from the recent code review. The priority is to restore essential quality gates (local pre-commit checks, CI commit linting), stabilize the automation framework by fixing fragile configurations and pinning dependencies, and then address code quality issues (dead code, unclear tests) and documentation gaps. This order ensures foundational stability before tackling deeper code and documentation improvements.

## Strike List
| Seq | CR-ID | Title                                                 | Effort | Owner?    |
|-----|-------|-------------------------------------------------------|--------|-----------|
| 1   | cr-03 | Fix Fragile Local Pre-commit Hooks Definition         | xs     | Dev Team  |
| 2   | cr-09 | Pin Unpinned Commitlint Dependency                    | xs     | Dev Team  |
| 3   | cr-04 | Fix Missing Commit Message Enforcement in CI          | xs     | Dev Team  |
| 4   | cr-01 | Reinstate Lost Pre-commit File Length & Test Checks   | s      | Dev Team  |
| 5   | cr-06 | Remove Redundant and Misleading Pre-commit CI Jobs    | xs     | Dev Team  |
| 6   | cr-07 | Remove Committed Temporary/Example Files              | xs     | Dev Team  |
| 7   | cr-02 | Remove Undocumented Suppression of Dead Code Warnings | m      | Dev Team  |
| 8   | cr-08 | Clarify Unclear Test Variable Usage                   | s      | Dev Team  |
| 9   | cr-05 | Restore Essential Planning/Rationale Documentation    | m      | Dev Team  |

## Detailed Remedies

### cr-03 Fix Fragile Local Pre-commit Hooks Definition
- **Problem:** `rustfmt` and `clippy` hooks use `repo: local` and `language: system`, causing non-reproducible behavior based on developer environments.
- **Impact:** Violates Dependency Management ("Pin and audit") and Simplicity (Reproducibility). Leads to inconsistent checks and bypasses pre-commit's tool version management.
- **Chosen Fix:** Replace `repo: local` definitions with standard, version-pinned pre-commit hook repositories (Option 1).
- **Steps:**
  1. Edit `.pre-commit-config.yaml`.
  2. Replace the `repo: local` entries for `rustfmt` and `clippy` with entries using standard repositories like `pre-commit/mirrors-rustfmt` and `doublify/pre-commit-clippy` (or equivalents).
  3. Pin each repository to a specific, audited `rev:` (tag or commit hash).
  4. Run `pre-commit install --install-hooks` and `pre-commit run -a` locally to verify.
- **Done-When:** `.pre-commit-config.yaml` uses standard remote repos with pinned revisions for rustfmt/clippy, and local hooks run consistently.

### cr-09 Pin Unpinned Commitlint Dependency
- **Problem:** The `commitlint` hook's `additional_dependencies` entry for `@commitlint/config-conventional` lacks a specific version pin.
- **Impact:** Violates Dependency Management. Can lead to unexpected CI failures or behavior changes if the dependency updates automatically.
- **Chosen Fix:** Pin the dependency to a specific version.
- **Steps:**
  1. Edit `.pre-commit-config.yaml`.
  2. Modify the `additional_dependencies` line for the `commitlint` hook.
  3. Change `['@commitlint/config-conventional']` to `['@commitlint/config-conventional@<specific_version>']` (e.g., `'@commitlint/config-conventional@17.4.4'` or latest verified version).
  4. Run `pre-commit run commitlint --hook-stage commit-msg -m "feat: test pinning"` (or similar) locally to verify.
- **Done-When:** `@commitlint/config-conventional` dependency is pinned to a specific version in `.pre-commit-config.yaml`.

### cr-04 Fix Missing Commit Message Enforcement in CI
- **Problem:** CI job runs `pre-commit run --all-files`, which skips the `commit-msg` stage, failing to enforce Conventional Commits via `commitlint`.
- **Impact:** Violates Automation Quality Gates and Conventional Commits standard. Allows invalid commit messages into history, breaking SemVer/changelog automation.
- **Chosen Fix:** Add an explicit step in CI to run the `commit-msg` stage hooks (Option 1).
- **Steps:**
  1. Edit `.github/workflows/ci.yml`.
  2. Within one of the primary jobs (e.g., `lint-linux` or a dedicated step/job), add a step *after* checkout.
  3. Use `pre-commit run --hook-stage commit-msg --from HEAD~${{ github.event.pull_request.commits }} --to HEAD` for PRs, or `pre-commit run --hook-stage commit-msg --from HEAD~1 --to HEAD` for push events on master. Alternatively, use `npx commitlint --from HEAD~... --to HEAD`. Adjust the commit range as needed for the trigger event.
  4. Verify the step runs and fails on invalid commit messages in a test PR.
- **Done-When:** CI pipeline fails if commits in a PR do not adhere to the Conventional Commits standard.

### cr-01 Reinstate Lost Pre-commit File Length & Test Checks
- **Problem:** Replacing the old shell hook with `.pre-commit-config.yaml` dropped local checks for file length limits and `cargo test`.
- **Impact:** Violates Automation Quality Gates and Design for Testability (Local Gating). Removes vital fast feedback loops for developers.
- **Chosen Fix:** Re-implement these checks within `.pre-commit-config.yaml` (Option 1).
- **Steps:**
  1. Edit `.pre-commit-config.yaml`.
  2. Add a hook for file size checks. Consider `pre-commit-hooks` `check-added-large-files` or `check-yaml` (if applicable), or potentially a small custom `repo: local` script hook (`language: script`) checking line counts (`wc -l`) against WARN/ERROR thresholds (500/1000 lines).
  3. Add a `repo: local` hook with `id: cargo-test`, `name: cargo test`, `entry: cargo test --no-fail-fast`, `language: system`, `stages: [commit]`, `pass_filenames: false`.
  4. *If* full `cargo test` is too slow locally:
     *   Configure it to run a faster subset (e.g., `cargo test --lib` or specific unit tests).
     *   Alternatively, make the hook optional (`stages: [manual]`) but document it as strongly recommended (`CONTRIBUTING.md`).
  5. Document the exact behavior (what tests run, how long, thresholds) in `CONTRIBUTING.md`.
- **Done-When:** Local `pre-commit` run checks file length and executes `cargo test` (or a documented subset).

### cr-06 Remove Redundant and Misleading Pre-commit CI Jobs
- **Problem:** CI runs `pre-commit run --all-files` jobs (`pre-commit-linux`, `pre-commit-macos`) which are redundant as `fmt` and `clippy` are checked by dedicated `cargo` jobs. The `cross-platform-verification` job name is misleading.
- **Impact:** Violates Simplicity First and Automation Quality Gates. Wastes CI resources and build time, adds unnecessary complexity.
- **Chosen Fix:** Remove redundant jobs and rename the summary job.
- **Steps:**
  1. Edit `.github/workflows/ci.yml`.
  2. **Delete** the `pre-commit-linux` and `pre-commit-macos` jobs entirely.
  3. Update the `needs` section of the `cross-platform-verification` job to remove dependencies on the deleted jobs.
  4. Rename the `cross-platform-verification` job to something accurate, like `Linux_macOS_Verification_Summary`.
- **Done-When:** Redundant pre-commit CI jobs are removed, `needs` section is updated, and summary job is renamed.

### cr-07 Remove Committed Temporary/Example Files
- **Problem:** `test-file.txt` and `valid-commit-msg.txt` are committed to the repository.
- **Impact:** Violates Simplicity First, Keep It Lean. Adds clutter unrelated to source code.
- **Chosen Fix:** Delete the files and embed examples in documentation if needed.
- **Steps:**
  1. Delete `test-file.txt` locally.
  2. Delete `valid-commit-msg.txt` locally.
  3. If the content of `valid-commit-msg.txt` is a useful example, copy it into `README.md` or `CONTRIBUTING.md` within a relevant section (e.g., Conventional Commits).
  4. Commit the deletions.
- **Done-When:** `test-file.txt` and `valid-commit-msg.txt` are removed from the repository.

### cr-02 Remove Undocumented Suppression of Dead Code Warnings
- **Problem:** Numerous `#[allow(dead_code)]` attributes exist in `tests/common/mod.rs` without justification.
- **Impact:** Violates Coding Standards ("Address Violations, Don't Suppress"), Simplicity First. Hides potential bugs, unused code, and increases maintenance cost.
- **Chosen Fix:** Remove all allowances and investigate each resulting compiler warning (Option 1).
- **Steps:**
  1. Open `tests/common/mod.rs`.
  2. Delete *every* instance of `#[allow(dead_code)]`.
  3. Run `cargo check --tests` or `cargo clippy --tests`.
  4. For each reported `dead_code` warning:
     *   Investigate if the function/struct/field is truly unused by any test. Use IDE tools (find references) or text search across the `tests/` directory.
     *   If confirmed unused: **DELETE** the code.
     *   If it *is* used (e.g., by a test in another file calling `common::some_helper()`), the compiler warning should disappear after removing the `allow`.
     *   If the compiler *still* warns incorrectly (rare, possibly complex macro/conditional compilation interaction), add a specific `// ALLOWANCE: Used by integration test setup in [specific test file/scenario], compiler fails to detect usage.` comment *directly above* the item, and only for that single item. This requires strong justification.
  5. Ensure `cargo clippy --tests -- -D warnings` passes cleanly regarding dead code in this module.
- **Done-When:** All `#[allow(dead_code)]` attributes are removed from `tests/common/mod.rs`, and all resulting warnings are resolved by either deleting unused code or verifying actual usage (eliminating the warning naturally).

### cr-08 Clarify Unclear Test Variable Usage
- **Problem:** Unused variables in tests are silenced using `#[allow(unused_variables)]` or `_`-prefixing, obscuring intent.
- **Impact:** Violates Coding Standards, Design for Testability. Makes tests harder to understand and verify correctness.
- **Chosen Fix:** Investigate each instance and remove the suppression/prefix or use the variable.
- **Steps:**
  1. Go to `tests/common_utils_test.rs:96`.
     *   Remove `#[allow(unused_variables)]` from `temp_dir_path`.
     *   Determine why `temp_dir_path` is not used in `test_directory_permissions_non_unix`. If it's truly unnecessary for the test logic, remove the variable binding `let temp_dir_path = ...`. If it *should* be used (e.g., passed to a function), update the test logic.
  2. Go to `tests/environment_log_paths_test.rs:135`.
     *   Investigate why `_config` is created but not used in `test_development_environment_paths`.
     *   If the `Config` instance creation itself has side effects needed for the test (unlikely) or is just setup for later steps, add a comment explaining why it's needed but not directly referenced.
     *   If it serves no purpose in this specific test's logic, remove the `let _config = ...` line.
     *   If it *should* be used (e.g., assertions based on config values), rename to `config` and add the relevant assertions/logic.
- **Done-When:** Allowances/prefixes for unused variables are removed, and variables are either used correctly or deleted from the tests.

### cr-05 Restore Essential Planning/Rationale Documentation
- **Problem:** Critical historical/decision documents (`PLAN.md`, `TODO.md`, `CI_FAILURES.md`, `CI_STATUS.md`) were deleted.
- **Impact:** Violates Documentation Approach ("Document Decisions, Not Mechanics"). Loss of project context hinders maintainability and onboarding.
- **Chosen Fix:** Resurrect key decisions and rationale from history and consolidate them into living documentation (Option 1).
- **Steps:**
  1. Use `git log --diff-filter=D --summary` or history browsing tools to find the commits where `PLAN.md`, `TODO.md`, `CI_FAILURES.md`, `CI_STATUS.md` were deleted.
  2. Retrieve the content of these files from commits prior to deletion (`git show <commit_hash>:<path>`).
  3. Review the content, focusing on:
     *   **Decisions:** Significant choices made (architecture, libraries, algorithms).
     *   **Rationale:** The "why" behind those decisions, trade-offs considered.
     *   **Learnings:** Important lessons from past failures (especially CI issues) or refactorings.
     *   **Core Plans:** Overarching goals or architectural direction from `PLAN.md`.
  4. Create a consolidated `DECISIONS.md` file in the repository root containing relevant extracted rationale, organized by topic/feature area.
  5. Update `README.md` to reference the new `DECISIONS.md` file and explain its purpose.
- **Done-When:** A permanent record of key decisions and rationale is preserved in a well-organized document, including Windows support removal rationale, CI troubleshooting lessons, and architecture decisions.

## Standards Alignment
- **Simplicity First**: Each fix is intentionally minimal and focused on resolving specific issues without over-engineering. Priority given to quality gates that prevent complexity (file size limits) from sneaking in.
- **Modularity**: Fixes maintain clear separation of concerns, especially with properly configured pre-commit hooks and documentation.
- **Testability**: Restoring local test gates is prioritized to ensure developers get immediate feedback on test failures.
- **Coding Standards**: Multiple fixes directly address coding standards violations (dead code suppression, unused variables).
- **Documentation**: Documentation improvements capture rationale ("why") over mechanics, preserving critical decision context.

## Validation Checklist
- All pre-commit hooks run successfully locally.
- CI pipeline passes on Linux and macOS.
- Conventional Commits validation works in CI.
- Tests pass with warning levels set to deny warnings (`-D warnings`).
- No warning suppressions exist without specific justification comments.
- Core decisions and rationale are preserved in structured documentation.
- All temporary files are removed from the repository.